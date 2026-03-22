use anyhow::anyhow;
use chrono::NaiveDateTime;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

fn parse_directory(s: &str) -> Result<PathBuf, anyhow::Error> {
	let directory = PathBuf::from(s);

	if !directory.exists() {
		anyhow::bail!("Directory does not exist!");
	}

	if !directory.is_dir() {
		anyhow::bail!("Not a directory!");
	}

	Ok(directory)
}

/// Processes and renames files in a directory based on the specified file type.
#[derive(Debug, Parser)]
pub struct Args {
	/// Directory path
	#[arg(long, value_hint = clap::ValueHint::DirPath, value_parser = parse_directory)]
	pub directory: PathBuf,

	/// File type to convert
	#[arg(long, default_value = "all")]
	pub file_type: FileType,

	/// Use to disable dry run (active by default)
	// by default, the action selected is ArgAction::SetTrue
	#[arg(long = "no-dry-run", action = clap::ArgAction::SetFalse)]
	pub dry_run: bool,

	/// Skip the interactive confirmation prompt when applying changes
	#[arg(long, requires = "dry_run")]
	pub force: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileType {
	All,
	Photo,
	Video,
}

/// Execution mode for filesystem operations
#[derive(Clone, Copy, Debug)]
pub enum RunMode {
	DryRun,
	Apply,
}

impl RunMode {
	#[must_use]
	pub const fn should_apply(&self) -> bool {
		matches!(self, Self::Apply)
	}
}

pub(crate) struct DateString;

impl DateString {
	pub(crate) fn parse_and_format(date_str: &str) -> anyhow::Result<String> {
		NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
			.or_else(|_| NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ"))
			.map(|dt| dt.date().to_string())
			.map_err(|_| anyhow!("Failed to parse and format date: {date_str}"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case("2024-10-16 12:34:56", Ok("2024-10-16".to_string()); "space separated datetime")]
	#[test_case("2024-10-16T12:34:56.123Z", Ok("2024-10-16".to_string()); "T separator with fractional seconds")]
	#[test_case("2024-10", Err("Failed to parse and format date: 2024-10".to_string()); "invalid datetime")]
	fn test_parse_and_format(input: &str, expected: Result<String, String>) {
		let result = DateString::parse_and_format(input).map_err(|e| e.to_string());

		assert_eq!(result, expected);
	}
}
