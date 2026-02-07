use super::*;

impl ScriptListApp {
    pub(crate) fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        self.scripts = scripts::read_scripts();
        // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
        self.scriptlets = scripts::load_scriptlets();
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Sync list component state and validate selection
        // This moves state mutation OUT of render() (anti-pattern fix)
        self.sync_list_state();
        self.selected_index = 0;
        self.validate_selection_bounds(cx);
        self.main_list_state
            .scroll_to_reveal_item(self.selected_index);
        self.last_scrolled_index = Some(self.selected_index);

        // Rebuild alias/shortcut registries and show HUD for any conflicts
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx); // 4s for conflict messages
        }

        logging::log(
            "APP",
            &format!(
                "Scripts refreshed: {} scripts, {} scriptlets loaded",
                self.scripts.len(),
                self.scriptlets.len()
            ),
        );
        cx.notify();
    }

    /// Refresh app launcher cache and invalidate search caches.
    ///
    /// Called by AppWatcher when applications are added/removed/updated.
    /// This properly invalidates filter/grouped caches so the main search
    /// immediately reflects new apps without requiring user to type.
    ///
    /// NOTE: cx.notify() is efficient - GPUI batches notifications and only
    /// re-renders when the event loop runs. We always call it because:
    /// 1. If user is in ScriptList, cached search results need updating
    /// 2. If user is in AppLauncherView, the list needs updating
    /// 3. The cost of an "unnecessary" notify is near-zero (just marks dirty)
    pub fn refresh_apps(&mut self, cx: &mut Context<Self>) {
        self.apps = crate::app_launcher::get_cached_apps();
        // Invalidate caches so main search includes new apps
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Sync list component state and validate selection
        // This ensures the GPUI list component knows about the new app count
        self.sync_list_state();
        self.validate_selection_bounds(cx);

        logging::log(
            "APP",
            &format!("Apps refreshed: {} applications loaded", self.apps.len()),
        );
        cx.notify();
    }

    /// Dismiss the bun warning banner
    pub(crate) fn dismiss_bun_warning(&mut self, cx: &mut Context<Self>) {
        logging::log("APP", "Bun warning banner dismissed by user");
        self.show_bun_warning = false;
        cx.notify();
    }

    /// Open bun.sh in the default browser
    pub(crate) fn open_bun_website(&self) {
        logging::log("APP", "Opening https://bun.sh in default browser");
        if let Err(e) = std::process::Command::new("open")
            .arg("https://bun.sh")
            .spawn()
        {
            logging::log("APP", &format!("Failed to open bun.sh: {}", e));
        }
    }

    /// Handle incremental scriptlet file change
    ///
    /// Instead of reloading all scriptlets, this method:
    /// 1. Parses only the changed file
    /// 2. Diffs against cached state to find what changed
    /// 3. Updates hotkeys/keyword triggers incrementally
    /// 4. Updates the scriptlets list
    ///
    /// # Arguments
    /// * `path` - Path to the changed/deleted scriptlet file
    /// * `is_deleted` - Whether the file was deleted (vs created/modified)
    /// * `cx` - The context for UI updates
    pub(crate) fn handle_scriptlet_file_change(
        &mut self,
        path: &std::path::Path,
        is_deleted: bool,
        cx: &mut Context<Self>,
    ) {
        use script_kit_gpui::scriptlet_cache::{diff_scriptlets, CachedScriptlet};

        logging::log(
            "APP",
            &format!(
                "Incremental scriptlet change: {} (deleted={})",
                path.display(),
                is_deleted
            ),
        );

        // Get old cached scriptlets for this file (if any)
        // Note: We're using a simple approach here - comparing name+shortcut+expand+alias
        let old_scriptlets: Vec<CachedScriptlet> = self
            .scriptlets
            .iter()
            .filter(|s| {
                s.file_path
                    .as_ref()
                    .map(|fp| fp.starts_with(&path.to_string_lossy().to_string()))
                    .unwrap_or(false)
            })
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.keyword.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Parse new scriptlets from file (empty if deleted)
        let new_scripts_scriptlets = if is_deleted {
            vec![]
        } else {
            scripts::read_scriptlets_from_file(path)
        };

        let new_scriptlets: Vec<CachedScriptlet> = new_scripts_scriptlets
            .iter()
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.keyword.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // ALWAYS update keyword triggers when a file changes
        // This is needed because the diff only tracks registration metadata (name, shortcut, keyword, alias)
        // but NOT the actual content. So content changes like "success three" -> "success four"
        // would be missed if we only update on diff changes.
        #[cfg(target_os = "macos")]
        {
            let (added, removed, updated) =
                crate::keyword_manager::update_keyword_triggers_for_file(
                    path,
                    &new_scripts_scriptlets,
                );
            if added > 0 || removed > 0 || updated > 0 {
                logging::log(
                    "KEYWORD",
                    &format!(
                        "Updated keyword triggers for {}: {} added, {} removed, {} updated",
                        path.display(),
                        added,
                        removed,
                        updated
                    ),
                );
            }
        }

        // Compute diff for registration metadata changes (shortcuts, aliases)
        let diff = diff_scriptlets(&old_scriptlets, &new_scriptlets);

        if diff.is_empty() {
            logging::log(
                "APP",
                &format!("No registration metadata changes in {}", path.display()),
            );
            // Still need to update the scriptlets list even if no registration changes
            // because the content might have changed
        } else {
            logging::log(
                "APP",
                &format!(
                    "Scriptlet diff: {} added, {} removed, {} shortcut changes, {} keyword changes, {} alias changes",
                    diff.added.len(),
                    diff.removed.len(),
                    diff.shortcut_changes.len(),
                    diff.keyword_changes.len(),
                    diff.alias_changes.len()
                ),
            );
        }

        // Apply hotkey changes
        for removed in &diff.removed {
            if removed.shortcut.is_some() {
                if let Err(e) = hotkeys::unregister_script_hotkey(&removed.file_path) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister hotkey for {}: {}", removed.name, e),
                    );
                }
            }
        }

        for added in &diff.added {
            if let Some(ref shortcut) = added.shortcut {
                if let Err(e) = hotkeys::register_script_hotkey(&added.file_path, shortcut) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register hotkey for {}: {}", added.name, e),
                    );
                }
            }
        }

        for change in &diff.shortcut_changes {
            if let Err(e) = hotkeys::update_script_hotkey(
                &change.file_path,
                change.old.as_deref(),
                change.new.as_deref(),
            ) {
                logging::log(
                    "HOTKEY",
                    &format!("Failed to update hotkey for {}: {}", change.name, e),
                );
            }
        }

        // Update the scriptlets list
        // Remove old scriptlets from this file
        let path_str = path.to_string_lossy().to_string();
        self.scriptlets.retain(|s| {
            !s.file_path
                .as_ref()
                .map(|fp| fp.starts_with(&path_str))
                .unwrap_or(false)
        });

        // Add new scriptlets from this file
        self.scriptlets.extend(new_scripts_scriptlets);

        // Sort by name to maintain consistent ordering
        self.scriptlets.sort_by(|a, b| a.name.cmp(&b.name));

        // Invalidate caches
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Sync list component state so GPUI renders the correct item count
        self.sync_list_state();
        self.validate_selection_bounds(cx);

        // Rebuild alias/shortcut registries for this file's scriptlets
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx);
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet file updated incrementally: {} now has {} total scriptlets",
                path.display(),
                self.scriptlets.len()
            ),
        );

        cx.notify();
    }

}
