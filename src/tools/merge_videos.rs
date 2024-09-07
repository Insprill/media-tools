use std::{fs, path::Path};

use anyhow::{Context, Result};
use simplelog::{error, info};

use crate::{path_to_str, utils};

pub fn run(
    base_path: &Path,
    content_path: &Path,
    dest_path: &Path,
    video_from_base: bool,
    audio_from_base: bool,
    use_content_names: bool,
    overwrite: bool,
    qffmpeg: bool,
) -> Result<()> {
    let base_files = utils::read_dir(base_path, |p| p.is_file())?;
    let content_files = utils::read_dir(content_path, |p| p.is_file())?;

    if base_files.len() != content_files.len() {
        error!("The base directory has {} files, but the stream directory has {} files! There must be the same amount of files in both directories.", base_files.len(), content_files.len());
        return Ok(());
    }

    for i in 0..base_files.len() {
        let base_file = &base_files[i];
        let content_file = &content_files[i];

        info!(
            "Combining {:?} with {:?}",
            base_file.file_name().unwrap_or_default(),
            content_file.file_name().unwrap_or_default()
        );

        fs::create_dir_all(dest_path)?;

        let file_name_to_copy = if use_content_names {
            content_file
        } else {
            base_file
        };
        let dest_file = dest_path.join(file_name_to_copy.file_name().context("No file name?")?);

        let mut args = vec![
            if overwrite { "-y" } else { "-n" },
            "-i",
            path_to_str!(base_file)?,
            "-i",
            path_to_str!(content_file)?,
        ];

        // Copy the video stream from input 1
        if !video_from_base {
            args.push("-map");
            args.push("1:v");
        }

        // Copy the audio stream from input 1
        if !audio_from_base {
            args.push("-map");
            args.push("1:a");
        }

        // Copy everything else from input 0
        args.push("-map");
        args.push("0");
        args.push("-map_metadata");
        args.push("0");

        // Don't copy video from input 0
        if !video_from_base {
            args.push("-map");
            args.push("-0:v");
        }

        // Don't copy audio from input 0
        if !audio_from_base {
            args.push("-map");
            args.push("-0:a");
        }

        // Don't re-encode anything
        args.push("-c");
        args.push("copy");
        args.push(path_to_str!(dest_file)?);

        utils::run_ffmpeg(qffmpeg, args)?;
    }

    Ok(())
}
