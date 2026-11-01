use crate::runtime_policy::{ExternalEffect, OwnedEvaluationPolicy};
use std::path::Path;

#[cfg(any(test, target_os = "windows"))]
fn escape_windows_cmd_open_target(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '^' | '&' | '|' | '<' | '>' | '(' | ')' | '%' | '!' | '"' => {
                escaped.push('^');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Open a file with the system default application
#[allow(dead_code)]
pub fn open_file(path: &str) -> Result<(), String> {
    use std::process::Command;

    crate::runtime_policy::check(ExternalEffect::OpenExternal).map_err(|e| e.to_string())?;

    // Brain activity journal: opening a file is a user decision the brain
    // should be able to answer questions about ("what was that png I just
    // opened?"). Single chokepoint — every open path (enter, mouse, action
    // dialog, automation) funnels through here. Off-thread, never blocks.
    crate::brain::record_activity("opened file", path);

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let escaped_path = escape_windows_cmd_open_target(path);
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&escaped_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }
}

/// Reveal a file in Finder (macOS) or file manager
#[allow(dead_code)]
pub fn reveal_in_finder(path: &str) -> Result<(), String> {
    use std::process::Command;

    crate::runtime_policy::check(ExternalEffect::OpenExternal).map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", path])
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Try to get the parent directory and open it
        let parent = Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", path])
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
        Ok(())
    }
}

// Only tests call this now that the move dialog that used it is gone.
#[allow(dead_code)]
pub(crate) fn terminal_working_directory(path: &str, is_dir: bool) -> String {
    if is_dir {
        return path.to_string();
    }

    Path::new(path)
        .parent()
        .and_then(|p| {
            let parent = p.to_string_lossy();
            if parent.is_empty() {
                None
            } else {
                Some(parent.to_string())
            }
        })
        .unwrap_or_else(|| ".".to_string())
}

/// Move a path to Trash and return where macOS put it, so the move can be
/// undone with [`restore_from_trash`].
pub fn move_to_trash(path: &str) -> Result<std::path::PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSString as _;
        use objc::{sel, sel_impl};

        unsafe fn ns_string_text(value: id) -> Option<String> {
            if value == nil {
                return None;
            }
            // SAFETY: `value` is a live NSString; UTF8String stays valid for
            // the enclosing autorelease pool.
            let utf8: *const std::os::raw::c_char = unsafe { objc::msg_send![value, UTF8String] };
            (!utf8.is_null()).then(|| {
                // SAFETY: UTF8String returns a NUL-terminated C string.
                unsafe { std::ffi::CStr::from_ptr(utf8) }
                    .to_string_lossy()
                    .into_owned()
            })
        }

        objc::rc::autoreleasepool(|| {
            // SAFETY: every object is created and used within this pool, the
            // NSString is released after use, and out-pointers are valid locals.
            unsafe {
                let ns_path = cocoa::foundation::NSString::alloc(nil).init_str(path);
                if ns_path == nil {
                    return Err("Could not read the path".to_string());
                }
                let url: id = objc::msg_send![objc::class!(NSURL), fileURLWithPath: ns_path];
                let manager: id = objc::msg_send![objc::class!(NSFileManager), defaultManager];
                let mut resulting: id = nil;
                let mut error: id = nil;
                let moved: objc::runtime::BOOL = objc::msg_send![
                    manager,
                    trashItemAtURL: url
                    resultingItemURL: &mut resulting
                    error: &mut error
                ];
                let _: () = objc::msg_send![ns_path, release];
                if moved == objc::runtime::NO {
                    let description: id = if error == nil {
                        nil
                    } else {
                        objc::msg_send![error, localizedDescription]
                    };
                    return Err(ns_string_text(description).unwrap_or_else(|| {
                        "The item could not be moved to the Trash".to_string()
                    }));
                }
                let trashed: id = if resulting == nil {
                    nil
                } else {
                    objc::msg_send![resulting, path]
                };
                ns_string_text(trashed)
                    .filter(|trashed| !trashed.is_empty())
                    .map(std::path::PathBuf::from)
                    .ok_or_else(|| "macOS did not report where the item went".to_string())
            }
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Move to Trash is currently only supported on macOS".to_string())
    }
}

