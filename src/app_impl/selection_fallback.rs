use super::*;

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

impl ScriptListApp {
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
            if let Some(builtin_id) = command_id.strip_prefix("builtin/") {
                let config = crate::config::BuiltInConfig::default();
                if let Some(entry) = builtins::get_builtin_entries(&config)
                    .into_iter()
                    .find(|e| e.id == builtin_id)
                {
                    logging::log(
                        "ALIAS",
                        &format!("Found builtin match: '{}' -> '{}'", alias, entry.name),
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

            // Find the script/scriptlet by path
            for script in &self.scripts {
                if script.path.to_string_lossy() == *command_id {
                    logging::log(
                        "ALIAS",
                        &format!("Found script match: '{}' -> '{}'", alias, script.name),
                    );
                    return Some(AliasMatch::Script(script.clone()));
                }
            }

            // Check scriptlets by file_path or name
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
        // Record input to history if filter has meaningful text
        if !self.filter_text.trim().is_empty() {
            self.input_history.add_entry(&self.filter_text);
            if let Err(e) = self.input_history.save() {
                tracing::warn!("Failed to save input history: {}", e);
            }
        }

        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        if let Some((resolved_index, idx)) =
            resolve_grouped_result_index(&grouped_items, self.selected_index)
        {
            if resolved_index != self.selected_index {
                self.selected_index = resolved_index;
            }
            if let Some(result) = flat_results.get(idx).cloned() {
                // Record frecency usage before executing (unless excluded)
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
                            Some(format!("builtin:{}", bm.entry.name))
                        }
                    }
                    scripts::SearchResult::Scriptlet(sm) => {
                        Some(format!("scriptlet:{}", sm.scriptlet.name))
                    }
                    scripts::SearchResult::Window(wm) => {
                        Some(format!("window:{}:{}", wm.window.app, wm.window.title))
                    }
                    scripts::SearchResult::Agent(am) => {
                        Some(format!("agent:{}", am.agent.path.to_string_lossy()))
                    }
                    // Fallbacks don't track frecency - they're utility commands
                    scripts::SearchResult::Fallback(_) => None,
                };
                if let Some(path) = frecency_path {
                    self.frecency_store.record_use(&path);
                    self.frecency_store.save().ok(); // Best-effort save
                    self.invalidate_grouped_cache(); // Invalidate cache so next show reflects frecency
                }

                // Log the action being performed (matches button text from get_default_action_text())
                let action_text = result.get_default_action_text();
                logging::log(
                    "EXEC",
                    &format!(
                        "Action: '{}' on '{}' (type: {})",
                        action_text,
                        result.name(),
                        result.type_label()
                    ),
                );

                match result {
                    scripts::SearchResult::Script(script_match) => {
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
                    scripts::SearchResult::Agent(agent_match) => {
                        // TODO: Implement agent execution via mdflow
                        self.last_output = Some(SharedString::from(format!(
                            "Agent execution not yet implemented: {}",
                            agent_match.agent.name
                        )));
                    }
                    scripts::SearchResult::Fallback(fallback_match) => {
                        // Execute the fallback with the current filter text as input
                        self.execute_fallback_item(&fallback_match.fallback, cx);
                    }
                }
            }
        }
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
                fallback.name(),
                input
            ),
        );

        // Check if this is a "stay open" action (like run-in-terminal which opens a view)
        // Check if this is a "stay open" action (opens its own view)
        let should_close = match fallback {
            crate::fallbacks::FallbackItem::Builtin(builtin) => {
                !matches!(builtin.id, "run-in-terminal" | "search-files")
            }
            crate::fallbacks::FallbackItem::Script(_) => false,
        };

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
        if !self.fallback_mode || self.cached_fallbacks.is_empty() {
            return;
        }

        let input = self.filter_text.clone();
        if let Some(fallback) = self
            .cached_fallbacks
            .get(self.fallback_selected_index)
            .cloned()
        {
            logging::log("EXEC", &format!("Executing fallback: {}", fallback.name()));

            // Check if this is a "stay open" action (opens its own view)
            let should_close = match &fallback {
                crate::fallbacks::FallbackItem::Builtin(builtin) => {
                    !matches!(builtin.id, "run-in-terminal" | "search-files")
                }
                crate::fallbacks::FallbackItem::Script(_) => false,
            };

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
                    crate::hud_manager::show_hud("Copied to clipboard".to_string(), Some(1500), cx);
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
                                Some(3000),
                                cx,
                            );
                        }
                        Err(e) => {
                            logging::log("FALLBACK", &format!("Calculate error: {}", e));
                            let message = calculate_fallback_error_message(&expression);
                            crate::hud_manager::show_hud(message, Some(3000), cx);
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
            },
            Err(e) => {
                logging::log("FALLBACK", &format!("Fallback execution error: {}", e));
            }
        }
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

        assert_eq!(resolve_grouped_result_index(&grouped_items, 0), Some((1, 3)));
    }

    #[test]
    fn test_resolve_grouped_result_index_clamps_out_of_bounds_selection() {
        let grouped_items = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(8),
            GroupedListItem::SectionHeader("Main".to_string(), None),
        ];

        assert_eq!(resolve_grouped_result_index(&grouped_items, 100), Some((1, 8)));
    }

    #[test]
    fn test_resolve_grouped_result_index_returns_none_for_header_only_rows() {
        let grouped_items = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::SectionHeader("Main".to_string(), None),
        ];

        assert_eq!(resolve_grouped_result_index(&grouped_items, 0), None);
    }
}
