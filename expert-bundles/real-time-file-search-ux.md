# Real-Time File Search UX Expert Bundle

## Original Goal

> I want to grab all the files related to the search files command so that we can get expert feedback on how to make this a real-time searching experience that has a nice animation and that provides a much more fluid, cleaner experience than what we currently have.

---

## Executive Summary

The file search feature uses macOS Spotlight (`mdfind`) as the backend but currently lacks smooth animations during search state transitions, loading feedback, and fluid result list updates. The UI updates are functional but abrupt - there's no skeleton loading, no result fade-in animations, and the loading state is a simple "Searching..." text.

### Key Problems:
1. **No animated transitions**: Results appear/disappear instantly without fade or slide animations. Loading state is just text with no visual interest.
2. **Debounce-only approach**: 200ms debounce for mdfind searches works but doesn't provide incremental feedback. Users see nothing during the debounce period.
3. **No skeleton loading**: Empty state → loading → results is jarring. Modern UIs use skeleton placeholders or shimmer effects.
4. **Static list rendering**: `uniform_list` renders items without entry animations. Selection changes are instant, not animated.
5. **Preview panel has no transitions**: File preview updates immediately on selection change with no cross-fade.

### Required Improvements:
1. **Add loading animation**: Implement a pulsing skeleton or shimmer effect during search (existing `transitions.rs` provides `Lerp` and easing functions).
2. **Animate result list entries**: Stagger fade-in animation for results as they arrive.
3. **Smooth selection transitions**: Use `HoverState` and `TransitionColor` from `transitions.rs` for smooth hover/selection color changes.
4. **Preview cross-fade**: Animate preview panel content changes with opacity transition.
5. **Consider streaming results**: mdfind can stream results - show partial results immediately rather than waiting for debounce.

### Files Included:
- `src/file_search.rs`: Core search logic, mdfind interface, FileResult types
- `src/transitions.rs`: Animation utilities (Lerp, easing, TransitionColor, Opacity, HoverState)
- `src/scripts/input_detection.rs`: Path detection for directory vs search mode
- `src/render_builtins.rs` (lines 2066-2659): `render_file_search()` UI rendering
- `src/app_impl.rs` (lines 2457-2625): FileSearchView state management, debouncing
- `src/app_execute.rs` (lines 1421-1483): `open_file_search()` entry point
- `src/main.rs` (excerpts): State struct definitions for file search

---

## CORE FILES (Full Content)

### src/file_search.rs

