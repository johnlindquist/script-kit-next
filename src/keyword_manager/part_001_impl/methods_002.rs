impl KeywordManager {
    /// Update triggers for a file with new scriptlet data
    ///
    /// This performs a diff between the existing triggers and the new triggers:
    /// - Triggers that no longer exist are removed
    /// - New triggers are added
    /// - Triggers with changed content are updated
    ///
    /// # Arguments
    /// * `path` - The path to the scriptlet file
    /// * `new_triggers` - The new trigger definitions: (trigger, name, content, tool)
    ///
    /// # Returns
    /// A tuple of (added_count, removed_count, updated_count)
    #[allow(dead_code)]
    pub fn update_triggers_for_file(
        &mut self,
        path: &Path,
        new_triggers: &[(String, String, String, String)],
    ) -> (usize, usize, usize) {
        // Get existing triggers for this file
        let existing_triggers: HashSet<String> = {
            let file_triggers_guard = self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            let result = file_triggers_guard.get(path).cloned().unwrap_or_default();
            info!(
                path = %path.display(),
                found = !result.is_empty(),
                existing_triggers = ?result,
                "Looking up existing triggers for path"
            );
            result
        };

        // Build set of new trigger keywords
        let new_trigger_keys: HashSet<String> =
            new_triggers.iter().map(|(t, _, _, _)| t.clone()).collect();

        // Find triggers to remove (exist in old but not in new)
        let to_remove: Vec<String> = existing_triggers
            .difference(&new_trigger_keys)
            .cloned()
            .collect();

        // Find triggers to add (exist in new but not in old)
        let to_add: Vec<_> = new_triggers
            .iter()
            .filter(|(t, _, _, _)| !existing_triggers.contains(t))
            .collect();

        // Find triggers to update (exist in both, check if content changed)
        let mut updated_count = 0;
        for (trigger, name, content, tool) in new_triggers {
            if existing_triggers.contains(trigger) {
                // Check if content changed
                let content_changed = {
                    let scriptlets_guard =
                        self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(existing) = scriptlets_guard.get(trigger) {
                        existing.content != *content
                            || existing.name != *name
                            || existing.tool != *tool
                    } else {
                        true // Treat as changed if not found
                    }
                };

                if content_changed {
                    // Update the scriptlet
                    let keyword_scriptlet = KeywordScriptlet {
                        trigger: trigger.clone(),
                        name: name.clone(),
                        content: content.clone(),
                        tool: tool.clone(),
                        source_path: Some(path.to_string_lossy().into_owned()),
                    };

                    {
                        let mut scriptlets_guard =
                            self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                        scriptlets_guard.insert(trigger.clone(), keyword_scriptlet);
                    }

                    debug!(
                        trigger = %trigger,
                        path = %path.display(),
                        "Updated trigger content"
                    );
                    updated_count += 1;
                }
            }
        }

        // Remove old triggers
        for trigger in &to_remove {
            {
                let mut scriptlets_guard =
                    self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                scriptlets_guard.remove(trigger);
            }
            {
                let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
                matcher_guard.unregister_trigger(trigger);
            }
            debug!(trigger = %trigger, path = %path.display(), "Removed trigger");
        }

        // Add new triggers
        for (trigger, name, content, tool) in &to_add {
            let keyword_scriptlet = KeywordScriptlet {
                trigger: trigger.clone(),
                name: name.clone(),
                content: content.clone(),
                tool: tool.clone(),
                source_path: Some(path.to_string_lossy().into_owned()),
            };

            {
                let mut scriptlets_guard =
                    self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                scriptlets_guard.insert(trigger.clone(), keyword_scriptlet);
            }

            {
                let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
                let dummy_path = PathBuf::from(format!("scriptlet:{}", name));
                matcher_guard.register_trigger(trigger, dummy_path);
            }

            debug!(trigger = %trigger, path = %path.display(), "Added trigger");
        }

        // Update file_triggers tracking
        {
            let mut file_triggers_guard =
                self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            if new_trigger_keys.is_empty() {
                file_triggers_guard.remove(path);
            } else {
                file_triggers_guard.insert(path.to_path_buf(), new_trigger_keys);
            }
        }

        let added_count = to_add.len();
        let removed_count = to_remove.len();

        info!(
            path = %path.display(),
            added = added_count,
            removed = removed_count,
            updated = updated_count,
            "Updated triggers for file"
        );

        (added_count, removed_count, updated_count)
    }
}
