# Expert Bundle: File Search System

## Overview

The File Search command (`search-files`) is a built-in feature that provides two modes:
1. **Search Mode**: Uses macOS Spotlight (`mdfind`) to search files by name/content
2. **Path Filtering Mode**: When input looks like a directory path (e.g., `~/dev/`), lists directory contents with fuzzy filtering

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `src/file_search.rs` | Core search logic: `search_files()`, `list_directory()`, `parse_directory_path()`, Nucleo filtering |
| `src/render_builtins.rs:1977-2460` | `render_file_search()` - UI rendering and key handling |
| `src/app_impl.rs:357-790` | Tab/Arrow/Enter key interceptors for navigation |
| `src/app_execute.rs:1054-1098` | `open_file_search()` - View initialization |
| `src/app_actions.rs:46-464` | `handle_action()` - Action execution (copy_path, reveal_in_finder, etc.) |
| `src/actions/builders.rs:9-109` | `get_file_context_actions()` - File-specific actions builder |
| `src/actions/dialog.rs` | ActionsDialog UI and keyboard routing |
| `src/scripts/input_detection.rs` | `is_directory_path()` - Detects if input is a path vs search query |

### Data Flow

```
User Input → is_directory_path() check
     │
     ├─ Yes (path-like: ~/dev/, /usr/, ./) 
     │       → list_directory() → cached_file_results
     │       → filter_results_nucleo_simple() for instant filtering
     │
     └─ No (search query: "readme")
             → search_files() via mdfind → cached_file_results
             → debounced updates
```

## Known Issues to Investigate

### Issue 1: Search Not Finding Expected Files

**Symptoms**: User searches for a file they know exists, but it doesn't appear in results.

**Possible Causes**:

1. **Spotlight Index Not Updated**: `mdfind` relies on macOS Spotlight. If Spotlight hasn't indexed a file/directory, it won't appear.
   - Check: `mdls /path/to/file` - if it shows "kMDItemDisplayName" etc., it's indexed
   - Fix: `mdimport /path/to/directory` to force re-index

2. **Excluded Directories**: Spotlight excludes certain directories by default:
   - `node_modules/`, `.git/`, `vendor/`, build directories
   - System files and hidden directories

3. **Result Limit**: `DEFAULT_LIMIT = 50` in `file_search.rs:113`
   ```rust
   pub const DEFAULT_LIMIT: usize = 50;
   ```
   If there are many matches, desired file may be cut off.

4. **Path Filtering vs Search Mode Confusion**:
   - If user types `~/dev` (no trailing slash), it triggers path filtering mode
   - User might expect full search but gets directory listing instead
   - Check `is_directory_path()` logic in `input_detection.rs:104-132`

**Debug Steps**:
```bash
# Test mdfind directly
mdfind "filename"
mdfind -onlyin ~/dev "filename"

# Check indexing status
mdutil -s /

# Force re-index a path
sudo mdutil -E /path/to/directory
```

**Code Path** (`file_search.rs:192-274`):
```rust
pub fn search_files(query: &str, onlyin: Option<&str>, limit: usize) -> Vec<FileResult> {
    // ... 
    let mut cmd = Command::new("mdfind");
    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }
    cmd.arg(query);
    // ...
}
```

### Issue 2: Tab Navigation Feels Off

**Symptoms**: When hitting Tab on a focused item in file search, behavior is inconsistent or unexpected.

**Current Tab Behavior** (`app_impl.rs:357-497`):

1. **Tab (no shift)**: If selected item is a directory, navigate INTO that directory
   - Sets input to `{directory_path}/`
   - Triggers directory listing
   - Resets `selected_index` to 0

2. **Shift+Tab**: Go UP one directory level
   - Parses current path to find parent
   - Sets input to `{parent_path}/`

**Potential Issues**:

1. **Selection State Reset**: When navigating into a directory, `selected_index` resets to 0 via `handle_filter_input_change`. This is intentional but may feel jarring.

2. **Tab Does Nothing on Files**: If selected item is a FILE (not directory), Tab does nothing. User might expect Tab to cycle through items.

3. **Path Shortening**: `shorten_path()` converts full paths to `~` notation. If this fails, paths get long and confusing.

4. **Filter State Preserved**: When entering a directory, the filter from the previous path is lost. E.g., `~/dev/fin` → Tab into `~/dev/final-project/` → filter "fin" is gone.

