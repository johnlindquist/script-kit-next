# Clipboard History Raycast Feature Parity Expert Bundle

## Executive Summary

This bundle provides complete context for implementing Raycast-like clipboard history actions in Script Kit GPUI. The current implementation has basic clipboard history functionality (view, paste, pin/unpin, remove) but lacks the rich action system that Raycast provides - including "Paste to Active App", "Paste as Plain Text", "Copy to Clipboard", "Delete", "Edit", and "Open With" actions accessible via keyboard shortcuts and a Cmd+K action menu.

### Key Problems:
1. **No actions menu for clipboard entries** - Unlike Raycast, users can't press Cmd+K to see available actions for a clipboard entry
2. **Limited keyboard shortcuts** - Only Enter (paste) and basic navigation work; no Cmd+Shift+V (paste plain text), Cmd+D (delete), etc.
3. **No "Paste as Plain Text" option** - Text entries paste with original formatting, not stripped
4. **No "Open With" actions** - Image entries can't be opened in Preview or other apps
5. **No "Edit" action** - Text entries can't be edited before pasting

### Required Fixes:
1. `src/main.rs`: Add actions menu integration for `ClipboardHistoryView` (similar to `ArgPrompt` actions)
2. `src/main.rs`: Add keyboard shortcut handlers for common Raycast actions
3. `src/clipboard_history.rs`: Add `paste_as_plain_text()` and `open_with_app()` functions
4. `src/actions.rs`: Create `get_clipboard_entry_actions()` function for context-aware actions

### Files Included:
- `src/clipboard_history.rs`: Core clipboard history storage, retrieval, and manipulation
- `src/protocol/types.rs`: Protocol types including `ClipboardHistoryAction` enum
- `src/builtins.rs`: Built-in feature registration for clipboard history
- `src/actions.rs`: Actions dialog system for context-aware actions
- `src/main.rs` (excerpts): Clipboard history view rendering and key handling

---

## Raycast Clipboard History Features (Reference)

Raycast's clipboard history provides these actions:

### Primary Actions (triggered on Enter or shown in action menu)
| Action | Shortcut | Description |
|--------|----------|-------------|
| Paste to Active App | Enter | Copy to clipboard, hide window, simulate Cmd+V |
| Copy to Clipboard | Cmd+C | Copy content without pasting |
| Paste as Plain Text | Cmd+Shift+V | Strip formatting, paste as plain text |
| Delete | Cmd+D or Backspace | Remove entry from history |
| Pin/Unpin | Cmd+P | Toggle pin status |
| Clear All | Cmd+Shift+Delete | Clear entire history |

### Secondary Actions (for images)
| Action | Shortcut | Description |
|--------|----------|-------------|
| Quick Look | Space | Preview image in floating window |
| Open With... | Cmd+O | Open in Preview/other apps |
| Save to File | Cmd+S | Save image to disk |
| Copy File Path | Cmd+Shift+C | Copy temp file path |

### Secondary Actions (for text)
| Action | Shortcut | Description |
|--------|----------|-------------|
| Edit | Cmd+E | Edit text before pasting |
| Open URL | Cmd+O | Open if text is URL |
| Search Google | Cmd+G | Search selected text |

### UI Features
| Feature | Description |
|---------|-------------|
| Time grouping | Today, Yesterday, This Week, etc. |
| Content preview | Full text/image preview in side panel |
| Search/filter | Filter entries by content |
| Pinned entries | Pinned items stay at top, survive pruning |

---

## Current Implementation Status

### What's Working:
- SQLite-backed storage with WAL mode
- Background clipboard monitoring (500ms polling)
- Text and image storage (PNG-compressed)
- Pin/unpin entries
- Remove single entry
- Clear all history
- Time-based grouping (Today, Yesterday, etc.)
- Search/filter by content
- Preview panel for selected entry
- Enter to paste (copy + hide + Cmd+V simulation)
- 30-day retention with automatic pruning

