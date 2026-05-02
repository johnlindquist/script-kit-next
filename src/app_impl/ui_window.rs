use super::*;
use std::sync::Once;

static MAIN_FOOTER_ACTION_LISTENER: Once = Once::new();

/// Thin wrapper delegating to the canonical implementation in `window_resize`.
fn mini_main_window_sizing_from_grouped_items(
    grouped_items: &[GroupedListItem],
) -> crate::window_resize::MiniMainWindowSizing {
    crate::window_resize::mini_main_window_sizing_from_grouped_items(grouped_items)
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

        let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
        if actions_open && !action.is_actions() {
            if let super::actions_dialog::ActionsSupport::SharedDialog(host) =
                self.actions_support_for_view()
            {
                self.close_actions_popup(host, window, cx);
                tracing::info!(
                    target: "script_kit::footer_popup",
                    event = "main_window_footer_action_closed_actions_only",
                    source,
                    action = ?action,
                    host = ?host,
                    "Closed actions dialog from footer outside-click target without dispatching action"
                );
            }
            return;
        }

        match action {
            crate::footer_popup::FooterAction::Run => {
                if !self.try_run_ready_acp_script(cx) {
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
                self.open_tab_ai_acp_with_entry_intent(None, cx);
            }
            crate::footer_popup::FooterAction::Apply => {
                if matches!(self.current_view, AppView::ConfirmPrompt { .. }) {
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
                if matches!(self.current_view, AppView::ConfirmPrompt { .. }) {
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
                    self.close_tab_ai_harness_terminal_with_window(window, cx);
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
            FooterButtonConfig::new(FooterAction::Ai, "⌘↵", "AI").enabled(!footer_disabled),
        ];

        if self.current_view_supports_shared_actions()
            || matches!(self.current_view, AppView::PathPrompt { .. })
        {
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
        match &self.current_view {
            AppView::ScriptList => Some("script_list"),
            AppView::SelectPrompt { .. } => Some("select_prompt"),
            AppView::DivPrompt { .. } => Some("div_prompt"),
            AppView::FormPrompt { .. } => Some("form_prompt"),
            AppView::EditorPrompt { .. } => Some("editor_prompt"),
            AppView::EnvPrompt { .. } => Some("env_prompt"),
            AppView::DropPrompt { .. } => Some("drop_prompt"),
            AppView::TemplatePrompt { .. } => Some("template_prompt"),
            AppView::MiniPrompt { .. } => Some("mini_prompt"),
            AppView::ClipboardHistoryView { .. } => Some("clipboard_history"),
            AppView::FileSearchView { .. } => Some("file_search"),
            AppView::WebcamView { .. } => Some("webcam_prompt"),
            AppView::NamingPrompt { .. } => Some("naming_prompt"),
            AppView::CreationFeedback { .. } => Some("creation_feedback"),
            AppView::ArgPrompt { .. } => Some("arg_prompt"),
            AppView::EmojiPickerView { .. } => Some("emoji_picker"),
            AppView::AcpHistoryView { .. } => Some("acp_history"),
            AppView::BrowserHistoryView { .. } => Some("browser_history"),
            AppView::DictationHistoryView { .. } => Some("dictation_history"),
            AppView::AcpChatView { .. } => Some("acp_chat"),
            AppView::ChatPrompt { .. } => Some("chat_prompt"),
            AppView::QuickTerminalView { .. } => Some("quick_terminal"),
            AppView::PathPrompt { .. } => Some("path_prompt"),
            AppView::AppLauncherView { .. } => Some("app_launcher"),
            AppView::WindowSwitcherView { .. } => Some("window_switcher"),
            AppView::BrowserTabsView { .. } => Some("browser_tabs"),
            AppView::DesignGalleryView { .. } => Some("design_gallery"),
            AppView::ScratchPadView { .. } => Some("scratch_pad"),
            AppView::ThemeChooserView { .. } => Some("theme_chooser"),
            AppView::BrowseKitsView { .. } => Some("browse_kits"),
            AppView::InstalledKitsView { .. } => Some("installed_kits"),
            AppView::ProcessManagerView { .. } => Some("process_manager"),
            AppView::CurrentAppCommandsView { .. } => Some("current_app_commands"),
            AppView::SearchAiPresetsView { .. } => Some("search_ai_presets"),
            AppView::CreateAiPresetView { .. } => Some("create_ai_preset"),
            AppView::SettingsView { .. } => Some("settings"),
            AppView::FavoritesBrowseView { .. } => Some("favorites"),
            AppView::ConfirmPrompt { .. } => Some("confirm_prompt"),
            _ => None,
        }
    }

    /// Quick Terminal footer buttons. Scoped to actions actually meaningful in
    /// the Quick Terminal surface: always Close (⌘W), plus Apply (⌘↩) only
    /// when a tab-AI apply-back route AND its return view are both present.
    /// Run/AI/Actions are intentionally omitted — Quick Terminal shares the
    /// main menu's native footer chrome but not its main-menu-specific actions.
    fn quick_terminal_footer_buttons(&self) -> Vec<crate::footer_popup::FooterButtonConfig> {
        use crate::footer_popup::{FooterAction, FooterButtonConfig};

        let footer_disabled = self.main_window_footer_buttons_blocked();
        let enabled = !footer_disabled;
        let can_apply = self.tab_ai_harness_apply_back_route.is_some()
            && self.tab_ai_harness_return_view.is_some();

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

        // For ACP views, hide the Run button until a validated script receipt
        // exists. This prevents running the wrong target (first main-list item)
        // when a script is still being generated/validated.
        if matches!(self.current_view, AppView::AcpChatView { .. }) {
            let ready = self.acp_ready_script_path.is_some();
            let buttons = self.acp_footer_buttons(ready);
            tracing::info!(
                target: "script_kit::footer_popup",
                event = "main_window_footer_buttons_resolved",
                view = ?self.current_view,
                button_count = buttons.len(),
                acp_ready = ready,
                "Resolved ACP footer buttons"
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

    /// Build footer buttons for the ACP chat surface. The `Run` button only
    /// appears when a validated `SCRIPT_READY` receipt exists.
    fn acp_footer_buttons(&self, ready: bool) -> Vec<crate::footer_popup::FooterButtonConfig> {
        use crate::footer_popup::{FooterAction, FooterButtonConfig};

        let footer_disabled = self.main_window_footer_buttons_blocked();
        let actions_open = self.show_actions_popup || crate::actions::is_actions_window_open();
        let enabled = !footer_disabled;

        let mut buttons = Vec::new();
        if ready {
            buttons.push(FooterButtonConfig::new(FooterAction::Run, "↵", "Run").enabled(enabled));
        }
        buttons.push(
            FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
                .selected(actions_open)
                .enabled(enabled),
        );
        buttons
    }

    pub(crate) fn main_window_footer_config(
        &self,
    ) -> Option<crate::footer_popup::MainWindowFooterConfig> {
        use crate::footer_popup::MainWindowFooterConfig;

        let surface = self.main_window_footer_surface()?;
        let buttons = self.main_window_footer_buttons_for_current_view();

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
        crate::is_main_window_visible() && self.main_window_footer_config().is_some()
    }

    /// When the native main-window footer is active, replace the GPUI footer
    /// with a transparent spacer so content stays clear of the AppKit footer.
    pub(crate) fn main_window_footer_slot(
        &self,
        gpui_footer: gpui::AnyElement,
    ) -> Option<gpui::AnyElement> {
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

        if self.main_window_footer_config().is_none() || !crate::is_main_window_visible() {
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

    pub(crate) fn sync_main_footer_popup(&self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.ensure_main_footer_action_listener(window, cx);

        let mut config = if crate::is_main_window_visible() {
            self.main_window_footer_config()
        } else {
            None
        };

        // Enrich with ACP streaming/model info when on the ACP chat view.
        if let Some(ref mut cfg) = config {
            self.enrich_footer_config_with_acp_info(cfg, cx);
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
        cx: &gpui::App,
    ) {
        if let AppView::AcpChatView { entity } = &self.current_view {
            let view = entity.read(cx);
            if !view.is_setup_mode() {
                let thread = view.live_thread().read(cx);
                config.left_info = Some(crate::footer_popup::FooterLeftInfo {
                    dot_status: view.footer_dot_status(cx),
                    model_name: thread.selected_model_display().to_string(),
                    prefer_accent_for_active_states: true,
                });
            }
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
                    Some((ViewType::ArgPromptNoChoices, 0))
                } else {
                    Some((ViewType::ArgPromptWithChoices, filtered.len().min(5)))
                }
            }
            AppView::MicroPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::DivPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FormPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::SelectPrompt { .. } => Some((ViewType::ArgPromptWithChoices, 0)),
            AppView::PathPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::EnvPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)), // Compact: header + footer only
            AppView::DropPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => Some((ViewType::DivPrompt, 0)), // Template prompt uses div size
            AppView::ChatPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Chat prompt uses div size
            AppView::TermPrompt { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                None
            }
            // P0 FIX: Clipboard history and app launcher use standard height (same as script list)
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
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::EmojiPickerView {
                filter,
                selected_category,
                ..
            } => {
                let row_count = crate::emoji::filtered_grid_row_count(filter, *selected_category);
                Some((ViewType::ScriptList, row_count))
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
                Some((ViewType::ScriptList, filtered_count))
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
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::DesignGalleryView { filter, .. } => {
                // Calculate total gallery items (separators + icons)
                let total_items = designs::separator_variations::SeparatorStyle::count()
                    + designs::icon_variations::total_icon_count();
                let filtered_count = if filter.is_empty() {
                    total_items
                } else {
                    // For now, return total - filtering can be added later
                    total_items
                };
                Some((ViewType::ScriptList, filtered_count))
            }
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
                Some((ViewType::ScriptList, filtered_count))
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
                Some((ViewType::ScriptList, filtered_count))
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
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::ScratchPadView { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::QuickTerminalView { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::WebcamView { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FileSearchView { ref query, .. } => {
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
                Some((ViewType::ScriptList, filtered_count))
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
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::CreationFeedback { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::ScriptIssuesView { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::SdkReferenceView {
                entries, filter, ..
            } => {
                let (_, count) =
                    crate::mcp_resources::sdk_reference_dataset_and_visible_counts(entries, filter);
                Some((ViewType::ScriptList, count))
            }
            AppView::ScriptTemplateCatalogView {
                templates, filter, ..
            } => {
                let (_, count) =
                    crate::mcp_resources::script_template_catalog_dataset_and_visible_counts(
                        templates, filter,
                    );
                Some((ViewType::ScriptList, count))
            }
            AppView::NamingPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::BrowseKitsView { results, .. } => Some((ViewType::ScriptList, results.len())),
            AppView::InstalledKitsView { kits, .. } => Some((ViewType::ScriptList, kits.len())),
            AppView::SearchAiPresetsView { .. } => {
                // Presets list - defaults (5) + user presets
                let count = crate::ai::presets::load_presets()
                    .map(|p| 5 + p.len())
                    .unwrap_or(5);
                Some((ViewType::ScriptList, count))
            }
            AppView::CreateAiPresetView { .. } => {
                // Fixed-size form with 3 fields
                Some((ViewType::ArgPromptNoChoices, 0))
            }
            AppView::SettingsView { .. } => Some((ViewType::ScriptList, 0)),
            AppView::FavoritesBrowseView { .. } => Some((ViewType::ScriptList, 0)),
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
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::BrowserHistoryView { filter, .. } => Some((
                ViewType::ScriptList,
                crate::browser_history::fuzzy_search_browser_history(
                    &self.cached_browser_history,
                    filter,
                )
                .len(),
            )),
            AppView::DictationHistoryView { filter, .. } => Some((
                ViewType::ScriptList,
                crate::dictation::search_history(filter, 100).len(),
            )),
            AppView::NotesBrowseView { filter, .. } => Some((
                ViewType::ScriptList,
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
            AppView::AcpChatView { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::ConfirmPrompt { .. } => Some((ViewType::DivPrompt, 0)),
        }
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
            crate::window_resize::resize_first_window_to_size(target_height, Some(target_width));
            return;
        }

        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            let target_height = crate::window_resize::height_for_view(view_type, item_count);
            crate::window_resize::resize_first_window_to_size(target_height, Some(target_width));
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
        if let Some(ref r) = receipt {
            crate::main_window_preflight::log_main_window_preflight_receipt(r);
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
