//! Default configuration values
//!
//! All constants used throughout the config module are defined here.

/// Default padding values for content areas
pub const DEFAULT_PADDING_TOP: f32 = 8.0;
pub const DEFAULT_PADDING_LEFT: f32 = 12.0;
pub const DEFAULT_PADDING_RIGHT: f32 = 12.0;

/// Default font sizes
pub const DEFAULT_EDITOR_FONT_SIZE: f32 = 16.0;
pub const DEFAULT_TERMINAL_FONT_SIZE: f32 = 14.0;

/// Default UI scale
#[cfg(test)]
pub const DEFAULT_UI_SCALE: f32 = 1.0;

/// Default launcher/layout heights (pixels)
pub const DEFAULT_LAYOUT_STANDARD_HEIGHT: f32 = 500.0;
pub const DEFAULT_LAYOUT_MAX_HEIGHT: f32 = 700.0;

/// Default built-in feature flags
pub const DEFAULT_CLIPBOARD_HISTORY: bool = true;
pub const DEFAULT_APP_LAUNCHER: bool = true;
pub const DEFAULT_WINDOW_SWITCHER: bool = true;
pub const DEFAULT_AI_HOTKEY_ENABLED: bool = true;
pub const DEFAULT_LOGS_HOTKEY_ENABLED: bool = true;
pub const DEFAULT_DICTATION_HOTKEY_ENABLED: bool = true;

/// Default unified root-search feature flags.
pub const DEFAULT_UNIFIED_SEARCH_ENABLED: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_FILES_ENABLED: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_FILES_GLOBAL_SEARCH: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_FILES_RECENT_FILES: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_FILES_DIRECTORY_BROWSE: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_ENABLED: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_MAX_RESULTS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_MIN_QUERY_CHARS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_NOTES_ENABLED: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_NOTES_MAX_RESULTS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_NOTES_MIN_QUERY_CHARS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_NOTES_SEARCH_CONTENT: bool = true;
pub const DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_ENABLED: bool = false;
pub const DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_MAX_RESULTS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_MIN_QUERY_CHARS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_SCAN_LIMIT: usize = 200;
pub const DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_ENABLED: bool = false;
pub const DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MAX_RESULTS: usize = 3;
pub const DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MIN_QUERY_CHARS: usize = 4;
pub const DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_MAX_AGE_DAYS: u32 = 90;
pub const DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_SEARCH_URLS: bool = true;

/// Default max text length for clipboard history entries (bytes)
pub const DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH: usize = 100_000;

/// Default process limits
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 5000;

/// Default watcher tuning values
pub const DEFAULT_WATCHER_DEBOUNCE_MS: u64 = 500;
pub const DEFAULT_WATCHER_STORM_THRESHOLD: usize = 200;
pub const DEFAULT_WATCHER_INITIAL_BACKOFF_MS: u64 = 100;
pub const DEFAULT_WATCHER_MAX_BACKOFF_MS: u64 = 30_000;
pub const DEFAULT_WATCHER_MAX_NOTIFY_ERRORS: u32 = 10;

/// Default suggested section settings
pub const DEFAULT_SUGGESTED_ENABLED: bool = true;
pub const DEFAULT_SUGGESTED_MAX_ITEMS: usize = 10;
pub const DEFAULT_SUGGESTED_MIN_SCORE: f64 = 0.1;
pub const DEFAULT_SUGGESTED_HALF_LIFE_DAYS: f64 = 7.0;
pub const DEFAULT_SUGGESTED_TRACK_USAGE: bool = true;

/// Commands that require confirmation before execution by default.
/// Users can override this behavior per-command in config.ts using `confirmationRequired`.
pub const DEFAULT_CONFIRMATION_COMMANDS: &[&str] = &[
    "builtin/shut-down",
    "builtin/restart",
    "builtin/log-out",
    "builtin/empty-trash",
    "builtin/sleep",
    "builtin/quit-script-kit",
    "builtin/force-quit",
    "builtin/stop-all-processes",
    "builtin/clear-suggested",
    "builtin/sync-to-github",
    "builtin/test-confirmation", // Dev test item
];

/// Commands that should be excluded from frecency/suggested tracking.
/// These are commands that don't make sense to suggest (e.g., quit).
pub const DEFAULT_FRECENCY_EXCLUDED_COMMANDS: &[&str] = &["builtin/quit-script-kit"];

// ============================================
// CLAUDE CODE CLI DEFAULTS
// ============================================

/// Whether Claude Code CLI provider is enabled (default: false, requires opt-in)
pub const DEFAULT_CLAUDE_CODE_ENABLED: bool = false;
/// Default permission mode for Claude Code CLI ("plan" is safe, "dontAsk" for sandbox)
pub const DEFAULT_CLAUDE_CODE_PERMISSION_MODE: &str = "plan";
