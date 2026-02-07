use super::*;

impl ScriptListApp {
    fn is_in_prompt(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ArgPrompt { .. }
                | AppView::DivPrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::ScratchPadView { .. }
                | AppView::QuickTerminalView { .. }
        )
    }

    /// Submit a response to the current prompt
    ///
    /// Uses try_send() to avoid blocking the UI thread if the script's input
    /// channel is full. User-initiated actions should never freeze the UI.
    fn submit_prompt_response(
        &mut self,
        id: String,
        value: Option<String>,
        _cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!("Submitting response for {}: {:?}", id, value),
        );

        let response = Message::Submit { id, value };

        if let Some(ref sender) = self.response_sender {
            // Use try_send to avoid blocking UI thread
            // If channel is full, the script isn't reading - log warning but don't freeze UI
            match sender.try_send(response) {
                Ok(()) => {
                    logging::log("UI", "Response queued for script");
                }
                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                    // Channel is full - script isn't reading stdin fast enough
                    // This shouldn't happen in normal operation, log as warning
                    logging::log(
                        "WARN",
                        "Response channel full - script may be stuck. Response dropped.",
                    );
                }
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    // Channel disconnected - script has exited
                    logging::log("UI", "Response channel disconnected - script exited");
                }
            }
        } else {
            logging::log("UI", "No response sender available");
        }

        // Return to waiting state (script will send next prompt or exit)
        // Don't change view here - wait for next message from script
    }

    /// Get filtered choices for arg prompt
    fn filtered_arg_choices(&self) -> Vec<(usize, &Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices.iter().enumerate().collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// P0: Get filtered choices as owned data for uniform_list closure
    fn get_filtered_arg_choices_owned(&self) -> Vec<(usize, Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Create box shadows from theme configuration
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        let shadow_config = self.theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        // For black (0x000000), we use h=0, s=0, l=0
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion for shadow color
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0) // achromatic
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Show inline AI chat prompt with built-in AI provider support.
    /// This switches to the ChatPrompt view with direct AI integration (no SDK needed).
    /// Prefers Vercel AI Gateway if configured, otherwise uses the first available provider.
    pub fn show_inline_ai_chat(&mut self, initial_query: Option<String>, cx: &mut Context<Self>) {
        use crate::ai::ProviderRegistry;
        use crate::prompts::{ChatEscapeCallback, ChatPrompt, ChatSubmitCallback};

        // Mark as opened from main menu so ESC returns to main menu
        self.opened_from_main_menu = true;

        // Create escape callback that signals via channel
        let escape_sender = self.inline_chat_escape_sender.clone();
        let escape_callback: ChatEscapeCallback = std::sync::Arc::new(move |_id| {
            let _ = escape_sender.try_send(());
        });

        // Use cached registry if available, otherwise build synchronously as fallback
        let registry = self
            .cached_provider_registry
            .clone()
            .unwrap_or_else(|| ProviderRegistry::from_environment_with_config(Some(&self.config)));

        if !registry.has_any_provider() {
            crate::logging::log("CHAT", "No AI providers configured - showing setup card");

            // Create configure callback that signals via channel
            let configure_sender = self.inline_chat_configure_sender.clone();
            let configure_callback: crate::prompts::ChatConfigureCallback =
                std::sync::Arc::new(move || {
                    crate::logging::log("CHAT", "Configure callback triggered - sending signal");
                    let _ = configure_sender.try_send(());
                });

            // Create Claude Code callback that signals via channel
            let claude_code_sender = self.inline_chat_claude_code_sender.clone();
            let claude_code_callback: crate::prompts::ChatClaudeCodeCallback =
                std::sync::Arc::new(move || {
                    crate::logging::log("CHAT", "Claude Code callback triggered - sending signal");
                    let _ = claude_code_sender.try_send(());
                });

            // Create a no-op submit callback since we're in setup mode
            let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
                crate::logging::log("CHAT", "No providers - submission ignored (setup mode)");
            });

            let chat_prompt = ChatPrompt::new(
                "inline-ai-setup".to_string(),
                Some("Configure API key to continue...".to_string()),
                vec![],
                None, // No hint needed - setup card is the UI
                None,
                self.focus_handle.clone(),
                noop_callback,
                std::sync::Arc::clone(&self.theme),
            )
            .with_title("Ask AI")
            .with_save_history(false) // Don't save setup state to history
            .with_escape_callback(escape_callback.clone())
            .with_needs_setup(true)
            .with_configure_callback(configure_callback)
            .with_claude_code_callback(claude_code_callback);

            let entity = cx.new(|_| chat_prompt);
            self.current_view = AppView::ChatPrompt {
                id: "inline-ai-setup".to_string(),
                entity,
            };
            self.focused_input = FocusedInput::None;
            self.pending_focus = Some(FocusTarget::ChatPrompt);
            resize_to_view_sync(ViewType::DivPrompt, 0);
            cx.notify();
            return;
        }

        crate::logging::log(
            "CHAT",
            &format!(
                "Showing inline AI chat with {} providers",
                registry.provider_ids().len()
            ),
        );

        // Create a no-op callback since built-in AI handles submissions internally
        let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
            // Built-in AI mode handles this internally
        });

        let placeholder = Some("Ask anything...".to_string());

        let mut chat_prompt = ChatPrompt::new(
            "inline-ai".to_string(),
            placeholder,
            vec![],
            None,
            None,
            self.focus_handle.clone(),
            noop_callback,
            std::sync::Arc::clone(&self.theme),
        )
        .with_title("Ask AI")
        .with_save_history(true)
        .with_escape_callback(escape_callback)
        .with_builtin_ai(registry, true); // true = prefer Vercel AI Gateway

        // If there's an initial query, set it in the input and auto-submit
        if let Some(query) = initial_query {
            chat_prompt.input.set_text(&query);
            chat_prompt = chat_prompt.with_pending_submit(true);
        }

        let entity = cx.new(|_| chat_prompt);
        self.current_view = AppView::ChatPrompt {
            id: "inline-ai".to_string(),
            entity,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::ChatPrompt);
        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }

    /// Rebuild the cached provider registry in a background thread.
    /// Called after config changes (API key saved, Claude Code enabled, etc.)
    pub fn rebuild_provider_registry_async(&mut self, cx: &mut Context<Self>) {
        self.cached_provider_registry = None;
        let config_clone = self.config.clone();
        let (tx, rx) = async_channel::bounded::<crate::ai::ProviderRegistry>(1);

        std::thread::spawn(move || {
            let registry =
                crate::ai::ProviderRegistry::from_environment_with_config(Some(&config_clone));
            if tx.send_blocking(registry).is_err() {
                logging::log(
                    "APP",
                    "Provider registry rebuild result dropped: receiver unavailable",
                );
            }
        });

        cx.spawn(async move |this, cx| {
            let Ok(registry) = rx.recv().await else {
                logging::log("APP", "Provider registry rebuild failed: channel closed");
                return;
            };

            let provider_count = registry.provider_ids().len();
            let _ = cx.update(|cx| {
                this.update(cx, |app, _cx| {
                    app.cached_provider_registry = Some(registry);
                    logging::log(
                        "APP",
                        &format!("Provider registry rebuilt: {} providers", provider_count),
                    );
                })
            });
        })
        .detach();
    }
}
}
