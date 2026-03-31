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

fn move_destination_default_directory(path: &str, is_dir: bool) -> String {
    if is_dir {
        return Path::new(path)
            .parent()
            .and_then(|p| {
                let parent = p.to_string_lossy();
                if parent.is_empty() {
                    None
                } else {
                    Some(parent.to_string())
                }
            })
            .unwrap_or_else(|| ".".to_string());
    }

    terminal_working_directory(path, false)
}

/// Open a terminal window at the target path.
///
/// Returns the resolved working directory used to launch the terminal.
pub fn open_in_terminal(path: &str, is_dir: bool) -> Result<String, String> {
    let dir_path = terminal_working_directory(path, is_dir);

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let escaped_dir_path = crate::utils::escape_applescript_string(&dir_path);
        let script = format!(
            r#"tell application "Terminal"
                do script "cd " & quoted form of "{}"
                activate
            end tell"#,
            escaped_dir_path
        );

        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
        Ok(dir_path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        let _ = is_dir;
        Err("Open in Terminal is currently only supported on macOS".to_string())
    }
}

/// Move a path to Trash.
pub fn move_to_trash(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let escaped_path = crate::utils::escape_applescript_string(path);
        let script = format!(
            r#"tell application "Finder"
                delete POSIX file "{}"
            end tell"#,
            escaped_path
        );

        let mut child = Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to spawn trash command: {}", e))?;

        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for trash command: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Trash command exited with status: {}", status))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Move to Trash is currently only supported on macOS".to_string())
    }
}

/// Preview a file using Quick Look (macOS)
#[allow(dead_code)]
pub fn quick_look(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("qlmanage")
            .args(["-p", path])
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

/// Show the "Open With" dialog for a file (macOS)
#[allow(dead_code)]
pub fn open_with(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to trigger the "Open With" menu
        let script = format!(
            r#"tell application "Finder"
                activate
                set theFile to POSIX file "{}"
                open information window of theFile
            end tell"#,
            crate::utils::escape_applescript_string(path)
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open 'Open With' dialog: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Open With is only supported on macOS".to_string())
    }
}

/// Show the Get Info window for a file in Finder (macOS)
#[allow(dead_code)]
pub fn show_info(path: &str) -> Result<(), String> {
    use std::process::Command;

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
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to show file info: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Show Info is only supported on macOS".to_string())
    }
}

/// Run an AppleScript and return the text result, or `None` if the user cancelled.
#[cfg(target_os = "macos")]
fn run_osascript_capture(script: &str) -> Result<Option<String>, String> {
    use std::process::Command;
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to run AppleScript: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(Some(stdout));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("User canceled") || stderr.contains("(-128)") {
        return Ok(None);
    }

    Err(format!("AppleScript failed: {}", stderr.trim()))
}

/// Show a native rename dialog and return the user-entered new name, or `None` if cancelled.
pub fn prompt_rename_target_name(path: &str) -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        let current_name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Selected item has no filename".to_string())?;

        let escaped_default = crate::utils::escape_applescript_string(current_name);
        let script = format!(
            r#"tell application "System Events"
                activate
                display dialog "Rename selected item" default answer "{}" buttons {{"Cancel", "Rename"}} default button "Rename"
                return text returned of result
            end tell"#,
            escaped_default
        );
        run_osascript_capture(&script)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Rename is currently only supported on macOS".to_string())
    }
}

/// Rename a file or directory in-place and return the new full path.
pub fn rename_path(path: &str, new_name: &str) -> Result<String, String> {
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

    if target == current_path {
        return Ok(path.to_string());
    }

    std::fs::rename(current_path, &target).map_err(|e| format!("Failed to rename item: {}", e))?;

    Ok(target.to_string_lossy().to_string())
}

/// Show a native move-destination dialog and return the user-entered directory, or `None` if cancelled.
pub fn prompt_move_destination_dir(path: &str, is_dir: bool) -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        let default_dir = move_destination_default_directory(path, is_dir);
        let escaped_default = crate::utils::escape_applescript_string(&default_dir);
        let script = format!(
            r#"tell application "System Events"
                activate
                display dialog "Move selected item to folder" default answer "{}" buttons {{"Cancel", "Move"}} default button "Move"
                return text returned of result
            end tell"#,
            escaped_default
        );
        run_osascript_capture(&script)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        let _ = is_dir;
        Err("Move is currently only supported on macOS".to_string())
    }
}

/// Move a file or directory to a new parent folder and return the new full path.
pub fn move_path(path: &str, destination_dir: &str) -> Result<String, String> {
    let current_path = Path::new(path);
    let filename = current_path
        .file_name()
        .ok_or_else(|| "Selected item has no filename".to_string())?;

    let expanded_destination = crate::file_search::expand_path(destination_dir)
        .unwrap_or_else(|| destination_dir.to_string());
    let destination_path = Path::new(&expanded_destination);

    if !destination_path.is_dir() {
        return Err(format!(
            "Destination is not a folder: {}",
            destination_path.display()
        ));
    }

    let target = destination_path.join(filename);
    if target == current_path {
        return Ok(path.to_string());
    }

    std::fs::rename(current_path, &target).map_err(|e| format!("Failed to move item: {}", e))?;

    Ok(target.to_string_lossy().to_string())
}

fn duplicate_target_path(path: &Path) -> Result<std::path::PathBuf, String> {
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
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Could not find an available duplicate name".to_string())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir(dst).map_err(|e| format!("Failed to create duplicate folder: {}", e))?;
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("Failed to read source folder: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read folder entry: {}", e))?;
        let entry_path = entry.path();
        let target_path = dst.join(entry.file_name());
        let metadata = std::fs::symlink_metadata(&entry_path)
            .map_err(|e| format!("Failed to inspect folder entry: {}", e))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            copy_symlink(&entry_path, &target_path)?;
        } else if file_type.is_dir() {
            copy_dir_recursive(&entry_path, &target_path)?;
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
    let src = Path::new(path);
    if !src.exists() {
        return Err(format!("Path does not exist: {}", src.display()));
    }
    let target = duplicate_target_path(src)?;
    if src.is_dir() {
        copy_dir_recursive(src, &target)?;
    } else {
        std::fs::copy(src, &target).map_err(|e| format!("Failed to duplicate item: {}", e))?;
    }
    Ok(target.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        duplicate_path, escape_windows_cmd_open_target, move_destination_default_directory,
        terminal_working_directory,
    };
    use tempfile::tempdir;

    #[test]
    fn test_terminal_working_directory_returns_parent_for_file_paths() {
        let resolved = terminal_working_directory("/tmp/a/b/file.txt", false);
        assert_eq!(resolved, "/tmp/a/b");
    }

    #[test]
    fn test_move_destination_default_directory_returns_parent_for_directories() {
        let resolved = move_destination_default_directory("/tmp/a/b/folder", true);
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
