# Script Kit GPUI - Skills-Based Code Improvements

This document identifies improvements needed to align the codebase with GPUI and Script Kit skill best practices. Issues are prioritized by severity and grouped by category.

---

## Executive Summary

The codebase shows evidence of iterative improvement and generally follows GPUI patterns correctly. However, analysis against 17 GPUI/Script Kit skills revealed issues ranging from critical (potential panics) to minor (code quality).

| Priority | Count | Summary |
|----------|-------|---------|
| **CRITICAL** | 3 | RefCell panic risks, missing application cursor mode |
| **HIGH** | 8 | Hardcoded colors, expensive theme cloning, missing scroll coalescing |
| **MEDIUM** | 12 | Missing cx.notify(), blocking main thread, inconsistent patterns |
| **LOW** | 10+ | Code quality, documentation, unused code |

---

## CRITICAL PRIORITY (Fix Immediately)

### 1. Direct Window Resize Calls Risk RefCell Panics

**Skill Reference:** `script-kit-window-control`, AGENTS.md section 28.1

**Problem:** Direct calls to `resize_first_window_to_height()` during render cycle can cause RefCell borrow conflicts.

**Locations:**
| File | Line | Context |
|------|------|---------|
| `src/app_impl.rs` | 1840 | `update_window_size()` |
| `src/app_impl.rs` | 3749 | `reset_to_script_list()` |

**Current Code:**
```rust
// WRONG - can cause RefCell panic if called during render
let target_height = height_for_view(view_type, item_count);
resize_first_window_to_height(target_height);
```

**Fix:**
```rust
// Use Window::defer to schedule at end of effect cycle
use crate::window_ops::queue_resize;
queue_resize(target_height, window, cx);
```

---

### 2. Missing Application Cursor Mode in Terminal

**Skill Reference:** `script-kit-terminal`

**Problem:** Terminal always sends normal mode arrow key sequences, breaking vim, less, htop, fzf.

**Location:** `src/term_prompt.rs` lines 777-780

**Current Code:**
```rust
// WRONG - always sends normal mode sequences
"up" | "arrowup" => Some(b"\x1b[A"),
"down" | "arrowdown" => Some(b"\x1b[B"),
```

**Fix:**
```rust
let app_cursor = this.terminal.is_application_cursor_mode();
match key_str.as_str() {
    "up" | "arrowup" => Some(if app_cursor { b"\x1bOA" } else { b"\x1b[A" }),
    "down" | "arrowdown" => Some(if app_cursor { b"\x1bOB" } else { b"\x1b[B" }),
    "right" | "arrowright" => Some(if app_cursor { b"\x1bOC" } else { b"\x1b[C" }),
    "left" | "arrowleft" => Some(if app_cursor { b"\x1bOD" } else { b"\x1b[D" }),
}
```

---

### 3. State Mutation During Render

**Skill Reference:** `gpui` anti-patterns

**Problem:** `selected_index` modified during render, which is an anti-pattern.

**Location:** `src/render_script_list.rs` lines 50, 53, 59

**Current Code:**
```rust
// WRONG - mutating state during render
self.selected_index = valid_idx;
self.selected_index = 0;
```

**Fix:** Move validation to event handlers or pre-render phase, not inside `render()` method.

---

## HIGH PRIORITY (Fix This Sprint)

### 4. Hardcoded Colors (100+ occurrences)

**Skill Reference:** `script-kit-theme`

**Problem:** Production code uses hardcoded `rgb(0x...)` instead of theme tokens.

**Locations:**
| File | Line | Code |
|------|------|------|
| `src/render_prompts/arg.rs` | 53 | `.text_color(rgb(0xffffff))` |
| `src/render_script_list.rs` | 316 | `rgb(0xB85C00)` |
| `src/app_shell/shell.rs` | 233, 265, 372 | `rgb(0x000000)`, `rgba(0x00000080)` |
| `src/theme/helpers.rs` | 131 | `rgb(0x00ffff)` cursor |
| `src/editor.rs` | 1009 | `rgb(0xffffff)` |
| `src/components/prompt_header.rs` | 526 | `rgb(0x000000)` |
| `src/prompts/env.rs` | 235 | `.text_color(rgb(0xffffff))` |

**Fix Pattern:**
```rust
// BAD
.text_color(rgb(0xffffff))

// GOOD
.text_color(rgb(colors.text.primary))
// or
.text_color(rgb(design_colors.text_on_accent))
```

---

### 5. Expensive Theme Cloning into Closures (18 occurrences)

**Skill Reference:** `script-kit-theme`

**Problem:** `Arc::new(self.theme.clone())` clones the entire Theme struct instead of using lightweight Copy color structs.

