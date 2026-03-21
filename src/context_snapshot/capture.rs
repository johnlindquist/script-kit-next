use super::types::*;
use anyhow::Context;

#[cfg(target_os = "macos")]
fn summarize_menu_item(item: &crate::menu_bar::MenuBarItem) -> MenuBarItemSummary {
    MenuBarItemSummary {
        title: item.title.clone(),
        enabled: item.enabled,
        shortcut: item.shortcut.as_ref().map(|s| s.to_display_string()),
        children: item.children.iter().map(summarize_menu_item).collect(),
    }
}

/// Capture a deterministic snapshot of AI-relevant desktop context.
///
/// Individual providers that fail produce a warning string rather than
/// failing the entire snapshot — callers always get partial results.
pub fn capture_context_snapshot(options: &CaptureContextOptions) -> AiContextSnapshot {
    let mut snapshot = AiContextSnapshot::default();

    if options.include_selected_text {
        #[cfg(target_os = "macos")]
        match crate::selected_text::get_selected_text() {
            Ok(text) if !text.trim().is_empty() => {
                tracing::info!(len = text.len(), "context_snapshot: captured selected text");
                snapshot.selected_text = Some(text);
            }
            Ok(_) => {}
            Err(error) => {
                let msg = format!("selectedText: {error}");
                tracing::warn!(warning = %msg, "context_snapshot: provider failed");
                snapshot.warnings.push(msg);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            snapshot
                .warnings
                .push("selectedText: unsupported platform".to_string());
        }
    }

    if options.include_frontmost_app {
        #[cfg(target_os = "macos")]
        if let Some(app) = crate::frontmost_app_tracker::get_last_real_app() {
            tracing::info!(
                pid = app.pid,
                bundle_id = %app.bundle_id,
                name = %app.name,
                "context_snapshot: captured frontmost app"
            );
            snapshot.frontmost_app = Some(FrontmostAppContext {
                pid: app.pid,
                bundle_id: app.bundle_id,
                name: app.name,
            });
        }
    }

    if options.include_menu_bar {
        #[cfg(target_os = "macos")]
        {
            let items = crate::frontmost_app_tracker::get_cached_menu_items();
            tracing::info!(count = items.len(), "context_snapshot: captured menu bar items");
            snapshot.menu_bar_items = items.iter().map(summarize_menu_item).collect();
        }
    }

    if options.include_browser_url {
        match crate::platform::get_focused_browser_tab_url() {
            Ok(url) if !url.trim().is_empty() => {
                tracing::info!("context_snapshot: captured browser URL");
                snapshot.browser = Some(BrowserContext { url });
            }
            Ok(_) => {}
            Err(error) => {
                let msg = format!("browserUrl: {error}");
                tracing::warn!(warning = %msg, "context_snapshot: provider failed");
                snapshot.warnings.push(msg);
            }
        }
    }

    if options.include_focused_window {
        match crate::platform::capture_focused_window_screenshot() {
            Ok(capture) => {
                tracing::info!(
                    title = %capture.window_title,
                    width = capture.width,
                    height = capture.height,
                    "context_snapshot: captured focused window"
                );
                snapshot.focused_window = Some(FocusedWindowContext {
                    title: capture.window_title,
                    width: capture.width,
                    height: capture.height,
                    used_fallback: capture.used_fallback,
                });
            }
            Err(error) => {
                let msg = format!("focusedWindow: {error}");
                tracing::warn!(warning = %msg, "context_snapshot: provider failed");
                snapshot.warnings.push(msg);
            }
        }
    }

    tracing::info!(
        warnings = snapshot.warnings.len(),
        "context_snapshot: capture complete"
    );
    snapshot
}

/// Capture and serialize snapshot as pretty-printed JSON.
pub fn capture_context_snapshot_json(options: &CaptureContextOptions) -> anyhow::Result<String> {
    serde_json::to_string_pretty(&capture_context_snapshot(options))
        .context("Failed to serialize context snapshot")
}
