impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: default_main_hotkey(),
            bun_path: None,           // Will use system PATH if not specified
            editor: None,             // Will use $EDITOR or fallback to "code"
            padding: None,            // Will use ContentPadding::default() via getter
            editor_font_size: None,   // Will use DEFAULT_EDITOR_FONT_SIZE via getter
            terminal_font_size: None, // Will use DEFAULT_TERMINAL_FONT_SIZE via getter
            ui_scale: None,           // Will use DEFAULT_UI_SCALE via getter
            built_ins: None,          // Will use BuiltInConfig::default() via getter
            process_limits: None,     // Will use ProcessLimits::default() via getter
            clipboard_history_max_text_length: None, // Will use default via getter
            suggested: None,          // Will use SuggestedConfig::default() via getter
            notes_hotkey: None,       // No default shortcut; must be explicitly configured
            ai_hotkey: None,          // Will use HotkeyConfig::default_ai_hotkey() via getter
            ai_hotkey_enabled: None,  // Defaults to true via getter
            logs_hotkey: None,        // Will use HotkeyConfig::default_logs_hotkey() via getter
            logs_hotkey_enabled: None, // Defaults to true via getter
            watcher: None,            // Will use WatcherConfig::default() via getter
            layout: None,             // Will use LayoutConfig::default() via getter
            commands: None,           // No per-command overrides by default
            claude_code: None,        // Will use ClaudeCodeConfig::default() via getter
        }
    }
}

fn sanitize_positive_f32(value: Option<f32>, fallback: f32) -> f32 {
    match value {
        Some(value) if value.is_finite() && value > 0.0 => value,
        _ => fallback,
    }
}

fn sanitize_process_limits(mut limits: ProcessLimits) -> ProcessLimits {
    if limits.health_check_interval_ms == 0 {
        limits.health_check_interval_ms = DEFAULT_HEALTH_CHECK_INTERVAL_MS;
    }
    limits
}

