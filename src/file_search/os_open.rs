use std::path::Path;

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
        Command::new("cmd")
            .args(["/C", "start", "", path])
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