/// Move an item [`move_to_trash`] trashed back to its original path. Never
/// replaces anything: if the original path is taken again, the item stays in
/// the Trash and the error says why.
pub fn restore_from_trash(trashed: &Path, original: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt as _;

        let from = std::ffi::CString::new(trashed.as_os_str().as_bytes())
            .map_err(|_| "Trash path contains a NUL byte".to_string())?;
        let to = std::ffi::CString::new(original.as_os_str().as_bytes())
            .map_err(|_| "Original path contains a NUL byte".to_string())?;
        // SAFETY: both pointers are valid NUL-terminated strings for the call.
        let status = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
        if status == 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        Err(match error.kind() {
            std::io::ErrorKind::AlreadyExists => format!(
                "Something named \"{}\" is already there",
                original
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| original.display().to_string())
            ),
            std::io::ErrorKind::NotFound => "It is no longer in the Trash".to_string(),
            _ => error.to_string(),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (trashed, original);
        Err("Restoring from Trash is currently only supported on macOS".to_string())
    }
}

/// Preview a file using Quick Look (macOS)
#[allow(dead_code)]
pub fn quick_look(path: &str) -> Result<(), String> {
    use std::process::Command;

    crate::runtime_policy::check(ExternalEffect::OpenExternal).map_err(|e| e.to_string())?;

    if !Path::new(path).exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("qlmanage")
            .arg("-p")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to preview file: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Quick Look is macOS-only; fall back to opening the file
        open_file(path)
    }
}

/// An app that can open a file, for Open With.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWithApp {
    pub name: String,
    pub path: String,
    /// The app macOS opens this file with by default.
    pub is_default: bool,
}

/// The apps macOS says can open `path`, the default one first, then by name.
pub fn apps_for_open_with(path: &str) -> Result<Vec<OpenWithApp>, String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSString as _;
        use objc::{sel, sel_impl};

        unsafe fn url_path(url: id) -> Option<String> {
            if url == nil {
                return None;
            }
            // SAFETY: `url` is a live NSURL; `path` returns an autoreleased NSString or nil.
            let ns: id = unsafe { objc::msg_send![url, path] };
            if ns == nil {
                return None;
            }
            let utf8: *const std::os::raw::c_char = unsafe { objc::msg_send![ns, UTF8String] };
            (!utf8.is_null()).then(|| {
                // SAFETY: UTF8String returns a NUL-terminated string valid in this pool.
                unsafe { std::ffi::CStr::from_ptr(utf8) }
                    .to_string_lossy()
                    .into_owned()
            })
        }

        if std::fs::symlink_metadata(path).is_err() {
            return Err("The file isn't there anymore".to_string());
        }
        let (default_app, all_apps) = objc::rc::autoreleasepool(|| {
            // SAFETY: every object is created and used inside this pool; the NSString we
            // allocate is released, and nil results are checked before use.
            unsafe {
                let ns_path = cocoa::foundation::NSString::alloc(nil).init_str(path);
                if ns_path == nil {
                    return (None, Vec::new());
                }
                let url: id = objc::msg_send![objc::class!(NSURL), fileURLWithPath: ns_path];
                let _: () = objc::msg_send![ns_path, release];
                let workspace: id = objc::msg_send![objc::class!(NSWorkspace), sharedWorkspace];
                if url == nil || workspace == nil {
                    return (None, Vec::new());
                }
                let default_url: id = objc::msg_send![workspace, URLForApplicationToOpenURL: url];
                let default_app = url_path(default_url);
                let responds: objc::runtime::BOOL = objc::msg_send![
                    workspace,
                    respondsToSelector: sel!(URLsForApplicationsToOpenURL:)
                ];
                let mut all = Vec::new();
                if responds != objc::runtime::NO {
                    let urls: id = objc::msg_send![workspace, URLsForApplicationsToOpenURL: url];
                    if urls != nil {
                        let count: usize = objc::msg_send![urls, count];
                        for i in 0..count {
                            let app: id = objc::msg_send![urls, objectAtIndex: i];
                            if let Some(app_path) = url_path(app) {
                                all.push(app_path);
                            }
                        }
                    }
                }
                (default_app, all)
            }
        });
        Ok(order_open_with_apps(default_app, all_apps))
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Open With is only supported on macOS".to_string())
    }
}

fn app_display_name(app_path: &str) -> String {
    Path::new(app_path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| app_path.to_string())
}

