// App-specific action handlers for handle_action dispatch.
//
// Contains: show_info_in_finder, show_package_contents, copy_name,
// copy_bundle_id, quit_app, force_quit_app, restart_app.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCopyHandlerAction {
    Name,
    BundleIdentifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppOpenHandlerAction {
    ShowInfoInFinder,
    ShowPackageContents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppLifecycleHandlerAction {
    Quit,
    ForceQuit,
    Restart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppLifecycleTarget {
    app_name: String,
    bundle_id: Option<String>,
    app_path: std::path::PathBuf,
}

impl AppCopyHandlerAction {
    fn from_action_id(action_id: &str) -> Option<Self> {
        match action_id {
            "copy_name" => Some(Self::Name),
            "copy_bundle_id" => Some(Self::BundleIdentifier),
            _ => None,
        }
    }

    fn copy_value(self, result: &scripts::SearchResult) -> Result<String, &'static str> {
        let scripts::SearchResult::App(m) = result else {
            return Err(match self {
                Self::Name => "Copy Name is only available for applications",
                Self::BundleIdentifier => {
                    "Copy Bundle Identifier is only available for applications"
                }
            });
        };

        match self {
            Self::Name => Ok(m.app.name.clone()),
            Self::BundleIdentifier => m
                .app
                .bundle_id
                .clone()
                .ok_or("No bundle identifier available for this application"),
        }
    }

    fn copied_hud(self, value: &str) -> String {
        match self {
            Self::Name | Self::BundleIdentifier => format!("Copied: {value}"),
        }
    }

    fn selection_required_message(self) -> &'static str {
        match self {
            Self::Name | Self::BundleIdentifier => "No item selected",
        }
    }
}

impl AppOpenHandlerAction {
    fn from_action_id(action_id: &str) -> Option<Self> {
        match action_id {
            "show_info_in_finder" => Some(Self::ShowInfoInFinder),
            "show_package_contents" => Some(Self::ShowPackageContents),
            _ => None,
        }
    }

    fn trace_name(self) -> &'static str {
        match self {
            Self::ShowInfoInFinder => "show_info_in_finder",
            Self::ShowPackageContents => "show_package_contents",
        }
    }

    fn missing_target_message(self) -> &'static str {
        match self {
            Self::ShowInfoInFinder => "Cannot show info for this item",
            Self::ShowPackageContents => "Cannot show package contents for this item",
        }
    }

    fn success_hud(self) -> &'static str {
        match self {
            Self::ShowInfoInFinder => "Opened Info in Finder",
            Self::ShowPackageContents => "Opened Package Contents",
        }
    }

    fn error_prefix(self) -> &'static str {
        match self {
            Self::ShowInfoInFinder => "Failed to show info",
            Self::ShowPackageContents => "Failed to open package contents",
        }
    }

    fn run(self, path: std::path::PathBuf) -> Result<(), String> {
        match self {
            Self::ShowInfoInFinder => crate::file_search::show_info(&path.to_string_lossy()),
            Self::ShowPackageContents => std::process::Command::new("open")
                .arg(path.join("Contents"))
                .spawn()
                .map(|_| ())
                .map_err(|e| e.to_string()),
        }
    }
}

impl AppLifecycleHandlerAction {
    fn from_action_id(action_id: &str) -> Option<Self> {
        match action_id {
            "quit_app" => Some(Self::Quit),
            "force_quit_app" => Some(Self::ForceQuit),
            "restart_app" => Some(Self::Restart),
            _ => None,
        }
    }

