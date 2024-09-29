#![warn(clippy::all, clippy::pedantic)]

use anyhow::{bail, Result};
use clap::Parser;
use log::{debug, info};
use nomen::models::Args;
use nomen::renamer::Renamer;
use std::path::Path;

fn main() -> Result<()> {
	env_logger::init();
	let args = Args::parse();
	debug!("Input: {args:?}");

	let directory: &Path = &args.directory;
	// make sure the path is a valid directory
	if !directory.is_dir() {
		bail!("{directory:?} is not a valid directory");
	}

	if args.dry_run {
		info!("🦺 This is a dry run: no changes will be made.");
	} else {
		info!("🚨 Running program: changes will be applied.");
	}

	directory.process(args.file_type, args.dry_run)?;

	info!("✅  Process completed!");
	Ok(())
}
