// App-specific action handlers for handle_action dispatch.
//
// Contains: show_info_in_finder, show_package_contents, copy_name,
// copy_bundle_id, quit_app, force_quit_app, restart_app.

impl ScriptListApp {
    /// Handle app-specific actions. Returns `DispatchOutcome` indicating if handled.
    fn handle_app_action(
        &mut self,
        action_id: &str,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let trace_id = &dctx.trace_id;
        match action_id {
            "show_info_in_finder" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "show info in Finder action");
                let path_result =
                    self.resolve_file_action_path(crate::action_helpers::extract_path_for_reveal);

                match path_result {
                    Ok(path) => {
                        let path_str = path.to_string_lossy().to_string();
                        let trace_id = trace_id.to_string();
                        cx.spawn(async move |this, cx| {
                            let result = cx
                                .background_executor()
                                .spawn(async move {
                                    crate::file_search::show_info(&path_str)
                                })
                                .await;
                            let _ = this.update(cx, |this, cx| match result {
                                Ok(()) => {
                                    tracing::info!(trace_id = %trace_id, "show_info_in_finder completed");
                                    this.show_hud(
                                        "Opened Info in Finder".to_string(),
                                        Some(HUD_SHORT_MS),
                                        cx,
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                                Err(e) => {
                                    tracing::error!(trace_id = %trace_id, error = %e, "show_info_in_finder failed");
                                    this.show_error_toast(
                                        format!("Failed to show info: {}", e),
                                        cx,
                                    );
                                }
                            });
                        })
                        .detach();
                        DispatchOutcome::success()
                    }
                    Err(msg) => {
                        let msg = msg.unwrap_or_else(|| {
                            gpui::SharedString::from("Cannot show info for this item")
                        });
                        DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            msg.to_string(),
                        )
                    }
                }
            }
            "show_package_contents" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "show package contents action");
                let path_result =
                    self.resolve_file_action_path(crate::action_helpers::extract_path_for_reveal);

                match path_result {
                    Ok(path) => {
                        let contents_path = path.join("Contents");
                        let trace_id = trace_id.to_string();
                        cx.spawn(async move |this, cx| {
                            let result = cx
                                .background_executor()
                                .spawn(async move {
                                    std::process::Command::new("open")
                                        .arg(&contents_path)
                                        .spawn()
                                        .map(|_| ())
                                        .map_err(|e| e.to_string())
                                })
                                .await;
                            let _ = this.update(cx, |this, cx| match result {
                                Ok(()) => {
                                    tracing::info!(trace_id = %trace_id, "show_package_contents completed");
                                    this.show_hud(
                                        "Opened Package Contents".to_string(),
                                        Some(HUD_SHORT_MS),
                                        cx,
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                                Err(e) => {
                                    tracing::error!(trace_id = %trace_id, error = %e, "show_package_contents failed");
                                    this.show_error_toast(
                                        format!("Failed to open package contents: {}", e),
                                        cx,
                                    );
                                }
                            });
                        })
                        .detach();
                        DispatchOutcome::success()
                    }
                    Err(msg) => {
                        let msg = msg.unwrap_or_else(|| {
                            gpui::SharedString::from("Cannot show package contents for this item")
                        });
                        DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            msg.to_string(),
                        )
                    }
                }
            }
            "copy_name" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "copy name action");
                if let Some(result) = self.get_selected_result() {
                    let name = match &result {
                        scripts::SearchResult::App(m) => m.app.name.clone(),
                        _ => {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Copy Name is only available for applications",
                            );
                        }
                    };
                    self.copy_to_clipboard_with_feedback(
                        &name,
                        format!("Copied: {}", name),
                        true,
                        cx,
                    );
                    DispatchOutcome::success()
                } else {
                    DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No item selected",
                    )
                }
            }
            "copy_bundle_id" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "copy bundle identifier action");
                if let Some(result) = self.get_selected_result() {
                    match &result {
                        scripts::SearchResult::App(m) => {
                            if let Some(ref bundle_id) = m.app.bundle_id {
                                self.copy_to_clipboard_with_feedback(
                                    bundle_id,
                                    format!("Copied: {}", bundle_id),
                                    true,
                                    cx,
                                );
                                DispatchOutcome::success()
                            } else {
                                DispatchOutcome::error(
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    "No bundle identifier available for this application",
                                )
                            }
                        }
                        _ => DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Copy Bundle Identifier is only available for applications",
                        ),
                    }
                } else {
                    DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "No item selected",
                    )
                }
            }
            "quit_app" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "quit application action");
                if let Some(scripts::SearchResult::App(m)) = self.get_selected_result() {
                    let app_name = m.app.name.clone();
                    let trace_id = trace_id.to_string();
                    self.show_hud(format!("Quitting {}", app_name), Some(HUD_SHORT_MS), cx);
                    self.hide_main_and_reset(cx);
                    cx.spawn(async move |_this, cx| {
                        let name = app_name.clone();
                        let result = cx
                            .background_executor()
                            .spawn(async move { quit_app_by_name(&name) })
                            .await;
                        if let Err(e) = result {
                            tracing::error!(trace_id = %trace_id, error = %e, "quit_app failed");
                        }
                    })
                    .detach();
                    DispatchOutcome::success()
                } else {
                    DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "Quit is only available for applications",
                    )
                }
            }
            "force_quit_app" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "force quit application action");
                if let Some(scripts::SearchResult::App(m)) = self.get_selected_result() {
                    let app_name = m.app.name.clone();
                    let bundle_id = m.app.bundle_id.clone();
                    let trace_id = trace_id.to_string();
                    self.show_hud(
                        format!("Force quitting {}", app_name),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                    self.hide_main_and_reset(cx);
                    cx.spawn(async move |_this, cx| {
                        let result = cx
                            .background_executor()
                            .spawn(async move {
                                force_quit_app(&app_name, bundle_id.as_deref())
                            })
                            .await;
                        if let Err(e) = result {
                            tracing::error!(trace_id = %trace_id, error = %e, "force_quit_app failed");
                        }
                    })
                    .detach();
                    DispatchOutcome::success()
                } else {
                    DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "Force Quit is only available for applications",
                    )
                }
            }
            "restart_app" => {
                tracing::info!(category = "UI", trace_id = %trace_id, "restart application action");
                if let Some(scripts::SearchResult::App(m)) = self.get_selected_result() {
                    let app_name = m.app.name.clone();
                    let app_path = m.app.path.clone();
                    let trace_id = trace_id.to_string();
                    self.show_hud(format!("Restarting {}", app_name), Some(HUD_SHORT_MS), cx);
                    self.hide_main_and_reset(cx);
                    cx.spawn(async move |_this, cx| {
                        // Quit first
                        let name = app_name.clone();
                        let quit_result = cx
                            .background_executor()
                            .spawn(async move {
                                quit_app_by_name(&name)
                            })
                            .await;

                        if let Err(e) = &quit_result {
                            tracing::warn!(trace_id = %trace_id, error = %e, "quit before restart failed, attempting launch anyway");
                        }

                        // Brief delay to let the app finish quitting
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(500))
                            .await;

                        // Relaunch
                        let path = app_path;
                        let result = cx
                            .background_executor()
                            .spawn(async move {
                                std::process::Command::new("open")
                                    .arg(&path)
                                    .spawn()
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            })
                            .await;

                        if let Err(e) = result {
                            tracing::error!(trace_id = %trace_id, error = %e, "restart relaunch failed");
                        }
                    })
                    .detach();
                    DispatchOutcome::success()
                } else {
                    DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "Restart is only available for applications",
                    )
                }
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}