/// Default app first, then the rest by name, each app once.
pub(crate) fn order_open_with_apps(
    default_app: Option<String>,
    apps: Vec<String>,
) -> Vec<OpenWithApp> {
    let mut rest: Vec<OpenWithApp> = Vec::new();
    for app in apps {
        if Some(&app) == default_app.as_ref() || rest.iter().any(|known| known.path == app) {
            continue;
        }
        rest.push(OpenWithApp {
            name: app_display_name(&app),
            path: app,
            is_default: false,
        });
    }
    rest.sort_by_key(|app| app.name.to_lowercase());
    let mut ordered = Vec::with_capacity(rest.len() + 1);
    if let Some(default_app) = default_app {
        ordered.push(OpenWithApp {
            name: app_display_name(&default_app),
            path: default_app,
            is_default: true,
        });
    }
    ordered.extend(rest);
    ordered
}

/// Opens `path` with the app at `app_path`.
pub fn open_with(path: &str, app_path: &str) -> Result<(), String> {
    crate::runtime_policy::check(ExternalEffect::OpenExternal).map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .arg("-a")
            .arg(app_path)
            .arg(path)
            .status()
            .map_err(|e| format!("Couldn't start it: {e}"))?;
        if !status.success() {
            return Err(format!("{} couldn't open it", app_display_name(app_path)));
        }
        crate::brain::record_activity("opened file", path);
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (path, app_path);
        Err("Open With is only supported on macOS".to_string())
    }
}

/// Show the Get Info window for a file in Finder (macOS)
#[allow(dead_code)]
pub fn show_info(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to open the Get Info window
        let script = format!(
            r#"tell application "Finder"
                activate
                set theFile to POSIX file "{}"
                open information window of theFile
            end tell"#,
            crate::utils::escape_applescript_string(path)
        );
        crate::platform::run_osascript(&script, "file_search_show_info")
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Show Info is only supported on macOS".to_string())
    }
}

/// Rename a file or directory in-place and return the new full path.
pub fn rename_path(path: &str, new_name: &str) -> Result<String, String> {
    rename_path_with_policy(path, new_name, crate::runtime_policy::owned_evaluation())
}

fn rename_path_with_policy(
    path: &str,
    new_name: &str,
    policy: Option<&OwnedEvaluationPolicy>,
) -> Result<String, String> {
    let trimmed_name = new_name.trim();
    if trimmed_name.is_empty() {
        return Err("New name cannot be empty".to_string());
    }
    if trimmed_name.contains('/') {
        return Err("New name cannot contain '/'".to_string());
    }

    let current_path = Path::new(path);
    let parent = current_path
        .parent()
        .ok_or_else(|| "Cannot rename a root path".to_string())?;
    let target = parent.join(trimmed_name);
    if let Some(policy) = policy {
        validate_owned_tree(policy, current_path, &target)?;
    }
    super::file_ops::validate_new_name(trimmed_name)?;

    if target == current_path {
        return Ok(path.to_string());
    }

    super::file_ops::rename_no_replace(current_path, &target)?;

    Ok(target.to_string_lossy().to_string())
}

#[cfg(test)]
fn move_path_with_policy(
    path: &str,
    destination_dir: &str,
    policy: Option<&OwnedEvaluationPolicy>,
) -> Result<String, String> {
    place_path_with_policy(
        PlaceKind::Move,
        path,
        destination_dir,
        super::file_ops::OnConflict::Refuse,
        policy,
    )
    .map(|placed| placed.path.to_string_lossy().to_string())
    .map_err(String::from)
}

/// Whether a move or a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceKind {
    Move,
    Copy,
}

/// A rename being typed in the main input: the input holds the new name while File Search
/// keeps its list, and ↵ renames or Esc puts the search back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRenameEdit {
    pub path: String,
    pub previous_display_index: usize,
    /// The name typed so far.
    pub typed: String,
}

/// File Search choosing a folder for Move to… or Copy to…: ↵ on a folder (or ⌘↵ for the folder
/// being shown) puts the item there; Esc goes back to the search it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePlacePick {
    pub kind: PlaceKind,
    pub path: String,
    pub origin_query: String,
    pub previous_display_index: usize,
}

/// A move or copy waiting for an answer because its name is taken in the destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPlace {
    pub kind: PlaceKind,
    pub path: String,
    pub destination_dir: String,
    pub previous_display_index: usize,
}

