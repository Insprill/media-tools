use anyhow::{Context, Result};
use std::{fs, path::Path};

use crate::{path_to_str, utils};

pub fn run(
    src_path: &Path,
    dest_path: &Path,
    preset: u8,
    crf: u8,
    keyframe_interval: u16,
    force_10bit: bool,
    overwrite: bool,
    qffmpeg: bool,
) -> Result<()> {
    fs::create_dir_all(dest_path)?;

    for path in utils::read_dir(src_path, |p| {
        p.is_file()
            && p.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .filter(|e| e == "mkv" || e == "mp4" || e == "mov")
                .is_some()
    })? {
        let rel_path = path.strip_prefix(src_path)?;
        let out_path = dest_path.join(rel_path);

        let preset_str = &preset.to_string();
        let crf_str = &crf.to_string();
        let keyframe_interval_str = &keyframe_interval.to_string();

        let mut args = vec![
            if overwrite { "-y" } else { "-n" },
            "-i",
            path_to_str!(&path)?,
            "-map",
            "0",
            "-c",
            "copy",
            "-c:v",
            "libsvtav1",
            "-preset",
            preset_str,
            "-crf",
            crf_str,
            "-g",
            keyframe_interval_str,
        ];

        if force_10bit {
            args.push("-pix_fmt");
            args.push("yuv420p10le");
        }

        args.push(path_to_str!(out_path)?);

        utils::run_ffmpeg(qffmpeg, args)?;
    }

    Ok(())
}
