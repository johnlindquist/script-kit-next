use super::types::*;
use anyhow::Context;
use std::sync::atomic::{AtomicBool, Ordering};

/// When set to `true`, `capture_context_snapshot` uses a default empty seed
/// instead of calling live OS providers. This prevents real keystrokes
/// (e.g. Cmd+C from the `get-selected-text` clipboard fallback) during tests.
static DETERMINISTIC_CONTEXT: AtomicBool = AtomicBool::new(false);

/// Enable deterministic (no-op) context capture for the rest of this process.
///
/// Call this from integration tests that resolve `kit://context` URIs but do
/// not need live OS data. Without this, resolution triggers real Cmd+C
/// keystrokes via the `get-selected-text` crate's clipboard fallback.
#[allow(dead_code)] // Public API surface for integration tests
pub fn enable_deterministic_context_capture() {
    DETERMINISTIC_CONTEXT.store(true, Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
fn summarize_menu_item(item: &crate::menu_bar::MenuBarItem) -> MenuBarItemSummary {
    MenuBarItemSummary {
        title: item.title.clone(),
        enabled: item.enabled,
        shortcut: item.shortcut.as_ref().map(|s| s.to_display_string()),
        children: item.children.iter().map(summarize_menu_item).collect(),
    }
}

/// Pre-resolved provider results that `capture_context_snapshot_from_seed` can
/// consume deterministically — no live OS calls required.
#[derive(Debug, Clone)]
pub(crate) struct CaptureContextSeed {
    pub(crate) selected_text: Result<Option<String>, String>,
    pub(crate) frontmost_app: Result<Option<FrontmostAppContext>, String>,
    pub(crate) menu_bar_items: Result<Vec<MenuBarItemSummary>, String>,
    pub(crate) browser: Result<Option<BrowserContext>, String>,
    pub(crate) focused_window: Result<Option<FocusedWindowContext>, String>,
}

impl Default for CaptureContextSeed {
    fn default() -> Self {
        Self {
            selected_text: Ok(None),
            frontmost_app: Ok(None),
            menu_bar_items: Ok(Vec::new()),
            browser: Ok(None),
            focused_window: Ok(None),
        }
    }
}

/// Build a snapshot from pre-resolved provider results. This is the core
/// deterministic function that both live capture and tests use.
pub(crate) fn capture_context_snapshot_from_seed(
    options: &CaptureContextOptions,
    seed: CaptureContextSeed,
) -> AiContextSnapshot {
    let mut snapshot = AiContextSnapshot::default();

    if options.include_selected_text {
        match seed.selected_text {
            Ok(Some(text)) if !text.trim().is_empty() => snapshot.selected_text = Some(text),
            Ok(_) => {}
            Err(error) => snapshot.warnings.push(format!("selectedText: {error}")),
        }
    }

    if options.include_frontmost_app {
        match seed.frontmost_app {
            Ok(Some(app)) => snapshot.frontmost_app = Some(app),
            Ok(None) => {}
            Err(error) => snapshot.warnings.push(format!("frontmostApp: {error}")),
        }
    }

    if options.include_menu_bar {
        match seed.menu_bar_items {
            Ok(items) => snapshot.menu_bar_items = items,
            Err(error) => snapshot.warnings.push(format!("menuBar: {error}")),
        }
    }

    if options.include_browser_url {
        match seed.browser {
            Ok(Some(b)) => snapshot.browser = Some(b),
            Ok(None) => {}
            Err(error) => snapshot.warnings.push(format!("browserUrl: {error}")),
        }
    }

    if options.include_focused_window {
        match seed.focused_window {
            Ok(Some(w)) => snapshot.focused_window = Some(w),
            Ok(None) => {}
            Err(error) => snapshot.warnings.push(format!("focusedWindow: {error}")),
        }
    }

    tracing::info!(
        schema_version = snapshot.schema_version,
        warnings = snapshot.warnings.len(),
        has_selected_text = snapshot.selected_text.is_some(),
        has_frontmost_app = snapshot.frontmost_app.is_some(),
        menu_bar_count = snapshot.menu_bar_items.len(),
        has_browser = snapshot.browser.is_some(),
        has_focused_window = snapshot.focused_window.is_some(),
        "context_snapshot: seed capture complete"
    );

    snapshot
}

// ── Live provider helpers ────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn capture_selected_text_live() -> Result<Option<String>, String> {
    match crate::selected_text::get_selected_text() {
        Ok(text) if !text.trim().is_empty() => {
            tracing::info!(len = text.len(), "context_snapshot: captured selected text");
            Ok(Some(text))
        }
        Ok(_) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(not(target_os = "macos"))]
fn capture_selected_text_live() -> Result<Option<String>, String> {
    Err("unsupported platform".to_string())
}

#[cfg(target_os = "macos")]
fn capture_frontmost_app_live() -> Result<Option<FrontmostAppContext>, String> {
    let app = crate::frontmost_app_tracker::get_last_real_app().map(|app| {
        tracing::info!(
            pid = app.pid,
            bundle_id = %app.bundle_id,
            name = %app.name,
            "context_snapshot: captured frontmost app"
        );
        FrontmostAppContext {
            pid: app.pid,
            bundle_id: app.bundle_id,
            name: app.name,
        }
    });
    Ok(app)
}

#[cfg(not(target_os = "macos"))]
fn capture_frontmost_app_live() -> Result<Option<FrontmostAppContext>, String> {
    Ok(None)
}

#[cfg(target_os = "macos")]
fn capture_menu_bar_live() -> Result<Vec<MenuBarItemSummary>, String> {
    let items = crate::frontmost_app_tracker::get_cached_menu_items();
    tracing::info!(
        count = items.len(),
        "context_snapshot: captured menu bar items"
    );
    Ok(items.iter().map(summarize_menu_item).collect())
}

#[cfg(not(target_os = "macos"))]
fn capture_menu_bar_live() -> Result<Vec<MenuBarItemSummary>, String> {
    Ok(Vec::new())
}

fn capture_browser_live() -> Result<Option<BrowserContext>, String> {
    match crate::platform::get_focused_browser_tab_url() {
        Ok(url) if !url.trim().is_empty() => {
            tracing::info!("context_snapshot: captured browser URL");
            Ok(Some(BrowserContext { url }))
        }
        Ok(_) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn capture_focused_window_live() -> Result<Option<FocusedWindowContext>, String> {
    if let Some(app) = crate::frontmost_app_tracker::get_last_real_app() {
        if let Some(title) = app.window_title {
            let title = title.trim().to_string();
            if !title.is_empty() {
                tracing::info!(
                    pid = app.pid,
                    bundle_id = %app.bundle_id,
                    title = %title,
                    "context_snapshot: captured focused window from tracker cache"
                );
                return Ok(Some(FocusedWindowContext {
                    title,
                    width: 0,
                    height: 0,
                    used_fallback: false,
                }));
            }
        }
    }

    match crate::platform::capture_focused_window_screenshot() {
        Ok(capture) => {
            tracing::info!(
                title = %capture.window_title,
                width = capture.width,
                height = capture.height,
                "context_snapshot: captured focused window"
            );
            Ok(Some(FocusedWindowContext {
                title: capture.window_title,
                width: capture.width,
                height: capture.height,
                used_fallback: capture.used_fallback,
            }))
        }
        Err(error) => Err(error.to_string()),
    }
}

/// Capture a deterministic snapshot of AI-relevant desktop context.
///
/// Individual providers that fail produce a warning string rather than
/// failing the entire snapshot — callers always get partial results.
pub fn capture_context_snapshot(options: &CaptureContextOptions) -> AiContextSnapshot {
    let seed = if DETERMINISTIC_CONTEXT.load(Ordering::SeqCst) {
        tracing::info!("context_snapshot: using deterministic seed (test mode)");
        CaptureContextSeed::default()
    } else {
        CaptureContextSeed {
            selected_text: if options.include_selected_text {
                capture_selected_text_live()
            } else {
                Ok(None)
            },
            frontmost_app: if options.include_frontmost_app {
                capture_frontmost_app_live()
            } else {
                Ok(None)
            },
            menu_bar_items: if options.include_menu_bar {
                capture_menu_bar_live()
            } else {
                Ok(Vec::new())
            },
            browser: if options.include_browser_url {
                capture_browser_live()
            } else {
                Ok(None)
            },
            focused_window: if options.include_focused_window {
                capture_focused_window_live()
            } else {
                Ok(None)
            },
        }
    };

    let snapshot = capture_context_snapshot_from_seed(options, seed);

    tracing::info!(
        warnings = snapshot.warnings.len(),
        "context_snapshot: capture complete"
    );
    snapshot
}

/// Capture and serialize snapshot as pretty-printed JSON.
#[allow(dead_code)] // Public API surface for lib consumers
pub fn capture_context_snapshot_json(options: &CaptureContextOptions) -> anyhow::Result<String> {
    serde_json::to_string_pretty(&capture_context_snapshot(options))
        .context("Failed to serialize context snapshot")
}
