use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::defaults::*;

// ============================================
// BUILT-IN CONFIG
// ============================================

/// Configuration for built-in features (clipboard history, app launcher, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltInConfig {
    /// Enable clipboard history built-in (default: true)
    #[serde(default = "default_clipboard_history")]
    pub clipboard_history: bool,
    /// Enable app launcher built-in (default: true)
    #[serde(default = "default_app_launcher")]
    pub app_launcher: bool,
    /// Enable window switcher built-in (default: true)
    #[serde(default = "default_window_switcher")]
    pub window_switcher: bool,
}

fn default_clipboard_history() -> bool {
    DEFAULT_CLIPBOARD_HISTORY
}
fn default_app_launcher() -> bool {
    DEFAULT_APP_LAUNCHER
}
fn default_window_switcher() -> bool {
    DEFAULT_WINDOW_SWITCHER
}

impl Default for BuiltInConfig {
    fn default() -> Self {
        BuiltInConfig {
            clipboard_history: DEFAULT_CLIPBOARD_HISTORY,
            app_launcher: DEFAULT_APP_LAUNCHER,
            window_switcher: DEFAULT_WINDOW_SWITCHER,
        }
    }
}

// ============================================
// PROCESS LIMITS
// ============================================

/// Configuration for process resource limits and health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessLimits {
    /// Maximum memory usage in MB (None = no limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    /// Maximum runtime in seconds (None = no limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_runtime_seconds: Option<u64>,
    /// Health check interval in milliseconds (default: 5000)
    #[serde(default = "default_health_check_interval_ms")]
    pub health_check_interval_ms: u64,
}

fn default_health_check_interval_ms() -> u64 {
    DEFAULT_HEALTH_CHECK_INTERVAL_MS
}

impl Default for ProcessLimits {
    fn default() -> Self {
        ProcessLimits {
            max_memory_mb: None,
            max_runtime_seconds: None,
            health_check_interval_ms: DEFAULT_HEALTH_CHECK_INTERVAL_MS,
        }
    }
}

// ============================================
// SUGGESTED CONFIG
// ============================================

/// Configuration for the "Suggested" section (frecency-based ranking)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedConfig {
    /// Whether the Suggested section is shown (default: true)
    #[serde(default = "default_suggested_enabled")]
    pub enabled: bool,
    /// Maximum number of items to show in SUGGESTED section (default: 10)
    #[serde(default = "default_suggested_max_items")]
    pub max_items: usize,
    /// Minimum score threshold for items to appear in Suggested (default: 0.1)
    /// Items with scores below this won't appear even if there's room
    #[serde(default = "default_suggested_min_score")]
    pub min_score: f64,
    /// Half-life in days for score decay (default: 7.0)
    /// Lower values = more weight on recent items
    /// Higher values = more weight on frequently used items
    #[serde(default = "default_suggested_half_life_days")]
    pub half_life_days: f64,
    /// Whether to track script usage for suggestions (default: true)
    /// If false, no new usage is recorded but existing data is preserved
    #[serde(default = "default_suggested_track_usage")]
    pub track_usage: bool,
    /// Commands to exclude from frecency tracking (default: ["builtin-quit-script-kit"])
    /// These commands won't appear in the Suggested section
    #[serde(default = "default_suggested_excluded_commands")]
    pub excluded_commands: Vec<String>,
}

fn default_suggested_enabled() -> bool {
    DEFAULT_SUGGESTED_ENABLED
}
fn default_suggested_max_items() -> usize {
    DEFAULT_SUGGESTED_MAX_ITEMS
}
fn default_suggested_min_score() -> f64 {
    DEFAULT_SUGGESTED_MIN_SCORE
}
fn default_suggested_half_life_days() -> f64 {
    DEFAULT_SUGGESTED_HALF_LIFE_DAYS
}
fn default_suggested_track_usage() -> bool {
    DEFAULT_SUGGESTED_TRACK_USAGE
}
fn default_suggested_excluded_commands() -> Vec<String> {
    DEFAULT_FRECENCY_EXCLUDED_COMMANDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for SuggestedConfig {
    fn default() -> Self {
        SuggestedConfig {
            enabled: DEFAULT_SUGGESTED_ENABLED,
            max_items: DEFAULT_SUGGESTED_MAX_ITEMS,
            min_score: DEFAULT_SUGGESTED_MIN_SCORE,
            half_life_days: DEFAULT_SUGGESTED_HALF_LIFE_DAYS,
            track_usage: DEFAULT_SUGGESTED_TRACK_USAGE,
            excluded_commands: default_suggested_excluded_commands(),
        }
    }
}

// ============================================
// CONTENT PADDING
// ============================================

/// Content padding configuration for prompts (terminal, editor, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPadding {
    #[serde(default = "default_padding_top")]
    pub top: f32,
    #[serde(default = "default_padding_left")]
    pub left: f32,
    #[serde(default = "default_padding_right")]
    pub right: f32,
}

