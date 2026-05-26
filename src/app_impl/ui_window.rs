use super::*;
use std::sync::Once;

static MAIN_FOOTER_ACTION_LISTENER: Once = Once::new();

/// Thin wrapper delegating to the canonical implementation in `window_resize`.
fn mini_main_window_sizing_from_grouped_items(
    grouped_items: &[GroupedListItem],
) -> crate::window_resize::MiniMainWindowSizing {
    crate::window_resize::mini_main_window_sizing_from_grouped_items(grouped_items)
}

pub(crate) fn compact_ai_view_type_for_mode(mode: MainWindowMode) -> ViewType {
    match mode {
        MainWindowMode::Mini => ViewType::MiniAiChat,
        MainWindowMode::Full => ViewType::DivPrompt,
    }
}

pub(crate) fn mini_prompt_view_type() -> ViewType {
    ViewType::MiniPrompt
}

fn footer_frontmost_app_name() -> Option<String> {
    crate::frontmost_app_tracker::get_last_real_app().and_then(|app| {
        let trimmed = app.name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn paste_into_frontmost_app_label(frontmost_app_name: Option<&str>) -> String {
    match frontmost_app_name
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        Some(app_name) => format!("Paste into {app_name}"),
        None => "Paste into Active App".to_string(),
    }
}

fn main_window_result_action_label(
    result: &crate::scripts::SearchResult,
    frontmost_app_name: Option<&str>,
) -> String {
    match result {
        crate::scripts::SearchResult::Scriptlet(sm)
            if matches!(sm.scriptlet.tool.as_str(), "paste" | "snippet") =>
        {
            paste_into_frontmost_app_label(frontmost_app_name)
        }
        _ => result.get_default_action_text().to_string(),
    }
}

fn has_selected_clipboard_entry(app: &ScriptListApp) -> bool {
    let AppView::ClipboardHistoryView {
        filter,
        selected_index,
    } = &app.current_view
    else {
        return false;
    };

    let filtered_entries: Vec<_> = if filter.is_empty() {
        app.cached_clipboard_entries.iter().collect()
    } else {
        let filter_lower = filter.to_lowercase();
        app.cached_clipboard_entries
            .iter()
            .filter(|entry| {
                entry.text_preview.to_lowercase().contains(&filter_lower)
                    || entry
                        .ocr_text
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&filter_lower)
            })
            .collect()
    };

    filtered_entries.get(*selected_index).is_some()
}

fn has_selected_emoji_entry(app: &ScriptListApp) -> bool {
    let AppView::EmojiPickerView {
        filter,
        selected_index,
        selected_category,
    } = &app.current_view
    else {
        return false;
    };

    crate::emoji::filtered_ordered_emojis(filter, *selected_category)
        .get(*selected_index)
        .is_some()
}

fn has_selected_dictation_history_entry(app: &ScriptListApp) -> bool {
    let AppView::DictationHistoryView {
        filter,
        selected_index,
    } = &app.current_view
    else {
        return false;
    };

    crate::dictation::search_history(filter, 100)
        .get(*selected_index)
        .is_some()
}

impl ScriptListApp {
    pub(crate) fn main_window_primary_action_label(&self) -> String {
        let frontmost_app_name = footer_frontmost_app_name();

        match &self.current_view {
            AppView::ClipboardHistoryView { .. } => {
                return if has_selected_clipboard_entry(self) {
                    paste_into_frontmost_app_label(frontmost_app_name.as_deref())
                } else {
                    "Run".to_string()
                };
            }
            AppView::EmojiPickerView { .. } => {
                return if has_selected_emoji_entry(self) {
                    paste_into_frontmost_app_label(frontmost_app_name.as_deref())
                } else {
                    "Run".to_string()
                };
            }
            AppView::DictationHistoryView { .. } => {
                return if has_selected_dictation_history_entry(self) {
                    paste_into_frontmost_app_label(frontmost_app_name.as_deref())
                } else {
                    "Run".to_string()
                };
            }
            AppView::ThemeChooserView { .. } => {
                return "Apply".to_string();
            }
            AppView::ScriptList => {}
            _ => return "Run".to_string(),
        }

        let Some(selected_index) = crate::list_item::coerce_selection(
            &self.main_menu_result_caches.grouped_items(),
            self.selected_index,
        ) else {
            return "Run".to_string();
        };

        let Some(result_idx) = self
            .main_menu_result_caches
            .flat_result_index_for_grouped_item(selected_index)
        else {
            return "Run".to_string();
        };

        if self
            .inline_calculator_for_result_index(result_idx)
            .is_some()
        {
            return "Copy".to_string();
        }

        self.main_menu_result_caches
            .search_result_for_flat_index(result_idx)
            .map(|result| main_window_result_action_label(result, frontmost_app_name.as_deref()))
            .unwrap_or_else(|| "Run".to_string())
    }

    pub(crate) fn dispatch_main_window_footer_action(
        &mut self,
        action: crate::footer_popup::FooterAction,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
        source: &'static str,
    ) {
        tracing::info!(
            target: "script_kit::footer_popup",
            event = "main_window_footer_action_dispatch",
            source,
            action = ?action,
            view = ?self.current_view,
            main_window_mode = ?self.main_window_mode,
            "Dispatching main-window footer action"
        );

        let shared_actions_open = self.show_actions_popup;
        let detached_actions_open = crate::actions::is_actions_window_open();
        if (shared_actions_open || detached_actions_open) && !action.is_actions() {
            let mut closed = false;
            if let super::actions_dialog::ActionsSupport::SharedDialog(host) =
                self.actions_support_for_view()
            {
                self.close_actions_popup(host, window, cx);
                closed = true;
            }
            if detached_actions_open {
                crate::actions::close_actions_window(cx);
                closed = true;
            }
            if shared_actions_open || detached_actions_open {
                tracing::info!(
                    target: "script_kit::footer_popup",
                    event = "main_window_footer_action_closed_actions_only",
                    source,
                    action = ?action,
                    main_window_mode = ?self.main_window_mode,
                    closed,
                    "Closed actions dialog from footer outside-click target without dispatching action"
                );
            }
            return;
        }

        match action {
            crate::footer_popup::FooterAction::Run => {
                if let AppView::AcpChatView { entity } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |chat, cx| {
                        chat.submit_with_expanded_tokens(cx);
                    });
                    return;
                } else if let AppView::ScriptIssuesView { report } = &self.current_view {
                    let report = report.clone();
                    self.fix_script_issues_in_agent(&report, cx);
                    return;
                } else if self.dispatch_design_gallery_select_footer_action(cx) {
                    return;
                } else if self.dispatch_footer_gallery_select_footer_action(cx) {
                    return;
                } else if self.dispatch_kit_store_primary_footer_action(cx) {
                    return;
                } else if matches!(self.current_view, AppView::ThemeChooserView { .. }) {
                    self.submit_theme_chooser_from_input_enter(window, cx);
                    return;
                } else if let AppView::TemplatePrompt { entity, .. } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |prompt, cx| prompt.submit(cx));
                } else if matches!(self.current_view, AppView::WebcamView { .. }) {
                    if self.capture_webcam_photo(cx) {
                        self.hide_main_and_reset(cx);
                    }
                } else if let AppView::PathPrompt { entity, .. } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |prompt, cx| prompt.handle_enter(cx));
                } else if let AppView::EnvPrompt { entity, .. } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |prompt, cx| prompt.submit(cx));
                } else if let AppView::DropPrompt { entity, .. } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |prompt, _cx| prompt.submit());
                } else if !self.try_run_ready_acp_script(cx) {
                    self.execute_selected(cx);
                }
            }
            crate::footer_popup::FooterAction::Actions => {
                let handled = self.dispatch_actions_toggle_for_current_view(window, cx, source);
                tracing::info!(
                    target: "script_kit::footer_popup",
                    event = "main_window_footer_actions_routed",
                    source,
                    handled,
                    selected_index = self.selected_index,
                    show_actions_popup = self.show_actions_popup,
                    actions_window_open = crate::actions::is_actions_window_open(),
                    "Routed footer Actions through shared dispatcher"
                );
            }
            crate::footer_popup::FooterAction::Ai => {
                if let AppView::AcpChatView { entity } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |chat, cx| {
                        chat.open_profile_trigger_picker_in_window(window, cx);
                    });
                } else if let AppView::TemplatePrompt { entity, .. } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |prompt, cx| prompt.next_input(cx));
                } else {
                    self.open_tab_ai_acp_with_entry_intent(None, cx);
                }
            }
            crate::footer_popup::FooterAction::Stop => {
                if let AppView::AcpChatView { entity } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |chat, cx| {
                        let _ = chat.cancel_streaming_from_escape(cx);
                    });
                } else {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "main_window_footer_stop_ignored",
                        view = ?self.current_view,
                        "Ignored Stop footer action outside ACP chat"
                    );
                }
            }
            crate::footer_popup::FooterAction::PasteResponse => {
                self.paste_latest_acp_response_to_frontmost(None, cx);
            }
            crate::footer_popup::FooterAction::Replace
            | crate::footer_popup::FooterAction::Append
            | crate::footer_popup::FooterAction::Copy
            | crate::footer_popup::FooterAction::Expand
            | crate::footer_popup::FooterAction::Retry => {
                if let AppView::AcpChatView { entity } = &self.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |chat, cx| {
                        chat.dispatch_footer_button(action, window, cx);
                    });
                }
            }
            crate::footer_popup::FooterAction::Apply => {
                if let AppView::ScriptIssuesView { report } = &self.current_view {
                    let report = report.clone();
                    self.copy_script_issues_to_clipboard(&report, cx);
                    return;
                } else if self.dispatch_kit_store_remove_footer_action(cx) {
                    return;
                } else if matches!(self.current_view, AppView::ConfirmPrompt { .. }) {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "confirm_prompt_footer_apply",
                        "Confirming in-window confirm prompt from native footer"
                    );
                    self.resolve_confirm_prompt(true, window, cx);
                } else if let AppView::QuickTerminalView { entity } = &self.current_view {
                    let entity = entity.clone();
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "quick_terminal_footer_apply",
                        "Applying quick-terminal result from native footer"
                    );
                    self.apply_tab_ai_result_from_terminal(entity, cx);
                } else {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "main_window_footer_apply_ignored",
                        view = ?self.current_view,
                        "Ignored Apply footer action outside QuickTerminalView"
                    );
                }
            }
            crate::footer_popup::FooterAction::Close => {
                if self.dispatch_kit_store_browse_back_footer_action(window, cx) {
                    return;
                } else if matches!(self.current_view, AppView::ConfirmPrompt { .. }) {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "confirm_prompt_footer_close",
                        "Cancelling in-window confirm prompt from native footer"
                    );
                    self.resolve_confirm_prompt(false, window, cx);
                } else if matches!(self.current_view, AppView::QuickTerminalView { .. }) {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "quick_terminal_footer_close",
                        "Closing quick terminal from native footer"
                    );
                    self.close_quick_terminal_main_window_state_first(cx);
                } else if let AppView::HotkeyPrompt { id, .. } = &self.current_view {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "hotkey_prompt_footer_cancel",
                        "Cancelling hotkey prompt from native footer"
                    );
                    self.submit_prompt_response(id.clone(), None, cx);
                    self.cancel_script_execution(cx);
                } else {
                    tracing::info!(
                        target: "script_kit::footer_popup",
                        event = "main_window_footer_close_ignored",
                        view = ?self.current_view,
                        "Ignored Close footer action outside QuickTerminalView"
                    );
                }
            }
        }
    }

    /// If the current view is an ACP chat with a validated `SCRIPT_READY` receipt,
    /// execute that specific script and return `true`. Otherwise return `false`
    /// so the caller can fall back to `execute_selected`.
    fn try_run_ready_acp_script(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(self.current_view, AppView::AcpChatView { .. }) {
            return false;
        }
        let Some(path) = self.acp_ready_script_path.clone() else {
            return false;
        };
        let path_str = path.to_string_lossy().to_string();
        tracing::info!(
            target: "script_kit::footer_popup",
            event = "acp_footer_run_dispatched",
            path = %path_str,
        );
        self.execute_script_by_path(&path_str, cx);
        true
    }

    /// Paste assistant output into the frontmost app. When `text_override` is
    /// `Some`, that text is pasted directly. Otherwise the current ACP view
    /// resolves pastable text (selected focused-text variation when present,
    /// else the latest assistant message).
    pub(crate) fn paste_latest_acp_response_to_frontmost(
        &mut self,
        text_override: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(text) = text_override.or_else(|| self.latest_acp_assistant_response(cx)) else {
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "acp_footer_paste_response_ignored",
                "Ignored Paste Response footer action because no assistant response exists"
            );
            return;
        };

        crate::platform::defer_hide_main_window(cx);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let injector = crate::text_injector::TextInjector::new();
            if let Err(error) = injector.paste_text(&text) {
                tracing::warn!(
                    target: "script_kit::footer_popup",
                    event = "acp_footer_paste_response_failed",
                    %error,
                    "Failed to paste ACP response into frontmost app"
                );
            }
        });

        tracing::info!(
            target: "script_kit::footer_popup",
            event = "acp_footer_paste_response_dispatched",
            "Dispatched latest ACP assistant response to frontmost app"
        );
    }

    fn latest_acp_assistant_response(&self, cx: &App) -> Option<String> {
        let AppView::AcpChatView { entity } = &self.current_view else {
            return None;
        };

        entity.read(cx).pastable_response_text(cx)
    }

    fn dispatch_design_gallery_select_footer_action(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(self.current_view, AppView::DesignGalleryView { .. }) {
            return false;
        }

        tracing::info!(
            target: "script_kit::footer_popup",
            event = "design_gallery_footer_select_ignored",
            "Design Gallery native footer Select preserves current no-op selection behavior"
        );
        cx.notify();
        true
    }

    fn dispatch_footer_gallery_select_footer_action(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(self.current_view, AppView::FooterGalleryView { .. }) {
            return false;
        }

        tracing::info!(
            target: "script_kit::footer_popup",
            event = "footer_gallery_footer_select_ignored",
            "Footer Gallery native footer Select preserves current no-op selection behavior"
        );
        cx.notify();
        true
    }

    /// Start a one-time async bridge that drains `footer_action_channel()` and
    /// dispatches each action into the existing `ScriptListApp` methods.
    fn ensure_main_footer_action_listener(&self, window: &Window, cx: &mut Context<Self>) {
        MAIN_FOOTER_ACTION_LISTENER.call_once(|| {
            let rx = crate::footer_popup::footer_action_channel().1.clone();
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "native_footer_listener_started",
                "Started native footer action listener"
            );
            cx.spawn_in(window, async move |this, cx| {
                while let Ok(action) = rx.recv().await {
                    if let Err(error) = this.update_in(cx, |app, window, cx| {
                        app.handle_main_footer_action(action, window, cx);
                    }) {
                        tracing::warn!(
                            target: "script_kit::footer_popup",
                            event = "native_footer_action_dispatch_failed",
                            action = ?action,
                            %error,
                            "Failed to dispatch native footer action into ScriptListApp"
                        );
                    }
                }
            })
            .detach();
        });
    }

    fn standard_main_window_footer_buttons(&self) -> Vec<crate::footer_popup::FooterButtonConfig> {
        use crate::footer_popup::{FooterAction, FooterButtonConfig};

        let footer_disabled = self.main_window_footer_buttons_blocked();
        let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
        let run_label = self.main_window_primary_action_label();

        let mut buttons = vec![
            FooterButtonConfig::new(FooterAction::Run, "↵", run_label).enabled(!footer_disabled),
            FooterButtonConfig::new(
                FooterAction::Ai,
                "⌘↵",
                crate::ai::acp::labels::AGENT_CHAT_LABEL,
            )
            .enabled(!footer_disabled),
        ];

        if self.current_view_supports_shared_actions() {
            buttons.push(
                FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                    .selected(actions_open)
                    .enabled(!footer_disabled),
            );
        }
        buttons
    }

    fn main_window_footer_buttons_blocked(&self) -> bool {
        crate::confirm::is_confirm_window_open()
    }

    fn main_window_footer_surface(&self) -> Option<&'static str> {
        self.current_view.native_footer_surface()
    }

    /// Quick Terminal footer buttons. Scoped to actions actually meaningful in
    /// the Quick Terminal surface: always Close (⌘W), plus Apply (⌘↩) only
    /// when a tab-AI apply-back route AND its return view are both present.
    /// Run/AI/Actions are intentionally omitted — Quick Terminal shares the
    /// main menu's native footer chrome but not its main-menu-specific actions.
    pub(crate) fn quick_terminal_can_apply_back(&self) -> bool {
        self.tab_ai_harness_apply_back_route.is_some() && self.tab_ai_harness_return_view.is_some()
    }

    fn quick_terminal_footer_buttons(&self) -> Vec<crate::footer_popup::FooterButtonConfig> {
        use crate::footer_popup::{FooterAction, FooterButtonConfig};

        let footer_disabled = self.main_window_footer_buttons_blocked();
        let enabled = !footer_disabled;
        let can_apply = self.quick_terminal_can_apply_back();

        let mut buttons = Vec::with_capacity(if can_apply { 2 } else { 1 });
        if can_apply {
            buttons
                .push(FooterButtonConfig::new(FooterAction::Apply, "⌘↩", "Apply").enabled(enabled));
        }
        buttons.push(FooterButtonConfig::new(FooterAction::Close, "⌘W", "Close").enabled(enabled));

        tracing::info!(
            target: "script_kit::footer_popup",
            event = "quick_terminal_footer_buttons_resolved",
            can_apply,
            footer_disabled,
            button_count = buttons.len(),
            "Resolved quick-terminal native footer buttons"
        );

        buttons
    }

    /// Footer buttons for an in-window `ConfirmPrompt`. Reuses the native
    /// Apply/Close slots so no AppKit ObjC selector wiring needs to change —
    /// only the labels and `selected` flag change per options + focused button.
    fn confirm_prompt_footer_buttons(
        &self,
        options: &crate::confirm::ParentConfirmOptions,
        focused_button: ConfirmFocusedButton,
    ) -> Vec<crate::footer_popup::FooterButtonConfig> {
        use crate::footer_popup::{FooterAction, FooterButtonConfig};

        let confirm_focused = matches!(focused_button, ConfirmFocusedButton::Confirm);
        let cancel_focused = matches!(focused_button, ConfirmFocusedButton::Cancel);

        vec![
            FooterButtonConfig::new(FooterAction::Apply, "↵", options.confirm_text.to_string())
                .selected(confirm_focused)
                .enabled(true),
            FooterButtonConfig::new(FooterAction::Close, "Esc", options.cancel_text.to_string())
                .selected(cancel_focused)
                .enabled(true),
        ]
    }

    fn main_window_footer_buttons_for_current_view(
        &self,
        cx: Option<&gpui::App>,
    ) -> Vec<crate::footer_popup::FooterButtonConfig> {
        // ConfirmPrompt: Apply (Confirm) + Close (Cancel) labeled per options.
        if let AppView::ConfirmPrompt {
            options,
            focused_button,
            ..
        } = &self.current_view
        {
            let buttons = self.confirm_prompt_footer_buttons(options, *focused_button);
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved ConfirmPrompt footer buttons"
            );
            return buttons;
        }

        // Quick Terminal: scoped Close (+ optional Apply) — never Run/AI/Actions.
        if matches!(self.current_view, AppView::QuickTerminalView { .. }) {
            let buttons = self.quick_terminal_footer_buttons();
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved Quick Terminal footer buttons"
            );
            return buttons;
        }

        // ACP owns its own footer state: Send/Paste Response/Stop + Actions.
        if matches!(self.current_view, AppView::AcpChatView { .. }) {
            let buttons = self.acp_footer_buttons();
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved ACP footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::TemplatePrompt { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
            let enabled = !footer_disabled;
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Run, "↵", "Submit").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Ai, "⇥", "Next Field").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                    .selected(actions_open)
                    .enabled(enabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved TemplatePrompt footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::HotkeyPrompt { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let enabled = !footer_disabled;
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Close, "Esc", "Cancel").enabled(enabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved HotkeyPrompt footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::ScriptIssuesView { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let enabled = !footer_disabled;
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Run, "↵", "Fix in Agent").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Apply, "⌘C", "Copy Issues").enabled(enabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved Script Issues footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::EnvPrompt { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let enabled = !footer_disabled;
            let buttons =
                vec![FooterButtonConfig::new(FooterAction::Run, "↵", "Submit").enabled(enabled)];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved EnvPrompt footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::WebcamView { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
            let enabled = !footer_disabled;
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Run, "↵", "Capture Photo").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                    .selected(actions_open)
                    .enabled(enabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved Webcam footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::PathPrompt { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
            let enabled = !footer_disabled;
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Run, "↵", "Select").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                    .selected(actions_open)
                    .enabled(enabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved PathPrompt footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::DropPrompt { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
            let has_files = match (&self.current_view, cx) {
                (AppView::DropPrompt { entity, .. }, Some(cx)) => {
                    !entity.read(cx).dropped_files.is_empty()
                }
                _ => false,
            };
            let submit_button = if footer_disabled {
                FooterButtonConfig::new(FooterAction::Run, "↵", "Submit").enabled(false)
            } else if has_files {
                FooterButtonConfig::new(FooterAction::Run, "↵", "Submit").enabled(true)
            } else {
                FooterButtonConfig::new(FooterAction::Run, "↵", "Submit")
                    .disabled_reason("no_files")
            };
            let buttons = vec![
                submit_button,
                FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                    .selected(actions_open)
                    .enabled(!footer_disabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                has_files,
                "Resolved DropPrompt footer buttons"
            );
            return buttons;
        }

        if matches!(self.current_view, AppView::DesignGalleryView { .. }) {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let buttons =
                vec![FooterButtonConfig::new(FooterAction::Run, "↵", "Select")
                    .enabled(!footer_disabled)];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved Design Gallery footer buttons"
            );
            return buttons;
        }

        if let AppView::BrowseKitsView {
            selected_index,
            results,
            query,
            ..
        } = &self.current_view
        {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let enabled = !footer_disabled && results.get(*selected_index).is_some();
            let secondary_label = if query.is_empty() {
                "Back"
            } else {
                "Clear Search"
            };
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Run, "↵", "Install").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Close, "Esc", secondary_label)
                    .enabled(!footer_disabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved Kit Store browse footer buttons"
            );
            return buttons;
        }

        if let AppView::InstalledKitsView {
            selected_index,
            kits,
            ..
        } = &self.current_view
        {
            use crate::footer_popup::{FooterAction, FooterButtonConfig};

            let footer_disabled = self.main_window_footer_buttons_blocked();
            let enabled = !footer_disabled && kits.get(*selected_index).is_some();
            let buttons = vec![
                FooterButtonConfig::new(FooterAction::Run, "↵", "Update").enabled(enabled),
                FooterButtonConfig::new(FooterAction::Apply, "⌫", "Remove").enabled(enabled),
            ];
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                "Resolved Kit Store installed footer buttons"
            );
            return buttons;
        }

        let buttons = self.standard_main_window_footer_buttons();
        tracing::info!(
            target: "script_kit::footer_popup",
            event = "main_window_footer_buttons_resolved",
            view = ?self.current_view,
            button_count = buttons.len(),
            "Resolved main-window native footer buttons"
        );
        buttons
    }

    /// Build footer buttons for the ACP chat surface from the child-owned
    /// composer/thread state snapshot.
    fn acp_footer_buttons(&self) -> Vec<crate::footer_popup::FooterButtonConfig> {
        use crate::footer_popup::{FooterAction, FooterButtonConfig};

        let footer_disabled = self.main_window_footer_buttons_blocked();
        let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
        let enabled = !footer_disabled;

        if let Some(snapshot) = self.acp_footer_snapshot.as_ref() {
            if !snapshot.visible {
                return Vec::new();
            }
            return snapshot
                .buttons
                .iter()
                .map(|button| {
                    let mut config =
                        FooterButtonConfig::new(button.action, button.key, button.label)
                            .selected(button.selected)
                            .enabled(enabled && button.enabled);
                    if let Some(reason) = button.disabled_reason {
                        config = config.disabled_reason(reason);
                    }
                    config
                })
                .collect();
        }

        vec![
            FooterButtonConfig::new(FooterAction::Run, "↵", "Send").disabled_reason("loading_acp"),
            FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                .selected(actions_open)
                .enabled(enabled),
        ]
    }

    pub(crate) fn main_window_footer_config(
        &self,
    ) -> Option<crate::footer_popup::MainWindowFooterConfig> {
        self.main_window_footer_config_with_cx(None)
    }

    pub(crate) fn main_window_footer_config_with_cx(
        &self,
        cx: Option<&gpui::App>,
    ) -> Option<crate::footer_popup::MainWindowFooterConfig> {
        use crate::footer_popup::MainWindowFooterConfig;

        if let AppView::AcpChatView { entity } = &self.current_view {
            let hidden_by_live_view = cx
                .map(|cx| !entity.read(cx).main_window_footer_visible(cx))
                .unwrap_or(false);
            let hidden_by_cached_snapshot = cx.is_none()
                && self
                    .acp_footer_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| !snapshot.visible);
            if hidden_by_live_view || hidden_by_cached_snapshot {
                return None;
            }
        }

        let surface = self.main_window_footer_surface()?;
        let buttons = self.main_window_footer_buttons_for_current_view(cx);

        tracing::info!(
            target: "script_kit::footer_popup",
            event = "main_window_footer_config_resolved",
            view = ?self.current_view,
            surface,
            button_count = buttons.len(),
            "Resolved main-window native footer config"
        );

        Some(MainWindowFooterConfig::new(surface, buttons))
    }

    pub(crate) fn main_window_uses_native_footer(&self) -> bool {
        crate::is_main_window_visible()
            && self
                .main_window_footer_surface()
                .is_some_and(|expected_surface| {
                    crate::footer_popup::active_main_window_footer_surface()
                        == Some(expected_surface)
                })
    }

    /// When the native main-window footer is active, replace the GPUI footer
    /// with a transparent spacer so content stays clear of the AppKit footer.
    pub(crate) fn main_window_footer_slot(
        &self,
        gpui_footer: gpui::AnyElement,
    ) -> Option<gpui::AnyElement> {
        if matches!(self.current_view, AppView::AcpChatView { .. })
            && self
                .acp_footer_snapshot
                .as_ref()
                .is_some_and(|snapshot| !snapshot.visible)
        {
            return None;
        }
        if self.main_window_uses_native_footer() {
            Some(crate::components::prompt_layout_shell::render_native_main_window_footer_spacer())
        } else {
            Some(gpui_footer)
        }
    }

    fn handle_main_footer_action(
        &mut self,
        action: crate::footer_popup::FooterAction,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::footer_popup",
            event = "main_window_footer_action_dispatch",
            source = "native_footer",
            action = ?action,
            view = ?self.current_view,
            main_window_mode = ?self.main_window_mode,
            "Dispatching main-window footer action"
        );

        if self.main_window_footer_config_with_cx(Some(&*cx)).is_none()
            || !crate::is_main_window_visible()
        {
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_action_ignored_inactive_surface",
                source = "native_footer",
                action = ?action,
                view = ?self.current_view,
                main_window_mode = ?self.main_window_mode,
                "Ignored native footer action because current view is not using the native footer"
            );
            return;
        }

        self.dispatch_main_window_footer_action(action, window, cx, "native_footer");
    }

    pub(crate) fn sync_main_window_resize_lock(
        &self,
        window: &mut gpui::Window,
        cx: &Context<Self>,
    ) {
        let should_lock = match &self.current_view {
            AppView::AcpChatView { entity } => {
                entity.read_with(cx, |view, _cx| view.locks_main_window_resize())
            }
            _ => false,
        };
        crate::platform::set_window_resizable(window, !should_lock);
    }

    pub(crate) fn sync_main_footer_popup(&self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.ensure_main_footer_action_listener(window, cx);

        let mut config = if crate::is_main_window_visible() {
            self.main_window_footer_config_with_cx(Some(&*cx))
        } else {
            None
        };

        // Enrich with ACP streaming/model info when on the ACP chat view.
        if let Some(ref mut cfg) = config {
            self.enrich_footer_config_with_acp_info(cfg);
        }

        tracing::info!(
            target: "script_kit::footer_popup",
            event = "main_window_footer_sync",
            view = ?self.current_view,
            show = config.is_some(),
            surface = config.as_ref().map(|c| c.surface).unwrap_or("none"),
            button_count = config.as_ref().map(|c| c.buttons.len()).unwrap_or(0),
            "Syncing native main window footer"
        );

        crate::footer_popup::sync_main_footer_popup(window, config.as_ref(), &mut *cx);
    }

    pub(crate) fn enrich_footer_config_with_acp_info(
        &self,
        config: &mut crate::footer_popup::MainWindowFooterConfig,
    ) {
        if matches!(self.current_view, AppView::AcpChatView { .. }) {
            if let Some(snapshot) = self.acp_footer_snapshot.as_ref() {
                config.left_info = Some(snapshot.profile_left_info());
                return;
            }

            let (Some(dot_status), Some(model_name)) = (
                self.acp_footer_dot_status,
                self.acp_footer_model_display.as_ref(),
            ) else {
                return;
            };

            config.left_info = Some(crate::footer_popup::FooterLeftInfo {
                dot_status,
                model_name: model_name.clone(),
                prefer_accent_for_active_states: true,
                profile_name: None,
                icon_token: Some(
                    crate::components::footer_chrome::FOOTER_PROFILE_ICON_TOKEN.to_string(),
                ),
                action: Some(crate::footer_popup::FooterAction::Ai),
                selected: false,
            });
        }
    }

    pub(crate) fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.show_logs = !self.show_logs;
        cx.notify();
    }

    /// Toggle the focused-info panel visibility (Cmd+I / "Show Info" action).
    pub(crate) fn toggle_info_panel(&mut self, cx: &mut Context<Self>) {
        self.show_info_panel = !self.show_info_panel;
        tracing::info!(
            category = "UI",
            event = "toggle_info_panel",
            visible = self.show_info_panel,
            "Info panel toggled"
        );
        cx.notify();
    }

    /// Hide the mouse cursor while typing.
    /// The cursor will be shown again when the mouse moves.
    pub(crate) fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = true;
            crate::platform::hide_cursor_until_mouse_moves();
            cx.notify();
        }
    }

    /// Show the mouse cursor (called when mouse moves).
    /// Also switches to Mouse input mode to re-enable hover effects.
    /// Only calls cx.notify() when state actually changes, to avoid
    /// render churn during passive scrolling.
    pub(crate) fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        let mut changed = false;

        if !matches!(self.input_mode, InputMode::Mouse) {
            self.input_mode = InputMode::Mouse;
            changed = true;
        }

        if self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = false;
            changed = true;
        }

        if changed {
            cx.notify();
        }
    }

    /// Calculate view type and item count for window sizing.
    /// Extracted from update_window_size for reuse.
    pub(crate) fn calculate_window_size_params(&mut self) -> Option<(ViewType, usize)> {
        self.calculate_window_size_params_with_app(None)
    }

    pub(crate) fn calculate_window_size_params_with_app(
        &mut self,
        cx: Option<&gpui::App>,
    ) -> Option<(ViewType, usize)> {
        match &self.current_view {
            AppView::ScriptList => {
                // Get grouped results which includes section headers (cached)
                let (grouped_items, _) = self.get_grouped_results_cached();
                let count = grouped_items.len();
                let view_type = match self.main_window_mode {
                    MainWindowMode::Full => ViewType::ScriptList,
                    MainWindowMode::Mini => ViewType::MiniMainWindow,
                };
                Some((view_type, count))
            }
            AppView::About { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::ArgPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    Some((ViewType::ArgPromptNoChoices, 0))
                } else {
                    Some((ViewType::ArgPromptWithChoices, filtered.len()))
                }
            }
            AppView::MiniPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    Some((mini_prompt_view_type(), 0))
                } else {
                    Some((mini_prompt_view_type(), filtered.len().min(5)))
                }
            }
            AppView::MicroPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::DivPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FormPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::SelectPrompt { .. } => Some((ViewType::ArgPromptWithChoices, 0)),
            AppView::PathPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::EnvPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::DropPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => Some((ViewType::DivPrompt, 0)), // Template prompt uses div size
            AppView::HotkeyPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Hotkey prompt uses compact recorder surface
            AppView::ChatPrompt { .. } => {
                Some((compact_ai_view_type_for_mode(self.main_window_mode), 0))
            }
            AppView::TermPrompt { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                None
            }
            // Preview/detail builtins widen from the mini launcher without
            // increasing height, so the shared header/input stays fixed.
            // View state only - data comes from self fields
            AppView::ClipboardHistoryView { filter, .. } => {
                let entries = &self.cached_clipboard_entries;
                let filtered_count = if filter.is_empty() {
                    entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries
                        .iter()
                        .filter(|e| e.text_preview.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::ExpandedMainWindow, filtered_count))
            }
            AppView::EmojiPickerView {
                filter,
                selected_category,
                ..
            } => {
                let row_count = crate::emoji::filtered_grid_row_count(filter, *selected_category);
                Some((ViewType::MiniMainWindow, row_count))
            }
            AppView::AppLauncherView { filter, .. } => {
                let apps = &self.apps;
                let filtered_count = if filter.is_empty() {
                    apps.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    apps.iter()
                        .filter(|a| a.name.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::MiniMainWindow, filtered_count))
            }
            AppView::WindowSwitcherView { filter, .. } => {
                let windows = &self.cached_windows;
                let filtered_count = if filter.is_empty() {
                    windows.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    windows
                        .iter()
                        .filter(|w| {
                            w.title.to_lowercase().contains(&filter_lower)
                                || w.app.to_lowercase().contains(&filter_lower)
                        })
                        .count()
                };
                Some((ViewType::MiniMainWindow, filtered_count))
            }
            AppView::DesignGalleryView { filter, .. } => Some((
                ViewType::MiniMainWindow,
                crate::design_gallery_filtered_len(filter),
            )),
            AppView::FooterGalleryView { filter, .. } => Some((
                ViewType::MiniMainWindow,
                crate::footer_gallery_filtered_len(filter),
            )),
            AppView::NonListStatesView { .. } => Some((ViewType::DivPrompt, 0)),
            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::ProcessManagerView { filter, .. } => {
                let filtered_count = if filter.is_empty() {
                    self.cached_processes.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    self.cached_processes
                        .iter()
                        .filter(|p| p.script_path.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::MiniMainWindow, filtered_count))
            }
            AppView::CurrentAppCommandsView { filter, .. } => {
                let filtered_count = if filter.is_empty() {
                    self.cached_current_app_entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    self.cached_current_app_entries
                        .iter()
                        .filter(|e| {
                            e.name.to_lowercase().contains(&filter_lower)
                                || e.keywords.iter().any(|k| k.contains(&filter_lower))
                        })
                        .count()
                };
                Some((ViewType::MiniMainWindow, filtered_count))
            }
            AppView::BrowserTabsView { filter, .. } => {
                let filtered_count = if filter.is_empty() {
                    self.cached_browser_tabs.len()
                } else {
                    crate::browser_tabs::fuzzy_search_browser_tabs(
                        &self.cached_browser_tabs,
                        filter,
                    )
                    .len()
                };
                Some((ViewType::MiniMainWindow, filtered_count))
            }
            AppView::ScratchPadView { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::QuickTerminalView { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::WebcamView { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FileSearchView {
                ref query,
                presentation,
                ..
            } => {
                let results = &self.cached_file_results;
                let filtered_count = if query.is_empty() {
                    results.len()
                } else {
                    let query_lower = query.to_lowercase();
                    results
                        .iter()
                        .filter(|r| r.name.to_lowercase().contains(&query_lower))
                        .count()
                };
                let view_type = match presentation {
                    FileSearchPresentation::Mini => ViewType::MiniMainWindow,
                    FileSearchPresentation::Full => ViewType::ExpandedMainWindow,
                };
                Some((view_type, filtered_count))
            }
            AppView::ThemeChooserView { ref filter, .. } => {
                let presets = theme::presets::presets_cached();
                let filtered_count = if filter.is_empty() {
                    presets.len()
                } else {
                    let f = filter.to_lowercase();
                    presets
                        .iter()
                        .filter(|p| {
                            p.name.to_lowercase().contains(&f)
                                || p.description.to_lowercase().contains(&f)
                        })
                        .count()
                };
                Some((ViewType::ExpandedMainWindow, filtered_count))
            }
            AppView::CreationFeedback { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::ScriptIssuesView { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::SdkReferenceView {
                entries, filter, ..
            } => {
                let (_, count) =
                    crate::mcp_resources::sdk_reference_dataset_and_visible_counts(entries, filter);
                Some((ViewType::ExpandedMainWindow, count))
            }
            AppView::ScriptTemplateCatalogView {
                templates, filter, ..
            } => {
                let (_, count) =
                    crate::mcp_resources::script_template_catalog_dataset_and_visible_counts(
                        templates, filter,
                    );
                Some((ViewType::ExpandedMainWindow, count))
            }
            AppView::NamingPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::BrowseKitsView { results, .. } => {
                Some((ViewType::MiniMainWindow, results.len()))
            }
            AppView::InstalledKitsView { kits, .. } => Some((ViewType::MiniMainWindow, kits.len())),
            AppView::SearchAiPresetsView { .. } => {
                // Presets list - defaults (5) + user presets
                let count = crate::ai::presets::load_presets()
                    .map(|p| 5 + p.len())
                    .unwrap_or(5);
                Some((ViewType::MiniMainWindow, count))
            }
            AppView::CreateAiPresetView { .. } => {
                // Fixed-size form with 3 fields
                Some((ViewType::ArgPromptNoChoices, 0))
            }
            AppView::SettingsView { .. } => Some((ViewType::MiniMainWindow, 0)),
            AppView::FavoritesBrowseView { .. } => Some((ViewType::MiniMainWindow, 0)),
            AppView::AcpHistoryView { filter, .. } => {
                let entries = crate::ai::acp::history::load_history();
                let filtered_count = if filter.is_empty() {
                    entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries
                        .iter()
                        .filter(|entry| {
                            entry.first_message.to_lowercase().contains(&filter_lower)
                                || entry.timestamp.to_lowercase().contains(&filter_lower)
                        })
                        .count()
                };
                Some((ViewType::ExpandedMainWindow, filtered_count))
            }
            AppView::BrowserHistoryView { filter, .. } => Some((
                ViewType::ExpandedMainWindow,
                crate::browser_history::fuzzy_search_browser_history(
                    &self.cached_browser_history,
                    filter,
                )
                .len(),
            )),
            AppView::DictationHistoryView { filter, .. } => Some((
                ViewType::ExpandedMainWindow,
                crate::dictation::search_history(filter, 100).len(),
            )),
            AppView::NotesBrowseView { filter, .. } => Some((
                ViewType::ExpandedMainWindow,
                if filter.is_empty() {
                    crate::notes::get_all_notes()
                        .map(|notes| notes.len())
                        .unwrap_or(0)
                } else {
                    crate::notes::search_notes(filter)
                        .map(|notes| notes.len())
                        .unwrap_or(0)
                },
            )),
            AppView::AcpChatView { entity } => {
                if let Some(cx) = cx {
                    if let Some(item_count) = entity.read(cx).focused_text_mini_sizing_count(cx) {
                        return Some((ViewType::FocusedTextMini, item_count));
                    }
                }
                Some((compact_ai_view_type_for_mode(self.main_window_mode), 0))
            }
            AppView::ConfirmPrompt { .. } => Some((ViewType::DivPrompt, 0)),
        }
    }

    pub(crate) fn set_main_window_mode(
        &mut self,
        mode: MainWindowMode,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
        source: &'static str,
    ) {
        let old = self.main_window_mode;
        if old == mode {
            return;
        }

        self.main_window_mode = mode;

        if let AppView::ChatPrompt { entity, .. } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |chat, _cx| {
                chat.set_mini_mode(mode == MainWindowMode::Mini);
            });
        }

        let shared_actions_open = self.show_actions_popup;
        let detached_actions_open = crate::actions::is_actions_window_open();
        if shared_actions_open {
            if let super::actions_dialog::ActionsSupport::SharedDialog(host) =
                self.actions_support_for_view()
            {
                self.close_actions_popup(host, window, cx);
            } else {
                self.clear_actions_popup_state();
            }
        }
        if detached_actions_open {
            crate::actions::close_actions_window(cx);
        }

        self.update_window_size_deferred(window, cx);
        self.sync_main_footer_popup(window, cx);
        tracing::info!(
            target: "script_kit::window_mode",
            event = "main_window_mode_changed",
            source,
            old = ?old,
            new = ?mode,
            view = ?self.current_view,
            "Main window mode changed atomically"
        );
    }

    pub(crate) fn set_main_window_mode_state_only(
        &mut self,
        mode: MainWindowMode,
        cx: &mut Context<Self>,
        source: &'static str,
    ) {
        let old = self.main_window_mode;
        if old == mode {
            return;
        }

        self.main_window_mode = mode;
        if let AppView::ChatPrompt { entity, .. } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |chat, _cx| {
                chat.set_mini_mode(mode == MainWindowMode::Mini);
            });
        }
        tracing::info!(
            target: "script_kit::window_mode",
            event = "main_window_mode_changed",
            source,
            old = ?old,
            new = ?mode,
            view = ?self.current_view,
            "Main window mode changed without window handle"
        );
    }

    /// Calculate sizing only when the current view still matches the caller's
    /// expected async resize target.
    pub(crate) fn calculate_window_size_params_if_current_view(
        &mut self,
        reason: &'static str,
        is_expected_view: impl FnOnce(&AppView) -> bool,
    ) -> Option<(ViewType, usize)> {
        if !is_expected_view(&self.current_view) {
            tracing::debug!(
                target: "WINDOW_RESIZE",
                reason,
                current_view = ?self.current_view,
                "Skipping stale deferred resize for inactive view"
            );
            return None;
        }

        self.calculate_window_size_params()
    }

    /// Returns the focused button when the active view is `ConfirmPrompt`.
    pub(crate) fn confirm_prompt_focused_button(&self) -> Option<ConfirmFocusedButton> {
        if let AppView::ConfirmPrompt { focused_button, .. } = &self.current_view {
            Some(*focused_button)
        } else {
            None
        }
    }

    /// Flip Tab focus between Confirm and Cancel inside an active `ConfirmPrompt`.
    pub(crate) fn toggle_confirm_prompt_focus(&mut self, cx: &mut Context<Self>) {
        if let AppView::ConfirmPrompt { focused_button, .. } = &mut self.current_view {
            *focused_button = focused_button.toggled();
            cx.notify();
        }
    }

    /// Send the confirm/cancel result to the awaiting caller and restore the
    /// previous launcher view. No-op if the active view is not `ConfirmPrompt`.
    pub(crate) fn resolve_confirm_prompt(
        &mut self,
        confirmed: bool,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let restore = if let AppView::ConfirmPrompt {
            sender, previous, ..
        } = &self.current_view
        {
            let _ = sender.try_send(confirmed);
            Some((**previous).clone())
        } else {
            None
        };

        if let Some(previous) = restore {
            self.current_view = previous;
            self.sync_main_footer_popup(window, cx);
            cx.notify();
        }
    }

    /// Update window size using deferred execution (SAFE during render/event cycles).
    ///
    /// Uses Window::defer to schedule the resize at the end of the current effect cycle,
    /// preventing RefCell borrow conflicts that can occur when calling platform APIs
    /// during GPUI's render or event processing.
    ///
    /// Use this version when you have access to `window` and `cx`.
    pub(crate) fn update_window_size_deferred(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Content-aware mini mode sizing bypasses the flat (ViewType, item_count) path.
        if matches!(self.current_view, AppView::ScriptList)
            && self.main_window_mode == MainWindowMode::Mini
        {
            let (grouped_items, _) = self.get_grouped_results_cached();
            let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);
            let target_height = crate::window_resize::height_for_mini_main_window(sizing);
            crate::window_resize::log_mini_window_sizing(
                crate::window_resize::MiniResizeReason::FilterChanged,
                sizing,
                f32::from(target_height),
            );
            crate::window_resize::defer_resize_to_mini_main_window(sizing, window, &mut *cx);
            return;
        }

        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            crate::window_resize::defer_resize_to_view(view_type, item_count, window, &mut *cx);
        }
    }

    /// Update window size synchronously.
    ///
    /// SAFETY: Only call from async handlers (cx.spawn closures, message handlers)
    /// that run OUTSIDE the GPUI render cycle. Calling during render will cause
    /// RefCell borrow panics.
    ///
    /// Prefer `update_window_size_deferred` when you have window/cx access.
    pub(crate) fn update_window_size(&mut self) {
        // Content-aware mini mode sizing bypasses the flat (ViewType, item_count) path.
        if matches!(self.current_view, AppView::ScriptList)
            && self.main_window_mode == MainWindowMode::Mini
        {
            let (grouped_items, _) = self.get_grouped_results_cached();
            let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);
            let target_height = crate::window_resize::height_for_mini_main_window(sizing);
            crate::window_resize::log_mini_window_sizing(
                crate::window_resize::MiniResizeReason::GroupedResultsChanged,
                sizing,
                f32::from(target_height),
            );
            crate::window_resize::resize_to_mini_main_window_sync(sizing);
            return;
        }

        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            crate::window_resize::resize_to_view_sync(view_type, item_count);
        }
    }

    /// Resize the current surface to its canonical height while restoring an
    /// explicit width.
    pub(crate) fn resize_current_view_to_width(&mut self, target_width: f32) {
        if !target_width.is_finite() || target_width <= 0.0 {
            self.update_window_size();
            return;
        }

        // Content-aware mini mode sizing bypasses the flat (ViewType, item_count) path.
        if matches!(self.current_view, AppView::ScriptList)
            && self.main_window_mode == MainWindowMode::Mini
        {
            let (grouped_items, _) = self.get_grouped_results_cached();
            let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);
            let target_height = crate::window_resize::height_for_mini_main_window(sizing);
            crate::window_resize::log_mini_window_sizing(
                crate::window_resize::MiniResizeReason::GroupedResultsChanged,
                sizing,
                f32::from(target_height),
            );
            let width = if self.main_window_mode == MainWindowMode::Mini {
                crate::window_resize::width_for_view(ViewType::MiniMainWindow)
                    .unwrap_or(target_width)
            } else {
                target_width
            };
            crate::window_resize::resize_first_window_to_size(target_height, Some(width));
            return;
        }

        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            let target_height = crate::window_resize::height_for_view(view_type, item_count);
            let width = if self.main_window_mode == MainWindowMode::Mini {
                crate::window_resize::width_for_view(ViewType::MiniMainWindow)
                    .unwrap_or(target_width)
            } else {
                target_width
            };
            crate::window_resize::resize_first_window_to_size(target_height, Some(width));
        }
    }

    /// Try to insert text into the current prompt's input field.
    ///
    /// Returns `true` when the current view accepted the text (i.e. there is an
    /// active prompt with an input field), `false` otherwise.  Used by dictation
    /// to decide whether to fall back to paste-to-frontmost-app.
    /// Returns `true` when the launcher/main-menu filter is active and can
    /// accept dictated text (i.e. `AppView::ScriptList`).
    pub(crate) fn can_accept_dictation_into_main_filter(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
    }

    /// Returns `true` when the current view can accept dictated text directly.
    pub(crate) fn can_accept_dictation_into_prompt(&self) -> bool {
        matches!(
            &self.current_view,
            AppView::ArgPrompt { .. }
                | AppView::MiniPrompt { .. }
                | AppView::MicroPrompt { .. }
                | AppView::PathPrompt { .. }
                | AppView::SelectPrompt { .. }
                | AppView::EnvPrompt { .. }
                | AppView::TemplatePrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::FileSearchView { .. }
        )
    }

    pub(crate) fn try_set_prompt_input(&mut self, text: String, cx: &mut Context<Self>) -> bool {
        match &mut self.current_view {
            AppView::ArgPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                self.arg_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.update_window_size();
                cx.notify();
                true
            }
            AppView::MiniPrompt { .. } | AppView::MicroPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                self.arg_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.update_window_size();
                cx.notify();
                true
            }
            AppView::PathPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
                true
            }
            AppView::SelectPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
                true
            }
            AppView::EnvPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
                true
            }
            AppView::TemplatePrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
                true
            }
            AppView::FormPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
                true
            }
            AppView::AcpChatView { entity } => {
                entity.update(cx, |view, cx| view.set_input(text, cx));
                true
            }
            AppView::ChatPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
                true
            }
            AppView::FileSearchView {
                query,
                selected_index,
                ..
            } => {
                let results = ScriptListApp::resolve_file_search_results(&text);
                logging::log(
                    "EXEC",
                    &format!(
                        "File search setInput '{}' found {} results",
                        text,
                        results.len()
                    ),
                );
                *query = text.clone();
                *selected_index = 0;
                self.update_file_search_results(results);
                self.file_search_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.filter_text = text;
                self.pending_filter_sync = true;
                cx.notify();
                true
            }
            _ => false,
        }
    }

    pub(crate) fn set_prompt_input(&mut self, text: String, cx: &mut Context<Self>) {
        let _ = self.try_set_prompt_input(text, cx);
    }

    /// Helper to get filtered arg choices without cloning
    pub(crate) fn get_filtered_arg_choices<'a>(&self, choices: &'a [Choice]) -> Vec<&'a Choice> {
        if self.arg_input.is_empty() {
            choices.iter().collect()
        } else {
            let filter = self.arg_input.text().to_lowercase();
            choices
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&filter))
                .collect()
        }
    }

    pub(crate) fn focus_main_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_input = FocusedInput::MainFilter;
        let input_state = self.gpui_input_state.clone();
        input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    /// Apply a dictated transcript to the launcher's shared main-filter input.
    ///
    /// Returns `true` when the launcher was active and the text was applied,
    /// `false` otherwise (caller should fall back to frontmost-app paste).
    pub(crate) fn try_set_main_window_filter_from_dictation(
        &mut self,
        text: String,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.can_accept_dictation_into_main_filter() {
            return false;
        }

        tracing::info!(
            category = "DICTATION",
            event = "dictation_set_main_window_filter",
            text_len = text.len(),
            "Applying dictated transcript to launcher filter"
        );

        self.filter_text = text.clone();
        self.pending_filter_sync = true;
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        self.hovered_index = None;
        self.selected_index = 0;
        self.queue_filter_compute(text, cx);
        cx.notify();
        true
    }

    /// Clear the cached preflight receipt so it is rebuilt on the next
    /// call to `rebuild_main_window_preflight_if_needed`.
    /// Kept as explicit API for context-chip toggles and view transitions.
    #[allow(dead_code)]
    pub(crate) fn invalidate_main_window_preflight(&mut self) {
        self.cached_main_window_preflight = None;
        self.main_window_preflight_cache_key.clear();
    }

    /// Rebuild the preflight receipt when the cache key has changed.
    /// Call this from mutation paths (filter change, selection change)
    /// — never from `render()`.
    pub(crate) fn rebuild_main_window_preflight_if_needed(&mut self) {
        let new_key = format!(
            "{}:{}:{:?}",
            self.filter_text, self.selected_index, self.current_view
        );
        if new_key == self.main_window_preflight_cache_key {
            return;
        }
        self.main_window_preflight_cache_key = new_key;
        let receipt = crate::main_window_preflight::build_main_window_preflight_receipt(self);
        if crate::logging::filter_perf_trace_enabled() {
            if let Some(ref r) = receipt {
                crate::main_window_preflight::log_main_window_preflight_receipt(r);
            }
        }
        self.cached_main_window_preflight = receipt;
    }
}

