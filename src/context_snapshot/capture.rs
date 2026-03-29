use super::types::*;
use anyhow::Context;
use base64::Engine;
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
    pub(crate) focused_window_image: Result<Option<Base64PngContext>, String>,
    pub(crate) script_kit_panel_image: Result<Option<Base64PngContext>, String>,
}

impl Default for CaptureContextSeed {
    fn default() -> Self {
        Self {
            selected_text: Ok(None),
            frontmost_app: Ok(None),
            menu_bar_items: Ok(Vec::new()),
            browser: Ok(None),
            focused_window: Ok(None),
            focused_window_image: Ok(None),
            script_kit_panel_image: Ok(None),
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

    if options.include_screenshot {
        match seed.focused_window_image {
            Ok(Some(img)) => snapshot.focused_window_image = Some(img),
            Ok(None) => {}
            Err(error) => snapshot.warnings.push(format!("screenshot: {error}")),
        }
    }

    if options.include_panel_screenshot {
        match seed.script_kit_panel_image {
            Ok(Some(img)) => snapshot.script_kit_panel_image = Some(img),
            Ok(None) => {}
            Err(error) => snapshot.warnings.push(format!("panelScreenshot: {error}")),
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
        has_screenshot = snapshot.focused_window_image.is_some(),
        has_panel_screenshot = snapshot.script_kit_panel_image.is_some(),
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

/// Capture focused-window metadata and optionally its PNG image bytes.
///
/// When `include_image` is false, tries the fast path (tracker cache with
/// metadata only). When true, always goes through `capture_focused_window_screenshot`
/// to obtain pixel data alongside metadata.
fn capture_focused_window_with_image_live(
    include_image: bool,
) -> Result<(Option<FocusedWindowContext>, Option<Base64PngContext>), String> {
    // Fast path: when screenshot is not needed, use cached metadata
    if !include_image {
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
                    return Ok((
                        Some(FocusedWindowContext {
                            title,
                            width: 0,
                            height: 0,
                            used_fallback: false,
                        }),
                        None,
                    ));
                }
            }
        }
    }

    match crate::platform::capture_focused_window_screenshot() {
        Ok(capture) => {
            tracing::info!(
                title = %capture.window_title,
                width = capture.width,
                height = capture.height,
                include_image,
                "context_snapshot: captured focused window"
            );
            let window = FocusedWindowContext {
                title: capture.window_title.clone(),
                width: capture.width,
                height: capture.height,
                used_fallback: capture.used_fallback,
            };
            let image = if include_image {
                Some(Base64PngContext {
                    mime_type: "image/png".to_string(),
                    width: capture.width,
                    height: capture.height,
                    base64_data: base64::engine::general_purpose::STANDARD
                        .encode(&capture.png_data),
                    title: Some(capture.window_title),
                })
            } else {
                None
            };
            Ok((Some(window), image))
        }
        Err(error) => Err(error.to_string()),
    }
}

/// Attempt to capture Script Kit's own panel window as a base64 PNG.
///
/// Returns `Ok(None)` when the platform helper is unavailable or the panel
/// cannot be found, `Err` when capture was attempted but failed.
fn capture_script_kit_panel_image_live() -> Result<Option<Base64PngContext>, String> {
    match crate::platform::capture_script_kit_panel_screenshot() {
        Ok(capture) => {
            tracing::info!(
                title = %capture.window_title,
                width = capture.width,
                height = capture.height,
                "context_snapshot: captured script kit panel"
            );
            Ok(Some(Base64PngContext {
                mime_type: "image/png".to_string(),
                width: capture.width,
                height: capture.height,
                base64_data: base64::engine::general_purpose::STANDARD.encode(&capture.png_data),
                title: Some(capture.window_title),
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
        // When both focused_window and screenshot are requested, capture once
        // and split metadata from image bytes to avoid a double capture.
        let (focused_window, focused_window_image) =
            if options.include_focused_window || options.include_screenshot {
                match capture_focused_window_with_image_live(options.include_screenshot) {
                    Ok((window, image)) => (Ok(window), Ok(image)),
                    Err(error) => {
                        let err_str = error.to_string();
                        (
                            if options.include_focused_window {
                                Err(err_str.clone())
                            } else {
                                Ok(None)
                            },
                            if options.include_screenshot {
                                Err(err_str)
                            } else {
                                Ok(None)
                            },
                        )
                    }
                }
            } else {
                (Ok(None), Ok(None))
            };

        let script_kit_panel_image = if options.include_panel_screenshot {
            capture_script_kit_panel_image_live()
        } else {
            Ok(None)
        };

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
            focused_window,
            focused_window_image,
            script_kit_panel_image,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_capture_includes_screenshot_when_enabled() {
        let snapshot = capture_context_snapshot_from_seed(
            &CaptureContextOptions {
                include_selected_text: false,
                include_frontmost_app: false,
                include_menu_bar: false,
                include_browser_url: false,
                include_focused_window: true,
                include_screenshot: true,
                include_panel_screenshot: false,
            },
            CaptureContextSeed {
                focused_window: Ok(Some(FocusedWindowContext {
                    title: "Finder - Test".to_string(),
                    width: 640,
                    height: 480,
                    used_fallback: false,
                })),
                focused_window_image: Ok(Some(Base64PngContext {
                    mime_type: "image/png".to_string(),
                    width: 640,
                    height: 480,
                    base64_data: "ZmFrZQ==".to_string(),
                    title: Some("Finder - Test".to_string()),
                })),
                ..Default::default()
            },
        );
        assert_eq!(
            snapshot
                .focused_window
                .as_ref()
                .expect("focused window")
                .title,
            "Finder - Test"
        );
        assert_eq!(
            snapshot
                .focused_window_image
                .as_ref()
                .expect("focused window image")
                .base64_data,
            "ZmFrZQ=="
        );
    }

    #[test]
    fn seed_capture_omits_screenshot_when_disabled() {
        let snapshot = capture_context_snapshot_from_seed(
            &CaptureContextOptions {
                include_selected_text: false,
                include_frontmost_app: false,
                include_menu_bar: false,
                include_browser_url: false,
                include_focused_window: true,
                include_screenshot: false,
                include_panel_screenshot: false,
            },
            CaptureContextSeed {
                focused_window: Ok(Some(FocusedWindowContext {
                    title: "Finder - Test".to_string(),
                    width: 640,
                    height: 480,
                    used_fallback: false,
                })),
                focused_window_image: Ok(Some(Base64PngContext {
                    mime_type: "image/png".to_string(),
                    width: 640,
                    height: 480,
                    base64_data: "ZmFrZQ==".to_string(),
                    title: Some("Finder - Test".to_string()),
                })),
                ..Default::default()
            },
        );
        assert!(snapshot.focused_window.is_some());
        assert!(
            snapshot.focused_window_image.is_none(),
            "focused_window_image must be omitted when include_screenshot=false"
        );
    }

    #[test]
    fn seed_capture_includes_panel_screenshot_when_enabled() {
        let snapshot = capture_context_snapshot_from_seed(
            &CaptureContextOptions {
                include_selected_text: false,
                include_frontmost_app: false,
                include_menu_bar: false,
                include_browser_url: false,
                include_focused_window: true,
                include_screenshot: true,
                include_panel_screenshot: true,
            },
            CaptureContextSeed {
                focused_window: Ok(Some(FocusedWindowContext {
                    title: "Finder - Test".to_string(),
                    width: 640,
                    height: 480,
                    used_fallback: false,
                })),
                focused_window_image: Ok(Some(Base64PngContext {
                    mime_type: "image/png".to_string(),
                    width: 640,
                    height: 480,
                    base64_data: "Zm9jdXNlZA==".to_string(),
                    title: Some("Finder - Test".to_string()),
                })),
                script_kit_panel_image: Ok(Some(Base64PngContext {
                    mime_type: "image/png".to_string(),
                    width: 700,
                    height: 520,
                    base64_data: "cGFuZWw=".to_string(),
                    title: Some("Script Kit - Clipboard History".to_string()),
                })),
                ..Default::default()
            },
        );
        assert_eq!(
            snapshot
                .script_kit_panel_image
                .as_ref()
                .expect("panel image")
                .base64_data,
            "cGFuZWw="
        );
    }

    #[test]
    fn seed_capture_omits_panel_screenshot_when_disabled() {
        let snapshot = capture_context_snapshot_from_seed(
            &CaptureContextOptions {
                include_selected_text: false,
                include_frontmost_app: false,
                include_menu_bar: false,
                include_browser_url: false,
                include_focused_window: true,
                include_screenshot: true,
                include_panel_screenshot: false,
            },
            CaptureContextSeed {
                script_kit_panel_image: Ok(Some(Base64PngContext {
                    mime_type: "image/png".to_string(),
                    width: 700,
                    height: 520,
                    base64_data: "cGFuZWw=".to_string(),
                    title: Some("Script Kit - Clipboard History".to_string()),
                })),
                ..Default::default()
            },
        );
        assert!(
            snapshot.script_kit_panel_image.is_none(),
            "script_kit_panel_image must be omitted when include_panel_screenshot=false"
        );
    }
}