fn default_padding_top() -> f32 {
    DEFAULT_PADDING_TOP
}
fn default_padding_left() -> f32 {
    DEFAULT_PADDING_LEFT
}
fn default_padding_right() -> f32 {
    DEFAULT_PADDING_RIGHT
}

impl Default for ContentPadding {
    fn default() -> Self {
        ContentPadding {
            top: DEFAULT_PADDING_TOP,
            left: DEFAULT_PADDING_LEFT,
            right: DEFAULT_PADDING_RIGHT,
        }
    }
}

// ============================================
// WATCHER + LAYOUT CONFIG
// ============================================

/// File watcher tuning values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherConfig {
    /// Debounce window for file-system events.
    #[serde(default = "default_watcher_debounce_ms")]
    pub debounce_ms: u64,
    /// Event storm threshold before collapsing to full reload.
    #[serde(default = "default_watcher_storm_threshold")]
    pub storm_threshold: usize,
    /// Initial supervisor restart delay.
    #[serde(default = "default_watcher_initial_backoff_ms")]
    pub initial_backoff_ms: u64,
    /// Maximum supervisor restart delay.
    #[serde(default = "default_watcher_max_backoff_ms")]
    pub max_backoff_ms: u64,
    /// Maximum consecutive notify errors before restart.
    #[serde(default = "default_watcher_max_notify_errors")]
    pub max_notify_errors: u32,
}

fn default_watcher_debounce_ms() -> u64 {
    DEFAULT_WATCHER_DEBOUNCE_MS
}
fn default_watcher_storm_threshold() -> usize {
    DEFAULT_WATCHER_STORM_THRESHOLD
}
fn default_watcher_initial_backoff_ms() -> u64 {
    DEFAULT_WATCHER_INITIAL_BACKOFF_MS
}
fn default_watcher_max_backoff_ms() -> u64 {
    DEFAULT_WATCHER_MAX_BACKOFF_MS
}
fn default_watcher_max_notify_errors() -> u32 {
    DEFAULT_WATCHER_MAX_NOTIFY_ERRORS
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_WATCHER_DEBOUNCE_MS,
            storm_threshold: DEFAULT_WATCHER_STORM_THRESHOLD,
            initial_backoff_ms: DEFAULT_WATCHER_INITIAL_BACKOFF_MS,
            max_backoff_ms: DEFAULT_WATCHER_MAX_BACKOFF_MS,
            max_notify_errors: DEFAULT_WATCHER_MAX_NOTIFY_ERRORS,
        }
    }
}

/// Core launcher sizing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    /// Standard list/panel height in pixels.
    #[serde(default = "default_layout_standard_height")]
    pub standard_height: f32,
    /// Full-height content views (editor, terminal) in pixels.
    #[serde(default = "default_layout_max_height")]
    pub max_height: f32,
}

fn default_layout_standard_height() -> f32 {
    DEFAULT_LAYOUT_STANDARD_HEIGHT
}
fn default_layout_max_height() -> f32 {
    DEFAULT_LAYOUT_MAX_HEIGHT
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            standard_height: DEFAULT_LAYOUT_STANDARD_HEIGHT,
            max_height: DEFAULT_LAYOUT_MAX_HEIGHT,
        }
    }
}

/// Theme selection preferences loaded from user settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSelectionPreferences {
    /// Optional preset identifier (for example: "catppuccin-mocha").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
}