/// Moves or copies `path` into `destination_dir`, resolving a taken name with `on_conflict`.
/// A replaced item goes to the Trash first and is reported in [`super::file_ops::Placed`].
pub fn place_path(
    kind: PlaceKind,
    path: &str,
    destination_dir: &str,
    on_conflict: super::file_ops::OnConflict,
) -> Result<super::file_ops::Placed, super::file_ops::FileOpError> {
    place_path_with_policy(
        kind,
        path,
        destination_dir,
        on_conflict,
        crate::runtime_policy::owned_evaluation(),
    )
}

fn place_path_with_policy(
    kind: PlaceKind,
    path: &str,
    destination_dir: &str,
    on_conflict: super::file_ops::OnConflict,
    policy: Option<&OwnedEvaluationPolicy>,
) -> Result<super::file_ops::Placed, super::file_ops::FileOpError> {
    use super::file_ops::FileOpError;

    let current_path = Path::new(path);
    let filename = current_path
        .file_name()
        .ok_or_else(|| FileOpError::Other("Selected item has no filename".to_string()))?;

    let expanded_destination = crate::file_search::expand_path(destination_dir)
        .unwrap_or_else(|| destination_dir.to_string());
    let destination_path = Path::new(&expanded_destination);
    require_owned_paths(policy, &[current_path, destination_path]).map_err(FileOpError::Other)?;

    if !destination_path.is_dir() {
        return Err(FileOpError::Other(format!(
            "Destination is not a folder: {}",
            destination_path.display()
        )));
    }

    if let Some(policy) = policy {
        validate_owned_tree(policy, current_path, &destination_path.join(filename))
            .map_err(FileOpError::Other)?;
        if on_conflict == super::file_ops::OnConflict::Replace {
            // Replacing needs the Trash, which is outside an owned evaluation.
            return Err(FileOpError::Other(
                "Replacing isn't available while evaluating".to_string(),
            ));
        }
    }

    let trash = |p: &Path| move_to_trash(&p.to_string_lossy());
    let restore = |trashed: &Path, original: &Path| restore_from_trash(trashed, original);
    match kind {
        PlaceKind::Move => {
            super::file_ops::move_into(current_path, destination_path, on_conflict, trash, restore)
        }
        PlaceKind::Copy => {
            super::file_ops::copy_into(current_path, destination_path, on_conflict, trash, restore)
        }
    }
}

