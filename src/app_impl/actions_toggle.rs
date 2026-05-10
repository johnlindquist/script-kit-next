use super::*;

pub(crate) const TERM_PROMPT_CLEAR_ACTION_ID: &str = "clear";
pub(crate) const TERM_PROMPT_CLEAR_SHORTCUT: &str = "⌘K";
pub(crate) const TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID: &str = "term_prompt_toggle_actions";
pub(crate) const TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT: &str = "⌘⇧K";
const TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID: &str = "scroll_to_bottom";

fn terminal_action_sort_key(action_id: &str) -> Option<usize> {
    match action_id {
        "copy" => Some(0),
        "copy_all" => Some(1),
        "copy_last_command" => Some(2),
        "copy_last_output" => Some(3),
        "paste" => Some(4),
        "select_all" => Some(5),
        "find" => Some(6),
        "scroll_to_top" => Some(7),
        TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID => Some(8),
        TERM_PROMPT_CLEAR_ACTION_ID => Some(9),
        "reset" => Some(10),
        _ => None,
    }
}

fn terminal_action_section(action_id: &str) -> Option<&'static str> {
    match action_id {
        "copy" | "copy_all" | "copy_last_command" | "copy_last_output" | "paste" | "select_all" => {
            Some("Clipboard")
        }
        "find" => Some("Search"),
        "scroll_to_top" | TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID => Some("Navigation"),
        TERM_PROMPT_CLEAR_ACTION_ID | "reset" => Some("Session"),
        _ => None,
    }
}

fn terminal_action_icon(action_id: &str) -> Option<crate::designs::icon_variations::IconName> {
    use crate::designs::icon_variations::IconName;

    match action_id {
        "copy" | "copy_all" | "copy_last_command" | "copy_last_output" | "paste" | "select_all" => {
            Some(IconName::Copy)
        }
        "find" => Some(IconName::MagnifyingGlass),
        "scroll_to_top" => Some(IconName::ArrowUp),
        TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID => Some(IconName::ArrowDown),
        TERM_PROMPT_CLEAR_ACTION_ID => Some(IconName::Trash),
        "reset" => Some(IconName::Refresh),
        _ => None,
    }
}

fn terminal_actions_dialog_config() -> crate::actions::ActionsDialogConfig {
    use crate::actions::{ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle};

    ActionsDialogConfig {
        search_position: SearchPosition::Top,
        section_style: SectionStyle::Headers,
        anchor: AnchorPosition::Top,
        show_icons: true,
        ..ActionsDialogConfig::default()
    }
}

fn terminal_actions_for_dialog() -> Vec<crate::actions::Action> {
    use crate::actions::{Action, ActionCategory};
    use crate::designs::icon_variations::IconName;

    let mut actions: Vec<Action> = crate::terminal::get_terminal_commands()
        .into_iter()
        .filter_map(|cmd| {
            let action_id = cmd.action.id();
            let sort_key = terminal_action_sort_key(action_id)?;

            let shortcut = if action_id == TERM_PROMPT_CLEAR_ACTION_ID {
                Some(TERM_PROMPT_CLEAR_SHORTCUT.to_string())
            } else {
                cmd.shortcut.clone()
            };

            let mut action = Action::new(
                action_id,
                cmd.name.clone(),
                Some(cmd.description.clone()),
                ActionCategory::Terminal,
            )
            .with_shortcut_opt(shortcut);

            if let Some(section) = terminal_action_section(action_id) {
                action = action.with_section(section);
            }

            if let Some(icon) = terminal_action_icon(action_id) {
                action = action.with_icon(icon);
            }

            Some((sort_key, action))
        })
        .map(|(_sort_key, action)| action)
        .collect();

    if !actions
        .iter()
        .any(|action| action.id == TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID)
    {
        actions.push(
            Action::new(
                TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID,
                "Scroll to Bottom",
                Some("Jump to the bottom (latest output)".to_string()),
                ActionCategory::Terminal,
            )
            .with_shortcut("⌘↓")
            .with_section("Navigation")
            .with_icon(IconName::ArrowDown),
        );
    }

    actions
        .sort_by_key(|action| terminal_action_sort_key(action.id.as_str()).unwrap_or(usize::MAX));

    actions.push(
        Action::new(
            TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID,
            "Toggle Actions",
            Some("Open or close the terminal actions palette".to_string()),
            ActionCategory::Terminal,
        )
        .with_shortcut(TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT)
        .with_icon(IconName::Settings),
    );

    actions
}

