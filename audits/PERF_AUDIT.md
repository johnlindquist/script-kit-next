# Performance Audit Report

**Date**: 2025-01-03  
**Scope**: CPU and memory hotspots in `src/` Rust codebase  
**Methodology**: Static analysis of allocation patterns, loops, blocking operations, and caching

---

## Executive Summary

| Severity | Count | Description |
|----------|-------|-------------|
| High | 4 | Cloning large structs, blocking startup I/O |
| Medium | 11 | Timer inefficiency, unbounded caches, format! in logs |
| Low | 8 | Minor allocations, acceptable polling |

**Most Impactful Fixes**:
1. Use `Arc<Script>` to avoid cloning scripts on every filter
2. Load preview content asynchronously
3. Stop cursor blink timer when window hidden
4. Use `tracing` macros instead of `format!` in logs

---

## HIGH Priority Issues

### H1: Script/Scriptlet Clones on Every Filter

**Files**: `src/app_impl.rs:735`, `src/scripts.rs:1160,1226,1255,1332`

```rust
// src/app_impl.rs:735
return self.cached_filtered_results.clone();  // Clones Vec<SearchResult>

// src/scripts.rs:1160
script: s.clone(),  // Full Script struct with 7+ String fields
```

**Impact**: `Script` has `name`, `path`, `description`, `icon`, `alias`, `shortcut` - each clone allocates. With 100+ scripts and rapid typing, this adds up.

**Fix**: Store scripts as `Arc<Script>` in `ScriptListApp`. Clone only the Arc (cheap refcount bump).

```rust
// Before
pub scripts: Vec<Script>,

// After
pub scripts: Vec<Arc<Script>>,
```

---

### H2: Theme Cloned on Every Actions Dialog Open

**File**: `src/app_impl.rs:1387,1437`

```rust
let theme_arc = std::sync::Arc::new(self.theme.clone());  // Full Theme clone
```

**Impact**: Theme struct contains nested `ColorScheme` (~40 u32 fields). Cloned each time actions dialog opens.

**Fix**: Store `Arc<Theme>` in `ScriptListApp`, clone Arc on dialog creation.

```rust
// In ScriptListApp
pub theme: Arc<Theme>,  // Not Theme

// On dialog
let theme_arc = self.theme.clone();  // Just Arc clone
```

---

### H3: Preview Content Read Blocks UI

**File**: `src/app_impl.rs:938`

```rust
self.preview_cache_lines = match std::fs::read_to_string(script_path) {
    Ok(content) => { ... }  // BLOCKING
```

**Impact**: Selecting a new script freezes UI while file is read from disk. Especially noticeable on slow storage or large files.

**Fix**: Spawn async task for file read:

```rust
cx.spawn(async move {
    let content = async_std::fs::read_to_string(path).await?;
    // Update state via cx.update(...)
}).detach();
```

---

### H4: Syntax Highlighting in Render Path

**File**: `src/app_impl.rs:942`

```rust
syntax::highlight_code_lines(&preview, lang)  // CPU-intensive
```

**Impact**: Syntax highlighting is CPU-bound. Called synchronously when selecting scripts.

**Fix**: Cache highlighted lines per (path, mtime) tuple. Compute async if not cached.

---

## MEDIUM Priority Issues

### M1: Cursor Blink Timer Runs Forever

**File**: `src/app_impl.rs:113-137`

```rust
loop {
    Timer::after(Duration::from_millis(530)).await;
    // Early exit check but loop continues
    if !is_main_window_visible() || app.focused_input == FocusedInput::None {
        return;  // Returns from closure, loop continues
    }
    cx.notify();  // Wakes render even when hidden
}
```

**Impact**: CPU wakeup every 530ms even when window is hidden.

**Fix**: Use conditional loop that exits when window hides:

```rust
loop {
    Timer::after(Duration::from_millis(530)).await;
    let should_continue = cx.update(|cx| {
        this.update(cx, |app, cx| {
            if !is_main_window_visible() {
                return false;  // Stop timer
            }
            // ... blink logic
            true
        }).unwrap_or(false)
    }).unwrap_or(false);
    
    if !should_continue { break; }
}
```

---

### M2: Entry Cache Unbounded

**File**: `src/clipboard_history.rs:241,256`

```rust
static ENTRY_CACHE: OnceLock<Mutex<Vec<ClipboardEntry>>> = OnceLock::new();
```

**Note**: `refresh_entry_cache()` limits to 500, but if code bypasses it, cache grows unbounded.

**Fix**: Use `LruCache` like `IMAGE_CACHE` does:

```rust
static ENTRY_CACHE: OnceLock<Mutex<LruCache<String, ClipboardEntry>>> = OnceLock::new();
```

---

### M3: Format Allocations in Log Paths

**Files**: Multiple throughout codebase

```rust
// src/app_impl.rs:731-744
logging::log_debug("CACHE", &format!("Filter cache HIT for '{}'", filter_text));

// src/app_impl.rs:755-764
logging::log("PERF", &format!("Search '{}' took {:.2}ms", ...));
```

**Impact**: `format!` allocates even when log level is disabled. Called on every filter keystroke.

**Fix**: Use `tracing` macros which are lazy:

```rust
// Before
logging::log("PERF", &format!("Search took {}ms", elapsed));

// After
tracing::info!(filter = %filter_text, duration_ms = elapsed, "Search completed");
```

---

### M4: 35+ Background Threads with Polling Loops

**Files**: See grep results for `thread::spawn` and `loop {`

