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
pub const DEFAULT_UI_SCALE: f32 = 1.0;

/// Default built-in feature flags
pub const DEFAULT_CLIPBOARD_HISTORY: bool = true;
pub const DEFAULT_APP_LAUNCHER: bool = true;
pub const DEFAULT_WINDOW_SWITCHER: bool = true;

/// Default max text length for clipboard history entries (bytes)
pub const DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH: usize = 100_000;

/// Default process limits
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 5000;

/// Default frecency settings
pub const DEFAULT_FRECENCY_HALF_LIFE_DAYS: f64 = 7.0;
pub const DEFAULT_FRECENCY_MAX_RECENT_ITEMS: usize = 10;
pub const DEFAULT_FRECENCY_ENABLED: bool = true;

/// Commands that require confirmation before execution by default.
/// Users can override this behavior per-command in config.ts using `confirmationRequired`.
pub const DEFAULT_CONFIRMATION_COMMANDS: &[&str] = &[
    "builtin-shut-down",
    "builtin-restart",
    "builtin-log-out",
    "builtin-empty-trash",
    "builtin-sleep",
    "builtin-quit-script-kit",
    "builtin-test-confirmation", // Dev test item
];