pub(crate) fn root_file_actions_for(
    file: &crate::file_search::FileResult,
) -> Vec<crate::actions::Action> {
    use crate::actions::{Action, ActionCategory};
    use crate::designs::icon_variations::IconName;

    let is_dir = file.file_type == crate::file_search::FileType::Directory;
    let open_title = if is_dir { "Open Folder" } else { "Open File" };
    let open_description = if is_dir {
        "Opens this folder".to_string()
    } else {
        "Opens with the default app".to_string()
    };

    vec![
        Action::new(
            crate::action_helpers::ROOT_FILE_OPEN_ACTION_ID,
            open_title,
            Some(open_description),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{21b5}")
        .with_icon(if is_dir {
            IconName::FolderOpen
        } else {
            IconName::File
        })
        .with_section("Actions"),
        Action::new(
            crate::action_helpers::ROOT_FILE_REVEAL_IN_FINDER_ACTION_ID,
            "Reveal in Finder",
            Some("Shows this item in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{21e7}F")
        .with_icon(IconName::FolderOpen)
        .with_section("Share"),
        Action::new(
            crate::action_helpers::ROOT_FILE_COPY_PATH_ACTION_ID,
            "Copy Path",
            Some("Copies the full path to the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{21e7}C")
        .with_icon(IconName::Copy)
        .with_section("Share"),
        Action::new(
            crate::action_helpers::ROOT_FILE_QUICK_LOOK_ACTION_ID,
            "Quick Look",
            Some("Previews this item with Quick Look".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}Y")
        .with_icon(IconName::File)
        .with_section("Actions"),
    ]
}

fn actions_dialog_host_label(host: &ActionsDialogHost) -> &'static str {
    match host {
        ActionsDialogHost::MainList => "MainList",
        ActionsDialogHost::ClipboardHistory => "ClipboardHistory",
        ActionsDialogHost::DictationHistory => "DictationHistory",
        ActionsDialogHost::Favorites => "Favorites",
        ActionsDialogHost::ThemeChooser => "ThemeChooser",
        ActionsDialogHost::EmojiPicker => "EmojiPicker",
        ActionsDialogHost::FileSearch => "FileSearch",
        ActionsDialogHost::ChatPrompt => "ChatPrompt",
        ActionsDialogHost::ArgPrompt => "ArgPrompt",
        ActionsDialogHost::DivPrompt => "DivPrompt",
        ActionsDialogHost::EditorPrompt => "EditorPrompt",
        ActionsDialogHost::TermPrompt => "TermPrompt",
        ActionsDialogHost::FormPrompt => "FormPrompt",
        ActionsDialogHost::WebcamPrompt => "WebcamPrompt",
        ActionsDialogHost::AppLauncher => "AppLauncher",
        ActionsDialogHost::BuiltinList => "BuiltinList",
        ActionsDialogHost::AcpChat => "AcpChat",
        ActionsDialogHost::AcpHistory => "AcpHistory",
        ActionsDialogHost::AcpDetached => "AcpDetached",
    }
}

impl ScriptListApp {
    pub(crate) fn make_actions_window_on_close_callback(
        app_entity: Entity<Self>,
        host: ActionsDialogHost,
        log_message: &'static str,
    ) -> std::sync::Arc<dyn Fn(&mut gpui::App) + Send + Sync> {
        std::sync::Arc::new(move |cx| {
            let app_entity = app_entity.clone();
            cx.defer(move |cx| {
                app_entity.update(cx, |app, cx| {
                    if !app.show_actions_popup && app.actions_dialog.is_none() {
                        return;
                    }

                    let should_hide_main_after_actions_focus_loss =
                        matches!(host, ActionsDialogHost::MainList)
                            && app.can_preserve_hide_script_list_on_passive_focus_loss()
                            && !crate::platform::is_main_window_focused();

                    app.mark_actions_popup_closed();
                    app.mark_filter_resync_after_actions_if_needed();
                    app.pop_focus_overlay(cx);

                    if should_hide_main_after_actions_focus_loss {
                        logging::log(
                            "FOCUS",
                            "Actions popup closed after ScriptList focus loss - hiding main while preserving state",
                        );
                        app.hide_main_window_preserving_state_for_focus_loss(cx);
                        cx.notify();
                        return;
                    }

                    if !script_kit_gpui::is_main_window_visible() {
                        logging::log(
                            "FOCUS",
                            "Actions popup closed after main was already hidden - skipping focus restoration",
                        );
                        cx.notify();
                        return;
                    }

                    app.request_focus_restore_for_actions_host(host);
                    logging::log("FOCUS", log_message);
                    cx.notify();
                });
            });
        })
    }

    pub(crate) fn spawn_open_actions_window(
        cx: &mut Context<Self>,
        parent_window_handle: gpui::AnyWindowHandle,
        main_bounds: gpui::Bounds<gpui::Pixels>,
        display_id: Option<gpui::DisplayId>,
        dialog: Entity<ActionsDialog>,
        position: crate::actions::WindowPosition,
        opened_log: &'static str,
        failed_prefix: &'static str,
    ) {
        dialog.update(cx, |dialog, _cx| {
            dialog.set_skip_track_focus(true);
        });

        let parent_automation_id = crate::windows::focused_automation_window_id();
        cx.spawn(async move |this, cx| {
            cx.update(|cx| {
                match open_actions_window(
                    cx,
                    parent_window_handle,
                    main_bounds,
                    display_id,
                    dialog,
                    position,
                    parent_automation_id.as_deref(),
                ) {
                    Ok(_handle) => {
                        logging::log("ACTIONS", opened_log);
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "{}", failed_prefix);
                        crate::actions::emit_actions_popup_event(
                            crate::actions::ActionsPopupEvent::OpenFailed,
                            None,
                            None,
                            None,
                            None,
                            None,
                        );
                        // Roll back popup state and show Toast on failure
                        let _ = this.update(cx, |app, cx| {
                            app.clear_actions_popup_state();
                            app.pop_focus_overlay(cx);
                            app.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("{}: {}", failed_prefix, e),
                                    &app.theme,
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

    /// Resolve the actions popup window position based on the current window mode.
    ///
    /// Mini mode uses `TopCenter` to keep the popup integrated within the compact
    /// 480px launcher panel.  Full mode uses the default `BottomRight` above the
    /// footer, matching the existing Raycast-style layout.
    fn main_list_actions_window_position(&self) -> crate::actions::WindowPosition {
        match self.main_window_mode {
            MainWindowMode::Mini => crate::actions::WindowPosition::TopCenter,
            MainWindowMode::Full => crate::actions::WindowPosition::BottomRight,
        }
    }

    fn begin_actions_popup_window_open(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        self.mark_actions_popup_opening();
        self.hovered_index = None;
        self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
    }

    fn actions_dialog_host_for_current_view(&self) -> Option<ActionsDialogHost> {
        self.current_actions_host()
    }

    /// Predicate for the stdin `simulateKey` generic Cmd+K fallback.
    ///
    /// Per-view simulateKey arms can still claim richer Cmd+K behavior first.
    /// This helper is only for the outer fallback: plain Cmd+K on a view that
    /// participates in the shared actions dialog should toggle actions instead
    /// of falling through to the unhandled-view warning.
    pub(crate) fn simulate_key_requests_generic_actions_toggle(
        &self,
        key_lower: &str,
        has_cmd: bool,
        has_shift: bool,
        has_alt: bool,
        has_ctrl: bool,
    ) -> bool {
        has_cmd
            && !has_shift
            && !has_alt
            && !has_ctrl
            && key_lower == "k"
            && self.current_actions_host().is_some()
    }

    /// Single per-view actions-toggle dispatcher.
    ///
    /// Every footer click and `Cmd+K` keystroke should funnel through this
    /// method so that the correct view-specific toggle runs regardless of
    /// the trigger source.  Returns `true` when the current view handled
    /// the toggle (caller should stop propagation), `false` otherwise.
    pub(crate) fn dispatch_actions_toggle_for_current_view(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        trigger: &'static str,
    ) -> bool {
        tracing::info!(
            target: "script_kit::actions",
            event = "actions_toggle_dispatch_started",
            trigger = trigger,
            view = ?self.current_view,
            show_actions_popup = self.show_actions_popup,
            actions_window_open = crate::actions::is_actions_window_open(),
            "Dispatching shared actions toggle for current view"
        );

        if matches!(&self.current_view, AppView::ScriptList) {
            if let Some(file) = self.selected_root_file_result_owned() {
                self.toggle_root_file_actions(&file, window, cx);
                return true;
            }

            if self.has_actions()
                || self.show_actions_popup
                || crate::actions::is_actions_window_open()
            {
                self.toggle_actions(cx, window);
                return true;
            } else {
                tracing::info!(
                    target: "script_kit::actions",
                    event = "actions_toggle_dispatch_ignored_no_actions",
                    trigger = trigger,
                    view = ?self.current_view,
                    selected_index = self.selected_index,
                    "Ignored shared actions toggle because the current script selection has no actions"
                );
                return false;
            }
        }

        if matches!(&self.current_view, AppView::FileSearchView { .. }) {
            let selected = self.selected_file_search_result_owned();
            if let Some((display_index, _)) = &selected {
                if let AppView::FileSearchView { selected_index, .. } = &mut self.current_view {
                    *selected_index = *display_index;
                }
            }
            self.toggle_file_search_actions(selected.as_ref().map(|(_, f)| f), window, cx);
            return true;
        }

        if matches!(&self.current_view, AppView::ArgPrompt { .. }) {
            self.toggle_arg_actions(cx, window);
            return true;
        }

        if matches!(&self.current_view, AppView::ChatPrompt { .. }) {
            self.toggle_chat_actions(cx, window);
            return true;
        }

        if matches!(&self.current_view, AppView::WebcamView { .. }) {
            self.toggle_webcam_actions(cx, window);
            return true;
        }

        if matches!(&self.current_view, AppView::ClipboardHistoryView { .. }) {
            if let Some(entry) = self.selected_clipboard_entry() {
                self.toggle_clipboard_actions(entry, window, cx);
                return true;
            }
            if self.show_actions_popup || crate::actions::is_actions_window_open() {
                self.toggle_actions(cx, window);
                return true;
            }
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_clipboard_selection",
                trigger = trigger,
                "Ignored shared actions toggle because clipboard history has no selected entry"
            );
            return false;
        }

        if matches!(&self.current_view, AppView::DictationHistoryView { .. }) {
            if let Some(entry) = self.selected_dictation_history_entry() {
                self.toggle_dictation_history_actions(entry, window, cx);
                return true;
            }
            if self.show_actions_popup || crate::actions::is_actions_window_open() {
                self.toggle_actions(cx, window);
                return true;
            }
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_dictation_selection",
                trigger = trigger,
                "Ignored shared actions toggle because dictation history has no selected entry"
            );
            return false;
        }

        if matches!(&self.current_view, AppView::FavoritesBrowseView { .. }) {
            if self.selected_favorite_id().is_some()
                || self.show_actions_popup
                || crate::actions::is_actions_window_open()
            {
                self.toggle_favorites_actions(window, cx);
                return true;
            }
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_favorite_selection",
                trigger = trigger,
                "Ignored shared actions toggle because favorites has no selected item"
            );
            return false;
        }

        if matches!(&self.current_view, AppView::ThemeChooserView { .. }) {
            self.toggle_theme_chooser_actions(window, cx);
            return true;
        }

        if matches!(&self.current_view, AppView::AcpChatView { .. }) {
            self.toggle_actions(cx, window);
            return true;
        }

        if let AppView::PathPrompt { entity, .. } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |prompt, cx| {
                prompt.toggle_actions(cx);
            });
            return true;
        }

        // Generic fallback for any remaining view that advertises SharedDialog
        // support via actions_support_for_view() but doesn't need a dedicated
        // branch with selection-specific context (e.g. DivPrompt, EditorPrompt,
        // TermPrompt, FormPrompt, EmojiPicker, AcpHistory).
        if self.current_view_supports_shared_actions() {
            return self.dispatch_shared_actions_toggle_fallback(window, cx, trigger);
        }

        tracing::info!(
            target: "script_kit::actions",
            event = "actions_toggle_dispatch_ignored_unsupported_view",
            trigger = trigger,
            view = ?self.current_view,
            "Ignored shared actions toggle because current view does not expose footer actions"
        );
        false
    }

    /// Shared fallback for views that advertise SharedDialog support but do not
    /// need a dedicated branch with selection-specific context.
    fn dispatch_shared_actions_toggle_fallback(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        trigger: &'static str,
    ) -> bool {
        let can_toggle = self.has_actions()
            || self.show_actions_popup
            || crate::actions::is_actions_window_open();

        if !can_toggle {
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_actions",
                trigger = trigger,
                view = ?self.current_view,
                show_actions_popup = self.show_actions_popup,
                actions_window_open = crate::actions::is_actions_window_open(),
                "Ignored shared actions toggle because the current shared-dialog view has no actions"
            );
            return false;
        }

        self.toggle_actions(cx, window);
        tracing::info!(
            target: "script_kit::actions",
            event = "actions_toggle_dispatch_routed_shared_dialog_fallback",
            trigger = trigger,
            view = ?self.current_view,
            show_actions_popup = self.show_actions_popup,
            actions_window_open = crate::actions::is_actions_window_open(),
            "Routed shared actions toggle through generic shared-dialog fallback"
        );
        true
    }

    /// Route `Cmd+K` through the shared actions dispatcher.
    ///
    /// This ensures the keyboard shortcut uses the same path as footer clicks
    /// and the native mini-footer bridge, preventing behavioral drift.
    pub(crate) fn handle_cmd_k_actions_toggle(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let handled = self.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k");
        tracing::info!(
            target: "script_kit::actions",
            event = "cmd_k_actions_routed",
            handled,
            view = ?self.current_view,
            selected_index = self.selected_index,
            show_actions_popup = self.show_actions_popup,
            actions_window_open = crate::actions::is_actions_window_open(),
            "Routed Cmd+K through shared actions dispatcher"
        );
        handled
    }

    pub(crate) fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        let Some(host) = self.actions_dialog_host_for_current_view() else {
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_ignored_unsupported_view",
                view = ?self.current_view,
                "Ignored actions toggle because current view does not participate in the shared actions dialog"
            );
            cx.notify();
            return;
        };

        let host_label = actions_dialog_host_label(&host);
        let recently_closed = self.was_actions_recently_closed();

        tracing::info!(
            target: "script_kit::actions",
            event = "actions_toggle_routed",
            host = host_label,
            view = ?self.current_view,
            show_actions_popup = self.show_actions_popup,
            actions_window_open = is_actions_window_open(),
            "Routing actions toggle through canonical view host"
        );

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(host, window, cx);
        } else if recently_closed {
            // The activation-triggered close (focus_lost) already closed the dialog
            // between mouseDown and the click handler. Suppress reopen.
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_suppressed_recent_close",
                host = host_label,
                view = ?self.current_view,
                "Suppressed actions reopen because the dialog was just closed"
            );
        } else {
            // Any view that reaches this point has passed the live-host
            // resolver. Selection-specific hosts such as File Search may still
            // open with only global rows, but generic BuiltinList surfaces are
            // filtered out before this path so Theme Chooser cannot advertise
            // misleading launcher actions.
            let _ = self.has_actions();

            let position = self.main_list_actions_window_position();
            crate::actions::emit_actions_popup_event(
                crate::actions::ActionsPopupEvent::OpenRequested,
                Some(host_label),
                Some(position),
                None,
                None,
                None,
            );

            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open actions as a separate window with vibrancy blur
            self.begin_actions_popup_window_open(cx, window);

            let acp_context = if let AppView::AcpChatView { ref entity } = self.current_view {
                // Trigger a preflight `session/new` so the agent re-advertises its
                // model catalog before we snapshot `available_models` for the
                // Change Model drill-down. Fire-and-forget: this dialog opening
                // uses whatever the thread has right now; subsequent openings pick
                // up whatever the agent just advertised.
                let thread_for_refresh = if let crate::ai::acp::AcpChatSession::Live(ref thread) =
                    entity.read(cx).session
                {
                    Some(thread.clone())
                } else {
                    None
                };
                if let Some(thread) = thread_for_refresh {
                    thread.update(cx, |thread, cx| thread.refresh_models(cx));
                }

                let (selected_agent_id, catalog_entries, selected_model_id, available_models) = {
                    let view = entity.read(cx);
                    match &view.session {
                        crate::ai::acp::AcpChatSession::Setup(state) => (
                            state
                                .selected_agent
                                .as_ref()
                                .map(|agent| agent.id.to_string()),
                            state.catalog_entries.clone(),
                            None,
                            Vec::new(),
                        ),
                        crate::ai::acp::AcpChatSession::Live(thread) => {
                            let thread = thread.read(cx);
                            (
                                thread.selected_agent_id().map(str::to_string),
                                thread.available_agents().to_vec(),
                                thread.selected_model_id().map(str::to_string),
                                thread.available_models().to_vec(),
                            )
                        }
                    }
                };

                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_actions_context_built",
                    selected_agent_id = ?selected_agent_id,
                    catalog_count = catalog_entries.len(),
                    selected_model_id = ?selected_model_id,
                    model_count = available_models.len(),
                );

                Some((
                    selected_agent_id,
                    catalog_entries,
                    selected_model_id,
                    available_models,
                ))
            } else {
                None
            };
            // Defensive guard for any explicit BuiltinList host path: its
            // selected_index belongs to that built-in list, not the script list
            // cache read by `get_focused_script_info`.
            let on_builtin_list = matches!(host, ActionsDialogHost::BuiltinList);
            let script_info = if on_builtin_list {
                None
            } else {
                self.get_focused_script_info()
            };

            // Get the full scriptlet with actions if focused item is a scriptlet
            let focused_scriptlet = if on_builtin_list {
                None
            } else {
                self.get_focused_scriptlet_with_actions()
            };

            // Run 12 Pass 7 — compute the Power Syntax section from the live
            // filter parse + active mode BEFORE the dialog-construction closure
            // (can't borrow `self` inside `cx.new`). Returns None when not
            // composing a Power Syntax expression so the dialog falls back to
            // the legacy script_context + global rows.
            let power_syntax_section_for_dialog: Option<
                crate::menu_syntax_actions::PowerSyntaxActionSection,
            > = {
                use crate::menu_syntax::{builtin_schema, MenuSyntaxActionState};
                let raw = self.filter_text().to_string();
                let mode = &self.menu_syntax_mode;
                if let Some(invocation) = mode.capture_for(&raw) {
                    let target = invocation.target.clone();
                    let schema = builtin_schema(&target);
                    let state = MenuSyntaxActionState::CaptureComposer {
                        target: &target,
                        payload: invocation,
                        schema: schema.as_ref(),
                    };
                    Some(crate::menu_syntax_actions::power_syntax_action_section(
                        &state,
                    ))
                } else if let Some(argv) = mode.command_for(&raw) {
                    let state = MenuSyntaxActionState::CommandComposer {
                        head: &argv.head,
                        argv: &argv.argv,
                    };
                    Some(crate::menu_syntax_actions::power_syntax_action_section(
                        &state,
                    ))
                } else if let Some(query) = mode.advanced_query_for(&raw) {
                    let state = MenuSyntaxActionState::RefineQuery { query };
                    Some(crate::menu_syntax_actions::power_syntax_action_section(
                        &state,
                    ))
                } else {
                    None
                }
            };

            // Create the dialog entity HERE in main app (for keyboard routing)
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let is_mini = matches!(self.main_window_mode, MainWindowMode::Mini);
            let is_acp_actions_dialog = acp_context.is_some();
            // Create the dialog entity
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = if let Some((
                    ref selected_agent_id,
                    ref catalog_entries,
                    ref selected_model_id,
                    ref available_models,
                )) = acp_context
                {
                    // ACP chat view: use route-based dialog with drill-down agent/model pickers
                    ActionsDialog::with_acp_chat(
                        focus_handle,
                        std::sync::Arc::new(|_action_id| {}),
                        crate::actions::AcpActionsDialogContext {
                            catalog_entries,
                            selected_agent_id: selected_agent_id.as_deref(),
                            available_models,
                            selected_model_id: selected_model_id.as_deref(),
                        },
                        std::sync::Arc::clone(&theme_arc),
                    )
                } else {
                    ActionsDialog::with_script(
                        focus_handle,
                        std::sync::Arc::new(|_action_id| {}),
                        script_info.clone(),
                        theme_arc,
                    )
                };

                // Mini mode: input at top, anchor top (collapses from bottom up)
                if is_mini {
                    dialog.set_config(crate::actions::ActionsDialogConfig {
                        search_position: crate::actions::SearchPosition::Top,
                        section_style: crate::actions::SectionStyle::Headers,
                        anchor: crate::actions::AnchorPosition::Top,
                        show_icons: true,
                        search_placeholder: script_info.as_ref().map(|script| script.name.clone()),
                        show_context_header: false,
                        ..crate::actions::ActionsDialogConfig::default()
                    });
                } else {
                    dialog.set_config(crate::actions::ActionsDialogConfig {
                        search_position: crate::actions::SearchPosition::Bottom,
                        section_style: crate::actions::SectionStyle::Headers,
                        anchor: crate::actions::AnchorPosition::Bottom,
                        search_placeholder: script_info.as_ref().map(|script| script.name.clone()),
                        show_context_header: false,
                        ..crate::actions::ActionsDialogConfig::default()
                    });
                }

                // If we have a scriptlet with actions, pass it to the dialog.
                // ACP owns its route stack and action source; script/global
                // rebuild hooks would replace Change Agent/Model with launcher
                // actions.
                if !is_acp_actions_dialog {
                    if let Some(ref scriptlet) = focused_scriptlet {
                        dialog.set_focused_scriptlet(script_info.clone(), Some(scriptlet.clone()));
                    }
                }

                // Run 12 Pass 7 — wire the cmdk-actions Power Syntax section.
                // The owned section was computed BEFORE entering this closure
                // (can't borrow `self` inside `cx.new`); push it now. ACP uses
                // its own route-backed actions, so skip the generic script/global
                // action rebuild there.
                if !is_acp_actions_dialog {
                    dialog.set_menu_syntax_section(power_syntax_section_for_dialog.clone());
                }

                // Skip track_focus so the parent window keeps keyboard routing
                // (matches command_bar.rs pattern)
                dialog.set_skip_track_focus(true);
                dialog.set_match_main_window_background(true);
                dialog
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            // This ensures the same cleanup happens whether closing via Cmd+K toggle or Escape
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_activation(Self::make_actions_dialog_activation_callback(
                    app_entity.clone(),
                    host,
                ));
                d.set_on_close(Self::make_actions_window_on_close_callback(
                    app_entity.clone(),
                    host,
                    "Actions closed via escape, focus restored via coordinator",
                ));
            });

            // Get main window bounds and display_id for positioning the actions popup
            //
            // CRITICAL: We use GPUI's window.bounds() which returns SCREEN-RELATIVE coordinates
            // (top-left origin, relative to the window's current screen). We also capture the
            // display_id so the actions window is created on the SAME screen as the main window.
            //
            // This fixes multi-monitor issues where the actions popup would appear on the wrong
            // screen or at wrong coordinates when the main window was on a secondary display.
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Main window bounds (GPUI screen-relative): origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
                    main_bounds.origin.x, main_bounds.origin.y,
                    main_bounds.size.width, main_bounds.size.height,
                    display_id
                ),
            );

            // Open the actions window via spawn, passing the shared dialog entity and display_id
            Self::spawn_open_actions_window(
                cx,
                window.window_handle(),
                main_bounds,
                display_id,
                dialog,
                position,
                "Actions popup window opened",
                "Failed to open actions window",
            );

            logging::log("FOCUS", "Actions opened, keyboard routing active");
        }
        cx.notify();
    }

    pub(crate) fn toggle_root_file_actions(
        &mut self,
        file: &crate::file_search::FileResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let host = ActionsDialogHost::MainList;
        let host_label = actions_dialog_host_label(&host);
        let recently_closed = self.was_actions_recently_closed();

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(host, window, cx);
            cx.notify();
            return;
        }

        if recently_closed {
            tracing::info!(
                target: "script_kit::actions",
                event = "root_file_actions_toggle_suppressed_recent_close",
                path = %file.path,
                "Suppressed root file actions reopen because the dialog was just closed"
            );
            cx.notify();
            return;
        }

        let actions = root_file_actions_for(file);
        self.pending_root_file_actions_file = Some(file.clone());
        let context_title = Some(file.name.clone());
        let theme_arc = std::sync::Arc::clone(&self.theme);
        let is_mini = matches!(self.main_window_mode, MainWindowMode::Mini);
        let config = crate::actions::ActionsDialogConfig {
            search_position: if is_mini {
                crate::actions::SearchPosition::Top
            } else {
                crate::actions::SearchPosition::Bottom
            },
            section_style: crate::actions::SectionStyle::Headers,
            anchor: if is_mini {
                crate::actions::AnchorPosition::Top
            } else {
                crate::actions::AnchorPosition::Bottom
            },
            show_icons: true,
            search_placeholder: context_title.clone(),
            show_context_header: false,
            ..crate::actions::ActionsDialogConfig::default()
        };

        let position = self.main_list_actions_window_position();
        crate::actions::emit_actions_popup_event(
            crate::actions::ActionsPopupEvent::OpenRequested,
            Some(host_label),
            Some(position),
            None,
            None,
            None,
        );

        self.resync_filter_input_after_actions_if_needed(window, cx);
        self.begin_actions_popup_window_open(cx, window);

        let dialog = cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = crate::actions::ActionsDialog::from_actions_with_context(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}),
                actions,
                None,
                None,
                theme_arc,
                crate::designs::DesignVariant::Default,
                context_title,
                config,
            );
            dialog.set_skip_track_focus(true);
            dialog.set_match_main_window_background(true);
            dialog
        });

        self.actions_dialog = Some(dialog.clone());
        let app_entity = cx.entity().clone();
        dialog.update(cx, |d, _cx| {
            d.set_on_activation(Self::make_actions_dialog_activation_callback(
                app_entity.clone(),
                host,
            ));
            d.set_on_close(Self::make_actions_window_on_close_callback(
                app_entity.clone(),
                host,
                "Root file actions closed via escape, focus restored via coordinator",
            ));
        });

        let main_bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());
        Self::spawn_open_actions_window(
            cx,
            window.window_handle(),
            main_bounds,
            display_id,
            dialog,
            position,
            "Root file actions popup window opened",
            "Failed to open root file actions window",
        );

        logging::log("FOCUS", "Root file actions opened, keyboard routing active");
        cx.notify();
    }

    /// Toggle actions dialog for arg prompts with SDK-defined actions.
    ///
    /// Opens the dialog inline (not as a separate window) since arg prompts
    /// host the actions overlay within the main window. Uses the same
    /// open/close state contract as other popup toggles:
    /// - Sets `show_actions_popup` + pushes focus overlay on open
    /// - Clears `gpui_input_focused` to prevent stale input routing
    /// - Always ends with `cx.notify()` to flush UI state
    pub(crate) fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log(
            "KEY",
            &format!(
                "toggle_arg_actions called: show_actions_popup={}, actions_dialog.is_some={}, sdk_actions.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some(),
                self.sdk_actions.is_some()
            ),
        );
        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::ArgPrompt, window, cx);
        } else if self.was_actions_recently_closed() {
            logging::log(
                "KEY",
                "Suppressed arg actions reopen — closed within debounce window",
            );
        } else {
            // Clone SDK actions early to avoid borrow conflicts
            let sdk_actions_opt = self.sdk_actions.clone();

            // Check if we have SDK actions
            if let Some(sdk_actions) = sdk_actions_opt {
                logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
                if !sdk_actions.is_empty() {
                    self.resync_filter_input_after_actions_if_needed(window, cx);
                    // Open - push overlay to save arg prompt focus state
                    self.mark_actions_popup_opening();
                    self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);
                    self.gpui_input_focused = false;

                    let theme_arc = std::sync::Arc::clone(&self.theme);
                    let dialog = cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        let mut dialog = ActionsDialog::with_script(
                            focus_handle,
                            std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                            None,                                 // No script info for arg prompts
                            theme_arc,
                        );
                        // Set SDK actions to replace built-in actions
                        dialog.set_sdk_actions(sdk_actions);
                        dialog
                    });

                    // Focus the dialog's internal focus handle
                    self.actions_dialog = Some(dialog.clone());
                    let app_entity = cx.entity().clone();
                    dialog.update(cx, |d, _cx| {
                        d.set_on_activation(Self::make_actions_dialog_activation_callback(
                            app_entity,
                            ActionsDialogHost::ArgPrompt,
                        ));
                    });
                    let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                    window.focus(&dialog_focus_handle, cx);
                    logging::log(
                        "FOCUS",
                        &format!(
                            "Arg actions OPENED: show_actions_popup={}, actions_dialog.is_some={}",
                            self.show_actions_popup,
                            self.actions_dialog.is_some()
                        ),
                    );
                } else {
                    logging::log("KEY", "No SDK actions available to show (empty list)");
                }
            } else {
                logging::log("KEY", "No SDK actions defined for this arg prompt (None)");
            }
        }
        cx.notify();
    }
    /// Toggle actions dialog for webcam prompt (built-in command).
    /// Opens as a separate window (same pattern as toggle_chat_actions).
    pub fn toggle_webcam_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{ActionsDialog, ActionsDialogConfig};

        logging::log(
            "KEY",
            &format!(
                "toggle_webcam_actions called: show_actions_popup={}, is_actions_window_open={}",
                self.show_actions_popup,
                is_actions_window_open()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            // Close — delegate to central close_actions_popup
            self.close_actions_popup(ActionsDialogHost::WebcamPrompt, window, cx);
        } else {
            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open actions as a separate window — same pattern as toggle_chat_actions
            self.begin_actions_popup_window_open(cx, window);

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let webcam_actions = Self::webcam_actions_for_dialog();

            // Use native Action rows with default actions config so webcam uses the same
            // filtering/navigation behavior as the main actions dialog.
            let dialog = cx.new(move |cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_config(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}),
                    webcam_actions,
                    theme_arc,
                    ActionsDialogConfig::default(),
                );
                dialog.set_context_title(Some("Webcam".to_string()));
                dialog.set_match_main_window_background(true);
                dialog
            });

            self.actions_dialog = Some(dialog.clone());

            // Set up on_close callback — same pattern as toggle_chat_actions
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_activation(Self::make_actions_dialog_activation_callback(
                    app_entity.clone(),
                    ActionsDialogHost::WebcamPrompt,
                ));
                d.set_on_close(Self::make_actions_window_on_close_callback(
                    app_entity.clone(),
                    ActionsDialogHost::WebcamPrompt,
                    "Webcam actions closed via escape, focus restored via coordinator",
                ));
            });

            // Get main window bounds for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            // Open the actions window — same as toggle_chat_actions
            Self::spawn_open_actions_window(
                cx,
                window.window_handle(),
                main_bounds,
                display_id,
                dialog,
                crate::actions::WindowPosition::BottomRight,
                "Webcam actions popup window opened",
                "Failed to open webcam actions window",
            );

            logging::log("FOCUS", "Webcam actions opened, keyboard routing active");
        }
        cx.notify();
    }

    /// Toggle terminal command bar for built-in terminal
    /// Shows common terminal actions (Clear, Copy, Paste, Scroll, etc.)
    /// Opens as a separate vibrancy window for native macOS blur effect.
    #[allow(dead_code)]
    pub fn toggle_terminal_commands(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::ActionsDialog;

        logging::log(
            "KEY",
            &format!(
                "toggle_terminal_commands called: show_actions_popup={}, actions_dialog.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::TermPrompt, window, cx);
        } else {
            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open as a separate vibrancy window for native macOS blur
            self.begin_actions_popup_window_open(cx, window);

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let actions = terminal_actions_for_dialog();
            let config = terminal_actions_dialog_config();

            let dialog = cx.new(move |cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_config(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}),
                    actions,
                    theme_arc,
                    config,
                );
                dialog.set_match_main_window_background(true);
                dialog
            });
            dialog.update(cx, |d, _cx| {
                d.set_context_title(Some("Terminal".to_string()));
            });

            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_activation(Self::make_actions_dialog_activation_callback(
                    app_entity.clone(),
                    ActionsDialogHost::TermPrompt,
                ));
                d.set_on_close(Self::make_actions_window_on_close_callback(
                    app_entity,
                    ActionsDialogHost::TermPrompt,
                    "Terminal actions closed, focus restored via coordinator",
                ));
            });

            self.actions_dialog = Some(dialog.clone());

            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            Self::spawn_open_actions_window(
                cx,
                window.window_handle(),
                main_bounds,
                display_id,
                dialog,
                crate::actions::WindowPosition::BottomRight,
                "Terminal actions popup window opened",
                "Failed to open terminal actions window",
            );

            logging::log("FOCUS", "Terminal actions opened with vibrancy window");
        }
        cx.notify();
    }

    /// Resolve the chat actions popup window position based on the current window mode.
    ///
    /// Mini mode uses `TopCenter` to match the mini main launcher feel.
    /// Full mode uses `BottomRight` (existing behavior).
    fn chat_actions_window_position(&self) -> crate::actions::WindowPosition {
        let position = match self.main_window_mode {
            MainWindowMode::Mini => crate::actions::WindowPosition::TopCenter,
            MainWindowMode::Full => crate::actions::WindowPosition::BottomRight,
        };
        tracing::info!(
            event = "chat_actions_window_position.resolved",
            mode = ?self.main_window_mode,
            position = ?position,
            "Resolved chat actions anchor position"
        );
        position
    }

    /// Toggle actions dialog for chat prompts
    /// Opens ActionsDialog with model selection and chat-specific actions
    pub fn toggle_chat_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{ChatModelInfo, ChatPromptInfo};

        logging::log(
            "KEY",
            &format!(
                "toggle_chat_actions called: show_actions_popup={}, actions_dialog.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::ChatPrompt, window, cx);
        } else {
            // Get chat info from current ChatPrompt entity
            let chat_info = if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                let chat = entity.read(cx);
                ChatPromptInfo {
                    current_model: chat.model.clone(),
                    available_models: chat
                        .models
                        .iter()
                        .map(|m| ChatModelInfo {
                            id: m.id.clone(),
                            display_name: m.name.clone(),
                            provider: m.provider.clone(),
                        })
                        .collect(),
                    has_messages: !chat.messages.is_empty(),
                    has_response: chat
                        .messages
                        .iter()
                        .any(|m| m.position == crate::protocol::ChatMessagePosition::Left),
                }
            } else {
                logging::log(
                    "KEY",
                    "toggle_chat_actions called but current view is not ChatPrompt",
                );
                return;
            };

            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open actions as a separate window with vibrancy blur
            self.begin_actions_popup_window_open(cx, window);

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let is_mini = matches!(self.main_window_mode, MainWindowMode::Mini);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_chat(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &chat_info,
                    theme_arc,
                );

                // Mini mode: input at top, anchor top (collapses from bottom up)
                if is_mini {
                    dialog.set_config(crate::actions::ActionsDialogConfig {
                        search_position: crate::actions::SearchPosition::Top,
                        section_style: crate::actions::SectionStyle::Headers,
                        anchor: crate::actions::AnchorPosition::Top,
                        show_icons: true,
                        ..crate::actions::ActionsDialogConfig::default()
                    });
                }

                dialog.set_match_main_window_background(true);
                dialog
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_activation(Self::make_actions_dialog_activation_callback(
                    app_entity.clone(),
                    ActionsDialogHost::ChatPrompt,
                ));
                d.set_on_close(Self::make_actions_window_on_close_callback(
                    app_entity.clone(),
                    ActionsDialogHost::ChatPrompt,
                    "Chat actions closed via escape, focus restored via coordinator",
                ));
            });

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Chat actions: Main window bounds origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
                    main_bounds.origin.x, main_bounds.origin.y,
                    main_bounds.size.width, main_bounds.size.height,
                    display_id
                ),
            );

            let position = self.chat_actions_window_position();

            // Open the actions window via spawn
            Self::spawn_open_actions_window(
                cx,
                window.window_handle(),
                main_bounds,
                display_id,
                dialog,
                position,
                "Chat actions popup window opened",
                "Failed to open chat actions window",
            );

            logging::log("FOCUS", "Chat actions opened, keyboard routing active");
        }
        cx.notify();
    }
}