impl Config {
    /// Returns the configured editor, falling back to $EDITOR env var or "code" (VS Code)
    /// Used by ActionsDialog "Open in Editor" action
    #[allow(dead_code)] // Will be used by ActionsDialog worker
    pub fn get_editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "code".to_string())
    }

    /// Returns the content padding, or defaults if not configured
    #[allow(dead_code)] // Will be used by TermPrompt/EditorPrompt workers
    pub fn get_padding(&self) -> ContentPadding {
        self.padding.clone().unwrap_or_default()
    }

    /// Returns the editor font size, or DEFAULT_EDITOR_FONT_SIZE if not configured
    #[allow(dead_code)] // Will be used by EditorPrompt worker
    pub fn get_editor_font_size(&self) -> f32 {
        sanitize_positive_f32(self.editor_font_size, DEFAULT_EDITOR_FONT_SIZE)
    }

    /// Returns the terminal font size, or DEFAULT_TERMINAL_FONT_SIZE if not configured
    #[allow(dead_code)] // Will be used by TermPrompt worker
    pub fn get_terminal_font_size(&self) -> f32 {
        sanitize_positive_f32(self.terminal_font_size, DEFAULT_TERMINAL_FONT_SIZE)
    }

    /// Returns the UI scale factor, or DEFAULT_UI_SCALE if not configured
    #[allow(dead_code)] // Will be used for UI scaling
    pub fn get_ui_scale(&self) -> f32 {
        sanitize_positive_f32(self.ui_scale, DEFAULT_UI_SCALE)
    }

    /// Returns the built-in features configuration, or defaults if not configured
    #[allow(dead_code)] // Will be used by builtins module
    pub fn get_builtins(&self) -> BuiltInConfig {
        self.built_ins.clone().unwrap_or_default()
    }

    /// Returns max clipboard history text length (bytes), or default if not configured
    #[allow(dead_code)] // Used for clipboard history limits
    pub fn get_clipboard_history_max_text_length(&self) -> usize {
        self.clipboard_history_max_text_length
            .unwrap_or(DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH)
    }

    /// Returns the process limits configuration, or defaults if not configured
    pub fn get_process_limits(&self) -> ProcessLimits {
        sanitize_process_limits(self.process_limits.clone().unwrap_or_default())
    }

    /// Returns the suggested section configuration, or defaults if not configured
    pub fn get_suggested(&self) -> SuggestedConfig {
        self.suggested.clone().unwrap_or_default()
    }

    /// Returns the notes hotkey configuration, or None if not configured.
    /// No default shortcut is provided - users must explicitly configure one.
    #[allow(dead_code)]
    pub fn get_notes_hotkey(&self) -> Option<HotkeyConfig> {
        self.notes_hotkey.clone()
    }

    /// Returns true if AI hotkey registration is enabled.
    pub fn is_ai_hotkey_enabled(&self) -> bool {
        self.ai_hotkey_enabled
            .unwrap_or_else(default_ai_hotkey_enabled)
    }

    /// Returns true if logs hotkey registration is enabled.
    pub fn is_logs_hotkey_enabled(&self) -> bool {
        self.logs_hotkey_enabled
            .unwrap_or_else(default_logs_hotkey_enabled)
    }

    /// Returns the AI hotkey configuration when enabled.
    /// Falls back to default (Cmd+Shift+Space) when enabled but not configured.
    #[allow(dead_code)]
    pub fn get_ai_hotkey(&self) -> Option<HotkeyConfig> {
        if !self.is_ai_hotkey_enabled() {
            return None;
        }
        Some(
            self.ai_hotkey
                .clone()
                .unwrap_or_else(HotkeyConfig::default_ai_hotkey),
        )
    }

    /// Returns the logs hotkey configuration when enabled.
    /// Falls back to default (Cmd+Shift+L) when enabled but not configured.
    #[allow(dead_code)]
    pub fn get_logs_hotkey(&self) -> Option<HotkeyConfig> {
        if !self.is_logs_hotkey_enabled() {
            return None;
        }
        Some(
            self.logs_hotkey
                .clone()
                .unwrap_or_else(HotkeyConfig::default_logs_hotkey),
        )
    }

    /// Returns watcher tuning config, or defaults.
    pub fn get_watcher(&self) -> WatcherConfig {
        self.watcher.clone().unwrap_or_default()
    }

    /// Returns layout sizing config, or defaults.
    #[allow(dead_code)]
    pub fn get_layout(&self) -> LayoutConfig {
        self.layout.clone().unwrap_or_default()
    }

    /// Returns command configuration for a specific command ID, or None if not configured.
    #[allow(dead_code)]
    pub fn get_command_config(&self, command_id: &str) -> Option<&CommandConfig> {
        self.commands.as_ref().and_then(|cmds| cmds.get(command_id))
    }

    /// Check if a command should be hidden from the main menu.
    #[allow(dead_code)]
    pub fn is_command_hidden(&self, command_id: &str) -> bool {
        self.get_command_config(command_id)
            .and_then(|c| c.hidden)
            .unwrap_or(false)
    }

    /// Get the shortcut for a command, if configured.
    #[allow(dead_code)]
    pub fn get_command_shortcut(&self, command_id: &str) -> Option<&HotkeyConfig> {
        self.get_command_config(command_id)
            .and_then(|c| c.shortcut.as_ref())
    }

    /// Check if a command requires confirmation before execution.
    ///
    /// Returns true if:
    /// - Command is in DEFAULT_CONFIRMATION_COMMANDS AND not explicitly disabled in config
    /// - OR command has confirmationRequired: true in config
    #[allow(dead_code)]
    pub fn requires_confirmation(&self, command_id: &str) -> bool {
        // Check if user has explicitly configured this command
        if let Some(cmd_config) = self.get_command_config(command_id) {
            if let Some(requires) = cmd_config.confirmation_required {
                return requires;
            }
        }
        // Fall back to defaults
        DEFAULT_CONFIRMATION_COMMANDS.contains(&command_id)
    }

    /// Returns the Claude Code CLI configuration, or defaults if not configured.
    ///
    /// Use this to check if Claude Code is enabled and get its settings:
    /// ```ignore
    /// let claude_config = config.get_claude_code();
    /// if claude_config.enabled {
    ///     // Register Claude Code provider
    /// }
    /// ```
    pub fn get_claude_code(&self) -> ClaudeCodeConfig {
        self.claude_code.clone().unwrap_or_default()
    }
}
