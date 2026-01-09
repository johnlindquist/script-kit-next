# Expert Bundle: File Search - Search Not Finding Expected Files

## Issue Summary

Users report that when using the file search command, expected files don't appear in results even though they know the files exist on their system.

## Symptoms

1. User searches for a filename they know exists
2. File doesn't appear in results
3. Directory listing mode works, but search mode doesn't find the file
4. Results seem incomplete or missing recent files

---

## Architecture Overview

### Two Search Modes

The file search has two distinct modes based on input detection:

```
User Input
    │
    ▼
is_directory_path() check (src/scripts/input_detection.rs)
    │
    ├─ TRUE: Path-like input (~/dev/, /usr/, ./, etc.)
    │        → list_directory() → Filesystem enumeration
    │        → Nucleo fuzzy filter on cached results
    │
    └─ FALSE: Search query (plain text like "readme")
             → search_files() → macOS mdfind (Spotlight)
             → Results limited by DEFAULT_LIMIT
```

### Key Understanding

- **Path mode** uses direct filesystem enumeration (`std::fs::read_dir`)
- **Search mode** uses macOS Spotlight via `mdfind` command
- The detection happens in `is_directory_path()` - if this misclassifies input, wrong mode is used

---

## Complete Source Code

### File: src/file_search.rs (Core Search Logic)