#[cfg(test)]
mod on_close_reentrancy_tests {
    use std::fs;

    #[test]
    fn test_actions_toggle_on_close_defers_script_list_app_updates() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");
        let impl_source = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("Expected implementation section before tests");

        let set_on_close_count = impl_source
            .matches("d.set_on_close(Self::make_actions_window_on_close_callback(")
            .count();
        let defer_count = impl_source.matches("cx.defer(move |cx| {").count();

        assert_eq!(
            set_on_close_count, 4,
            "actions_toggle should use the shared on_close callback factory at four popup-window call sites"
        );
        assert!(
            defer_count >= 1,
            "actions_toggle on_close callback factory should defer ScriptListApp updates"
        );
        assert!(
            impl_source.contains("if !app.show_actions_popup && app.actions_dialog.is_none()"),
            "actions_toggle on_close callbacks should guard already-closed popup state"
        );
    }

    #[test]
    fn test_toggle_actions_paths_resync_filter_input_state() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");
        let impl_source = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("Expected implementation section before tests");

        let pre_open_resync_count = impl_source
            .matches("self.resync_filter_input_after_actions_if_needed(window, cx);")
            .count();
        assert_eq!(
            pre_open_resync_count, 5,
            "all toggle_*_actions open paths should resync canonical filter input first"
        );

        let on_close_mark_count = impl_source
            .matches("app.mark_filter_resync_after_actions_if_needed();")
            .count();
        assert_eq!(
            on_close_mark_count, 1,
            "shared actions window on_close callback should mark filter resync for next render"
        );
    }

    #[test]
    fn test_actions_toggle_uses_shared_spawn_open_actions_window_helper() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");
        let impl_source = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("Expected implementation section before tests");

        let helper_call_count = impl_source
            .matches("Self::spawn_open_actions_window(")
            .count();
        assert_eq!(
            helper_call_count, 4,
            "toggle_actions, toggle_webcam_actions, toggle_chat_actions, and toggle_terminal_commands should use the shared spawn helper"
        );

        let open_call_count = impl_source.matches("match open_actions_window(").count();
        assert_eq!(
            open_call_count, 1,
            "open_actions_window match block should live only in spawn_open_actions_window helper"
        );

        let helper_start = impl_source
            .find("pub(crate) fn spawn_open_actions_window(")
            .expect("spawn_open_actions_window helper should exist");
        let helper_body = &impl_source[helper_start..];
        let helper_end = helper_body
            .find("/// Resolve the actions popup window position")
            .expect("helper should end before the next function");
        let helper_body = &helper_body[..helper_end];
        assert!(
            helper_body.contains("dialog.set_skip_track_focus(true);"),
            "spawn_open_actions_window should centralize detached popup focus ownership"
        );
    }

    #[test]
    fn test_begin_actions_popup_window_open_is_used_by_popup_window_toggles_only() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");
        let impl_source = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("Expected implementation section before tests");

        assert!(
            impl_source.contains(
                "fn begin_actions_popup_window_open(&mut self, cx: &mut Context<Self>, window: &mut Window) {"
            ),
            "actions_toggle should define begin_actions_popup_window_open helper"
        );

        let helper_call_count = impl_source
            .matches("self.begin_actions_popup_window_open(cx, window);")
            .count();
        assert_eq!(
            helper_call_count, 4,
            "toggle_actions, toggle_webcam_actions, toggle_chat_actions, and toggle_terminal_commands should call begin_actions_popup_window_open"
        );

        let toggle_actions_source = impl_source
            .split("pub(crate) fn toggle_actions")
            .nth(1)
            .and_then(|section| section.split("pub(crate) fn toggle_arg_actions").next())
            .expect("toggle_actions source section should exist");
        assert!(
            toggle_actions_source.contains("self.begin_actions_popup_window_open(cx, window);"),
            "toggle_actions should use begin_actions_popup_window_open"
        );

        let toggle_webcam_actions_source = impl_source
            .split("pub fn toggle_webcam_actions")
            .nth(1)
            .and_then(|section| section.split("pub fn toggle_terminal_commands").next())
            .expect("toggle_webcam_actions source section should exist");
        assert!(
            toggle_webcam_actions_source
                .contains("self.begin_actions_popup_window_open(cx, window);"),
            "toggle_webcam_actions should use begin_actions_popup_window_open"
        );

        let toggle_chat_actions_source = impl_source
            .split("pub fn toggle_chat_actions")
            .nth(1)
            .expect("toggle_chat_actions source section should exist");
        assert!(
            toggle_chat_actions_source
                .contains("self.begin_actions_popup_window_open(cx, window);"),
            "toggle_chat_actions should use begin_actions_popup_window_open"
        );

        let toggle_arg_actions_source = impl_source
            .split("pub(crate) fn toggle_arg_actions")
            .nth(1)
            .and_then(|section| section.split("pub fn toggle_webcam_actions").next())
            .expect("toggle_arg_actions source section should exist");
        assert!(
            !toggle_arg_actions_source.contains("self.begin_actions_popup_window_open(cx, window);"),
            "toggle_arg_actions should not use begin_actions_popup_window_open (inline dialog, not a window)"
        );

        // toggle_arg_actions must still follow the same state contract as window-based toggles
        assert!(
            toggle_arg_actions_source.contains("self.gpui_input_focused = false;"),
            "toggle_arg_actions must clear gpui_input_focused on open (same contract as begin_actions_popup_window_open)"
        );
        assert!(
            toggle_arg_actions_source.contains("cx.notify();"),
            "toggle_arg_actions must end with cx.notify() (same contract as other popup toggles)"
        );

        let toggle_terminal_commands_source = impl_source
            .split("pub fn toggle_terminal_commands")
            .nth(1)
            .and_then(|section| section.split("pub fn toggle_chat_actions").next())
            .expect("toggle_terminal_commands source section should exist");
        assert!(
            toggle_terminal_commands_source
                .contains("self.begin_actions_popup_window_open(cx, window);"),
            "toggle_terminal_commands should open a vibrancy popup window for native blur"
        );
    }
}

