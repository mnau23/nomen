use anyhow::anyhow;
use chrono::NaiveDateTime;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Processes and renames files in a directory based on the specified file type.
#[derive(Debug, Parser)]
pub struct Args {
	/// Directory path
	#[arg(long)]
	pub directory: PathBuf,

	/// File type to convert
	#[arg(long, default_value = "all")]
	pub file_type: FileType,

	/// Use to disable dry run (active by default)
	// by default, the action selected is ArgAction::SetTrue
	#[arg(long = "no-dry-run", action = clap::ArgAction::SetFalse)]
	pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileType {
	All,
	Photo,
	Video,
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

// TODO: add test for DateString
