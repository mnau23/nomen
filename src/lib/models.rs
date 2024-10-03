use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Reads all files in a directory and renames the photos with a specified format.
#[derive(Debug, Parser)]
pub struct Args {
	/// Directory path
	#[arg(long, env)]
	pub directory: PathBuf,

	/// File type to convert
	#[arg(long, env)]
	pub file_type: FileType,

	/// Perform a dry run
	#[arg(long = "dry-run", default_value_t = true, action = clap::ArgAction::SetTrue)]
	#[arg(long = "no-dry-run", action = clap::ArgAction::SetFalse)]
	pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileType {
	Photo,
}
