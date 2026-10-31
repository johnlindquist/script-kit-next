//! File mutations that never replace something already there.
//!
//! Rename, Move and Copy to all end here. A name that is taken is refused with
//! [`FileOpError::Exists`] instead of silently replacing the other item; the caller decides
//! whether to keep both (a free "name 2" name) or replace (the other item goes to the Trash
//! first, so it can come back). A move to another volume copies, then removes the original.

use std::fmt;
use std::path::{Path, PathBuf};

/// Why a file operation stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOpError {
    /// Something named `name` is already in `folder`.
    Exists { name: String, folder: String },
    /// The destination is on another volume, so a plain rename can't move it.
    CrossVolume,
    /// Anything else, already worded for the line above the input.
    Other(String),
}

impl fmt::Display for FileOpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileOpError::Exists { name, folder } => {
                write!(f, "\u{201c}{name}\u{201d} is already in {folder}")
            }
            FileOpError::CrossVolume => write!(f, "The destination is on another disk"),
            FileOpError::Other(message) => write!(f, "{message}"),
        }
    }
}

impl From<FileOpError> for String {
    fn from(error: FileOpError) -> Self {
        error.to_string()
    }
}

/// What to do when the destination name is taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    /// Stop with [`FileOpError::Exists`].
    Refuse,
    /// Use the first free "name 2", "name 3" … name instead.
    KeepBoth,
    /// Move the item that's there to the Trash first.
    Replace,
}

/// Where a move or copy put the item, and what it set aside to get there.
#[derive(Debug)]
pub struct Placed {
    pub path: PathBuf,
    pub undo_identity: FileIdentity,
    /// The item that was at the destination and went to the Trash (`OnConflict::Replace`),
    /// with where the Trash put it, so an undo can bring it back.
    pub replaced: Option<(PathBuf, PathBuf)>,
    /// The move crossed volumes, so it was a copy followed by removing the original.
    pub across_volumes: bool,
}

/// An open handle keeps the item alive across the undo interval. This prevents an unlinked
/// file's inode from being recycled for a replacement, and identifies a symlink itself.
#[derive(Debug)]
pub struct FileIdentity(std::fs::File);

impl FileIdentity {
    pub fn capture(path: &Path) -> Result<Self, FileOpError> {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.custom_flags(libc::O_EVTONLY | libc::O_SYMLINK);
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.custom_flags(libc::O_PATH | libc::O_NOFOLLOW);
        }
        options
            .open(path)
            .map(Self)
            .map_err(|e| FileOpError::Other(format!("Couldn't identify the item: {e}")))
    }

    fn matches(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            match (self.0.metadata(), std::fs::symlink_metadata(path)) {
                (Ok(expected), Ok(found)) => {
                    expected.dev() == found.dev() && expected.ino() == found.ino()
                }
                _ => false,
            }
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            false
        }
    }

    /// Claim the entry in a private sibling directory, then check its identity again before
    /// moving or trashing it. A concurrent swap between the initial check and rename must
    /// never authorize the inverse operation on the replacement.
    pub fn undo_at(
        &self,
        path: &Path,
        undo: impl FnOnce(&Path) -> Result<(), FileOpError>,
    ) -> Result<(), FileOpError> {
        let changed = || FileOpError::Other("The item at that location has changed".into());
        if !self.matches(path) {
            return Err(changed());
        }
        let parent = path.parent().ok_or_else(changed)?;
        let name = path.file_name().ok_or_else(changed)?;
        let staging = tempfile::Builder::new()
            .prefix(".script-kit-undo-")
            .tempdir_in(parent)
            .map_err(|e| FileOpError::Other(format!("Couldn't prepare the undo: {e}")))?;
        let held = staging.path().join(name);
        rename_no_replace(path, &held)?;
        let result = if self.matches(&held) {
            undo(&held)
        } else {
            Err(changed())
        };
        if result.is_err() && exists_no_follow(&held) {
            if let Err(e) = rename_no_replace(&held, path) {
                // Another writer can claim the now-free name. Preserve BOTH items rather
                // than letting TempDir's destructor remove the one we set aside.
                let _retained = staging.keep();
                return Err(FileOpError::Other(format!(
                    "Item preserved at {}: {e}",
                    held.display()
                )));
            }
        }
        result
    }
}

