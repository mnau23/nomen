use crate::helpers::{add_padded_numbering, print_header};
use crate::models::{DateString, FileType};
use exif::{In, Reader, Tag};
use log::warn;
use std::ffi::OsStr;
use std::fs;
use std::io::BufReader;
use std::path::Path;

pub trait Renamer {
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

	/// TODO
	fn is_hidden(&self) -> bool;

	fn is_photo(&self) -> bool;

	fn is_video(&self) -> bool;

	/// Processes and renames files in a directory based on the specified file type.
	///
	/// # Arguments
	///
	/// - `file_type`: the type of file to be processed.
	/// - `dry_run`: if `true`, no actual renaming will occur; the function will only print the intended changes.
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
	fn process(&self, file_type: FileType, dry_run: bool) -> anyhow::Result<()>;

	/// Performs the actual renaming of a file and applies the changes when it's not a dry run execution.
	///
	/// # Arguments
	///
	/// - `file_type`: the type of file being renamed.
	/// - `num`: a sequential number to append to the new file name.
	/// - `dry_run`: if `true`, the renaming operation is simulated - only the intended renaming is printed to the console.
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
	fn rename(&self, file_type: FileType, num: usize, dry_run: bool) -> anyhow::Result<()>;

	fn to_lowercase_extension(&self) -> Option<String>;
}

impl Renamer for Path {
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
		let file = fs::File::open(self)?;
		let mut buf_reader = BufReader::new(&file);
		let exif_reader = Reader::new();

		let Ok(exif_data) = exif_reader.read_from_container(&mut buf_reader) else {
			warn!("Unable to read EXIF data from: {}", self.display());
			return Ok("NA".to_string());
		};

		exif_data
			.get_field(Tag::DateTimeOriginal, In::PRIMARY)
			.map_or_else(
				|| {
					warn!("Missing DateTimeOriginal tag");
					Ok("NA".to_string())
				},
				|field| {
					let value = field.display_value().with_unit(&exif_data);
					DateString::parse_and_format(&value.to_string())
				},
			)
	}

	fn get_date_from_video(&self) -> anyhow::Result<String> {
		let context = ffmpeg_next::format::input(&self)?;

		context.metadata().get("creation_time").map_or_else(
			|| {
				warn!("Missing creation_time tag");
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
		matches!(
			self.to_lowercase_extension().as_deref(),
			Some("heic" | "jpg" | "jpeg" | "png")
		)
	}

	fn is_video(&self) -> bool {
		matches!(self.to_lowercase_extension().as_deref(), Some("avi" | "mov" | "mp4"))
	}

	fn process(&self, file_type: FileType, dry_run: bool) -> anyhow::Result<()> {
		print_header();

		// collect all entries, filter out any invalid ones, then sort by current file name
		let mut entries: Vec<_> = fs::read_dir(self)?.filter_map(Result::ok).collect();
		entries.sort_by_key(fs::DirEntry::file_name);

		let mut img_counter = 1;
		let mut vid_counter = 1;
		for entry in entries {
			let file_path: &Self = &entry.path();

			// ignore hidden files
			if file_path.is_hidden() {
				continue;
			}

			if file_path.is_file() {
				let rename = |file_type, counter: &mut usize| -> anyhow::Result<()> {
					file_path.rename(file_type, *counter, dry_run)?;
					*counter += 1;
					Ok(())
				};

				match file_type {
					FileType::Photo if file_path.is_photo() => {
						rename(FileType::Photo, &mut img_counter)?;
					}
					FileType::Video if file_path.is_video() => {
						rename(FileType::Video, &mut vid_counter)?;
					}
					FileType::All => {
						if file_path.is_photo() {
							rename(FileType::Photo, &mut img_counter)?;
						} else if file_path.is_video() {
							rename(FileType::Video, &mut vid_counter)?;
						}
					}
					_ => unreachable!("Unexpected match"),
				}
			}
		}

		println!();
		Ok(())
	}

	fn rename(&self, file_type: FileType, num: usize, dry_run: bool) -> anyhow::Result<()> {
		if let Some(parent_path) = self.parent() {
			if let Some(old_file_name) = self.file_name() {
				let new_file_name = self.generate_new_name(file_type, num)?;
				println!("| {:<40} | {new_file_name:<40} |", old_file_name.to_string_lossy());

				// apply changes only if it's not a dry run
				if !dry_run {
					let new_file_path = parent_path.join(new_file_name);
					fs::rename(self, &new_file_path)?;
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
