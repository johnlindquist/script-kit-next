use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FocusHandle, Focusable, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Render, ScrollDelta, ScrollWheelEvent, SharedString,
    Timer, Window,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, trace, warn};
use crate::config::Config;
use crate::prompts::SubmitCallback;
use crate::terminal::{
    CellAttributes, TerminalAction, TerminalContent, TerminalEvent, TerminalHandle,
};
use crate::theme::Theme;
const SLOW_RENDER_THRESHOLD_MS: u128 = 16; // 60fps threshold

/// Base font size for calculating ratios
const BASE_FONT_SIZE: f32 = 14.0;
/// Line height multiplier - 1.3 provides room for descenders (g, y, p, q, j)
/// and ascenders while keeping text readable
const LINE_HEIGHT_MULTIPLIER: f32 = 1.3;
/// Terminal cell dimensions at base font size
/// Cell width for Menlo 14pt is 8.4287px (measured). We use a slightly larger value
/// to be conservative and prevent the last character from wrapping to the next line.
/// Using 8.5px ensures we never tell the PTY we have more columns than can render.
const BASE_CELL_WIDTH: f32 = 8.5; // Conservative value for Menlo 14pt (actual: 8.4287px)
/// Default cell height at base font size (used for tests and static calculations)
const BASE_CELL_HEIGHT: f32 = BASE_FONT_SIZE * LINE_HEIGHT_MULTIPLIER; // 18.2px for 14pt

// Aliases for backwards compatibility with tests
#[allow(dead_code)]
const CELL_WIDTH: f32 = BASE_CELL_WIDTH;
#[allow(dead_code)]
const CELL_HEIGHT: f32 = BASE_CELL_HEIGHT;
/// Terminal refresh interval (ms) - 30fps is plenty for terminal output
const REFRESH_INTERVAL_MS: u64 = 16; // ~60fps, matches modern GPU-accelerated terminals

/// Minimum terminal size
const MIN_COLS: u16 = 20;
const MIN_ROWS: u16 = 5;
/// Duration for bell visual flash
const BELL_FLASH_DURATION_MS: u64 = 150;
/// Truncate a string to at most `max_bytes` bytes, ensuring the result is valid UTF-8.
/// Truncates at a character boundary, never in the middle of a multibyte character.
fn truncate_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // Find the last valid character boundary at or before max_bytes
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
/// Terminal prompt GPUI component
pub struct TermPrompt {
    pub id: String,
    pub terminal: TerminalHandle,
    pub focus_handle: FocusHandle,
    pub on_submit: SubmitCallback,
    pub theme: Arc<Theme>,
    pub config: Arc<Config>,
    exited: bool,
    exit_code: Option<i32>,
    /// Whether the refresh timer is active
    refresh_timer_active: bool,
    /// Last known terminal size (cols, rows)
    last_size: (u16, u16),
    /// Explicit content height - GPUI entities don't inherit parent flex sizing
    content_height: Option<Pixels>,
    /// Time until which the bell flash should be visible
    bell_flash_until: Option<Instant>,
    /// Terminal title from OSC escape sequences
    title: Option<String>,
    /// Whether mouse is currently dragging for selection
    is_selecting: bool,
    /// Start position of mouse selection (in terminal grid coordinates: col, row)
    selection_start: Option<(usize, usize)>,
    /// Time of last mouse click for multi-click detection
    last_click_time: Option<Instant>,
    /// Position of last mouse click (col, row)
    last_click_position: Option<(usize, usize)>,
    /// Count of rapid clicks at same position (1=single, 2=double, 3=triple)
    click_count: u8,
    /// When true, ignore all key events (used when actions panel is open)
    pub suppress_keys: bool,
}