**Key Code** (`app_impl.rs:416-464`):
```rust
// Tab: Enter selected directory
if file.file_type == crate::file_search::FileType::Directory {
    let shortened = crate::file_search::shorten_path(&file.path);
    let new_path = format!("{}/", shortened);
    // Just update the input - handle_filter_input_change will:
    // - Update query
    // - Reset selected_index to 0
    // - Detect directory change
    // - Trigger async directory load
    this.gpui_input_state.update(cx, |state, cx| {
        state.set_value(new_path, window, cx);
    });
}
```

**Suggested Investigation**:
- Add logging to Tab handler to see what's detected
- Check if `file.file_type` is correctly set (especially for symlinks)
- Verify `filter_results_nucleo_simple()` returns correct item at `selected_index`

### Issue 3: Actions (Copy Path, etc.) Not Working

**Symptoms**: User opens actions dialog (Cmd+K), selects "Copy Path", presses Enter, but nothing happens.

**Action Execution Flow**:

```
Cmd+K → toggle_file_search_actions() → opens ActionsDialog
         stores file.path in self.file_search_actions_path
                    │
Enter → actions_interceptor (app_impl.rs:726-756)
         → handle_action(action_id, cx)
                    │
handle_action() in app_actions.rs:46-464
         │
         ├─ Built-in file actions: copy_path, reveal_in_finder, etc.
         │   → check self.file_search_actions_path
         │   → execute action
         │
         └─ Fallback to trigger_sdk_action_internal()
             (for SDK-provided actions)
```

**Potential Failure Points**:

1. **`file_search_actions_path` is None**: The path wasn't stored when opening actions.
   - Check `toggle_file_search_actions()` in `render_builtins.rs:6-102`
   - Line 53: `self.file_search_actions_path = Some(file.path.clone());`

2. **Action ID Mismatch**: The action ID from dialog doesn't match expected strings.
   - Built-in IDs: `"copy_path"`, `"reveal_in_finder"`, `"open_file"`, `"quick_look"`, etc.
   - Check `get_file_context_actions()` in `actions/builders.rs:9-109`

3. **Early Return Without Action**: Some code paths in `handle_action()` return early without doing anything.

**Copy Path Implementation** (`app_actions.rs:438-456`):
```rust
"copy_path" => {
    if let Some(ref path) = self.file_search_actions_path {
        logging::log("UI", &format!("Copy path (file search): {}", path));
        #[cfg(target_os = "macos")]
        {
            let _ = self.pbcopy(&path);  // <-- Note: error is silently ignored!
        }
        self.last_output = Some(SharedString::from(format!("Copied: {}", path)));
        self.file_search_actions_path = None;
        cx.notify();
        return;
    }
    // Falls through to trigger_sdk_action_internal if no file_search_actions_path
}
```

**Critical Bug Risk**: If `file_search_actions_path` is None, the `copy_path` case falls through to the `_` catch-all which calls `trigger_sdk_action_internal()`. This will fail silently because there's no SDK action named "copy_path".

**Debug Steps**:
1. Add logging at start of `handle_action()`:
   ```rust
   logging::log("ACTIONS", &format!(
       "handle_action: {} file_search_path={:?}", 
       action_id, self.file_search_actions_path
   ));
   ```

2. Check if `toggle_file_search_actions()` is storing the path:
   - Look for log: `"Opening file search actions for: {name}"`

3. Verify `pbcopy()` isn't failing silently:
   ```rust
   match self.pbcopy(&path) {
       Ok(_) => logging::log("UI", "pbcopy succeeded"),
       Err(e) => logging::log("ERROR", &format!("pbcopy failed: {}", e)),
   }
   ```

## State Variables

```rust
// In ScriptListApp (main.rs)
cached_file_results: Vec<file_search::FileResult>,  // Search results cache
file_search_scroll_handle: UniformListScrollHandle, // Virtual scroll state
file_search_loading: bool,                          // Loading spinner flag
file_search_debounce_task: Option<Task<()>>,        // Debounce for search
file_search_current_dir: Option<String>,            // Current directory for path mode
file_search_actions_path: Option<String>,           // Path for actions dialog

// AppView::FileSearchView variant
query: String,         // Current search/path query
selected_index: usize, // Currently highlighted item
```

## Key Handlers Summary

