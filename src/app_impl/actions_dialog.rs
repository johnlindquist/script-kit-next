use super::*;

/// Whether a view supports the shared ActionsDialog, and if so, which host
/// identity to use for focus-restore and key routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionsSupport {
    /// View participates in the shared ActionsDialog with the given host.
    SharedDialog(ActionsDialogHost),
    /// View does not support the shared ActionsDialog.
    None,
}

impl ScriptListApp {
    fn is_builtin_list_actions_view(view: &AppView) -> bool {
        matches!(
            view,
            AppView::BrowserHistoryView { .. }
                | AppView::BrowserTabsView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::CurrentAppCommandsView { .. }
                | AppView::ProcessManagerView { .. }
                | AppView::SearchAiPresetsView { .. }
                | AppView::CreateAiPresetView { .. }
                | AppView::SettingsView { .. }
                | AppView::FavoritesBrowseView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::BrowseKitsView { .. }
                | AppView::InstalledKitsView { .. }
        )
    }

    /// Static host identity for actions-related routing/focus contracts.
    ///
    /// This is broader than live Cmd+K popup support. Use
    /// `live_actions_host_for_view` or `current_actions_host` when deciding
    /// whether the visible view may open the shared ActionsDialog.
    pub(crate) fn actions_host_for_view(view: &AppView) -> Option<ActionsDialogHost> {
        match view {
            AppView::ScriptList => Some(ActionsDialogHost::MainList),
            AppView::ClipboardHistoryView { .. } => Some(ActionsDialogHost::ClipboardHistory),
            AppView::DictationHistoryView { .. } => Some(ActionsDialogHost::DictationHistory),
            AppView::FavoritesBrowseView { .. } => Some(ActionsDialogHost::Favorites),
            AppView::ThemeChooserView { .. } => Some(ActionsDialogHost::ThemeChooser),
            AppView::EmojiPickerView { .. } => Some(ActionsDialogHost::EmojiPicker),
            AppView::FileSearchView { .. } => Some(ActionsDialogHost::FileSearch),
            AppView::ChatPrompt { .. } => Some(ActionsDialogHost::ChatPrompt),
            AppView::ArgPrompt { .. } => Some(ActionsDialogHost::ArgPrompt),
            AppView::DivPrompt { .. } => Some(ActionsDialogHost::DivPrompt),
            AppView::EditorPrompt { .. } => Some(ActionsDialogHost::EditorPrompt),
            AppView::TermPrompt { .. } => Some(ActionsDialogHost::TermPrompt),
            AppView::FormPrompt { .. } => Some(ActionsDialogHost::FormPrompt),
            AppView::WebcamView { .. } => Some(ActionsDialogHost::WebcamPrompt),
            AppView::AcpChatView { .. } => Some(ActionsDialogHost::AcpChat),
            AppView::AcpHistoryView { .. } => Some(ActionsDialogHost::AcpHistory),
            AppView::AppLauncherView { .. } => Some(ActionsDialogHost::AppLauncher),
            AppView::BrowserHistoryView { .. }
            | AppView::BrowserTabsView { .. }
            | AppView::WindowSwitcherView { .. }
            | AppView::CurrentAppCommandsView { .. }
            | AppView::ProcessManagerView { .. }
            | AppView::SearchAiPresetsView { .. }
            | AppView::CreateAiPresetView { .. }
            | AppView::SettingsView { .. }
            | AppView::DesignGalleryView { .. }
            | AppView::BrowseKitsView { .. }
            | AppView::InstalledKitsView { .. } => Some(ActionsDialogHost::BuiltinList),
            _ => None,
        }
    }

    /// Live routing host for main-window popup handling.
    ///
    /// Generic BuiltinList views remain in `actions_host_for_view` so static
    /// routing and focus-restore code can identify them, but they are not live
    /// Cmd+K hosts until each view provides selection-specific actions.
    pub(crate) fn live_actions_host_for_view(view: &AppView) -> Option<ActionsDialogHost> {
        if Self::is_builtin_list_actions_view(view) {
            return None;
        }

        Self::actions_host_for_view(view)
    }

    pub(crate) fn current_actions_host(&self) -> Option<ActionsDialogHost> {
        Self::live_actions_host_for_view(&self.current_view)
    }

    /// Canonical static resolver: map the current view to shared-actions
    /// identity for routing/focus contracts.
    ///
    /// This is not proof that the visible view may open the shared
    /// ActionsDialog. Live popup-open decisions must use
    /// `live_actions_host_for_view` or `current_actions_host`.
    pub(crate) fn actions_support_for_view(&self) -> ActionsSupport {
        match Self::actions_host_for_view(&self.current_view) {
            Some(host) => ActionsSupport::SharedDialog(host),
            None => ActionsSupport::None,
        }
    }

    /// Convenience: does the current view participate in the shared actions dialog?
    pub(crate) fn current_view_supports_shared_actions(&self) -> bool {
        self.current_actions_host().is_some()
    }

    pub(crate) fn make_actions_dialog_activation_callback(
        app_entity: Entity<Self>,
        host: ActionsDialogHost,
    ) -> std::sync::Arc<
        dyn Fn(crate::actions::ActionsDialogActivation, &mut Window, &mut gpui::App) + Send + Sync,
    > {
        std::sync::Arc::new(move |activation, window, cx| {
            let app_entity = app_entity.clone();
            window.defer(cx, move |window, cx| {
                let _ = app_entity.update(cx, |app, cx| {
                    app.handle_actions_dialog_activation(host, activation.clone(), window, cx);
                });
            });
        })
    }