#[cfg(test)]
mod tests {
    use super::{main_window_result_action_label, paste_into_frontmost_app_label};
    use crate::scripts::{MatchIndices, Scriptlet, ScriptletMatch};
    use std::sync::Arc;

    fn make_scriptlet_result(tool: &str) -> crate::scripts::SearchResult {
        crate::scripts::SearchResult::Scriptlet(ScriptletMatch {
            scriptlet: Arc::new(Scriptlet {
                name: "Test Scriptlet".to_string(),
                description: None,
                code: "echo test".to_string(),
                tool: tool.to_string(),
                shortcut: None,
                keyword: None,
                group: None,
                plugin_id: String::new(),
                plugin_title: None,
                file_path: None,
                command: None,
                alias: None,
            }),
            score: 100,
            display_file_path: None,
            match_indices: MatchIndices::default(),
            match_evidence: None,
        })
    }

    #[test]
    fn paste_into_frontmost_app_label_uses_app_name() {
        assert_eq!(
            paste_into_frontmost_app_label(Some("Safari")),
            "Paste into Safari"
        );
    }

    #[test]
    fn paste_into_frontmost_app_label_falls_back_to_active_app() {
        assert_eq!(
            paste_into_frontmost_app_label(None),
            "Paste into Active App"
        );
    }

    #[test]
    fn main_window_result_action_label_uses_frontmost_app_for_paste_scriptlets() {
        let result = make_scriptlet_result("paste");
        assert_eq!(
            main_window_result_action_label(&result, Some("TextEdit")),
            "Paste into TextEdit"
        );
    }

    #[test]
    fn main_window_result_action_label_keeps_default_for_non_paste_scriptlets() {
        let result = make_scriptlet_result("bash");
        assert_eq!(
            main_window_result_action_label(&result, Some("TextEdit")),
            "Run Command"
        );
    }
}