```rust
//! File Search Module using macOS Spotlight (mdfind)
//!
//! This module provides file search functionality using macOS's mdfind command,
//! which interfaces with the Spotlight index for fast file searching.

use std::path::Path;
use std::process::Command;
use std::time::UNIX_EPOCH;
use tracing::{debug, instrument, warn};

/// File type classification based on extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    File,
    Directory,
    Application,
    Image,
    Document,
    Audio,
    Video,
    #[default]
    Other,
}

/// Information about a file for the actions dialog
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    #[allow(dead_code)]
    pub file_type: FileType,
    pub is_dir: bool,
}

impl FileInfo {
    pub fn from_result(result: &FileResult) -> Self {
        FileInfo {
            path: result.path.clone(),
            name: result.name.clone(),
            file_type: result.file_type,
            is_dir: result.file_type == FileType::Directory,
        }
    }

    #[allow(dead_code)]
    pub fn from_path(path: &str) -> Self {
        let path_obj = std::path::Path::new(path);
        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let is_dir = path_obj.is_dir();
        let file_type = if is_dir {
            FileType::Directory
        } else {
            FileType::File
        };

        FileInfo {
            path: path.to_string(),
            name,
            file_type,
            is_dir,
        }
    }
}

/// Result of a file search
#[derive(Debug, Clone)]
pub struct FileResult {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: u64,
    pub file_type: FileType,
}

/// Default limit for search results
pub const DEFAULT_LIMIT: usize = 50;

/// Detect file type based on extension
fn detect_file_type(path: &Path) -> FileType {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // macOS .app bundles are directories but should be classified as Applications
    if extension.as_deref() == Some("app") {
        return FileType::Application;
    }

    if path.is_dir() {
        return FileType::Directory;
    }

    match extension.as_deref() {
        Some("app") => FileType::Application,
        Some(
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "heic"
            | "heif",
        ) => FileType::Image,
        Some(
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt"
            | "ods" | "odp" | "pages" | "numbers" | "key",
        ) => FileType::Document,
        Some("mp3" | "wav" | "aac" | "flac" | "ogg" | "wma" | "m4a" | "aiff") => FileType::Audio,
        Some("mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" | "mpg") => {
            FileType::Video
        }
        Some(_) => FileType::File,
        None => {
            if path.exists() {
                if path.is_dir() {
                    FileType::Directory
                } else {
                    FileType::File
                }
            } else {
                FileType::Other
            }
        }
    }
}

/// Search for files using macOS mdfind (Spotlight)
///
/// # Arguments
/// * `query` - Search query string
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

    let mut cmd = Command::new("mdfind");

    // Add -onlyin if specified
    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }

    // Add the query (no escaping needed as Command handles it)
    cmd.arg(query);

    debug!(command = ?cmd, "Executing mdfind");

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => {
            warn!(error = %e, "Failed to execute mdfind");
            return Vec::new();
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(stderr = %stderr, "mdfind returned non-zero exit status");
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    // NOTE: .lines() already strips newline characters (\n, \r\n).
    // We intentionally do NOT call trim() because macOS paths CAN contain
    // leading/trailing spaces (rare but valid). Trimming would corrupt such paths.
    for line in stdout.lines().take(limit) {
        if line.is_empty() {
            continue;
        }

        let path = Path::new(line);

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
            path: line.to_string(),
            name,
            size,
            modified,
            file_type,
        });
    }

    debug!(result_count = results.len(), "Search completed");
    results
}

/// Expand a path string, replacing ~ with the home directory
pub fn expand_path(path: &str) -> Option<String> {
    let trimmed = path.trim();

    if trimmed.is_empty() {
        return None;
    }

    if trimmed == "~" {
        return dirs::home_dir().and_then(|p| p.to_str().map(|s| s.to_string()));
    }

    if let Some(rest) = trimmed.strip_prefix("~/") {
        return dirs::home_dir().and_then(|home| home.join(rest).to_str().map(|s| s.to_string()));
    }

    if trimmed == "." || trimmed.starts_with("./") {
        let cwd = std::env::current_dir().ok()?;
        let suffix = trimmed.strip_prefix("./").unwrap_or("");
        if suffix.is_empty() {
            return cwd.to_str().map(|s| s.to_string());
        }
        return cwd.join(suffix).to_str().map(|s| s.to_string());
    }

    if trimmed == ".." || trimmed.starts_with("../") {
        let cwd = std::env::current_dir().ok()?;
        let parent = cwd.parent()?;
        let suffix = trimmed.strip_prefix("../").unwrap_or("");
        if suffix.is_empty() {
            return parent.to_str().map(|s| s.to_string());
        }
        return parent.join(suffix).to_str().map(|s| s.to_string());
    }

    if trimmed.starts_with('/') {
        return Some(trimmed.to_string());
    }

    None
}

/// List contents of a directory
#[instrument(skip_all, fields(dir_path = %dir_path, limit = limit))]
pub fn list_directory(dir_path: &str, limit: usize) -> Vec<FileResult> {
    debug!("Starting directory listing");

    let expanded = match expand_path(dir_path) {
        Some(p) => p,
        None => {
            debug!("Failed to expand path: {}", dir_path);
            return Vec::new();
        }
    };

    let path = Path::new(&expanded);

    if !path.is_dir() {
        debug!("Path is not a directory: {}", expanded);
        return Vec::new();
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(error = %e, "Failed to read directory: {}", expanded);
            return Vec::new();
        }
    };

    let mut results: Vec<FileResult> = Vec::new();

    for entry in entries.flatten() {
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

        // Skip hidden files (starting with .)
        if name.starts_with('.') {
            continue;
        }

        let (size, modified) = match std::fs::metadata(&entry_path) {
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

        let file_type = detect_file_type(&entry_path);

        results.push(FileResult {
            path: path_str,
            name,
            size,
            modified,
            file_type,
        });
    }

    // Sort: directories first, then alphabetically by name
    results.sort_by(|a, b| {
        let a_is_dir = matches!(a.file_type, FileType::Directory);
        let b_is_dir = matches!(b.file_type, FileType::Directory);

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    results.truncate(limit);

    debug!(result_count = results.len(), "Directory listing completed");
    results
}

/// Re-export for convenience
pub use crate::scripts::input_detection::is_directory_path;

/// Result of parsing a directory path with potential filter
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDirPath {
    pub directory: String,
    pub filter: Option<String>,
}

/// Parse a directory path into its directory component and optional filter
///
/// Examples:
/// - `~/dev/` -> directory=`~/dev/`, filter=None
/// - `~/dev/fin` -> directory=`~/dev/`, filter=Some("fin")
/// - `/usr/local/bin` -> directory=`/usr/local/`, filter=Some("bin")
#[instrument(skip_all, fields(path = %path))]
pub fn parse_directory_path(path: &str) -> Option<ParsedDirPath> {
    let trimmed = path.trim();

    if !is_directory_path(trimmed) {
        return None;
    }

    if trimmed == "~" || trimmed == "~/" {
        return Some(ParsedDirPath {
            directory: "~".to_string(),
            filter: None,
        });
    }

    if trimmed.ends_with('/') {
        if let Some(expanded) = expand_path(trimmed.trim_end_matches('/')) {
            let p = Path::new(&expanded);
            if p.is_dir() {
                return Some(ParsedDirPath {
                    directory: trimmed.to_string(),
                    filter: None,
                });
            }
        }
        return None;
    }

    // Path doesn't end with / - split into parent dir and potential filter
    if let Some(last_slash_idx) = trimmed.rfind('/') {
        let parent = &trimmed[..=last_slash_idx];
        let potential_filter = &trimmed[last_slash_idx + 1..];

        let parent_to_check = if parent == "/" {
            "/"
        } else {
            parent.trim_end_matches('/')
        };

        if let Some(expanded) = expand_path(parent_to_check) {
            let p = Path::new(&expanded);
            if p.is_dir() {
                let filter = if potential_filter.is_empty() {
                    None
                } else {
                    Some(potential_filter.to_string())
                };
                return Some(ParsedDirPath {
                    directory: parent.to_string(),
                    filter,
                });
            }
        }
    }

    None
}

/// Filter FileResults using Nucleo fuzzy matching
#[allow(dead_code)]
pub fn filter_results_nucleo_simple<'a>(
    results: &'a [FileResult],
    filter_pattern: &str,
) -> Vec<(usize, &'a FileResult)> {
    use crate::scripts::NucleoCtx;

    let mut nucleo = NucleoCtx::new(filter_pattern);
    let mut scored: Vec<(usize, &FileResult, u32)> = results
        .iter()
        .enumerate()
        .filter_map(|(idx, r)| nucleo.score(&r.name).map(|score| (idx, r, score)))
        .collect();

    scored.sort_by(|a, b| b.2.cmp(&a.2));
    scored.into_iter().map(|(idx, r, _)| (idx, r)).collect()
}
```