fn exists_no_follow(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn folder_label(path: &Path) -> String {
    let folder = path.parent().unwrap_or(path);
    folder
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| folder.display().to_string())
}

fn exists_error(target: &Path) -> FileOpError {
    FileOpError::Exists {
        name: target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        folder: folder_label(target),
    }
}

/// Whether two paths name the same item on disk (same device and inode).
fn same_item(a: &Path, b: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        match (std::fs::symlink_metadata(a), std::fs::symlink_metadata(b)) {
            (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
            _ => false,
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (a, b);
        false
    }
}

/// Checks a new name typed for an item and returns it trimmed.
pub fn validate_new_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Type a name".to_string());
    }
    if trimmed == "." || trimmed == ".." {
        return Err(format!("\u{201c}{trimmed}\u{201d} can't be used as a name"));
    }
    if trimmed.contains('/') {
        return Err("A name can't contain \u{201c}/\u{201d}".to_string());
    }
    if trimmed.contains('\0') {
        return Err("A name can't contain a null character".to_string());
    }
    if trimmed.len() > 255 {
        return Err("That name is too long".to_string());
    }
    Ok(trimmed.to_string())
}

/// The first free name in `folder` for `name`, the way Finder keeps both: "report.pdf" becomes
/// "report 2.pdf", then "report 3.pdf". Folders and dotfiles number the whole name.
pub fn keep_both_path(folder: &Path, name: &str) -> PathBuf {
    let candidate = folder.join(name);
    if !exists_no_follow(&candidate) {
        return candidate;
    }
    let (stem, ext) = match name.rfind('.') {
        Some(dot) if dot > 0 && !folder.join(name).is_dir() => (&name[..dot], Some(&name[dot..])),
        _ => (name, None),
    };
    for n in 2..10_000 {
        let numbered = match ext {
            Some(ext) => format!("{stem} {n}{ext}"),
            None => format!("{stem} {n}"),
        };
        let candidate = folder.join(numbered);
        if !exists_no_follow(&candidate) {
            return candidate;
        }
    }
    folder.join(format!("{stem} {}", std::process::id()))
}

/// Renames `from` to `to` unless something else is already at `to`.
///
/// A rename that only changes letter case (or otherwise names the same item) is allowed.
pub fn rename_no_replace(from: &Path, to: &Path) -> Result<(), FileOpError> {
    if from == to {
        return Ok(());
    }
    if exists_no_follow(to) && !same_item(from, to) {
        return Err(exists_error(to));
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt as _;

        if same_item(from, to) {
            // Case-only rename on a case-insensitive volume: the "existing" item is this one.
            return std::fs::rename(from, to)
                .map_err(|e| FileOpError::Other(format!("Couldn't rename it: {e}")));
        }
        let c_from = std::ffi::CString::new(from.as_os_str().as_bytes())
            .map_err(|_| FileOpError::Other("The path contains a null character".to_string()))?;
        let c_to = std::ffi::CString::new(to.as_os_str().as_bytes())
            .map_err(|_| FileOpError::Other("The path contains a null character".to_string()))?;
        // SAFETY: both pointers are valid NUL-terminated strings for the duration of the call.
        let status = unsafe { libc::renamex_np(c_from.as_ptr(), c_to.as_ptr(), libc::RENAME_EXCL) };
        if status == 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::EEXIST) => Err(exists_error(to)),
            Some(libc::EXDEV) => Err(FileOpError::CrossVolume),
            // Some volumes (older network shares, FAT) don't support RENAME_EXCL; the
            // existence check above already ran, so fall back to a plain rename there.
            Some(libc::ENOTSUP) | Some(libc::EINVAL) => {
                std::fs::rename(from, to).map_err(|e| map_rename_error(e, to))
            }
            _ => Err(map_rename_error(error, to)),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::fs::rename(from, to).map_err(|e| map_rename_error(e, to))
    }
}

