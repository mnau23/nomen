use crate::helpers::{add_padded_numbering, print_header};
use crate::models::{DateString, FileType, RunMode};
use anyhow::Context;
use exif::{In, Reader, Tag};
use std::ffi::OsStr;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DetectedKind {
	Other,
	Photo,
	Video,
}

pub trait Renamer {
	/// Detects whether a path points to a photo or a video.
	///
	/// Detection is implemented as:
	/// - magic-bytes detection via `infer` when the file exists and is readable
	/// - fallback to extension-based detection otherwise
	fn detect_kind(&self) -> DetectedKind;

	/// Deterministic, zero-I/O check that classifies this path as a photo based solely on its file extension.
	fn extension_is_photo(&self) -> bool;

	/// Deterministic, zero-I/O check that classifies this path as a video based solely on its file extension.
	fn extension_is_video(&self) -> bool;

	/// Generates a new filename based on its metadata.
	///
	/// # Returns
	/// A formatted filename string, e.g. `YYYY-MM-DD_Photo001.jpg`.
	///
	/// # Errors
	/// Returns an error if the file's date cannot be retrieved.
	fn generate_new_name(&self, file_type: FileType, num: usize) -> anyhow::Result<String>;

	/// Retrieves the original date when an item was taken from its EXIF metadata.
	///
	/// # Returns
	///
	/// - `Ok(String)` containing the date in `YYYY-MM-DD` format if the date is found.
	/// - `Ok(String)` containing `"NA"` (Not Available) if the EXIF data is unreadable or the tag is absent.
	/// - `Err` if there is a problem.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - the file cannot be opened (e.g., the file does not exist or access is denied).
	/// - there is an issue parsing the date from the EXIF metadata (e.g., if the date format is invalid).
	fn get_date_from_photo(&self) -> anyhow::Result<String>;

	/// Retrieves the creation date from its metadata.
	///
	/// # Returns
	///
	/// - `Ok(String)` containing the date in `YYYY-MM-DD` format if the date is found.
	/// - `Ok(String)` containing `"NA"` (Not Available) if the tag is absent.
	/// - `Err` if there is a problem.
	///
	/// # Errors
	///
	/// Returns an `anyhow::Error` if there is an issue accessing or reading the video file.
	fn get_date_from_video(&self) -> anyhow::Result<String>;

	fn is_hidden(&self) -> bool;

	fn is_photo(&self) -> bool;

	fn is_video(&self) -> bool;

	/// Processes and renames files in a directory based on the specified file type.
	///
	/// # Arguments
	///
	/// - `file_type`: the type of file to be processed.
	/// - `mode`: whether to only preview (`DryRun`) or apply changes (`Apply`).
	///
	/// # Returns
	///
	/// - `Ok(())` if all files are processed successfully.
	/// - `Err` if any error occur during the renaming or file operations.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - the directory cannot be read (e.g., invalid path or insufficient permissions).
	/// - an error occurs while reading a directory entry (e.g., invalid file or I/O error).
	/// - an error occurs during file renaming (e.g., due to filesystem restrictions, permissions issues, or name conflicts).
	fn process(&self, file_type: FileType, mode: RunMode) -> anyhow::Result<()>;

	/// Performs the actual renaming of a file and applies the changes when it's not a dry run execution.
	///
	/// # Arguments
	///
	/// - `file_type`: the type of file being renamed.
	/// - `num`: a sequential number to append to the new file name.
	/// - `mode`: whether to only preview (`DryRun`) or apply changes (`Apply`).
	///
	/// # Returns
	///
	/// - `Ok(())` if it is a dry run or if the file is renamed successfully.
	/// - `Err` if any file operation fails.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - the directory cannot be read (e.g., invalid path or insufficient permissions).
	/// - any file entry in the directory cannot be accessed (e.g., due to filesystem issues).
	/// - an error occurs during file renaming (e.g., due to filesystem restrictions, permissions issues, or name conflicts).
	fn rename(&self, file_type: FileType, num: usize, mode: RunMode) -> anyhow::Result<()>;

	fn to_lowercase_extension(&self) -> Option<String>;
}