```rust
//! File Search Module using macOS Spotlight (mdfind)
//!
//! This module provides file search functionality using macOS's mdfind command,
//! which interfaces with the Spotlight index for fast file searching.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
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

/// Limit for interactive mdfind searches (500 results, <1s response)
pub const DEFAULT_SEARCH_LIMIT: usize = 500;

/// Cache limit for directory listing (fast, can handle 2000)
pub const DEFAULT_CACHE_LIMIT: usize = 2000;

fn looks_like_advanced_mdquery(q: &str) -> bool {
    let q = q.trim();
    q.contains("kMDItem") || q.contains("==") || q.contains("!=") 
        || q.contains(">=") || q.contains("<=") || q.contains("&&") || q.contains("||")
}

fn escape_md_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn build_mdquery(user_query: &str) -> String {
    let q = user_query.trim();
    if looks_like_advanced_mdquery(q) {
        return q.to_string();
    }
    format!(r#"kMDItemFSName == "*{}*"c"#, escape_md_string(q))
}

fn detect_file_type(path: &Path) -> FileType {
    let extension = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());

    if extension.as_deref() == Some("app") {
        return FileType::Application;
    }
    if path.is_dir() {
        return FileType::Directory;
    }

    match extension.as_deref() {
        Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "heic" | "heif") => FileType::Image,
        Some("pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt" | "ods" | "odp" | "pages" | "numbers" | "key") => FileType::Document,
        Some("mp3" | "wav" | "aac" | "flac" | "ogg" | "wma" | "m4a" | "aiff") => FileType::Audio,
        Some("mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" | "mpg") => FileType::Video,
        Some(_) => FileType::File,
        None => if path.is_dir() { FileType::Directory } else { FileType::File },
    }
}

/// Search for files using macOS mdfind (Spotlight)
/// Uses STREAMING to avoid buffering all results - key for real-time UX
#[instrument(skip_all, fields(query = %query, onlyin = ?onlyin, limit = limit))]
pub fn search_files(query: &str, onlyin: Option<&str>, limit: usize) -> Vec<FileResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let mdquery = build_mdquery(query);
    let mut cmd = Command::new("mdfind");

    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }
    cmd.arg(&mdquery);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(_) => return Vec::new(),
    };

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => { let _ = child.kill(); return Vec::new(); }
    };

    let reader = BufReader::new(stdout);
    let mut results = Vec::new();

    // STREAMING: Read line-by-line, stop at limit
    for line_result in reader.lines() {
        if results.len() >= limit { break; }
        let line = match line_result { Ok(l) => l, Err(_) => continue };
        if line.is_empty() { continue; }

        let path = Path::new(&line);
        let (size, modified) = std::fs::metadata(path)
            .map(|m| (m.len(), m.modified().ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs()).unwrap_or(0)))
            .unwrap_or((0, 0));

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let file_type = detect_file_type(path);

        results.push(FileResult { path: line, name, size, modified, file_type });
    }

    if results.len() >= limit { let _ = child.kill(); }
    let _ = child.wait();
    results
}

/// List directory contents (fast, no mdfind)
pub fn list_directory(dir_path: &str, _limit: usize) -> Vec<FileResult> {
    let expanded = match expand_path(dir_path) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let path = Path::new(&expanded);
    if !path.is_dir() { return Vec::new(); }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut results: Vec<FileResult> = entries.flatten()
        .filter_map(|entry| {
            let entry_path = entry.path();
            let path_str = entry_path.to_str()?.to_string();
            let name = entry_path.file_name()?.to_str()?.to_string();
            if name.starts_with('.') { return None; }
            
            let (size, modified) = std::fs::metadata(&entry_path)
                .map(|m| (m.len(), m.modified().ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()).unwrap_or(0)))
                .unwrap_or((0, 0));
            
            Some(FileResult { path: path_str, name, size, modified, file_type: detect_file_type(&entry_path) })
        })
        .collect();

    // Sort: directories first, then alphabetically
    results.sort_by(|a, b| {
        let a_dir = matches!(a.file_type, FileType::Directory);
        let b_dir = matches!(b.file_type, FileType::Directory);
        match (a_dir, b_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    if results.len() > 5000 { results.truncate(5000); }
    results
}

/// UI icon for file type
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

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB { format!("{:.1} GB", bytes as f64 / GB as f64) }
    else if bytes >= MB { format!("{:.1} MB", bytes as f64 / MB as f64) }
    else if bytes >= KB { format!("{:.1} KB", bytes as f64 / KB as f64) }
    else { format!("{} B", bytes) }
}

pub fn format_relative_time(unix_timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if unix_timestamp == 0 { return "Unknown".to_string(); }
    let diff = now.saturating_sub(unix_timestamp);
    
    const MINUTE: u64 = 60; const HOUR: u64 = 3600; const DAY: u64 = 86400;
    if diff < MINUTE { "Just now".to_string() }
    else if diff < HOUR { format!("{} mins ago", diff / MINUTE) }
    else if diff < DAY { format!("{} hours ago", diff / HOUR) }
    else { format!("{} days ago", diff / DAY) }
}

pub fn shorten_path(path: &str) -> String {
    dirs::home_dir()
        .and_then(|h| h.to_str().map(|s| s.to_string()))
        .and_then(|home| path.strip_prefix(&home).map(|rest| format!("~{}", rest)))
        .unwrap_or_else(|| path.to_string())
}

pub fn expand_path(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() { return None; }
    if trimmed == "~" { return dirs::home_dir().and_then(|p| p.to_str().map(|s| s.to_string())); }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        return dirs::home_dir().and_then(|h| h.join(rest).to_str().map(|s| s.to_string()));
    }
    if trimmed.starts_with('/') { return Some(trimmed.to_string()); }
    None
}

/// Check if query is a directory path (triggers directory listing mode)
pub use crate::scripts::input_detection::is_directory_path;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDirPath {
    pub directory: String,
    pub filter: Option<String>,
}

/// Parse directory path with optional filter (e.g., ~/dev/fin -> directory=~/dev/, filter=Some("fin"))
pub fn parse_directory_path(path: &str) -> Option<ParsedDirPath> {
    let trimmed = path.trim();
    if !is_directory_path(trimmed) { return None; }
    
    if trimmed == "~" || trimmed == "~/" {
        return Some(ParsedDirPath { directory: "~".to_string(), filter: None });
    }
    
    if trimmed.ends_with('/') {
        if let Some(expanded) = expand_path(trimmed.trim_end_matches('/')) {
            if Path::new(&expanded).is_dir() {
                return Some(ParsedDirPath { directory: trimmed.to_string(), filter: None });
            }
        }
        return None;
    }
    
    if let Some(last_slash) = trimmed.rfind('/') {
        let parent = &trimmed[..=last_slash];
        let filter = &trimmed[last_slash + 1..];
        let parent_check = if parent == "/" { "/" } else { parent.trim_end_matches('/') };
        
        if let Some(expanded) = expand_path(parent_check) {
            if Path::new(&expanded).is_dir() {
                return Some(ParsedDirPath {
                    directory: parent.to_string(),
                    filter: if filter.is_empty() { None } else { Some(filter.to_string()) },
                });
            }
        }
    }
    None
}

/// Fuzzy filter using Nucleo (high-performance)
pub fn filter_results_nucleo_simple<'a>(results: &'a [FileResult], filter_pattern: &str) -> Vec<(usize, &'a FileResult)> {
    use crate::scripts::NucleoCtx;
    
    let mut nucleo = NucleoCtx::new(filter_pattern);
    let mut scored: Vec<(usize, &FileResult, u32)> = results.iter().enumerate()
        .filter_map(|(idx, r)| nucleo.score(&r.name).map(|score| (idx, r, score)))
        .collect();
    scored.sort_by(|a, b| b.2.cmp(&a.2));
    scored.into_iter().map(|(idx, r, _)| (idx, r)).collect()
}

// File actions
pub fn open_file(path: &str) -> Result<(), String> {
    Command::new("open").arg(path).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reveal_in_finder(path: &str) -> Result<(), String> {
    Command::new("open").args(["-R", path]).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn quick_look(path: &str) -> Result<(), String> {
    Command::new("qlmanage").args(["-p", path]).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
```