| File | Purpose | Interval |
|------|---------|----------|
| `clipboard_history.rs:687` | Clipboard monitor | 500ms |
| `clipboard_history.rs:560` | Prune old entries | 1 hour |
| `watcher.rs` (4 loops) | File watchers | 100-500ms |
| `scheduler.rs:237` | Cron scheduler | Dynamic |
| `app_impl.rs:79` | App loading poll | 50ms |
| `app_impl.rs:430` | Menu bar poll | 50ms |

**Impact**: Many threads sleeping/waking. On battery, this hurts power consumption.

**Fix**:
1. Consolidate file watchers into single `notify` crate-based watcher
2. Use `kqueue`/`inotify` instead of polling where possible
3. Consider async runtime (tokio) instead of thread-per-watcher

---

### M5: Pre-Lowercase Script Names at Load Time

**File**: `src/scripts.rs:966-967`

```rust
let filter_lower = filter_text.to_lowercase();
// Each script name also lowercased during matching
```

**Impact**: `to_lowercase()` allocates. Called per-script during fuzzy search.

**Fix**: Store lowercased name alongside original at load time:

```rust
pub struct Script {
    pub name: String,
    pub name_lower: String,  // Pre-computed
    // ...
}
```

---

### M6: Menu Bar Items Reload Every Focus

**File**: `src/app_impl.rs:361-471`

```rust
fn load_menu_bar_items_async(&mut self, cx: &mut Context<Self>) {
    // Queries accessibility API every time
}
```

**Impact**: Accessibility API calls can take 50-200ms. Called on every window focus.

**Fix**: Cache menu bar items in memory. Invalidate only on app switch or after timeout.

---

### M7: App Icons Decoded Every Session

**File**: `src/app_launcher.rs:556-603`

**Impact**: PNG decoding is CPU-intensive. Done for every app icon on startup.

**Fix**: Consider caching decoded `RenderImage` bytes to disk (SQLite blob or file cache).

---

### M8-M11: Additional Medium Issues

| Issue | File | Description |
|-------|------|-------------|
| M8 | `scripts.rs:447` | `collect()` into Vec just to iterate |
| M9 | `app_impl.rs:984` | `to_string_lossy()` allocates for path comparison |
| M10 | Various | Multiple `format!` calls in debug logging |
| M11 | `render_script_list.rs` | `ListItemColors::from_theme()` called twice per render |

---

## LOW Priority Issues

### L1: Static String Allocations in Render

**File**: `src/render_script_list.rs:76-78`

```rust
"No scripts or snippets found".to_string()  // Allocates every render
```

**Fix**: Use `SharedString` or static reference.

---

### L2: Collect Before Iterate

**File**: `src/scripts.rs:447,458`

```rust
let lines: Vec<&str> = section.lines().collect();
```

**Fix**: Iterate directly without collecting:

```rust
for line in section.lines() { ... }
```

---

### L3-L8: Minor Issues

| Issue | Description |
|-------|-------------|
| L3 | `HashSet` created in `GroupedListState::from_groups()` |
| L4 | Trace logging in mouse wheel handler (acceptable) |
| L5 | Clone in `find_alias_match()` return |
| L6 | Path to string conversions |
| L7 | render_script_list debug log |
| L8 | Occasional theme clone on appearance change |

---

## Thread Count Analysis

**Current thread spawns**: 35+ `thread::spawn` calls identified

| Category | Count | Notes |
|----------|-------|-------|
| File watchers | 7 | Config, theme, scripts, scriptlets |
| Clipboard | 2 | Monitor + prune |
| UI loading | 2 | App loading, menu bar |
| MCP server | 2 | Connection handling |
| Script execution | 3+ | Per-script execution |
| Miscellaneous | 5+ | OCR, hotkeys, etc. |

**Recommendation**: Audit thread lifetime. Many could be consolidated or converted to async tasks.

---

## Recommendations Summary

### Immediate Wins (< 1 day each)

1. **`Arc<Script>`** - Prevent clone storms during filtering
2. **Async preview load** - Stop blocking UI on file reads  
3. **`tracing` macros** - Remove `format!` from hot paths
4. **Stop cursor timer** - Don't wake CPU when hidden

### Short Term (1-3 days each)

5. **Bounded caches everywhere** - Use `LruCache` pattern
6. **Pre-compute lowercase names** - Avoid `to_lowercase()` per search
7. **Cache menu bar items** - Don't query accessibility API on every focus
8. **Consolidate file watchers** - Use `notify` crate with single thread

### Medium Term (1 week+)

9. **Async runtime migration** - Consider tokio for unified async
10. **Disk cache for syntax highlighting** - Avoid recomputing on every session
11. **Profile thread wakeups** - Use Instruments/perf to find wake storms

---

## Profiling Commands

```bash
# macOS Instruments CPU profile
xcrun xctrace record --template 'Time Profiler' --launch ./target/release/script-kit-gpui

# Memory allocations
xcrun xctrace record --template 'Allocations' --launch ./target/release/script-kit-gpui

# Thread activity
xcrun xctrace record --template 'System Trace' --launch ./target/release/script-kit-gpui

# Linux perf
perf record -g ./target/release/script-kit-gpui
perf report
```

---

## Verification Checklist

After implementing fixes, verify:

- [ ] P95 filter latency < 50ms (measure with 100+ scripts)
- [ ] Memory stable after 1 hour of use (no leaks)
- [ ] CPU idle < 1% when window hidden
- [ ] Thread count < 20 at steady state
- [ ] Startup time < 500ms to first render

---

*Generated by performance audit. Review with `cargo flamegraph` for runtime validation.*