### File: src/scripts/input_detection.rs (Path Detection Logic)

```rust
//! Input detection module for smart fallback commands

use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    Url,
    FilePath,
    MathExpression,
    CodeSnippet,
    PlainText,
}

static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(https?://|file://)[^\s]+$").expect("Invalid URL regex"));

static MATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\d\s\+\-\*/%\^\(\)\.]+$").expect("Invalid math regex")
});

const CODE_KEYWORDS: &[&str] = &[
    "function", "const ", "let ", "var ", "import ", "export ", "=>", "class ", "def ", "fn ",
    "pub fn", "async ", "await ", "return ", "if (", "for (", "while (",
];

pub fn detect_input_type(input: &str) -> InputType {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return InputType::PlainText;
    }

    if is_url(trimmed) {
        return InputType::Url;
    }

    if is_file_path(trimmed) {
        return InputType::FilePath;
    }

    if is_math_expression(trimmed) {
        return InputType::MathExpression;
    }

    if is_code_snippet(trimmed) {
        return InputType::CodeSnippet;
    }

    InputType::PlainText
}

pub fn is_url(input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with("http://")
        && !trimmed.starts_with("https://")
        && !trimmed.starts_with("file://")
    {
        return false;
    }
    URL_REGEX.is_match(trimmed)
}

/// Check if the input looks like a directory path
///
/// CRITICAL: This function determines whether to use mdfind search or directory listing.
/// If it returns TRUE for input that should be searched, user won't get mdfind results.
///
/// Matches:
/// - `~` or `~/...` (home directory paths)
/// - `/...` (absolute paths)
/// - `.` or `./...` (current directory relative paths)
/// - `..` or `../...` (parent directory relative paths)
pub fn is_directory_path(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    // Home directory path (~ or ~/...)
    if trimmed == "~" || trimmed.starts_with("~/") {
        return true;
    }

    // Unix-style absolute path
    if trimmed.starts_with('/') {
        return true;
    }

    // Current directory (. or ./...)
    if trimmed == "." || trimmed.starts_with("./") {
        return true;
    }

    // Parent directory (.. or ../...)
    if trimmed == ".." || trimmed.starts_with("../") {
        return true;
    }

    false
}

pub fn is_file_path(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    if trimmed.starts_with('/') {
        return true;
    }

    if trimmed.starts_with("~/") {
        return true;
    }

    if trimmed.starts_with("./") || trimmed.starts_with("../") {
        return true;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3
        && chars[0].is_ascii_alphabetic()
        && chars[1] == ':'
        && (chars[2] == '\\' || chars[2] == '/')
    {
        return true;
    }

    false
}

pub fn is_math_expression(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !MATH_REGEX.is_match(trimmed) {
        return false;
    }
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }
    trimmed
        .chars()
        .any(|c| matches!(c, '+' | '-' | '*' | '/' | '%' | '^'))
}

pub fn is_code_snippet(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_lowercase();
    for keyword in CODE_KEYWORDS {
        if lower.contains(keyword) {
            return true;
        }
    }
    if lower.contains(" = ") && !lower.contains("==") {
        return true;
    }
    if Regex::new(r"\w+\s*\([^)]*\)")
        .map(|r| r.is_match(trimmed))
        .unwrap_or(false)
    {
        if Regex::new(r"[a-zA-Z_]\w*\s*\(")
            .map(|r| r.is_match(trimmed))
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}
```

### File: src/app_execute.rs (Search Initialization)

