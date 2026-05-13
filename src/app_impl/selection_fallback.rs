use super::*;

#[cfg(test)]
fn resolve_grouped_result_index(
    grouped_items: &[GroupedListItem],
    selected_index: usize,
) -> Option<(usize, usize)> {
    let coerced_index = crate::list_item::coerce_selection(grouped_items, selected_index)?;
    match grouped_items.get(coerced_index) {
        Some(GroupedListItem::Item(result_idx)) => Some((coerced_index, *result_idx)),
        _ => None,
    }
}

fn fallback_keeps_window_open(fallback: &crate::fallbacks::FallbackItem) -> bool {
    match fallback {
        crate::fallbacks::FallbackItem::Builtin(builtin) => matches!(
            builtin.id,
            "run-in-terminal"
                | crate::fallbacks::builtins::SEARCH_FILES_FALLBACK_ID
                | crate::fallbacks::builtins::DO_IN_CURRENT_APP_FALLBACK_ID
                | crate::fallbacks::builtins::SEND_TO_AI_FALLBACK_ID
        ),
        crate::fallbacks::FallbackItem::Script(_) => true,
    }
}

fn should_ignore_main_menu_open_carryover_input(
    current_view: &AppView,
    within_focus_grace_period: bool,
) -> bool {
    matches!(current_view, AppView::ScriptList) && within_focus_grace_period
}

impl ScriptListApp {
    fn should_ignore_selection_event_during_main_menu_open_guard(&self) -> bool {
        let within_focus_grace_period = script_kit_gpui::is_within_focus_grace_period();
        let should_ignore = should_ignore_main_menu_open_carryover_input(
            &self.current_view,
            within_focus_grace_period,
        );

        if should_ignore {
            tracing::info!(
                event = "main_menu_input_guard_blocked",
                selected_index = self.selected_index,
                fallback_mode = self.main_menu_fallback_state.is_active(),
                "Ignoring selection event during post-open main menu guard window"
            );
        }

        should_ignore
    }

    #[allow(dead_code)]
    pub(crate) fn filtered_scripts(&self) -> Vec<Arc<scripts::Script>> {
        let filter_text = self.filter_text();
        if filter_text.is_empty() {
            self.scripts.clone()
        } else {
            let filter_lower = filter_text.to_lowercase();
            self.scripts
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        }
    }

    /// Find a script or scriptlet by alias (case-insensitive exact match)
    /// Uses O(1) registry lookup instead of O(n) iteration
    pub(crate) fn find_alias_match(&self, alias: &str) -> Option<AliasMatch> {
        let alias_lower = alias.to_lowercase();

        // O(1) lookup in registry
        if let Some(command_id) = self.alias_registry.get(&alias_lower) {
            // Check for builtin/{id} command IDs
            if command_id.starts_with("builtin/") {
                let config = crate::config::BuiltInConfig::default();
                if let Some(entry) = builtins::get_builtin_entries(&config)
                    .into_iter()
                    .find(|e| e.id == *command_id)
                {
                    tracing::info!(
                        alias = %alias,
                        command_id = %command_id,
                        "alias_builtin_match_resolved"
                    );
                    return Some(AliasMatch::BuiltIn(std::sync::Arc::new(entry)));
                }
            }

            // Check for app/{bundle_id} command IDs
            if let Some(bundle_id) = command_id.strip_prefix("app/") {
                if let Some(app) = self
                    .apps
                    .iter()
                    .find(|a| a.bundle_id.as_deref() == Some(bundle_id))
                {
                    logging::log(
                        "ALIAS",
                        &format!("Found app match: '{}' -> '{}'", alias, app.name),
                    );
                    return Some(AliasMatch::App(std::sync::Arc::new(app.clone())));
                }
            }

            // Handle plugin-qualified script command IDs: "script/{plugin_id}:{name}"
            if let Some(identifier) = command_id.strip_prefix("script/") {
                let found = if let Some((plugin_id, name)) = identifier.split_once(':') {
                    self.scripts.iter().find(|s| {
                        s.name == name
                            && (s.plugin_id == plugin_id
                                || (s.plugin_id.is_empty()
                                    && s.kit_name.as_deref() == Some(plugin_id)))
                    })
                } else {
                    self.scripts.iter().find(|s| s.name == identifier)
                };
                if let Some(script) = found {
                    tracing::info!(
                        alias = %alias,
                        command_id = %command_id,
                        "alias_script_match_resolved"
                    );
                    return Some(AliasMatch::Script(script.clone()));
                }
            }

            // Handle plugin-qualified scriptlet command IDs: "scriptlet/{plugin_id}:{name}"
            if let Some(identifier) = command_id.strip_prefix("scriptlet/") {
                let found = if let Some((plugin_id, name)) = identifier.split_once(':') {
                    self.scriptlets.iter().find(|s| {
                        s.name == name
                            && (s.plugin_id == plugin_id
                                || (s.plugin_id.is_empty()
                                    && s.group.as_deref() == Some(plugin_id)))
                    })
                } else {
                    self.scriptlets.iter().find(|s| s.name == identifier)
                };
                if let Some(scriptlet) = found {
                    tracing::info!(
                        alias = %alias,
                        command_id = %command_id,
                        "alias_scriptlet_match_resolved"
                    );
                    return Some(AliasMatch::Scriptlet(scriptlet.clone()));
                }
            }

            // Legacy: find script/scriptlet by path (metadata-defined aliases store paths)
            for script in &self.scripts {
                if script.path.to_string_lossy() == *command_id {
                    logging::log(
                        "ALIAS",
                        &format!("Found script match: '{}' -> '{}'", alias, script.name),
                    );
                    return Some(AliasMatch::Script(script.clone()));
                }
            }

            // Legacy: check scriptlets by file_path or name
            for scriptlet in &self.scriptlets {
                let scriptlet_path = scriptlet.file_path.as_ref().unwrap_or(&scriptlet.name);
                if scriptlet_path == command_id {
                    logging::log(
                        "ALIAS",
                        &format!("Found scriptlet match: '{}' -> '{}'", alias, scriptlet.name),
                    );
                    return Some(AliasMatch::Scriptlet(scriptlet.clone()));
                }
            }

            // Command ID in registry but not found (stale entry)
            logging::log(
                "ALIAS",
                &format!(
                    "Stale registry entry: '{}' -> '{}' (not found)",
                    alias, command_id
                ),
            );
        }

        None
    }

