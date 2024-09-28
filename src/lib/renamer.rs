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
	fn get_taken_date(&self) -> Result<String>;
	fn is_photo(&self) -> bool;
	fn process(&self, types: FileType, dry_run: bool) -> Result<()>;
	fn rename(&self, file_type: FileType, num: usize, dry_run: bool) -> Result<()>;
	fn to_lowercase_extension(&self) -> Option<String>;
}

impl Renamer for Path {
	fn get_taken_date(&self) -> Result<String> {
		let file = File::open(self)?;
		let mut buf_reader = BufReader::new(&file);
		let exif_reader = Reader::new();
		let exif = exif_reader.read_from_container(&mut buf_reader)?;

		match exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
			Some(field) => {
				let value = field.display_value().with_unit(&exif);
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
		let mut counter = 1;

		println!("\n| {:^40} | {:^40} |", "CURRENT", "NEW");
		let separator = "=".repeat(40);
		println!("| {separator:<40} | {separator:<40} |");

		for entry in fs::read_dir(self)? {
			let entry = entry?;
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
