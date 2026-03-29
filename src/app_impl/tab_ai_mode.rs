use super::*;

/// Result of the AI worker thread: either a runnable script, a conversational
/// text response, or a hard failure (provider error, empty response, etc.).
enum TabAiWorkerResult {
    /// AI returned a fenced TypeScript block that parsed into a runnable script.
    Script { slug: String, source: String },
    /// AI returned prose or a non-script response — render as assistant text.
    Text(String),
    /// Hard failure: provider error, empty response, or channel issue.
    Error(String),
}

/// Resolved context returned by `build_tab_ai_context`, carrying bundle metadata
/// and warning counts alongside the serializable blob.
#[derive(Debug, Clone)]
struct TabAiResolvedContext {
    context: crate::ai::TabAiContextBlob,
    bundle_id: Option<String>,
    context_warning_count: usize,
    invocation_receipt: crate::ai::TabAiInvocationReceipt,
}

/// Shared helper that sets Tab AI chat error state on the entity, clears `running`,
/// and emits a structured log event. Used by the full-view chat submit path.
fn set_tab_ai_chat_error(
    entity: &Entity<TabAiChat>,
    cx: &mut Context<ScriptListApp>,
    kind: &'static str,
    message: impl Into<SharedString>,
    remediation: &'static str,
) {
    let message = message.into();
    tracing::warn!(
        target: "script_kit::tab_ai",
        event = "tab_ai_error_state_set",
        kind,
        remediation,
        message = %message,
        "tab ai error state set"
    );
    entity.update(cx, |chat, cx| {
        chat.set_running(false);
        chat.set_error(Some(message));
        cx.notify();
    });
}