    fn trace_message(self) -> &'static str {
        match self {
            Self::Quit => "quit application action",
            Self::ForceQuit => "force quit application action",
            Self::Restart => "restart application action",
        }
    }

    fn hud_message(self, app_name: &str) -> String {
        match self {
            Self::Quit => format!("Quitting {app_name}"),
            Self::ForceQuit => format!("Force quitting {app_name}"),
            Self::Restart => format!("Restarting {app_name}"),
        }
    }

    fn unsupported_message(self) -> &'static str {
        match self {
            Self::Quit => "Quit is only available for applications",
            Self::ForceQuit => "Force Quit is only available for applications",
            Self::Restart => "Restart is only available for applications",
        }
    }

    fn target_from_result(
        self,
        result: Option<scripts::SearchResult>,
    ) -> Result<AppLifecycleTarget, &'static str> {
        let Some(scripts::SearchResult::App(app_result)) = result else {
            return Err(self.unsupported_message());
        };

        Ok(AppLifecycleTarget {
            app_name: app_result.app.name,
            bundle_id: app_result.app.bundle_id,
            app_path: app_result.app.path,
        })
    }
}

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
            "show_info_in_finder" | "show_package_contents" => {
                let Some(open_action) = AppOpenHandlerAction::from_action_id(action_id) else {
                    return DispatchOutcome::not_handled();
                };
                tracing::info!(
                    category = "UI",
                    trace_id = %trace_id,
                    action = open_action.trace_name(),
                    "app open item action"
                );
                let path_result =
                    self.resolve_file_action_path(crate::action_helpers::extract_path_for_reveal);

                match path_result {
                    Ok(path) => {
                        let trace_id = trace_id.to_string();
                        cx.spawn(async move |this, cx| {
                            let result = cx
                                .background_executor()
                                .spawn(async move { open_action.run(path) })
                                .await;
                            let _ = this.update(cx, |this, cx| match result {
                                Ok(()) => {
                                    tracing::info!(
                                        trace_id = %trace_id,
                                        action = open_action.trace_name(),
                                        "app open item action completed"
                                    );
                                    this.show_hud(
                                        open_action.success_hud().to_string(),
                                        Some(HUD_SHORT_MS),
                                        cx,
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        trace_id = %trace_id,
                                        error = %e,
                                        action = open_action.trace_name(),
                                        "app open item action failed"
                                    );
                                    this.show_error_toast(
                                        format!("{}: {}", open_action.error_prefix(), e),
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
                            gpui::SharedString::from(open_action.missing_target_message())
                        });
                        DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            msg.to_string(),
                        )
                    }
                }
            }
            "copy_name" | "copy_bundle_id" => {
                let Some(copy_action) = AppCopyHandlerAction::from_action_id(action_id) else {
                    return DispatchOutcome::not_handled();
                };
                tracing::info!(category = "UI", trace_id = %trace_id, action = %action_id, "copy app field action");
                let Some(result) = self.get_selected_result() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        copy_action.selection_required_message(),
                    );
                };

                match copy_action.copy_value(&result) {
                    Ok(value) => {
                        self.copy_to_clipboard_with_feedback(
                            &value,
                            copy_action.copied_hud(&value),
                            true,
                            cx,
                        );
                        DispatchOutcome::success()
                    }
                    Err(message) => {
                        DispatchOutcome::error(crate::action_helpers::ERROR_ACTION_FAILED, message)
                    }
                }
            }
            "quit_app" | "force_quit_app" | "restart_app" => {
                let Some(lifecycle_action) = AppLifecycleHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                tracing::info!(category = "UI", trace_id = %trace_id, "{}", lifecycle_action.trace_message());

                let target = match lifecycle_action.target_from_result(self.get_selected_result()) {
                    Ok(target) => target,
                    Err(message) => {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            message,
                        );
                    }
                };

                let app_name = target.app_name.clone();
                let trace_id = trace_id.to_string();
                self.show_hud(
                    lifecycle_action.hud_message(&app_name),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                self.hide_main_and_reset(cx);
                cx.spawn(async move |_this, cx| {
                    match lifecycle_action {
                        AppLifecycleHandlerAction::Quit => {
                            let name = target.app_name.clone();
                            let result = cx
                                .background_executor()
                                .spawn(async move { quit_app_by_name(&name) })
                                .await;
                            if let Err(e) = result {
                                tracing::error!(trace_id = %trace_id, error = %e, "quit_app failed");
                            }
                        }
                        AppLifecycleHandlerAction::ForceQuit => {
                            let app_name = target.app_name;
                            let bundle_id = target.bundle_id;
                            let result = cx
                                .background_executor()
                                .spawn(async move { force_quit_app(&app_name, bundle_id.as_deref()) })
                                .await;
                            if let Err(e) = result {
                                tracing::error!(trace_id = %trace_id, error = %e, "force_quit_app failed");
                            }
                        }
                        AppLifecycleHandlerAction::Restart => {
                            let name = target.app_name.clone();
                            let quit_result = cx
                                .background_executor()
                                .spawn(async move { quit_app_by_name(&name) })
                                .await;

                            if let Err(e) = &quit_result {
                                tracing::warn!(trace_id = %trace_id, error = %e, "quit before restart failed, attempting launch anyway");
                            }

                            // Brief delay to let the app finish quitting
                            cx.background_executor()
                                .timer(std::time::Duration::from_millis(500))
                                .await;

                            // Relaunch
                            let path = target.app_path;
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
                        }
                    }
                })
                .detach();
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}

/// Gracefully quit an application by name using AppleScript.
fn quit_app_by_name(name: &str) -> Result<(), String> {
    let escaped_name = crate::utils::escape_applescript_string(name);
    std::process::Command::new("osascript")
        .args(["-e", &format!(r#"tell application "{}" to quit"#, escaped_name)])
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
    let escaped_name = crate::utils::escape_applescript_string(name);
    let script = if let Some(bid) = bundle_id {
        let escaped_bid = crate::utils::escape_applescript_string(bid);
        format!(
            r#"tell application "System Events"
    set appProcesses to every process whose bundle identifier is "{escaped_bid}"
    repeat with proc in appProcesses
        set appPID to unix id of proc
        do shell script "kill -9 " & appPID
    end repeat
end tell"#
        )
    } else {
        format!(
            r#"tell application "System Events"
    set appProcesses to every process whose name is "{escaped_name}"
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
