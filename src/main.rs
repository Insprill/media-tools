use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use tools::{cleanup_file_names, merge_videos, transcode_audio};

mod tools;
mod utils;

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
    /// Recursivley transcodes all audio files in a directory to a different format, while preserving all
    /// metadata.
    #[command(arg_required_else_help = true)]
    TranscodeAudio {
        /// The source directory to transcode from.
        src_path: PathBuf,
        /// The container (file extension) to search for. Only files with this file extension will
        /// be transcoded.
        src_container: String,
        /// The destination path. If it doesn't exist, it will be created. The first layer of files
        /// in the source directory will be placed directly in this folder.
        dest_path: PathBuf,
        /// The bitrate of the transcoded files.
        bitrate: String,
        /// The codec to be used for encoding the file.
        codec: String,
        /// The container to place the files in.
        container: String,
        /// Force overwrite any existing files.
        #[clap(long, short)]
        overwrite: bool,
        /// Hides FFmpeg's output. If commands aren't working as expected, omit this flag to see
        /// what's going on.
        #[clap(long, short)]
        qffmpeg: bool,
    },
    #[command(arg_required_else_help = true)]
    /// Merges the video/audio streams from one file and the metadata/attachments from another, for
    /// all files in two directories.
    MergeVideos {
        /// The directory containing the files to preserve metadata/attachments from.
        base_path: PathBuf,
        /// The directory containing the files to preserve the video/audio from.
        content_path: PathBuf,
        /// The directory to write the modified files into.
        dest_path: PathBuf,
        /// Whether the new files should take the name of the content file instead of the base
        /// file.
        #[clap(long, short)]
        use_content_names: bool,
        /// Force overwrite any existing files.
        #[clap(long, short)]
        overwrite: bool,
        /// Hides FFmpeg's output. If commands aren't working as expected, omit this flag to see
        /// what's going on.
        #[clap(long, short)]
        qffmpeg: bool,
    },
}
fn main() -> Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .with_context(|| "Failed to initialize logger!")?;

    let args = Cli::parse();

    match args.command {
        Commands::CleanupFileNames { path } => cleanup_file_names::run(path)?,
        Commands::TranscodeAudio {
            src_path,
            src_container,
            dest_path,
            bitrate,
            codec,
            container,
            overwrite,
            qffmpeg,
        } => transcode_audio::run(
            src_path.as_path(),
            &src_container,
            dest_path.as_path(),
            &bitrate,
            &codec,
            &container,
            overwrite,
            qffmpeg,
        )?,
        Commands::MergeVideos {
            base_path,
            content_path,
            dest_path,
            use_content_names,
            overwrite,
            qffmpeg,
        } => merge_videos::run(
            &base_path,
            &content_path,
            &dest_path,
            use_content_names,
            overwrite,
            qffmpeg,
        )?,
    }

    Ok(())
}