### What's Missing (Raycast parity):
- Cmd+K action menu for entries
- Paste as plain text (Cmd+Shift+V)
- Keyboard shortcuts (Cmd+D delete, Cmd+P pin, etc.)
- Edit text entry before pasting
- Open image with external app
- Quick Look preview (Space)
- Save image to file
- Context-aware actions (URL detection, etc.)

---

## Code Context

### src/clipboard_history.rs (Full - 15K tokens)

```rs
//! Clipboard History Module
//!
//! Provides SQLite-backed clipboard history with background monitoring.
//!
//! ## Features
//! - Stores text and base64-encoded images
//! - Background polling every 500ms
//! - Time-based retention (default 30 days)
//! - Pin/unpin entries to prevent deletion
//! - Pagination support for lazy loading
//! - Time-based grouping (Today, Yesterday, This Week, etc.)
//! - OCR text storage for image entries
//!
//! ## Usage
//! ```ignore
//! use crate::clipboard_history::{init_clipboard_history, get_clipboard_history_page, group_entries_by_time};
//!
//! // Initialize on app startup
//! init_clipboard_history()?;
//!
//! // Get paginated entries
//! let entries = get_clipboard_history_page(50, 0);
//! let total = get_total_entry_count();
//!
//! // Group by time for UI display
//! let grouped = group_entries_by_time(entries);
//! ```

use anyhow::{Context, Result};
use arboard::Clipboard;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use gpui::RenderImage;
use lru::LruCache;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use smallvec::SmallVec;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Default retention period in days (entries older than this are pruned)
const DEFAULT_RETENTION_DAYS: u32 = 30;

/// Interval between background pruning checks (1 hour)
const PRUNE_INTERVAL_SECS: u64 = 3600;

/// Maximum number of decoded images to keep in memory (LRU eviction)
const MAX_IMAGE_CACHE_ENTRIES: usize = 100;

/// Maximum entries to cache in memory for fast access
const MAX_CACHED_ENTRIES: usize = 500;

/// Polling interval for clipboard changes
const POLL_INTERVAL_MS: u64 = 500;

/// Default maximum number of bytes allowed for text clipboard entries.
const DEFAULT_MAX_TEXT_CONTENT_LEN: usize = 100_000;

/// Content types for clipboard entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Image,
}

impl ContentType {
    fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "image" => ContentType::Image,
            _ => ContentType::Text,
        }
    }
}

/// Time grouping for clipboard entries (like Raycast)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeGroup {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    Older,
}

impl TimeGroup {
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeGroup::Today => "Today",
            TimeGroup::Yesterday => "Yesterday",
            TimeGroup::ThisWeek => "This Week",
            TimeGroup::LastWeek => "Last Week",
            TimeGroup::ThisMonth => "This Month",
            TimeGroup::Older => "Older",
        }
    }

    pub fn sort_order(&self) -> u8 {
        match self {
            TimeGroup::Today => 0,
            TimeGroup::Yesterday => 1,
            TimeGroup::ThisWeek => 2,
            TimeGroup::LastWeek => 3,
            TimeGroup::ThisMonth => 4,
            TimeGroup::Older => 5,
        }
    }
}

/// A single clipboard history entry
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: ContentType,
    pub timestamp: i64,
    pub pinned: bool,
    pub ocr_text: Option<String>,
}

// ... (storage functions: init_clipboard_history, add_entry, etc.)

/// Copy an entry back to the clipboard
pub fn copy_entry_to_clipboard(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let (content, content_type): (String, String) = conn
        .query_row(
            "SELECT content, content_type FROM history WHERE id = ?",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("Entry not found")?;

    drop(conn);

    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

    match ContentType::from_str(&content_type) {
        ContentType::Text => {
            clipboard.set_text(&content).context("Failed to set clipboard text")?;
        }
        ContentType::Image => {
            if let Some(image_data) = decode_base64_image(&content) {
                clipboard.set_image(image_data).context("Failed to set clipboard image")?;
            } else {
                anyhow::bail!("Failed to decode image data");
            }
        }
    }

    // Update timestamp to move entry to top
    let conn = get_connection()?;
    let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
    let timestamp = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE history SET timestamp = ? WHERE id = ?",
        params![timestamp, id],
    )?;

    info!(id = %id, "Copied entry to clipboard");
    Ok(())
}