```rust
/// Open the file search view
///
/// This performs an mdfind-based file search and displays results in a Raycast-like UI.
pub fn open_file_search(&mut self, query: String, cx: &mut Context<Self>) {
    logging::log(
        "EXEC",
        &format!("Opening File Search with query: {}", query),
    );

    // Perform initial search or directory listing
    // Check if query looks like a directory path
    let results = if file_search::is_directory_path(&query) {
        logging::log(
            "EXEC",
            &format!("Detected directory path, listing: {}", query),
        );
        file_search::list_directory(&query, file_search::DEFAULT_LIMIT)
    } else {
        file_search::search_files(&query, None, file_search::DEFAULT_LIMIT)
    };
    logging::log(
        "EXEC",
        &format!("File search found {} results", results.len()),
    );

    // Cache the results
    self.cached_file_results = results;

    // Set up the view state
    self.filter_text = query.clone();
    self.pending_filter_sync = true;
    self.pending_placeholder = Some("Search files...".to_string());

    // Switch to file search view
    self.current_view = AppView::FileSearchView {
        query,
        selected_index: 0,
    };

    resize_to_view_sync(ViewType::ScriptList, 0);

    self.pending_focus = Some(FocusTarget::MainFilter);
    self.focused_input = FocusedInput::MainFilter;

    cx.notify();
}
```

---

## Root Cause Analysis

### Cause 1: Spotlight Index Not Updated

**Problem**: `mdfind` relies on macOS Spotlight indexing. If files haven't been indexed, they won't appear.

