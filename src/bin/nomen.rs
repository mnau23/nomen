use clap::Parser;
use nomen::models::{Args, FileType, RunMode, UserConfirmation};
use nomen::renamer::Renamer;
use std::io::{self, IsTerminal, Write};

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let args = Args::parse();
	log::debug!("Input: {args:?}");

	let directory = &args.directory;
	// make sure the path is a valid directory
	if !directory.is_dir() {
		anyhow::bail!("'{}' is not a valid directory!", directory.display());
	}

	// initialize FFmpeg only when needed
	if matches!(args.file_type, FileType::All | FileType::Video) {
		log::trace!("Initializing FFmpeg...");
		ffmpeg_next::init()?;
	} else {
		log::trace!("Skipping FFmpeg initialization...");
	}

	let run_mode = if args.dry_run {
		RunMode::DryRun
	} else if args.force {
		RunMode::Apply(UserConfirmation::Forced)
	} else {
		RunMode::Apply(ask_confirmation(&args)?)
	};

	match run_mode {
		RunMode::Apply(UserConfirmation::Aborted) => {
			println!("⛔ Process aborted by user or invalid input: no changes were applied.");
		}
		RunMode::Apply(UserConfirmation::Confirmed) => {
			println!("✅ User confirmed: applying changes...");
			directory.process(args.file_type, false)?;
		}
		RunMode::Apply(UserConfirmation::Forced) => {
			println!("🚨 Force mode enabled: applying changes without confirmation...");
			directory.process(args.file_type, false)?;
		}
		RunMode::DryRun => {
			println!("🦺 Dry run enabled: no changes will be applied.");
			directory.process(args.file_type, true)?;
		}
	}

	Ok(())
}

fn ask_confirmation(args: &Args) -> anyhow::Result<UserConfirmation> {
	// make sure to run in an interactive terminal
	if !io::stdin().is_terminal() {
		anyhow::bail!("Refusing to apply changes without confirmation on a non-interactive stdin.");
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

	if !is_yes(&input) {
		return Ok(UserConfirmation::Aborted);
	}

	Ok(UserConfirmation::Confirmed)
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