### src/transitions.rs (Animation Utilities)

```rust
//! UI Transitions Module - Provides transition helpers for smooth UI animations.

use gpui::Rgba;
use std::time::Duration;

/// Linear interpolation trait
pub trait Lerp {
    fn lerp(&self, to: &Self, delta: f32) -> Self;
}

// Standard durations
pub const DURATION_FAST: Duration = Duration::from_millis(100);
pub const DURATION_STANDARD: Duration = Duration::from_millis(150);
pub const DURATION_MEDIUM: Duration = Duration::from_millis(200);
pub const DURATION_SLOW: Duration = Duration::from_millis(300);

// Easing functions
pub fn linear(t: f32) -> f32 { t }
pub fn ease_out_quad(t: f32) -> f32 { 1.0 - (1.0 - t) * (1.0 - t) }
pub fn ease_in_quad(t: f32) -> f32 { t * t }
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}
pub fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }

impl Lerp for f32 {
    fn lerp(&self, to: &Self, delta: f32) -> Self { self + (to - self) * delta }
}

/// Color with lerp support for smooth transitions
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionColor { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

impl TransitionColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    pub fn from_hex_alpha(hex: u32, alpha: f32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as f32 / 255.0,
            g: ((hex >> 8) & 0xFF) as f32 / 255.0,
            b: (hex & 0xFF) as f32 / 255.0,
            a: alpha,
        }
    }
    pub fn transparent() -> Self { Self::new(0.0, 0.0, 0.0, 0.0) }
    pub fn to_rgba(self) -> Rgba { Rgba { r: self.r, g: self.g, b: self.b, a: self.a } }
}

impl Lerp for TransitionColor {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self {
            r: self.r + (to.r - self.r) * delta,
            g: self.g + (to.g - self.g) * delta,
            b: self.b + (to.b - self.b) * delta,
            a: self.a + (to.a - self.a) * delta,
        }
    }
}

/// Opacity for fade transitions
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(pub f32);

impl Opacity {
    pub const INVISIBLE: Self = Self(0.0);
    pub const VISIBLE: Self = Self(1.0);
    pub fn new(value: f32) -> Self { Self(value.clamp(0.0, 1.0)) }
}

impl Lerp for Opacity {
    fn lerp(&self, to: &Self, delta: f32) -> Self { Self(self.0 + (to.0 - self.0) * delta) }
}

/// Slide offset for animations
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SlideOffset { pub x: f32, pub y: f32 }

impl SlideOffset {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub fn from_bottom(amount: f32) -> Self { Self { x: 0.0, y: amount } }
    pub fn from_top(amount: f32) -> Self { Self { x: 0.0, y: -amount } }
}

impl Lerp for SlideOffset {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self { x: self.x + (to.x - self.x) * delta, y: self.y + (to.y - self.y) * delta }
    }
}

/// Combined opacity + slide for appear animations (toasts, results)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppearTransition { pub opacity: Opacity, pub offset: SlideOffset }

impl AppearTransition {
    pub fn hidden() -> Self { Self { opacity: Opacity::INVISIBLE, offset: SlideOffset::from_bottom(20.0) } }
    pub fn visible() -> Self { Self { opacity: Opacity::VISIBLE, offset: SlideOffset::ZERO } }
    pub fn dismissed() -> Self { Self { opacity: Opacity::INVISIBLE, offset: SlideOffset::from_top(10.0) } }
}

impl Lerp for AppearTransition {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self { opacity: self.opacity.lerp(&to.opacity, delta), offset: self.offset.lerp(&to.offset, delta) }
    }
}

/// Hover state for list items with background color transitions
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverState { pub background: TransitionColor }

impl HoverState {
    pub fn normal() -> Self { Self { background: TransitionColor::transparent() } }
    pub fn with_background(color: TransitionColor) -> Self { Self { background: color } }
}

impl Lerp for HoverState {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self { background: self.background.lerp(&to.background, delta) }
    }
}
```

