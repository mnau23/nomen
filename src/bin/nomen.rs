use anyhow::Context;
use clap::Parser;
use nomen::models::{Args, FileType, RunMode};
use nomen::renamer::Renamer;
use std::io::{self, IsTerminal, Write};

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let args = Args::parse();
	log::debug!("Input: {args:?}");

	// initialize FFmpeg only when needed
	if matches!(args.file_type, FileType::All | FileType::Video) {
		log::trace!("Initializing FFmpeg...");
		ffmpeg_next::init().context("Failed to initialize FFmpeg")?;
	} else {
		log::trace!("Skipping FFmpeg initialization...");
	}

	let run_mode = if args.dry_run {
		println!("🦺 Dry run enabled: no changes will be applied.");
		RunMode::DryRun
	} else if args.force {
		println!("🚨 Force mode enabled: applying changes without confirmation...");
		RunMode::Apply
	} else if ask_confirmation(&args)? {
		println!("✅ User confirmed: applying changes...");
		RunMode::Apply
	} else {
		println!("⛔ Process aborted: no changes were applied.");
		return Ok(());
	};

	args.directory
		.process(args.file_type, run_mode)
		.with_context(|| {
			format!(
				"Failed to process directory '{}' (file_type={:?}, mode={run_mode:?})",
				args.directory.display(),
				args.file_type
			)
		})?;

	Ok(())
}

fn ask_confirmation(args: &Args) -> anyhow::Result<bool> {
	// make sure to run in an interactive terminal
	if !io::stdin().is_terminal() {
		anyhow::bail!(
			"Refusing to apply changes without confirmation on a non-interactive stdin. Re-run with --force."
		);
	}

	println!(
		"Apply changes in '{}' for file types '{:?}'?",
		args.directory.display(),
		args.file_type
	);
	print!("Type 'yes' (or 'y') to continue: ");
	io::stdout().flush()?;

	let mut input = String::new();
	io::stdin().read_line(&mut input)?;

	Ok(is_yes(&input))
}

fn is_yes(input: &str) -> bool {
	matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

#[cfg(test)]
mod tests {
	use super::is_yes;
	use test_case::test_case;

	#[test_case("yes", true; "yes")]
	#[test_case(" YES ", true; "yes with spaces and caps")]
	#[test_case("y", true; "y")]
	#[test_case("no", false; "no")]
	#[test_case("", false; "empty")]
	#[test_case("boh", false; "wrong string")]
	fn test_is_yes(input: &str, expected: bool) {
		assert_eq!(is_yes(input), expected);
	}
}
