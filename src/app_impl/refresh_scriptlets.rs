use super::*;

static SCRIPT_REFRESH_REQUEST_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

struct AsyncScriptRefreshLoadResult {
    scripts: Vec<std::sync::Arc<scripts::Script>>,
    scriptlets: Vec<std::sync::Arc<scripts::Scriptlet>>,
    scripts_elapsed: std::time::Duration,
    scriptlets_elapsed: std::time::Duration,
    total_elapsed: std::time::Duration,
}

fn spawn_async_script_refresh_load(
    scripts_loader: impl FnOnce() -> Vec<std::sync::Arc<scripts::Script>> + Send + 'static,
    scriptlets_loader: impl FnOnce() -> Vec<std::sync::Arc<scripts::Scriptlet>> + Send + 'static,
) -> async_channel::Receiver<AsyncScriptRefreshLoadResult> {
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let load_started_at = std::time::Instant::now();
        let (scripts, scripts_elapsed, scriptlets, scriptlets_elapsed) = std::thread::scope(
            |scope| {
                let scripts_handle = scope.spawn(move || {
                    let started = std::time::Instant::now();
                    (scripts_loader(), started.elapsed())
                });
                let scriptlets_handle = scope.spawn(move || {
                    let started = std::time::Instant::now();
                    (scriptlets_loader(), started.elapsed())
                });

                let (scripts, scripts_elapsed) = match scripts_handle.join() {
                    Ok(result) => result,
                    Err(_) => {
                        logging::log(
                        "ERROR",
                        "script_refresh_async: attempted=load_scripts failed=thread_panicked state=background_loading",
                    );
                        (Vec::new(), std::time::Duration::ZERO)
                    }
                };
                let (scriptlets, scriptlets_elapsed) = match scriptlets_handle.join() {
                    Ok(result) => result,
                    Err(_) => {
                        logging::log(
                        "ERROR",
                        "script_refresh_async: attempted=load_scriptlets failed=thread_panicked state=background_loading",
                    );
                        (Vec::new(), std::time::Duration::ZERO)
                    }
                };

                (scripts, scripts_elapsed, scriptlets, scriptlets_elapsed)
            },
        );

        let result = AsyncScriptRefreshLoadResult {
            scripts,
            scriptlets,
            scripts_elapsed,
            scriptlets_elapsed,
            total_elapsed: load_started_at.elapsed(),
        };

        if tx.send_blocking(result).is_err() {
            logging::log(
                "ERROR",
                "script_refresh_async: attempted=send_load_result failed=receiver_dropped state=background_complete",
            );
        }
    });
    rx
}

impl ScriptListApp {
    pub(crate) fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        let request_id =
            SCRIPT_REFRESH_REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        logging::log(
            "APP",
            &format!(
                "script_refresh_async: state=dispatching_background_load request_id={} current_scripts={} current_scriptlets={}",
                request_id,
                self.scripts.len(),
                self.scriptlets.len()
            ),
        );

        let rx = spawn_async_script_refresh_load(scripts::read_scripts, scripts::load_scriptlets);
        cx.spawn(async move |this, cx| {
            let Ok(load_result) = rx.recv().await else {
                logging::log(
                    "ERROR",
                    "script_refresh_async: attempted=receive_load_result failed=channel_closed state=awaiting_background_load",
                );
                return;
            };

            let scripts_count = load_result.scripts.len();
            let scriptlets_count = load_result.scriptlets.len();
            let scripts_elapsed_ms = load_result.scripts_elapsed.as_secs_f64() * 1000.0;
            let scriptlets_elapsed_ms = load_result.scriptlets_elapsed.as_secs_f64() * 1000.0;
            let total_elapsed_ms = load_result.total_elapsed.as_secs_f64() * 1000.0;
            let latest_request_id =
                SCRIPT_REFRESH_REQUEST_ID.load(std::sync::atomic::Ordering::Relaxed);
            if request_id != latest_request_id {
                logging::log(
                    "APP",
                    &format!(
                        "script_refresh_async: state=discarding_stale_result request_id={} latest_request_id={} scripts={} scriptlets={}",
                        request_id,
                        latest_request_id,
                        scripts_count,
                        scriptlets_count
                    ),
                );
                return;
            }

            let update_result = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    app.apply_loaded_scripts_and_scriptlets(
                        load_result.scripts,
                        load_result.scriptlets,
                        cx,
                    );
                    logging::log(
                        "APP",
                        &format!(
                            "script_refresh_async: state=applied_to_ui request_id={} scripts={} scriptlets={} scripts_ms={:.2} scriptlets_ms={:.2} total_ms={:.2}",
                            request_id,
                            scripts_count,
                            scriptlets_count,
                            scripts_elapsed_ms,
                            scriptlets_elapsed_ms,
                            total_elapsed_ms
                        ),
                    );
                })
            });

            if update_result.is_err() {
                logging::log(
                    "ERROR",
                    "script_refresh_async: attempted=apply_loaded_results failed=ui_entity_unavailable state=applying_to_ui",
                );
            }
        })
        .detach();
    }

    fn apply_loaded_scripts_and_scriptlets(
        &mut self,
        loaded_scripts: Vec<std::sync::Arc<scripts::Script>>,
        loaded_scriptlets: Vec<std::sync::Arc<scripts::Scriptlet>>,
        cx: &mut Context<Self>,
    ) {
        self.scripts = loaded_scripts;
        // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
        self.scriptlets = loaded_scriptlets;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_spawn_async_script_refresh_load_returns_results_when_loaders_run_off_main_thread() {
        let main_thread_id = std::thread::current().id();
        let scripts_thread_id = Arc::new(Mutex::new(None));
        let scriptlets_thread_id = Arc::new(Mutex::new(None));

        let scripts_thread_id_clone = Arc::clone(&scripts_thread_id);
        let scriptlets_thread_id_clone = Arc::clone(&scriptlets_thread_id);

        let rx = spawn_async_script_refresh_load(
            move || {
                std::thread::sleep(Duration::from_millis(5));
                *scripts_thread_id_clone
                    .lock()
                    .expect("scripts thread id lock should succeed") =
                    Some(std::thread::current().id());
                Vec::new()
            },
            move || {
                std::thread::sleep(Duration::from_millis(5));
                *scriptlets_thread_id_clone
                    .lock()
                    .expect("scriptlets thread id lock should succeed") =
                    Some(std::thread::current().id());
                Vec::new()
            },
        );

        let result = rx
            .recv_blocking()
            .expect("background loaders should send exactly one result");

        assert!(result.total_elapsed >= result.scripts_elapsed);
        assert!(result.total_elapsed >= result.scriptlets_elapsed);

        let scripts_worker_thread = scripts_thread_id
            .lock()
            .expect("scripts thread id lock should succeed")
            .expect("scripts loader should execute");
        let scriptlets_worker_thread = scriptlets_thread_id
            .lock()
            .expect("scriptlets thread id lock should succeed")
            .expect("scriptlets loader should execute");

        assert_ne!(scripts_worker_thread, main_thread_id);
        assert_ne!(scriptlets_worker_thread, main_thread_id);
    }
}
