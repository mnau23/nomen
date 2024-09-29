use crate::models::FileType;
use anyhow::{bail, Result};
use chrono::NaiveDateTime;
use exif::{In, Reader, Tag};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub trait Renamer {
	/// Retrieves the original date when an item was taken, based on EXIF data, as a `String`.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - the file cannot be opened (e.g., the file does not exist or access is denied).
	/// - the EXIF data cannot be read from the file (e.g., if the file is not a valid image with EXIF data).
	/// - the `DateTimeOriginal` tag is missing from the EXIF metadata.
	/// - there is an issue parsing the date from the EXIF metadata (e.g., if the date format is invalid).
	fn get_taken_date(&self) -> Result<String>;

	fn is_photo(&self) -> bool;

	/// Processes files in the directory and renames them based on the given file type.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - the directory cannot be read (e.g., invalid path or insufficient permissions).
	/// - an error occurs while reading a directory entry (e.g., invalid file or I/O error).
	/// - an error occurs during file renaming (e.g., due to filesystem restrictions, permissions issues, or name conflicts).
	fn process(&self, file_type: FileType, dry_run: bool) -> Result<()>;

	/// Performs the actual renaming of a file and applies the changes when it's not a dry run execution.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - the directory cannot be read (e.g., invalid path or insufficient permissions).
	/// - any file entry in the directory cannot be accessed (e.g., due to filesystem issues).
	/// - an error occurs during file renaming (e.g., due to filesystem restrictions, permissions issues, or name conflicts).
	fn rename(&self, file_type: FileType, num: usize, dry_run: bool) -> Result<()>;

	fn to_lowercase_extension(&self) -> Option<String>;
}

impl Renamer for Path {
	fn get_taken_date(&self) -> Result<String> {
		let file = File::open(self)?;
		let mut buf_reader = BufReader::new(&file);
		let exif_reader = Reader::new();
		let exif_data = exif_reader.read_from_container(&mut buf_reader)?;

		match exif_data.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
			Some(field) => {
				let value = field.display_value().with_unit(&exif_data);
				let date = NaiveDateTime::parse_from_str(&value.to_string(), "%Y-%m-%d %H:%M:%S")?.date();
				Ok(date.to_string())
			}
			None => bail!("Missing DateTimeOriginal tag"),
		}
	}

	fn is_photo(&self) -> bool {
		matches!(
			self.to_lowercase_extension().as_deref(),
			Some("heic" | "jpg" | "jpeg" | "png")
		)
	}

	fn process(&self, file_type: FileType, dry_run: bool) -> Result<()> {
		print_header();

		// collect all entries, filter out any invalid ones, then sort by current file name
		let mut entries: Vec<_> = fs::read_dir(self)?.filter_map(Result::ok).collect();
		entries.sort_by_key(fs::DirEntry::file_name);

		let mut counter = 1;
		for entry in entries {
			let file_path: &Path = &entry.path();

			if file_path.is_file() {
				match file_type {
					FileType::Photo => {
						if file_path.is_photo() {
							file_path.rename(file_type, counter, dry_run)?;
							counter += 1;
						}
					}
				}
			}
		}

		println!();
		Ok(())
	}

	fn rename(&self, file_type: FileType, num: usize, dry_run: bool) -> Result<()> {
		if let Some(parent_path) = self.parent() {
			if let Some(old_file_name) = self.file_name() {
				// rules to create the new name
				let taken_date = self.get_taken_date().unwrap_or_else(|_| String::from("{}"));
				let number = add_padded_numbering(3, num);
				let extension = self.to_lowercase_extension().unwrap_or_default();

				let new_file_name = format!("{taken_date}_{file_type:?}{number}.{extension}");
				println!("| {:<40} | {new_file_name:<40} |", old_file_name.to_string_lossy());

				// apply changes only if it's not a dry run
				if !dry_run {
					let new_file_path = parent_path.join(new_file_name.clone());
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

fn add_padded_numbering(pad: usize, n: usize) -> String {
	format!("{n:0pad$}")
}

fn print_header() {
	println!("\n| {:^40} | {:^40} |", "CURRENT", "NEW");
	let separator = "=".repeat(40);
	println!("| {separator:<40} | {separator:<40} |");
}
