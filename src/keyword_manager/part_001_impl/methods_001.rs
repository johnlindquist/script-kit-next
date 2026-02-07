impl KeywordManager {
    /// Disable the keyword system (stop keyboard monitoring)
    #[instrument(skip(self))]
    pub fn disable(&mut self) {
        if !self.enabled {
            debug!("Keyword system already disabled");
            return;
        }

        info!("Disabling keyword system");

        if let Some(ref mut monitor) = self.monitor {
            monitor.stop();
        }
        self.monitor = None;
        self.enabled = false;

        info!("Keyword system disabled");
    }

    /// Check if the keyword system is currently enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the number of registered triggers
    #[allow(dead_code)]
    pub fn trigger_count(&self) -> usize {
        let matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
        matcher_guard.trigger_count()
    }

    /// Check if accessibility permissions are granted
    ///
    /// Returns true if the application has accessibility permissions.
    /// These are required for keyboard monitoring and text injection.
    pub fn has_accessibility_permission() -> bool {
        KeyboardMonitor::has_accessibility_permission()
    }

    /// Request accessibility permissions, showing the system dialog if needed
    ///
    /// Returns true if permissions are granted (either already or after user action).
    #[allow(dead_code)]
    pub fn request_accessibility_permission() -> bool {
        KeyboardMonitor::request_accessibility_permission()
    }

    /// Clear all registered triggers
    #[allow(dead_code)]
    pub fn clear_triggers(&mut self) {
        {
            let mut scriptlets_guard = self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
            scriptlets_guard.clear();
        }
        {
            let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
            matcher_guard.clear_triggers();
        }
        {
            let mut file_triggers_guard =
                self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            file_triggers_guard.clear();
        }

        debug!("All keyword triggers cleared");
    }

    /// Reload scriptlets (clear existing and load fresh)
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub fn reload(&mut self) -> Result<usize> {
        info!("Reloading keyword scriptlets");

        self.clear_triggers();
        self.load_scriptlets()
    }

    /// Get list of all registered triggers (for debugging/UI)
    pub fn list_triggers(&self) -> Vec<(String, String)> {
        let scriptlets_guard = self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
        scriptlets_guard
            .iter()
            .map(|(trigger, scriptlet)| (trigger.clone(), scriptlet.name.clone()))
            .collect()
    }

    /// Unregister a single trigger by its keyword
    ///
    /// This removes the trigger from the matcher and the scriptlets store.
    ///
    /// # Arguments
    /// * `trigger` - The trigger keyword to remove (e.g., ":sig")
    ///
    /// # Returns
    /// `true` if the trigger was removed, `false` if it didn't exist
    #[allow(dead_code)]
    pub fn unregister_trigger(&mut self, trigger: &str) -> bool {
        let scriptlet_removed = {
            let mut scriptlets_guard = self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
            scriptlets_guard.remove(trigger).is_some()
        };

        let matcher_removed = {
            let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
            matcher_guard.unregister_trigger(trigger)
        };

        // Also remove from file_triggers tracking
        {
            let mut file_triggers_guard =
                self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            for triggers_set in file_triggers_guard.values_mut() {
                triggers_set.remove(trigger);
            }
            // Clean up empty entries
            file_triggers_guard.retain(|_, triggers| !triggers.is_empty());
        }

        if scriptlet_removed || matcher_removed {
            debug!(trigger = %trigger, "Unregistered keyword trigger");
            true
        } else {
            false
        }
    }

    /// Clear all triggers that came from a specific file
    ///
    /// This is useful when a scriptlet file is deleted - all triggers
    /// registered from that file should be removed.
    ///
    /// # Arguments
    /// * `path` - The path to the scriptlet file
    ///
    /// # Returns
    /// The number of triggers that were removed
    #[allow(dead_code)]
    pub fn clear_triggers_for_file(&mut self, path: &Path) -> usize {
        // Get the triggers registered from this file
        let triggers_to_remove: Vec<String> = {
            let file_triggers_guard = self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            file_triggers_guard
                .get(path)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default()
        };

        if triggers_to_remove.is_empty() {
            debug!(path = %path.display(), "No triggers to clear for file");
            return 0;
        }

        let count = triggers_to_remove.len();

        // Remove each trigger
        for trigger in &triggers_to_remove {
            {
                let mut scriptlets_guard =
                    self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                scriptlets_guard.remove(trigger);
            }
            {
                let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
                matcher_guard.unregister_trigger(trigger);
            }
        }

        // Remove the file entry from tracking
        {
            let mut file_triggers_guard =
                self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            file_triggers_guard.remove(path);
        }

        info!(
            path = %path.display(),
            count = count,
            "Cleared triggers for file"
        );

        count
    }

    /// Get triggers registered for a specific file (for debugging/testing)
    #[allow(dead_code)]
    pub fn get_triggers_for_file(&self, path: &Path) -> Vec<String> {
        let file_triggers_guard = self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
        file_triggers_guard
            .get(path)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Register a trigger from a specific file
    ///
    /// This is like `register_trigger` but also tracks the source file
    /// for incremental updates.
    ///
    /// # Arguments
    /// * `trigger` - The trigger keyword (e.g., ":sig")
    /// * `name` - The scriptlet name
    /// * `content` - The replacement text
    /// * `tool` - The tool type (e.g., "paste", "type")
    /// * `source_path` - The file this trigger came from
    #[allow(dead_code)]
    pub fn register_trigger_from_file(
        &mut self,
        trigger: &str,
        name: &str,
        content: &str,
        tool: &str,
        source_path: &Path,
    ) {
        if trigger.is_empty() {
            debug!("Attempted to register empty trigger, ignoring");
            return;
        }

        info!(
            trigger = %trigger,
            name = %name,
            source = %source_path.display(),
            "Registering keyword trigger from file"
        );

        let keyword_scriptlet = KeywordScriptlet {
            trigger: trigger.to_string(),
            name: name.to_string(),
            content: content.to_string(),
            tool: tool.to_string(),
            source_path: Some(source_path.to_string_lossy().into_owned()),
        };

        {
            let mut scriptlets_guard = self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
            scriptlets_guard.insert(trigger.to_string(), keyword_scriptlet);
        }

        {
            let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
            let dummy_path = PathBuf::from(format!("manual:{}", name));
            matcher_guard.register_trigger(trigger, dummy_path);
        }

        // Track the file -> trigger mapping
        {
            let mut file_triggers_guard =
                self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
            file_triggers_guard
                .entry(source_path.to_path_buf())
                .or_default()
                .insert(trigger.to_string());
        }
    }

}