    pub(crate) fn handle_actions_dialog_activation(
        &mut self,
        host: ActionsDialogHost,
        activation: crate::actions::ActionsDialogActivation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match activation {
            crate::actions::ActionsDialogActivation::DrillDownPushed { .. } => {
                crate::actions::notify_actions_window(cx);
                if let Some(dialog) = self.actions_dialog.as_ref() {
                    crate::actions::resize_actions_window(cx, dialog);
                    let (route_id, search_placeholder, route_depth, escape_hint) = {
                        let dialog_ref = dialog.read(cx);
                        (
                            dialog_ref.current_route_id().map(str::to_string),
                            dialog_ref.current_search_placeholder().map(str::to_string),
                            dialog_ref.route_depth(),
                            dialog_ref.route_hint_label(),
                        )
                    };
                    tracing::info!(
                        target: "script_kit::actions",
                        host = ?host,
                        route_id = ?route_id,
                        route_depth,
                        escape_hint,
                        search_placeholder = ?search_placeholder,
                        "actions_dialog_route_visible"
                    );
                }
            }
            crate::actions::ActionsDialogActivation::Executed {
                action_id,
                should_close,
            } => {
                let root_unified_context = if should_close
                    && matches!(host, ActionsDialogHost::MainList)
                    && crate::root_unified_result_actions::RootUnifiedResultAction::from_action_id(&action_id).is_some()
                {
                    self.pending_root_unified_actions_subject.clone()
                } else {
                    None
                };
                let root_file_context = if root_unified_context.is_none()
                    && should_close
                    && matches!(host, ActionsDialogHost::MainList)
                    && crate::action_helpers::is_root_file_action_id(&action_id)
                {
                    self.pending_root_file_actions_file
                        .clone()
                        .or_else(|| self.selected_root_file_result_owned())
                } else {
                    None
                };
                if should_close {
                    self.close_actions_popup(host, window, cx);
                }
                if let Some(subject) = root_unified_context {
                    if crate::root_unified_result_actions::execute_root_unified_result_action(
                        self,
                        &action_id,
                        &subject,
                        window,
                        cx,
                    ) {
                        return;
                    }
                }
                if let Some(file) = root_file_context {
                    if self.execute_root_file_action(&action_id, &file, window, cx) {
                        return;
                    }
                }
                self.execute_action_for_actions_host(host, action_id, window, cx);
            }
            crate::actions::ActionsDialogActivation::NoSelection => {}
        }
    }

    pub(crate) fn execute_actions_route_action(
        &mut self,
        host: ActionsDialogHost,
        action_id: String,
        should_close: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.handle_actions_dialog_activation(
            host,
            crate::actions::ActionsDialogActivation::Executed {
                action_id,
                should_close,
            },
            window,
            cx,
        );
    }

    pub(crate) fn execute_action_for_actions_host(
        &mut self,
        host: ActionsDialogHost,
        action_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::actions",
            event = "actions_host_execute",
            host = ?host,
            action_id = %action_id,
        );

        // Run 12 Pass 9 — cmdk-safe-action-effects dispatch. When the dialog's
        // chosen row id is `menu_syntax:<original-id>`, route through
        // [[src/menu_syntax/action_effects.rs#apply_safe_effect]] BEFORE the
        // legacy host fallback so the launcher applies the effect (cancel /
        // setFilter / clipboard) instead of falling through to handle_action's
        // unknown-id path.
        if let Some(stripped) = action_id.strip_prefix("menu_syntax:") {
            self.dispatch_menu_syntax_safe_effect(host, stripped, window, cx);
            return;
        }

