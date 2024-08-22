use anyhow::{ensure, Context, Result};
use simplelog::{error, warn};
use std::{fs, path::Path};

use crate::{path_to_str, utils};

pub fn run(
    src_path: &Path,
    src_container: &str,
    dest_path: &Path,
    bitrate: &str,
    codec: &str,
    container: &str,
    overwrite: bool,
    qffmpeg: bool,
) -> Result<()> {
    if !validate_params(bitrate, codec, container) {
        return Ok(());
    }
    process_dir(
        src_path,
        src_container,
        dest_path,
        bitrate,
        codec,
        container,
        qffmpeg,
        overwrite,
        false,
    )
}

fn process_dir(
    src_path: &Path,
    src_container: &str,
    dest_path: &Path,
    bitrate: &str,
    codec: &str,
    container: &str,
    qffmpeg: bool,
    overwrite: bool,
    nested: bool,
) -> Result<()> {
    if src_path.is_file() {
        transcode_file(
            src_path, dest_path, bitrate, codec, container, overwrite, qffmpeg,
        )?;
        return Ok(());
    }

    fs::create_dir_all(dest_path)?;

    for path in utils::read_dir(src_path, |_| true)? {
        let strip_prefix = if nested {
            src_path.parent().context("no parent?")?
        } else {
            src_path
        };
        let rel_path = path.strip_prefix(strip_prefix)?;
        let out_path = dest_path.join(rel_path);

        if path.is_dir() {
            process_dir(
                &path,
                src_container,
                dest_path,
                bitrate,
                codec,
                container,
                overwrite,
                qffmpeg,
                true,
            )?;
        } else {
            if path.extension().unwrap_or_default().to_string_lossy() != src_container {
                continue;
            }
            if let Some(parent_dir) = out_path.parent() {
                fs::create_dir_all(parent_dir)?;
            }
            transcode_file(
                &path, &out_path, bitrate, codec, container, overwrite, qffmpeg,
            )?;
        }
    }
    Ok(())
}

fn validate_params(bitrate: &str, codec: &str, container: &str) -> bool {
    if !bitrate.ends_with("k") && bitrate.parse::<usize>().unwrap_or_default() < 1000 {
        warn!(
            "Bitrate {} seems too low, did you mean {}k?",
            bitrate, bitrate
        );
    }
    if codec == "aac" || codec == "libfdk_aac" {
        if container != "m4a" && container != "mkv" {
            error!("The AAC codec can only be in an m4a or mkv container!");
            return false;
        }
        if codec == "aac" {
            warn!("You should use 'libfdk_acc' instead of 'aac' for better quality!");
        }
    } else if codec == "mp3" {
        if container != "mp3" {
            error!("The MP3 codec can only be placed in an mp3 container.");
            return false;
        }
    } else if codec == "opus" {
        if container != "ogg" {
            error!("The opus codec can only be placed in an ogg container.");
            return false;
        }
    } else {
        error!("Unsupported codec '{}'!", codec);
    }
    true
}

fn transcode_file(
    src: &Path,
    dest: &Path,
    bitrate: &str,
    codec: &str,
    container: &str,
    overwrite: bool,
    qffmpeg: bool,
) -> Result<()> {
    ensure!(src.is_file(), "transcode_file does not accept directories!");

    let mut args = vec![
        if overwrite { "-y" } else { "-n" },
        "-i",
        path_to_str!(src)?,
        "-acodec",
        codec,
        "-ab",
        bitrate,
        "-map_metadata",
        "0",
        "-id3v2_version",
        "3",
    ];

    // Codec-specific args
    if codec.contains("aac") {
        args.push("-vn");
    }

    // Last arg must be the output file
    let dest_path = dest.with_extension(container);
    args.push(path_to_str!(dest_path)?);

    utils::run_ffmpeg(qffmpeg, args, Option::None)
}
