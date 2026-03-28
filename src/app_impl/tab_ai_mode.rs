use super::*;

/// Resolved context returned by `build_tab_ai_context`, carrying bundle metadata
/// and warning counts alongside the serializable blob.
#[derive(Debug, Clone)]
struct TabAiResolvedContext {
    context: crate::ai::TabAiContextBlob,
    bundle_id: Option<String>,
    context_warning_count: usize,
}

impl ScriptListApp {
    /// Open the Tab AI overlay from any surface.
    ///
    /// Captures a UI snapshot at invocation time and shows the mini input.
    /// The underlying view remains visible and unchanged.
    pub(crate) fn open_tab_ai_overlay(&mut self, cx: &mut Context<Self>) {
        // Already open or save-offer visible — do nothing
        if self.tab_ai_state.is_some() || self.tab_ai_save_offer_state.is_some() {
            return;
        }

        let ui_snapshot = self.snapshot_tab_ai_ui(cx);

        // Cheap frontmost-app capture — no screenshots, no selected text
        let frontmost_bundle_id = crate::context_snapshot::capture_context_snapshot(
            &crate::context_snapshot::CaptureContextOptions {
                include_selected_text: false,
                include_frontmost_app: true,
                include_menu_bar: false,
                include_browser_url: false,
                include_focused_window: false,
            },
        )
        .frontmost_app
        .map(|app| app.bundle_id);

        tracing::info!(
            event = "tab_ai_open",
            prompt_type = %ui_snapshot.prompt_type,
            has_frontmost_bundle_id = frontmost_bundle_id.is_some(),
        );

        self.tab_ai_state = Some(TabAiOverlayState {
            intent: String::new(),
            ui_snapshot,
            frontmost_bundle_id,
            memory_hint: None,
            running: false,
            error: None,
        });

        // Close actions popup if open
        self.show_actions_popup = false;
        self.actions_dialog = None;

        cx.notify();
    }

    /// Close the Tab AI overlay and restore focus.
    pub(crate) fn close_tab_ai_overlay(&mut self, cx: &mut Context<Self>) {
        if self.tab_ai_state.is_some() {
            tracing::info!(event = "tab_ai_close");
            self.tab_ai_state = None;
            self.tab_ai_task = None;
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    /// Returns whether the Tab AI overlay is currently visible.
    #[allow(dead_code)] // Will be used by key interceptors in future phases
    pub(crate) fn is_tab_ai_overlay_open(&self) -> bool {
        self.tab_ai_state.is_some()
    }

    /// Refresh the memory hint based on the current intent and frontmost bundle id.
    /// Degrades to `None` with a warning log on failure — never aborts the overlay.
    fn refresh_tab_ai_memory_hint(&mut self) {
        let (intent, bundle_id) = match self.tab_ai_state.as_ref() {
            Some(state) => (state.intent.trim().to_string(), state.frontmost_bundle_id.clone()),
            None => return,
        };

        let memory_hint = match crate::ai::resolve_tab_ai_memory_suggestions(
            &intent,
            bundle_id.as_deref(),
            1,
        ) {
            Ok(suggestions) => suggestions.into_iter().next(),
            Err(error) => {
                tracing::warn!(event = "tab_ai_memory_hint_failed", error = %error);
                None
            }
        };

        if let Some(state) = &mut self.tab_ai_state {
            state.memory_hint = memory_hint;
        }
    }

    /// Build a clipboard context summary from the most recent cached clipboard entry.
    /// Uses only cached data — no new clipboard reads or screenshot capture.
    fn resolve_tab_ai_clipboard_context(&self) -> Option<crate::ai::TabAiClipboardContext> {
        let entry = self.cached_clipboard_entries.first()?;

        let preview = if entry.content_type.as_str() == "image" {
            entry
                .ocr_text
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| entry.display_preview())
        } else {
            entry.display_preview()
        };

        Some(crate::ai::TabAiClipboardContext {
            content_type: entry.content_type.as_str().to_string(),
            preview: crate::ai::truncate_tab_ai_text(&preview, 240),
            ocr_text: entry
                .ocr_text
                .clone()
                .filter(|text| !text.trim().is_empty())
                .map(|text| crate::ai::truncate_tab_ai_text(&text, 240)),
        })
    }

    /// Capture a snapshot of the current UI state for context assembly.
    #[allow(dead_code)]
    fn snapshot_tab_ai_ui(&self, cx: &Context<Self>) -> crate::ai::TabAiUiSnapshot {
        let prompt_type = self.app_view_name();

        // Collect visible elements (capped to keep token cost low)
        let outcome = self.collect_visible_elements(12, cx);

        let input_text = self.current_input_text();

        crate::ai::TabAiUiSnapshot {
            prompt_type,
            input_text,
            focused_semantic_id: outcome.focused_semantic_id(),
            selected_semantic_id: outcome.selected_semantic_id(),
            visible_elements: outcome.elements,
        }
    }

    /// Build the full context blob for AI submission.
    ///
    /// Returns a `TabAiResolvedContext` carrying bundle metadata and warning
    /// counts alongside the serializable blob.  Uses
    /// `TabAiContextBlob::from_parts(...)` so the runtime path and
    /// deterministic test path share one constructor.
    fn build_tab_ai_context(&self, _cx: &Context<Self>) -> TabAiResolvedContext {
        let ui = self
            .tab_ai_state
            .as_ref()
            .map(|s| s.ui_snapshot.clone())
            .unwrap_or_default();

        let intent_for_lookup = self
            .tab_ai_state
            .as_ref()
            .map(|s| s.intent.clone())
            .unwrap_or_default();

        // Desktop context — use the lightweight recommendation profile
        let desktop = crate::context_snapshot::capture_context_snapshot(
            &crate::context_snapshot::CaptureContextOptions::recommendation(),
        );

        let bundle_id = desktop
            .frontmost_app
            .as_ref()
            .map(|app| app.bundle_id.clone());

        let context_warning_count = desktop.warnings.len();

        // Recent input history (most recent first, bounded)
        let recent_inputs = self.input_history.recent_entries(5);

        // Cached clipboard context (no new reads)
        let clipboard = self.resolve_tab_ai_clipboard_context();

        // Prior automation suggestions (up to 3)
        let prior_automations = match crate::ai::resolve_tab_ai_memory_suggestions(
            &intent_for_lookup,
            bundle_id.as_deref(),
            3,
        ) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(event = "tab_ai_prior_automation_lookup_failed", error = %error);
                Vec::new()
            }
        };