        match host {
            ActionsDialogHost::MainList => {
                if let Some(subject) = self.pending_root_unified_actions_subject.clone() {
                    if crate::root_unified_result_actions::execute_root_unified_result_action(
                        self,
                        &action_id,
                        &subject,
                        window,
                        cx,
                    ) {
                        return;
                    }
                }
                if crate::root_unified_result_actions::RootUnifiedResultAction::from_action_id(&action_id).is_some() {
                    tracing::warn!(
                        target: "script_kit::actions",
                        event = "root_unified_result_action_missing_subject",
                        action_id = %action_id,
                        "Root result action ignored because no pending subject was captured"
                    );
                    return;
                }
                if crate::action_helpers::is_root_file_action_id(&action_id) {
                    if let Some(file) = self
                        .pending_root_file_actions_file
                        .clone()
                        .or_else(|| self.selected_root_file_result_owned())
                    {
                        if self.execute_root_file_action(&action_id, &file, window, cx) {
                            return;
                        }
                    }
                }
                self.handle_action(action_id, window, cx);
            }
            ActionsDialogHost::ChatPrompt => self.execute_chat_action(&action_id, cx),
            ActionsDialogHost::ArgPrompt => {
                self.trigger_action_by_name(&action_id, cx);
            }
            ActionsDialogHost::WebcamPrompt => {
                let start = std::time::Instant::now();
                let dctx = crate::action_helpers::DispatchContext::for_builtin("builtin/webcam");
                let outcome = self.execute_webcam_action(&action_id, &dctx, cx);
                Self::log_builtin_outcome(
                    "builtin/webcam",
                    &dctx,
                    "webcam_action",
                    &outcome,
                    &start,
                );
            }
            ActionsDialogHost::AcpDetached => {
                let dispatched =
                    crate::ai::acp::chat_window::dispatch_action_to_detached(&action_id, cx);
                tracing::info!(
                    target: "script_kit::actions",
                    event = "actions_host_execute_acp_detached",
                    action_id = %action_id,
                    dispatched,
                );
            }
            ActionsDialogHost::ThemeChooser => {
                self.execute_theme_chooser_action(&action_id, window, cx);
            }
            _ => {
                self.handle_action(action_id, window, cx);
            }
        }
    }

    /// Run 12 Pass 9 — `cmdk-safe-action-effects` dispatch. Looks up the
    /// original `MenuSyntaxAction` by id, calls `apply_safe_effect` to
    /// resolve the effect, then applies it (Cancel → close, SetFilterText →
    /// `set_filter_text_immediate`, WriteClipboard → `cx.write_to_clipboard`).
    /// Always closes the popup at the end so the dispatch is atomic.
    pub(crate) fn dispatch_menu_syntax_safe_effect(
        &mut self,
        host: ActionsDialogHost,
        original_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::menu_syntax::{
            action_effects::{apply_safe_effect, ActionEffect},
            builtin_schema, current_menu_syntax_actions, MenuSyntaxActionState,
        };

        let raw = self.filter_text().to_string();
        let mode = &self.menu_syntax_mode;

        // Reconstruct the live state (mirror of Pass 7's actions_toggle.rs
        // pre-closure block). Borrow lifetimes constrain us to a flat match
        // here.
        let effect: ActionEffect = if let Some(invocation) = mode.capture_for(&raw) {
            let target = invocation.target.clone();
            let schema = builtin_schema(&target);
            let state = MenuSyntaxActionState::CaptureComposer {
                target: &target,
                payload: invocation,
                schema: schema.as_ref(),
            };
            let actions = current_menu_syntax_actions(&state);
            match actions.into_iter().find(|a| a.id == original_id) {
                Some(action) => apply_safe_effect(&state, &action.kind),
                None => ActionEffect::Unsupported,
            }
        } else if let Some(argv) = mode.command_for(&raw) {
            let state = MenuSyntaxActionState::CommandComposer {
                head: &argv.head,
                argv: &argv.argv,
            };
            let actions = current_menu_syntax_actions(&state);
            match actions.into_iter().find(|a| a.id == original_id) {
                Some(action) => apply_safe_effect(&state, &action.kind),
                None => ActionEffect::Unsupported,
            }
        } else if let Some(query) = mode.advanced_query_for(&raw) {
            let state = MenuSyntaxActionState::RefineQuery { query };
            let actions = current_menu_syntax_actions(&state);
            match actions.into_iter().find(|a| a.id == original_id) {
                Some(action) => apply_safe_effect(&state, &action.kind),
                None => ActionEffect::Unsupported,
            }
        } else {
            ActionEffect::Unsupported
        };

        tracing::info!(
            target: "script_kit::actions",
            event = "menu_syntax_safe_effect_dispatched",
            host = ?host,
            original_id = %original_id,
            effect = ?effect,
        );

        match effect {
            ActionEffect::Cancel => {
                // Close popup AND clear the composer filter so Cancel is
                // unambiguous. Mirrors the user expectation "Cancel without
                // saving".
                self.set_filter_text_immediate(String::new(), window, cx);
            }
            ActionEffect::SetFilterText { new_text } => {
                self.set_filter_text_immediate(new_text, window, cx);
            }
            ActionEffect::WriteClipboard { content } => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
            }
            ActionEffect::Unsupported => {
                // Quiet no-op — the dialog row should not have been clickable
                // for an unsupported effect, but if it slipped through, just
                // close cleanly.
            }
        }

        self.close_actions_popup(host, window, cx);
    }

    pub(crate) fn toggle_actions_for_host(
        &mut self,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        tracing::info!(
            target: "script_kit::actions",
            event = "actions_host_toggle_requested",
            host = ?host,
            show_actions_popup = self.show_actions_popup,
        );

        match host {
            ActionsDialogHost::MainList => {
                if let Some(result) = self.selected_main_list_search_result_owned() {
                    match crate::root_unified_result_actions::root_unified_action_owner_for_result(&result) {
                        crate::root_unified_result_actions::RootUnifiedResultActionOwner::RootSubject(subject) => {
                            self.toggle_root_unified_result_actions(subject, window, cx);
                        }
                        crate::root_unified_result_actions::RootUnifiedResultActionOwner::ExistingScriptActions => {
                            if self.has_actions() {
                                self.toggle_actions(cx, window);
                            }
                        }
                        crate::root_unified_result_actions::RootUnifiedResultActionOwner::None => {
                            if self.has_actions() {
                                self.toggle_actions(cx, window);
                            }
                        }
                    }
                } else if self.has_actions() {
                    self.toggle_actions(cx, window);
                }
                true
            }
            ActionsDialogHost::FileSearch => {
                let selected = self.selected_file_search_result_owned();
                self.toggle_file_search_actions(
                    selected.as_ref().map(|(_, file)| file),
                    window,
                    cx,
                );
                true
            }
            ActionsDialogHost::ClipboardHistory => {
                if let Some(entry) = self.selected_clipboard_entry() {
                    self.toggle_clipboard_actions(entry, window, cx);
                    true
                } else if self.show_actions_popup || crate::actions::is_actions_window_open() {
                    self.toggle_actions(cx, window);
                    true
                } else {
                    false
                }
            }
            ActionsDialogHost::DictationHistory => {
                if let Some(entry) = self.selected_dictation_history_entry() {
                    self.toggle_dictation_history_actions(entry, window, cx);
                    true
                } else if self.show_actions_popup || crate::actions::is_actions_window_open() {
                    self.toggle_actions(cx, window);
                    true
                } else {
                    false
                }
            }
            ActionsDialogHost::Favorites => {
                if self.selected_favorite_id().is_some()
                    || self.show_actions_popup
                    || crate::actions::is_actions_window_open()
                {
                    self.toggle_favorites_actions(window, cx);
                    true
                } else {
                    false
                }
            }
            ActionsDialogHost::ThemeChooser => {
                self.toggle_theme_chooser_actions(window, cx);
                true
            }
            ActionsDialogHost::ArgPrompt => {
                self.toggle_arg_actions(cx, window);
                true
            }
            ActionsDialogHost::ChatPrompt => {
                self.toggle_chat_actions(cx, window);
                true
            }
            ActionsDialogHost::WebcamPrompt => {
                self.toggle_webcam_actions(cx, window);
                true
            }
            ActionsDialogHost::EmojiPicker
            | ActionsDialogHost::AppLauncher
            | ActionsDialogHost::AcpChat
            | ActionsDialogHost::AcpHistory
            | ActionsDialogHost::DivPrompt
            | ActionsDialogHost::EditorPrompt
            | ActionsDialogHost::TermPrompt
            | ActionsDialogHost::FormPrompt => {
                self.toggle_actions(cx, window);
                true
            }
            ActionsDialogHost::BuiltinList => {
                if self.show_actions_popup || crate::actions::is_actions_window_open() {
                    self.close_actions_popup(host, window, cx);
                    true
                } else {
                    tracing::info!(
                        target: "script_kit::actions",
                        event = "actions_host_toggle_ignored_builtin_list",
                        host = ?host,
                        view = ?self.current_view,
                        "Ignored BuiltinList actions open because it has no selection-specific dialog"
                    );
                    false
                }
            }
            ActionsDialogHost::AcpDetached => {
                // The detached window has its own Cmd+K handler
                // (`toggle_detached_actions` in `src/ai/acp/chat_window.rs`).
                // The main view never advertises AcpDetached via
                // `current_actions_host()`, so this arm is defensive-only
                // and should never run in practice; leave the main view
                // untouched and report "not handled".
                false
            }
        }
    }

    pub(crate) fn route_key_to_actions_dialog(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        modifiers: &gpui::Modifiers,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ActionsRoute {
        // Not open - let caller handle the key
        if !self.show_actions_popup {
            return ActionsRoute::NotHandled;
        }

        // Defensive: if UI says it's open but dialog is None, don't leak keys
        let Some(ref dialog) = self.actions_dialog else {
            return ActionsRoute::Handled;
        };

        // Use allocation-free key helpers from ui_foundation
        use crate::ui_foundation::{
            is_key_backspace, is_key_down, is_key_enter, is_key_escape, is_key_up, printable_char,
        };

        // Cmd+K toggles the popup closed through the shared close path
        if modifiers.platform
            && !modifiers.shift
            && !modifiers.control
            && !modifiers.alt
            && key.eq_ignore_ascii_case("k")
        {
            self.close_actions_popup(host, window, cx);
            return ActionsRoute::Handled;
        }

        if is_key_up(key) {
            dialog.update(cx, |d, cx| d.move_up(cx));
            crate::actions::notify_actions_window(cx);
            return ActionsRoute::Handled;
        }

        if is_key_down(key) {
            dialog.update(cx, |d, cx| d.move_down(cx));
            crate::actions::notify_actions_window(cx);
            return ActionsRoute::Handled;
        }

        let is_home = key.eq_ignore_ascii_case("home");
        let is_end = key.eq_ignore_ascii_case("end");
        let is_page_up = key.eq_ignore_ascii_case("pageup");
        let is_page_down = key.eq_ignore_ascii_case("pagedown");
        const ACTIONS_PAGE_JUMP: usize = 8;

        if is_home || is_end || is_page_up || is_page_down {
            dialog.update(cx, |d, cx| {
                if d.grouped_items.is_empty() {
                    return;
                }

                if is_home || is_page_up {
                    let steps = if is_home {
                        d.grouped_items.len()
                    } else {
                        ACTIONS_PAGE_JUMP
                    };
                    for _ in 0..steps {
                        let previous = d.selected_index;
                        d.move_up(cx);
                        if d.selected_index == previous {
                            break;
                        }
                    }
                    return;
                }

                let steps = if is_end {
                    d.grouped_items.len()
                } else {
                    ACTIONS_PAGE_JUMP
                };
                for _ in 0..steps {
                    let previous = d.selected_index;
                    d.move_down(cx);
                    if d.selected_index == previous {
                        break;
                    }
                }
            });
            crate::actions::notify_actions_window(cx);
            return ActionsRoute::Handled;
        }

        // Cmd+Enter: send selected action to ACP Chat as a canonical target chip.
        // Must precede the generic Enter branch to avoid being swallowed.
        if modifiers.platform
            && !modifiers.shift
            && !modifiers.control
            && !modifiers.alt
            && is_key_enter(key)
        {
            if let Some(action) = dialog.read(cx).get_selected_action().cloned() {
                let host_label = format!("{:?}", host);
                let target = crate::ai::build_action_target_for_ai(&action, &host_label);
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "tab_ai_actions_dialog_cmd_enter",
                    host = %host_label,
                    action_id = %action.id,
                    semantic_id = %target.semantic_id,
                );
                self.close_actions_popup(host, window, cx);
                self.open_tab_ai_acp_with_explicit_target_preserving_return(target, cx);
                return ActionsRoute::Handled;
            }
        }

        if is_key_enter(key) {
            match dialog.update(cx, |d, cx| d.activate_selected(cx)) {
                crate::actions::ActionsDialogActivation::DrillDownPushed { .. } => {
                    crate::actions::notify_actions_window(cx);
                    crate::actions::resize_actions_window(cx, dialog);
                    let (route_id, search_placeholder, route_depth, escape_hint) = {
                        let dialog_ref = dialog.read(cx);
                        (
                            dialog_ref.current_route_id().map(str::to_string),
                            dialog_ref.current_search_placeholder().map(str::to_string),
                            dialog_ref.route_depth(),
                            dialog_ref.route_hint_label(),
                        )
                    };
                    tracing::info!(
                        target: "script_kit::actions",
                        host = ?host,
                        route_id = ?route_id,
                        route_depth,
                        escape_hint,
                        search_placeholder = ?search_placeholder,
                        "actions_dialog_route_visible"
                    );
                    return ActionsRoute::Handled;
                }
                crate::actions::ActionsDialogActivation::Executed {
                    action_id,
                    should_close,
                } => {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "Actions dialog executing action: {} (close={}, host={:?})",
                            action_id, should_close, host
                        ),
                    );
                    return ActionsRoute::Execute {
                        action_id,
                        should_close,
                    };
                }
                crate::actions::ActionsDialogActivation::NoSelection => {
                    return ActionsRoute::Handled;
                }
            }
        }

        if is_key_escape(key) {
            let outcome = dialog.update(cx, |d, cx| d.handle_escape(cx));
            match outcome {
                crate::actions::ActionsDialogEscapeOutcome::PoppedRoute => {
                    crate::actions::notify_actions_window(cx);
                    crate::actions::resize_actions_window(cx, dialog);
                    let (route_id, search_placeholder, route_depth, escape_hint) = {
                        let dialog_ref = dialog.read(cx);
                        (
                            dialog_ref.current_route_id().map(str::to_string),
                            dialog_ref.current_search_placeholder().map(str::to_string),
                            dialog_ref.route_depth(),
                            dialog_ref.route_hint_label(),
                        )
                    };
                    tracing::info!(
                        target: "script_kit::actions",
                        host = ?host,
                        route_id = ?route_id,
                        route_depth,
                        escape_hint,
                        search_placeholder = ?search_placeholder,
                        "actions_dialog_route_visible"
                    );
                }
                crate::actions::ActionsDialogEscapeOutcome::CloseDialog => {
                    self.close_actions_popup(host, window, cx);
                }
            }
            return ActionsRoute::Handled;
        }

        if is_key_backspace(key) {
            dialog.update(cx, |d, cx| d.handle_backspace(cx));
            crate::actions::notify_actions_window(cx);
            crate::actions::resize_actions_window(cx, dialog);
            return ActionsRoute::Handled;
        }

        // Check for printable character input (only when no modifiers are held)
        // This prevents Cmd+E from being treated as typing 'e' into the search
        if !modifiers.platform && !modifiers.control && !modifiers.alt {
            if let Some(ch) = printable_char(key_char) {
                dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                crate::actions::notify_actions_window(cx);
                crate::actions::resize_actions_window(cx, dialog);
                return ActionsRoute::Handled;
            }
        }

        // Check if keystroke matches any action shortcut in the dialog
        // This allows Cmd+E, Cmd+L, etc. to execute the corresponding action
        let keystroke_shortcut = shortcuts::keystroke_to_shortcut(key, modifiers);
        let matched_action_id = {
            let dialog_ref = dialog.read(cx);
            crate::actions::matching_filtered_action_id_for_keystroke(
                &dialog_ref.actions,
                &dialog_ref.filtered_actions,
                key,
                modifiers,
            )
        };

        if let Some(action_id) = matched_action_id {
            logging::log(
                "ACTIONS",
                &format!(
                    "Actions dialog shortcut matched: {} -> {} (host={:?})",
                    keystroke_shortcut, action_id, host
                ),
            );

            match dialog.update(cx, |d, cx| d.activate_action_id(action_id.clone(), cx)) {
                crate::actions::ActionsDialogActivation::DrillDownPushed { .. } => {
                    crate::actions::notify_actions_window(cx);
                    crate::actions::resize_actions_window(cx, dialog);
                    let (route_id, search_placeholder, route_depth, escape_hint) = {
                        let dialog_ref = dialog.read(cx);
                        (
                            dialog_ref.current_route_id().map(str::to_string),
                            dialog_ref.current_search_placeholder().map(str::to_string),
                            dialog_ref.route_depth(),
                            dialog_ref.route_hint_label(),
                        )
                    };
                    tracing::info!(
                        target: "script_kit::actions",
                        host = ?host,
                        route_id = ?route_id,
                        route_depth,
                        escape_hint,
                        search_placeholder = ?search_placeholder,
                        "actions_dialog_route_visible"
                    );
                    return ActionsRoute::Handled;
                }
                crate::actions::ActionsDialogActivation::Executed {
                    action_id,
                    should_close,
                } => {
                    return ActionsRoute::Execute {
                        action_id,
                        should_close,
                    };
                }
                crate::actions::ActionsDialogActivation::NoSelection => {
                    return ActionsRoute::Handled;
                }
            }
        }

        // Modal behavior: swallow all other keys while popup is open
        ActionsRoute::Handled
    }

    /// Convert a display shortcut (⌘⇧E) to normalized form (cmd+shift+e)
    pub(crate) fn normalize_display_shortcut(hint: &str) -> String {
        crate::components::hint_strip::canonical_shortcut_hint(hint)
    }

    fn should_preserve_main_filter_while_actions_open(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            || self.current_view_uses_shared_filter_input()
    }

    pub(crate) fn mark_filter_resync_after_actions_if_needed(&mut self) {
        if !self.should_preserve_main_filter_while_actions_open() {
            return;
        }

        self.pending_filter_sync = true;
        logging::log(
            "ACTIONS",
            &format!(
                "ACTIONS_FILTER_RESYNC marked pending (show_actions_popup={}, filter_text='{}')",
                self.show_actions_popup, self.filter_text
            ),
        );
    }

    pub(crate) fn resync_filter_input_after_actions_if_needed(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.mark_filter_resync_after_actions_if_needed();
        self.sync_filter_input_if_needed(window, cx);
    }

    pub(crate) fn request_focus_restore_for_actions_host(&mut self, host: ActionsDialogHost) {
        use crate::focus_coordinator::FocusRequest;

        let request = match host {
            ActionsDialogHost::ArgPrompt => FocusRequest::arg_prompt(),
            ActionsDialogHost::ChatPrompt => FocusRequest::chat_prompt(),
            ActionsDialogHost::EditorPrompt => FocusRequest::editor_prompt(),
            ActionsDialogHost::FormPrompt => FocusRequest::form_prompt(),
            ActionsDialogHost::DivPrompt => FocusRequest::div_prompt(),
            ActionsDialogHost::TermPrompt => FocusRequest::term_prompt(),
            ActionsDialogHost::WebcamPrompt => FocusRequest::div_prompt(),
            ActionsDialogHost::AcpChat => FocusRequest::acp_chat(),
            ActionsDialogHost::MainList
            | ActionsDialogHost::FileSearch
            | ActionsDialogHost::ClipboardHistory
            | ActionsDialogHost::DictationHistory
            | ActionsDialogHost::Favorites
            | ActionsDialogHost::ThemeChooser
            | ActionsDialogHost::EmojiPicker
            | ActionsDialogHost::AppLauncher
            | ActionsDialogHost::BuiltinList
            | ActionsDialogHost::AcpHistory
            | ActionsDialogHost::AcpDetached => FocusRequest::main_filter(),
        };

        self.focus_coordinator.request(request);
        self.sync_coordinator_to_legacy();
    }

    /// Check if the actions popup was closed very recently (within 300ms).
    ///
    /// This guards against a race where clicking the footer ⌘K button causes
    /// the actions window's activation observer to close the dialog (deferred)
    /// before the click handler fires `toggle_actions`. Without this debounce
    /// the toggle would see the dialog as closed and immediately reopen it.
    pub(crate) fn was_actions_recently_closed(&self) -> bool {
        const ACTIONS_CLOSE_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(300);
        self.actions_closed_at
            .map(|t| t.elapsed() < ACTIONS_CLOSE_DEBOUNCE)
            .unwrap_or(false)
    }

    /// Mark the shared actions popup as opening.
    ///
    /// This is the mutation owner for the popup-open flag and debounce reset.
    /// Keep the `Clear debounce on open` phrase here so source audits protect
    /// the footer-Cmd+K race guard without requiring every call site to repeat
    /// the raw field writes.
    pub(crate) fn mark_actions_popup_opening(&mut self) {
        self.show_actions_popup = true;
        self.actions_closed_at = None; // Clear debounce on open
    }

    /// Clear shared actions popup state without recording a recent-close debounce.
    ///
    /// Use this for route changes, resets, and stale-overlay cleanup where the
    /// UI is not handling a user close gesture that should debounce footer Cmd+K.
    pub(crate) fn clear_actions_popup_state(&mut self) {
        self.show_actions_popup = false;
        self.actions_dialog = None;
    }

    pub(crate) fn clear_actions_context_for_host(&mut self, host: ActionsDialogHost) {
        if matches!(host, ActionsDialogHost::MainList) {
            self.pending_root_file_actions_file = None;
            self.pending_root_unified_actions_subject = None;
        }
    }

    /// Mark the shared actions popup as closed.
    ///
    /// This is the mutation owner for the popup-open flag and close timestamp.
    /// Keep the `Record debounce on close` phrase here so close paths share the
    /// same 300ms recent-close behavior.
    pub(crate) fn mark_actions_popup_closed(&mut self) {
        self.clear_actions_popup_state();
        self.actions_closed_at = Some(std::time::Instant::now()); // Record debounce on close
    }

    /// Close the actions popup and restore focus based on host type.
    ///
    /// This centralizes close behavior, ensuring cx.notify() is always called
    /// and focus is correctly restored based on which prompt hosted the dialog.
    pub(crate) fn close_actions_popup(
        &mut self,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let closing_from_actions_window = crate::actions::is_actions_window(window);
        if closing_from_actions_window {
            crate::platform::activate_main_window();
        }

        let overlay_depth_before_on_close = self.focus_coordinator.overlay_depth();
        let on_close_callback = self
            .actions_dialog
            .as_ref()
            .and_then(|dialog| dialog.read(cx).on_close.clone());

        if let Some(on_close) = on_close_callback {
            logging::log(
                "ACTIONS",
                &format!(
                    "ACTIONS_CLOSE_POPUP invoking on_close callback (host={:?}, overlay_depth_before={})",
                    host, overlay_depth_before_on_close
                ),
            );
            on_close(cx);
        }

        let overlay_depth_after_on_close = self.focus_coordinator.overlay_depth();
        let callback_restored_focus = overlay_depth_after_on_close < overlay_depth_before_on_close;

        self.mark_actions_popup_closed();
        self.resync_filter_input_after_actions_if_needed(window, cx);

        // Close the separate actions window if open
        // This ensures consistent behavior whether closing via Cmd+K, Escape, backdrop click,
        // or any other close mechanism
        if is_actions_window_open() {
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                });
            })
            .detach();
        }

        // Use coordinator to pop overlay and restore previous focus.
        // Skip pop when the dialog callback already restored focus to avoid double-pop.
        if !callback_restored_focus {
            self.pop_focus_overlay(cx);
        }

        self.request_focus_restore_for_actions_host(host);
        self.clear_actions_context_for_host(host);

        // Apply restored focus immediately rather than deferring to next render.
        // pop_focus_overlay sets pending_focus to the saved target (e.g. ChatPrompt).
        // Applying it now avoids race conditions with the async window close.
        if closing_from_actions_window {
            logging::log(
                "FOCUS",
                "Actions popup closed from actions window; pending focus will apply on the main window",
            );
        } else if !self.apply_pending_focus(window, cx) {
            // Fallback: focus app root if no pending focus was applied
            window.focus(&self.focus_handle, cx);
        }
        logging::log(
            "FOCUS",
            &format!(
                "Actions popup closed (host={:?}), focus restored via coordinator",
                host
            ),
        );
        cx.notify();

        // Check for a pending ACP handoff target enqueued by the detached
        // actions window's Cmd+Enter handler. The slot is only populated
        // when a secondary surface explicitly requested the handoff.
        if let Some(target) = crate::ai::take_pending_explicit_acp_target() {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "tab_ai_pending_acp_target_picked_up",
                item_source = %target.source,
                semantic_id = %target.semantic_id,
                current_view = ?self.current_view,
            );
            self.open_tab_ai_acp_with_explicit_target_preserving_return(target, cx);
        }
    }

    pub(crate) fn close_actions_popup_for_current_view(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(host) = self.current_actions_host() {
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_close_current_view",
                host = ?host,
            );
            self.close_actions_popup(host, window, cx);
            return;
        }

        tracing::warn!(
            target: "script_kit::actions",
            event = "actions_close_without_live_host",
            view = ?self.current_view,
        );

        self.mark_actions_popup_closed();
        self.mark_filter_resync_after_actions_if_needed();
        if is_actions_window_open() {
            close_actions_window(cx);
        }
        self.pop_focus_overlay(cx);
        cx.notify();
    }
}