**Locations:**
| File | Count | Lines |
|------|-------|-------|
| `src/prompt_handler.rs` | 10 | 134, 220, 317, 333, 344, 1078, 1175, 1241, 1294, 1362 |
| `src/app_impl.rs` | 4 | 449, 1956, 2044, 2467 |
| `src/app_execute.rs` | 3 | 875, 952, 1101 |
| `src/render_prompts/path.rs` | 1 | 17 |

**Fix Pattern:**
```rust
// BAD - heap allocation on every use
let theme_arc = Arc::new(self.theme.clone());
closure.do_something(theme_arc)

// GOOD - Copy type, zero allocation
let colors = ListItemColors::from_theme(&self.theme);
closure.do_something(colors)
```

---

### 6. Missing Scroll Coalescing in Builtin Views

**Skill Reference:** `script-kit-components`

**Problem:** Rapid arrow key presses in builtin views cause lag/jank due to missing `NavCoalescer`.

**Locations:**
| View | File | Lines |
|------|------|-------|
| Clipboard History | `src/render_builtins.rs` | 95-112 |
| Window Switcher | `src/render_builtins.rs` | 983-997 |
| App Launcher | `src/render_builtins.rs` | 661-704 |

**Fix:** Apply `NavCoalescer` pattern from `src/navigation.rs`:
```rust
use crate::navigation::NavCoalescer;

let coalescer = NavCoalescer::new();
// In key handler:
coalescer.process_arrow(direction, |delta| {
    self.selected_index = (self.selected_index as i32 + delta)
        .clamp(0, max_idx as i32) as usize;
    cx.notify();
});
```

---

### 7. Duplicate ListItemColors Definitions

**Skill Reference:** `script-kit-components`

**Problem:** Two different `ListItemColors` structs cause confusion.

**Locations:**
1. `src/theme/helpers.rs:19-36` - Uses `Rgba` type
2. `src/list_item.rs:220-268` - Uses `u32` type

**Fix:** Consolidate to one definition, deprecate the other.

---

### 8. Inconsistent Opacity Values

**Skill Reference:** `script-kit-theme`

**Problem:** Selection/hover opacity values vary across the codebase.

| Location | Selected | Hover |
|----------|----------|-------|
| `theme/types.rs` (default) | 0.15 | 0.08 |
| `theme/types.rs` (default_*_opacity) | 0.95 | 0.85 |
| `list_item.rs` | 0.95 | 0.85 |
| `theme/helpers.rs` | 0x40 (25%) | 0x59 (35%) |

**Fix:** Standardize per AGENTS.md:
- Selection highlight: ~50% alpha (0x80)
- Hover highlight: ~25% alpha (0x40)
- Disabled state: ~30% opacity

---

## MEDIUM PRIORITY (Fix Soon)

### 9. Missing cx.notify() in Submit Methods

**Skill Reference:** `gpui` state management

**Problem:** Submit/cancel methods modify state without calling `cx.notify()`.

**Locations:**
| File | Methods |
|------|---------|
| `src/prompts/div.rs` | `submit()`, `submit_with_value()` |
| `src/prompts/drop.rs` | `submit()`, `submit_cancel()` |
| `src/prompts/template.rs` | `submit()`, `submit_cancel()` |
| `src/prompts/select.rs` | `submit()`, `submit_cancel()` |
| `src/prompts/path.rs` | `submit_cancel()` |
| `src/prompts/env.rs` | `submit()`, `submit_cancel()` |

**Fix:** Add `cx.notify()` before callback invocation for consistency.

---

### 10. Blocking Main Thread - Clipboard Operations

**Skill Reference:** `script-kit-clipboard`

**Problem:** `copy_entry_to_clipboard()` blocks main thread with SQLite queries and image decoding.

**Location:** `src/clipboard_history/clipboard.rs:23-75`

**Operations that block:**
1. DB mutex lock acquisition
2. SQLite query for content
3. Base64 image decoding
4. System clipboard write
5. Timestamp update query

**Fix:** Wire up the existing `db_worker/` module (currently marked `#[allow(dead_code)]`).

---

### 11. Synchronous SQLite Query During Render

**Skill Reference:** `script-kit-clipboard`

**Location:** `src/render_builtins.rs:469`

```rust
// WRONG - SQLite query in render loop
let content = clipboard_history::get_entry_content(&entry.id)
    .unwrap_or_else(|| entry.text_preview.clone());
```

**Fix:** Use async fetch with "Loading..." placeholder, or prefetch content.

---

### 12. Mutex in Render Path (PathPrompt)

**Skill Reference:** `gpui` anti-patterns

**Location:** `src/prompts/path.rs` lines 339, 501, 609, 614

```rust
// Potential deadlock if another thread holds lock
self.actions_showing.lock().map(|g| *g).unwrap_or(false)
```