fn map_rename_error(error: std::io::Error, to: &Path) -> FileOpError {
    match error.raw_os_error() {
        #[cfg(unix)]
        Some(libc::EXDEV) => FileOpError::CrossVolume,
        #[cfg(unix)]
        Some(libc::EEXIST) | Some(libc::ENOTEMPTY) => exists_error(to),
        _ => FileOpError::Other(format!("Couldn't move it: {error}")),
    }
}

/// Copies `from` (a file, folder or link) to `to` unless something is already at `to`.
/// Keeps permissions, dates, tags and other extended attributes; links are copied as links.
/// A copy that fails part way is removed.
pub fn copy_no_replace(from: &Path, to: &Path) -> Result<(), FileOpError> {
    copy_with_identity(from, to).map(|_| ())
}

fn copy_with_identity(from: &Path, to: &Path) -> Result<FileIdentity, FileOpError> {
    if exists_no_follow(to) {
        return Err(exists_error(to));
    }
    let metadata = std::fs::symlink_metadata(from)
        .map_err(|e| FileOpError::Other(format!("Couldn't read the original: {e}")))?;
    let staging = tempfile::Builder::new()
        .prefix(".script-kit-copy-")
        .tempdir_in(
            to.parent()
                .ok_or_else(|| FileOpError::Other("The destination has no parent".into()))?,
        )
        .map_err(|e| FileOpError::Other(format!("Couldn't prepare the copy: {e}")))?;
    let copy = staging.path().join("item");
    copy_tree(from, &copy, &metadata)?;
    let identity = FileIdentity::capture(&copy)?;
    rename_no_replace(&copy, to)?;
    Ok(identity)
}

#[cfg(target_os = "macos")]
fn copy_tree(from: &Path, to: &Path, metadata: &std::fs::Metadata) -> Result<(), FileOpError> {
    use std::os::unix::ffi::OsStrExt as _;

    let c_from = std::ffi::CString::new(from.as_os_str().as_bytes())
        .map_err(|_| FileOpError::Other("The path contains a null character".to_string()))?;
    let c_to = std::ffi::CString::new(to.as_os_str().as_bytes())
        .map_err(|_| FileOpError::Other("The path contains a null character".to_string()))?;
    let mut flags = libc::COPYFILE_METADATA
        | libc::COPYFILE_DATA
        | libc::COPYFILE_EXCL
        | libc::COPYFILE_NOFOLLOW_SRC;
    if metadata.is_dir() {
        flags |= libc::COPYFILE_RECURSIVE;
    } else {
        // A clone shares blocks on APFS and falls back to a real copy elsewhere.
        flags |= libc::COPYFILE_CLONE;
    }
    // SAFETY: both pointers are valid NUL-terminated strings; a null state is allowed.
    let status =
        unsafe { libc::copyfile(c_from.as_ptr(), c_to.as_ptr(), std::ptr::null_mut(), flags) };
    if status == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    Err(match error.raw_os_error() {
        Some(libc::EEXIST) => exists_error(to),
        Some(libc::ENOSPC) => {
            FileOpError::Other("There isn't enough space on that disk".to_string())
        }
        _ => FileOpError::Other(format!("Couldn't copy it: {error}")),
    })
}

#[cfg(not(target_os = "macos"))]
fn copy_tree(from: &Path, to: &Path, metadata: &std::fs::Metadata) -> Result<(), FileOpError> {
    let other = |e: std::io::Error| FileOpError::Other(format!("Couldn't copy it: {e}"));
    if metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            let target = std::fs::read_link(from).map_err(other)?;
            return std::os::unix::fs::symlink(target, to).map_err(other);
        }
        #[cfg(not(unix))]
        return Err(FileOpError::Other(
            "Copying links isn't supported here".to_string(),
        ));
    }
    if metadata.is_dir() {
        std::fs::create_dir(to).map_err(other)?;
        for entry in std::fs::read_dir(from).map_err(other)? {
            let entry = entry.map_err(other)?;
            let child = entry.path();
            let child_meta = std::fs::symlink_metadata(&child).map_err(other)?;
            copy_tree(&child, &to.join(entry.file_name()), &child_meta)?;
        }
        return Ok(());
    }
    std::fs::copy(from, to).map(|_| ()).map_err(other)
}

