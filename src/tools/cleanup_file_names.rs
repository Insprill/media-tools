use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use lazy_regex::{lazy_regex, Lazy, Regex};
use log::info;

static REMOVE_PATTERN: Lazy<Regex> = lazy_regex!(r"\s*\[[^\]]*\]");

pub fn run(path: PathBuf) -> Result<()> {
    info!("Cleaning up file names in {}", path.display());

    for entry in fs::read_dir(&path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?
    {
        let entry =
            entry.with_context(|| format!("Failed to access entry in: {}", path.display()))?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            run(entry_path.clone())?;
        }

        if let Some(file_name) = entry_path.file_name().and_then(|s| s.to_str()) {
            let new_name = REMOVE_PATTERN.replace_all(file_name, "").into_owned();
            if new_name == file_name {
                continue;
            }

            let new_path = entry_path.with_file_name(&new_name);
            fs::rename(&entry_path, &new_path).with_context(|| {
                format!(
                    "Failed to rename: {} to {}",
                    entry_path.display(),
                    new_path.display()
                )
            })?;
        }
    }

    Ok(())
}
