impl KeywordManager {
    /// Create a new KeywordManager with default configuration
    pub fn new() -> Self {
        Self::with_config(KeywordManagerConfig::default())
    }

    /// Create a new KeywordManager with custom configuration
    pub fn with_config(config: KeywordManagerConfig) -> Self {
        let injector = TextInjector::with_config(config.injector_config.clone());

        Self {
            config,
            scriptlets: Arc::new(Mutex::new(HashMap::new())),
            matcher: Arc::new(Mutex::new(KeywordMatcher::new())),
            file_triggers: Arc::new(Mutex::new(HashMap::new())),
            monitor: None,
            injector,
            enabled: false,
        }
    }

    /// Load scriptlets with keyword metadata from ~/.scriptkit/kit/*/extensions/
    ///
    /// This scans all markdown files and registers any scriptlet that has
    /// an `keyword` metadata field as a trigger.
    #[instrument(skip(self))]
    pub fn load_scriptlets(&mut self) -> Result<usize> {
        info!("Loading scriptlets with keyword triggers");

        // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
        let scriptlets = load_scriptlets();
        let mut loaded_count = 0;

        for scriptlet in scriptlets {
            // Only process scriptlets with keyword metadata
            if let Some(ref keyword_trigger) = scriptlet.keyword {
                if keyword_trigger.is_empty() {
                    debug!(
                        name = %scriptlet.name,
                        "Skipping scriptlet with empty keyword trigger"
                    );
                    continue;
                }

                info!(
                    trigger = %keyword_trigger,
                    name = %scriptlet.name,
                    tool = %scriptlet.tool,
                    "Registering keyword trigger"
                );

                // Store the scriptlet info
                let keyword_scriptlet = KeywordScriptlet {
                    trigger: keyword_trigger.clone(),
                    name: scriptlet.name.clone(),
                    content: scriptlet.code.clone(),
                    tool: scriptlet.tool.clone(),
                    source_path: scriptlet.file_path.clone(),
                };

                // Register with matcher and scriptlets store
                {
                    let mut scriptlets_guard =
                        self.scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                    scriptlets_guard.insert(keyword_trigger.clone(), keyword_scriptlet);
                }

                {
                    let mut matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
                    // Use a dummy path since we store scriptlet data separately
                    let dummy_path = PathBuf::from(format!("scriptlet:{}", scriptlet.name));
                    matcher_guard.register_trigger(keyword_trigger, dummy_path);
                }

                // Track which file this trigger came from for incremental updates
                // Note: scriptlet.file_path includes an anchor like "/path/file.md#slug"
                // We need to strip the anchor for file-level tracking
                if let Some(ref file_path) = scriptlet.file_path {
                    let base_path = if let Some(hash_idx) = file_path.find('#') {
                        PathBuf::from(&file_path[..hash_idx])
                    } else {
                        PathBuf::from(file_path)
                    };
                    let mut file_triggers_guard =
                        self.file_triggers.lock().unwrap_or_else(|e| e.into_inner());
                    file_triggers_guard
                        .entry(base_path)
                        .or_default()
                        .insert(keyword_trigger.clone());
                }

                loaded_count += 1;
            }
        }

        info!(
            count = loaded_count,
            "Loaded keyword triggers from scriptlets"
        );
        Ok(loaded_count)
    }

