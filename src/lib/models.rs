use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
	/// Directory path
	#[arg(long, env)]
	pub directory: PathBuf,

	/// File type to convert
	#[arg(long, env)]
	pub file_type: FileType,

	/// Perform a dry run
	#[arg(long, env, default_value = "true")]
	pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileType {
	Photo,
}
