//! Atomic filesystem writes for the brain substrate.

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context as _, Result};

/// Process-wide serialization for every mutation of files under the brain
/// substrate. Day/note/fragment files have multiple concurrent writers (editor
/// autosave, `;todo` capture, clipboard sediment, dictation, agent traces);
/// several perform an unlocked read-modify-write. Without one lock a background
/// append can land between a save's disk read and its overwrite, silently
/// dropping the appended line.
static BRAIN_FILE_WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` while holding the process-wide brain file write lock. All mutations
/// of files under the brain substrate (writes, appends, editor saves) MUST go
/// through this so read-modify-write appends can never interleave with saves.
///
/// The lock is NOT reentrant: never call a `with_brain_write_lock`-wrapped
/// function from inside another wrapped closure. `atomic_write` deliberately
/// does not take the lock so it can be used as the write primitive inside a
/// wrapped read-modify-write scope.
pub fn with_brain_write_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = BRAIN_FILE_WRITE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f()
}

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
    temp.as_file()
        .sync_all()
        .with_context(|| format!("syncing temp file {}", temp.path().display()))?;

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

/// Append `line` to an existing file atomically (read-modify-write). The entire
/// read-modify-write runs under [`with_brain_write_lock`] so a concurrent append
/// or editor save cannot interleave between the disk read and the rewrite.
pub fn atomic_append_line(path: &Path, line: &str) -> Result<()> {
    with_brain_write_lock(|| {
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
    })
}