#[cfg(test)]
mod terminal_command_shortcut_tests {
    use super::*;
    use crate::actions::{AnchorPosition, SearchPosition, SectionStyle};
    use crate::designs::icon_variations::IconName;
    use std::fs;

    #[test]
    fn test_terminal_actions_for_dialog_shows_cmd_k_for_clear_terminal() {
        let clear_action = terminal_actions_for_dialog()
            .into_iter()
            .find(|action| action.id == TERM_PROMPT_CLEAR_ACTION_ID)
            .expect("clear action should exist in terminal actions");

        assert_eq!(
            clear_action.shortcut.as_deref(),
            Some(TERM_PROMPT_CLEAR_SHORTCUT)
        );
    }

    #[test]
    fn test_terminal_actions_for_dialog_adds_cmd_shift_k_toggle_shortcut() {
        let toggle_actions = terminal_actions_for_dialog()
            .into_iter()
            .find(|action| action.id == TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID)
            .expect("toggle actions entry should exist in terminal actions");

        assert_eq!(
            toggle_actions.shortcut.as_deref(),
            Some(TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT)
        );
    }

    #[test]
    fn test_terminal_actions_for_dialog_groups_sections_and_icons() {
        let actions = terminal_actions_for_dialog();

        let copy_action = actions
            .iter()
            .find(|action| action.id == "copy")
            .expect("copy action should exist");
        assert_eq!(copy_action.section.as_deref(), Some("Clipboard"));
        assert_eq!(copy_action.icon, Some(IconName::Copy));

        let find_action = actions
            .iter()
            .find(|action| action.id == "find")
            .expect("find action should exist");
        assert_eq!(find_action.section.as_deref(), Some("Search"));
        assert_eq!(find_action.icon, Some(IconName::MagnifyingGlass));

        let scroll_to_top_action = actions
            .iter()
            .find(|action| action.id == "scroll_to_top")
            .expect("scroll_to_top action should exist");
        assert_eq!(scroll_to_top_action.section.as_deref(), Some("Navigation"));
        assert_eq!(scroll_to_top_action.icon, Some(IconName::ArrowUp));

        let scroll_to_bottom_action = actions
            .iter()
            .find(|action| action.id == TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID)
            .expect("scroll_to_bottom action should exist");
        assert_eq!(
            scroll_to_bottom_action.section.as_deref(),
            Some("Navigation")
        );
        assert_eq!(scroll_to_bottom_action.icon, Some(IconName::ArrowDown));

        let clear_action = actions
            .iter()
            .find(|action| action.id == TERM_PROMPT_CLEAR_ACTION_ID)
            .expect("clear action should exist");
        assert_eq!(clear_action.section.as_deref(), Some("Session"));
        assert_eq!(clear_action.icon, Some(IconName::Trash));

        let reset_action = actions
            .iter()
            .find(|action| action.id == "reset")
            .expect("reset action should exist");
        assert_eq!(reset_action.section.as_deref(), Some("Session"));
        assert_eq!(reset_action.icon, Some(IconName::Refresh));
    }

