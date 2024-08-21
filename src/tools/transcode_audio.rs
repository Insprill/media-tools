use anyhow::{bail, ensure, Context, Result};
use simplelog::{error, info, warn};
use std::{fs, path::Path, process::Command};

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

    for entry in fs::read_dir(&src_path)
        .with_context(|| format!("Failed to read directory: {}", src_path.display()))?
    {
        let entry =
            entry.with_context(|| format!("Failed to access entry in: {}", src_path.display()))?;
        let path = entry.path();

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
    return true;
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

    // Command essentials
    let mut cmd = Command::new("ffmpeg");
    cmd.arg(if overwrite { "-y" } else { "-n" })
        .arg("-i")
        .arg(src)
        .arg("-acodec")
        .arg(codec)
        .arg("-ab")
        .arg(bitrate)
        .arg("-map_metadata")
        .arg("0")
        .arg("-id3v2_version")
        .arg("3");

    // Codec-specific args
    if codec.contains("aac") {
        cmd.arg("-vn");
    }

    // Last arg must be the output file
    cmd.arg(dest.with_extension(container));

    info!("{:?}", cmd);

    let output = cmd.output()?;

    let mut failed = false;
    //FFmpeg outputs everything to stderr, *not* stdout!
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        let out = format!("<green>[FFmpeg]</> {}", line);
        if line.contains("Error")
            || line.contains("Conversion failed")
            || line.contains("Unknown encoder")
        {
            error!("{}", out);
            failed = true;
        } else if !qffmpeg {
            info!("{}", out)
        }
    }
    if failed {
        bail!(
            "Aborting due to ffmpeg failure!{}",
            if qffmpeg {
                " Run without '-q' or '--qffmpeg' to see the full ffmpeg output for why it failed."
            } else {
                ""
            }
        );
    }

    Ok(())
}
