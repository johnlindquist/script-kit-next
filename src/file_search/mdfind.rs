use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use tracing::{debug, instrument, warn};

use super::{build_mdquery, detect_file_type, FileResult};

/// Events emitted during streaming search
#[derive(Debug, Clone)]
pub enum SearchEvent {
    /// A new file result was found
    Result(FileResult),
    /// Search completed (either finished or cancelled)
    Done,
}

/// Cancel token for streaming searches
///
/// Set to `true` to cancel an in-flight search.
/// The search thread will check this token and stop early.
pub type CancelToken = Arc<AtomicBool>;

/// Create a new cancel token
pub fn new_cancel_token() -> CancelToken {
    Arc::new(AtomicBool::new(false))
}

/// Search for files using macOS mdfind (Spotlight)
///
/// Uses streaming to avoid buffering all results when only `limit` are needed.
/// Converts simple queries to filename-matching mdfind queries.
///
/// # Arguments
/// * `query` - Search query string (will be converted to filename query if simple)
/// * `onlyin` - Optional directory to limit search scope
/// * `limit` - Maximum number of results to return
///
/// # Returns
/// Vector of FileResult structs containing file information
#[instrument(skip_all, fields(query = %query, onlyin = ?onlyin, limit = limit))]
pub fn search_files(query: &str, onlyin: Option<&str>, limit: usize) -> Vec<FileResult> {
    debug!("Starting mdfind search");

    if query.is_empty() {
        debug!("Empty query, returning empty results");
        return Vec::new();
    }

    // Convert user query to proper mdfind query (filename matching)
    let mdquery = build_mdquery(query);
    debug!(mdquery = %mdquery, "Built mdfind query");

    let mut cmd = Command::new("mdfind");

    // Add -onlyin if specified
    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }

    // Add the query
    cmd.arg(&mdquery);

    // Set up streaming: pipe stdout instead of buffering.
    // Keep stderr detached so we cannot deadlock on an undrained stderr pipe.
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    debug!(command = ?cmd, "Spawning mdfind");

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            warn!(error = %e, "Failed to spawn mdfind");
            return Vec::new();
        }
    };

    // Take stdout for streaming reads
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            warn!("Failed to capture mdfind stdout");
            let _ = child.kill();
            let _ = child.wait();
            return Vec::new();
        }
    };

    let reader = BufReader::new(stdout);
    let mut results = Vec::new();

    // Stream line-by-line, stopping after limit
    for line_result in reader.lines() {
        if results.len() >= limit {
            break;
        }

        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                debug!(error = %e, "Error reading mdfind output line");
                continue;
            }
        };

        // Only skip truly empty lines, not lines with spaces
        // NOTE: .lines() already strips newline characters (\n, \r\n).
        // We intentionally do NOT call trim() because macOS paths CAN contain
        // leading/trailing spaces (rare but valid).
        if line.is_empty() {
            continue;
        }

        let path = Path::new(&line);

        // Get file metadata
        let (size, modified) = match std::fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (size, modified)
            }
            Err(_) => (0, 0),
        };

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let file_type = detect_file_type(path);

        results.push(FileResult {
            path: line,
            name,
            size,
            modified,
            file_type,
        });
    }

    // Clean up the child process
    // If we stopped early (hit limit), kill the process
    if results.len() >= limit {
        let _ = child.kill();
    }
    // Wait for process to fully exit (prevents zombies)
    let _ = child.wait();

    debug!(result_count = results.len(), "Search completed");
    results
}

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
    cmd.stdout(Stdio::piped()).stderr(Stdio::null());

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