    #[test]
    fn test_terminal_actions_dialog_config_enables_visual_features() {
        let config = terminal_actions_dialog_config();

        assert_eq!(config.search_position, SearchPosition::Top);
        assert_eq!(config.section_style, SectionStyle::Headers);
        assert_eq!(config.anchor, AnchorPosition::Top);
        assert!(config.show_icons);
        assert!(
            !config.show_footer,
            "Terminal actions should stay footerless because shortcuts are rendered inline"
        );
    }

    #[test]
    fn test_toggle_terminal_commands_sets_terminal_context_title() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");

        assert!(
            source.contains("d.set_context_title(Some(\"Terminal\".to_string()));"),
            "toggle_terminal_commands should set terminal context title"
        );
    }
}

#[cfg(test)]
mod root_file_action_tests {
    use super::*;
    use crate::file_search::{FileResult, FileType};

    fn root_file(file_type: FileType) -> FileResult {
        FileResult {
            path: "/Users/example/Desktop/fix spelling.png".to_string(),
            name: "fix spelling.png".to_string(),
            size: 0,
            modified: 0,
            file_type,
        }
    }

    #[test]
    fn root_file_actions_for_regular_file_has_minimal_four_rows() {
        let actions = root_file_actions_for(&root_file(FileType::Image));
        let titles = actions
            .iter()
            .map(|action| action.title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            titles,
            vec!["Open File", "Reveal in Finder", "Copy Path", "Quick Look"]
        );
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn root_file_actions_for_directory_labels_open_folder() {
        let actions = root_file_actions_for(&root_file(FileType::Directory));

        assert_eq!(actions[0].title, "Open Folder");
        assert_eq!(
            actions[0].id,
            crate::action_helpers::ROOT_FILE_OPEN_ACTION_ID
        );
    }

    #[test]
    fn root_file_action_ids_are_stable() {
        let actions = root_file_actions_for(&root_file(FileType::Image));
        let ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                crate::action_helpers::ROOT_FILE_OPEN_ACTION_ID,
                crate::action_helpers::ROOT_FILE_REVEAL_IN_FINDER_ACTION_ID,
                crate::action_helpers::ROOT_FILE_COPY_PATH_ACTION_ID,
                crate::action_helpers::ROOT_FILE_QUICK_LOOK_ACTION_ID,
            ]
        );
    }

    #[test]
    fn root_file_actions_do_not_include_deferred_file_search_actions() {
        let actions = root_file_actions_for(&root_file(FileType::Image));
        let ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();

        for deferred in [
            "open_with",
            "show_info",
            "attach_to_ai",
            "copy_filename",
            "move_to_trash",
            "duplicate_file",
            "copy_file",
            "file:open_with",
            "file:show_info",
            "file:attach_to_ai",
            "file:copy_filename",
            "file:move_to_trash",
            "file:duplicate_path",
        ] {
            assert!(
                !ids.contains(&deferred),
                "root file action palette should not include deferred action {deferred}"
            );
        }
    }
}