fn remove_original(path: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

/// Moves `from` to `to` without replacing anything. Across volumes it copies, then removes the
/// original; if the original can't be removed the copy stays and the error says so.
pub fn move_no_replace(from: &Path, to: &Path) -> Result<bool, FileOpError> {
    move_no_replace_with(from, to, rename_no_replace).map(|(across, _)| across)
}

fn move_no_replace_with(
    from: &Path,
    to: &Path,
    rename: impl Fn(&Path, &Path) -> Result<(), FileOpError>,
) -> Result<(bool, FileIdentity), FileOpError> {
    let original = FileIdentity::capture(from)?;
    match rename(from, to) {
        Ok(()) => Ok((false, original)),
        Err(FileOpError::CrossVolume) => {
            let copy = copy_with_identity(from, to)?;
            original.undo_at(from, |held| {
                remove_original(held).map_err(|e| {
                    FileOpError::Other(format!("Copied it, but couldn't remove the original: {e}"))
                })
            })?;
            Ok((true, copy))
        }
        Err(error) => Err(error),
    }
}

/// Where `item` would land in `folder` under `on_conflict`, and whether that sets aside an
/// existing item. `Refuse` errors when the name is taken.
fn destination_for(
    item: &Path,
    folder: &Path,
    on_conflict: OnConflict,
) -> Result<(PathBuf, bool), FileOpError> {
    let name = item
        .file_name()
        .ok_or_else(|| FileOpError::Other("The item has no name".to_string()))?;
    if !folder.is_dir() {
        return Err(FileOpError::Other(format!(
            "{} isn't a folder",
            folder.display()
        )));
    }
    let target = folder.join(name);
    if !exists_no_follow(&target) || same_item(item, &target) {
        return Ok((target, false));
    }
    match on_conflict {
        OnConflict::Refuse => Err(exists_error(&target)),
        OnConflict::KeepBoth => Ok((keep_both_path(folder, &name.to_string_lossy()), false)),
        OnConflict::Replace => Ok((target, true)),
    }
}

/// Whether moving or copying `item` into `folder` would meet an item with the same name.
pub fn name_taken_in(item: &Path, folder: &Path) -> bool {
    item.file_name().is_some_and(|name| {
        let target = folder.join(name);
        exists_no_follow(&target) && !same_item(item, &target)
    })
}

fn is_inside(path: &Path, ancestor: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    canon(path).starts_with(canon(ancestor))
}

/// Moves `item` into `folder`. `trash` puts a replaced item in the Trash and returns where it went.
pub fn move_into(
    item: &Path,
    folder: &Path,
    on_conflict: OnConflict,
    trash: impl Fn(&Path) -> Result<PathBuf, String>,
    restore: impl Fn(&Path, &Path) -> Result<(), String>,
) -> Result<Placed, FileOpError> {
    if item.is_dir() && is_inside(folder, item) {
        return Err(FileOpError::Other(
            "A folder can't be moved into itself".to_string(),
        ));
    }
    let (target, replaces) = destination_for(item, folder, on_conflict)?;
    if target == item {
        return Ok(Placed {
            undo_identity: FileIdentity::capture(item)?,
            path: target,
            replaced: None,
            across_volumes: false,
        });
    }
    let replaced = if replaces {
        let trashed = trash(&target).map_err(FileOpError::Other)?;
        Some((target.clone(), trashed))
    } else {
        None
    };
    match move_no_replace_with(item, &target, rename_no_replace) {
        Ok((across_volumes, undo_identity)) => Ok(Placed {
            undo_identity,
            path: target,
            replaced,
            across_volumes,
        }),
        Err(error) => {
            if let Some((original, trashed)) = &replaced {
                let _ = restore(trashed, original);
            }
            Err(error)
        }
    }
}

/// Copies `item` into `folder`. `trash` puts a replaced item in the Trash and returns where it went.
pub fn copy_into(
    item: &Path,
    folder: &Path,
    on_conflict: OnConflict,
    trash: impl Fn(&Path) -> Result<PathBuf, String>,
    restore: impl Fn(&Path, &Path) -> Result<(), String>,
) -> Result<Placed, FileOpError> {
    if item.is_dir() && is_inside(folder, item) {
        return Err(FileOpError::Other(
            "A folder can't be copied into itself".to_string(),
        ));
    }
    let (mut target, replaces) = destination_for(item, folder, on_conflict)?;
    if target == item {
        // Copying into its own folder keeps both, like Finder's duplicate.
        target = keep_both_path(
            folder,
            &item.file_name().unwrap_or_default().to_string_lossy(),
        );
    }
    let replaced = if replaces {
        let trashed = trash(&target).map_err(FileOpError::Other)?;
        Some((target.clone(), trashed))
    } else {
        None
    };
    match copy_with_identity(item, &target) {
        Ok(undo_identity) => Ok(Placed {
            undo_identity,
            path: target,
            replaced,
            across_volumes: false,
        }),
        Err(error) => {
            if let Some((original, trashed)) = &replaced {
                let _ = restore(trashed, original);
            }
            Err(error)
        }
    }
}

/// What the rename row says for the words typed so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePreview {
    pub title: String,
    pub sub: String,
    pub ready: bool,
}

