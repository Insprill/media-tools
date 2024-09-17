use anyhow::{Context, Result};
use std::{fs, path::Path};

use crate::{path_to_str, utils};

pub fn run(
    src_path: &Path,
    dest_path: &Path,
    audio_stream: u8,
    subtitle_stream: u8,
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

        let disposition_audio = &format!("-disposition:a:{}", audio_stream);
        let disposition_subtitles = &format!("-disposition:s:{}", subtitle_stream);

        let args = vec![
            if overwrite { "-y" } else { "-n" },
            "-i",
            path_to_str!(&path)?,
            "-map",
            "0",
            "-c",
            "copy",
            "-disposition:a",
            "0",
            "-disposition:s",
            "0",
            disposition_audio,
            "default",
            disposition_subtitles,
            "default",
            path_to_str!(out_path)?,
        ];

        utils::run_ffmpeg(qffmpeg, args)?;
    }

    Ok(())
}