/// Split a completed AI response body into word-boundary chunks for progressive
/// reveal in the chat UI. Returns one chunk per whitespace boundary (text) or
/// newline (code). The caller feeds these into `append_turn_chunk` with a small
/// timer between each to simulate streaming.
fn tab_ai_stream_chunks(body: &str, code: bool) -> Vec<String> {
    if code {
        let mut chunks: Vec<String> = body
            .split_inclusive('\n')
            .map(ToString::to_string)
            .collect();
        if chunks.is_empty() {
            chunks.push(body.to_string());
        }
        return chunks;
    }
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in body.chars() {
        current.push(ch);
        if ch.is_whitespace() || current.len() >= 24 {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Maximum visible elements captured per UI snapshot for Tab AI context.
const TAB_AI_VISIBLE_ELEMENT_LIMIT: usize = 24;

/// Maximum visible targets resolved per surface for Tab AI context.
const TAB_AI_VISIBLE_TARGET_LIMIT: usize = 10;

/// Maximum clipboard history entries included in the Tab AI context blob.
const TAB_AI_CLIPBOARD_HISTORY_LIMIT: usize = 8;

/// Maximum character length for hydrated clipboard text entries.
const TAB_AI_CLIPBOARD_TEXT_LIMIT: usize = 1000;

impl ScriptListApp {
    /// Open the Tab AI surface.
    ///
    /// Always opens the full-view `TabAiChat` — the primary power-user
    /// experience. The harness terminal path is preserved for explicit
    /// action invocation but is no longer the implicit Tab entry.
    pub(crate) fn open_tab_ai_chat(&mut self, cx: &mut Context<Self>) {
        if self.tab_ai_save_offer_state.is_some() {
            return;
        }
        self.open_tab_ai_full_view_chat(cx);
    }

    /// Core harness-terminal open: snapshot context, ensure PTY, switch view, inject.
    /// Preserved for future explicit action invocation (e.g. via Actions dialog).
    #[allow(dead_code)]
    fn open_tab_ai_harness_terminal(&mut self, cx: &mut Context<Self>) {
        let source_view = self.current_view.clone();
        let (ui_snapshot, invocation_receipt) = self.snapshot_tab_ai_ui(cx);

        // Emit the receipt as a standalone structured log line for agent/test consumption
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_invocation_receipt",
            prompt_type = %invocation_receipt.prompt_type,
            input_status = %invocation_receipt.input_status,
            focus_status = %invocation_receipt.focus_status,
            elements_status = %invocation_receipt.elements_status,
            has_input_text = invocation_receipt.has_input_text,
            has_focus_target = invocation_receipt.has_focus_target,
            element_count = invocation_receipt.element_count,
            warning_count = invocation_receipt.warning_count,
            rich = invocation_receipt.rich,
            degradation_reasons = ?invocation_receipt.degradation_reasons,
            receipt_json = %serde_json::to_string(&invocation_receipt).unwrap_or_default(),
        );

        let (entity, was_cold_start) = match self.ensure_tab_ai_harness_terminal(cx) {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(
                    event = "tab_ai_harness_start_failed",
                    error = %error,
                );
                self.toast_manager.push(
                    crate::components::toast::Toast::error(
                        format!("Failed to start harness: {error}. Install the configured CLI or update ~/.scriptkit/harness.json"),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_ERROR_MS)),
                );
                cx.notify();
                return;
            }
        };

        // Save the originating surface so Escape and re-entry can use it
        self.tab_ai_harness_return_view = Some(source_view.clone());
        self.tab_ai_harness_return_focus_target = Some(self.tab_ai_return_focus_target());

        self.current_view = AppView::QuickTerminalView {
            entity: entity.clone(),
        };
        self.focused_input = FocusedInput::None;
        self.show_actions_popup = false;
        self.actions_dialog = None;
        self.pending_focus = Some(FocusTarget::TermPrompt);

        // Deferred resize to avoid RefCell borrow error
        cx.spawn(async move |_this, _cx| {
            resize_to_view_sync(ViewType::TermPrompt, 0);
        })
        .detach();
        cx.notify();

        // Build and inject context (harness path captures fresh desktop snapshot)
        let desktop = crate::context_snapshot::capture_context_snapshot(
            &crate::context_snapshot::CaptureContextOptions::tab_ai(),
        );
        let resolved = self.build_tab_ai_context_from(
            String::new(),
            source_view,
            ui_snapshot,
            desktop,
            invocation_receipt,
            cx,
        );

        match crate::ai::build_tab_ai_harness_submission(
            &resolved.context,
            None,
            crate::ai::TabAiHarnessSubmissionMode::PasteOnly,
        ) {
            Ok(submission) => {
                self.inject_tab_ai_harness_submission(entity, submission, was_cold_start, false, cx);
            }
            Err(error) => {
                tracing::warn!(
                    event = "tab_ai_harness_context_build_failed",
                    error = %error,
                );
                self.toast_manager.push(
                    crate::components::toast::Toast::error(
                        format!("Failed to build harness context: {error}"),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_ERROR_MS)),
                );
                cx.notify();
            }
        }
    }

    /// Ensure a harness terminal session exists and is alive.
    /// Returns the entity and whether this was a cold start (newly created).
    #[allow(dead_code)]
    fn ensure_tab_ai_harness_terminal(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Result<(gpui::Entity<crate::term_prompt::TermPrompt>, bool), String> {
        // Reuse existing session if alive
        if let Some(existing) = &self.tab_ai_harness {
            if existing.entity.read(cx).is_alive() {
                return Ok((existing.entity.clone(), false));
            }
        }

        let config = crate::ai::read_tab_ai_harness_config()?;
        crate::ai::validate_tab_ai_harness_config(&config)?;

        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {});

        let term_height = crate::window_resize::layout::MAX_HEIGHT
            - gpui::px(crate::window_resize::layout::FOOTER_HEIGHT);

        let prompt = crate::term_prompt::TermPrompt::with_height(
            "tab-ai-harness".to_string(),
            Some(config.command_line()),
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        )
        .map_err(|e| format!("tab_ai_harness_terminal_create_failed: {e}"))?;

        let entity = cx.new(|_| prompt);

        tracing::info!(
            event = "tab_ai_harness_terminal_created",
            backend = ?config.backend,
            command = %config.command,
        );

        self.tab_ai_harness = Some(crate::ai::TabAiHarnessSessionState::new(
            config,
            entity.clone(),
            "tab-ai-harness",
        ));

        Ok((entity, true))
    }

    /// Inject the context submission into the harness PTY, with a startup
    /// delay for cold starts.
    ///
    /// When `submit` is true, the payload is sent as a full line (appends CR).
    /// When false, the payload is pasted without a trailing CR so the user
    /// can type their intent before pressing Enter.
    #[allow(dead_code)]
    fn inject_tab_ai_harness_submission(
        &self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        submission: String,
        was_cold_start: bool,
        submit: bool,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity().downgrade();
        let entity_weak = entity.downgrade();
        // Give new processes a moment to render their prompt
        let delay_ms: u64 = if was_cold_start { 120 } else { 0 };

        cx.spawn(async move |_this, cx| {
            if delay_ms > 0 {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(delay_ms))
                    .await;
            }

            let _ = cx.update(|cx| {
                let Some(entity) = entity_weak.upgrade() else {
                    return;
                };
                let result = entity.update(cx, |term, _cx| {
                    if submit {
                        term.send_line(&submission).map_err(|e| e.to_string())
                    } else {
                        term.send_text_as_paste(&submission)
                            .map_err(|e| e.to_string())
                    }
                });
                if let Err(error) = result {
                    if let Some(app) = app.upgrade() {
                        app.update(cx, |this, cx| {
                            this.toast_manager.push(
                                crate::components::toast::Toast::error(
                                    format!("Failed to inject Tab AI context: {error}"),
                                    &this.theme,
                                )
                                .duration_ms(Some(TOAST_ERROR_MS)),
                            );
                            cx.notify();
                        });
                    }
                }
            });
        })
        .detach();
    }

    /// Close the Tab AI chat and restore the previous view + focus.
    pub(crate) fn close_tab_ai_chat(&mut self, cx: &mut Context<Self>) {
        let AppView::TabAiChat { entity } = self.current_view.clone() else {
            return;
        };
        let (return_view, return_focus_target) = entity.read(cx).restore_target();
        tracing::info!(
            event = "tab_ai_chat_close",
            focus_target = %format!("{return_focus_target:?}"),
        );
        self.current_view = return_view;
        self.tab_ai_task = None;
        self.pending_focus = Some(return_focus_target);
        cx.notify();
    }

    /// Close the Tab AI harness terminal and restore the previous view + focus.
    pub(crate) fn close_tab_ai_harness_terminal(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.current_view, AppView::QuickTerminalView { .. }) {
            return;
        }
        let return_view = self
            .tab_ai_harness_return_view
            .take()
            .unwrap_or(AppView::ScriptList);
        let return_focus_target = self
            .tab_ai_harness_return_focus_target
            .take()
            .unwrap_or(FocusTarget::MainFilter);

        tracing::info!(
            event = "tab_ai_harness_terminal_close",
            focus_target = %format!("{return_focus_target:?}"),
        );

        self.current_view = return_view;
        self.pending_focus = Some(return_focus_target);
        self.focused_input = match return_focus_target {
            FocusTarget::MainFilter => FocusedInput::MainFilter,
            FocusTarget::ActionsDialog => FocusedInput::ActionsSearch,
            _ => FocusedInput::None,
        };
        cx.notify();
    }

    /// Open Tab AI as a full-view chat with preview context cards.
    ///
    /// Captures desktop context once with `CaptureContextOptions::recommendation()`
    /// and passes it through both card construction and chat state so the preview
    /// cannot drift from what the submit path sees.
    pub(crate) fn open_tab_ai_full_view_chat(&mut self, cx: &mut Context<Self>) {
        if self.tab_ai_save_offer_state.is_some() {
            return;
        }
        if matches!(self.current_view, AppView::TabAiChat { .. }) {
            return;
        }

        let source_view = self.current_view.clone();
        let (ui_snapshot, invocation_receipt) = self.snapshot_tab_ai_ui(cx);

        // Capture desktop context once with the lightweight recommendation profile
        let desktop_preview = crate::context_snapshot::capture_context_snapshot(
            &crate::context_snapshot::CaptureContextOptions::recommendation(),
        );
        let frontmost_bundle_id = desktop_preview
            .frontmost_app
            .as_ref()
            .map(|app| app.bundle_id.clone());

        let context_cards =
            self.build_tab_ai_context_cards(&source_view, &ui_snapshot, &desktop_preview);

        let return_focus_target = self.tab_ai_return_focus_target();

        let focus_handle = self.focus_handle.clone();
        let entity = cx.new(|cx| {
            let mut chat = TabAiChat::new(
                source_view,
                return_focus_target,
                ui_snapshot,
                invocation_receipt,
                frontmost_bundle_id,
                desktop_preview.clone(),
                context_cards,
                focus_handle,
            );
            chat.start_cursor_blink(cx);
            chat
        });

        tracing::info!(
            event = "tab_ai_full_view_chat_open",
            card_count = entity.read(cx).context_cards.len(),
            suggestion_count = entity.read(cx).context_suggestions().len(),
        );

        self.current_view = AppView::TabAiChat { entity };
        self.focused_input = FocusedInput::None;
        self.show_actions_popup = false;
        self.actions_dialog = None;
        self.pending_focus = Some(FocusTarget::TabAiChat);

        cx.spawn(async move |_this, _cx| {
            resize_to_view_sync(ViewType::DivPrompt, 0);
        })
        .detach();
        cx.notify();
    }

    /// Build typed context cards from the current UI state for the Tab AI empty state.
    /// Uses the same resolution paths as the submit flow so the preview cannot drift.
    fn build_tab_ai_context_cards(
        &self,
        source_view: &AppView,
        ui: &crate::ai::TabAiUiSnapshot,
        desktop: &crate::context_snapshot::AiContextSnapshot,
    ) -> Vec<TabAiContextCard> {
        let (focused_target, visible_targets) =
            self.resolve_tab_ai_surface_targets_for_view(source_view, ui);

        let clipboard_selected_index = match source_view {
            AppView::ClipboardHistoryView { selected_index, .. } => Some(*selected_index),
            _ => None,
        };
        let clipboard = self.resolve_tab_ai_clipboard_context(clipboard_selected_index);

        let prior_automations = crate::ai::recent_tab_ai_automations_for_bundle(
            desktop
                .frontmost_app
                .as_ref()
                .map(|app| app.bundle_id.as_str()),
            3,
        )
        .unwrap_or_default();

        let mut cards = Vec::new();

        // -- Selected item card (with suggestions) --
        if let Some(target) = focused_target.as_ref() {
            cards.push(TabAiContextCard {
                kind: TabAiContextCardKind::SelectedItem,
                label: "Selected Item".into(),
                title: target.label.clone().into(),
                body: Some(format!("{} \u{00B7} {}", target.kind, target.source).into()),
                rows: vec![TabAiContextRow::new(
                    "Semantic ID",
                    target.semantic_id.clone(),
                )],
                suggestions: crate::ai::build_tab_ai_suggested_intents(
                    Some(target),
                    clipboard.as_ref(),
                    &prior_automations,
                )
                .into_iter()
                .map(Into::into)
                .collect(),
            });
        }

        // -- Filter text card --
        if let Some(text) = ui.input_text.as_ref().filter(|t| !t.trim().is_empty()) {
            cards.push(TabAiContextCard {
                kind: TabAiContextCardKind::FilterText,
                label: "Filter Text".into(),
                title: text.clone().into(),
                body: None,
                rows: Vec::new(),
                suggestions: Vec::new(),
            });
        }

        // -- Visible items card --
        let visible_target_count = visible_targets.len();
        let visible_rows: Vec<TabAiContextRow> = visible_targets
            .iter()
            .take(5)
            .map(|t| TabAiContextRow::new(t.kind.clone(), t.label.clone()))
            .collect();
        if !visible_rows.is_empty() {
            cards.push(TabAiContextCard {
                kind: TabAiContextCardKind::VisibleItems,
                label: "Visible Items".into(),
                title: format!("{} visible targets", visible_target_count).into(),
                body: None,
                rows: visible_rows,
                suggestions: Vec::new(),
            });
        }

        // -- Desktop context card --
        let mut desktop_rows = Vec::new();
        if let Some(app) = desktop.frontmost_app.as_ref() {
            desktop_rows.push(TabAiContextRow::new("App", app.name.clone()));
        }
        if let Some(browser) = desktop.browser.as_ref() {
            if !browser.url.trim().is_empty() {
                desktop_rows.push(TabAiContextRow::new(
                    "Browser",
                    crate::ai::truncate_tab_ai_text(&browser.url, 80),
                ));
            }
        }
        if let Some(window) = desktop.focused_window.as_ref() {
            if !window.title.trim().is_empty() {
                desktop_rows.push(TabAiContextRow::new("Window", window.title.clone()));
            }
        }
        if let Some(selection) = desktop
            .selected_text
            .as_ref()
            .filter(|t| !t.trim().is_empty())
        {
            desktop_rows.push(TabAiContextRow::new(
                "Selection",
                crate::ai::truncate_tab_ai_text(selection, 80),
            ));
        }
        if !desktop_rows.is_empty() {
            // If no focused target, put default suggestions on the desktop card
            let desktop_suggestions = if focused_target.is_none() {
                crate::ai::build_tab_ai_suggested_intents(
                    None,
                    clipboard.as_ref(),
                    &prior_automations,
                )
                .into_iter()
                .map(Into::into)
                .collect()
            } else {
                Vec::new()
            };
            cards.push(TabAiContextCard {
                kind: TabAiContextCardKind::Desktop,
                label: "Desktop".into(),
                title: "Current Context".into(),
                body: None,
                rows: desktop_rows,
                suggestions: desktop_suggestions,
            });
        }

        // -- Clipboard card --
        if let Some(cb) = clipboard {
            cards.push(TabAiContextCard {
                kind: TabAiContextCardKind::Clipboard,
                label: "Clipboard".into(),
                title: cb.content_type.clone().into(),
                body: Some(cb.preview.clone().into()),
                rows: Vec::new(),
                suggestions: Vec::new(),
            });
        }

        // -- Prior automations card --
        if !prior_automations.is_empty() {
            cards.push(TabAiContextCard {
                kind: TabAiContextCardKind::PriorAutomations,
                label: "Prior Automations".into(),
                title: format!("{} recent successes", prior_automations.len()).into(),
                body: None,
                rows: prior_automations
                    .iter()
                    .map(|item| TabAiContextRow::new(item.slug.clone(), item.effective_query.clone()))
                    .collect(),
                suggestions: prior_automations
                    .iter()
                    .take(1)
                    .map(|item| {
                        TabAiSuggestedIntent::from(crate::ai::TabAiSuggestedIntentSpec::new(
                            format!("Repeat {}", item.slug),
                            item.effective_query.clone(),
                        ))
                    })
                    .collect(),
            });
        }

        cards
    }

    /// Render the Tab AI chat as a full-view wrapper (dispatched from render_impl).
    pub(crate) fn render_tab_ai_chat(
        &mut self,
        entity: Entity<TabAiChat>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let key_entity = entity.clone();
        div()
            .id("tab-ai-chat-root")
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
                this.handle_tab_ai_chat_key_down(key_entity.clone(), event, cx);
            }))
            .child(entity)
    }

    /// Handle key-down events within the Tab AI chat view.
    /// Escape closes, Enter submits, everything else is routed through
    /// `TextInputState::handle_key()` for selection, clipboard, and editing.
    fn handle_tab_ai_chat_key_down(
        &mut self,
        entity: Entity<TabAiChat>,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        if crate::ui_foundation::is_key_escape(key) {
            self.close_tab_ai_chat(cx);
            cx.stop_propagation();
            return;
        }

        // When input is empty, Up/Down cycle suggestions and Enter submits the selected one
        if entity.read(cx).current_intent().trim().is_empty() {
            if crate::ui_foundation::is_key_up(key) {
                let handled = entity.update(cx, |chat, cx| {
                    if chat.context_suggestions().is_empty() {
                        return false;
                    }
                    chat.move_selected_suggestion(-1);
                    cx.notify();
                    true
                });
                if handled {
                    cx.stop_propagation();
                    return;
                }
            }

            if crate::ui_foundation::is_key_down(key) {
                let handled = entity.update(cx, |chat, cx| {
                    if chat.context_suggestions().is_empty() {
                        return false;
                    }
                    chat.move_selected_suggestion(1);
                    cx.notify();
                    true
                });
                if handled {
                    cx.stop_propagation();
                    return;
                }
            }

            if crate::ui_foundation::is_key_enter(key) && !event.keystroke.modifiers.shift {
                let suggestion = entity.read(cx).selected_suggestion();
                if let Some(suggestion) = suggestion {
                    self.submit_tab_ai_chat_with_intent(
                        entity,
                        suggestion.intent.to_string(),
                        cx,
                    );
                    cx.stop_propagation();
                    return;
                }
            }
        }

        if crate::ui_foundation::is_key_enter(key) && !event.keystroke.modifiers.shift {
            if entity.read(cx).can_submit() {
                self.submit_tab_ai_chat(entity, cx);
            }
            cx.stop_propagation();
            return;
        }

        // Let ⌘K propagate so the Actions dialog can open.
        if event.keystroke.modifiers.platform && key.eq_ignore_ascii_case("k") {
            cx.propagate();
            return;
        }

        // Delegate all other keys to TextInputState (handles backspace, delete,
        // arrows, word-jump, select-all, copy, cut, paste, undo, redo, and
        // printable character insertion).
        let key_lower = event.keystroke.key.to_ascii_lowercase();
        let key_char = event.keystroke.key_char.as_deref();
        let handled = entity.update(cx, |chat, cx| {
            if chat.running {
                return false;
            }
            let handled = chat.input.handle_key(
                key_lower.as_str(),
                key_char,
                event.keystroke.modifiers.platform,
                event.keystroke.modifiers.alt,
                event.keystroke.modifiers.shift,
                cx,
            );
            if handled {
                chat.cursor_visible = true;
                chat.error = None;
                chat.refresh_memory_hint();
                cx.notify();
            }
            handled
        });
        if handled {
            cx.stop_propagation();
        } else {
            cx.propagate();
        }
    }

    /// Build context from explicit inputs, resolving targets and clipboard
    /// against the provided `source_view` (the view that was active when Tab
    /// was pressed) rather than `self.current_view` (which is now `TabAiChat`).
    fn build_tab_ai_context_from(
        &self,
        intent_for_lookup: String,
        source_view: AppView,
        ui: crate::ai::TabAiUiSnapshot,
        desktop: crate::context_snapshot::AiContextSnapshot,
        invocation_receipt: crate::ai::TabAiInvocationReceipt,
        _cx: &Context<Self>,
    ) -> TabAiResolvedContext {
        let bundle_id = desktop
            .frontmost_app
            .as_ref()
            .map(|app| app.bundle_id.clone());
        let context_warning_count = desktop.warnings.len();
        let recent_inputs = self.input_history.recent_entries(5);
        let clipboard_selected_index = match &source_view {
            AppView::ClipboardHistoryView { selected_index, .. } => Some(*selected_index),
            _ => None,
        };
        let clipboard_history = self.resolve_tab_ai_clipboard_history(
            clipboard_selected_index,
            TAB_AI_CLIPBOARD_HISTORY_LIMIT,
        );
        let clipboard = clipboard_history.first().map(|entry| {
            crate::ai::TabAiClipboardContext {
                content_type: entry.content_type.clone(),
                preview: entry.preview.clone(),
                ocr_text: entry.ocr_text.clone(),
            }
        });
        let prior_automations = match crate::ai::resolve_tab_ai_memory_suggestions_with_outcome(
            &intent_for_lookup,
            bundle_id.as_deref(),
            3,
        ) {
            Ok(resolution) => resolution.suggestions,
            Err(error) => {
                tracing::warn!(event = "tab_ai_prior_automation_lookup_failed", error = %error);
                Vec::new()
            }
        };
        let (focused_target, visible_targets) =
            self.resolve_tab_ai_surface_targets_for_view(&source_view, &ui);
        let context = crate::ai::TabAiContextBlob::from_parts_with_targets(
            ui,
            focused_target,
            visible_targets,
            desktop,
            recent_inputs,
            clipboard,
            clipboard_history,
            prior_automations,
            chrono::Utc::now().to_rfc3339(),
        );

        TabAiResolvedContext {
            context,
            bundle_id,
            context_warning_count,
            invocation_receipt,
        }
    }

    /// Submit the Tab AI chat intent — gather context, call AI, execute script.
    /// Unlike the overlay version, the chat stays mounted and shows turns.
    fn submit_tab_ai_chat(
        &mut self,
        entity: Entity<TabAiChat>,
        cx: &mut Context<Self>,
    ) {
        let intent = entity.read(cx).current_intent();
        self.submit_tab_ai_chat_with_intent(entity, intent, cx);
    }

    /// Submit an explicit intent string (from typed input or a selected suggestion).
    fn submit_tab_ai_chat_with_intent(
        &mut self,
        entity: Entity<TabAiChat>,
        intent: String,
        cx: &mut Context<Self>,
    ) {
        if intent.trim().is_empty() {
            return;
        }

        if entity.read(cx).running {
            return;
        }

        let (source_view, ui_snapshot, desktop_snapshot, invocation_receipt) = {
            let chat = entity.read(cx);
            (
                chat.restore_target().0,
                chat.ui_snapshot.clone(),
                chat.preview_desktop_snapshot.clone(),
                chat.invocation_receipt.clone(),
            )
        };

        tracing::info!(event = "tab_ai_submit", intent = %intent);

        entity.update(cx, |chat, cx| {
            chat.append_user_turn(intent.clone());
            chat.clear_input();
            chat.set_running(true);
            chat.set_error(None);
            cx.notify();
        });

        let resolved_context = self.build_tab_ai_context_from(
            intent.clone(),
            source_view,
            ui_snapshot,
            desktop_snapshot,
            invocation_receipt,
            cx,
        );

        // Reject implicit-object intents when no stable target exists
        if resolved_context.context.focused_target.is_none()
            && crate::ai::tab_ai_intent_uses_implicit_target(&intent)
        {
            set_tab_ai_chat_error(
                &entity,
                cx,
                "missing_implicit_target",
                "No stable target is selected on this surface. Select an item or describe the target explicitly.",
                "select_target_or_use_explicit_intent",
            );
            return;
        }

        let bundle_id = resolved_context.bundle_id.clone();
        let context_warning_count = resolved_context.context_warning_count;
        let context_json = match serde_json::to_string_pretty(&resolved_context.context) {
            Ok(json) => json,
            Err(error) => {
                set_tab_ai_chat_error(
                    &entity,
                    cx,
                    "context_serialize_failed",
                    format!("Context error: {error}"),
                    "fix_context_serialization",
                );
                return;
            }
        };

        // Emit the invocation receipt at submit time for observability
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_submit_receipt",
            prompt_type = %resolved_context.invocation_receipt.prompt_type,
            rich = resolved_context.invocation_receipt.rich,
            input_status = %resolved_context.invocation_receipt.input_status,
            focus_status = %resolved_context.invocation_receipt.focus_status,
            elements_status = %resolved_context.invocation_receipt.elements_status,
            degradation_reasons = ?resolved_context.invocation_receipt.degradation_reasons,
        );

        let user_prompt = build_tab_ai_user_prompt(&intent, &context_json);

        // Resolve AI provider + model
        let registry = self
            .cached_provider_registry
            .clone()
            .unwrap_or_else(|| {
                crate::ai::ProviderRegistry::from_environment_with_config(Some(&self.config))
            });

        let selected_model = match crate::prompt_ai::select_default_ai_script_model(&registry) {
            Some(m) => m,
            None => {
                set_tab_ai_chat_error(
                    &entity,
                    cx,
                    "no_model_configured",
                    "No AI model configured. Open Settings \u{2192} AI and add a provider API key.",
                    "configure_ai_provider",
                );
                return;
            }
        };

        let provider = match registry
            .find_provider_for_model(&selected_model.id)
            .cloned()
        {
            Some(p) => p,
            None => {
                set_tab_ai_chat_error(
                    &entity,
                    cx,
                    "no_provider_matched",
                    "No AI provider matched the selected model. Reopen Settings \u{2192} AI and reselect a model.",
                    "reselect_model_or_provider",
                );
                return;
            }
        };

        let model_id = selected_model.id.clone();
        let provider_id = provider.provider_id().to_string();

        // Channel for worker thread → async GPUI task
        let (tx, rx) = async_channel::bounded::<TabAiWorkerResult>(1);

        // Create a placeholder assistant turn before the worker starts so the UI
        // shows "Thinking…" immediately while the provider is processing.
        let assistant_turn_index = entity.update(cx, |chat, cx| {
            let ix = chat.start_assistant_turn(TabAiTurnKind::AssistantText);
            cx.notify();
            ix
        });

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
                    if raw_response.trim().is_empty() {
                        TabAiWorkerResult::Error(
                            "AI returned an empty response. Retry with a clearer intent."
                                .to_string(),
                        )
                    } else {
                        match crate::ai::script_generation::prepare_script_from_ai_response(
                            &user_prompt,
                            &raw_response,
                        ) {
                            Ok((slug, source)) => {
                                TabAiWorkerResult::Script { slug, source }
                            }
                            Err(_) => TabAiWorkerResult::Text(raw_response),
                        }
                    }
                }
                Err(error) => TabAiWorkerResult::Error(format!(
                    "tab_ai_send_message model_id={worker_model_id}: {error:#}"
                )),
            };

            let _ = tx.send_blocking(result);
        });

        let dispatch_model_id = model_id.clone();
        let dispatch_provider_id = provider_id.clone();
        let dispatch_bundle_id = bundle_id.clone();
        let app_entity = cx.entity().downgrade();
        let chat_entity = entity.downgrade();

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
                let Some(chat_entity) = chat_entity.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| match response {
                    TabAiWorkerResult::Script { slug, source } => {
                        tracing::info!(
                            event = "tab_ai_script_extracted",
                            source_len = source.len(),
                        );

                        // Reveal the code into the pre-created assistant turn
                        chat_entity.update(cx, |chat, cx| {
                            chat.set_turn_kind(assistant_turn_index, TabAiTurnKind::AssistantCode);
                            chat.append_turn_chunk(assistant_turn_index, &source);
                            chat.complete_turn_stream(assistant_turn_index);
                            chat.set_running(false);
                            cx.notify();
                        });

                        match crate::execution_scripts::create_interactive_temp_script(
                            &source,
                            ".ts",
                            crate::execution_scripts::InteractiveTempFileMode::InterpreterFed,
                        ) {
                            Ok(temp_path) => {
                                let path_str: String = temp_path.to_string_lossy().to_string();

                                let prompt_type =
                                    chat_entity.read(cx).ui_snapshot.prompt_type.clone();

                                let record = crate::ai::TabAiExecutionRecord::from_parts(
                                    intent.clone(),
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

                                this.pending_tab_ai_execution = Some(record.clone());

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

                                this.execute_script_by_path(&path_str, cx);
                            }
                            Err(e) => {
                                set_tab_ai_chat_error(
                                    &chat_entity,
                                    cx,
                                    "temp_script_create_failed",
                                    format!("Failed to create temp script: {e}"),
                                    "check_temp_dir_permissions",
                                );
                            }
                        }
                    }
                    TabAiWorkerResult::Text(text) => {
                        tracing::info!(
                            event = "tab_ai_text_response",
                            text_len = text.len(),
                        );

                        // Progressively reveal text chunks into the pre-created
                        // assistant turn to simulate streaming.
                        let chunks = tab_ai_stream_chunks(&text, false);
                        let ce = chat_entity.clone();
                        cx.spawn(async move |_this, cx| {
                            for chunk in chunks {
                                cx.background_executor()
                                    .timer(std::time::Duration::from_millis(16))
                                    .await;
                                cx.update(|cx| {
                                    ce.update(cx, |chat, cx| {
                                        chat.append_turn_chunk(
                                            assistant_turn_index,
                                            &chunk,
                                        );
                                        cx.notify();
                                    });
                                });
                            }
                            cx.update(|cx| {
                                ce.update(cx, |chat, cx| {
                                    chat.complete_turn_stream(assistant_turn_index);
                                    chat.set_running(false);
                                    cx.notify();
                                });
                            });
                        })
                        .detach();
                    }
                    TabAiWorkerResult::Error(e) => {
                        // Complete the placeholder turn before showing the error
                        chat_entity.update(cx, |chat, cx| {
                            chat.complete_turn_stream(assistant_turn_index);
                            chat.set_running(false);
                            cx.notify();
                        });
                        set_tab_ai_chat_error(
                            &chat_entity,
                            cx,
                            "ai_execution_failed",
                            e,
                            "retry_with_clearer_intent_or_check_provider_logs",
                        );
                    }
                });
            });
        });

        self.tab_ai_task = Some(task);
    }

    /// Return the correct `FocusTarget` for the originating surface so that
    /// closing the Tab AI overlay restores focus to the right place.
    fn tab_ai_return_focus_target(&self) -> FocusTarget {
        match &self.current_view {
            AppView::ScriptList
            | AppView::ClipboardHistoryView { .. }
            | AppView::AppLauncherView { .. }
            | AppView::WindowSwitcherView { .. }
            | AppView::FileSearchView { .. }
            | AppView::ThemeChooserView { .. }
            | AppView::EmojiPickerView { .. }
            | AppView::BrowseKitsView { .. }
            | AppView::InstalledKitsView { .. }
            | AppView::ProcessManagerView { .. }
            | AppView::SearchAiPresetsView { .. }
            | AppView::CreateAiPresetView { .. }
            | AppView::SettingsView { .. }
            | AppView::FavoritesBrowseView { .. }
            | AppView::CurrentAppCommandsView { .. }
            | AppView::DesignGalleryView { .. }
            | AppView::CreationFeedback { .. }
            | AppView::ActionsDialog => FocusTarget::MainFilter,

            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. }
            | AppView::DivPrompt { .. }
            | AppView::WebcamView { .. } => FocusTarget::AppRoot,

            AppView::FormPrompt { .. } => FocusTarget::FormPrompt,

            AppView::EditorPrompt { .. } | AppView::ScratchPadView { .. } => {
                FocusTarget::EditorPrompt
            }

            AppView::SelectPrompt { .. } => FocusTarget::SelectPrompt,
            AppView::PathPrompt { .. } => FocusTarget::PathPrompt,
            AppView::EnvPrompt { .. } => FocusTarget::EnvPrompt,
            AppView::DropPrompt { .. } => FocusTarget::DropPrompt,
            AppView::TemplatePrompt { .. } => FocusTarget::TemplatePrompt,

            AppView::TermPrompt { .. } | AppView::QuickTerminalView { .. } => {
                FocusTarget::TermPrompt
            }

            AppView::ChatPrompt { .. } => FocusTarget::ChatPrompt,
            AppView::NamingPrompt { .. } => FocusTarget::NamingPrompt,
            AppView::TabAiChat { .. } => FocusTarget::TabAiChat,

            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => FocusTarget::AppRoot,
        }
    }

    /// Resolve a bounded clipboard history with hydrated content from the
    /// clipboard database. The selected entry (if any) is always first,
    /// followed by other recent entries in order, deduplicated by content.
    fn resolve_tab_ai_clipboard_history(
        &self,
        selected_index: Option<usize>,
        limit: usize,
    ) -> Vec<crate::ai::TabAiClipboardHistoryEntry> {
        let mut ordered: Vec<&crate::clipboard_history::ClipboardEntryMeta> = Vec::new();

        // Selected entry goes first
        if let Some(selected) =
            selected_index.and_then(|index| self.cached_clipboard_entries.get(index))
        {
            ordered.push(selected);
        }

        for entry in &self.cached_clipboard_entries {
            if ordered.iter().any(|candidate| candidate.id == entry.id) {
                continue;
            }
            ordered.push(entry);
            if ordered.len() >= limit {
                break;
            }
        }

        let mut last_dedupe_key: Option<String> = None;
        let mut result = Vec::new();

        for entry in ordered {
            // Hydrate text content from the database for text-like entries
            let full_text = match entry.content_type {
                crate::clipboard_history::ContentType::Text
                | crate::clipboard_history::ContentType::Link
                | crate::clipboard_history::ContentType::File
                | crate::clipboard_history::ContentType::Color => {
                    crate::clipboard_history::get_entry_content(&entry.id)
                        .filter(|content| !content.trim().is_empty())
                        .map(|content| {
                            crate::ai::truncate_tab_ai_text(&content, TAB_AI_CLIPBOARD_TEXT_LIMIT)
                        })
                }
                crate::clipboard_history::ContentType::Image => None,
            };

            let preview_source = full_text
                .clone()
                .or_else(|| entry.ocr_text.clone())
                .unwrap_or_else(|| entry.display_preview());

            // Deduplicate consecutive identical entries
            let dedupe_key = format!("{}::{}", entry.content_type.as_str(), preview_source);
            if last_dedupe_key.as_deref() == Some(dedupe_key.as_str()) {
                continue;
            }
            last_dedupe_key = Some(dedupe_key);

            result.push(crate::ai::TabAiClipboardHistoryEntry {
                id: entry.id.clone(),
                content_type: entry.content_type.as_str().to_string(),
                timestamp: entry.timestamp,
                preview: crate::ai::truncate_tab_ai_text(
                    &preview_source,
                    TAB_AI_CLIPBOARD_TEXT_LIMIT,
                ),
                full_text,
                ocr_text: entry
                    .ocr_text
                    .clone()
                    .filter(|text| !text.trim().is_empty())
                    .map(|text| {
                        crate::ai::truncate_tab_ai_text(&text, TAB_AI_CLIPBOARD_TEXT_LIMIT)
                    }),
                image_width: entry.image_width,
                image_height: entry.image_height,
            });
        }

        result
    }

    /// Build a single clipboard context summary (legacy compat for context cards).
    fn resolve_tab_ai_clipboard_context(
        &self,
        selected_index: Option<usize>,
    ) -> Option<crate::ai::TabAiClipboardContext> {
        self.resolve_tab_ai_clipboard_history(selected_index, 1)
            .into_iter()
            .next()
            .map(|entry| crate::ai::TabAiClipboardContext {
                content_type: entry.content_type,
                preview: entry.preview,
                ocr_text: entry.ocr_text,
            })
    }

    fn tab_ai_target_from_element(
        prompt_type: &str,
        element: &crate::protocol::ElementInfo,
    ) -> crate::ai::TabAiTargetContext {
        crate::ai::TabAiTargetContext {
            source: prompt_type.to_string(),
            kind: format!("{:?}", element.element_type).to_lowercase(),
            semantic_id: element.semantic_id.clone(),
            label: element
                .text
                .clone()
                .or_else(|| element.value.clone())
                .unwrap_or_else(|| element.semantic_id.clone()),
            metadata: Some(serde_json::json!({
                "text": element.text.clone(),
                "value": element.value.clone(),
                "selected": element.selected,
                "focused": element.focused,
                "index": element.index,
            })),
        }
    }

    /// Source-view-aware target resolution: resolves targets against an explicit view
    /// instead of `self.current_view`. Used at submit time when `current_view`
    /// has already switched to `TabAiChat`.
    fn resolve_tab_ai_surface_targets_for_view(
        &self,
        view: &AppView,
        ui: &crate::ai::TabAiUiSnapshot,
    ) -> (
        Option<crate::ai::TabAiTargetContext>,
        Vec<crate::ai::TabAiTargetContext>,
    ) {
        match view {
            AppView::ClipboardHistoryView { selected_index, .. } => {
                let focused_target =
                    self.cached_clipboard_entries
                        .get(*selected_index)
                        .map(|entry| {
                            let preview = if entry.content_type.as_str() == "image" {
                                entry
                                    .ocr_text
                                    .clone()
                                    .filter(|text| !text.trim().is_empty())
                                    .unwrap_or_else(|| entry.display_preview())
                            } else {
                                entry.display_preview()
                            };
                            crate::ai::TabAiTargetContext {
                                source: "ClipboardHistory".to_string(),
                                kind: "clipboard_entry".to_string(),
                                semantic_id: crate::protocol::generate_semantic_id(
                                    "choice",
                                    *selected_index,
                                    &entry.text_preview,
                                ),
                                label: preview.clone(),
                                metadata: Some(serde_json::json!({
                                    "id": entry.id.clone(),
                                    "timestamp": entry.timestamp,
                                    "contentType": entry.content_type.as_str(),
                                    "preview": preview,
                                    "ocrText": entry.ocr_text.clone(),
                                    "imageWidth": entry.image_width,
                                    "imageHeight": entry.image_height,
                                })),
                            }
                        });
                let visible_targets = self
                    .cached_clipboard_entries
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| {
                        let preview = if entry.content_type.as_str() == "image" {
                            entry
                                .ocr_text
                                .clone()
                                .filter(|text| !text.trim().is_empty())
                                .unwrap_or_else(|| entry.display_preview())
                        } else {
                            entry.display_preview()
                        };
                        crate::ai::TabAiTargetContext {
                            source: "ClipboardHistory".to_string(),
                            kind: "clipboard_entry".to_string(),
                            semantic_id: crate::protocol::generate_semantic_id(
                                "choice",
                                index,
                                &entry.text_preview,
                            ),
                            label: preview.clone(),
                            metadata: Some(serde_json::json!({
                                "id": entry.id.clone(),
                                "timestamp": entry.timestamp,
                                "contentType": entry.content_type.as_str(),
                                "preview": preview,
                                "ocrText": entry.ocr_text.clone(),
                                "imageWidth": entry.image_width,
                                "imageHeight": entry.image_height,
                            })),
                        }
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::FileSearchView { selected_index, .. } => {
                let focused_target = self.cached_file_results.get(*selected_index).map(|entry| {
                    crate::ai::TabAiTargetContext {
                        source: "FileSearch".to_string(),
                        kind: if entry.file_type == crate::file_search::FileType::Directory {
                            "directory".to_string()
                        } else {
                            "file".to_string()
                        },
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            *selected_index,
                            &entry.name,
                        ),
                        label: entry.name.clone(),
                        metadata: Some(serde_json::json!({
                            "path": entry.path.clone(),
                            "fileType": format!("{:?}", entry.file_type),
                        })),
                    }
                });
                let visible_targets = self
                    .cached_file_results
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| crate::ai::TabAiTargetContext {
                        source: "FileSearch".to_string(),
                        kind: if entry.file_type == crate::file_search::FileType::Directory {
                            "directory".to_string()
                        } else {
                            "file".to_string()
                        },
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            index,
                            &entry.name,
                        ),
                        label: entry.name.clone(),
                        metadata: Some(serde_json::json!({
                            "path": entry.path.clone(),
                            "fileType": format!("{:?}", entry.file_type),
                        })),
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::WindowSwitcherView { selected_index, .. } => {
                let focused_target = self.cached_windows.get(*selected_index).map(|entry| {
                    let label = format!("{} — {}", entry.app, entry.title);
                    crate::ai::TabAiTargetContext {
                        source: "WindowSwitcher".to_string(),
                        kind: "window".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            *selected_index,
                            &label,
                        ),
                        label,
                        metadata: Some(serde_json::json!({
                            "app": entry.app.clone(),
                            "title": entry.title.clone(),
                        })),
                    }
                });
                let visible_targets = self
                    .cached_windows
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| {
                        let label = format!("{} — {}", entry.app, entry.title);
                        crate::ai::TabAiTargetContext {
                            source: "WindowSwitcher".to_string(),
                            kind: "window".to_string(),
                            semantic_id: crate::protocol::generate_semantic_id(
                                "choice",
                                index,
                                &label,
                            ),
                            label,
                            metadata: Some(serde_json::json!({
                                "app": entry.app.clone(),
                                "title": entry.title.clone(),
                            })),
                        }
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::AppLauncherView { selected_index, .. } => {
                let focused_target = self.apps.get(*selected_index).map(|app| {
                    crate::ai::TabAiTargetContext {
                        source: "AppLauncher".to_string(),
                        kind: "app".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            *selected_index,
                            &app.name,
                        ),
                        label: app.name.clone(),
                        metadata: Some(serde_json::json!({
                            "name": app.name.clone(),
                        })),
                    }
                });
                let visible_targets = self
                    .apps
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, app)| crate::ai::TabAiTargetContext {
                        source: "AppLauncher".to_string(),
                        kind: "app".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            index,
                            &app.name,
                        ),
                        label: app.name.clone(),
                        metadata: Some(serde_json::json!({
                            "name": app.name.clone(),
                        })),
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::ProcessManagerView { selected_index, .. } => {
                let focused_target =
                    self.cached_processes.get(*selected_index).map(|process| {
                        crate::ai::TabAiTargetContext {
                            source: "ProcessManager".to_string(),
                            kind: "process".to_string(),
                            semantic_id: crate::protocol::generate_semantic_id(
                                "choice",
                                *selected_index,
                                &process.script_path,
                            ),
                            label: process.script_path.clone(),
                            metadata: Some(serde_json::json!({
                                "scriptPath": process.script_path.clone(),
                            })),
                        }
                    });
                let visible_targets = self
                    .cached_processes
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, process)| crate::ai::TabAiTargetContext {
                        source: "ProcessManager".to_string(),
                        kind: "process".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            index,
                            &process.script_path,
                        ),
                        label: process.script_path.clone(),
                        metadata: Some(serde_json::json!({
                            "scriptPath": process.script_path.clone(),
                        })),
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::CurrentAppCommandsView { selected_index, .. } => {
                let focused_target = self
                    .cached_current_app_entries
                    .get(*selected_index)
                    .map(|entry| crate::ai::TabAiTargetContext {
                        source: "CurrentAppCommands".to_string(),
                        kind: "menu_command".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            *selected_index,
                            &entry.name,
                        ),
                        label: entry.name.clone(),
                        metadata: Some(serde_json::json!({
                            "name": entry.name.clone(),
                        })),
                    });
                let visible_targets = self
                    .cached_current_app_entries
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| crate::ai::TabAiTargetContext {
                        source: "CurrentAppCommands".to_string(),
                        kind: "menu_command".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            index,
                            &entry.name,
                        ),
                        label: entry.name.clone(),
                        metadata: Some(serde_json::json!({
                            "name": entry.name.clone(),
                        })),
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            _ => {
                let visible_targets: Vec<crate::ai::TabAiTargetContext> = ui
                    .visible_elements
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .map(|element| Self::tab_ai_target_from_element(&ui.prompt_type, element))
                    .collect();

                let focused_target = ui
                    .selected_semantic_id
                    .as_deref()
                    .or(ui.focused_semantic_id.as_deref())
                    .and_then(|semantic_id| {
                        visible_targets
                            .iter()
                            .find(|target| target.semantic_id == semantic_id)
                            .cloned()
                            .or_else(|| {
                                ui.visible_elements
                                    .iter()
                                    .find(|element| element.semantic_id == semantic_id)
                                    .map(|element| {
                                        Self::tab_ai_target_from_element(
                                            &ui.prompt_type,
                                            element,
                                        )
                                    })
                            })
                    });

                (focused_target, visible_targets)
            }
        }
    }

    /// Capture a snapshot of the current UI state for context assembly.
    ///
    /// Returns the snapshot and a machine-readable invocation receipt that
    /// identifies whether UI context was rich or degraded with explicit
    /// reason codes.
    #[allow(dead_code)]
    fn snapshot_tab_ai_ui(
        &self,
        cx: &Context<Self>,
    ) -> (crate::ai::TabAiUiSnapshot, crate::ai::TabAiInvocationReceipt) {
        let prompt_type = self.app_view_name();

        // Collect visible elements
        let outcome = self.collect_visible_elements(TAB_AI_VISIBLE_ELEMENT_LIMIT, cx);

        let input_text = self.current_input_text(cx);
        let focused_id = outcome.focused_semantic_id();
        let selected_id = outcome.selected_semantic_id();

        // Build the machine-readable invocation receipt
        let receipt = crate::ai::TabAiInvocationReceipt::from_snapshot(
            &prompt_type,
            &input_text,
            &focused_id,
            &selected_id,
            outcome.elements.len(),
            &outcome.warnings,
        );

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_snapshot_captured",
            prompt_type = %prompt_type,
            input_status = %receipt.input_status,
            focus_status = %receipt.focus_status,
            elements_status = %receipt.elements_status,
            has_input_text = receipt.has_input_text,
            has_focus_target = receipt.has_focus_target,
            element_count = receipt.element_count,
            warning_count = receipt.warning_count,
            rich = receipt.rich,
            degradation_reasons = ?receipt.degradation_reasons,
            "tab ai snapshot captured"
        );

        let snapshot = crate::ai::TabAiUiSnapshot {
            prompt_type,
            input_text,
            focused_semantic_id: focused_id,
            selected_semantic_id: selected_id,
            visible_elements: outcome.elements,
        };

        (snapshot, receipt)
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
            AppView::TabAiChat { .. } => "TabAiChat".to_string(),
        }
    }

    /// Return the current input text from whichever view is active.
    ///
    /// Returns `Some(text)` when the view has user-editable text that is
    /// non-empty, `None` otherwise.  Entity-based prompts are read via
    /// `entity.read(cx)` so this method requires a context reference.
    #[allow(dead_code)]
    fn current_input_text(&self, cx: &Context<Self>) -> Option<String> {
        let non_empty = |s: String| if s.is_empty() { None } else { Some(s) };

        match &self.current_view {
            AppView::ScriptList => non_empty(self.filter_text.clone()),

            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. } => {
                non_empty(self.arg_input.text().to_string())
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
            | AppView::DesignGalleryView { filter, .. } => non_empty(filter.clone()),

            AppView::FileSearchView { query, .. } => non_empty(query.clone()),

            AppView::BrowseKitsView { query, .. } => non_empty(query.clone()),

            // --- Entity-based prompts ---

            AppView::EditorPrompt { entity, .. } => {
                entity.read_with(cx, |editor, app| {
                    non_empty(editor.content_from_app(app))
                })
            }
            AppView::ScratchPadView { entity, .. } => {
                entity.read_with(cx, |editor, app| {
                    non_empty(editor.content_from_app(app))
                })
            }
            AppView::ChatPrompt { entity, .. } => {
                non_empty(entity.read(cx).input.text().to_string())
            }
            AppView::PathPrompt { entity, .. } => {
                let p = entity.read(cx);
                // Prefer active filter text; fall back to current directory path
                non_empty(p.filter_text.clone())
                    .or_else(|| non_empty(p.current_path.clone()))
            }
            AppView::EnvPrompt { entity, .. } => {
                let p = entity.read(cx);
                // Return the user-entered value (masked text is still useful
                // for "is something typed?" without revealing secrets)
                if p.secret {
                    // For secret fields, report presence but not content
                    let text = p.input_text();
                    if text.is_empty() { None } else { Some("[secret]".to_string()) }
                } else {
                    non_empty(p.input_text().to_string())
                }
            }
            AppView::SelectPrompt { entity, .. } => {
                non_empty(entity.read(cx).filter_text.clone())
            }
            AppView::NamingPrompt { entity, .. } => {
                non_empty(entity.read(cx).friendly_name.clone())
            }
            AppView::TemplatePrompt { entity, .. } => {
                let p = entity.read(cx);
                // Return the value of the currently focused template input
                p.values.get(p.current_input).and_then(|v| non_empty(v.clone()))
            }
            AppView::CreateAiPresetView { name, system_prompt, model, active_field } => {
                // Return whichever field is active
                match active_field {
                    0 => non_empty(name.clone()),
                    1 => non_empty(system_prompt.clone()),
                    2 => non_empty(model.clone()),
                    _ => non_empty(name.clone()),
                }
            }

            // --- Views with no meaningful user-editable text ---
            // DivPrompt: script-rendered HTML, no user input
            // FormPrompt: multi-field form — field values are in elements,
            //   not a single "input text" (use visible_elements instead)
            // TermPrompt/QuickTerminal: terminal content, not user text input
            // DropPrompt: file drop zone, no typed text
            // WebcamView: camera feed, no text
            // CreationFeedback: read-only confirmation
            // ActionsDialog: transient overlay, not a primary surface
            // SettingsView/InstalledKitsView: navigation-only, no free text
            AppView::TabAiChat { entity } => non_empty(entity.read(cx).current_intent()),

            AppView::DivPrompt { .. }
            | AppView::FormPrompt { .. }
            | AppView::TermPrompt { .. }
            | AppView::QuickTerminalView { .. }
            | AppView::DropPrompt { .. }
            | AppView::WebcamView { .. }
            | AppView::CreationFeedback { .. }
            | AppView::ActionsDialog
            | AppView::SettingsView { .. }
            | AppView::InstalledKitsView { .. } => None,

            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => None,
        }
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

        // Push completion status into the Tab AI chat if it's the current view
        if let AppView::TabAiChat { entity } = &self.current_view {
            let status_message = if success {
                "Script finished successfully.".to_string()
            } else {
                error
                    .clone()
                    .unwrap_or_else(|| "Tab AI script failed".to_string())
            };
            entity.update(cx, |chat, cx| {
                chat.append_assistant_text_turn(status_message);
                cx.notify();
            });
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
            self.pending_focus = Some(match self.current_view {
                AppView::TabAiChat { .. } => FocusTarget::TabAiChat,
                _ => FocusTarget::MainFilter,
            });
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

        // Whisper chrome colors — same tokens as the main Tab AI overlay
        let bg_scrim = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.background.main,
            crate::theme::opacity::OPACITY_NEAR_FULL,
        ));
        let text_primary = gpui::rgb(theme.colors.text.primary);
        let error_color = gpui::rgb(theme.colors.ui.error);
        let divider_rgba = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.text.primary,
            crate::theme::opacity::OPACITY_GHOST,
        ));

        let hint_px: f32 = crate::window_resize::mini_layout::HINT_STRIP_PADDING_X;

        let message: SharedString = format!("Save as {}.ts?", state.filename_stem).into();

        // Full-width inline panel — matches main Tab AI overlay chrome,
        // not a floating card. Footer uses HintStrip with save-specific
        // hints (justified exception: this is a confirmation dialog, not
        // the primary input surface).
        let overlay = div()
            .id("tab-ai-save-offer")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .bg(bg_scrim)
            // Message row — bare text, no card, no accent bar
            .child(
                div()
                    .w_full()
                    .px(px(hint_px))
                    .py(px(10.))
                    .child(
                        div()
                            .text_sm()
                            .font_family(crate::list_item::FONT_MONO)
                            .text_color(text_primary)
                            .child(message),
                    ),
            )
            // Hairline divider — ghost opacity
            .child(div().w_full().h(px(1.)).bg(divider_rgba))
            // Error message if present — below divider, minimal
            .when_some(state.error.clone(), |d, msg| {
                d.child(
                    div()
                        .w_full()
                        .px(px(hint_px))
                        .py(px(4.))
                        .text_xs()
                        .text_color(error_color)
                        .child(msg),
                )
            })
            // Spacer pushes footer to bottom
            .child(div().flex_1())
            // Footer — save-specific hint strip via shared component
            // (justified exception: confirmation dialog uses ↵ Save / Esc Dismiss
            // instead of the canonical three-key strip)
            .child(components::HintStrip::new(vec![
                "\u{21B5} Save".into(),
                "Esc Dismiss".into(),
            ]))
            .on_key_down(cx.listener(Self::handle_tab_ai_save_offer_key_down));

        Some(overlay.into_any_element())
    }
}