pub fn rename_preview(path: &str, typed: &str) -> RenamePreview {
    let current = std::path::Path::new(path);
    let old_name = current
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = current
        .parent()
        .map(|parent| super::shorten_path(&parent.to_string_lossy()))
        .unwrap_or_default();
    let name = match validate_new_name(typed) {
        Ok(name) => name,
        Err(problem) => {
            return RenamePreview {
                title: problem,
                sub: format!("Renaming \u{201c}{old_name}\u{201d} in {folder}"),
                ready: false,
            }
        }
    };
    if name == old_name {
        return RenamePreview {
            title: format!("Type a new name for \u{201c}{old_name}\u{201d}"),
            sub: format!("In {folder}"),
            ready: false,
        };
    }
    if let Some(parent) = current.parent() {
        let target = parent.join(&name);
        let taken =
            std::fs::symlink_metadata(&target).is_ok() && !name.eq_ignore_ascii_case(&old_name);
        if taken {
            return RenamePreview {
                title: format!("\u{201c}{name}\u{201d} is already in {folder}"),
                sub: "Choose a name nothing else in this folder uses".to_string(),
                ready: false,
            };
        }
    }
    RenamePreview {
        title: format!("Rename to \u{201c}{name}\u{201d}"),
        sub: format!("\u{201c}{old_name}\u{201d} in {folder}"),
        ready: true,
    }
}

