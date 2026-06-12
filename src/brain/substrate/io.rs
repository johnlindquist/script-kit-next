//! Atomic filesystem writes for the brain substrate.

use std::fs;
use std::io::Write as _;
use std::path::Path;

use anyhow::{Context as _, Result};

/// Write `contents` to `path` atomically via a temp file in the same directory.
pub fn atomic_write(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("brain path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating brain dir {}", parent.display()))?;

    let mut temp = tempfile::Builder::new()
        .prefix(".brain-write.")
        .suffix(".tmp")
        .tempfile_in(parent)
        .with_context(|| format!("creating temp file in {}", parent.display()))?;

    temp.write_all(contents.as_bytes())
        .with_context(|| format!("writing temp file {}", temp.path().display()))?;
    temp.flush()
        .with_context(|| format!("flushing temp file {}", temp.path().display()))?;

    temp.persist(path).map_err(|error| {
        anyhow::anyhow!(
            "renaming {} to {}: {}",
            error.file.path().display(),
            path.display(),
            error.error
        )
    })?;

    Ok(())
}

/// Append `line` to an existing file atomically (read-modify-write).
pub fn atomic_append_line(path: &Path, line: &str) -> Result<()> {
    let mut contents = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?
    } else {
        String::new()
    };

    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(line);
    if !line.ends_with('\n') {
        contents.push('\n');
    }

    atomic_write(path, &contents)
}