**Fix:** Use `AtomicBool` instead:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
pub actions_showing: Arc<AtomicBool>,
// Usage:
let is_showing = self.actions_showing.load(Ordering::Relaxed);
```

---

### 13. Missing Focus-Aware Color Handling

**Skill Reference:** `script-kit-theme`

**Problem:** Most components don't use `theme.get_colors(is_focused)` for dimmed unfocused state.

**Current Usage:** Only 4 locations check focus state:
- `src/components/form_fields.rs:563, 1144, 1348`
- `src/form_prompt.rs:217`

**Fix:** All components should use focus-aware colors:
```rust
let is_focused = self.focus_handle.is_focused(window);
let colors = self.theme.get_colors(is_focused);
// Use colors.* for all rendering
```

---

### 14. Inline Rendering Instead of Component Abstraction

**Skill Reference:** `script-kit-components`

**Locations:**
| File | Lines | Issue |
|------|-------|-------|
| `src/actions/dialog.rs` | 995-1086 | Action item rendering (~90 lines inline) |
| `src/render_builtins.rs` | 813-877 | Search input with cursor (duplicated) |
| `src/render_script_list.rs` | 839-899 | Header rendering (~60 lines inline) |

**Fix:** Extract into reusable components:
- `ActionItem` component
- `SearchInputWithCursor` component
- Consistently use existing `PromptHeader`

---

### 15. Missing .context() for Error Handling

**Skill Reference:** AGENTS.md section 14

**Problem:** Error handling uses logging but lacks structured context propagation.

**Current Pattern:**
```rust
let stdin = child.stdin.take()
    .ok_or_else(|| "Failed to open script stdin".to_string())?;
```

**Fix:**
```rust
use anyhow::Context;
let stdin = child.stdin.take()
    .context(format!("Failed to open stdin for script: {}", script_path))?;
```

---

### 16. Byte Indexing in hex_color Parser

**Skill Reference:** `script-kit-prompts`

**Location:** `src/prompts/div.rs:85-112`

**Problem:** Byte slicing on strings is safe for ASCII hex but no guard against non-ASCII.

**Fix:**
```rust
fn parse_hex_color(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    
    // Guard against non-ASCII
    if !hex.is_ascii() {
        return None;
    }
    // ... rest of implementation
}
```

---

## LOW PRIORITY (Technical Debt)

### 17. Large ScriptListApp Struct (~100 fields)

**Location:** `src/main.rs:951-1133`

**Recommendation:** Extract into sub-structs:
- `NavState`
- `CacheState`
- View-specific state modules
- Scroll handles group

---

### 18. Proliferation of #[allow(dead_code)]

**Location:** Multiple files

**Action:** Audit and either implement or remove unused code.

---

### 19. DB Worker Module Not Wired Up

**Location:** `src/clipboard_history/db_worker/`

**Status:** Complete implementation exists but marked `#[allow(dead_code)]`

**Action:** Wire up for clipboard operations to eliminate main thread blocking.

---

### 20. Thread Join Missing in Terminal Drop

**Location:** `src/terminal/alacritty.rs`

**Issue:** Reader thread not explicitly joined on drop.

**Fix:** Add join with timeout in `Drop` impl.

---

## Verification Checklist

Before marking issues fixed, verify:

- [ ] `cargo check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] Visual testing via stdin JSON protocol
- [ ] Screenshot captured and analyzed for UI changes

---

## Files Most Needing Attention

| File | Issue Count | Priority Issues |
|------|-------------|-----------------|
| `src/app_impl.rs` | 5 | RefCell panic, theme cloning |
| `src/render_builtins.rs` | 5 | Missing coalescing, SQLite in render |
| `src/prompts/*.rs` | 10 | Missing notify, hardcoded colors |
| `src/term_prompt.rs` | 2 | Application cursor mode |
| `src/theme/` | 3 | Duplicate types, inconsistent opacity |

---

## Reference Skills Used

This analysis compared the codebase against these skills:

1. `gpui` - Core GPUI patterns
2. `gpui-component` - Component library patterns
3. `script-kit-app-shell` - Shell/chrome patterns
4. `script-kit-components` - UI component patterns
5. `script-kit-theme` - Theme system patterns
6. `script-kit-icons` - Icon system patterns
7. `script-kit-prompts` - Prompt patterns
8. `script-kit-terminal` - Terminal integration
9. `script-kit-clipboard` - Clipboard history
10. `script-kit-executor` - Script execution
11. `script-kit-config` - Configuration
12. `script-kit-scripts` - Script loading
13. `script-kit-window-control` - Window management
14. `script-kit-notifications` - Notification system
15. `script-kit-mcp` - MCP protocol
16. `script-kit-platform` - Platform integration
17. `script-kit-menu-bar` - Menu bar system

---

*Generated: 2026-01-07*
*Codebase analyzed: script-kit-gpui*
