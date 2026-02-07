/// Streaming search: yields results as they arrive via callback.
///
/// This is the preferred API for real-time search UX because:
/// - Results appear immediately as mdfind outputs them
/// - Cancellation actually stops work (kills mdfind process)
/// - Caller can batch UI updates however they want
///
/// # Arguments
/// * `query` - Search query string
/// * `onlyin` - Optional directory to limit search scope
/// * `limit` - Maximum number of results to return
/// * `cancel` - Cancel token; set to true to stop search and kill mdfind
/// * `skip_metadata` - If true, skip stat() calls for faster results (size/modified = 0)
/// * `on_event` - Callback receiving SearchEvent for each result and final Done
///
/// # Example
/// ```ignore
/// let cancel = file_search::new_cancel_token();
/// let cancel_clone = cancel.clone();
///
/// // Start search in background thread
/// std::thread::spawn(move || {
///     file_search::search_files_streaming(
///         "query",
///         None,
///         500,
///         cancel_clone,
///         false, // include metadata
///         |event| {
///             // Send event to UI thread via channel
///             let _ = tx.send(event);
///         },
///     );
/// });
///
/// // Later, to cancel:
/// cancel.store(true, Ordering::Relaxed);
/// ```
#[instrument(skip_all, fields(query = %query, onlyin = ?onlyin, limit = limit, skip_metadata = skip_metadata))]
pub fn search_files_streaming<F>(
    query: &str,
    onlyin: Option<&str>,
    limit: usize,
    cancel: CancelToken,
    skip_metadata: bool,
    mut on_event: F,
) where
    F: FnMut(SearchEvent),
{
    if query.trim().is_empty() {
        debug!("Empty query, returning Done immediately");
        on_event(SearchEvent::Done);
        return;
    }

    // Convert user query to proper mdfind query
    let mdquery = build_mdquery(query);
    debug!(mdquery = %mdquery, "Built mdfind query for streaming");

    let mut cmd = Command::new("mdfind");
    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }
    cmd.arg(&mdquery);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child: Child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            warn!(error = %e, "Failed to spawn mdfind");
            on_event(SearchEvent::Done);
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            warn!("Failed to capture mdfind stdout");
            let _ = child.kill();
            let _ = child.wait();
            on_event(SearchEvent::Done);
            return;
        }
    };

    let reader = BufReader::new(stdout);
    let mut count = 0usize;

    for line_result in reader.lines() {
        // Check cancellation token before processing each line
        if cancel.load(Ordering::Relaxed) {
            debug!("Search cancelled, killing mdfind");
            let _ = child.kill();
            break;
        }

        if count >= limit {
            debug!("Hit limit {}, killing mdfind", limit);
            let _ = child.kill();
            break;
        }

        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                debug!(error = %e, "Error reading mdfind output line");
                continue;
            }
        };

        if line.is_empty() {
            continue;
        }

        let path = Path::new(&line);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Optionally skip metadata for faster results
        let (size, modified) = if skip_metadata {
            (0, 0)
        } else {
            std::fs::metadata(path)
                .map(|m| {
                    (
                        m.len(),
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0))
        };

        let file_type = detect_file_type(path);

        on_event(SearchEvent::Result(FileResult {
            path: line,
            name,
            size,
            modified,
            file_type,
        }));
        count += 1;
    }

    // Clean up the child process
    let _ = child.wait();
    debug!(result_count = count, "Streaming search completed");
    on_event(SearchEvent::Done);
}
/// Streaming directory listing: yields results as they're read.
///
/// Similar to `search_files_streaming` but for directory contents.
/// Useful for large directories where you want progressive loading.
///
/// # Arguments
/// * `dir_path` - Directory path (can include ~, ., ..)
/// * `cancel` - Cancel token
/// * `skip_metadata` - If true, skip stat() calls (size/modified = 0)
/// * `on_event` - Callback for each result
#[instrument(skip_all, fields(dir_path = %dir_path, skip_metadata = skip_metadata))]
pub fn list_directory_streaming<F>(
    dir_path: &str,
    cancel: CancelToken,
    skip_metadata: bool,
    mut on_event: F,
) where
    F: FnMut(SearchEvent),
{
    // Expand the path
    let expanded = match expand_path(dir_path) {
        Some(p) => p,
        None => {
            debug!("Failed to expand path: {}", dir_path);
            on_event(SearchEvent::Done);
            return;
        }
    };

    let path = Path::new(&expanded);
    if !path.is_dir() {
        debug!("Path is not a directory: {}", expanded);
        on_event(SearchEvent::Done);
        return;
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(error = %e, "Failed to read directory: {}", expanded);
            on_event(SearchEvent::Done);
            return;
        }
    };

    let mut count = 0usize;

    for entry in entries.flatten() {
        // Check cancellation
        if cancel.load(Ordering::Relaxed) {
            debug!("Directory listing cancelled");
            break;
        }

        // Internal cap
        if count >= MAX_DIRECTORY_ENTRIES {
            debug!("Hit internal cap {}", MAX_DIRECTORY_ENTRIES);
            break;
        }

        let entry_path = entry.path();
        let path_str = match entry_path.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };

        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        let (size, modified) = if skip_metadata {
            (0, 0)
        } else {
            std::fs::metadata(&entry_path)
                .map(|m| {
                    (
                        m.len(),
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0))
        };

        let file_type = detect_file_type(&entry_path);

        on_event(SearchEvent::Result(FileResult {
            path: path_str,
            name,
            size,
            modified,
            file_type,
        }));
        count += 1;
    }

    debug!(result_count = count, "Directory listing completed");
    on_event(SearchEvent::Done);
}
/// Get detailed metadata for a specific file
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// Some(FileMetadata) if the file exists and is readable, None otherwise
///
#[allow(dead_code)]
#[instrument(skip_all, fields(path = %path))]
pub fn get_file_metadata(path: &str) -> Option<FileMetadata> {
    debug!("Getting file metadata");

    let path_obj = Path::new(path);

    let metadata = match std::fs::metadata(path_obj) {
        Ok(m) => m,
        Err(e) => {
            debug!(error = %e, "Failed to get file metadata");
            return None;
        }
    };

    let name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let size = metadata.len();

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let file_type = if metadata.is_dir() {
        FileType::Directory
    } else {
        detect_file_type(path_obj)
    };

    // Check permissions
    let readable = path_obj.exists(); // If we got metadata, it's readable
    let writable = !metadata.permissions().readonly();

    Some(FileMetadata {
        path: path.to_string(),
        name,
        size,
        modified,
        file_type,
        readable,
        writable,
    })
}
// ============================================================================
// UI Helper Functions
// These functions are prepared for file search UI that's being implemented.
// Allow dead_code temporarily until the file search view is complete.
// ============================================================================

