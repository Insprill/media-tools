use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use lazy_regex::{lazy_regex, Lazy, Regex};
use simplelog::info;

use crate::{path_to_str, utils};

// CD | Track | Title | Start Time
// Examples:
// 1 1 Break Out - Version 1 0:00
// 4 1 Prove It All Night - Version 1 2:41:38
static TIMESTAMP_PATTERN: Lazy<Regex> =
    lazy_regex!(r"([0-9*]*) ([0-9]*) (.*) ([0-9]{1,2}:[0-9]{1,2}:?[0-9]{1,2})");

pub fn run(
    src_file: &Path,
    dest_path: &Path,
    timestamps_file: &Path,
    artist: Option<String>,
    album: Option<String>,
    date: Option<String>,
    overwrite: bool,
    qffmpeg: bool,
) -> Result<()> {
    fs::create_dir_all(dest_path)?;

    let ext = src_file
        .extension()
        .context("No extension on source file!")?
        .to_string_lossy();

    let raw_timestamps = fs::read_to_string(timestamps_file)?;
    let timestamps = raw_timestamps
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| parse_timestamp(l))
        .collect::<Result<Vec<Timestamp>>>()?;

    let is_one_disc = timestamps.iter().all(|t| t.disc == 1);

    let meta_artist = artist.as_ref().map(|a| format!("artist={}", a));
    let meta_album_artist = artist.as_ref().map(|a| format!("albumartist={}", a));
    let meta_album = album.as_ref().map(|a| format!("album={}", a));
    let meta_date = date.as_ref().map(|a| format!("date={}", a));

    for i in 0..timestamps.len() {
        let stamp = &timestamps[i];

        let file_name = &format!("{:02}. {}.{}", stamp.track, stamp.title, ext);

        let mut args = Vec::new();
        args.push(if overwrite { "-y" } else { "-n" });
        args.push("-i");
        args.push(path_to_str!(src_file)?);

        args.push("-ss");
        args.push(stamp.start_time);
        if i + 1 < timestamps.len() {
            args.push("-to");
            args.push(timestamps[i + 1].start_time);
        }

        args.push("-write_id3v2");
        args.push("1");

        if let Some(ref artist) = meta_artist {
            args.push("-metadata");
            args.push(&artist);
        }
        if let Some(ref album_artist) = meta_album_artist {
            args.push("-metadata");
            args.push(&album_artist);
        }
        if let Some(ref album) = meta_album {
            args.push("-metadata");
            args.push(&album);
        }
        if let Some(ref date) = meta_date {
            args.push("-metadata");
            args.push(&date);
        }

        let meta_title = &format!("title={}", stamp.title);
        args.push("-metadata");
        args.push(meta_title);
        let meta_disc = &format!("disc={}", stamp.disc);
        args.push("-metadata");
        args.push(meta_disc);
        let meta_track = &format!("track={}", stamp.track);
        args.push("-metadata");
        args.push(meta_track);

        args.push("-c");
        args.push("copy");

        let out_file = if is_one_disc {
            dest_path.join(file_name)
        } else {
            let disc_dir = dest_path.join(format!("CD{}", stamp.disc));
            fs::create_dir_all(&disc_dir)?;
            disc_dir.join(file_name)
        };
        args.push(path_to_str!(out_file)?);

        utils::run_ffmpeg(qffmpeg, args, Option::None)?;
    }

    Ok(())
}

fn parse_timestamp(raw: &str) -> Result<Timestamp> {
    let matches = TIMESTAMP_PATTERN
        .captures(raw)
        .context(format!("Invalid timestamp '{}'!", raw))?;

    if matches.len() < 4 {
        bail!(format!(
            "Invalid timestamp '{}' (Only found {}/4 matches)!",
            raw,
            matches.len()
        ));
    }

    Ok(Timestamp {
        // Can't panic since we check length above
        disc: matches.get(1).unwrap().as_str().parse()?,
        track: matches.get(2).unwrap().as_str().parse()?,
        title: matches.get(3).unwrap().as_str(),
        start_time: matches.get(4).unwrap().as_str(),
    })
}

#[derive(Debug)]
struct Timestamp<'a> {
    disc: usize,
    track: usize,
    title: &'a str,
    start_time: &'a str,
}