### src/scripts/input_detection.rs (Path Detection)

```rust
//! Input detection for smart fallback commands (Raycast-style)

use regex::Regex;
use std::sync::LazyLock;

pub enum InputType { Url, FilePath, MathExpression, CodeSnippet, PlainText }

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| 
    Regex::new(r"^(https?://|file://)[^\s]+$").unwrap()
);

/// Check if input looks like a directory path (triggers directory listing mode)
pub fn is_directory_path(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() { return false; }
    
    // Home paths: ~ or ~/...
    if trimmed == "~" || trimmed.starts_with("~/") { return true; }
    // Absolute paths: /...
    if trimmed.starts_with('/') { return true; }
    // Relative: . or ./... or .. or ../...
    if trimmed == "." || trimmed.starts_with("./") { return true; }
    if trimmed == ".." || trimmed.starts_with("../") { return true; }
    
    false
}

pub fn is_file_path(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() { return false; }
    if trimmed.starts_with('/') || trimmed.starts_with("~/") { return true; }
    if trimmed.starts_with("./") || trimmed.starts_with("../") { return true; }
    // Windows: C:\ or D:/
    let chars: Vec<char> = trimmed.chars().collect();
    chars.len() >= 3 && chars[0].is_ascii_alphabetic() && chars[1] == ':' && (chars[2] == '\\' || chars[2] == '/')
}
```

---

## EXCERPT: UI Rendering (render_builtins.rs lines 2066-2659)