        let timestamp = chrono::Utc::now().to_rfc3339();

        tracing::info!(
            event = "tab_ai_context_built",
            prompt_type = %ui.prompt_type,
            visible_count = ui.visible_elements.len(),
            has_bundle_id = bundle_id.is_some(),
            has_clipboard = clipboard.is_some(),
            prior_automation_count = prior_automations.len(),
            context_warning_count,
        );

        TabAiResolvedContext {
            context: crate::ai::TabAiContextBlob::from_parts(
                ui,
                desktop,
                recent_inputs,
                clipboard,
                prior_automations,
                timestamp,
            ),
            bundle_id,
            context_warning_count,
        }
    }

    /// Return a human-readable name for the current `AppView` variant.
    #[allow(dead_code)]
    fn app_view_name(&self) -> String {
        match &self.current_view {
            AppView::ScriptList => "ScriptList".to_string(),
            AppView::ArgPrompt { .. } => "ArgPrompt".to_string(),
            AppView::MiniPrompt { .. } => "MiniPrompt".to_string(),
            AppView::MicroPrompt { .. } => "MicroPrompt".to_string(),
            AppView::DivPrompt { .. } => "DivPrompt".to_string(),
            AppView::FormPrompt { .. } => "FormPrompt".to_string(),
            AppView::TermPrompt { .. } => "TermPrompt".to_string(),
            AppView::EditorPrompt { .. } => "EditorPrompt".to_string(),
            AppView::SelectPrompt { .. } => "SelectPrompt".to_string(),
            AppView::PathPrompt { .. } => "PathPrompt".to_string(),
            AppView::EnvPrompt { .. } => "EnvPrompt".to_string(),
            AppView::DropPrompt { .. } => "DropPrompt".to_string(),
            AppView::TemplatePrompt { .. } => "TemplatePrompt".to_string(),
            AppView::ChatPrompt { .. } => "ChatPrompt".to_string(),
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory".to_string(),
            AppView::AppLauncherView { .. } => "AppLauncher".to_string(),
            AppView::WindowSwitcherView { .. } => "WindowSwitcher".to_string(),
            AppView::FileSearchView { .. } => "FileSearch".to_string(),
            AppView::ThemeChooserView { .. } => "ThemeChooser".to_string(),
            AppView::EmojiPickerView { .. } => "EmojiPicker".to_string(),
            AppView::WebcamView { .. } => "Webcam".to_string(),
            AppView::ScratchPadView { .. } => "ScratchPad".to_string(),
            AppView::QuickTerminalView { .. } => "QuickTerminal".to_string(),
            AppView::NamingPrompt { .. } => "NamingPrompt".to_string(),
            AppView::CreationFeedback { .. } => "CreationFeedback".to_string(),
            AppView::DesignGalleryView { .. } => "DesignGallery".to_string(),
            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => "DesignExplorer".to_string(),
            AppView::ActionsDialog => "ActionsDialog".to_string(),
            AppView::BrowseKitsView { .. } => "BrowseKits".to_string(),
            AppView::InstalledKitsView { .. } => "InstalledKits".to_string(),
            AppView::ProcessManagerView { .. } => "ProcessManager".to_string(),
            AppView::SearchAiPresetsView { .. } => "SearchAiPresets".to_string(),
            AppView::CreateAiPresetView { .. } => "CreateAiPreset".to_string(),
            AppView::SettingsView { .. } => "Settings".to_string(),
            AppView::FavoritesBrowseView { .. } => "FavoritesBrowse".to_string(),
            AppView::CurrentAppCommandsView { .. } => "CurrentAppCommands".to_string(),
        }
    }

    /// Return the current input text from whichever view is active.
    #[allow(dead_code)]
    fn current_input_text(&self) -> Option<String> {
        match &self.current_view {
            AppView::ScriptList => {
                let text = self.filter_text.clone();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. } => {
                let text = self.arg_input.text().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            AppView::ClipboardHistoryView { filter, .. }
            | AppView::AppLauncherView { filter, .. }
            | AppView::WindowSwitcherView { filter, .. }
            | AppView::ThemeChooserView { filter, .. }
            | AppView::EmojiPickerView { filter, .. }
            | AppView::ProcessManagerView { filter, .. }
            | AppView::SearchAiPresetsView { filter, .. }
            | AppView::FavoritesBrowseView { filter, .. }
            | AppView::CurrentAppCommandsView { filter, .. }
            | AppView::DesignGalleryView { filter, .. } => {
                if filter.is_empty() {
                    None
                } else {
                    Some(filter.clone())
                }
            }
            AppView::FileSearchView { query, .. } => {
                if query.is_empty() {
                    None
                } else {
                    Some(query.clone())
                }
            }
            _ => None,
        }
    }

    /// Render the Tab AI overlay element if the overlay is open.
    ///
    /// Returns `None` when the overlay is hidden. The caller layers this
    /// on top of `main_content` using `.when_some(...)`.
    pub(crate) fn render_tab_ai_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        let state = self.tab_ai_state.as_ref()?;
        let theme = crate::theme::get_cached_theme();

        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

        let intent_text: SharedString = state.intent.clone().into();
        let is_running = state.running;
        let error_msg = state.error.clone();
        let memory_hint = state.memory_hint.clone();

        // Placeholder text depends on running state
        let placeholder: SharedString = if is_running {
            "Generating script...".into()
        } else {
            "What do you want to do?".into()
        };

        // Gold accent bar color
        let accent = gpui::rgb(theme.colors.accent.selected);
        // Colors derived from theme hex values
        let bg_scrim = gpui::rgba(
            crate::ui_foundation::hex_to_rgba_with_opacity(theme.colors.background.main, 0.85),
        );
        let card_bg = gpui::rgba(
            crate::ui_foundation::hex_to_rgba_with_opacity(theme.colors.background.main, 0.6),
        );
        let text_primary = gpui::rgb(theme.colors.text.primary);
        let text_hint = gpui::rgba(
            crate::ui_foundation::hex_to_rgba_with_opacity(theme.colors.text.primary, 0.4),
        );
        let text_muted = gpui::rgba(
            crate::ui_foundation::hex_to_rgba_with_opacity(theme.colors.text.primary, 0.5),
        );
        let error_color = gpui::rgb(theme.colors.ui.error);

        // Build the overlay element
        let overlay = div()
            .id("tab-ai-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(bg_scrim)
            .child(
                div()
                    .id("tab-ai-card")
                    .w(px(420.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    // Gold top accent bar
                    .child(div().w_full().h(px(2.)).bg(accent))
                    // Input row
                    .child(
                        div()
                            .w_full()
                            .px(px(12.))
                            .py(px(8.))
                            .bg(card_bg)
                            .rounded_b(px(8.))
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    // Tab glyph
                                    .child(div().text_sm().text_color(accent).child("⇥"))
                                    // Intent text or placeholder
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .font_family(crate::list_item::FONT_MONO)
                                            .when(intent_text.is_empty(), |d| {
                                                d.text_color(text_hint).child(placeholder)
                                            })
                                            .when(!intent_text.is_empty(), |d| {
                                                d.text_color(text_primary)
                                                    .child(intent_text.clone())
                                            }),
                                    )
                                    // Spinner when running
                                    .when(is_running, |d| {
                                        d.child(
                                            div().text_sm().text_color(text_muted).child("⏳"),
                                        )
                                    }),
                            ),
                    )
                    // Prior automation hint row
                    .when_some(memory_hint, |d, hint| {
                        d.child(
                            div()
                                .w_full()
                                .px(px(12.))
                                .py(px(4.))
                                .text_xs()
                                .text_color(text_muted)
                                .child(format!(
                                    "Similar prior automation: {} \u{2014} {} ({:.2})",
                                    hint.slug, hint.effective_query, hint.score
                                )),
                        )
                    })
                    // Error message if present
                    .when_some(error_msg, |d, msg| {
                        d.child(
                            div()
                            .w_full()
                            .px(px(12.))
                            .py(px(4.))
                            .text_xs()
                            .text_color(error_color)
                            .child(msg),
                        )
                    })
                    // Hint strip
                    .child(
                        div()
                            .w_full()
                            .px(px(12.))
                            .py(px(4.))
                            .flex()
                            .flex_row()
                            .justify_between()
                            .text_xs()
                            .text_color(text_hint)
                            .child("↵ Run")
                            .child("Esc Cancel"),
                    ),
            )
            // Keyboard handling: capture keys for this overlay
            .on_key_down(cx.listener(Self::handle_tab_ai_key_down));

        Some(overlay.into_any_element())
    }

    /// Handle key-down events within the Tab AI overlay.
    fn handle_tab_ai_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        if crate::ui_foundation::is_key_escape(key) {
            self.close_tab_ai_overlay(cx);
            cx.stop_propagation();
            return;
        }

        if crate::ui_foundation::is_key_enter(key) {
            if let Some(state) = &self.tab_ai_state {
                if !state.intent.trim().is_empty() && !state.running {
                    self.submit_tab_ai_overlay(cx);
                }
            }
            cx.stop_propagation();
            return;
        }

        if key == "backspace" || key == "Backspace" {
            if let Some(state) = &mut self.tab_ai_state {
                if !state.running {
                    state.intent.pop();
                    self.refresh_tab_ai_memory_hint();
                    cx.notify();
                }
            }
            cx.stop_propagation();
            return;
        }

        // Type printable characters
        if event.keystroke.key.len() == 1
            && !event.keystroke.modifiers.platform
            && !event.keystroke.modifiers.control
            && !event.keystroke.modifiers.alt
        {
            if let Some(state) = &mut self.tab_ai_state {
                if !state.running {
                    state.intent.push_str(&event.keystroke.key);
                    state.error = None; // Clear error on new input
                    self.refresh_tab_ai_memory_hint();
                    cx.notify();
                }
            }
            cx.stop_propagation();
            return;
        }

        // Handle space (it's a printable but might be " " or "space")
        if key == " " || key.eq_ignore_ascii_case("space") {
            if let Some(state) = &mut self.tab_ai_state {
                if !state.running {
                    state.intent.push(' ');
                    state.error = None;
                    self.refresh_tab_ai_memory_hint();
                    cx.notify();
                }
            }
            cx.stop_propagation();
            return;
        }

        // Let other keys propagate
        cx.propagate();
    }

    /// Submit the Tab AI overlay intent — gather context, call AI, execute script.
    fn submit_tab_ai_overlay(&mut self, cx: &mut Context<Self>) {
        let intent = match &self.tab_ai_state {
            Some(state) => state.intent.clone(),
            None => return,
        };

        if intent.trim().is_empty() {
            return;
        }

        tracing::info!(event = "tab_ai_submit", intent = %intent);

        // Mark as running
        if let Some(state) = &mut self.tab_ai_state {
            state.running = true;
            state.error = None;
        }
        cx.notify();

        // Build context blob (returns bundle metadata + warning counts)
        let resolved_context = self.build_tab_ai_context(cx);
        let bundle_id = resolved_context.bundle_id.clone();
        let context_warning_count = resolved_context.context_warning_count;
        let context_json = match serde_json::to_string_pretty(&resolved_context.context) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!(event = "tab_ai_context_serialize_failed", error = %e);
                if let Some(state) = &mut self.tab_ai_state {
                    state.running = false;
                    state.error = Some(format!("Context error: {}", e).into());
                }
                cx.notify();
                return;
            }
        };

        // Build the prompt
        let user_prompt = build_tab_ai_user_prompt(&intent, &context_json);

        tracing::info!(
            event = "tab_ai_execute_start",
            intent_len = intent.len(),
            context_len = context_json.len(),
        );

        // Resolve AI provider + model before spawning the worker thread
        let registry = self
            .cached_provider_registry
            .clone()
            .unwrap_or_else(|| {
                crate::ai::ProviderRegistry::from_environment_with_config(Some(&self.config))
            });

        let selected_model = match crate::prompt_ai::select_default_ai_script_model(&registry) {
            Some(m) => m,
            None => {
                tracing::error!(event = "tab_ai_no_model");
                if let Some(state) = &mut self.tab_ai_state {
                    state.running = false;
                    state.error = Some(
                        "No AI model configured. Open Settings \u{2192} AI and add a provider API key.".into(),
                    );
                }
                cx.notify();
                return;
            }
        };

        let provider = match registry
            .find_provider_for_model(&selected_model.id)
            .cloned()
        {
            Some(p) => p,
            None => {
                tracing::error!(
                    event = "tab_ai_no_provider",
                    model_id = %selected_model.id,
                );
                if let Some(state) = &mut self.tab_ai_state {
                    state.running = false;
                    state.error = Some(
                        "No AI provider matched the selected model. Reopen Settings \u{2192} AI and reselect a model.".into(),
                    );
                }
                cx.notify();
                return;
            }
        };

        let model_id = selected_model.id.clone();
        let provider_id = provider.provider_id().to_string();

        // Channel for worker thread → async GPUI task
        let (tx, rx) = async_channel::bounded::<Result<(String, String), String>>(1);

        // Blocking AI call on worker thread
        let worker_model_id = model_id.clone();
        std::thread::spawn(move || {
            let messages = vec![
                crate::ai::ProviderMessage::system(
                    crate::prompt_ai::AI_SCRIPT_GENERATION_SYSTEM_PROMPT,
                ),
                crate::ai::ProviderMessage::user(&user_prompt),
            ];

            let result = match provider.send_message(&messages, &worker_model_id) {
                Ok(raw_response) => {
                    match crate::ai::script_generation::prepare_script_from_ai_response(
                        &user_prompt,
                        &raw_response,
                    ) {
                        Ok((slug, source)) => Ok((slug, source)),
                        Err(_) => Err(
                            "AI returned no runnable script. Retry with a clearer verb and target.".to_string(),
                        ),
                    }
                }
                Err(error) => Err(format!(
                    "tab_ai_send_message model_id={worker_model_id}: {error:#}"
                )),
            };

            // Ignore send error — the receiver may have been dropped if overlay was closed
            let _ = tx.send_blocking(result);
        });

        // Clone metadata for the async closure
        let dispatch_model_id = model_id.clone();
        let dispatch_provider_id = provider_id.clone();
        let dispatch_bundle_id = bundle_id.clone();

        // Async GPUI task: await response, create temp file, execute
        let app_entity = cx.entity().downgrade();
        let task = cx.spawn(async move |_this, cx| {
            let response = match rx.recv().await {
                Ok(r) => r,
                Err(_) => {
                    tracing::error!(event = "tab_ai_channel_closed");
                    return;
                }
            };

            cx.update(|cx| {
                let Some(app) = app_entity.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| match response {
                    Ok((slug, source)) => {
                        tracing::info!(
                            event = "tab_ai_script_extracted",
                            source_len = source.len(),
                        );

                        // Write to temp file
                        match crate::execution_scripts::create_interactive_temp_script(
                            &source,
                            ".ts",
                            crate::execution_scripts::InteractiveTempFileMode::InterpreterFed,
                        ) {
                            Ok(temp_path) => {
                                let path_str: String = temp_path.to_string_lossy().to_string();
                                tracing::info!(
                                    event = "tab_ai_execute_script",
                                    path = %path_str,
                                );

                                // Capture intent and context before closing overlay
                                let intent = this
                                    .tab_ai_state
                                    .as_ref()
                                    .map(|s| s.intent.clone())
                                    .unwrap_or_default();
                                let prompt_type = this
                                    .tab_ai_state
                                    .as_ref()
                                    .map(|s| s.ui_snapshot.prompt_type.clone())
                                    .unwrap_or_else(|| "Unknown".to_string());

                                // Close overlay before execution
                                this.tab_ai_state = None;
                                this.tab_ai_task = None;
                                cx.notify();

                                // Build execution record with full metadata
                                let record =
                                    crate::ai::TabAiExecutionRecord::from_parts(
                                        intent,
                                        source.clone(),
                                        path_str.clone(),
                                        slug,
                                        prompt_type,
                                        dispatch_bundle_id,
                                        dispatch_model_id.clone(),
                                        dispatch_provider_id.clone(),
                                        context_warning_count,
                                        chrono::Utc::now().to_rfc3339(),
                                    );

                                // Store pending record — bookkeeping deferred until completion
                                this.pending_tab_ai_execution = Some(record.clone());

                                // Write dispatched audit receipt
                                if let Err(e) = crate::ai::append_tab_ai_execution_receipt(
                                    &crate::ai::build_tab_ai_execution_receipt(
                                        &record,
                                        crate::ai::TabAiExecutionStatus::Dispatched,
                                        false,
                                        false,
                                        None,
                                    ),
                                ) {
                                    tracing::warn!(
                                        event = "tab_ai_execution_audit_write_failed",
                                        error = %e,
                                    );
                                }

                                // Execute the ephemeral script
                                this.execute_script_by_path(&path_str, cx);

                                tracing::info!(
                                    event = "tab_ai_post_execution_deferred",
                                    slug = %record.slug,
                                    prompt_type = %record.prompt_type,
                                    model_id = %record.model_id,
                                    provider_id = %record.provider_id,
                                    "memory/save/cleanup deferred until actual completion",
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    event = "tab_ai_temp_file_failed",
                                    error = %e,
                                );
                                if let Some(state) = &mut this.tab_ai_state {
                                    state.running = false;
                                    state.error = Some(
                                        format!("Failed to create temp script: {e}").into(),
                                    );
                                }
                                cx.notify();
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(event = "tab_ai_execute_failed", error = %e);
                        if let Some(state) = &mut this.tab_ai_state {
                            state.running = false;
                            state.error = Some(SharedString::from(e));
                        }
                        cx.notify();
                    }
                });
            });
        });

        self.tab_ai_task = Some(task);
    }

    /// Complete the pending Tab AI execution after the script actually exits.
    ///
    /// Gates memory write-back, save-offer, and temp-file cleanup on real
    /// completion status — never at dispatch time.
    ///
    /// Called from the prompt-handler `ScriptExit` / `ScriptError` paths
    /// once the ephemeral process terminates. Uses `take()` on the pending
    /// record so only the first caller does work — subsequent calls are no-ops.
    pub(crate) fn complete_tab_ai_execution(
        &mut self,
        success: bool,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(record) = self.pending_tab_ai_execution.take() else {
            return;
        };

        let cleanup_attempted = true;
        let cleanup_succeeded = crate::ai::cleanup_tab_ai_temp_script(&record.temp_script_path);

        let status = if success {
            crate::ai::TabAiExecutionStatus::Succeeded
        } else {
            crate::ai::TabAiExecutionStatus::Failed
        };

        let receipt = crate::ai::build_tab_ai_execution_receipt(
            &record,
            status,
            cleanup_attempted,
            cleanup_succeeded,
            error.clone(),
        );

        if let Err(audit_error) = crate::ai::append_tab_ai_execution_receipt(&receipt) {
            tracing::warn!(
                event = "tab_ai_execution_audit_write_failed",
                error = %audit_error,
            );
        }

        if success {
            if let Err(memory_error) = crate::ai::write_tab_ai_memory_entry(&record) {
                tracing::warn!(
                    event = "tab_ai_memory_writeback_failed",
                    error = %memory_error,
                );
            }

            if crate::ai::should_offer_save(&record) {
                tracing::info!(
                    event = "tab_ai_save_offer_open",
                    slug = %record.slug,
                    prompt_type = %record.prompt_type,
                );
                self.open_tab_ai_save_offer(record, cx);
            }
        } else {
            let message = error.unwrap_or_else(|| "Tab AI script failed".to_string());
            self.toast_manager.push(
                components::toast::Toast::error(message, &self.theme)
                    .duration_ms(Some(TOAST_ERROR_MS)),
            );
            cx.notify();
        }
    }
    // ── Tab AI save-offer overlay ──────────────────────────────────────

    fn tab_ai_default_save_name(record: &crate::ai::TabAiExecutionRecord) -> String {
        let derived = super::prompt_ai::derive_script_name_from_description(&record.intent);
        if derived == "ai-generated-script" || derived.is_empty() {
            record.slug.clone()
        } else {
            derived
        }
    }

    fn open_tab_ai_save_offer(
        &mut self,
        record: crate::ai::TabAiExecutionRecord,
        cx: &mut Context<Self>,
    ) {
        let filename_stem = Self::tab_ai_default_save_name(&record);
        tracing::info!(
            event = "tab_ai_save_offer_state_set",
            filename_stem = %filename_stem,
        );
        self.tab_ai_save_offer_state = Some(TabAiSaveOfferState {
            record,
            filename_stem,
            error: None,
        });
        cx.notify();
    }

    fn close_tab_ai_save_offer(&mut self, cx: &mut Context<Self>) {
        if self.tab_ai_save_offer_state.take().is_some() {
            tracing::info!(event = "tab_ai_save_offer_dismissed");
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    fn save_tab_ai_script(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.tab_ai_save_offer_state.clone() else {
            return;
        };

        let created_path = match crate::script_creation::create_new_script(&state.filename_stem) {
            Ok(path) => path,
            Err(error) => {
                tracing::warn!(
                    event = "tab_ai_save_create_failed",
                    error = %error,
                    filename_stem = %state.filename_stem,
                );
                if let Some(save_state) = &mut self.tab_ai_save_offer_state {
                    save_state.error =
                        Some(format!("Failed to create script: {error}").into());
                }
                cx.notify();
                return;
            }
        };

        if let Err(error) = std::fs::write(&created_path, &state.record.generated_source) {
            tracing::warn!(
                event = "tab_ai_save_write_failed",
                error = %error,
                path = %created_path.display(),
            );
            if let Some(save_state) = &mut self.tab_ai_save_offer_state {
                save_state.error =
                    Some(format!("Failed to write script: {error}").into());
            }
            cx.notify();
            return;
        }

        tracing::info!(
            event = "tab_ai_script_saved",
            filename_stem = %state.filename_stem,
            path = %created_path.display(),
        );

        let created_file_path = if created_path.is_absolute() {
            created_path.clone()
        } else {
            match std::env::current_dir() {
                Ok(cwd) => cwd.join(&created_path),
                Err(_) => created_path.clone(),
            }
        };

        let editor_error =
            crate::script_creation::open_in_editor(&created_path, &self.config).err();

        self.tab_ai_save_offer_state = None;

        match editor_error {
            Some(error) => {
                tracing::warn!(
                    event = "tab_ai_save_editor_open_failed",
                    error = %error,
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Saved script but failed to open editor: {error}"),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_ERROR_MS)),
                );
            }
            None => {
                self.toast_manager.push(
                    components::toast::Toast::success(
                        format!(
                            "Saved '{}' and opened in editor",
                            state.filename_stem
                        ),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_SUCCESS_MS)),
                );
            }
        }

        self.current_view = AppView::CreationFeedback {
            path: created_file_path,
        };
        self.opened_from_main_menu = true;
        cx.notify();
    }

    fn handle_tab_ai_save_offer_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        if crate::ui_foundation::is_key_escape(key) {
            self.close_tab_ai_save_offer(cx);
            cx.stop_propagation();
            return;
        }

        if crate::ui_foundation::is_key_enter(key) {
            self.save_tab_ai_script(cx);
            cx.stop_propagation();
            return;
        }

        cx.propagate();
    }

    pub(crate) fn render_tab_ai_save_offer_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        let state = self.tab_ai_save_offer_state.as_ref()?;
        let theme = crate::theme::get_cached_theme();

        // Ensure the main focus handle is focused so key events route here
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

        let accent = gpui::rgb(theme.colors.accent.selected);
        let bg_scrim = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.background.main,
            0.85,
        ));
        let card_bg = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.background.main,
            0.6,
        ));
        let text_primary = gpui::rgb(theme.colors.text.primary);
        let text_hint = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.text.primary,
            0.4,
        ));
        let error_color = gpui::rgb(theme.colors.ui.error);

        let message: SharedString = format!("Save as {}.ts?", state.filename_stem).into();

        let overlay = div()
            .id("tab-ai-save-offer")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(bg_scrim)
            .child(
                div()
                    .id("tab-ai-save-card")
                    .w(px(420.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().w_full().h(px(2.)).bg(accent))
                    .child(
                        div()
                            .w_full()
                            .px(px(12.))
                            .py(px(8.))
                            .bg(card_bg)
                            .rounded_b(px(8.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_family(crate::list_item::FONT_MONO)
                                    .text_color(text_primary)
                                    .child(message),
                            ),
                    )
                    .when_some(state.error.clone(), |d, msg| {
                        d.child(
                            div()
                                .w_full()
                                .px(px(12.))
                                .py(px(4.))
                                .text_xs()
                                .text_color(error_color)
                                .child(msg),
                        )
                    })
                    .child(
                        div()
                            .w_full()
                            .px(px(12.))
                            .py(px(4.))
                            .flex()
                            .flex_row()
                            .justify_between()
                            .text_xs()
                            .text_color(text_hint)
                            .child("↵ Save")
                            .child("Esc Dismiss"),
                    ),
            )
            .on_key_down(cx.listener(Self::handle_tab_ai_save_offer_key_down));

        Some(overlay.into_any_element())
    }
}