**Common scenarios**:
- Recently created files (Spotlight hasn't indexed yet)
- Files in excluded directories
- External drives that aren't indexed
- Network drives

**Verification**:
```bash
# Check if file is indexed
mdls /path/to/expected/file.txt

# If you see metadata (kMDItemDisplayName, etc.), it's indexed
# If you see errors or empty results, it's not indexed
```

**Fix**:
```bash
# Force re-index a directory
sudo mdimport /path/to/directory

# Check Spotlight status
mdutil -s /

# Re-enable indexing for a volume
sudo mdutil -i on /Volumes/MyDrive
```

### Cause 2: DEFAULT_LIMIT = 50

**Problem**: Only 50 results are returned. If the desired file ranks below 50, it won't appear.

**Code location**: `src/file_search.rs:113`
```rust
pub const DEFAULT_LIMIT: usize = 50;
```

**Impact**: If mdfind returns 1000 results for "readme", only the first 50 are shown.

**Potential fix**: Increase limit or add pagination.

### Cause 3: Path Detection Misclassification

**Problem**: If `is_directory_path()` returns true for something that should be searched, the wrong mode is used.

**Example scenario**:
- User types `/readme` (intending to search for "readme" files)
- `is_directory_path("/readme")` returns `true` (starts with `/`)
- System tries to list `/readme` as a directory → fails → no results

**Current logic** (`input_detection.rs:104-132`):
```rust
pub fn is_directory_path(input: &str) -> bool {
    let trimmed = input.trim();
    
    // These patterns trigger path mode, NOT search mode:
    if trimmed == "~" || trimmed.starts_with("~/") { return true; }
    if trimmed.starts_with('/') { return true; }        // <-- Any /foo triggers path mode!
    if trimmed == "." || trimmed.starts_with("./") { return true; }
    if trimmed == ".." || trimmed.starts_with("../") { return true; }
    
    false
}
```

**Edge case**: User types `/Users` thinking it's a search query, but it's treated as a path.

### Cause 4: Spotlight Exclusions

macOS Spotlight excludes many directories by default:

- `node_modules/`
- `.git/`
- `vendor/`
- Build output directories
- System directories
- Hidden directories (starting with `.`)

**Check exclusions**:
```bash
# List Privacy exclusions (System Preferences → Spotlight → Privacy)
defaults read /.Spotlight-V100/VolumeConfiguration Exclusions
```

### Cause 5: Query Syntax

`mdfind` has specific query syntax. Plain text queries may not work as expected.

**How mdfind interprets queries**:
- `"readme"` → searches for "readme" in filename AND content
- `"kMDItemDisplayName == '*readme*'"` → only filename
- `"kMDItemContentType == public.text"` → only text files

**Current implementation** just passes the raw query:
```rust
cmd.arg(query);  // Direct pass-through
```

---

## Debugging Strategy

### Step 1: Test mdfind Directly

```bash
# Basic search
mdfind "your-search-term"

# Limit to a directory
mdfind -onlyin ~/dev "your-search-term"

# Search by filename only
mdfind "kMDItemDisplayName == '*your-term*'c"

# Check result count
mdfind "your-search-term" | wc -l
```

### Step 2: Check Spotlight Indexing

```bash
# Check if Spotlight is enabled
mdutil -s /

# Check specific file metadata
mdls /path/to/file

# Force re-index
sudo mdimport /path/to/directory
```

### Step 3: Add Debug Logging

In `search_files()`:
```rust
pub fn search_files(query: &str, onlyin: Option<&str>, limit: usize) -> Vec<FileResult> {
    logging::log("SEARCH", &format!(
        "search_files called: query='{}' onlyin={:?} limit={}",
        query, onlyin, limit
    ));

    // ... existing code ...

    let output = match cmd.output() {
        Ok(output) => {
            logging::log("SEARCH", &format!(
                "mdfind returned: status={} stdout_len={} stderr='{}'",
                output.status,
                output.stdout.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
            output
        }
        Err(e) => {
            logging::log("ERROR", &format!("mdfind execution failed: {}", e));
            return Vec::new();
        }
    };

    // ... rest of function ...

    logging::log("SEARCH", &format!(
        "search_files returning {} results for '{}'",
        results.len(), query
    ));
    results
}
```

### Step 4: Verify Mode Selection

In `open_file_search()`:
```rust
pub fn open_file_search(&mut self, query: String, cx: &mut Context<Self>) {
    let is_path = file_search::is_directory_path(&query);
    logging::log("EXEC", &format!(
        "open_file_search: query='{}' is_directory_path={}",
        query, is_path
    ));
    
    // ... rest of function ...
}
```

### Step 5: Test from App

```bash
# Build and run with logging
cargo build
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'search|mdfind|file|EXEC'
```

---

## Potential Fixes

### Fix 1: Improve mdfind Query Syntax

```rust
pub fn search_files(query: &str, onlyin: Option<&str>, limit: usize) -> Vec<FileResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut cmd = Command::new("mdfind");

    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }

    // Use a more specific query for better results
    // Search in display name (filename) with wildcards
    let mdquery = format!("kMDItemDisplayName == '*{}*'c", query);
    cmd.arg(&mdquery);

    // ... rest unchanged ...
}
```

### Fix 2: Increase Result Limit

```rust
// In file_search.rs
pub const DEFAULT_LIMIT: usize = 200;  // Was 50
```

Or make it configurable in `config.ts`:
```typescript
export default {
  fileSearchLimit: 200,
  // ...
} satisfies Config;
```

### Fix 3: Add Fallback for Failed Path Detection

```rust
pub fn open_file_search(&mut self, query: String, cx: &mut Context<Self>) {
    let results = if file_search::is_directory_path(&query) {
        let path_results = file_search::list_directory(&query, file_search::DEFAULT_LIMIT);
        // If path mode returns empty results, try search mode as fallback
        if path_results.is_empty() {
            logging::log("EXEC", "Path mode returned empty, falling back to search");
            file_search::search_files(&query, None, file_search::DEFAULT_LIMIT)
        } else {
            path_results
        }
    } else {
        file_search::search_files(&query, None, file_search::DEFAULT_LIMIT)
    };
    // ...
}
```

### Fix 4: Add Search Scope Option

Allow searching within a specific directory:
```rust
// In the UI, detect if query looks like "in:~/dev readme"
let (scope, actual_query) = parse_search_scope(&query);
let results = file_search::search_files(&actual_query, scope.as_deref(), limit);
```

---

## Testing Commands

```bash
# Build the app
cargo build

# Test file search with specific query
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search-basic.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Filter logs for search-related entries
echo '...' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'SEARCH|EXEC|mdfind|file'

# Test mdfind directly with the same query
mdfind "your-test-query"
mdfind "your-test-query" | head -50

# Check Spotlight status
mdutil -as
```

---

## Files to Modify for Fixes

| File | Purpose |
|------|---------|
| `src/file_search.rs:113` | `DEFAULT_LIMIT` constant |
| `src/file_search.rs:192-274` | `search_files()` - mdfind execution |
| `src/scripts/input_detection.rs:104-132` | `is_directory_path()` - mode detection |
| `src/app_execute.rs:1054-1098` | `open_file_search()` - initialization logic |
| `src/config.rs` | Add `file_search_limit` config option |

---

## Summary

The most likely causes for "search not finding files":

1. **Spotlight not indexing** the target directory/file
2. **Result limit (50)** cutting off results
3. **Path mode triggered** when search mode was intended
4. **mdfind query syntax** not matching what user expects

Start debugging by testing `mdfind` directly in terminal with the same query the user is typing.