/// Pin a clipboard entry to prevent LRU eviction
pub fn pin_entry(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute("UPDATE history SET pinned = 1 WHERE id = ?", params![id])
        .context("Failed to pin entry")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    info!(id = %id, "Pinned clipboard entry");
    Ok(())
}

/// Unpin a clipboard entry
pub fn unpin_entry(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute("UPDATE history SET pinned = 0 WHERE id = ?", params![id])
        .context("Failed to unpin entry")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    info!(id = %id, "Unpinned clipboard entry");
    Ok(())
}

/// Remove a single entry from clipboard history
pub fn remove_entry(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute("DELETE FROM history WHERE id = ?", params![id])
        .context("Failed to remove entry")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    info!(id = %id, "Removed clipboard entry");
    Ok(())
}

/// Clear all clipboard history
pub fn clear_history() -> Result<()> {
    let conn = get_connection()?;
    let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    conn.execute("DELETE FROM history", []).context("Failed to clear history")?;

    info!("Cleared all clipboard history");
    Ok(())
}
```

### src/protocol/types.rs (Relevant excerpts)

```rs
/// Clipboard history action type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardHistoryAction {
    List,
    Pin,
    Unpin,
    Remove,
    Clear,
    #[serde(rename = "trimOversize")]
    TrimOversize,
}

/// Clipboard history entry data for list responses
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClipboardHistoryEntryData {
    #[serde(rename = "entryId")]
    pub entry_id: String,
    pub content: String,
    #[serde(rename = "contentType")]
    pub content_type: ClipboardEntryType,
    pub timestamp: String,
    pub pinned: bool,
}

/// Protocol action for the Actions API
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolAction {
    pub name: String,
    pub description: Option<String>,
    pub shortcut: Option<String>,
    pub value: Option<String>,
    pub has_action: bool,
    pub visible: Option<bool>,
    pub close: Option<bool>,
}
```

### src/actions.rs (Action system - key excerpts)

```rs
/// Available actions in the actions menu
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ActionCategory,
    pub shortcut: Option<String>,
    pub has_action: bool,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    ScriptContext,
    ScriptOps,
    GlobalOps,
}