/// User preferences loaded from `<SK_PATH>/kit/settings.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptKitUserPreferences {
    /// Launcher/window layout settings.
    #[serde(default)]
    pub layout: LayoutConfig,
    /// Theme selection settings.
    #[serde(default)]
    pub theme: ThemeSelectionPreferences,
}

// ============================================
// COMMAND CONFIG
// ============================================

/// Configuration for a specific command (script, built-in, or app).
///
/// Used to set per-command shortcuts and visibility options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandConfig {
    /// Optional keyboard shortcut to invoke this command directly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<HotkeyConfig>,
    /// Whether this command should be hidden from the main menu
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Whether this command requires confirmation before execution.
    /// Overrides the default behavior from DEFAULT_CONFIRMATION_COMMANDS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_required: Option<bool>,
}

/// Check if a string is a valid command ID format.
///
/// Valid command IDs start with one of:
/// - `builtin/` - Built-in Script Kit features
/// - `app/` - macOS applications (by bundle identifier)
/// - `script/` - User scripts (by filename)
/// - `scriptlet/` - Inline scriptlets (by UUID or name)
#[allow(dead_code)]
pub fn is_valid_command_id(id: &str) -> bool {
    id.starts_with("builtin/")
        || id.starts_with("app/")
        || id.starts_with("script/")
        || id.starts_with("scriptlet/")
}

/// Convert a command ID to its deeplink URL.
///
/// The deeplink format is: `scriptkit://commands/{commandId}`
/// Note: The app registers 'scriptkit' URL scheme (not 'kit')
#[allow(dead_code)]
pub fn command_id_to_deeplink(command_id: &str) -> String {
    format!("scriptkit://commands/{}", command_id)
}

// ============================================
// CLAUDE CODE CLI CONFIG
// ============================================

/// Configuration for the Claude Code CLI provider.
///
/// This allows Script Kit to use the local `claude` CLI as an AI provider,
/// speaking JSONL over stdin/stdout for streaming responses.
///
/// # Example
///
/// ```typescript
/// // In ~/.scriptkit/config.ts
/// export default {
///   hotkey: { modifiers: ["meta"], key: "Semicolon" },
///   claudeCode: {
///     enabled: true,
///     permissionMode: "plan",
///     allowedTools: "Read,Edit,Bash(git:*)",
///     addDirs: ["/home/user/projects"]
///   }
/// } satisfies Config;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeConfig {
    /// Enable the Claude Code CLI provider.
    /// When enabled, "Claude Code" models will appear in the AI chat model picker.
    ///
    /// @default false (requires explicit opt-in)
    #[serde(default = "default_claude_code_enabled")]
    pub enabled: bool,

    /// Custom path to the `claude` CLI binary.
    /// If not specified, will look for `claude` in PATH.
    ///
    /// @default undefined (uses "claude" from PATH)
    /// @example "/opt/homebrew/bin/claude"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Permission mode for Claude Code.
    /// - "plan": Safe default - Claude plans but asks before executing tools
    /// - "dontAsk": Agent can execute tools without confirmation (sandbox only!)
    ///
    /// @default "plan"
    #[serde(default = "default_claude_code_permission_mode")]
    pub permission_mode: String,

    /// Comma-separated list of allowed tools.
    /// Restricts which tools Claude Code can use.
    ///
    /// @default undefined (uses Claude Code defaults)
    /// @example "Read,Edit,Bash(git:*)"
    /// @example "Read,Edit,Bash,Write"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<String>,

    /// Additional directories to add to Claude Code's workspace.
    /// Each path is passed as `--add-dir` to the CLI.
    ///
    /// @default [] (empty)
    /// @example ["/home/user/projects", "/tmp/scratch"]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_dirs: Vec<String>,
}

fn default_claude_code_enabled() -> bool {
    DEFAULT_CLAUDE_CODE_ENABLED
}

fn default_claude_code_permission_mode() -> String {
    DEFAULT_CLAUDE_CODE_PERMISSION_MODE.to_string()
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        ClaudeCodeConfig {
            enabled: DEFAULT_CLAUDE_CODE_ENABLED,
            path: None,
            permission_mode: DEFAULT_CLAUDE_CODE_PERMISSION_MODE.to_string(),
            allowed_tools: None,
            add_dirs: vec![],
        }
    }
}