#[cfg(test)]
mod close_actions_popup_regression_tests {
    use std::fs;

    #[test]
    fn test_close_actions_popup_invokes_on_close_before_clearing_dialog_state() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let on_close_pos = close_fn
            .find("on_close(cx);")
            .expect("close_actions_popup must invoke on_close callback");
        let clear_dialog_pos = close_fn
            .find("self.actions_dialog = None;")
            .expect("close_actions_popup must clear actions_dialog state");

        assert!(
            on_close_pos < clear_dialog_pos,
            "close_actions_popup must invoke on_close before clearing actions_dialog state"
        );
    }

    #[test]
    fn test_close_actions_popup_resyncs_filter_input_after_clearing_dialog_state() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let clear_dialog_pos = close_fn
            .find("self.actions_dialog = None;")
            .expect("close_actions_popup must clear actions_dialog state");
        let resync_pos = close_fn
            .find("self.resync_filter_input_after_actions_if_needed(window, cx);")
            .expect("close_actions_popup must resync canonical filter input state");

        assert!(
            clear_dialog_pos < resync_pos,
            "close_actions_popup must resync filter input after clearing actions dialog state"
        );
    }

    #[test]
    fn test_close_actions_popup_notifies_after_focus_restore_paths() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let fallback_focus_pos = close_fn
            .find("window.focus(&self.focus_handle, cx);")
            .expect("close_actions_popup must keep fallback root focus");
        let notify_pos = close_fn
            .find("cx.notify();")
            .expect("close_actions_popup must notify after closing popup");

        assert!(
            fallback_focus_pos < notify_pos,
            "close_actions_popup must notify after focus restore paths complete"
        );
    }

    #[test]
    fn test_close_actions_popup_restores_host_focus_before_apply_pending_focus() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let host_restore_pos = close_fn
            .find("self.request_focus_restore_for_actions_host(host);")
            .expect("close_actions_popup must request host-specific focus restore");
        let apply_pending_pos = close_fn
            .find("self.apply_pending_focus(window, cx)")
            .expect("close_actions_popup must apply pending focus after host restore");

        assert!(
            host_restore_pos < apply_pending_pos,
            "close_actions_popup should request host-specific focus before applying pending focus"
        );
    }

    #[test]
    fn test_close_actions_popup_defers_focus_apply_from_actions_window() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let detect_pos = close_fn
            .find("let closing_from_actions_window = crate::actions::is_actions_window(window);")
            .expect("close_actions_popup must detect actions-window-originated closes");
        let activate_pos = close_fn
            .find("crate::platform::activate_main_window();")
            .expect("actions-window-originated close must activate the main window");
        let host_restore_pos = close_fn
            .find("self.request_focus_restore_for_actions_host(host);")
            .expect("host focus restore request missing");
        let deferred_branch_pos = close_fn
            .find("if closing_from_actions_window")
            .expect("actions-window-originated close must skip child-window focus apply");
        let apply_pending_pos = close_fn
            .find("self.apply_pending_focus(window, cx)")
            .expect("non-actions-window close path must still apply focus immediately");

        assert!(
            detect_pos < activate_pos && activate_pos < host_restore_pos,
            "close_actions_popup should activate the main window before queuing host focus restore"
        );
        assert!(
            host_restore_pos < deferred_branch_pos && deferred_branch_pos < apply_pending_pos,
            "actions-window closes must defer focus application instead of focusing the popup window"
        );
    }

    #[test]
    fn test_actions_host_focus_restore_maps_prompt_hosts() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let helper_start = source
            .find("fn request_focus_restore_for_actions_host")
            .expect("request_focus_restore_for_actions_host function not found");
        let helper_fn = &source[helper_start..];

        for expected in [
            "ActionsDialogHost::ArgPrompt => FocusRequest::arg_prompt()",
            "ActionsDialogHost::ChatPrompt => FocusRequest::chat_prompt()",
            "ActionsDialogHost::AcpChat => FocusRequest::acp_chat()",
            "ActionsDialogHost::EditorPrompt => FocusRequest::editor_prompt()",
            "ActionsDialogHost::FormPrompt => FocusRequest::form_prompt()",
            "ActionsDialogHost::DivPrompt => FocusRequest::div_prompt()",
            "ActionsDialogHost::TermPrompt => FocusRequest::term_prompt()",
            "ActionsDialogHost::WebcamPrompt => FocusRequest::div_prompt()",
            "ActionsDialogHost::MainList",
            "ActionsDialogHost::FileSearch",
            "ActionsDialogHost::ClipboardHistory",
            "ActionsDialogHost::EmojiPicker",
            "ActionsDialogHost::AppLauncher",
            "FocusRequest::main_filter()",
        ] {
            assert!(
                helper_fn.contains(expected),
                "request_focus_restore_for_actions_host should include mapping fragment: {}",
                expected
            );
        }
    }
}