    pub(crate) fn execute_selected(&mut self, cx: &mut Context<Self>) {
        if self.should_ignore_selection_event_during_main_menu_open_guard() {
            return;
        }

        if let Some(invocation) = self
            .menu_syntax_mode
            .command_for(&self.filter_text)
            .cloned()
        {
            self.execute_menu_syntax_command_invocation(invocation, cx);
            return;
        }

        if let Some(invocation) = self
            .menu_syntax_mode
            .capture_for(&self.filter_text)
            .cloned()
        {
            let mut handlers =
                crate::menu_syntax::rank_scripts_handling_capture(&self.scripts, &invocation);
            if let Some(script) = handlers.drain(..).next() {
                self.execute_menu_syntax_capture_script(script, invocation, cx);
            } else {
                self.show_hud(
                    format!("No capture handler for +{}", invocation.target),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
            }
            return;
        }

        if self
            .menu_syntax_mode
            .capture_composer_owns_input_for(&self.filter_text)
        {
            self.show_hud(
                "Type something to capture".to_string(),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        }

        if self
            .menu_syntax_mode
            .command_owns_input_for(&self.filter_text)
        {
            self.show_hud(
                "Choose a Script Kit command".to_string(),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        }

        let history_query = (!self.filter_text.trim().is_empty()).then(|| self.filter_text.clone());

        // Populate grouped results so the result-cache owner can resolve the
        // selected visual row into the flat result backing store.
        self.get_grouped_results_cached();

        if let Some((resolved_index, idx)) = self
            .main_menu_result_caches
            .flat_result_index_for_coerced_grouped_selection(self.selected_index)
        {
            if resolved_index != self.selected_index {
                self.selected_index = resolved_index;
                self.rebuild_main_window_preflight_if_needed();
            }

            let selected_result = self
                .main_menu_result_caches
                .cloned_search_result_for_flat_index(idx);
            if let Some(query) = history_query.as_deref() {
                self.input_history.add_entry_with_selection(
                    query,
                    selected_result
                        .as_ref()
                        .and_then(|result| result.history_result_key()),
                );
                if let Err(e) = self.input_history.save() {
                    tracing::warn!("Failed to save input history: {}", e);
                }
                self.invalidate_grouped_cache();
            }

            if let Some(formatted_value) = self
                .inline_calculator_for_result_index(idx)
                .map(|calculator| calculator.formatted.clone())
            {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(formatted_value.clone()));
                self.show_hud(
                    format!("Copied: {}", formatted_value),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
                self.close_and_reset_window(cx);
                return;
            }

            if self.is_in_attachment_portal() && matches!(self.current_view, AppView::ScriptList) {
                if let Some(part) =
                    self.build_attachment_portal_part_for_selected_script_list_result()
                {
                    self.close_attachment_portal_with_part(part, cx);
                }
                return;
            }

            if let Some(result) = selected_result {
                // Record frecency usage before executing (unless excluded).
                // Skills and scriptlets use plugin-qualified keys.
                let frecency_path: Option<String> = match &result {
                    scripts::SearchResult::Script(sm) => {
                        Some(sm.script.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::App(am) => {
                        Some(am.app.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::BuiltIn(bm) => {
                        // Skip frecency tracking for excluded builtins (e.g., "Quit Script Kit")
                        let excluded = &self.config.get_suggested().excluded_commands;
                        if bm.entry.should_exclude_from_frecency(excluded) {
                            None
                        } else {
                            Some(format!("builtin:{}", bm.entry.id))
                        }
                    }
                    scripts::SearchResult::Scriptlet(sm) => Some(format!(
                        "scriptlet:{}:{}",
                        sm.scriptlet.plugin_id, sm.scriptlet.name
                    )),
                    scripts::SearchResult::Skill(sm) => Some(format!(
                        "skill:{}:{}",
                        sm.skill.plugin_id, sm.skill.skill_id
                    )),
                    scripts::SearchResult::Window(wm) => {
                        Some(format!("window:{}:{}", wm.window.app, wm.window.title))
                    }
                    // Root file opens record frecency only after the OS open succeeds;
                    // execute_root_file_open is shared by Enter and the root-file Open action.
                    scripts::SearchResult::File(_) => None,
                    scripts::SearchResult::Note(_) => None,
                    scripts::SearchResult::AcpHistory(_) => None,
                    scripts::SearchResult::ClipboardHistory(_) => None,
                    scripts::SearchResult::DictationHistory(_) => None,
                    scripts::SearchResult::BrowserTab(_) => None,
                    scripts::SearchResult::BrowserHistory(_) => None,
                    // Suppressed: agents don't track frecency in the launcher
                    scripts::SearchResult::Agent(_) => None,
                    // Fallbacks don't track frecency - they're utility commands
                    scripts::SearchResult::Fallback(_) => None,
                    // Script Issues is a synthetic diagnostic row — no frecency.
                    scripts::SearchResult::ScriptIssue(_) => None,
                };
                if let Some(path) = frecency_path {
                    self.frecency_store.record_use(&path);
                    self.frecency_store.save().ok(); // Best-effort save
                    self.invalidate_grouped_cache(); // Invalidate cache so next show reflects frecency
                }

                // Log the action being performed using the same label path as the footer/preflight.
                let action_text = self.main_window_primary_action_label();
                logging::log(
                    "EXEC",
                    &format!(
                        "Action: '{}' on '{}' (type: {})",
                        action_text,
                        result.name(),
                        result.type_label()
                    ),
                );

                // Menu-syntax capture rows intercept the normal Script path.
                // `MenuSyntaxMode` is raw-guarded against the current filter
                // text, so a stale parse can never route a non-capture
                // selection into the capture pipeline.
                let capture_invocation = self
                    .menu_syntax_mode
                    .capture_for(&self.computed_filter_text)
                    .cloned();
                if let (Some(invocation), scripts::SearchResult::Script(script_match)) =
                    (capture_invocation, &result)
                {
                    let script = script_match.script.clone();
                    self.execute_menu_syntax_capture_script(script, invocation, cx);
                    return;
                }

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        // Run 13 Pass 2 (user bug report) — if the user
                        // selected a menu-syntax CAPTURE handler script
                        // directly from the main list (no `;target …`
                        // composer active), running it would crash on the
                        // missing `KIT_MENU_SYNTAX_PAYLOAD_PATH` env var.
                        // Pivot to the power-syntax composer by writing
                        // `;target ` into the filter so the user can
                        // complete the capture invocation in-place.
                        if let Some(target) =
                            crate::menu_syntax::first_concrete_capture_target_for_script(
                                &script_match.script,
                            )
                        {
                            let new_filter = format!(";{} ", target);
                            tracing::info!(
                                target: "script_kit::menu_syntax",
                                event = "script_list_pivot_to_power_syntax_composer",
                                script = %script_match.script.name,
                                pivot_filter = %new_filter,
                                "Pivoting main-list capture-handler launch into ;target composer"
                            );
                            self.filter_text = new_filter.clone();
                            self.computed_filter_text = new_filter.clone();
                            self.pending_filter_sync = true;
                            self.set_menu_syntax_mode_from_filter(&new_filter);
                            self.invalidate_grouped_cache();
                            cx.notify();
                            return;
                        }
                        // Run 13 Pass 4 — symmetric pivot for `command.v1`
                        // handlers: pivot into the `!head ` command composer
                        // instead of running the script process bare.
                        if let Some(head) =
                            crate::menu_syntax::first_command_head_for_script(&script_match.script)
                        {
                            let new_filter = format!("!{} ", head);
                            tracing::info!(
                                target: "script_kit::menu_syntax",
                                event = "script_list_pivot_to_command_composer",
                                script = %script_match.script.name,
                                pivot_filter = %new_filter,
                                "Pivoting main-list command-handler launch into !head composer"
                            );
                            self.filter_text = new_filter.clone();
                            self.computed_filter_text = new_filter.clone();
                            self.pending_filter_sync = true;
                            self.set_menu_syntax_mode_from_filter(&new_filter);
                            self.invalidate_grouped_cache();
                            cx.notify();
                            return;
                        }
                        self.execute_interactive(&script_match.script, cx);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        self.execute_scriptlet(&scriptlet_match.scriptlet, cx);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        self.execute_builtin(&builtin_match.entry, cx);
                    }
                    scripts::SearchResult::App(app_match) => {
                        self.execute_app(&app_match.app, cx);
                    }
                    scripts::SearchResult::Window(window_match) => {
                        self.execute_window_focus(&window_match.window, cx);
                    }
                    scripts::SearchResult::File(file_match) => {
                        self.execute_root_file_open(&file_match.file, cx);
                    }
                    scripts::SearchResult::Note(note_match) => {
                        self.execute_root_note_open(note_match.hit.id, cx);
                    }
                    scripts::SearchResult::AcpHistory(acp_history_match) => {
                        self.resume_acp_conversation_from_history(
                            &acp_history_match.entry.session_id,
                            acp_history_match.entry.first_message.as_str(),
                            cx,
                        );
                    }
                    scripts::SearchResult::ClipboardHistory(clipboard_match) => {
                        self.execute_root_clipboard_history_paste(&clipboard_match.entry.id, cx);
                    }
                    scripts::SearchResult::DictationHistory(dictation_match) => {
                        self.execute_root_dictation_history_paste(&dictation_match.id, cx);
                    }
                    scripts::SearchResult::BrowserTab(browser_tab_match) => {
                        self.execute_root_browser_tab_switch(&browser_tab_match.hit, cx);
                    }
                    scripts::SearchResult::BrowserHistory(browser_match) => {
                        self.execute_root_browser_history_open(&browser_match.hit.url, cx);
                    }
                    scripts::SearchResult::Skill(skill_match) => {
                        // Skills always open Agent Chat with the selected skill staged
                        let owner = if skill_match.skill.plugin_title.is_empty() {
                            skill_match.skill.plugin_id.as_str()
                        } else {
                            skill_match.skill.plugin_title.as_str()
                        };
                        tracing::info!(
                            event = "acp_skill_launch_requested",
                            plugin_id = %skill_match.skill.plugin_id,
                            skill_id = %skill_match.skill.skill_id,
                            path = %skill_match.skill.path.display(),
                            owner,
                            "Skill selected from main menu"
                        );
                        self.show_hud(
                            format!("Opening {} \u{b7} {}", owner, skill_match.skill.title),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                        tracing::info!(
                            event = "acp_skill_launch_hud_shown",
                            plugin_id = %skill_match.skill.plugin_id,
                            skill_id = %skill_match.skill.skill_id,
                            owner,
                            "Displayed ACP skill launch HUD"
                        );
                        self.open_acp_with_selected_skill(&skill_match.skill, cx);
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        // Suppressed: agents are not launchable from the main menu.
                        // Skills replace agents as the first-class reusable AI artifact.
                        // ACP agent catalog/provider selection remains in src/ai/acp/.
                        tracing::info!(
                            event = "legacy_agent_result_suppressed",
                            agent_name = %agent_match.agent.name,
                            agent_path = %agent_match.agent.path.display(),
                            "Agent execution suppressed in main menu - use skills or Agent Chat"
                        );
                    }
                    scripts::SearchResult::Fallback(fallback_match) => {
                        // Execute the fallback with the current filter text as input
                        self.execute_fallback_item(&fallback_match.fallback, cx);
                    }
                    scripts::SearchResult::ScriptIssue(_) => {
                        self.open_script_issues_view(cx);
                    }
                }
            }
        }
    }

    pub(crate) fn selected_main_list_search_result_owned(
        &mut self,
    ) -> Option<scripts::SearchResult> {
        if !matches!(self.current_view, AppView::ScriptList) {
            return None;
        }

        self.get_grouped_results_cached();
        let (resolved_index, result_idx) = self
            .main_menu_result_caches
            .flat_result_index_for_coerced_grouped_selection(self.selected_index)?;
        if resolved_index != self.selected_index {
            self.selected_index = resolved_index;
            self.rebuild_main_window_preflight_if_needed();
        }

        if self
            .inline_calculator_for_result_index(result_idx)
            .is_some()
        {
            return None;
        }

        self.main_menu_result_caches
            .cloned_search_result_for_flat_index(result_idx)
    }

    pub(crate) fn selected_root_file_result_owned(
        &mut self,
    ) -> Option<crate::file_search::FileResult> {
        match self.selected_main_list_search_result_owned()? {
            scripts::SearchResult::File(file_match) => Some(file_match.file),
            _ => None,
        }
    }

    pub(crate) fn execute_root_note_open(
        &mut self,
        note_id: crate::notes::NoteId,
        cx: &mut Context<Self>,
    ) {
        match crate::notes::open_note_in_notes_window(cx, note_id) {
            Ok(()) => self.hide_main_and_reset(cx),
            Err(error) => {
                logging::log("ERROR", &format!("Failed to open root note: {error}"));
                self.show_hud("Failed to open note".to_string(), Some(HUD_MEDIUM_MS), cx);
            }
        }
    }

    pub(crate) fn selected_root_directory_query_owned(&mut self) -> Option<String> {
        let file = self.selected_root_file_result_owned()?;
        Self::root_file_search_in_folder_query(&file)
    }

    fn record_root_file_open_use(&mut self, file: &crate::file_search::FileResult) {
        self.frecency_store
            .record_use(&format!("file/{}", file.path));
        if let Err(error) = self.frecency_store.save() {
            tracing::warn!(
                path = %file.path,
                error = %error,
                "Failed to save root file frecency after open"
            );
        }
        self.invalidate_grouped_cache();
    }

    pub(crate) fn execute_root_file_open(
        &mut self,
        file: &crate::file_search::FileResult,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = crate::file_search::open_file(&file.path) {
            logging::log(
                "ROOT_FILE_SEARCH",
                &format!("failed_to_open path={} error={}", file.path, error),
            );
            self.show_hud(
                format!("Failed to open {}", file.name),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        }
        self.record_root_file_open_use(file);
        self.close_and_reset_window(cx);
    }

    pub(crate) fn execute_root_clipboard_history_paste(
        &mut self,
        entry_id: &str,
        cx: &mut Context<Self>,
    ) {
        match crate::clipboard_history::copy_entry_to_clipboard(entry_id) {
            Ok(()) => {
                logging::log("EXEC", &format!("Root clipboard entry copied: {entry_id}"));
                self.hide_main_and_reset(cx);
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if let Err(error) = crate::selected_text::simulate_paste_with_cg() {
                        logging::log(
                            "ERROR",
                            &format!("Failed to simulate root clipboard paste: {error}"),
                        );
                    } else {
                        logging::log("EXEC", "Simulated root clipboard paste");
                    }
                });
            }
            Err(error) => {
                logging::log(
                    "ERROR",
                    &format!("Failed to copy root clipboard entry: {error}"),
                );
                self.show_hud(
                    "Failed to paste clipboard entry".to_string(),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
            }
        }
    }

    pub(crate) fn execute_root_dictation_history_paste(
        &mut self,
        entry_id: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = crate::dictation::get_history_entry(entry_id) else {
            logging::log(
                "ERROR",
                &format!("Root dictation history entry not found: {entry_id}"),
            );
            self.show_hud(
                "Failed to paste dictation".to_string(),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        };

        let transcript = entry.transcript;
        self.hide_main_and_reset(cx);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let injector = crate::text_injector::TextInjector::new();
            if let Err(error) = injector.paste_text(&transcript) {
                logging::log(
                    "ERROR",
                    &format!("Failed to paste root dictation history: {error}"),
                );
            } else {
                logging::log("EXEC", "Pasted root dictation history transcript");
            }
        });
    }

    pub(crate) fn execute_root_browser_history_open(
        &mut self,
        url: &str,
        cx: &mut Context<Self>,
    ) {
        match crate::browser_history::open_browser_history_url(url) {
            Ok(()) => {
                logging::log("EXEC", &format!("Opened root browser history URL: {url}"));
                self.hide_main_and_reset(cx);
            }
            Err(error) => {
                logging::log(
                    "ERROR",
                    &format!("Failed to open root browser history URL: {error}"),
                );
                self.show_hud(
                    "Failed to open browser history page".to_string(),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
            }
        }
    }

    pub(crate) fn execute_root_browser_tab_switch(
        &mut self,
        hit: &crate::browser_tabs::RootBrowserTabSearchHit,
        cx: &mut Context<Self>,
    ) {
        match crate::browser_tabs::focus_root_browser_tab(hit) {
            Ok(()) => {
                logging::log("EXEC", &format!("Focused root browser tab: {}", hit.title));
                self.hide_main_and_reset(cx);
            }
            Err(error) => {
                logging::log("ERROR", &format!("Failed to focus root browser tab: {error}"));
                self.show_hud(
                    "Failed to switch browser tab".to_string(),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
            }
        }
    }

    pub(crate) fn root_file_search_in_folder_query(
        file: &crate::file_search::FileResult,
    ) -> Option<String> {
        (file.file_type == crate::file_search::FileType::Directory)
            .then(|| crate::file_search::ensure_trailing_slash(&file.path))
    }

    pub(crate) fn root_file_browse_parent_folder_query(
        file: &crate::file_search::FileResult,
    ) -> Option<String> {
        if file.file_type == crate::file_search::FileType::Directory {
            return None;
        }
        crate::file_search::parent_folder_search_query(&file.path)
    }

    pub(crate) fn root_file_parent_query_for_filter(filter_text: &str) -> Option<String> {
        if !crate::file_search::looks_like_root_directory_browse_query(filter_text) {
            return None;
        }

        let parsed = crate::file_search::parse_directory_path(filter_text)?;
        if parsed.filter.is_some() {
            Some(parsed.directory)
        } else {
            crate::file_search::parent_dir_display(&parsed.directory)
        }
    }

    fn clear_main_list_selection_for_root_file_handoff(&mut self) {
        self.selected_index = 0;
        self.main_list_state
            .scroll_to_reveal_item(self.selected_index);
        self.last_scrolled_index = Some(self.selected_index);
    }

    pub(crate) fn try_navigate_root_file_directory_with_tab(
        &mut self,
        has_shift: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::ScriptList) || self.show_actions_popup {
            return false;
        }

        let next_query = if has_shift {
            Self::root_file_parent_query_for_filter(&self.filter_text)
        } else {
            self.selected_root_directory_query_owned()
        };

        let Some(next_query) = next_query else {
            return false;
        };

        self.set_filter_text_immediate(next_query, window, cx);
        true
    }

    pub(crate) fn execute_root_file_action(
        &mut self,
        action_id: &str,
        file: &crate::file_search::FileResult,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match action_id {
            crate::action_helpers::ROOT_FILE_OPEN_ACTION_ID => {
                self.pending_root_file_actions_file = None;
                self.execute_root_file_open(file, cx);
                true
            }
            crate::action_helpers::ROOT_FILE_REVEAL_IN_FINDER_ACTION_ID => {
                if let Err(error) = crate::file_search::reveal_in_finder(&file.path) {
                    logging::log(
                        "ROOT_FILE_SEARCH",
                        &format!("failed_to_reveal path={} error={}", file.path, error),
                    );
                    self.show_hud(
                        format!("Failed to reveal {}", file.name),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return true;
                }
                self.pending_root_file_actions_file = None;
                self.close_and_reset_window(cx);
                true
            }
            crate::action_helpers::ROOT_FILE_COPY_PATH_ACTION_ID => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(file.path.clone()));
                self.show_hud(
                    format!("Copied path: {}", file.name),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
                true
            }
            crate::action_helpers::ROOT_FILE_COPY_NAME_ACTION_ID => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(file.name.clone()));
                self.show_hud(
                    format!("Copied name: {}", file.name),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
                true
            }
            crate::action_helpers::ROOT_FILE_QUICK_LOOK_ACTION_ID => {
                match crate::file_search::quick_look(&file.path) {
                    Ok(()) => {
                        self.show_hud(
                            format!("Previewing {}", file.name),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    Err(error) => {
                        logging::log(
                            "ROOT_FILE_SEARCH",
                            &format!("failed_to_quick_look path={} error={}", file.path, error),
                        );
                        self.show_hud(
                            format!("Failed to preview {}: {}", file.name, error),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                }
                true
            }
            crate::action_helpers::ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID => {
                let Some(query) = Self::root_file_search_in_folder_query(file) else {
                    self.show_hud(
                        format!("Not a folder: {}", file.name),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return true;
                };
                self.pending_root_file_actions_file = None;
                self.open_file_search(query, cx);
                true
            }
            crate::action_helpers::ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID => {
                let Some(query) = Self::root_file_browse_parent_folder_query(file) else {
                    self.show_hud(
                        format!("No parent folder for {}", file.name),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return true;
                };
                self.pending_root_file_actions_file = None;
                self.clear_main_list_selection_for_root_file_handoff();
                self.open_file_search(query, cx);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn try_execute_root_file_action_shortcut(
        &mut self,
        key_lower: &str,
        has_cmd: bool,
        has_shift: bool,
        has_alt: bool,
        has_ctrl: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::ScriptList)
            || self.show_actions_popup
            || crate::actions::is_actions_window_open()
        {
            return false;
        }
        if !has_cmd || has_alt || has_ctrl {
            return false;
        }

        let action_id = match (key_lower, has_shift) {
            ("y", false) => crate::action_helpers::ROOT_FILE_QUICK_LOOK_ACTION_ID,
            ("c", true) => crate::action_helpers::ROOT_FILE_COPY_PATH_ACTION_ID,
            ("f", true) => crate::action_helpers::ROOT_FILE_REVEAL_IN_FINDER_ACTION_ID,
            _ => return false,
        };

        let Some(file) = self.selected_root_file_result_owned() else {
            return false;
        };
        self.execute_root_file_action(action_id, &file, window, cx)
    }

    /// Execute a fallback item (from the "Use with..." section in search results)
    /// This is called when a fallback is selected from the grouped list
    pub fn execute_fallback_item(
        &mut self,
        fallback: &crate::fallbacks::FallbackItem,
        cx: &mut Context<Self>,
    ) {
        let input = self.filter_text.clone();

        logging::log(
            "EXEC",
            &format!(
                "Executing fallback item: {} with input: '{}'",
                fallback.display_name(),
                input
            ),
        );

        let should_close = !fallback_keeps_window_open(fallback);

        // Execute the fallback action
        match fallback {
            crate::fallbacks::FallbackItem::Builtin(builtin) => {
                let fallback_id = builtin.id.to_string();
                self.execute_builtin_fallback_inline(&fallback_id, &input, cx);
            }
            crate::fallbacks::FallbackItem::Script(config) => {
                self.execute_interactive(&config.script, cx);
            }
        }

        // Close the window after executing (unless it's a stay-open action)
        if should_close {
            self.close_and_reset_window(cx);
        }
    }

    /// Execute the currently selected fallback command
    /// This is called from keyboard handler, so we need to defer window access
    pub fn execute_selected_fallback(&mut self, cx: &mut Context<Self>) {
        if self.should_ignore_selection_event_during_main_menu_open_guard() {
            return;
        }

        if let Some(scripts::SearchResult::Fallback(fallback_match)) =
            self.selected_main_list_search_result_owned()
        {
            logging::log(
                "EXEC",
                &format!(
                    "Executing selected grouped fallback: {}",
                    fallback_match.display_name()
                ),
            );
            self.execute_fallback_item(&fallback_match.fallback, cx);
            return;
        }

        let input = self.filter_text.clone();
        if let Some(fallback) = self.main_menu_fallback_state.selected_item().cloned() {
            logging::log(
                "EXEC",
                &format!("Executing fallback: {}", fallback.display_name()),
            );

            let should_close = !fallback_keeps_window_open(&fallback);

            // Execute the fallback action
            match &fallback {
                crate::fallbacks::FallbackItem::Builtin(builtin) => {
                    let fallback_id = builtin.id.to_string();
                    self.execute_builtin_fallback_inline(&fallback_id, &input, cx);
                }
                crate::fallbacks::FallbackItem::Script(config) => {
                    self.execute_interactive(&config.script, cx);
                }
            }

            // Close the window after executing (unless it's a stay-open action)
            if should_close {
                self.close_and_reset_window(cx);
            }
        }
    }

    /// Execute a built-in fallback action without window reference
    pub(crate) fn execute_builtin_fallback_inline(
        &mut self,
        fallback_id: &str,
        input: &str,
        cx: &mut Context<Self>,
    ) {
        use crate::fallbacks::builtins::{get_builtin_fallbacks, FallbackResult};

        logging::log(
            "FALLBACK",
            &format!("Executing fallback '{}' with input: {}", fallback_id, input),
        );

        // Find the fallback by ID
        let fallbacks = get_builtin_fallbacks();
        let fallback = fallbacks.iter().find(|f| f.id == fallback_id);

        let Some(fallback) = fallback else {
            logging::log("FALLBACK", &format!("Unknown fallback ID: {}", fallback_id));
            return;
        };

        // Execute the fallback and get the result
        match fallback.execute(input) {
            Ok(result) => match result {
                FallbackResult::RunTerminal { command } => {
                    logging::log("FALLBACK", &format!("RunTerminal: {}", command));
                    // Open the built-in terminal with the command
                    self.open_terminal_with_command(command, cx);
                }
                FallbackResult::AddNote { content } => {
                    logging::log("FALLBACK", &format!("AddNote: {}", content));
                    let item = gpui::ClipboardItem::new_string(content);
                    cx.write_to_clipboard(item);
                    if let Err(e) = crate::notes::open_notes_window(cx) {
                        logging::log("FALLBACK", &format!("Failed to open Notes: {}", e));
                    }
                }
                FallbackResult::Copy { text } => {
                    logging::log("FALLBACK", &format!("Copy: {} chars", text.len()));
                    let item = gpui::ClipboardItem::new_string(text);
                    cx.write_to_clipboard(item);
                    crate::hud_manager::show_hud(
                        "Copied to clipboard".to_string(),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                }
                FallbackResult::OpenUrl { url } => {
                    logging::log("FALLBACK", &format!("OpenUrl: {}", url));
                    let _ = open::that(&url);
                }
                FallbackResult::Calculate { expression } => {
                    // Evaluate the expression using meval
                    logging::log("FALLBACK", &format!("Calculate: {}", expression));
                    match meval::eval_str(&expression) {
                        Ok(result) => {
                            let item = gpui::ClipboardItem::new_string(result.to_string());
                            cx.write_to_clipboard(item);
                            crate::hud_manager::show_hud(
                                format!("{} = {}", expression, result),
                                Some(HUD_LONG_MS),
                                cx,
                            );
                        }
                        Err(e) => {
                            logging::log("FALLBACK", &format!("Calculate error: {}", e));
                            let message = calculate_fallback_error_message(&expression);
                            crate::hud_manager::show_hud(message, Some(HUD_LONG_MS), cx);
                        }
                    }
                }
                FallbackResult::OpenFile { path } => {
                    logging::log("FALLBACK", &format!("OpenFile: {}", path));
                    let expanded = if path.starts_with("~") {
                        if let Some(home) = dirs::home_dir() {
                            path.replacen("~", &home.to_string_lossy(), 1)
                        } else {
                            path.clone()
                        }
                    } else {
                        path.clone()
                    };
                    let _ = open::that(&expanded);
                }
                FallbackResult::SearchFiles { query } => {
                    logging::log("FALLBACK", &format!("SearchFiles: {}", query));
                    self.open_file_search(query, cx);
                }
                FallbackResult::ExecuteBuiltin { builtin_id } => {
                    logging::log(
                        "FALLBACK",
                        &format!(
                            "ExecuteBuiltin: builtin_id='{}' input_len={}",
                            builtin_id,
                            input.len()
                        ),
                    );

                    let builtin_entry = self
                        .builtin_entries
                        .iter()
                        .find(|entry| entry.id == builtin_id)
                        .cloned();

                    let Some(entry) = builtin_entry else {
                        logging::log(
                            "FALLBACK",
                            &format!(
                                "state=failed attempted=execute_builtin_fallback reason=builtin_not_found builtin_id={}",
                                builtin_id
                            ),
                        );
                        return;
                    };

                    self.execute_builtin_with_query(&entry, Some(input), cx);
                }
                FallbackResult::SendToAiHarness { query } => {
                    logging::log("FALLBACK", &format!("SendToAiHarness: {}", query));
                    let normalized = query.trim().to_string();
                    let intent = if normalized.is_empty() {
                        None
                    } else {
                        Some(normalized)
                    };
                    self.open_tab_ai_acp_with_entry_intent(intent, cx);
                }
            },
            Err(e) => {
                logging::log("FALLBACK", &format!("Fallback execution error: {}", e));
            }
        }
    }

    /// Enter action for the synthetic "Script Issues" pinned row. Snapshots the
    /// current [`ValidationReport`] into the new [`AppView::ScriptIssuesView`]
    /// so authors can read fatal issues and related colliders, then Escape back
    /// to the launcher or Cmd+C to copy diagnostics to the clipboard.
    pub(crate) fn open_script_issues_view(&mut self, cx: &mut Context<Self>) {
        let Some(report) = self.script_validation_report.clone() else {
            crate::hud_manager::show_hud(
                "No script validation report available".to_string(),
                Some(2500),
                cx,
            );
            return;
        };

        tracing::info!(
            event = "script_issues_row_activated",
            failed_count = report.failed_scripts.len(),
            fatal_count = report.fatal_count,
            warning_count = report.warning_count,
            "Script Issues row activated"
        );

        self.current_view = AppView::ScriptIssuesView { report };
        self.request_script_list_main_filter_focus(cx);
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_grouped_result_index_coerces_section_header_selection() {
        let grouped_items = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(3),
            GroupedListItem::Item(4),
        ];

        assert_eq!(
            resolve_grouped_result_index(&grouped_items, 0),
            Some((1, 3))
        );
    }

    #[test]
    fn test_resolve_grouped_result_index_clamps_out_of_bounds_selection() {
        let grouped_items = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(8),
            GroupedListItem::SectionHeader("Main".to_string(), None),
        ];

        assert_eq!(
            resolve_grouped_result_index(&grouped_items, 100),
            Some((1, 8))
        );
    }

    #[test]
    fn test_resolve_grouped_result_index_returns_none_for_header_only_rows() {
        let grouped_items = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::SectionHeader("Main".to_string(), None),
        ];

        assert_eq!(resolve_grouped_result_index(&grouped_items, 0), None);
    }

    #[test]
    fn test_fallback_keeps_window_open_for_send_to_ai() {
        let fallback = crate::fallbacks::FallbackItem::Builtin(
            crate::fallbacks::builtins::get_builtin_fallbacks()
                .into_iter()
                .find(|fallback| fallback.id == crate::fallbacks::builtins::SEND_TO_AI_FALLBACK_ID)
                .expect("send-to-ai fallback should exist"),
        );

        assert!(fallback_keeps_window_open(&fallback));
    }

    #[test]
    fn test_fallback_keeps_window_open_for_do_in_current_app() {
        let fallback = crate::fallbacks::FallbackItem::Builtin(
            crate::fallbacks::builtins::get_builtin_fallbacks()
                .into_iter()
                .find(|fallback| {
                    fallback.id == crate::fallbacks::builtins::DO_IN_CURRENT_APP_FALLBACK_ID
                })
                .expect("do-in-current-app fallback should exist"),
        );

        assert!(fallback_keeps_window_open(&fallback));
    }

    #[test]
    fn test_fallback_keeps_window_open_is_false_for_regular_builtin() {
        let fallback = crate::fallbacks::FallbackItem::Builtin(
            crate::fallbacks::builtins::get_builtin_fallbacks()
                .into_iter()
                .find(|fallback| fallback.id == "search-google")
                .expect("search-google fallback should exist"),
        );

        assert!(!fallback_keeps_window_open(&fallback));
    }

    fn root_file_result(
        path: &str,
        file_type: crate::file_search::FileType,
    ) -> crate::file_search::FileResult {
        crate::file_search::FileResult {
            path: path.to_string(),
            name: std::path::Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(path)
                .to_string(),
            size: 0,
            modified: 0,
            file_type,
        }
    }

    #[test]
    fn root_file_search_in_folder_query_accepts_directories() {
        let file = root_file_result(
            "/tmp/example-folder",
            crate::file_search::FileType::Directory,
        );

        assert_eq!(
            ScriptListApp::root_file_search_in_folder_query(&file),
            Some("/tmp/example-folder/".to_string())
        );
    }

    #[test]
    fn root_file_search_in_folder_query_rejects_regular_files() {
        let file = root_file_result("/tmp/example.txt", crate::file_search::FileType::Document);

        assert_eq!(ScriptListApp::root_file_search_in_folder_query(&file), None);
    }

    #[test]
    fn root_file_browse_parent_folder_query_accepts_regular_files() {
        let file = root_file_result(
            "/tmp/example-folder/readme.md",
            crate::file_search::FileType::Document,
        );

        assert_eq!(
            ScriptListApp::root_file_browse_parent_folder_query(&file),
            Some("/tmp/example-folder/".to_string())
        );
    }

    #[test]
    fn root_file_browse_parent_folder_query_rejects_directories() {
        let file = root_file_result(
            "/tmp/example-folder",
            crate::file_search::FileType::Directory,
        );

        assert_eq!(
            ScriptListApp::root_file_browse_parent_folder_query(&file),
            None
        );
    }

    #[test]
    fn root_file_parent_query_for_filter_accepts_directory_browse_queries() {
        let base = std::env::temp_dir().join(format!(
            "script-kit-root-parent-test-{}",
            std::process::id()
        ));
        let nested = base.join("example-folder");
        std::fs::create_dir_all(&nested).expect("create nested temp directory");
        let nested_query = format!("{}/", nested.display());

        assert_eq!(
            ScriptListApp::root_file_parent_query_for_filter(&nested_query),
            Some(format!("{}/", base.display()))
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn root_file_parent_query_for_filter_clears_child_fragment_first() {
        let base = std::env::temp_dir().join(format!(
            "script-kit-root-filter-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&base).expect("create temp directory");
        let filtered_query = format!("{}/al", base.display());

        assert_eq!(
            ScriptListApp::root_file_parent_query_for_filter(&filtered_query),
            Some(format!("{}/", base.display()))
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn root_file_parent_query_for_filter_rejects_plain_search_queries() {
        assert_eq!(
            ScriptListApp::root_file_parent_query_for_filter("fix"),
            None
        );
    }
}
