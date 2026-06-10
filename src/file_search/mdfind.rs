use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};

use tracing::{debug, instrument, warn};

use super::{
    build_mdquery, detect_file_type, expand_path, looks_like_advanced_mdquery, FileResult,
};

const FILESYSTEM_FALLBACK_MAX_VISITED: usize = 75_000;
const MDFIND_TIMEOUT: Duration = Duration::from_secs(3);
const MDFIND_POLL_INTERVAL: Duration = Duration::from_millis(25);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchFilesStreamingOptions {
    pub skip_metadata: bool,
    pub allow_filesystem_fallback: bool,
}

impl SearchFilesStreamingOptions {
    pub fn dedicated_file_search(skip_metadata: bool) -> Self {
        Self {
            skip_metadata,
            allow_filesystem_fallback: true,
        }
    }

    pub fn root_search() -> Self {
        Self {
            skip_metadata: true,
            allow_filesystem_fallback: false,
        }
    }
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

    let (line_tx, line_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            let _ = line_tx.send(line_result);
        }
    });

    let mut results = Vec::new();
    let deadline = Instant::now() + MDFIND_TIMEOUT;

    // Stream line-by-line, stopping after limit
    while results.len() < limit {
        if results.len() >= limit {
            break;
        }

        let line_result = match line_rx.recv_timeout(MDFIND_POLL_INTERVAL) {
            Ok(line_result) => line_result,
            Err(RecvTimeoutError::Timeout) => {
                if child.try_wait().ok().flatten().is_some() {
                    break;
                }
                if Instant::now() >= deadline {
                    warn!("mdfind search timed out; falling back if possible");
                    let _ = child.kill();
                    break;
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };

        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                debug!(error = %e, "Error reading mdfind output line");
                continue;
            }
        };

        if let Some(result) = file_result_from_mdfind_line(line, false) {
            results.push(result);
        }
    }

    // Clean up the child process
    // If we stopped early (hit limit), kill the process
    if results.len() >= limit {
        let _ = child.kill();
    }
    // Wait for process to fully exit (prevents zombies)
    let _ = child.wait();

    if results.is_empty() && !looks_like_advanced_mdquery(query) {
        let fallback = search_files_filesystem_fallback(query, onlyin, limit);
        if !fallback.is_empty() {
            debug!(
                result_count = fallback.len(),
                "Search completed with filesystem fallback results"
            );
            return fallback;
        }
    }

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
    on_event: F,
) where
    F: FnMut(SearchEvent),
{
    search_files_streaming_with_options(
        query,
        onlyin,
        limit,
        cancel,
        SearchFilesStreamingOptions::dedicated_file_search(skip_metadata),
        on_event,
    );
}

#[instrument(skip_all, fields(query = %query, onlyin = ?onlyin, limit = limit, skip_metadata = options.skip_metadata, allow_filesystem_fallback = options.allow_filesystem_fallback))]
pub fn search_files_streaming_with_options<F>(
    query: &str,
    onlyin: Option<&str>,
    limit: usize,
    cancel: CancelToken,
    options: SearchFilesStreamingOptions,
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

    let (line_tx, line_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            let _ = line_tx.send(line_result);
        }
    });

    let mut count = 0usize;
    let deadline = Instant::now() + MDFIND_TIMEOUT;

    loop {
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

        let line_result = match line_rx.recv_timeout(MDFIND_POLL_INTERVAL) {
            Ok(line_result) => line_result,
            Err(RecvTimeoutError::Timeout) => {
                if child.try_wait().ok().flatten().is_some() {
                    break;
                }
                if Instant::now() >= deadline {
                    warn!("mdfind streaming search timed out; falling back if possible");
                    let _ = child.kill();
                    break;
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };

        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                debug!(error = %e, "Error reading mdfind output line");
                continue;
            }
        };

        if let Some(result) = file_result_from_mdfind_line(line, options.skip_metadata) {
            on_event(SearchEvent::Result(result));
            count += 1;
        }
    }

    // Clean up the child process
    let _ = child.wait();

    if options.allow_filesystem_fallback
        && count == 0
        && !cancel.load(Ordering::Relaxed)
        && !looks_like_advanced_mdquery(query)
    {
        let fallback = search_files_filesystem_fallback(query, onlyin, limit);
        for result in fallback {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            on_event(SearchEvent::Result(result));
        }
    }

    debug!(result_count = count, "Streaming search completed");
    on_event(SearchEvent::Done);
}

fn file_result_from_mdfind_line(line: String, skip_metadata: bool) -> Option<FileResult> {
    // Only skip truly empty lines, not lines with spaces.
    // .lines() already strips newline characters; macOS paths can contain
    // leading/trailing spaces, so do not trim.
    if line.is_empty() {
        return None;
    }

    let path = Path::new(&line);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

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

    Some(FileResult {
        path: line,
        name,
        size,
        modified,
        file_type,
    })
}

fn search_files_filesystem_fallback(
    query: &str,
    onlyin: Option<&str>,
    limit: usize,
) -> Vec<FileResult> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() || limit == 0 {
        return Vec::new();
    }

    let roots = fallback_roots(onlyin);
    if roots.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut visited = 0usize;
    let mut stack = roots;

    while let Some(dir) = stack.pop() {
        if results.len() >= limit || visited >= FILESYSTEM_FALLBACK_MAX_VISITED {
            break;
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            if results.len() >= limit || visited >= FILESYSTEM_FALLBACK_MAX_VISITED {
                break;
            }
            visited += 1;

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };

            if metadata.is_dir() {
                if should_skip_fallback_dir(&name) {
                    continue;
                }
                stack.push(path.clone());
            }

            if !name.to_lowercase().contains(&needle) {
                continue;
            }

            let Some(path_str) = path.to_str().map(str::to_string) else {
                continue;
            };
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            results.push(FileResult {
                path: path_str,
                name,
                size: metadata.len(),
                modified,
                file_type: detect_file_type(&path),
            });
        }
    }

    results.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.path.cmp(&b.path))
    });
    results
}