#[cfg(test)]
mod actions_host_mapping_tests {
    use super::*;

    #[test]
    fn actions_host_for_view_maps_in_scope_surfaces() {
        let cases = vec![
            (AppView::ScriptList, Some(ActionsDialogHost::MainList)),
            (
                AppView::AppLauncherView {
                    filter: String::new(),
                    selected_index: 0,
                },
                Some(ActionsDialogHost::AppLauncher),
            ),
            (
                AppView::ThemeChooserView {
                    filter: String::new(),
                    selected_index: 0,
                },
                Some(ActionsDialogHost::ThemeChooser),
            ),
            (
                AppView::SettingsView {
                    filter: String::new(),
                    selected_index: 0,
                },
                Some(ActionsDialogHost::BuiltinList),
            ),
            (
                AppView::AcpHistoryView {
                    filter: String::new(),
                    selected_index: 0,
                },
                Some(ActionsDialogHost::AcpHistory),
            ),
            (
                AppView::ClipboardHistoryView {
                    filter: String::new(),
                    selected_index: 0,
                },
                Some(ActionsDialogHost::ClipboardHistory),
            ),
            (
                AppView::BrowserHistoryView {
                    filter: String::new(),
                    selected_index: 0,
                },
                Some(ActionsDialogHost::BuiltinList),
            ),
            (
                AppView::EmojiPickerView {
                    filter: String::new(),
                    selected_index: 0,
                    selected_category: None,
                },
                Some(ActionsDialogHost::EmojiPicker),
            ),
            (
                AppView::MiniPrompt {
                    id: String::new(),
                    placeholder: String::new(),
                    choices: Vec::new(),
                },
                None,
            ),
        ];

        for (view, expected) in cases {
            assert_eq!(ScriptListApp::actions_host_for_view(&view), expected);
        }
    }