fn require_owned_paths(
    policy: Option<&OwnedEvaluationPolicy>,
    paths: &[&Path],
) -> Result<(), String> {
    if let Some(policy) = policy {
        for path in paths {
            policy.require_owned_path(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// Preflight the entire tree before the first mutation. Never follow symlinks,
// including links below a directory being renamed or moved to a new parent.
fn validate_owned_tree(
    policy: &OwnedEvaluationPolicy,
    src: &Path,
    dst: &Path,
) -> Result<(), String> {
    require_owned_paths(Some(policy), &[src, dst])?;
    let metadata = std::fs::symlink_metadata(src)
        .map_err(|e| format!("Failed to inspect source '{}': {}", src.display(), e))?;
    if metadata.is_dir() {
        for entry in
            std::fs::read_dir(src).map_err(|e| format!("Failed to read source folder: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read folder entry: {}", e))?;
            validate_owned_tree(policy, &entry.path(), &dst.join(entry.file_name()))?;
        }
    } else if !metadata.is_file() {
        return Err(format!("Unsupported owned file type: {}", src.display()));
    }
    Ok(())
}

fn duplicate_target_path(
    path: &Path,
    policy: Option<&OwnedEvaluationPolicy>,
) -> Result<std::path::PathBuf, String> {
    require_owned_paths(policy, &[path])?;
    let parent = path
        .parent()
        .ok_or_else(|| "Cannot duplicate a root path".to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Selected item has no filename".to_string())?;
    let is_dir = path.is_dir();
    let (base, ext) = if is_dir {
        (file_name.to_string(), None)
    } else {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name)
            .to_string();
        (stem, ext)
    };
    for index in 1..=999 {
        let candidate_name = match (&ext, index) {
            (Some(ext), 1) => format!("{} copy.{}", base, ext),
            (Some(ext), n) => format!("{} copy {}.{}", base, n, ext),
            (None, 1) => format!("{} copy", base),
            (None, n) => format!("{} copy {}", base, n),
        };
        let candidate = parent.join(candidate_name);
        require_owned_paths(policy, &[&candidate])?;
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Could not find an available duplicate name".to_string())
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    policy: Option<&OwnedEvaluationPolicy>,
) -> Result<(), String> {
    require_owned_paths(policy, &[src, dst])?;
    std::fs::create_dir(dst).map_err(|e| format!("Failed to create duplicate folder: {}", e))?;
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("Failed to read source folder: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read folder entry: {}", e))?;
        let entry_path = entry.path();
        let target_path = dst.join(entry.file_name());
        require_owned_paths(policy, &[&entry_path, &target_path])?;
        let metadata = std::fs::symlink_metadata(&entry_path)
            .map_err(|e| format!("Failed to inspect folder entry: {}", e))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            copy_symlink(&entry_path, &target_path)?;
        } else if file_type.is_dir() {
            copy_dir_recursive(&entry_path, &target_path, policy)?;
        } else {
            std::fs::copy(&entry_path, &target_path)
                .map_err(|e| format!("Failed to copy '{}': {}", entry_path.display(), e))?;
        }
    }
    Ok(())
}

#[cfg(target_family = "unix")]
fn copy_symlink(src: &Path, dst: &Path) -> Result<(), String> {
    let target = std::fs::read_link(src)
        .map_err(|e| format!("Failed to read symlink '{}': {}", src.display(), e))?;
    std::os::unix::fs::symlink(&target, dst)
        .map_err(|e| format!("Failed to duplicate symlink '{}': {}", src.display(), e))
}

#[cfg(not(target_family = "unix"))]
fn copy_symlink(src: &Path, _dst: &Path) -> Result<(), String> {
    Err(format!(
        "Duplicating symlinks is not currently supported on this platform: {}",
        src.display()
    ))
}

/// Duplicate a file or directory and return the new path.
pub fn duplicate_path(path: &str) -> Result<String, String> {
    duplicate_path_with_policy(path, crate::runtime_policy::owned_evaluation())
}

fn duplicate_path_with_policy(
    path: &str,
    policy: Option<&OwnedEvaluationPolicy>,
) -> Result<String, String> {
    let src = Path::new(path);
    require_owned_paths(policy, &[src])?;
    if !src.exists() {
        return Err(format!("Path does not exist: {}", src.display()));
    }
    let target = duplicate_target_path(src, policy)?;
    if let Some(policy) = policy {
        validate_owned_tree(policy, src, &target)?;
    }
    if src.is_dir() {
        copy_dir_recursive(src, &target, policy)?;
    } else {
        std::fs::copy(src, &target).map_err(|e| format!("Failed to duplicate item: {}", e))?;
    }
    Ok(target.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        duplicate_path, duplicate_path_with_policy, escape_windows_cmd_open_target,
        move_path_with_policy, quick_look, rename_path_with_policy, terminal_working_directory,
        OwnedEvaluationPolicy,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn owned_fixture() -> (tempfile::TempDir, OwnedEvaluationPolicy) {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("owned");
        fs::create_dir(&root).expect("owned root");
        let policy = OwnedEvaluationPolicy::new(
            &root.canonicalize().expect("canonical root"),
            "file-actions".to_string(),
            "test-generation".to_string(),
        )
        .expect("owned policy");
        (temp, policy)
    }

    #[test]
    fn owned_file_actions_preserve_contents_and_duplicate_numbering() {
        let (_temp, policy) = owned_fixture();
        let source = policy.root().join("note.txt");
        let destination = policy.root().join("destination");
        fs::write(&source, "owned contents").unwrap();
        fs::create_dir(&destination).unwrap();
        let duplicate =
            duplicate_path_with_policy(source.to_str().unwrap(), Some(&policy)).unwrap();
        assert_eq!(
            PathBuf::from(&duplicate),
            policy.root().join("note copy.txt")
        );
        let second = duplicate_path_with_policy(source.to_str().unwrap(), Some(&policy)).unwrap();
        assert_eq!(PathBuf::from(second), policy.root().join("note copy 2.txt"));
        let renamed = rename_path_with_policy(&duplicate, "renamed.txt", Some(&policy)).unwrap();
        assert!(!PathBuf::from(duplicate).exists());
        let moved =
            move_path_with_policy(&renamed, destination.to_str().unwrap(), Some(&policy)).unwrap();
        assert!(!PathBuf::from(renamed).exists());
        assert_eq!(fs::read_to_string(&source).unwrap(), "owned contents");
        assert_eq!(fs::read_to_string(&moved).unwrap(), "owned contents");
        assert_eq!(
            rename_path_with_policy(&moved, "renamed.txt", Some(&policy)).unwrap(),
            moved
        );
        assert_eq!(
            move_path_with_policy(&moved, destination.to_str().unwrap(), Some(&policy)).unwrap(),
            moved
        );
    }

    #[test]
    fn owned_directory_actions_preserve_nested_contents() {
        let (_temp, policy) = owned_fixture();
        let source = policy.root().join("folder");
        let destination = policy.root().join("destination");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(source.join("nested/note.txt"), "nested contents").unwrap();
        let duplicate =
            duplicate_path_with_policy(source.to_str().unwrap(), Some(&policy)).unwrap();
        let renamed = rename_path_with_policy(&duplicate, "renamed", Some(&policy)).unwrap();
        let moved =
            move_path_with_policy(&renamed, destination.to_str().unwrap(), Some(&policy)).unwrap();
        assert_eq!(
            fs::read_to_string(PathBuf::from(moved).join("nested/note.txt")).unwrap(),
            "nested contents"
        );
        assert_eq!(
            fs::read_to_string(source.join("nested/note.txt")).unwrap(),
            "nested contents"
        );
    }

    #[test]
    fn owned_actions_reject_outside_sources_before_noop_or_existence_checks() {
        let (temp, policy) = owned_fixture();
        let outside = temp.path().canonicalize().unwrap().join("operator.txt");
        fs::write(&outside, "operator contents").unwrap();
        for source in [&outside, &outside.with_file_name("missing.txt")] {
            let path = source.to_str().unwrap();
            assert_eq!(
                duplicate_path_with_policy(path, Some(&policy)).unwrap_err(),
                "evaluation_path_outside_owner"
            );
            assert_eq!(
                rename_path_with_policy(
                    path,
                    source.file_name().unwrap().to_str().unwrap(),
                    Some(&policy),
                )
                .unwrap_err(),
                "evaluation_path_outside_owner"
            );
            assert_eq!(
                move_path_with_policy(
                    path,
                    source.parent().unwrap().to_str().unwrap(),
                    Some(&policy)
                )
                .unwrap_err(),
                "evaluation_path_outside_owner"
            );
        }
        assert_eq!(fs::read_to_string(&outside).unwrap(), "operator contents");
        assert!(!outside.with_file_name("operator copy.txt").exists());
    }

    #[test]
    fn owned_actions_reject_outside_or_parent_destinations_without_mutation() {
        let (temp, policy) = owned_fixture();
        let source = policy.root().join("note.txt");
        fs::write(&source, "owned contents").unwrap();
        let outside = temp
            .path()
            .canonicalize()
            .unwrap()
            .join("missing-destination");
        assert_eq!(
            move_path_with_policy(
                source.to_str().unwrap(),
                outside.to_str().unwrap(),
                Some(&policy)
            )
            .unwrap_err(),
            "evaluation_path_outside_owner"
        );
        let parent_escape = policy.root().join("..");
        assert_eq!(
            move_path_with_policy(
                source.to_str().unwrap(),
                parent_escape.to_str().unwrap(),
                Some(&policy),
            )
            .unwrap_err(),
            "evaluation_path_not_normalized"
        );
        assert_eq!(
            rename_path_with_policy(source.to_str().unwrap(), "..", Some(&policy)).unwrap_err(),
            "evaluation_path_not_normalized"
        );
        assert_eq!(fs::read_to_string(&source).unwrap(), "owned contents");
        assert!(!outside.exists());
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn owned_actions_reject_symlink_sources_and_ancestors() {
        let (temp, policy) = owned_fixture();
        let outside = temp.path().canonicalize().unwrap().join("operator");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("note.txt"), "operator contents").unwrap();
        let directory_link = policy.root().join("directory-link");
        let file_link = policy.root().join("file-link");
        let dangling_link = policy.root().join("dangling-link");
        std::os::unix::fs::symlink(&outside, &directory_link).unwrap();
        std::os::unix::fs::symlink(outside.join("note.txt"), &file_link).unwrap();
        std::os::unix::fs::symlink(outside.join("missing"), &dangling_link).unwrap();
        for source in [
            directory_link.join("note.txt"),
            directory_link,
            file_link,
            dangling_link,
        ] {
            let path = source.to_str().unwrap();
            assert_eq!(
                duplicate_path_with_policy(path, Some(&policy)).unwrap_err(),
                "evaluation_path_symlink"
            );
            assert_eq!(
                rename_path_with_policy(path, "renamed", Some(&policy)).unwrap_err(),
                "evaluation_path_symlink"
            );
            assert_eq!(
                move_path_with_policy(path, policy.root().to_str().unwrap(), Some(&policy))
                    .unwrap_err(),
                "evaluation_path_symlink"
            );
        }
        assert_eq!(
            fs::read_to_string(outside.join("note.txt")).unwrap(),
            "operator contents"
        );
        assert!(!policy.root().join("renamed").exists());
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn owned_tree_preflight_rejects_nested_symlink_before_creating_or_moving_anything() {
        let (temp, policy) = owned_fixture();
        let outside = temp.path().canonicalize().unwrap().join("operator.txt");
        fs::write(&outside, "operator contents").unwrap();
        let source = policy.root().join("folder");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("safe.txt"), "owned contents").unwrap();
        std::os::unix::fs::symlink(&outside, source.join("nested/link")).unwrap();
        let destination = policy.root().join("destination");
        fs::create_dir(&destination).unwrap();
        let path = source.to_str().unwrap();
        assert_eq!(
            duplicate_path_with_policy(path, Some(&policy)).unwrap_err(),
            "evaluation_path_symlink"
        );
        assert_eq!(
            rename_path_with_policy(path, "renamed", Some(&policy)).unwrap_err(),
            "evaluation_path_symlink"
        );
        assert_eq!(
            move_path_with_policy(path, destination.to_str().unwrap(), Some(&policy)).unwrap_err(),
            "evaluation_path_symlink"
        );
        assert!(!policy.root().join("folder copy").exists());
        assert!(!policy.root().join("renamed").exists());
        assert!(!destination.join("folder").exists());
        assert_eq!(
            fs::read_to_string(source.join("safe.txt")).unwrap(),
            "owned contents"
        );
        assert_eq!(fs::read_to_string(&outside).unwrap(), "operator contents");
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn owned_actions_reject_symlink_destinations_including_dangling_duplicate_names() {
        let (temp, policy) = owned_fixture();
        let outside = temp.path().canonicalize().unwrap().join("operator");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("note.txt"), "operator contents").unwrap();
        let source = policy.root().join("note.txt");
        fs::write(&source, "owned contents").unwrap();
        let destination = policy.root().join("destination");
        fs::create_dir(&destination).unwrap();
        std::os::unix::fs::symlink(
            outside.join("missing.txt"),
            policy.root().join("note copy.txt"),
        )
        .unwrap();
        std::os::unix::fs::symlink(outside.join("note.txt"), policy.root().join("renamed.txt"))
            .unwrap();
        std::os::unix::fs::symlink(outside.join("note.txt"), destination.join("note.txt")).unwrap();
        let directory_link = policy.root().join("directory-link");
        std::os::unix::fs::symlink(&outside, &directory_link).unwrap();
        let path = source.to_str().unwrap();
        assert_eq!(
            duplicate_path_with_policy(path, Some(&policy)).unwrap_err(),
            "evaluation_path_symlink"
        );
        assert_eq!(
            rename_path_with_policy(path, "renamed.txt", Some(&policy)).unwrap_err(),
            "evaluation_path_symlink"
        );
        for directory in [&destination, &directory_link] {
            assert_eq!(
                move_path_with_policy(path, directory.to_str().unwrap(), Some(&policy))
                    .unwrap_err(),
                "evaluation_path_symlink"
            );
        }
        assert!(!outside.join("missing.txt").exists());
        assert_eq!(
            fs::read_to_string(outside.join("note.txt")).unwrap(),
            "operator contents"
        );
        assert_eq!(fs::read_to_string(&source).unwrap(), "owned contents");
    }

    #[test]
    fn open_with_lists_the_default_app_first_then_by_name_once_each() {
        let apps = super::order_open_with_apps(
            Some("/Applications/Preview.app".to_string()),
            vec![
                "/Applications/Safari.app".to_string(),
                "/Applications/Preview.app".to_string(),
                "/System/Applications/Affinity.app".to_string(),
                "/Applications/Safari.app".to_string(),
            ],
        );
        let names: Vec<(&str, bool)> = apps
            .iter()
            .map(|a| (a.name.as_str(), a.is_default))
            .collect();
        assert_eq!(
            names,
            vec![("Preview", true), ("Affinity", false), ("Safari", false)]
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn open_with_asks_launch_services_for_real_apps() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("note.txt");
        fs::write(&file, "hello").unwrap();
        let apps = super::apps_for_open_with(file.to_str().unwrap()).expect("app list");
        assert!(!apps.is_empty(), "macOS always has an app for .txt");
        assert!(apps[0].is_default, "the default app comes first");
        assert_eq!(apps.iter().filter(|app| app.is_default).count(), 1);
        assert!(apps.iter().all(|app| app.path.ends_with(".app")));
        assert!(super::apps_for_open_with("/definitely/missing.txt").is_err());
    }

    #[test]
    fn rename_and_move_never_replace_an_existing_item() {
        let temp = tempdir().expect("tempdir");
        let a = temp.path().join("a.txt");
        let b = temp.path().join("b.txt");
        let folder = temp.path().join("folder");
        fs::write(&a, "A").unwrap();
        fs::write(&b, "B").unwrap();
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("a.txt"), "other A").unwrap();

        let renamed = rename_path_with_policy(a.to_str().unwrap(), "b.txt", None);
        assert!(renamed.unwrap_err().contains("already in"));
        assert_eq!(fs::read_to_string(&b).unwrap(), "B");

        let moved = move_path_with_policy(a.to_str().unwrap(), folder.to_str().unwrap(), None);
        assert!(moved.unwrap_err().contains("already in"));
        assert_eq!(fs::read_to_string(folder.join("a.txt")).unwrap(), "other A");
        assert_eq!(fs::read_to_string(&a).unwrap(), "A");
    }

    #[test]
    fn quick_look_missing_path_returns_error_without_panic() {
        let missing = format!(
            "/tmp/script-kit-gpui-missing-quick-look-{}",
            std::process::id()
        );

        let error = quick_look(&missing).expect_err("missing path should return an error");
        assert!(error.contains("Path does not exist"));
    }

    #[test]
    fn test_terminal_working_directory_returns_parent_for_file_paths() {
        let resolved = terminal_working_directory("/tmp/a/b/file.txt", false);
        assert_eq!(resolved, "/tmp/a/b");
    }

    #[test]
    fn test_escape_windows_cmd_open_target_escapes_shell_metacharacters() {
        let escaped = escape_windows_cmd_open_target(r#"C:\tmp\a&b|c<d>e(f)g^h%i!j"k.txt"#);
        assert_eq!(escaped, r#"C:\tmp\a^&b^|c^<d^>e^(f^)g^^h^%i^!j^"k.txt"#);
    }

    #[test]
    fn test_duplicate_path_copies_regular_file() {
        let dir = tempdir().expect("tempdir should be created");
        let source = dir.path().join("note.txt");
        std::fs::write(&source, "hello").expect("source file should be written");

        let duplicated = duplicate_path(source.to_str().expect("utf8 path")).expect("duplicate");
        let duplicated_path = std::path::PathBuf::from(duplicated);

        assert!(duplicated_path.exists());
        assert_eq!(
            std::fs::read_to_string(duplicated_path).expect("duplicate file should be readable"),
            "hello"
        );
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_duplicate_path_preserves_directory_symlinks() {
        let dir = tempdir().expect("tempdir should be created");
        let source = dir.path().join("folder");
        let nested = source.join("nested");
        std::fs::create_dir_all(&nested).expect("source directory should be created");

        let target_dir = dir.path().join("linked-target");
        std::fs::create_dir(&target_dir).expect("symlink target dir should be created");
        let symlink_path = source.join("link");
        std::os::unix::fs::symlink(&target_dir, &symlink_path)
            .expect("directory symlink should be created");

        let duplicated = duplicate_path(source.to_str().expect("utf8 path")).expect("duplicate");
        let duplicated_link = std::path::PathBuf::from(duplicated).join("link");
        let metadata =
            std::fs::symlink_metadata(&duplicated_link).expect("duplicate symlink metadata");

        assert!(metadata.file_type().is_symlink());
        assert_eq!(
            std::fs::read_link(&duplicated_link).expect("duplicate symlink target"),
            target_dir
        );
    }
}