impl Render for TabAiChat {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let bg = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.background.main,
            crate::theme::opacity::OPACITY_NEAR_FULL,
        ));
        let divider = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.text.primary,
            crate::theme::opacity::OPACITY_GHOST,
        ));
        let hint = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.text.primary,
            crate::theme::opacity::OPACITY_DISABLED,
        ));
        let text_color = gpui::rgb(theme.colors.text.primary);
        let accent_color = gpui::rgb(theme.colors.accent.selected);
        let error_color = gpui::rgb(theme.colors.ui.error);

        let is_focused = self.focus_handle.is_focused(window);
        let input_text = self.input.text();
        let cursor_pos = self.input.cursor().min(input_text.chars().count());
        let chars: Vec<char> = input_text.chars().collect();
        let before: SharedString = chars[..cursor_pos].iter().collect::<String>().into();
        let after: SharedString = chars[cursor_pos..].iter().collect::<String>().into();

        let placeholder: SharedString = if self.running {
            "Generating\u{2026}".into()
        } else {
            self.input_placeholder()
        };

        let card_ghost_bg = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            theme.colors.background.main,
            crate::theme::opacity::OPACITY_GHOST,
        ));

        // Memory hint text
        let memory_hint_element = self.memory_hint.as_ref().map(|mh| {
            div()
                .w_full()
                .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
                .pb(px(4.))
                .text_xs()
                .text_color(hint)
                .child(SharedString::from(format!(
                    "Similar prior automation: {} \u{2014} {} ({:.2})",
                    mh.slug, mh.effective_query, mh.score
                )))
        });

        // Collect suggestion pills across all cards (capped to 3)
        let all_suggestions = self.context_suggestions();
        let selected_idx = self.selected_suggestion_index;

        // Center body: context cards (empty state) or scrollable turns list
        let body: gpui::AnyElement = if self.turns.is_empty() {
            let context_cards: Vec<gpui::AnyElement> = self
                .context_cards
                .iter()
                .cloned()
                .map(|card| {
                    let card_id = match card.kind {
                        TabAiContextCardKind::SelectedItem => "tab-ai-card-selected",
                        TabAiContextCardKind::FilterText => "tab-ai-card-filter",
                        TabAiContextCardKind::VisibleItems => "tab-ai-card-visible",
                        TabAiContextCardKind::Desktop => "tab-ai-card-desktop",
                        TabAiContextCardKind::Clipboard => "tab-ai-card-clipboard",
                        TabAiContextCardKind::PriorAutomations => "tab-ai-card-automations",
                    };
                    let mut card_el = div()
                        .id(card_id)
                        .w_full()
                        .mb(px(8.))
                        .px(px(12.))
                        .py(px(10.))
                        .bg(card_ghost_bg)
                        .child(
                            div()
                                .text_xs()
                                .text_color(hint)
                                .child(card.label),
                        )
                        // Single-line title
                        .child(
                            div()
                                .mt(px(4.))
                                .text_sm()
                                .text_color(text_color)
                                .child(card.title),
                        );

                    // Optional body
                    if let Some(body_text) = card.body {
                        card_el = card_el.child(
                            div()
                                .mt(px(2.))
                                .text_xs()
                                .text_color(hint)
                                .child(body_text),
                        );
                    }

                    // Up to 5 rows
                    for row in card.rows.iter().take(5) {
                        card_el = card_el.child(
                            div()
                                .mt(px(2.))
                                .flex()
                                .flex_row()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(hint)
                                        .child(row.label.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child(row.value.clone()),
                                ),
                        );
                    }

                    card_el.into_any_element()
                })
                .collect();

            // Build suggestion pills (capped to 3 across the whole surface)
            let suggestion_pills: Vec<gpui::AnyElement> = all_suggestions
                .iter()
                .enumerate()
                .map(|(idx, suggestion)| {
                    let is_selected = idx == selected_idx;
                    let pill_bg = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
                        theme.colors.accent.selected,
                        0.15,
                    ));
                    div()
                        .px(px(8.))
                        .py(px(4.))
                        .text_xs()
                        .when(is_selected, |d| {
                            d.bg(pill_bg)
                                .text_color(accent_color)
                        })
                        .when(!is_selected, |d| d.text_color(hint))
                        .child(suggestion.label.clone())
                        .into_any_element()
                })
                .collect();

            let mut container = div()
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .px(px(12.))
                .py(px(12.))
                .children(context_cards);

            if !suggestion_pills.is_empty() {
                container = container.child(
                    div()
                        .mt(px(8.))
                        .flex()
                        .flex_row()
                        .gap(px(4.))
                        .children(suggestion_pills),
                );
            }

            container.into_any_element()
        } else {
            let turns = self.turns.clone();
            let entity = cx.entity();
            list(self.turns_list_state.clone(), move |ix, _window, _cx| {
                let theme = crate::theme::get_cached_theme();
                let card_bg = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
                    theme.colors.background.main,
                    crate::theme::opacity::OPACITY_GHOST,
                ));
                let text_rgb = gpui::rgb(theme.colors.text.primary);
                let hint_rgba = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
                    theme.colors.text.primary,
                    crate::theme::opacity::OPACITY_DISABLED,
                ));
                let _ = &entity; // prevent entity from being dropped

                let prompt_colors =
                    crate::theme::PromptColors::from_theme(&theme);

                if let Some(turn) = turns.get(ix) {
                    let title: SharedString = match turn.kind {
                        TabAiTurnKind::User => "You".into(),
                        TabAiTurnKind::AssistantText => "AI".into(),
                        TabAiTurnKind::AssistantCode => "Generated Script".into(),
                    };

                    let body_element: gpui::AnyElement =
                        if matches!(turn.kind, TabAiTurnKind::User) {
                            div()
                                .mt(px(4.))
                                .text_sm()
                                .text_color(text_rgb)
                                .child(turn.body.clone())
                                .into_any_element()
                        } else if turn.streaming
                            && turn.body.to_string().is_empty()
                        {
                            div()
                                .mt(px(4.))
                                .text_xs()
                                .opacity(0.6)
                                .child("Thinking\u{2026}")
                                .into_any_element()
                        } else {
                            let markdown_source = match turn.kind {
                                TabAiTurnKind::AssistantCode => {
                                    format!(
                                        "```ts\n{}\n```",
                                        turn.body.to_string()
                                    )
                                }
                                _ => turn.body.to_string(),
                            };
                            div()
                                .mt(px(4.))
                                .w_full()
                                .min_w_0()
                                .overflow_x_hidden()
                                .child(
                                    crate::prompts::markdown::render_markdown(
                                        markdown_source.as_str(),
                                        &prompt_colors,
                                    ),
                                )
                                .into_any_element()
                        };

                    div()
                        .w_full()
                        .mb(px(8.))
                        .px(px(12.))
                        .py(px(10.))
                        .bg(card_bg)
                        .child(
                            div()
                                .text_xs()
                                .text_color(hint_rgba)
                                .child(title),
                        )
                        .child(body_element)
                        .into_any_element()
                } else {
                    div().w_full().into_any_element()
                }
            })
            .with_sizing_behavior(ListSizingBehavior::Infer)
            .flex_1()
            .min_h(px(0.))
            .px(px(12.))
            .py(px(12.))
            .into_any_element()
        };

        div()
            .id("tab-ai-chat")
            .size_full()
            .flex()
            .flex_col()
            .bg(bg)
            .track_focus(&self.focus_handle)
            // Input row with cursor
            .child(
                div()
                    .id("tab-ai-chat-input")
                    .w_full()
                    .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
                    .py(px(12.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_size(gpui::rems(1.125))
                            .font_family(crate::list_item::FONT_MONO)
                            // Text before cursor
                            .when(!before.is_empty(), |d| {
                                d.child(div().text_color(text_color).child(before.clone()))
                            })
                            // Cursor bar
                            .child(
                                div()
                                    .w(px(2.))
                                    .h(px(18.))
                                    .when(is_focused && self.cursor_visible, |d| {
                                        d.bg(accent_color)
                                    }),
                            )
                            // Placeholder or text after cursor
                            .when(input_text.is_empty(), |d| {
                                d.child(div().text_color(hint).child(placeholder.clone()))
                            })
                            .when(!after.is_empty(), |d| {
                                d.child(div().text_color(text_color).child(after.clone()))
                            }),
                    ),
            )
            // Hairline divider
            .child(div().w_full().h(px(1.)).bg(divider))
            // Error message
            .when_some(self.error.clone(), |d, message| {
                d.child(
                    div()
                        .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
                        .py(px(6.))
                        .text_xs()
                        .text_color(error_color)
                        .child(message),
                )
            })
            // Memory hint
            .when_some(memory_hint_element, |d, el| d.child(el))
            // Scrollable body
            .child(body)
            // Bottom divider
            .child(div().w_full().h(px(1.)).bg(divider))
            // Footer hint strip
            .child(components::HintStrip::new(vec![
                "\u{21B5} Send".into(),
                "\u{2318}K Actions".into(),
                "Esc Back".into(),
            ]))
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
    fn tab_ai_chat_uses_three_key_footer_contract() {
        const TAB_AI_SOURCE: &str = include_str!("tab_ai_mode.rs");
        assert!(
            TAB_AI_SOURCE.contains(r#""\u{21B5} Send"#),
            "tab ai chat should expose the Send hint"
        );
        assert!(
            TAB_AI_SOURCE.contains(r#""\u{2318}K Actions"#),
            "tab ai chat should expose the Actions hint"
        );
        assert!(
            TAB_AI_SOURCE.contains(r#""Esc Back"#),
            "tab ai chat should expose the Esc Back hint"
        );
    }

    #[test]
    fn tab_ai_overlay_preserves_memory_hint_rendering() {
        const TAB_AI_SOURCE: &str = include_str!("tab_ai_mode.rs");
        assert!(
            TAB_AI_SOURCE.contains("Similar prior automation:"),
            "visual cleanup must not silently remove memory-hint behavior"
        );
    }

    #[test]
    fn tab_ai_overlay_uses_named_opacity_constants() {
        const TAB_AI_SOURCE: &str = include_str!("tab_ai_mode.rs");
        // The render function should reference OPACITY_GHOST, not raw 0.06
        assert!(
            TAB_AI_SOURCE.contains("OPACITY_GHOST"),
            "tab ai overlay should use named ghost opacity constant"
        );
    }

    #[test]
    fn tab_ai_overlay_uses_shared_hint_strip_component() {
        const TAB_AI_SOURCE: &str = include_str!("tab_ai_mode.rs");
        assert!(
            TAB_AI_SOURCE.contains("HintStrip::new"),
            "tab ai overlay should use the shared HintStrip component"
        );
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