    #[test]
    fn live_actions_host_for_view_excludes_generic_builtin_list_views() {
        let browser_history = AppView::BrowserHistoryView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&browser_history),
            None
        );

        let settings = AppView::SettingsView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(ScriptListApp::live_actions_host_for_view(&settings), None);

        let theme_chooser = AppView::ThemeChooserView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&theme_chooser),
            Some(ActionsDialogHost::ThemeChooser)
        );

        let current_app_commands = AppView::CurrentAppCommandsView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&current_app_commands),
            None
        );

        let design_gallery = AppView::DesignGalleryView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&design_gallery),
            None
        );

        let process_manager = AppView::ProcessManagerView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&process_manager),
            None
        );
    }

    #[test]
    fn live_actions_host_for_view_keeps_selection_specific_hosts() {
        let file_search = AppView::FileSearchView {
            query: String::new(),
            selected_index: 0,
            presentation: FileSearchPresentation::Full,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&file_search),
            Some(ActionsDialogHost::FileSearch)
        );

        let clipboard = AppView::ClipboardHistoryView {
            filter: String::new(),
            selected_index: 0,
        };
        assert_eq!(
            ScriptListApp::live_actions_host_for_view(&clipboard),
            Some(ActionsDialogHost::ClipboardHistory)
        );
    }
}

