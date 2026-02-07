// ============================================
// HOTKEY CONFIG
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl HotkeyConfig {
    /// Create a default AI hotkey (Cmd+Shift+Space)
    pub fn default_ai_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Space".to_string(),
        }
    }

    /// Create a default logs capture hotkey (Cmd+Shift+L)
    pub fn default_logs_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyL".to_string(),
        }
    }

    /// Convert to a human-readable display string using macOS symbols (e.g., "⌘⇧K").
    ///
    /// Uses standard macOS modifier symbols in order: ⌃ (Control), ⌥ (Option), ⇧ (Shift), ⌘ (Command)
    pub fn to_display_string(&self) -> String {
        let mut result = String::new();

        // Standard macOS order: Control, Option, Shift, Command
        let has_ctrl = self.modifiers.iter().any(|m| m == "ctrl" || m == "control");
        let has_alt = self.modifiers.iter().any(|m| m == "alt" || m == "option");
        let has_shift = self.modifiers.iter().any(|m| m == "shift");
        let has_cmd = self.modifiers.iter().any(|m| m == "meta" || m == "cmd");

        if has_ctrl {
            result.push('⌃');
        }
        if has_alt {
            result.push('⌥');
        }
        if has_shift {
            result.push('⇧');
        }
        if has_cmd {
            result.push('⌘');
        }

        // Normalize key for display
        let key_display = if self.key.starts_with("Key") {
            // "KeyA" -> "A"
            self.key[3..].to_uppercase()
        } else if self.key.starts_with("Digit") {
            // "Digit0" -> "0"
            self.key[5..].to_string()
        } else {
            // Keep as-is but uppercase first char for consistency
            let mut chars = self.key.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };
        result.push_str(&key_display);

        result
    }

    /// Convert to canonical shortcut string format (e.g., "cmd+shift+k").
    ///
    /// Maps modifier names from config format to shortcut format:
    /// - "meta" -> "cmd"
    /// - "ctrl" -> "ctrl"
    /// - "alt" -> "alt"
    /// - "shift" -> "shift"
    ///
    /// Keys are normalized:
    /// - "KeyX" -> "x" (strip Key prefix, lowercase)
    /// - "Digit0" -> "0" (strip Digit prefix)
    /// - Other keys kept as-is but lowercased
    pub fn to_shortcut_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Convert modifiers (maintain consistent order: alt, cmd, ctrl, shift)
        let has_alt = self.modifiers.iter().any(|m| m == "alt" || m == "option");
        let has_cmd = self.modifiers.iter().any(|m| m == "meta" || m == "cmd");
        let has_ctrl = self.modifiers.iter().any(|m| m == "ctrl" || m == "control");
        let has_shift = self.modifiers.iter().any(|m| m == "shift");

        if has_alt {
            parts.push("alt".to_string());
        }
        if has_cmd {
            parts.push("cmd".to_string());
        }
        if has_ctrl {
            parts.push("ctrl".to_string());
        }
        if has_shift {
            parts.push("shift".to_string());
        }

        // Normalize key
        let key = if self.key.starts_with("Key") {
            // "KeyA" -> "a"
            self.key[3..].to_lowercase()
        } else if self.key.starts_with("Digit") {
            // "Digit0" -> "0"
            self.key[5..].to_string()
        } else {
            // Keep as-is but lowercase
            self.key.to_lowercase()
        };
        parts.push(key);

        parts.join("+")
    }
}

fn default_main_hotkey() -> HotkeyConfig {
    HotkeyConfig {
        modifiers: vec!["meta".to_string()],
        key: "Semicolon".to_string(),
    }
}

fn default_ai_hotkey_enabled() -> bool {
    DEFAULT_AI_HOTKEY_ENABLED
}

fn default_logs_hotkey_enabled() -> bool {
    DEFAULT_LOGS_HOTKEY_ENABLED
}

// ============================================
// MAIN CONFIG
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_main_hotkey")]
    pub hotkey: HotkeyConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bun_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    /// Padding for content areas (terminal, editor, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<ContentPadding>,
    /// Font size for the editor prompt (in pixels)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "editorFontSize"
    )]
    pub editor_font_size: Option<f32>,
    /// Font size for the terminal prompt (in pixels)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "terminalFontSize"
    )]
    pub terminal_font_size: Option<f32>,
    /// UI scale factor (1.0 = 100%)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "uiScale")]
    pub ui_scale: Option<f32>,
    /// Built-in features configuration (clipboard history, app launcher, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "builtIns")]
    pub built_ins: Option<BuiltInConfig>,
    /// Process resource limits and health monitoring configuration
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "processLimits"
    )]
    pub process_limits: Option<ProcessLimits>,
    /// Maximum text length for clipboard history entries (bytes). 0 = no limit.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "clipboardHistoryMaxTextLength"
    )]
    pub clipboard_history_max_text_length: Option<usize>,
    /// Suggested section configuration (frecency-based ranking)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested: Option<SuggestedConfig>,
    /// Hotkey for opening Notes window (no default; user-configured only)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "notesHotkey"
    )]
    pub notes_hotkey: Option<HotkeyConfig>,
    /// Hotkey for opening AI Chat window (default: Cmd+Shift+Space)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "aiHotkey")]
    pub ai_hotkey: Option<HotkeyConfig>,
    /// Whether AI hotkey is enabled (default: true)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "aiHotkeyEnabled"
    )]
    pub ai_hotkey_enabled: Option<bool>,
    /// Hotkey for toggling log capture (default: Cmd+Shift+L)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "logsHotkey"
    )]
    pub logs_hotkey: Option<HotkeyConfig>,
    /// Whether logs hotkey is enabled (default: true)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "logsHotkeyEnabled"
    )]
    pub logs_hotkey_enabled: Option<bool>,
    /// Watcher tuning settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watcher: Option<WatcherConfig>,
    /// Window/layout sizing settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutConfig>,
    /// Per-command configuration overrides (shortcuts, visibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<HashMap<String, CommandConfig>>,
    /// Claude Code CLI provider configuration.
    /// Enable and configure the local `claude` CLI as an AI provider.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "claudeCode"
    )]
    pub claude_code: Option<ClaudeCodeConfig>,
}