    /// Register a single keyword trigger manually
    ///
    /// This is useful for adding triggers that don't come from scriptlets.
    #[allow(dead_code)]
    pub fn register_trigger(&mut self, trigger: &str, name: &str, content: &str, tool: &str) {
        if trigger.is_empty() {
            debug!("Attempted to register empty trigger, ignoring");
            return;
        }

        info!(
            trigger = %trigger,
            name = %name,
            "Manually registering keyword trigger"
        );

        let keyword_scriptlet = KeywordScriptlet {
            trigger: trigger.to_string(),
            name: name.to_string(),
            content: content.to_string(),
            tool: tool.to_string(),
            source_path: None,
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
    }

    /// Enable the keyword system (start keyboard monitoring)
    ///
    /// # Errors
    /// - `AccessibilityNotGranted`: Accessibility permissions not enabled
    /// - `EventTapCreationFailed`: Failed to create macOS event tap
    #[instrument(skip(self))]
    pub fn enable(&mut self) -> Result<(), KeyboardMonitorError> {
        if self.enabled {
            debug!("Keyword system already enabled");
            return Ok(());
        }

        info!("Enabling keyword system");

        // Check trigger count
        let trigger_count = {
            let matcher_guard = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
            matcher_guard.trigger_count()
        };

        if trigger_count == 0 {
            warn!("No keyword triggers registered, keyboard monitoring will be ineffective");
        }

        // Clone Arc references for the closure
        let matcher = Arc::clone(&self.matcher);
        let scriptlets = Arc::clone(&self.scriptlets);
        // Wrap configs in Arc to avoid cloning on every keystroke
        let config = Arc::new(self.config.clone());
        let injector_config = Arc::new(self.config.injector_config.clone());

        // Create keyboard monitor with callback
        let mut monitor = KeyboardMonitor::new(move |event: KeyEvent| {
            // Only process printable characters (ignore modifier keys, etc.)
            if let Some(ref character) = event.character {
                // Skip if any modifier is held (except shift for capitals)
                if event.command || event.control || event.option {
                    keystroke_logger().record_skipped();
                    return;
                }

                // Process each character in the string (usually just 1)
                for c in character.chars() {
                    // Record keystroke for debounced logging
                    keystroke_logger().record_keystroke(c);

                    // Feed to matcher
                    let match_result = {
                        let mut matcher_guard = matcher.lock().unwrap_or_else(|e| e.into_inner());
                        matcher_guard.process_keystroke(c)
                    };

                    // Handle match if found
                    if let Some(result) = match_result {
                        // Log match immediately (important event)
                        keystroke_logger().log_match(&result.trigger, result.chars_to_delete);

                        // Get the scriptlet content
                        let scriptlet_opt = {
                            let scriptlets_guard =
                                scriptlets.lock().unwrap_or_else(|e| e.into_inner());
                            scriptlets_guard.get(&result.trigger).cloned()
                        };

                        if let Some(scriptlet) = scriptlet_opt {
                            // Perform expansion in a separate thread to not block the callback
                            let chars_to_delete = result.chars_to_delete;
                            let content = scriptlet.content.clone();
                            let tool = scriptlet.tool.clone();
                            let name = scriptlet.name.clone();
                            // Arc clone is cheap - just increments reference count
                            let config_clone = Arc::clone(&config);
                            let injector_config_clone = Arc::clone(&injector_config);

                            thread::spawn(move || {
                                // Small delay to let the keyboard event complete
                                thread::sleep(Duration::from_millis(config_clone.stop_delay_ms));

                                // Get raw content based on tool type
                                let raw_content = match tool.as_str() {
                                    "paste" | "type" | "template" => content.clone(),
                                    _ => {
                                        // For other tools, use the content as-is for now
                                        // Future: execute the scriptlet and capture output
                                        info!(
                                            tool = %tool,
                                            name = %name,
                                            "Tool type not yet fully supported for keyword, using raw content"
                                        );
                                        content.clone()
                                    }
                                };

                                // Substitute template variables (${clipboard}, ${date}, etc.)
                                // Uses the centralized template_variables module
                                let replacement = substitute_variables(&raw_content);

                                debug!(
                                    original_len = raw_content.len(),
                                    substituted_len = replacement.len(),
                                    had_substitutions = raw_content != replacement,
                                    "Variable substitution completed"
                                );

                                // Create injector and perform expansion
                                // Dereference Arc to get the config
                                let injector =
                                    TextInjector::with_config((*injector_config_clone).clone());

                                // Delete trigger characters
                                if let Err(e) = injector.delete_chars(chars_to_delete) {
                                    error!(
                                        error = %e,
                                        chars = chars_to_delete,
                                        "Failed to delete trigger characters"
                                    );
                                    return;
                                }

                                // Small delay between delete and paste
                                thread::sleep(Duration::from_millis(50));

                                // Paste replacement text
                                if let Err(e) = injector.paste_text(&replacement) {
                                    error!(
                                        error = %e,
                                        "Failed to paste replacement text"
                                    );
                                    return;
                                }

                                info!(
                                    trigger = %name,
                                    replacement_len = replacement.len(),
                                    "Expansion completed successfully"
                                );
                            });

                            // Clear the buffer after a match to prevent re-triggering
                            let mut matcher_guard =
                                matcher.lock().unwrap_or_else(|e| e.into_inner());
                            matcher_guard.clear_buffer();
                        } else {
                            warn!(
                                trigger = %result.trigger,
                                "Matched trigger but scriptlet not found in store"
                            );
                        }
                    }
                }
            }
        });

        // Start the monitor
        monitor.start()?;

        self.monitor = Some(monitor);
        self.enabled = true;

        info!("Keyword system enabled, keyboard monitoring active");
        Ok(())
    }

}
