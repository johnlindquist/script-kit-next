use super::*;

pub(crate) const TERM_PROMPT_CLEAR_ACTION_ID: &str = "clear";
// ⌘K means "Actions" on every surface of the app; the terminal is no
// exception (audit finding #24). Clear moved to ⌘⇧K so scrollback can't be
// destroyed by the app-wide Actions chord.
pub(crate) const TERM_PROMPT_CLEAR_SHORTCUT: &str = "⌘⇧K";
pub(crate) const TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID: &str = "term_prompt_toggle_actions";
pub(crate) const TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT: &str = "⌘K";
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
    let parent_folder_query = if !is_dir {
        crate::file_search::parent_folder_search_query(&file.path)
    } else {
        None
    };

    let mut actions = vec![Action::new(
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
    .with_section("Actions")];

    if is_dir {
        actions.push(
            Action::new(
                crate::action_helpers::ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID,
                "Search Inside Folder",
                Some(format!(
                    "Searches {} in File Search",
                    crate::file_search::shorten_path(&file.path)
                )),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::FolderOpen)
            .with_section("Actions"),
        );
    } else if let Some(parent_query) = parent_folder_query {
        actions.push(
            Action::new(
                crate::action_helpers::ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID,
                "Browse Parent Folder",
                Some(format!(
                    "Opens {} in File Search",
                    crate::file_search::shorten_path(&parent_query)
                )),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::FolderOpen)
            .with_section("Actions"),
        );
    }

    actions.extend([
        Action::new(
            crate::action_helpers::ROOT_FILE_COPY_FILE_ACTION_ID,
            "Copy File",
            Some("Copies the file itself, ready to paste into Finder, Mail or a chat".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}C")
        .with_icon(IconName::Copy)
        .with_section("Share"),
        Action::new(
            crate::action_helpers::ROOT_FILE_PASTE_FILE_ACTION_ID,
            "Paste File",
            Some("Pastes the file into the app you were using".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{21e7}V")
        .with_icon(IconName::Copy)
        .with_section("Share"),
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
            crate::action_helpers::ROOT_FILE_COPY_NAME_ACTION_ID,
            "Copy Name",
            Some("Copies only the file or folder name to the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{2325}C")
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
    ]);

    actions
}

fn actions_dialog_host_label(host: &ActionsDialogHost) -> &'static str {
    match host {
        ActionsDialogHost::MainList => "MainList",
        ActionsDialogHost::ClipboardHistory => "ClipboardHistory",
        ActionsDialogHost::Favorites => "Favorites",
        ActionsDialogHost::ThemeChooser => "ThemeChooser",
        ActionsDialogHost::EmojiPicker => "EmojiPicker",
        ActionsDialogHost::FileSearch => "FileSearch",
        ActionsDialogHost::PathPrompt => "PathPrompt",
        ActionsDialogHost::ChatPrompt => "ChatPrompt",
        ActionsDialogHost::ArgPrompt => "ArgPrompt",
        ActionsDialogHost::DivPrompt => "DivPrompt",
        ActionsDialogHost::EditorPrompt => "EditorPrompt",
        ActionsDialogHost::TemplatePrompt => "TemplatePrompt",
        ActionsDialogHost::TermPrompt => "TermPrompt",
        ActionsDialogHost::FormPrompt => "FormPrompt",
        ActionsDialogHost::WebcamPrompt => "WebcamPrompt",
        ActionsDialogHost::AppLauncher => "AppLauncher",
        ActionsDialogHost::BuiltinList => "BuiltinList",
        ActionsDialogHost::FlowDesk => "FlowDesk",
    }
}

fn actions_context_text_fingerprint(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

impl ScriptListApp {
    pub(crate) fn build_actions_host_context_snapshot(
        &mut self,
        host: ActionsDialogHost,
        parent_automation_id: Option<String>,
    ) -> crate::actions::ActionsHostContextSnapshot {
        let parent_automation_id = parent_automation_id
            .or_else(|| matches!(host, ActionsDialogHost::MainList).then(|| "main".to_string()));
        let parent_info = parent_automation_id
            .as_deref()
            .and_then(crate::windows::automation_window_by_id);
        let parent_kind = parent_info.as_ref().map(|info| format!("{:?}", info.kind));
        let parent_semantic_surface = parent_info.and_then(|info| info.semantic_surface);

        let mut parent_subject_id = None;
        let mut selected_semantic_id = None;
        if matches!(host, ActionsDialogHost::MainList) {
            parent_subject_id = self
                .pending_root_unified_actions_subject
                .as_ref()
                .and_then(|subject| subject.stable_key())
                .or_else(|| {
                    self.pending_root_file_actions_file
                        .as_ref()
                        .map(|file| format!("root-file/{}", file.path))
                });

            let (grouped_items, flat_results) = self.cached_grouped_results_snapshot();
            let selected_grouped =
                crate::list_item::coerce_selection(&grouped_items, self.selected_index);
            let mut row_index = 0usize;
            for (grouped_index, item) in grouped_items.iter().enumerate() {
                match item {
                    crate::list_item::GroupedListItem::SectionHeader(..) => {}
                    crate::list_item::GroupedListItem::Item(result_idx) => {
                        if Some(grouped_index) == selected_grouped {
                            if let Some(result) = flat_results.get(*result_idx) {
                                parent_subject_id =
                                    parent_subject_id.or_else(|| result.stable_selection_key());
                                let label = Self::script_list_result_label(result);
                                selected_semantic_id = Some(crate::protocol::generate_semantic_id(
                                    "choice", row_index, &label,
                                ));
                            }
                            break;
                        }
                        row_index += 1;
                    }
                    crate::list_item::GroupedListItem::Status(_) => {
                        row_index += 1;
                    }
                }
            }
        }

        let parent_subject_text_fingerprint = parent_subject_id
            .as_deref()
            .map(actions_context_text_fingerprint);

        crate::actions::ActionsHostContextSnapshot {
            host: actions_dialog_host_label(&host).to_string(),
            parent_automation_id,
            parent_kind,
            parent_semantic_surface,
            parent_subject_id,
            parent_subject_text_fingerprint,
            selected_semantic_id,
        }
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
            "Dispatching shared actions toggle for current view"
        );

        if matches!(&self.current_view, AppView::ScriptList) {
            return self.one_window_owns_list() && self.one_window_stage(cx);
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

        if matches!(&self.current_view, AppView::TermPrompt { .. }) {
            let mode = term_prompt_actions_mode(self.has_nonempty_sdk_actions());
            self.toggle_term_prompt_actions(mode, cx, window);
            return true;
        }

        if Self::is_script_prompt_view(&self.current_view) && self.has_nonempty_sdk_actions() {
            self.toggle_arg_actions(cx, window);
            return true;
        }

        if matches!(&self.current_view, AppView::TemplatePrompt { .. }) {
            self.toggle_actions(cx, window);
            return true;
        }

        if matches!(&self.current_view, AppView::ClipboardHistoryView { .. }) {
            if let Some(entry) = self.selected_clipboard_entry() {
                self.toggle_clipboard_actions(entry, window, cx);
                return true;
            }
            if self.show_actions_popup {
                self.toggle_actions(cx, window);
                return true;
            }
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_clipboard_selection",
                trigger = trigger,
                "Ignored shared actions toggle because clipboard history has no selected entry"
            );
            // The footer advertises ⌘K — a dead press must explain itself.
            if let Some(reason) = self.actions_toggle_dead_without_selection_reason() {
                self.one_window_notice(crate::line_notice::INFO, reason.to_string(), cx);
            }
            return false;
        }

        if matches!(&self.current_view, AppView::FavoritesBrowseView { .. }) {
            if self.selected_favorite_id().is_some() || self.show_actions_popup {
                self.toggle_favorites_actions(window, cx);
                return true;
            }
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_favorite_selection",
                trigger = trigger,
                "Ignored shared actions toggle because favorites has no selected item"
            );
            if let Some(reason) = self.actions_toggle_dead_without_selection_reason() {
                self.one_window_notice(crate::line_notice::INFO, reason.to_string(), cx);
            }
            return false;
        }

        if matches!(&self.current_view, AppView::ThemeChooserView { .. }) {
            self.toggle_theme_chooser_actions(window, cx);
            return true;
        }

        if matches!(
            &self.current_view,
            AppView::FlowUxView { .. } | AppView::FlowSessionView { .. }
        ) {
            self.toggle_flow_desk_actions(window, cx);
            return true;
        }

        if let AppView::PathPrompt { entity, .. } = &self.current_view {
            if self.show_actions_popup {
                self.close_actions_popup(ActionsDialogHost::PathPrompt, window, cx);
            } else {
                let entity = entity.clone();
                entity.update(cx, |prompt, cx| {
                    prompt.toggle_actions(cx);
                });
            }
            return true;
        }

        // Generic fallback for any remaining view that advertises SharedDialog
        // support via actions_support_for_view() but doesn't need a dedicated
        // branch with selection-specific context (e.g. DivPrompt, EditorPrompt,
        // FormPrompt, EmojiPicker, AgentChatHistory).
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
        let can_toggle = self.has_actions() || self.show_actions_popup;

        if !can_toggle {
            tracing::info!(
                target: "script_kit::actions",
                event = "actions_toggle_dispatch_ignored_no_actions",
                trigger = trigger,
                view = ?self.current_view,
                show_actions_popup = self.show_actions_popup,
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
            "Routing actions toggle through canonical view host"
        );

        if self.show_actions_popup {
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
            // stage with only global rows, but generic BuiltinList surfaces are
            // filtered out before this path so Theme Chooser cannot advertise
            // misleading launcher actions.
            let _ = self.has_actions();

            // Defensive guard for any explicit BuiltinList host path: its
            // selected_index belongs to that built-in list, not the script list
            // cache read by `get_focused_script_info`.
            let on_builtin_list = matches!(host, ActionsDialogHost::BuiltinList);
            // Day Page: the launcher's selected_index is stale background
            // state, so the dialog must not surface script-row actions there.
            // The Today host section below carries the contextual rows.
            let day_page_section: Option<Vec<crate::actions::Action>> =
                if let AppView::DayPage { ref entity } = self.current_view {
                    Some(day_page_host_actions_section(entity.read(cx), cx))
                } else {
                    None
                };
            // `selected_index` and the filter text describe the launcher's own
            // list; in every other view they are background state, and script
            // rows resolve their target from that list when run.
            let on_launcher =
                matches!(self.current_view, AppView::ScriptList) && !on_builtin_list;
            let script_info = if on_launcher {
                self.get_focused_script_info()
            } else {
                None
            };

            // Get the full scriptlet with actions if focused item is a scriptlet
            let focused_scriptlet = if on_launcher {
                self.get_focused_scriptlet_with_actions()
            } else {
                None
            };

            // Run 12 Pass 7 — compute the Power Syntax section from the live
            // filter parse + active mode BEFORE the dialog-construction closure
            // (can't borrow `self` inside `cx.new`). Returns None when not
            // composing a Power Syntax expression so the dialog falls back to
            // the legacy script_context + global rows.
            let power_syntax_section_for_dialog: Option<
                crate::menu_syntax_actions::PowerSyntaxActionSection,
            > = if !on_launcher {
                // The launcher filter text is background state while another
                // view owns the main window; its parse must not leak rows.
                None
            } else {
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

            let brain_inbox_route = self.brain_inbox_actions_route();
            // Create the dialog entity HERE in main app (for keyboard routing)
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let is_mini = matches!(self.main_window_mode, MainWindowMode::Mini);
            // Create the dialog entity
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_script(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}),
                    script_info.clone(),
                    theme_arc,
                );

                if let Some(route) = &brain_inbox_route {
                    dialog.prepend_root_route_actions(vec![crate::actions::Action::new(
                        "brain_inbox_all",
                        "Open Brain Inbox",
                        Some(format!(
                            "{} open items · browse and search the full inbox",
                            route.actions.len()
                        )),
                        crate::actions::ActionCategory::ScriptContext,
                    )]);
                    dialog.register_drill_down_route("brain_inbox_all", route.clone());
                }

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
                        search_position: crate::actions::SearchPosition::Top,
                        section_style: crate::actions::SectionStyle::Headers,
                        anchor: crate::actions::AnchorPosition::Bottom,
                        search_placeholder: script_info.as_ref().map(|script| script.name.clone()),
                        show_context_header: false,
                        ..crate::actions::ActionsDialogConfig::default()
                    });
                }

                // If we have a scriptlet with actions, pass it to the dialog.
                if let Some(ref scriptlet) = focused_scriptlet {
                    dialog.set_focused_scriptlet(script_info.clone(), Some(scriptlet.clone()));
                }

                // Run 12 Pass 7 — wire the cmdk-actions Power Syntax section.
                // The owned section was computed BEFORE entering this closure
                // (can't borrow `self` inside `cx.new`); push it now.
                dialog.set_menu_syntax_section(power_syntax_section_for_dialog.clone());
                dialog.set_host_section(day_page_section.clone());

                // Skip track_focus so the parent window keeps keyboard routing
                // (matches command_bar.rs pattern)
                dialog.set_skip_track_focus(true);
                dialog.set_match_main_window_background(true);
                dialog
            });

            let parent_automation_id = crate::windows::focused_automation_window_id();
            let host_context =
                self.build_actions_host_context_snapshot(host, parent_automation_id.clone());
            dialog.update(cx, |d, _cx| {
                d.set_host_context(host_context);
            });

            let title = script_info
                .as_ref()
                .map(|script| script.name.clone())
                .or_else(|| self.script_prompt_staged_subject_title());
            self.one_window_stage_in_view(host, dialog, title, window, cx);
        }
        cx.notify();
    }

    /// Resolve the persisted Agent Chat profile/model into footer display
    /// labels. Returns `(profile_display_name, model_display_name, provider_id)`.
    pub(crate) fn resolve_agent_model_footer_labels(
    ) -> (Option<String>, Option<String>, Option<String>) {
        let prefs = crate::config::load_user_preferences();
        let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
        let profile = crate::ai::agent_chat::profiles::resolve_effective_profile(&prefs.ai, &ctx);
        let profile_label = Some(profile.name.clone());
        let provider_id = profile.provider.clone();

        let Some(model_id) = profile.model.as_deref() else {
            return (profile_label, None, provider_id);
        };

        let model_label = profile.provider.as_deref().and_then(|provider_id| {
            crate::ai::agent_chat::profiles::provider_model_catalog()
                .iter()
                .find(|entry| entry.id == provider_id)
                .and_then(|entry| {
                    entry
                        .models
                        .iter()
                        .find(|(mid, _)| *mid == model_id)
                        .map(|(_, display)| display.to_string())
                })
        });

        (
            profile_label,
            Some(model_label.unwrap_or_else(|| model_id.to_string())),
            provider_id,
        )
    }

    /// Refresh the cached footer agent/model labels from persisted preferences.
    /// Called after the Profile Switcher persists a new selection.
    pub(crate) fn refresh_agent_model_footer_labels(&mut self) {
        let (agent_label, model_label, provider_id) = Self::resolve_agent_model_footer_labels();
        self.spine_agent_label = agent_label;
        self.spine_model_label = model_label;
        self.spine_provider_id = provider_id;
    }

    /// ⌘K on an arg prompt with SDK actions: the highlighted choice (or the typed value) is
    /// staged and the prompt's list becomes its actions.
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
        if self.show_actions_popup {
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

                    let title = self.arg_prompt_staged_subject_title();
                    self.one_window_stage_in_view(
                        ActionsDialogHost::ArgPrompt,
                        dialog,
                        title,
                        window,
                        cx,
                    );
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
    /// A script's own prompt: its ⌘K acts on that script, never the launcher row.
    fn is_script_prompt_view(view: &AppView) -> bool {
        matches!(
            view,
            AppView::DivPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::TemplatePrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::DropPrompt { .. }
        )
    }

    /// What a script prompt's staged actions act on: the script, else the prompt itself.
    fn script_prompt_staged_subject_title(&self) -> Option<String> {
        let prompt = match &self.current_view {
            AppView::DivPrompt { .. } => "Output",
            AppView::EditorPrompt { .. } => "Editor",
            AppView::FormPrompt { .. } => "Form",
            AppView::TemplatePrompt { .. } => "Template",
            AppView::TermPrompt { .. } => "Terminal",
            AppView::DropPrompt { .. } => "Dropped files",
            _ => return None,
        };
        Some(
            self.current_script_subject
                .as_ref()
                .map_or_else(|| prompt.to_string(), |script| script.name.clone()),
        )
    }

    /// What ⌘K on an arg prompt acts on: the highlighted choice, else what's typed.
    fn arg_prompt_staged_subject_title(&self) -> Option<String> {
        if !matches!(self.current_view, AppView::ArgPrompt { .. }) {
            return self.script_prompt_staged_subject_title();
        }
        self.filtered_arg_choices()
            .get(self.arg_selected_index)
            .map(|(_, choice)| choice.name.clone())
            .or_else(|| {
                let typed = self.arg_input.text();
                (!typed.trim().is_empty()).then(|| typed.to_string())
            })
    }

    /// ⌘K on the webcam prompt (built-in command).
    /// Opens as a separate window (same pattern as toggle_chat_actions).
    pub fn toggle_webcam_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{ActionsDialog, ActionsDialogConfig};

        logging::log(
            "KEY",
            &format!(
                "toggle_webcam_actions called: show_actions_popup={}",
                self.show_actions_popup
            ),
        );

        if self.show_actions_popup {
            // Close — delegate to central close_actions_popup
            self.close_actions_popup(ActionsDialogHost::WebcamPrompt, window, cx);
        } else {
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let webcam_actions = self.webcam_actions_for_dialog(cx);

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

            self.one_window_stage_in_view(
                ActionsDialogHost::WebcamPrompt,
                dialog,
                Some("Webcam".to_string()),
                window,
                cx,
            );

            logging::log("FOCUS", "Webcam actions staged, keyboard routing active");
        }
        cx.notify();
    }

    /// ⌘K on the built-in terminal: its commands (Clear, Copy, Paste, Scroll, etc.) in place.
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

        if self.show_actions_popup {
            self.close_actions_popup(ActionsDialogHost::TermPrompt, window, cx);
        } else {
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

            self.one_window_stage_in_view(
                ActionsDialogHost::TermPrompt,
                dialog,
                Some("Terminal".to_string()),
                window,
                cx,
            );

            logging::log("FOCUS", "Terminal actions staged in place");
        }
        cx.notify();
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

        if self.show_actions_popup {
            self.close_actions_popup(ActionsDialogHost::ChatPrompt, window, cx);
        } else {
            // Get chat info from current ChatPrompt entity
            let (chat_info, chat_command_bindings) =
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    let chat = entity.read(cx);
                    (
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
                        },
                        chat.conversation_command_bindings(),
                    )
                } else {
                    logging::log(
                        "KEY",
                        "toggle_chat_actions called but current view is not ChatPrompt",
                    );
                    return;
                };

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let is_mini = matches!(self.main_window_mode, MainWindowMode::Mini);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_chat_commands(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &chat_info,
                    chat_command_bindings,
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

            let title = chat_info.current_model.clone();
            self.one_window_stage_in_view(ActionsDialogHost::ChatPrompt, dialog, title, window, cx);

            logging::log("FOCUS", "Chat actions staged, keyboard routing active");
        }
        cx.notify();
    }
}

#[cfg(test)]
include!("actions_toggle_tests.rs");
