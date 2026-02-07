impl ScriptListApp {
    fn get_focused_script_info(&mut self) -> Option<ScriptInfo> {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the minimum score threshold for suggested items
        let min_score = self.config.get_suggested().min_score;

        // Get the result index from the grouped item
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx) {
                // Compute frecency path for each result type (same logic as app_impl.rs)
                let frecency_path: Option<String> = match result {
                    scripts::SearchResult::Script(m) => {
                        Some(m.script.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::Scriptlet(m) => {
                        Some(format!("scriptlet:{}", m.scriptlet.name))
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Check if excluded from frecency tracking
                        let excluded = &self.config.get_suggested().excluded_commands;
                        if m.entry.should_exclude_from_frecency(excluded) {
                            None
                        } else {
                            Some(format!("builtin:{}", m.entry.name))
                        }
                    }
                    scripts::SearchResult::App(m) => Some(m.app.path.to_string_lossy().to_string()),
                    scripts::SearchResult::Window(m) => {
                        Some(format!("window:{}:{}", m.window.app, m.window.title))
                    }
                    scripts::SearchResult::Agent(m) => {
                        Some(format!("agent:{}", m.agent.path.to_string_lossy()))
                    }
                    scripts::SearchResult::Fallback(_) => None, // Fallbacks don't track frecency
                };

                // Check if this item is "suggested" (has frecency data above min_score)
                let is_suggested = frecency_path
                    .as_ref()
                    .map(|path| self.frecency_store.get_score(path) >= min_score)
                    .unwrap_or(false);

                match result {
                    scripts::SearchResult::Script(m) => Some(
                        ScriptInfo::with_shortcut_and_alias(
                            &m.script.name,
                            m.script.path.to_string_lossy(),
                            m.script.shortcut.clone(),
                            m.script.alias.clone(),
                        )
                        .with_frecency(is_suggested, frecency_path),
                    ),
                    scripts::SearchResult::Scriptlet(m) => {
                        // Scriptlets use the markdown file path for edit/reveal actions
                        // Extract the path without anchor for file operations
                        let markdown_path = m
                            .scriptlet
                            .file_path
                            .as_ref()
                            .map(|p| p.split('#').next().unwrap_or(p).to_string())
                            .unwrap_or_else(|| format!("scriptlet:{}", &m.scriptlet.name));
                        Some(
                            ScriptInfo::scriptlet(
                                &m.scriptlet.name,
                                markdown_path,
                                m.scriptlet.shortcut.clone(),
                                m.scriptlet.alias.clone(),
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Built-ins use their id as identifier
                        // is_script=false: no editable file, hide "Edit Script" etc.
                        // Look up shortcut and alias from overrides for dynamic action menu
                        // Uses cached versions to avoid file I/O on every render
                        let command_id = format!("builtin/{}", &m.entry.id);
                        let shortcut_overrides = crate::shortcuts::get_cached_shortcut_overrides();
                        let alias_overrides = crate::aliases::get_cached_alias_overrides();
                        let shortcut = shortcut_overrides.get(&command_id).map(|s| s.to_string());
                        let alias = alias_overrides.get(&command_id).cloned();
                        Some(
                            ScriptInfo::with_all(
                                &m.entry.name,
                                format!("builtin:{}", &m.entry.id),
                                false,
                                "Run",
                                shortcut,
                                alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::App(m) => {
                        // Apps use their path as identifier
                        // is_script=false: apps aren't editable scripts
                        // Look up shortcut and alias from overrides for dynamic action menu
                        // Uses cached versions to avoid file I/O on every render
                        let command_id = if let Some(ref bundle_id) = m.app.bundle_id {
                            format!("app/{}", bundle_id)
                        } else {
                            format!("app/{}", m.app.name.to_lowercase().replace(' ', "-"))
                        };
                        let shortcut_overrides = crate::shortcuts::get_cached_shortcut_overrides();
                        let alias_overrides = crate::aliases::get_cached_alias_overrides();
                        let shortcut = shortcut_overrides.get(&command_id).map(|s| s.to_string());
                        let alias = alias_overrides.get(&command_id).cloned();
                        Some(
                            ScriptInfo::with_all(
                                &m.app.name,
                                m.app.path.to_string_lossy().to_string(),
                                false,
                                "Launch",
                                shortcut,
                                alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Window(m) => {
                        // Windows use their id as identifier
                        // is_script=false: windows aren't editable scripts
                        Some(
                            ScriptInfo::with_action_verb(
                                &m.window.title,
                                format!("window:{}", m.window.id),
                                false,
                                "Switch to",
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Agent(m) => {
                        // Agents use their path as identifier
                        Some(
                            ScriptInfo::new(
                                &m.agent.name,
                                format!("agent:{}", m.agent.path.to_string_lossy()),
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Fallback(m) => {
                        // Fallbacks use their name as identifier
                        // is_script depends on whether it's a built-in fallback or script-based
                        // Fallbacks don't track frecency, so is_suggested is always false
                        Some(ScriptInfo::with_action_verb(
                            m.fallback.name(),
                            format!("fallback:{}", m.fallback.name()),
                            !m.fallback.is_builtin(),
                            "Run",
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the full scriptlet with actions for the currently focused item
    ///
    /// This re-parses the markdown file to get the scriptlet's H3 actions
    /// and shared actions from the companion .actions.md file.
    /// Returns None if the focused item is not a scriptlet.
    pub fn get_focused_scriptlet_with_actions(&mut self) -> Option<crate::scriptlets::Scriptlet> {
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(scripts::SearchResult::Scriptlet(m)) = flat_results.get(idx) {
                // Get the file path from the UI scriptlet type
                let file_path = m.scriptlet.file_path.clone()?;
                let scriptlet_command = m.scriptlet.command.clone()?;

                // Extract just the file path (before #anchor)
                let file_only = file_path.split('#').next().unwrap_or(&file_path);

                // Read and parse the markdown file to get full scriptlet with actions
                if let Ok(content) = std::fs::read_to_string(file_only) {
                    let parsed_scriptlets =
                        crate::scriptlets::parse_markdown_as_scriptlets(&content, Some(file_only));

                    // Find the matching scriptlet by command
                    return parsed_scriptlets
                        .into_iter()
                        .find(|s| s.command == scriptlet_command);
                }
            }
        }

        None
    }

}