```rust
/// Render file search view with 50/50 split (list + preview)
pub(crate) fn render_file_search(
    &mut self,
    query: &str,
    selected_index: usize,
    cx: &mut Context<Self>,
) -> AnyElement {
    use crate::file_search::{self, FileType};
    
    let tokens = get_tokens(self.current_design);
    let design_spacing = tokens.spacing();
    let design_visual = tokens.visual();
    let box_shadows = self.create_box_shadows();

    // Theme colors for closures
    let text_primary = self.theme.colors.text.primary;
    let text_muted = self.theme.colors.text.muted;
    let text_dimmed = self.theme.colors.text.dimmed;
    let ui_border = self.theme.colors.ui.border;
    let list_hover = self.theme.colors.accent.selected_subtle;
    let list_selected = self.theme.colors.accent.selected_subtle;
    let opacity = self.theme.get_opacity();
    let selected_alpha = (opacity.selected * 255.0) as u32;
    let hover_alpha = (opacity.hover * 255.0) as u32;

    // Filter results using Nucleo fuzzy matching
    let filter_pattern = if let Some(parsed) = crate::file_search::parse_directory_path(query) {
        parsed.filter
    } else if !query.is_empty() {
        Some(query.to_string())
    } else {
        None
    };

    let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
        file_search::filter_results_nucleo_simple(&self.cached_file_results, pattern)
    } else {
        self.cached_file_results.iter().enumerate().collect()
    };
    let filtered_len = filtered_results.len();
    let selected_file = filtered_results.get(selected_index).map(|(_, r)| (*r).clone());

    let is_loading = self.file_search_loading;

    // Clone for uniform_list closure
    let files_for_closure: Vec<_> = filtered_results.iter().map(|(_, f)| (*f).clone()).collect();
    let current_selected = selected_index;

    // PROBLEM: List items appear instantly, no animation
    let list_element = if filtered_len == 0 {
        div().w_full().py(px(design_spacing.padding_xl)).text_center()
            .text_color(rgb(text_dimmed))
            .child(if query.is_empty() { "Type to search files" } else { "No files found" })
            .into_any_element()
    } else {
        uniform_list("file-search-list", filtered_len, move |visible_range, _w, _cx| {
            visible_range.map(|ix| {
                if let Some(file) = files_for_closure.get(ix) {
                    let is_selected = ix == current_selected;
                    // PROBLEM: Instant selection, no transition
                    let bg = if is_selected {
                        rgba((list_selected << 8) | selected_alpha)
                    } else {
                        rgba(0x00000000)
                    };
                    let hover_bg = rgba((list_hover << 8) | hover_alpha);

                    div().id(ix).w_full().h(px(52.)).flex().flex_row().items_center()
                        .px(px(12.)).gap(px(12.)).bg(bg).hover(move |s| s.bg(hover_bg))
                        .child(div().text_lg().text_color(rgb(text_muted))
                            .child(file_search::file_type_icon(file.file_type)))
                        .child(div().flex_1().flex().flex_col().gap(px(2.))
                            .child(div().text_sm().text_color(rgb(text_primary)).child(file.name.clone()))
                            .child(div().text_xs().text_color(rgb(text_dimmed))
                                .child(file_search::shorten_path(&file.path))))
                        .child(div().flex().flex_col().items_end().gap(px(2.))
                            .child(div().text_xs().text_color(rgb(text_dimmed))
                                .child(file_search::format_file_size(file.size)))
                            .child(div().text_xs().text_color(rgb(text_dimmed))
                                .child(file_search::format_relative_time(file.modified))))
                } else {
                    div().id(ix).h(px(52.))
                }
            }).collect()
        }).h_full().track_scroll(&self.file_search_scroll_handle).into_any_element()
    };

    // PROBLEM: Preview updates instantly, no cross-fade
    let preview_content = if let Some(file) = &selected_file {
        // ... preview content rendering (omitted for brevity)
        div().flex_1().flex().flex_col().p(px(design_spacing.padding_lg))
            .child(/* file details */)
    } else if is_loading {
        // PROBLEM: Just empty, no skeleton/shimmer
        div().flex_1()
    } else {
        div().flex_1().flex().items_center().justify_center()
            .child(div().text_sm().text_color(rgb(text_dimmed)).child("No file selected"))
    };

    // Main container
    div().key_context("FileSearchView")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .w_full().h_full().flex().flex_col()
        .shadow(box_shadows)
        .rounded(px(design_visual.radius_lg))
        // Header with input
        .child(/* search input header */)
        // Divider
        .child(div().mx(px(design_spacing.padding_lg)).h(px(design_visual.border_thin))
            .bg(rgba((ui_border << 8) | 0x60)))
        // PROBLEM: Loading state is just text "Searching..."
        .child(if is_loading && filtered_len == 0 {
            div().flex_1().w_full().flex().items_center().justify_center()
                .child(div().text_sm().text_color(rgb(text_dimmed)).child("Searching..."))
        } else if filtered_len == 0 {
            // Empty state
            div().flex_1().w_full().flex().items_center().justify_center()
                .child(div().text_color(rgb(text_dimmed)).child("No files found"))
        } else {
            // 50/50 split: list + preview
            div().flex_1().w_full().flex().flex_row().min_h(px(0.)).overflow_hidden()
                .child(div().flex_1().h_full().overflow_hidden()
                    .border_r(px(design_visual.border_thin))
                    .border_color(rgba((ui_border << 8) | 0x40))
                    .child(list_element))
                .child(div().flex_1().h_full().overflow_hidden().child(preview_content))
        })
        // Footer
        .child(PromptFooter::new(/* ... */))
        .into_any_element()
}
```

