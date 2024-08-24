use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use simplelog::{error, info};

pub fn read_dir<P>(path: &Path, predicate: P) -> Result<Vec<PathBuf>>
where
    P: Fn(&PathBuf) -> bool,
{
    let mut paths = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?
        .filter_map(|r| r.ok().map(|e| e.path()).take_if(|p| predicate(p)))
        .collect::<Vec<PathBuf>>();
    paths.sort_unstable();
    Ok(paths)
}

pub fn run_ffmpeg<I, S>(quiet: bool, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("ffmpeg");
    cmd.args(args);

    info!("{:?}", cmd);
    let output = cmd.output()?;

    let mut failed = false;
    // FFmpeg outputs everything to stderr, *not* stdout!
    // This means we have to guess what's an error and what isn't.
    // It's hacky and isn't fool-proof but it's good enough.
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        let out = format!("<green>[FFmpeg]</> {}", line);
        if line.contains("Error")
            || line.contains("Conversion failed")
            || line.contains("Unknown encoder")
            || line.contains("Invalid argument")
        {
            error!("{}", out);
            failed = true;
        } else if !quiet {
            info!("{}", out)
        }
    }
    if failed {
        bail!(
            "Aborting due to ffmpeg failure!{}",
            if quiet {
                " Run without '-q' or '--qffmpeg' to see the full ffmpeg output for why it failed."
            } else {
                ""
            }
        );
    }

    Ok(())
}

#[macro_export]
macro_rules! path_to_str {
    ($path:expr) => {
        $path.to_str().context("Path contains invalid characters")
    };
}