/// The part of a name Finder selects for renaming: everything before the last extension.
pub fn rename_stem_len(name: &str, is_dir: bool) -> usize {
    if is_dir {
        return name.len();
    }
    match name.rfind('.') {
        Some(dot) if dot > 0 => dot,
        _ => name.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn no_trash(_: &Path) -> Result<PathBuf, String> {
        panic!("nothing should be trashed")
    }
    fn no_restore(_: &Path, _: &Path) -> Result<(), String> {
        Ok(())
    }

    #[test]
    fn rename_refuses_a_taken_name_and_keeps_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, "A").unwrap();
        fs::write(&b, "B").unwrap();

        let error = rename_no_replace(&a, &b).unwrap_err();
        assert!(matches!(error, FileOpError::Exists { ref name, .. } if name == "b.txt"));
        assert_eq!(fs::read_to_string(&a).unwrap(), "A");
        assert_eq!(fs::read_to_string(&b).unwrap(), "B");
    }

    #[test]
    fn rename_refuses_a_taken_folder_name() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        fs::write(b.join("keep.txt"), "kept").unwrap();
        assert!(matches!(
            rename_no_replace(&a, &b),
            Err(FileOpError::Exists { .. })
        ));
        assert_eq!(fs::read_to_string(b.join("keep.txt")).unwrap(), "kept");
    }

    #[test]
    fn rename_to_a_free_name_and_case_only_rename_work() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        fs::write(&a, "A").unwrap();
        let c = dir.path().join("c.txt");
        rename_no_replace(&a, &c).unwrap();
        assert_eq!(fs::read_to_string(&c).unwrap(), "A");

        let upper = dir.path().join("C.txt");
        rename_no_replace(&c, &upper).unwrap();
        let names: Vec<String> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["C.txt".to_string()]);
    }

    #[test]
    fn a_dangling_link_still_counts_as_taken() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        fs::write(&a, "A").unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(dir.path().join("missing"), &link).unwrap();
        assert!(matches!(
            rename_no_replace(&a, &link),
            Err(FileOpError::Exists { .. })
        ));
        assert!(fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn move_into_refuses_then_keeps_both() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("src");
        let dst_dir = dir.path().join("dst");
        fs::create_dir(&src_dir).unwrap();
        fs::create_dir(&dst_dir).unwrap();
        let item = src_dir.join("report.pdf");
        fs::write(&item, "new").unwrap();
        fs::write(dst_dir.join("report.pdf"), "old").unwrap();

        let refused = move_into(&item, &dst_dir, OnConflict::Refuse, no_trash, no_restore);
        assert_eq!(
            refused.unwrap_err().to_string(),
            "\u{201c}report.pdf\u{201d} is already in dst"
        );
        assert!(item.exists());

        let placed =
            move_into(&item, &dst_dir, OnConflict::KeepBoth, no_trash, no_restore).unwrap();
        assert_eq!(placed.path, dst_dir.join("report 2.pdf"));
        assert_eq!(
            fs::read_to_string(dst_dir.join("report.pdf")).unwrap(),
            "old"
        );
        assert_eq!(fs::read_to_string(&placed.path).unwrap(), "new");
        assert!(!item.exists());
    }

    #[test]
    fn undo_never_takes_a_replacement_even_with_identical_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let moved = dir.path().join("moved");
        let parked = dir.path().join("parked");
        fs::write(&moved, "identical").unwrap();
        let identity = FileIdentity::capture(&moved).unwrap();
        fs::rename(&moved, &parked).unwrap();
        fs::copy(&parked, &moved).unwrap();
        assert!(identity.undo_at(&moved, |_| panic!("wrong file")).is_err());
        assert!(identity.matches(&parked));
        assert_eq!(fs::read(&parked).unwrap(), fs::read(&moved).unwrap());
    }

    #[test]
    fn undo_preserves_both_files_if_a_writer_claims_the_rollback_path() {
        let dir = tempfile::tempdir().unwrap();
        let moved = dir.path().join("moved");
        fs::write(&moved, "original").unwrap();
        let identity = FileIdentity::capture(&moved).unwrap();
        let mut preserved = PathBuf::new();
        let error = identity
            .undo_at(&moved, |held| {
                preserved = held.into();
                fs::write(&moved, "other writer").unwrap();
                Err(FileOpError::Other("cannot restore".into()))
            })
            .unwrap_err();
        assert!(error.to_string().contains(&*preserved.to_string_lossy()));
        assert!(identity.matches(&preserved));
        assert_eq!(fs::read_to_string(moved).unwrap(), "other writer");
    }

    #[test]
    fn move_into_replace_sets_the_old_item_aside_first() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("src");
        let dst_dir = dir.path().join("dst");
        let bin = dir.path().join("bin");
        for d in [&src_dir, &dst_dir, &bin] {
            fs::create_dir(d).unwrap();
        }
        let item = src_dir.join("notes.txt");
        fs::write(&item, "new").unwrap();
        fs::write(dst_dir.join("notes.txt"), "old").unwrap();
        let bin_for_trash = bin.clone();
        let trash = move |path: &Path| -> Result<PathBuf, String> {
            let to = bin_for_trash.join(path.file_name().unwrap());
            fs::rename(path, &to).map_err(|e| e.to_string())?;
            Ok(to)
        };
        let placed = move_into(&item, &dst_dir, OnConflict::Replace, trash, no_restore).unwrap();
        assert_eq!(fs::read_to_string(&placed.path).unwrap(), "new");
        let (original, trashed) = placed.replaced.unwrap();
        assert_eq!(original, dst_dir.join("notes.txt"));
        assert_eq!(fs::read_to_string(trashed).unwrap(), "old");
    }

    #[test]
    fn a_folder_cannot_move_into_itself() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("folder");
        let inner = folder.join("inner");
        fs::create_dir_all(&inner).unwrap();
        let error =
            move_into(&folder, &inner, OnConflict::Refuse, no_trash, no_restore).unwrap_err();
        assert_eq!(error.to_string(), "A folder can't be moved into itself");
        assert!(inner.exists());
    }

    #[test]
    fn a_move_across_volumes_copies_then_removes_the_original() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("folder");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("a.txt"), "A").unwrap();
        fs::create_dir(folder.join("sub")).unwrap();
        fs::write(folder.join("sub/b.txt"), "B").unwrap();
        let target = dir.path().join("moved");

        let (across, identity) =
            move_no_replace_with(&folder, &target, |_, _| Err(FileOpError::CrossVolume)).unwrap();
        assert!(across && identity.matches(&target));
        assert!(!folder.exists());
        assert_eq!(fs::read_to_string(target.join("sub/b.txt")).unwrap(), "B");
    }

    #[test]
    fn copy_into_keeps_attributes_and_refuses_a_taken_name() {
        let dir = tempfile::tempdir().unwrap();
        let dst_dir = dir.path().join("dst");
        fs::create_dir(&dst_dir).unwrap();
        let item = dir.path().join("a.sh");
        fs::write(&item, "echo").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&item, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let placed = copy_into(&item, &dst_dir, OnConflict::Refuse, no_trash, no_restore).unwrap();
        assert!(item.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(&placed.path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o755);
        }
        assert!(matches!(
            copy_into(&item, &dst_dir, OnConflict::Refuse, no_trash, no_restore),
            Err(FileOpError::Exists { .. })
        ));
    }

    #[test]
    fn copy_into_its_own_folder_keeps_both() {
        let dir = tempfile::tempdir().unwrap();
        let item = dir.path().join("a.txt");
        fs::write(&item, "A").unwrap();
        let placed =
            copy_into(&item, dir.path(), OnConflict::Refuse, no_trash, no_restore).unwrap();
        assert_eq!(placed.path, dir.path().join("a 2.txt"));
    }

    #[test]
    fn keep_both_numbers_like_finder() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("x.tar.gz"), "").unwrap();
        fs::write(dir.path().join(".env"), "").unwrap();
        fs::create_dir(dir.path().join("v1.2")).unwrap();
        assert_eq!(
            keep_both_path(dir.path(), "x.tar.gz"),
            dir.path().join("x.tar 2.gz")
        );
        assert_eq!(
            keep_both_path(dir.path(), ".env"),
            dir.path().join(".env 2")
        );
        assert_eq!(
            keep_both_path(dir.path(), "v1.2"),
            dir.path().join("v1.2 2")
        );
        assert_eq!(
            keep_both_path(dir.path(), "free.txt"),
            dir.path().join("free.txt")
        );
    }

    #[test]
    fn new_names_are_checked() {
        assert_eq!(validate_new_name("  b.txt ").unwrap(), "b.txt");
        assert!(validate_new_name("").is_err());
        assert!(validate_new_name("..").is_err());
        assert!(validate_new_name("a/b").is_err());
        assert!(validate_new_name(&"x".repeat(256)).is_err());
    }

    #[test]
    fn the_rename_row_says_what_will_happen() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        fs::write(&a, "").unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();
        let path = a.to_str().unwrap();

        let ready = rename_preview(path, "c.txt");
        assert!(ready.ready);
        assert_eq!(ready.title, "Rename to \u{201c}c.txt\u{201d}");

        let taken = rename_preview(path, "b.txt");
        assert!(!taken.ready);
        assert!(taken.title.contains("is already in"));

        assert!(!rename_preview(path, "a.txt").ready);
        assert!(!rename_preview(path, "").ready);
        assert!(!rename_preview(path, "x/y").ready);
        assert!(
            rename_preview(path, "A.txt").ready,
            "a case-only rename is allowed"
        );
    }

    #[test]
    fn the_name_is_selected_without_its_extension() {
        assert_eq!(
            rename_stem_len("report.final.pdf", false),
            "report.final".len()
        );
        assert_eq!(rename_stem_len(".env", false), ".env".len());
        assert_eq!(rename_stem_len("folder.v2", true), "folder.v2".len());
        assert_eq!(rename_stem_len("README", false), "README".len());
    }
}
