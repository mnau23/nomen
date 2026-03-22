use anyhow::{Result, bail};
use clap::Parser;
use nomen::models::{Args, FileType};
use nomen::renamer::Renamer;

fn main() -> Result<()> {
	env_logger::init();

	let args = Args::parse();
	log::debug!("Input: {args:?}");

	let directory = &args.directory;
	// make sure the path is a valid directory
	if !directory.is_dir() {
		bail!("{} is not a valid directory", directory.display());
	}

	if args.dry_run {
		println!("🦺 This is a dry run: no changes will be made.");
	} else {
		println!("🚨 Running program: changes will be applied.");
	}

	// initialize FFmpeg only when needed
	if matches!(args.file_type, FileType::All | FileType::Video) {
		log::trace!("Initializing FFmpeg...");
		ffmpeg_next::init()?;
	} else {
		log::trace!("Skipping FFmpeg initialization...");
	}

	directory.process(args.file_type, args.dry_run)?;

	Ok(())
}