/// Get an emoji icon for the file type (used in file search UI)
#[allow(dead_code)]
pub fn file_type_icon(file_type: FileType) -> &'static str {
    match file_type {
        FileType::Directory => "📁",
        FileType::Application => "📦",
        FileType::Image => "🖼️",
        FileType::Document => "📄",
        FileType::Audio => "🎵",
        FileType::Video => "🎬",
        FileType::File => "📃",
        FileType::Other => "📎",
    }
}
/// Format file size in human-readable format (e.g., "1.2 MB", "456 KB")
#[allow(dead_code)]
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
/// Format Unix timestamp as relative time (e.g., "2 hours ago", "3 days ago")
#[allow(dead_code)]
pub fn format_relative_time(unix_timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if unix_timestamp == 0 {
        return "Unknown".to_string();
    }

    let diff = now.saturating_sub(unix_timestamp);

    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;

    if diff < MINUTE {
        "Just now".to_string()
    } else if diff < HOUR {
        let mins = diff / MINUTE;
        format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < DAY {
        let hours = diff / HOUR;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < WEEK {
        let days = diff / DAY;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < MONTH {
        let weeks = diff / WEEK;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if diff < YEAR {
        let months = diff / MONTH;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = diff / YEAR;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
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
        let parent = std::path::Path::new(path)
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