| Key | Context | Action |
|-----|---------|--------|
| Up/Down | File list | Navigate selection |
| Enter | File selected | Open file with default app |
| Cmd+Enter | File selected | Reveal in Finder |
| Tab | Directory selected | Navigate into directory |
| Shift+Tab | Any | Go up one directory level |
| Cmd+K | Any | Toggle actions dialog |
| Cmd+Y | File selected | Quick Look preview |
| Cmd+I | File selected | Get Info in Finder |
| Escape | File search | Return to main menu |
| Escape | Actions open | Close actions dialog |

## Actions Available

From `get_file_context_actions()`:

| Action ID | Title | Shortcut | Notes |
|-----------|-------|----------|-------|
| `open_file` / `open_directory` | Open "{name}" | ↵ | Primary action |
| `reveal_in_finder` | Show in Finder | ⌘↵ | |
| `quick_look` | Quick Look | ⌘Y | macOS only, files only |
| `open_with` | Open With... | ⌘O | macOS only |
| `show_info` | Get Info | ⌘I | macOS only |
| `copy_path` | Copy Path | ⌘⇧C | Full path to clipboard |
| `copy_filename` | Copy Filename | ⌘C | Just the filename |

## Testing Commands

```bash
# Build and test file search
cargo build

# Test basic file search
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Filter logs for file search events
echo '{"type":"run","path":"..."}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'file.?search|actions|copy|path|mdfind'

# Check mdfind directly
mdfind "test"
mdfind -onlyin ~/dev "readme"

# Check Spotlight status
mdutil -s /
```

## Nucleo Fuzzy Matching

The file search uses Nucleo for fuzzy filtering (`filter_results_nucleo_simple()`):

```rust
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
    
    // Sort by score descending (higher = better match)
    scored.sort_by(|a, b| b.2.cmp(&a.2));
    scored.into_iter().map(|(idx, r, _)| (idx, r)).collect()
}
```

**Empty pattern returns all**: If filter_pattern is empty, Nucleo returns all items with score 0.

## Path Detection Logic

`is_directory_path()` in `input_detection.rs:104-132`:

```rust
pub fn is_directory_path(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() { return false; }
    
    // Home directory path (~ or ~/...)
    if trimmed == "~" || trimmed.starts_with("~/") { return true; }
    
    // Unix-style absolute path
    if trimmed.starts_with('/') { return true; }
    
    // Current directory (. or ./...)
    if trimmed == "." || trimmed.starts_with("./") { return true; }
    
    // Parent directory (.. or ../...)
    if trimmed == ".." || trimmed.starts_with("../") { return true; }
    
    false
}
```

`parse_directory_path()` splits into directory + filter:
- `~/dev/fin` → directory=`~/dev/`, filter=Some("fin")
- `~/dev/` → directory=`~/dev/`, filter=None
- `~` → directory=`~`, filter=None

## Recommended Investigation Order

1. **Copy Path Not Working**:
   - Add logging to `handle_action()` entry point
   - Verify `file_search_actions_path` is set when actions dialog opens
   - Check `pbcopy()` return value

2. **Tab Navigation Issues**:
   - Log in Tab interceptor: what file type is detected?
   - Check if `selected_index` points to correct item in filtered list
   - Verify `parse_directory_path()` correctly parses current input

3. **Search Not Finding Files**:
   - Test `mdfind` directly in terminal
   - Check `DEFAULT_LIMIT` (50) - increase if needed
   - Verify Spotlight indexing for target directories
   - Add debug logging to `search_files()`

## Code Snippets for Debugging

### Add Logging to handle_action
```rust
fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
    logging::log("ACTIONS", &format!(
        "handle_action called: action_id='{}' file_search_actions_path={:?}",
        action_id, self.file_search_actions_path
    ));
    // ... existing code
}
```

### Add Logging to Tab Handler
```rust
// In app_impl.rs Tab interceptor
if let Some((_, file)) = filtered_results.get(*selected_index) {
    logging::log("KEY", &format!(
        "Tab: selected_index={} file={} type={:?} path={}",
        selected_index, file.name, file.file_type, file.path
    ));
    if file.file_type == crate::file_search::FileType::Directory {
        // ...
    }
}
```

### Verbose mdfind Logging
```rust
// In search_files()
let output = match cmd.output() {
    Ok(output) => {
        logging::log("EXEC", &format!(
            "mdfind returned: status={} stdout_len={} stderr={}",
            output.status,
            output.stdout.len(),
            String::from_utf8_lossy(&output.stderr)
        ));
        output
    }
    // ...
};
```