#[cfg(test)]
mod actions_dialog_wiring_regression_tests {
    use std::fs;

    #[test]
    fn render_script_list_routes_popup_keys_before_generic_cmd_shortcuts() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        let route_pos = source
            .find("this.route_key_to_actions_dialog(")
            .expect("render_script_list must use the shared actions router");
        let cmd_pos = source
            .find("if has_cmd")
            .expect("render_script_list cmd shortcut block not found");

        assert!(
            route_pos < cmd_pos,
            "render_script_list must route popup keys before generic Cmd shortcuts"
        );
    }

    #[test]
    fn route_key_to_actions_dialog_notifies_after_arrow_navigation() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");

        let route_start = source
            .find("pub(crate) fn route_key_to_actions_dialog")
            .expect("route_key_to_actions_dialog not found");
        let route_fn = &source[route_start..];

        let up_start = route_fn
            .find("if is_key_up(key)")
            .expect("up branch missing");
        let down_start = route_fn
            .find("if is_key_down(key)")
            .expect("down branch missing");
        let jump_start = route_fn
            .find("let is_home = key.eq_ignore_ascii_case(\"home\")")
            .expect("jump-key section missing");

        assert!(
            route_fn[up_start..down_start].contains("crate::actions::notify_actions_window(cx);"),
            "up branch must notify the actions window"
        );
        assert!(
            route_fn[down_start..jump_start].contains("crate::actions::notify_actions_window(cx);"),
            "down branch must notify the actions window"
        );
    }

    #[test]
    fn route_key_to_actions_dialog_handles_cmd_k_close() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");

        assert!(
            source.contains("key.eq_ignore_ascii_case(\"k\")")
                && source.contains("self.close_actions_popup(host, window, cx);"),
            "shared actions router should close the popup on Cmd+K"
        );
    }

    #[test]
    fn route_key_to_actions_dialog_preserves_return_origin_for_explicit_acp_handoff() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");

        assert!(
            source.contains(
                "self.open_tab_ai_acp_with_explicit_target_preserving_return(target, cx);"
            ),
            "shared actions Cmd+Enter handoff should seed ACP return origin before opening ACP"
        );
    }

    #[test]
    fn render_script_list_has_no_duplicate_popup_handler() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        // The old inline popup handler used this pattern - it should be gone
        assert!(
            !source.contains("if this.show_actions_popup {"),
            "render_script_list must not contain a duplicate inline popup key handler"
        );
    }
}