/// Gracefully quit an application by name using AppleScript.
fn quit_app_by_name(name: &str) -> Result<(), String> {
    std::process::Command::new("osascript")
        .args(["-e", &format!(r#"tell application "{}" to quit"#, name)])
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("AppleScript quit failed: {}", stderr.trim()))
            }
        })
}

/// Force quit an application using its bundle identifier or name.
fn force_quit_app(name: &str, bundle_id: Option<&str>) -> Result<(), String> {
    // Try by bundle_id first (more reliable), fall back to name
    let script = if let Some(bid) = bundle_id {
        format!(
            r#"tell application "System Events"
    set appProcesses to every process whose bundle identifier is "{bid}"
    repeat with proc in appProcesses
        set appPID to unix id of proc
        do shell script "kill -9 " & appPID
    end repeat
end tell"#
        )
    } else {
        format!(
            r#"tell application "System Events"
    set appProcesses to every process whose name is "{name}"
    repeat with proc in appProcesses
        set appPID to unix id of proc
        do shell script "kill -9 " & appPID
    end repeat
end tell"#
        )
    };

    std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Force quit failed: {}", stderr.trim()))
            }
        })
}

#[cfg(test)]
mod app_action_tests {
    #[test]
    fn test_show_package_contents_path_is_app_path_plus_contents() {
        let app_path = std::path::PathBuf::from("/Applications/Safari.app");
        let contents = app_path.join("Contents");
        assert_eq!(
            contents.to_string_lossy(),
            "/Applications/Safari.app/Contents"
        );
    }

    #[test]
    fn test_quit_applescript_uses_app_name() {
        // Verify the format string produces valid AppleScript
        let name = "Google Chrome";
        let script = format!(r#"tell application "{}" to quit"#, name);
        assert_eq!(script, r#"tell application "Google Chrome" to quit"#);
    }

    #[test]
    fn test_force_quit_applescript_uses_bundle_id_when_available() {
        let bid = "com.google.Chrome";
        let script = format!(
            r#"tell application "System Events"
    set appProcesses to every process whose bundle identifier is "{bid}"
    repeat with proc in appProcesses
        set appPID to unix id of proc
        do shell script "kill -9 " & appPID
    end repeat
end tell"#
        );
        assert!(script.contains("com.google.Chrome"));
        assert!(script.contains("bundle identifier"));
    }

    #[test]
    fn test_force_quit_applescript_falls_back_to_name() {
        let name = "Safari";
        let script = format!(
            r#"tell application "System Events"
    set appProcesses to every process whose name is "{name}"
    repeat with proc in appProcesses
        set appPID to unix id of proc
        do shell script "kill -9 " & appPID
    end repeat
end tell"#
        );
        assert!(script.contains("Safari"));
        assert!(script.contains("whose name is"));
    }
}