---

## EXCERPT: State Management (app_impl.rs lines 2457-2625)

```rust
AppView::FileSearchView { query, selected_index } => {
    if *query != new_text {
        // Get old filter BEFORE updating (for frozen filter during transitions)
        let old_filter = if let Some(old_parsed) = crate::file_search::parse_directory_path(query) {
            old_parsed.filter
        } else if !query.is_empty() {
            Some(query.clone())
        } else {
            None
        };

        *query = new_text.clone();
        *selected_index = 0;
        self.file_search_debounce_task = None;

        // Directory path mode: instant filtering on cached results
        if let Some(parsed) = crate::file_search::parse_directory_path(&new_text) {
            let dir_changed = self.file_search_current_dir.as_ref() != Some(&parsed.directory);

            if dir_changed {
                // Directory changed - load new contents
                self.file_search_frozen_filter = Some(old_filter);  // Keep old results during load
                self.file_search_current_dir = Some(parsed.directory.clone());
                self.file_search_loading = true;
                cx.notify();

                // Debounced directory listing (50ms)
                let dir_to_list = parsed.directory.clone();
                let task = cx.spawn(async move |this, cx| {
                    Timer::after(Duration::from_millis(50)).await;
                    // Background thread for I/O
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let results = crate::file_search::list_directory(&dir_to_list, 2000);
                        let _ = tx.send(results);
                    });
                    // Poll for results
                    loop {
                        Timer::after(Duration::from_millis(10)).await;
                        match rx.try_recv() {
                            Ok(results) => {
                                let _ = cx.update(|cx| {
                                    this.update(cx, |app, cx| {
                                        app.cached_file_results = results;
                                        app.file_search_loading = false;
                                        app.file_search_frozen_filter = None;
                                        if let AppView::FileSearchView { selected_index, .. } = &mut app.current_view {
                                            *selected_index = 0;
                                        }
                                        app.file_search_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                                        cx.notify();
                                    })
                                });
                                break;
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                            Err(_) => break,
                        }
                    }
                });
                self.file_search_debounce_task = Some(task);
            } else {
                // Same directory - instant filter (no loading)
                self.file_search_frozen_filter = None;
                self.file_search_loading = false;
                cx.notify();
            }
            return;
        }

        // Not a directory - do mdfind search with 200ms debounce
        // PROBLEM: 200ms delay before any feedback
        self.file_search_current_dir = None;
        self.file_search_loading = true;
        cx.notify();

        let search_query = new_text.clone();
        let task = cx.spawn(async move |this, cx| {
            Timer::after(Duration::from_millis(200)).await;  // Debounce
            
            let (tx, rx) = std::sync::mpsc::channel();
            let q = search_query.clone();
            std::thread::spawn(move || {
                let results = crate::file_search::search_files(&q, None, 500);
                let _ = tx.send(results);
            });
            
            loop {
                Timer::after(Duration::from_millis(10)).await;
                match rx.try_recv() {
                    Ok(results) => {
                        let _ = cx.update(|cx| {
                            this.update(cx, |app, cx| {
                                if let AppView::FileSearchView { query, .. } = &app.current_view {
                                    if *query == search_query {
                                        app.cached_file_results = results;
                                        app.file_search_loading = false;
                                        if let AppView::FileSearchView { selected_index, .. } = &mut app.current_view {
                                            *selected_index = 0;
                                        }
                                        app.file_search_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                                        cx.notify();
                                    }
                                }
                            })
                        });
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                    Err(_) => break,
                }
            }
        });
        self.file_search_debounce_task = Some(task);
    }
    return;
}
```

---

## EXCERPT: State Definitions (main.rs)