/// Re-export the canonical prompt builder so sibling modules and tests can use it.
pub(crate) use crate::ai::tab_context::build_tab_ai_user_prompt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_ai_user_prompt_contains_intent_and_context() {
        let prompt = build_tab_ai_user_prompt("force quit", r#"{"ui":{}}"#);
        assert!(prompt.contains("force quit"));
        assert!(prompt.contains(r#"{"ui":{}}"#));
        assert!(prompt.contains("Script Kit TypeScript"));
    }

    #[test]
    fn tab_ai_user_prompt_contains_code_block_instruction() {
        let prompt = build_tab_ai_user_prompt("test intent", "{}");
        assert!(
            prompt.contains("fenced code block"),
            "Prompt must ask for a fenced code block so extract_generated_script_source works"
        );
    }

    #[test]
    fn tab_ai_user_prompt_separates_intent_from_context() {
        let prompt = build_tab_ai_user_prompt("copy url", r#"{"schemaVersion":1}"#);
        // The intent appears before the context
        let intent_pos = prompt.find("copy url").expect("intent present");
        let context_pos = prompt.find("schemaVersion").expect("context present");
        assert!(
            intent_pos < context_pos,
            "Intent should appear before context JSON"
        );
    }

    #[test]
    fn tab_ai_user_prompt_with_rich_context_json() {
        let context = serde_json::to_string_pretty(&crate::ai::TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                input_text: Some("slack".to_string()),
                focused_semantic_id: Some("input:filter".to_string()),
                selected_semantic_id: Some("choice:0:slack".to_string()),
                visible_elements: vec![],
            },
            Default::default(),
            vec!["recent1".to_string()],
            None,
            vec![],
            "2026-03-28T00:00:00Z".to_string(),
        ))
        .expect("serialize");

        let prompt = build_tab_ai_user_prompt("force quit this app", &context);

        assert!(prompt.contains("force quit this app"));
        assert!(prompt.contains("ScriptList"));
        assert!(prompt.contains("slack"));
        assert!(prompt.contains("choice:0:slack"));
        assert!(prompt.contains("recent1"));
    }

    #[test]
    fn tab_ai_default_save_name_falls_back_to_slug_when_intent_is_generic() {
        let record = crate::ai::TabAiExecutionRecord::from_parts(
            "".to_string(),
            "import \"@scriptkit/sdk\";\nawait notify(\"ok\");\n".to_string(),
            "/tmp/tab-ai.ts".to_string(),
            "tab-ai-script".to_string(),
            "ScriptList".to_string(),
            None,
            "vercel/test-model".to_string(),
            "vercel".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        assert_eq!(
            ScriptListApp::tab_ai_default_save_name(&record),
            "tab-ai-script"
        );
    }

    #[test]
    fn tab_ai_default_save_name_derives_from_intent_when_meaningful() {
        let record = crate::ai::TabAiExecutionRecord::from_parts(
            "force quit this app".to_string(),
            "code".to_string(),
            "/tmp/tab-ai.ts".to_string(),
            "force-quit-this-app".to_string(),
            "ScriptList".to_string(),
            None,
            "gpt-4.1".to_string(),
            "vercel".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        let name = ScriptListApp::tab_ai_default_save_name(&record);
        assert!(
            name.contains("force") && name.contains("quit"),
            "Should derive from intent, got: {name}"
        );
    }
}