impl Renamer for Path {
	fn detect_kind(&self) -> DetectedKind {
		// first try: magic-bytes detection (works even if extension is wrong)
		if let Some(kind) = infer::get_from_path(self).ok().flatten() {
			let mime = kind.mime_type();
			if mime.starts_with("image/") {
				return DetectedKind::Photo;
			}
			if mime.starts_with("video/") {
				return DetectedKind::Video;
			}
		}

		// fallback: extension-based detection
		if self.extension_is_photo() {
			return DetectedKind::Photo;
		}
		if self.extension_is_video() {
			return DetectedKind::Video;
		}

		DetectedKind::Other
	}

	fn extension_is_photo(&self) -> bool {
		matches!(
			self.extension()
				.and_then(OsStr::to_str)
				.map(str::to_ascii_lowercase)
				.as_deref(),
			Some("heic" | "jpg" | "jpeg" | "png")
		)
	}

	fn extension_is_video(&self) -> bool {
		matches!(
			self.extension()
				.and_then(OsStr::to_str)
				.map(str::to_ascii_lowercase)
				.as_deref(),
			Some("avi" | "mov" | "mp4")
		)
	}

	fn generate_new_name(&self, file_type: FileType, num: usize) -> anyhow::Result<String> {
		let taken_date = match file_type {
			FileType::Photo => self.get_date_from_photo()?,
			FileType::Video => self.get_date_from_video()?,
			FileType::All => unreachable!("\"All\" should not be passed here"),
		};
		let number = add_padded_numbering(3, num);
		let extension = self.to_lowercase_extension().unwrap_or_default();

		Ok(format!("{taken_date}_{file_type:?}{number}.{extension}"))
	}

	fn get_date_from_photo(&self) -> anyhow::Result<String> {
		let file =
			fs::File::open(self).with_context(|| format!("Failed to open '{}' for EXIF data", self.display()))?;
		let mut buf_reader = BufReader::new(&file);
		let exif_reader = Reader::new();

		let Ok(exif_data) = exif_reader.read_from_container(&mut buf_reader) else {
			log::warn!("Unable to read EXIF data from: {}", self.display());
			return Ok("NA".to_string());
		};

		exif_data
			.get_field(Tag::DateTimeOriginal, In::PRIMARY)
			.map_or_else(
				|| {
					log::warn!("Missing DateTimeOriginal tag");
					Ok("NA".to_string())
				},
				|field| {
					let value = field.display_value().with_unit(&exif_data);
					DateString::parse_and_format(&value.to_string())
				},
			)
	}

	fn get_date_from_video(&self) -> anyhow::Result<String> {
		let context = ffmpeg_next::format::input(&self)
			.with_context(|| format!("Failed to open '{}' with FFmpeg", self.display()))?;

		context.metadata().get("creation_time").map_or_else(
			|| {
				log::warn!("Missing creation_time tag");
				Ok("NA".to_string())
			},
			DateString::parse_and_format,
		)
	}

	fn is_hidden(&self) -> bool {
		self.file_name()
			.and_then(|n| n.to_str())
			.is_some_and(|n| n.starts_with('.'))
	}

	fn is_photo(&self) -> bool {
		self.detect_kind() == DetectedKind::Photo
	}

	fn is_video(&self) -> bool {
		self.detect_kind() == DetectedKind::Video
	}

