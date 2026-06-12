//! Trash and restore semantics for brain files.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context as _, Result};

use super::paths::BrainPaths;

/// Move `source` into `brain/trash/`, preserving the filename. On collision,
/// append a unix-timestamp suffix before the extension.
pub fn trash_file(paths: &BrainPaths, source: &Path) -> Result<PathBuf> {
    if !paths.contains(source) {
        bail!(
            "refusing to trash path outside brain tree: {}",
            source.display()
        );
    }
    if !source.exists() {
        bail!("cannot trash missing file: {}", source.display());
    }

    let filename = source
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid trash source filename: {}", source.display()))?;

    let trash_dir = paths.trash_dir();
    fs::create_dir_all(&trash_dir)
        .with_context(|| format!("creating trash dir {}", trash_dir.display()))?;

    let mut destination = trash_dir.join(filename);
    if destination.exists() {
        let stem = source
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("file");
        let extension = source
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_default();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        destination = trash_dir.join(format!("{stem}-{ts}{extension}"));
    }

    fs::rename(source, &destination).with_context(|| {
        format!(
            "moving {} to trash at {}",
            source.display(),
            destination.display()
        )
    })?;

    Ok(destination)
}

/// Move a file from `brain/trash/` back to `destination`.
pub fn restore_file(paths: &BrainPaths, trashed: &Path, destination: &Path) -> Result<()> {
    let trash_dir = paths.trash_dir();
    if !trashed.starts_with(&trash_dir) {
        bail!(
            "restore source must live in trash dir: {}",
            trashed.display()
        );
    }
    if !trashed.exists() {
        bail!("cannot restore missing trash entry: {}", trashed.display());
    }
    if destination.exists() {
        bail!(
            "restore destination already exists: {}",
            destination.display()
        );
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating restore dir {}", parent.display()))?;
    }

    fs::rename(trashed, destination).with_context(|| {
        format!(
            "restoring {} to {}",
            trashed.display(),
            destination.display()
        )
    })?;

    Ok(())
}