fn fallback_roots(onlyin: Option<&str>) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(dir) = onlyin {
        if let Some(expanded) = expand_path(dir) {
            push_fallback_root(&mut roots, PathBuf::from(expanded));
        }
        return roots;
    }

    if let Some(home) = dirs::home_dir() {
        push_fallback_root(&mut roots, home.clone());
        for child in [
            "Desktop",
            "Documents",
            "Downloads",
            "dev",
            "Developer",
            "Projects",
        ] {
            push_fallback_root(&mut roots, home.join(child));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        push_fallback_root(&mut roots, cwd);
    }

    roots
}

fn push_fallback_root(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if !path.is_dir() {
        return;
    }
    let canonical = path.canonicalize().unwrap_or(path);
    if !roots.iter().any(|existing| existing == &canonical) {
        roots.push(canonical);
    }
}

/// Most-recently-modified files under `root`, for scopes Spotlight cannot
/// serve (hidden dot-directory cwds like `~/.scriptkit`). Bounded walk with
/// the same skip rules as the search fallback, sorted by mtime descending.
/// Hidden files are skipped for this landing-state seed; typing a sub-query
/// still finds them through the search fallback.
pub fn recent_files_filesystem(root: &Path, limit: usize) -> Vec<FileResult> {
    if limit == 0 || !root.is_dir() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut visited = 0usize;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if visited >= FILESYSTEM_FALLBACK_MAX_VISITED {
            break;
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            if visited >= FILESYSTEM_FALLBACK_MAX_VISITED {
                break;
            }
            visited += 1;

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };

            if metadata.is_dir() {
                if !should_skip_fallback_dir(&name) && !name.starts_with('.') {
                    stack.push(path);
                }
                continue;
            }
            if name.starts_with('.') {
                continue;
            }
            let Some(path_str) = path.to_str().map(str::to_string) else {
                continue;
            };

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            results.push(FileResult {
                path: path_str,
                name,
                size: metadata.len(),
                modified,
                file_type: detect_file_type(&path),
            });
        }
    }

    results.sort_by(|a, b| b.modified.cmp(&a.modified));
    results.truncate(limit);
    results
}

fn should_skip_fallback_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | ".Trash"
            | ".cache"
            | "Library"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".turbo"
    )
}

#[cfg(test)]
mod tests {
    use super::{search_files_filesystem_fallback, SearchFilesStreamingOptions};

    #[test]
    fn streaming_options_keep_dedicated_file_search_fallback_enabled() {
        let options = SearchFilesStreamingOptions::dedicated_file_search(true);
        assert!(options.skip_metadata);
        assert!(options.allow_filesystem_fallback);
    }

    #[test]
    fn streaming_options_disable_root_search_fallback() {
        let options = SearchFilesStreamingOptions::root_search();
        assert!(options.skip_metadata);
        assert!(!options.allow_filesystem_fallback);
    }

    #[test]
    fn filesystem_fallback_finds_files_inside_onlyin_directory() {
        let temp = tempfile::tempdir().expect("temp dir");
        let wanted = temp.path().join("script-kit-search-target.txt");
        std::fs::write(&wanted, "fixture").expect("write fixture");

        let results = search_files_filesystem_fallback("search-target", temp.path().to_str(), 10);

        // The walker canonicalizes its roots (macOS tempdirs live behind the
        // /var → /private/var symlink), so compare canonical paths.
        let wanted = wanted.canonicalize().expect("canonicalize fixture path");
        assert!(
            results
                .iter()
                .any(|entry| std::path::Path::new(&entry.path) == wanted),
            "fallback should find filename matches under onlyin"
        );
    }

    #[test]
    fn recent_files_walk_skips_hidden_and_noise_dirs() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("visible.txt"), "fixture").expect("write fixture");
        std::fs::write(temp.path().join(".hidden.txt"), "fixture").expect("write fixture");
        std::fs::create_dir(temp.path().join("node_modules")).expect("mkdir");
        std::fs::write(temp.path().join("node_modules/dep.js"), "fixture").expect("write fixture");
        std::fs::create_dir(temp.path().join("src")).expect("mkdir");
        std::fs::write(temp.path().join("src/nested.rs"), "fixture").expect("write fixture");

        let results = super::recent_files_filesystem(temp.path(), 10);
        let names: Vec<&str> = results.iter().map(|entry| entry.name.as_str()).collect();

        assert!(names.contains(&"visible.txt"), "top-level file: {names:?}");
        assert!(names.contains(&"nested.rs"), "nested file: {names:?}");
        assert!(
            !names.contains(&".hidden.txt"),
            "hidden files are not recents seeds: {names:?}"
        );
        assert!(
            !names.contains(&"dep.js"),
            "noise dirs (node_modules) are skipped: {names:?}"
        );
    }

    #[test]
    fn filesystem_fallback_respects_limit() {
        let temp = tempfile::tempdir().expect("temp dir");
        for ix in 0..3 {
            std::fs::write(
                temp.path().join(format!("limit-target-{ix}.txt")),
                "fixture",
            )
            .expect("write fixture");
        }

        let results = search_files_filesystem_fallback("limit-target", temp.path().to_str(), 2);

        assert_eq!(results.len(), 2);
    }
}
