use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Processes and renames files in a directory based on the specified file type.
#[derive(Debug, Parser)]
pub struct Args {
	/// Directory path
	#[arg(long)]
	pub directory: PathBuf,

	/// File type to convert
	#[arg(long)]
	pub file_type: FileType,

	/// Use to disable dry run (active by default)
	// by default, the action selected is ArgAction::SetTrue
	#[arg(long = "no-dry-run", action = clap::ArgAction::SetFalse)]
	pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileType {
	Photo,
	Video,
}
