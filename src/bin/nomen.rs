use anyhow::{bail, Result};
use clap::Parser;
use log::{debug, info};
use nomen::models::NomenConfig;
use nomen::renamer::Renamer;
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();

    let config = NomenConfig::parse();
    debug!("Configuration: {config:?}");

    let directory: &Path = &config.directory;
    // make sure the path is a valid directory
    if !directory.is_dir() {
        bail!("{directory:?} is not a valid directory");
    }

    if config.dry_run {
        info!("🎉 This is a dry run: no changes will be made.");
    } else {
        info!("⚠️ Running program: changes will be applied.");
    }

    directory.process(config.file_type, config.dry_run)?;

    info!("✅  Process completed!");
    Ok(())
}