/// Get actions specific to a file/folder path
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action> {
    vec![
        Action::new("copy_path", "Copy Path", Some("Copy the full path to clipboard".to_string()), ActionCategory::ScriptContext).with_shortcut("⌘⇧C"),
        Action::new("open_in_finder", "Open in Finder", Some("Reveal in Finder".to_string()), ActionCategory::ScriptContext).with_shortcut("⌘⇧F"),
        // ... more actions
    ]
}
```

### src/main.rs (ClipboardHistoryView key handling - lines 9630-9745)

```rs
fn render_clipboard_history(&mut self, entries: Vec<clipboard_history::ClipboardEntry>, filter: String, selected_index: usize, cx: &mut Context<Self>) -> AnyElement {
    // ... setup code ...

    // Key handler for clipboard history
    let handle_key = cx.listener(
        move |this: &mut Self, event: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>| {
            let key_str = event.keystroke.key.to_lowercase();
            
            if let AppView::ClipboardHistoryView { entries, filter, selected_index } = &mut this.current_view {
                // Apply filter to get current filtered list
                let filtered_entries: Vec<_> = if filter.is_empty() {
                    entries.iter().enumerate().collect()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries.iter().enumerate()
                        .filter(|(_, e)| e.content.to_lowercase().contains(&filter_lower))
                        .collect()
                };
                let filtered_len = filtered_entries.len();

                match key_str.as_str() {
                    "up" | "arrowup" => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            this.clipboard_list_scroll_handle.scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            cx.notify();
                        }
                    }
                    "down" | "arrowdown" => {
                        if *selected_index < filtered_len.saturating_sub(1) {
                            *selected_index += 1;
                            this.clipboard_list_scroll_handle.scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            cx.notify();
                        }
                    }
                    "enter" => {
                        // Copy selected entry to clipboard, hide window, then paste
                        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
                            if let Err(e) = clipboard_history::copy_entry_to_clipboard(&entry.id) {
                                logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                            } else {
                                WINDOW_VISIBLE.store(false, Ordering::SeqCst);
                                cx.hide();
                                NEEDS_RESET.store(true, Ordering::SeqCst);

                                // Simulate Cmd+V paste after a brief delay
                                std::thread::spawn(|| {
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                    if let Err(e) = selected_text::simulate_paste_with_cg() {
                                        logging::log("ERROR", &format!("Failed to simulate paste: {}", e));
                                    }
                                });
                            }
                        }
                    }
                    "escape" => {
                        this.reset_to_script_list(cx);
                    }
                    "backspace" => {
                        if !filter.is_empty() {
                            filter.pop();
                            *selected_index = 0;
                            this.clipboard_list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                            cx.notify();
                        }
                    }
                    _ => {
                        // Character input for filter
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    filter.push(ch);
                                    *selected_index = 0;
                                    this.clipboard_list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                                    cx.notify();
                                }
                            }
                        }
                    }
                }
            }
        },
    );
    
    // ... rest of render code ...
}
```

---

## Implementation Guide

### Step 1: Add Clipboard Entry Actions Helper

Create a function in `src/actions.rs` to generate context-aware actions for clipboard entries:

```rust
// File: src/actions.rs
// Location: After get_path_context_actions() function (around line 183)