```rust
/// App state fields for file search
struct App {
    // ... other fields ...
    
    /// Cached file results for FileSearchView (avoids cloning per frame)
    cached_file_results: Vec<file_search::FileResult>,
    
    /// Scroll handle for file search list
    file_search_scroll_handle: UniformListScrollHandle,
    
    /// Loading state (true while mdfind is running)
    file_search_loading: bool,
    
    /// Debounce task (cancelled when new input arrives)
    file_search_debounce_task: Option<gpui::Task<()>>,
    
    /// Current directory being listed (for instant filter mode)
    file_search_current_dir: Option<String>,
    
    /// Frozen filter during directory transitions (prevents flash)
    file_search_frozen_filter: Option<Option<String>>,
    
    /// Path of file selected for actions
    file_search_actions_path: Option<String>,
}

/// AppView enum
enum AppView {
    // ... other variants ...
    
    /// Showing file search results
    FileSearchView {
        query: String,
        selected_index: usize,
    },
}
```

---

## Implementation Guide

### Step 1: Add Loading Animation State

```rust
// File: src/main.rs - Add to App struct
struct App {
    // ... existing fields ...
    
    /// Animation progress for search loading (0.0 = hidden, 1.0 = visible)
    file_search_loading_progress: f32,
    /// Animation start time for loading shimmer
    file_search_loading_start: Option<Instant>,
    /// Per-result animation progress (for staggered entry)
    file_search_result_animations: Vec<f32>,
}
```

### Step 2: Implement Skeleton/Shimmer Loading

```rust
// File: src/render_builtins.rs - Add skeleton component
fn render_skeleton_item(shimmer_progress: f32, colors: &Colors) -> impl IntoElement {
    use crate::transitions::{ease_in_out_quad, TransitionColor};
    
    // Animate shimmer from left to right
    let shimmer_offset = ease_in_out_quad(shimmer_progress) * 100.0;
    
    div()
        .w_full()
        .h(px(52.))
        .flex()
        .flex_row()
        .items_center()
        .px(px(12.))
        .gap(px(12.))
        // Icon placeholder
        .child(
            div()
                .w(px(24.))
                .h(px(24.))
                .rounded(px(4.))
                .bg(rgba((colors.ui.border << 8) | 0x40))
        )
        // Text placeholder with shimmer
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .w(px(120.))
                        .h(px(14.))
                        .rounded(px(2.))
                        .bg(rgba((colors.ui.border << 8) | 0x30))
                        // Shimmer overlay would be applied via CSS animation
                )
                .child(
                    div()
                        .w(px(200.))
                        .h(px(10.))
                        .rounded(px(2.))
                        .bg(rgba((colors.ui.border << 8) | 0x20))
                )
        )
}

// In render_file_search, replace loading state:
.child(if is_loading && filtered_len == 0 {
    // Show 5 skeleton items while loading
    div()
        .flex_1()
        .w_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .children((0..5).map(|i| {
            // Stagger shimmer animation
            let shimmer = ((animation_time + i as f32 * 0.1) % 1.0);
            render_skeleton_item(shimmer, &self.theme.colors)
        }))
} else { /* existing code */ })
```

### Step 3: Add Result Entry Animations

```rust
// File: src/render_builtins.rs - Animate list item entry
fn render_file_item_animated(
    file: &FileResult,
    ix: usize,
    is_selected: bool,
    entry_progress: f32,  // 0.0 = hidden, 1.0 = visible
    colors: &ListItemColors,
) -> impl IntoElement {
    use crate::transitions::{ease_out_cubic, Opacity, SlideOffset};
    
    // Apply easing
    let t = ease_out_cubic(entry_progress);
    
    // Fade and slide from bottom
    let opacity = Opacity::INVISIBLE.lerp(&Opacity::VISIBLE, t);
    let offset = SlideOffset::from_bottom(10.0).lerp(&SlideOffset::ZERO, t);
    
    div()
        .id(ix)
        .w_full()
        .h(px(52.))
        .opacity(opacity.0)
        .transform(format!("translateY({}px)", offset.y))  // Note: GPUI may need different API
        // ... rest of item rendering
}
```

### Step 4: Smooth Selection Transitions

