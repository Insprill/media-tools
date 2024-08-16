use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use tools::cleanup_file_names;

mod tools;

#[derive(Debug, Parser)]
#[command(name = "mediatools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Removes all IDs in square brackets from directory/file names recursively.
    /// E.g. 'Badlands [12345678].flac' -> 'Badlands.flac'
    #[command(arg_required_else_help = true)]
    CleanupFileNames { path: PathBuf },
}
fn main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = Cli::parse();

    match args.command {
        Commands::CleanupFileNames { path } => cleanup_file_names::run(path)?,
    }

    Ok(())
}
