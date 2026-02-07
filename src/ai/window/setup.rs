use super::*;

impl AiApp {
    pub(super) fn get_selected_chat(&self) -> Option<&Chat> {
        self.selected_chat_id
            .and_then(|id| self.chats.iter().find(|c| c.id == id))
    }

    /// Render the search input
    pub(super) fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Fixed height container to prevent layout shift when typing
        // Style the container and use appearance(false) on Input to remove its default white background
        // Use vibrancy-compatible background: white with low alpha (similar to selected items)
        let search_bg = cx.theme().muted.opacity(0.4);
        let border_color = cx.theme().border.opacity(0.3);

        div()
            .id("search-container")
            .w_full()
            .h(TITLEBAR_H) // Fixed height to prevent layout shift
            .flex()
            .items_center()
            .px_2()
            .rounded_md()
            .border_1()
            .border_color(border_color)
            .bg(search_bg) // Vibrancy-compatible semi-transparent background
            .tooltip(|window, cx| {
                Tooltip::new("Search chats")
                    .key_binding(gpui::Keystroke::parse("cmd-shift-f").ok().map(Kbd::new))
                    .build(window, cx)
            })
            .child(
                Input::new(&self.search_state)
                    .w_full()
                    .small()
                    .appearance(false) // Remove default background/border (we provide our own)
                    .focus_bordered(false), // Disable default focus border
            )
    }

    /// Toggle sidebar visibility
    pub(super) fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }

    /// Copy the setup command to clipboard and show feedback
    pub(super) fn copy_setup_command(&mut self, cx: &mut Context<Self>) {
        let setup_command = "export SCRIPT_KIT_ANTHROPIC_API_KEY=\"your-key-here\"";
        let item = gpui::ClipboardItem::new_string(setup_command.to_string());
        cx.write_to_clipboard(item);
        self.setup_copied_at = Some(std::time::Instant::now());
        info!("Setup command copied to clipboard");
        cx.notify();

        // Reset feedback after 2 seconds
        cx.spawn(async move |this, cx| {
            gpui::Timer::after(std::time::Duration::from_millis(2000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |this, cx| {
                    this.setup_copied_at = None;
                    cx.notify();
                })
            });
        })
        .detach();
    }

    /// Check if we're showing "Copied!" feedback
    pub(super) fn is_showing_copied_feedback(&self) -> bool {
        self.setup_copied_at
            .map(|t| t.elapsed() < std::time::Duration::from_millis(2000))
            .unwrap_or(false)
    }

    pub(super) const SETUP_BUTTON_COUNT: usize = 2;

    pub(super) fn next_setup_button_focus_index(current: usize, delta: isize) -> usize {
        let count = Self::SETUP_BUTTON_COUNT as isize;
        ((current % Self::SETUP_BUTTON_COUNT) as isize + delta).rem_euclid(count) as usize
    }

    pub(super) fn move_setup_button_focus(&mut self, delta: isize, cx: &mut Context<Self>) {
        let next_index = Self::next_setup_button_focus_index(self.setup_button_focus_index, delta);
        if next_index != self.setup_button_focus_index {
            self.setup_button_focus_index = next_index;
            cx.notify();
        }
    }

    /// Show the API key configuration input
    pub(super) fn show_api_key_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.showing_api_key_input = true;
        // Focus the API key input
        self.api_key_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
            state.set_selection(0, 0, window, cx);
        });
        cx.notify();
    }

    /// Hide the API key configuration input
    pub(super) fn hide_api_key_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.showing_api_key_input = false;
        // Refocus main handle for setup card keyboard navigation
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    /// Submit the API key from the configuration input
    pub(super) fn submit_api_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_input_state.read(cx).value().to_string();
        let api_key = api_key.trim();

        if api_key.is_empty() {
            info!("API key input is empty, ignoring submission");
            return;
        }

        // Save the API key to secrets storage
        if let Err(e) =
            crate::secrets::set_secret(crate::ai::config::env_vars::VERCEL_API_KEY, api_key)
        {
            tracing::error!(error = %e, "Failed to save Vercel API key");
            return;
        }

        info!("Vercel API key saved successfully");

        // Reinitialize the provider registry to pick up the new key
        let config = crate::config::load_config();
        self.provider_registry = ProviderRegistry::from_environment_with_config(Some(&config));
        self.available_models = self.provider_registry.get_all_models();

        // Select default model if available
        self.selected_model = self
            .available_models
            .iter()
            .find(|m| m.id.contains("haiku"))
            .or_else(|| self.available_models.first())
            .cloned();

        info!(
            providers = self.provider_registry.provider_ids().len(),
            models = self.available_models.len(),
            "Providers reinitialized after API key setup"
        );

        // Hide the input and show the welcome state
        self.showing_api_key_input = false;

        // Clear the input
        self.api_key_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        // Focus the main input
        self.focus_input(window, cx);

        cx.notify();
    }

    /// Enable Claude Code in config.ts by spawning bun to run config-cli.ts
    pub(super) fn enable_claude_code(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        info!("Enabling Claude Code in config.ts");

        // Get the path to config-cli.ts (in the app's scripts directory)
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let config_cli_path = format!("{home_dir}/.scriptkit/sdk/config-cli.ts");

        // Also check the dev path for development
        let dev_config_cli_path =
            std::env::current_dir().map(|p| p.join("scripts/config-cli.ts").display().to_string());

        // Try the SDK path first, then fall back to dev path
        let cli_path = if std::path::Path::new(&config_cli_path).exists() {
            config_cli_path
        } else if let Ok(ref dev_path) = dev_config_cli_path {
            if std::path::Path::new(dev_path).exists() {
                dev_path.clone()
            } else {
                // If neither exists, write config directly
                self.write_claude_code_config_directly(window, cx);
                return;
            }
        } else {
            self.write_claude_code_config_directly(window, cx);
            return;
        };

        // Get bun path from config or use default
        let config = crate::config::load_config();
        let bun_path = config
            .bun_path
            .as_ref()
            .and_then(|p| {
                if std::path::Path::new(p).exists() {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "bun".to_string());

        // Run: bun config-cli.ts set claudeCode.enabled true
        match std::process::Command::new(&bun_path)
            .arg(&cli_path)
            .arg("set")
            .arg("claudeCode.enabled")
            .arg("true")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Claude Code enabled successfully in config.ts");
                    self.finish_claude_code_setup(window, cx);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!(stderr = %stderr, "config-cli.ts failed, trying direct write");
                    self.write_claude_code_config_directly(window, cx);
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to run config-cli.ts, trying direct write");
                self.write_claude_code_config_directly(window, cx);
            }
        }
    }

    /// Write Claude Code config directly to config.ts (fallback when config-cli.ts unavailable)
    ///
    /// Uses the centralized safe-write path with validation, backup, and atomic rename.
    pub(super) fn write_claude_code_config_directly(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::config::editor::{self, WriteOutcome};

        let config_path =
            std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());
        let config = crate::config::load_config();
        let bun_path = config.bun_path.as_deref();

        match editor::enable_claude_code_safely(&config_path, bun_path) {
            Ok(WriteOutcome::Written | WriteOutcome::Created) => {
                info!("Claude Code enabled in config.ts");
            }
            Ok(WriteOutcome::AlreadySet) => {
                info!("Claude Code already enabled in config.ts");
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to modify config.ts");
                if let Err(recover_err) = editor::recover_from_backup(&config_path, bun_path) {
                    tracing::error!(error = %recover_err, "Backup recovery also failed");
                }
                return;
            }
        }

        self.finish_claude_code_setup(window, cx);
    }

    /// Finish Claude Code setup - reinitialize providers and update UI
    pub(super) fn finish_claude_code_setup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Clear any previous feedback
        self.claude_code_setup_feedback = None;

        // Reload config to pick up the change
        let config = crate::config::load_config();

        // Check if Claude CLI is available before reinitializing
        let claude_path = config
            .get_claude_code()
            .path
            .unwrap_or_else(|| "claude".to_string());
        let claude_available = std::process::Command::new(&claude_path)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        // Reinitialize the provider registry to pick up Claude Code
        self.provider_registry = ProviderRegistry::from_environment_with_config(Some(&config));
        self.available_models = self.provider_registry.get_all_models();

        // Select Claude Code model if available, otherwise first available
        self.selected_model = self
            .available_models
            .iter()
            .find(|m| m.provider.to_lowercase().contains("claude") && m.id.contains("code"))
            .or_else(|| self.available_models.first())
            .cloned();

        info!(
            providers = self.provider_registry.provider_ids().len(),
            models = self.available_models.len(),
            claude_cli_available = claude_available,
            "Providers reinitialized after Claude Code setup"
        );

        // If config was set but Claude CLI isn't available, show feedback
        if !claude_available && config.get_claude_code().enabled {
            self.claude_code_setup_feedback = Some(
                "Config saved! Install Claude CLI to complete setup: npm install -g @anthropic-ai/claude-code".to_string()
            );
        }

        // Focus the main input
        self.focus_input(window, cx);

        cx.notify();
    }
}