	fn process(&self, file_type: FileType, mode: RunMode) -> anyhow::Result<()> {
		// collect all entries, filter only valid files
		let mut entries: Vec<(PathBuf, DetectedKind)> = fs::read_dir(self)
			.with_context(|| {
				format!(
					"Failed to read directory '{}' (file_type={file_type:?}, mode={mode:?})",
					self.display()
				)
			})?
			.filter_map(|entry| match entry {
				Ok(entry) => Some(entry.path()),
				Err(err) => {
					log::warn!(
						"Skipping directory entry in '{}' due to an I/O error: {err}",
						self.display()
					);
					None
				}
			})
			.filter(|file_path| file_path.is_file() && !file_path.is_hidden())
			.filter_map(|file_path| {
				let kind = file_path.detect_kind();
				if kind == DetectedKind::Other {
					log::debug!("Skipping unsupported file type: {}", file_path.display());
					return None;
				}

				let matches_requested = match file_type {
					FileType::Photo => kind == DetectedKind::Photo,
					FileType::Video => kind == DetectedKind::Video,
					FileType::All => matches!(kind, DetectedKind::Photo | DetectedKind::Video),
				};

				// ignore the file if its detected kind doesn't match the requested CLI filter
				if !matches_requested {
					log::debug!(
						"Skipping file due to type mismatch: requested={file_type:?}, detected={kind:?}, path={}",
						file_path.display()
					);
					return None;
				}

				matches_requested.then_some((file_path, kind))
			})
			.collect();

		// sort by current file name
		entries.sort_by_key(|(path, _)| path.file_name().map(ToOwned::to_owned));

		if entries.is_empty() {
			println!("\nEmpty folder! No files to process.\n");
			return Ok(());
		}

		print_header();

		let mut img_counter = 1;
		let mut vid_counter = 1;
		for (file_path, kind) in entries {
			let rename = |rename_as: FileType, counter: &mut usize| -> anyhow::Result<()> {
				file_path.rename(rename_as, *counter, mode)?;
				*counter += 1;
				Ok(())
			};

			match (file_type, kind) {
				(FileType::Photo | FileType::All, DetectedKind::Photo) => {
					rename(FileType::Photo, &mut img_counter)?;
				}
				(FileType::Video | FileType::All, DetectedKind::Video) => {
					rename(FileType::Video, &mut vid_counter)?;
				}
				_ => {
					// it should be unreachable due to filtering - do not panic
					log::warn!("Skipping unsupported file: {}", file_path.display());
				}
			}
		}

		println!("\n✅  Process completed!");
		Ok(())
	}

	fn rename(&self, file_type: FileType, num: usize, mode: RunMode) -> anyhow::Result<()> {
		if let Some(parent_path) = self.parent() {
			if let Some(old_file_name) = self.file_name() {
				let new_file_name = self.generate_new_name(file_type, num)?;
				println!("| {:<40} | {new_file_name:<40} |", old_file_name.to_string_lossy());

				// apply changes only when explicitly requested
				if mode.should_apply() {
					let new_file_path = parent_path.join(&new_file_name);
					fs::rename(self, &new_file_path).with_context(|| {
						format!(
							"Failed to rename file '{}' -> '{}'",
							self.display(),
							&new_file_path.display()
						)
					})?;
				}
			}
		}

		Ok(())
	}

	fn to_lowercase_extension(&self) -> Option<String> {
		self.extension()
			.and_then(OsStr::to_str)
			.map(str::to_lowercase)
	}
}

#[cfg(test)]
mod tests {
	use crate::renamer::Renamer;
	use std::path::Path;
	use test_case::test_case;

	#[test_case("/random/directory/file.HEIC", true)]
	#[test_case("file.jpg", true)]
	#[test_case("/random/directory/file.GIF", false)]
	#[test_case("/random/directory/file.pdf", false)]
	fn test_is_photo(path_str: &str, expected: bool) {
		let path = Path::new(path_str);
		assert_eq!(path.is_photo(), expected, "Unexpected result for: {path_str}");
	}

	#[test_case("/random/directory/file.mp4", true)]
	#[test_case("file.MOV", true)]
	#[test_case("/random/directory/file.PNG", false)]
	#[test_case("filetxt", false)]
	fn test_is_video(path_str: &str, expected: bool) {
		let path = Path::new(path_str);
		assert_eq!(path.is_video(), expected, "Unexpected result for: {path_str}");
	}

	#[test_case("/random/directory/IMG_1234.HEIC", Some("heic".to_string()); "uppercase extension")]
	#[test_case("/path/avideo.mov", Some("mov".to_string()); "lowercase extension")]
	#[test_case("file", None; "no extension")]
	fn test_to_lowercase_extension(path_str: &str, expected: Option<String>) {
		let path = Path::new(path_str);
		assert_eq!(path.to_lowercase_extension(), expected);
	}
}
