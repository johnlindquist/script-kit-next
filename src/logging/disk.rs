//! The per-home log directory is app state, not an arbitrary SK_PATH. Check the
//! opened file before tightening its permissions or resetting a session log.
use std::{fs, io, path::{Path, PathBuf}};

fn refused() -> io::Error {
    io::Error::other("Log destination is not a direct private file")
}

fn direct_log(m: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: geteuid has no arguments or memory preconditions.
        m.is_file() && m.nlink() == 1 && m.uid() == unsafe { libc::geteuid() }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        m.is_file() && m.file_attributes() & 0x400 == 0 // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(not(any(unix, windows)))]
    { m.is_file() }
}

#[cfg(unix)]
fn parent(path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(path.parent().ok_or_else(refused)?)
}

#[cfg(unix)]
fn leaf(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;
    Ok(std::ffi::CString::new(path.file_name().ok_or_else(refused)?.as_bytes())?)
}

fn check_path(path: &Path) -> io::Result<Option<fs::Metadata>> {
    if !fs::symlink_metadata(path.parent().ok_or_else(refused)?)?.is_dir() {
        return Err(refused());
    }
    match fs::symlink_metadata(path) {
        Ok(m) if direct_log(&m) => Ok(Some(m)),
        Ok(_) => Err(refused()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn open_append(path: &Path) -> io::Result<fs::File> {
    check_path(path)?;
    #[cfg(unix)]
    let file = {
        use std::os::fd::{AsRawFd, FromRawFd};
        let dir = parent(path)?;
        let name = leaf(path)?;
        // Do not use O_TRUNC: a hard link is still a regular file. Create with
        // 0600 and validate the returned descriptor before the first mutation.
        let flags = libc::O_WRONLY | libc::O_APPEND | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC;
        // SAFETY: the directory and NUL-terminated basename live through openat;
        // the creation mode is passed as a promoted variadic integer.
        let fd = unsafe { libc::openat(dir.as_raw_fd(), name.as_ptr(), flags, 0o600 as libc::c_int) };
        if fd < 0 { return Err(io::Error::last_os_error()); }
        // SAFETY: take ownership of this newly opened descriptor exactly once.
        unsafe { fs::File::from_raw_fd(fd) }
    };
    #[cfg(not(unix))]
    let file = {
        let mut options = fs::OpenOptions::new();
        options.append(true).create(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
        }
        options.open(path)?
    };
    let m = file.metadata()?;
    if !direct_log(&m) { return Err(refused()); }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let named = fs::symlink_metadata(path)?;
        if !direct_log(&named) || (m.dev(), m.ino()) != (named.dev(), named.ino()) { return Err(refused()); }
        if m.mode() & 0o7777 != 0o600 { file.set_permissions(fs::Permissions::from_mode(0o600))?; }
    }
    Ok(file)
}

// A new launch must not truncate and reuse an inode still held by the previous
// writer or its panic hook. Reserve a clean private file and return that same fd.
fn replace_session(path: &Path) -> io::Result<fs::File> {
    let before = check_path(path)?;
    let dir = path.parent().ok_or_else(refused)?;
    let draft = tempfile::NamedTempFile::new_in(dir)?;
    let current = check_path(path)?;
    let file = match (before, current) {
        (None, None) => draft.persist_noclobber(path),
        (Some(old), Some(current)) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if (old.dev(), old.ino()) != (current.dev(), current.ino()) { return Err(refused()); }
            }
            #[cfg(not(unix))]
            let _ = (old, current);
            draft.persist(path)
        }
        _ => return Err(refused()),
    }.map_err(|e| e.error)?;
    Ok(file)
}

fn ensure_log_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)?;
    let named = fs::symlink_metadata(dir)?;
    if !named.is_dir() { return Err(refused()); }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: geteuid has no arguments or memory preconditions.
        if named.uid() != unsafe { libc::geteuid() } || named.mode() & 0o022 != 0 { return Err(refused()); }
    }
    Ok(())
}

pub(super) fn capture_file(dir: &Path, stamp: &str) -> io::Result<(PathBuf, fs::File)> {
    ensure_log_dir(dir)?;
    // Independent captures can begin in the same second. Never append to an
    // earlier capture or open a pre-existing name chosen by someone else.
    let (file, path) = tempfile::Builder::new().prefix(&format!("capture-{stamp}-"))
        .suffix(".jsonl").tempfile_in(dir)?.keep().map_err(|e| e.error)?;
    Ok((path, file))
}

pub(super) fn rotate_if_oversized(path: &Path, max_bytes: u64) {
    if max_bytes == 0 { return; }
    fn rotate(path: &Path, cap: u64) -> io::Result<()> {
        let dir_path = path.parent().ok_or_else(refused)?;
        if !fs::symlink_metadata(dir_path)?.is_dir() { return Err(refused()); }
        #[cfg(unix)]
        let dir = parent(path)?;
        let m = match fs::symlink_metadata(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            other => other?,
        };
        if !direct_log(&m) { return Err(refused()); }
        if m.len() <= cap { return Ok(()); }
        let mut archived_name = path.file_name().ok_or_else(refused)?.to_owned();
        archived_name.push(".1");
        let archived = dir_path.join(archived_name);
        match fs::symlink_metadata(&archived) {
            Ok(m) if !direct_log(&m) => return Err(refused()),
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            use std::os::unix::fs::MetadataExt;
            let held = dir.metadata()?;
            let named = fs::symlink_metadata(dir_path)?;
            if (held.dev(), held.ino()) != (named.dev(), named.ino()) { return Err(refused()); }
            let (from, to) = (leaf(path)?, leaf(&archived)?);
            // SAFETY: rename the two basenames inside the same open directory.
            if unsafe { libc::renameat(dir.as_raw_fd(), from.as_ptr(), dir.as_raw_fd(), to.as_ptr()) } != 0 { return Err(io::Error::last_os_error()); }
        }
        #[cfg(not(unix))]
        fs::rename(path, &archived)?;
        eprintln!("[SCRIPT-KIT-GPUI] Rotated oversized log ({} bytes) to {}",m.len(),archived.display());
        Ok(())
    }
    if let Err(e) = rotate(path, max_bytes) { eprintln!("[LOGGING] Failed to rotate {}: {e}", path.display()); }
}

/// Open the two actual startup sinks without installing a global subscriber.
/// A refused file stays untouched; the other writer and stderr remain usable.
pub(crate) fn session_files(dir: &Path, cap: u64) -> (io::Result<fs::File>, io::Result<fs::File>) {
    if let Err(error) = ensure_log_dir(dir) {
        return (Err(io::Error::new(error.kind(), error.to_string())), Err(error));
    }
    let full = dir.join("script-kit-gpui.jsonl");
    rotate_if_oversized(&full, cap);
    (open_append(&full), replace_session(&dir.join("latest-session.jsonl")))
}
