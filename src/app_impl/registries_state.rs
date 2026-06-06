use super::*;

impl ScriptListApp {
    pub(crate) fn rebuild_registries(&mut self) -> Vec<String> {
        let mut conflicts = Vec::new();
        self.alias_registry.clear();
        self.shortcut_registry.clear();

        // Register script aliases
        for script in &self.scripts {
            if let Some(ref alias) = script.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias,
                            script.path.display(),
                            existing_path
                        ),
                    );
                } else {
                    self.alias_registry
                        .insert(alias_lower, script.path.to_string_lossy().to_string());
                }
            }
        }

        // Register scriptlet aliases
        for scriptlet in &self.scriptlets {
            if let Some(ref alias) = scriptlet.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.alias_registry.insert(alias_lower, path);
                }
            }

            // Register scriptlet shortcuts
            if let Some(ref shortcut) = scriptlet.shortcut {
                let shortcut_lower = shortcut.to_lowercase();
                if let Some(existing_path) = self.shortcut_registry.get(&shortcut_lower) {
                    conflicts.push(format!(
                        "Shortcut conflict: '{}' already used by {}",
                        shortcut,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "Conflict: shortcut '{}' in {} blocked (already used by {})",
                            shortcut, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.shortcut_registry.insert(shortcut_lower, path);
                }
            }
        }

        // Load alias overrides from ~/.scriptkit/aliases.json
        // These provide aliases for built-ins, apps, and other commands
        // that don't have their own alias metadata
        if let Ok(alias_overrides) = crate::aliases::load_alias_overrides() {
            for (command_id, alias) in alias_overrides {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' for {} blocked (already used by {})",
                            alias, command_id, existing_path
                        ),
                    );
                } else {
                    // Use the command_id as the path identifier
                    // This allows find_alias_match to find built-ins and apps
                    self.alias_registry.insert(alias_lower, command_id);
                }
            }
        }

        logging::log(
            "REGISTRY",
            &format!(
                "Rebuilt registries: {} aliases, {} shortcuts, {} conflicts",
                self.alias_registry.len(),
                self.shortcut_registry.len(),
                conflicts.len()
            ),
        );

        conflicts
    }

    /// Reset all state and return to the script list view.
    /// This clears all prompt state and resizes the window appropriately.
    pub(crate) fn reset_to_script_list(&mut self, cx: &mut Context<Self>) {
        clear_main_state_restore_after_focus_loss();
        // Any detached floating popups (ACP @ picker / history popup /
        // menu-syntax trigger popup) are owned by the
        // outgoing view. Returning to the script list abandons that owner,
        // so make sure none of them survive past the transition.
        self.close_floating_popups_for_owner_loss("reset_to_script_list", cx);
        let closing_acp_chat = matches!(self.current_view, AppView::AcpChatView { .. });
        let old_view = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::About { .. } => "About",
            AppView::ActionsDialog => "ActionsDialog",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::HotkeyPrompt { .. } => "HotkeyPrompt",
            AppView::ChatPrompt { .. } => "ChatPrompt",
            AppView::MiniPrompt { .. } => "MiniPrompt",
            AppView::MicroPrompt { .. } => "MicroPrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistoryView",
            AppView::EmojiPickerView { .. } => "EmojiPickerView",
            AppView::AppLauncherView { .. } => "AppLauncherView",
            AppView::WindowSwitcherView { .. } => "WindowSwitcherView",
            AppView::BrowserTabsView { .. } => "BrowserTabsView",
            AppView::DesignGalleryView { .. } => "DesignGalleryView",
            AppView::FooterGalleryView { .. } => "FooterGalleryView",
            AppView::NonListStatesView { .. } => "NonListStatesView",
            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => "DesignExplorerView",
            AppView::ScratchPadView { .. } => "ScratchPadView",
            AppView::QuickTerminalView { .. } => "QuickTerminalView",
            AppView::WebcamView { .. } => "WebcamView",
            AppView::FileSearchView { .. } => "FileSearchView",
            AppView::ProfileSearchView { .. } => "ProfileSearchView",
            AppView::ThemeChooserView { .. } => "ThemeChooserView",
            AppView::CreationFeedback { .. } => "CreationFeedback",
            AppView::NamingPrompt { .. } => "NamingPrompt",
            AppView::BrowseKitsView { .. } => "BrowseKitsView",
            AppView::InstalledKitsView { .. } => "InstalledKitsView",
            AppView::ProcessManagerView { .. } => "ProcessManagerView",
            AppView::CurrentAppCommandsView { .. } => "CurrentAppCommandsView",
            AppView::SearchAiPresetsView { .. } => "SearchAiPresetsView",
            AppView::CreateAiPresetView { .. } => "CreateAiPresetView",
            AppView::SettingsView { .. } => "SettingsView",
            AppView::FavoritesBrowseView { .. } => "FavoritesBrowseView",
            AppView::AcpHistoryView { .. } => "AcpHistoryView",
            AppView::BrowserHistoryView { .. } => "BrowserHistoryView",
            AppView::DictationHistoryView { .. } => "DictationHistoryView",
            AppView::NotesBrowseView { .. } => "NotesBrowseView",
            AppView::AcpChatView { .. } => "AcpChatView",
            AppView::ScriptIssuesView { .. } => "ScriptIssuesView",
            AppView::SdkReferenceView { .. } => "SdkReferenceView",
            AppView::ScriptTemplateCatalogView { .. } => "ScriptTemplateCatalogView",
            AppView::ConfirmPrompt { .. } => "ConfirmPrompt",
        };

        let old_focused_input = self.focused_input;
        logging::log(
            "UI",
            &format!(
                "Resetting to script list (was: {}, focused_input: {:?})",
                old_view, old_focused_input
            ),
        );

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This runs BEFORE clearing channels to ensure cleanup even if Drop doesn't fire
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {} during reset", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Stop process manager refresh if it was running
        self.stop_process_manager_refresh();

        // If reset bypasses the normal ACP close button/Escape route, still
        // hide ACP-owned popups before the view is dropped. This covers
        // window hide/reset paths after launcher-triggered "/" or "@" opens.
        if let AppView::AcpChatView { entity } = &self.current_view {
            self.embedded_acp_chat = Some(entity.clone());
            entity.update(cx, |view, cx| {
                view.prepare_for_host_hide(cx);
            });
        }

        // Reset view
        self.current_view = AppView::ScriptList;
        self.set_main_window_mode_state_only(MainWindowMode::Mini, cx, "reset_to_script_list");

        if closing_acp_chat {
            self.acp_ready_script_path = None;
            self.acp_footer_dot_status = None;
            self.acp_footer_model_display = None;
            self.acp_footer_snapshot = None;
            self.tab_ai_harness_return_view = None;
            self.tab_ai_harness_return_focus_target = None;
            crate::windows::update_automation_semantic_surface(
                "main",
                Some("scriptList".to_string()),
            );
            crate::windows::ensure_embedded_ai_window(false);
            self.transition_acp_surface(
                crate::ai::acp::surface_state::AcpSurfaceEvent::EmbeddedClosed,
            );
        }

        // CRITICAL: Reset focused_input to MainFilter so the cursor appears
        // This was a bug where focused_input could remain as ArgPrompt/None after
        // script exit, causing the cursor to not show in the main filter.
        self.gpui_input_focused = false;
        self.request_script_list_main_filter_focus(cx);
        // Reset placeholder back to the root launcher discovery copy.
        self.pending_placeholder =
            Some(crate::dev_style_tool::runtime_overrides::effective_main_input_placeholder());
        logging::log(
            "FOCUS",
            "Reset focused_input to MainFilter for cursor display",
        );

        // Clear arg prompt state
        self.arg_input.clear();
        self.arg_selected_index = 0;
        // P0: Reset arg scroll handle
        self.arg_list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);

        // Clear filter and selection state for fresh menu
        self.reset_script_list_filter_and_selection_state(cx);

        // Clear favorites filter
        self.active_favorites = None;

        // NOTE: Window resize is NOT done here to avoid RefCell borrow conflicts.
        // Callers that need resize should use deferred resize via window_ops::queue_resize
        // after the update closure completes. The show_main_window_helper handles this
        // for the visibility flow. Other callers rely on next render to resize.

        // Clear output
        self.last_output = None;

        // Clear channels (they will be dropped, closing the connections)
        self.prompt_receiver = None;
        self.response_sender = self.default_response_sender.clone();

        // Clear script session (parking_lot mutex never poisons)
        *self.script_session.lock() = None;

        // Clear actions popup state (prevents stale actions dialog from persisting)
        self.clear_actions_popup_state();

        // Clear pending path action and close signal
        if let Ok(mut guard) = self.pending_path_action.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.close_path_actions.lock() {
            *guard = false;
        }

        logging::log(
            "UI",
            "State reset complete - view is now ScriptList (filter, selection, scroll cleared)",
        );
        cx.notify();
    }

    /// Ensure the selection is at the first selectable item.
    ///
    /// This is a lightweight method that only resets the selection position,
    /// without clearing the filter or other state. Call this when showing
    /// the main menu to ensure the user always starts at the top.
    ///
    /// FIX: Resolves bug where main menu sometimes opened with a random item
    /// selected instead of the first item (e.g., "Reset Window Positions"
    /// instead of "ACP Chat").
    pub fn ensure_selection_at_first_item(&mut self, cx: &mut Context<Self>) {
        // Only reset selection if we're in the script list view
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }

        // Invalidate cache to ensure fresh data
        self.invalidate_grouped_cache();
        self.sync_list_state();

        // Reset selection to first item
        self.selected_index = 0;
        self.hovered_index = None; // Reset hover state to prevent stale highlight on reopen
        self.validate_selection_bounds(cx);

        // Scroll to top
        self.main_list_state.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        self.last_scrolled_index = Some(self.selected_index);

        logging::log(
            "UI",
            &format!(
                "Selection reset to first item: selected_index={}, hovered_index=None",
                self.selected_index
            ),
        );
        cx.notify();
    }
}