```rust
// File: src/main.rs - Track selection animation state
struct App {
    // Selection animation
    file_search_selection_animation: Option<SelectionAnimation>,
}

struct SelectionAnimation {
    from_index: usize,
    to_index: usize,
    progress: f32,  // 0.0 to 1.0
    start_time: Instant,
}

// In selection change handler:
fn handle_selection_change(&mut self, new_index: usize, cx: &mut Context<Self>) {
    if let Some(old_index) = self.get_selected_index() {
        self.file_search_selection_animation = Some(SelectionAnimation {
            from_index: old_index,
            to_index: new_index,
            progress: 0.0,
            start_time: Instant::now(),
        });
        // Schedule animation frame updates
        cx.spawn(|this, mut cx| async move {
            while let Some(anim) = this.read(&cx).file_search_selection_animation {
                if anim.progress >= 1.0 { break; }
                Timer::after(Duration::from_millis(16)).await;  // ~60fps
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        if let Some(anim) = &mut app.file_search_selection_animation {
                            let elapsed = anim.start_time.elapsed().as_secs_f32();
                            anim.progress = (elapsed / 0.15).min(1.0);  // 150ms duration
                            cx.notify();
                        }
                    })
                });
            }
        }).detach();
    }
}
```

### Step 5: Preview Cross-Fade

```rust
// Track preview transition
struct PreviewTransition {
    old_file: Option<FileResult>,
    new_file: Option<FileResult>,
    progress: f32,
}

// In render, layer both previews with opacity
.child({
    let transition = &self.preview_transition;
    div().flex_1().relative()
        // Old content fading out
        .when_some(transition.as_ref().and_then(|t| t.old_file.as_ref()), |d, old| {
            d.child(
                render_preview(old)
                    .absolute().inset_0()
                    .opacity(1.0 - transition.map(|t| t.progress).unwrap_or(1.0))
            )
        })
        // New content fading in
        .when_some(selected_file.as_ref(), |d, file| {
            d.child(
                render_preview(file)
                    .opacity(transition.map(|t| t.progress).unwrap_or(1.0))
            )
        })
})
```

### Step 6: Consider Streaming Results

```rust
// Instead of waiting for all results, show them as they arrive
pub fn search_files_streaming<F>(query: &str, limit: usize, on_result: F)
where
    F: Fn(FileResult) + Send + 'static,
{
    // ... spawn mdfind ...
    for line in reader.lines() {
        // Parse and immediately callback
        if let Ok(result) = parse_result(&line) {
            on_result(result);
        }
    }
}

// Usage in UI:
let task = cx.spawn(async move |this, cx| {
    file_search::search_files_streaming(&query, 500, |result| {
        // Update UI incrementally
        let _ = cx.update(|cx| {
            this.update(cx, |app, cx| {
                app.cached_file_results.push(result);
                cx.notify();
            })
        });
    });
});
```

---

## Instructions for Next AI Agent

You are implementing a **real-time file search UX improvement** for Script Kit GPUI. The goal is to make the file search experience feel **fluid, animated, and responsive** like Raycast or Alfred.

### Context
- This is a Rust GPUI application (similar to Zed editor)
- File search uses macOS Spotlight (`mdfind`) as backend
- Current implementation works but feels static/abrupt

### Your Tasks
1. **Add skeleton loading**: When `file_search_loading` is true, render shimmer/skeleton placeholders instead of "Searching..." text
2. **Animate result entries**: When results arrive, stagger their appearance with fade+slide animations using `AppearTransition` from `transitions.rs`
3. **Smooth selection**: Use `HoverState` and `TransitionColor` to animate selection highlight changes
4. **Preview cross-fade**: Animate preview panel content changes with opacity transition
5. **Consider streaming**: The current 200ms debounce can be replaced with immediate streaming to show results as they arrive from mdfind

### Key Files to Modify
- `src/main.rs`: Add animation state fields to `App` struct
- `src/render_builtins.rs`: Update `render_file_search()` to use animations
- `src/app_impl.rs`: Wire up animation triggers in state handlers

### GPUI Animation Pattern
GPUI doesn't have built-in animation primitives. You need to:
1. Store animation progress in state
2. Use `cx.spawn()` to schedule frame updates (~60fps)
3. Call `cx.notify()` to trigger re-render
4. Apply easing from `transitions.rs` to progress values
5. Use lerped values for opacity/transform in render

### Testing
Run the app and type in the file search to verify:
```bash
cargo build && echo '{"type":"run","path":""}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui
```
Then trigger file search via the "Search Files" builtin.

### Success Criteria
- Skeleton shimmer visible during loading (not just text)
- Results fade/slide in when they appear
- Selection changes are animated (not instant)
- Preview panel cross-fades between files
- Overall feel is smooth and polished

---

</files>