/// Get actions specific to a clipboard history entry
pub fn get_clipboard_entry_actions(entry: &crate::clipboard_history::ClipboardEntry) -> Vec<Action> {
    let mut actions = vec![
        Action::new(
            "paste",
            "Paste to Active App",
            Some("Copy and paste to the active application".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
        Action::new(
            "copy",
            "Copy to Clipboard",
            Some("Copy without pasting".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘C"),
    ];

    // Text-specific actions
    if entry.content_type == crate::clipboard_history::ContentType::Text {
        actions.push(
            Action::new(
                "paste_plain",
                "Paste as Plain Text",
                Some("Strip formatting and paste".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧V"),
        );
        
        // Check if content looks like a URL
        let content_trimmed = entry.content.trim();
        if content_trimmed.starts_with("http://") || content_trimmed.starts_with("https://") {
            actions.push(
                Action::new(
                    "open_url",
                    "Open URL",
                    Some("Open in default browser".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘O"),
            );
        }
        
        actions.push(
            Action::new(
                "edit",
                "Edit Before Pasting",
                Some("Modify text before pasting".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E"),
        );
    }

    // Image-specific actions
    if entry.content_type == crate::clipboard_history::ContentType::Image {
        actions.push(
            Action::new(
                "quick_look",
                "Quick Look",
                Some("Preview image".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("␣"),
        );
        actions.push(
            Action::new(
                "open_with",
                "Open in Preview",
                Some("Open image in Preview app".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘O"),
        );
        actions.push(
            Action::new(
                "save_to_file",
                "Save to File",
                Some("Save image to disk".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘S"),
        );
    }

    // Common actions
    if entry.pinned {
        actions.push(
            Action::new(
                "unpin",
                "Unpin",
                Some("Allow entry to be pruned".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘P"),
        );
    } else {
        actions.push(
            Action::new(
                "pin",
                "Pin",
                Some("Keep entry forever".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘P"),
        );
    }

    actions.push(
        Action::new(
            "delete",
            "Delete",
            Some("Remove from history".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘D"),
    );

    actions
}
```

### Step 2: Add Helper Functions to clipboard_history.rs

```rust
// File: src/clipboard_history.rs
// Location: After copy_entry_to_clipboard() function

/// Copy entry as plain text (strip formatting)
/// For text entries, copies the raw text. For images, this is a no-op.
pub fn copy_entry_as_plain_text(id: &str) -> Result<()> {
    let entry = get_entry_by_id(id).context("Entry not found")?;
    
    if entry.content_type != ContentType::Text {
        anyhow::bail!("Cannot paste image as plain text");
    }
    
    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
    clipboard.set_text(&entry.content).context("Failed to set clipboard text")?;
    
    info!(id = %id, "Copied entry as plain text");
    Ok(())
}

/// Save image entry to a temporary file and return the path
pub fn save_image_to_temp_file(id: &str) -> Result<PathBuf> {
    let entry = get_entry_by_id(id).context("Entry not found")?;
    
    if entry.content_type != ContentType::Image {
        anyhow::bail!("Entry is not an image");
    }
    
    // Decode the image
    let image_bytes = decode_image_content(&entry.content)?;
    
    // Create temp file
    let temp_dir = std::env::temp_dir();
    let filename = format!("clipboard-{}.png", id);
    let temp_path = temp_dir.join(&filename);
    
    std::fs::write(&temp_path, image_bytes).context("Failed to write temp file")?;
    
    info!(id = %id, path = %temp_path.display(), "Saved image to temp file");
    Ok(temp_path)
}

/// Open image entry in the default viewer (Preview on macOS)
pub fn open_image_with_default_app(id: &str) -> Result<()> {
    let temp_path = save_image_to_temp_file(id)?;
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&temp_path)
            .spawn()
            .context("Failed to open image")?;
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("Open with default app not implemented for this platform");
    }
    
    Ok(())
}

/// Decode image content to raw PNG bytes
fn decode_image_content(content: &str) -> Result<Vec<u8>> {
    if content.starts_with("png:") {
        // PNG format: "png:{base64_data}"
        let base64_data = &content[4..];
        BASE64.decode(base64_data).context("Failed to decode PNG base64")
    } else if content.starts_with("rgba:") {
        // Legacy RGBA format - convert to PNG
        let parts: Vec<&str> = content.splitn(4, ':').collect();
        if parts.len() != 4 {
            anyhow::bail!("Invalid RGBA format");
        }
        
        let width: u32 = parts[1].parse().context("Invalid width")?;
        let height: u32 = parts[2].parse().context("Invalid height")?;
        let rgba_bytes = BASE64.decode(parts[3]).context("Failed to decode RGBA base64")?;
        
        let img = image::RgbaImage::from_raw(width, height, rgba_bytes)
            .context("Failed to create image from RGBA")?;
        
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .context("Failed to encode as PNG")?;
        
        Ok(png_bytes)
    } else {
        anyhow::bail!("Unknown image format")
    }
}
```

### Step 3: Add Keyboard Shortcuts to ClipboardHistoryView

```rust
// File: src/main.rs
// Location: Inside the key handler closure in render_clipboard_history()
// Replace the current match block with:

match key_str.as_str() {
    "up" | "arrowup" => {
        if *selected_index > 0 {
            *selected_index -= 1;
            this.clipboard_list_scroll_handle.scroll_to_item(*selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }
    "down" | "arrowdown" => {
        if *selected_index < filtered_len.saturating_sub(1) {
            *selected_index += 1;
            this.clipboard_list_scroll_handle.scroll_to_item(*selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }
    "enter" => {
        // Paste to active app (default action)
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            this.clipboard_paste_entry(&entry.id, cx);
        }
    }
    "escape" => {
        this.reset_to_script_list(cx);
    }
    "backspace" => {
        if event.keystroke.modifiers.is_empty() && !filter.is_empty() {
            filter.pop();
            *selected_index = 0;
            this.clipboard_list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
            cx.notify();
        }
    }
    // NEW: Cmd+K to open actions menu
    "k" if event.keystroke.modifiers.command => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            this.show_clipboard_actions_menu(entry.clone(), cx);
        }
    }
    // NEW: Cmd+C to copy without pasting
    "c" if event.keystroke.modifiers.command && !event.keystroke.modifiers.shift => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            if let Err(e) = clipboard_history::copy_entry_to_clipboard(&entry.id) {
                logging::log("ERROR", &format!("Failed to copy: {}", e));
            } else {
                this.toast_manager.push(components::toast::Toast::success("Copied to clipboard", &this.theme));
                cx.notify();
            }
        }
    }
    // NEW: Cmd+Shift+V to paste as plain text
    "v" if event.keystroke.modifiers.command && event.keystroke.modifiers.shift => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            if entry.content_type == clipboard_history::ContentType::Text {
                if let Err(e) = clipboard_history::copy_entry_as_plain_text(&entry.id) {
                    logging::log("ERROR", &format!("Failed to copy plain text: {}", e));
                } else {
                    WINDOW_VISIBLE.store(false, Ordering::SeqCst);
                    cx.hide();
                    NEEDS_RESET.store(true, Ordering::SeqCst);
                    std::thread::spawn(|| {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let _ = selected_text::simulate_paste_with_cg();
                    });
                }
            }
        }
    }
    // NEW: Cmd+D to delete entry
    "d" if event.keystroke.modifiers.command => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            let entry_id = entry.id.clone();
            if let Err(e) = clipboard_history::remove_entry(&entry_id) {
                logging::log("ERROR", &format!("Failed to delete: {}", e));
            } else {
                // Refresh entries
                let new_entries = clipboard_history::get_cached_entries(100);
                *entries = new_entries;
                *selected_index = (*selected_index).min(entries.len().saturating_sub(1));
                this.toast_manager.push(components::toast::Toast::success("Entry deleted", &this.theme));
                cx.notify();
            }
        }
    }
    // NEW: Cmd+P to toggle pin
    "p" if event.keystroke.modifiers.command => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            let entry_id = entry.id.clone();
            let was_pinned = entry.pinned;
            let result = if was_pinned {
                clipboard_history::unpin_entry(&entry_id)
            } else {
                clipboard_history::pin_entry(&entry_id)
            };
            if let Err(e) = result {
                logging::log("ERROR", &format!("Failed to toggle pin: {}", e));
            } else {
                // Refresh entries
                let new_entries = clipboard_history::get_cached_entries(100);
                *entries = new_entries;
                let msg = if was_pinned { "Entry unpinned" } else { "Entry pinned" };
                this.toast_manager.push(components::toast::Toast::success(msg, &this.theme));
                cx.notify();
            }
        }
    }
    // NEW: Cmd+O to open (URL or image)
    "o" if event.keystroke.modifiers.command => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            match entry.content_type {
                clipboard_history::ContentType::Image => {
                    if let Err(e) = clipboard_history::open_image_with_default_app(&entry.id) {
                        logging::log("ERROR", &format!("Failed to open image: {}", e));
                    }
                }
                clipboard_history::ContentType::Text => {
                    let content = entry.content.trim();
                    if content.starts_with("http://") || content.starts_with("https://") {
                        if let Err(e) = std::process::Command::new("open").arg(content).spawn() {
                            logging::log("ERROR", &format!("Failed to open URL: {}", e));
                        }
                    }
                }
            }
        }
    }
    // NEW: Space for Quick Look (images)
    " " | "space" => {
        if let Some((_, entry)) = filtered_entries.get(*selected_index) {
            if entry.content_type == clipboard_history::ContentType::Image {
                if let Ok(temp_path) = clipboard_history::save_image_to_temp_file(&entry.id) {
                    // Use qlmanage for Quick Look
                    let _ = std::process::Command::new("qlmanage")
                        .args(["-p", temp_path.to_str().unwrap_or("")])
                        .spawn();
                }
            }
        }
    }
    _ => {
        // Character input for filter
        if let Some(ref key_char) = event.keystroke.key_char {
            if let Some(ch) = key_char.chars().next() {
                if !ch.is_control() && !event.keystroke.modifiers.command {
                    filter.push(ch);
                    *selected_index = 0;
                    this.clipboard_list_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                    cx.notify();
                }
            }
        }
    }
}
```

### Step 4: Add Helper Methods to ScriptListApp

```rust
// File: src/main.rs
// Location: Add these methods to impl ScriptListApp

/// Paste clipboard entry to active app (copy + hide + Cmd+V)
fn clipboard_paste_entry(&mut self, entry_id: &str, cx: &mut Context<Self>) {
    if let Err(e) = clipboard_history::copy_entry_to_clipboard(entry_id) {
        logging::log("ERROR", &format!("Failed to copy entry: {}", e));
        return;
    }
    
    WINDOW_VISIBLE.store(false, Ordering::SeqCst);
    cx.hide();
    NEEDS_RESET.store(true, Ordering::SeqCst);

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Err(e) = selected_text::simulate_paste_with_cg() {
            logging::log("ERROR", &format!("Failed to simulate paste: {}", e));
        }
    });
}

/// Show actions menu for a clipboard entry
fn show_clipboard_actions_menu(&mut self, entry: clipboard_history::ClipboardEntry, cx: &mut Context<Self>) {
    use crate::actions::get_clipboard_entry_actions;
    
    let actions = get_clipboard_entry_actions(&entry);
    
    // Create actions dialog with clipboard-specific actions
    let actions_dialog = actions::ActionsDialog::with_script_and_design(
        cx.focus_handle(),
        Arc::new({
            let entry_id = entry.id.clone();
            move |action_id| {
                // Handle action (will be called via callback)
                logging::log("ACTIONS", &format!("Clipboard action: {} for entry {}", action_id, entry_id));
            }
        }),
        None,
        self.theme.clone(),
        self.current_design,
    );
    
    // Set the actions on the dialog
    // TODO: Convert actions to ProtocolAction format or modify ActionsDialog to accept Action directly
    
    self.show_actions_overlay = true;
    cx.notify();
}
```

### Testing

After implementing these changes:

1. **Test Keyboard Shortcuts:**
   ```bash
   # Build and run
   cargo build && echo '{"type": "show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   
   # In the app:
   # - Type "clip" to filter and select "Clipboard History"
   # - Press Enter to open
   # - Navigate with Up/Down arrows
   # - Cmd+C to copy
   # - Cmd+D to delete
   # - Cmd+P to pin/unpin
   # - Cmd+Shift+V to paste as plain text
   # - Cmd+K to open actions menu
   ```

2. **Verify Actions Menu:**
   - Press Cmd+K on a text entry - should show paste, copy, paste plain, edit, delete, pin
   - Press Cmd+K on an image entry - should show paste, copy, quick look, open, save, delete, pin

3. **Test Image Actions:**
   - Select an image entry
   - Press Space for Quick Look
   - Press Cmd+O to open in Preview

---

## Instructions For The Next AI Agent

You are reading the "Clipboard History Raycast Feature Parity Expert Bundle". This file is self-contained and includes all the context you should assume you have.

Your job:
- Design and describe the minimal, safe changes needed to fully resolve the issues described in the Executive Summary and Key Problems.
- Operate **only** on the files and code snippets included in this bundle. If you need additional files or context, clearly say so.

When you propose changes, follow these rules strictly:

1. Always provide **precise code snippets** that can be copy-pasted directly into the repo.
2. Always include **exact file paths** (e.g. `src/clipboard_history.rs`) and, when possible, line numbers or a clear description of the location.
3. Never describe code changes only in prose. Show the full function or block as it should look **after** the change.
4. Keep instructions **unmistakable and unambiguous**. A human or tool following your instructions should not need to guess what to do.
5. Assume you cannot see any files outside this bundle. If you must rely on unknown code, explicitly note assumptions and risks.

When you answer, you do not need to restate this bundle. Work directly with the code and instructions it contains and return a clear, step-by-step plan plus exact code edits.
