// ============================================================================
// Screen Capture for AI Commands
// ============================================================================

/// Capture a screenshot of the entire primary screen.
///
/// # Returns
/// A tuple of (png_data, width, height) on success.
pub fn capture_screen_screenshot(
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Monitor;

    let monitors = Monitor::all()?;

    // Get the primary monitor (first one, usually the main display)
    let monitor = monitors.into_iter().next().ok_or("No monitors found")?;

    tracing::debug!(
        name = %monitor.name().unwrap_or_default(),
        "Capturing primary monitor screenshot"
    );

    let image = monitor.capture_image()?;
    let width = image.width();
    let height = image.height();

    // Scale down to 1x for efficiency (monitors capture at retina resolution on macOS)
    let (final_image, final_width, final_height) = {
        let new_width = width / 2;
        let new_height = height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(
        &final_image,
        final_width,
        final_height,
        image::ExtendedColorType::Rgba8,
    )?;

    tracing::debug!(
        width = final_width,
        height = final_height,
        file_size = png_data.len(),
        "Screen screenshot captured"
    );

    Ok((png_data, final_width, final_height))
}

/// Result of a focused window capture, including whether a fallback was used.
pub struct FocusedWindowCapture {
    pub png_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub window_title: String,
    /// True if no focused window was found and we fell back to the first available window.
    pub used_fallback: bool,
}

/// Capture a screenshot of the currently focused window (not our app).
///
/// This function finds the frontmost window that is NOT Script Kit and captures it.
/// If no focused window is found, it falls back to the first available window and
/// sets `used_fallback = true` so the caller can warn the user.
///
/// # Returns
/// A `FocusedWindowCapture` on success.
pub fn capture_focused_window_screenshot(
) -> Result<FocusedWindowCapture, Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    // Find the focused window that is NOT our app
    let mut target_window = None;
    let mut found_focused = false;
    for window in windows {
        let app_name = window.app_name().unwrap_or_else(|_| String::new());
        let is_minimized = window.is_minimized().unwrap_or(true);
        let is_focused = window.is_focused().unwrap_or(false);

        // Skip our own app
        let is_our_app = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || app_name.contains("Script Kit");

        // Get window dimensions - skip tiny windows
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);
        let is_reasonable_size = width >= 100 && height >= 100;

        if !is_our_app && !is_minimized && is_reasonable_size {
            if is_focused {
                target_window = Some(window);
                found_focused = true;
                break;
            }
            // Keep the first reasonable non-our-app window as fallback
            if target_window.is_none() {
                target_window = Some(window);
            }
        }
    }

    let used_fallback = target_window.is_some() && !found_focused;

    let window = target_window.ok_or("No suitable window found to capture")?;
    let title = window.title().unwrap_or_else(|_| "Unknown".to_string());
    let app_name = window.app_name().unwrap_or_else(|_| "Unknown".to_string());

    if used_fallback {
        tracing::warn!(
            app_name = %app_name,
            title = %title,
            "No focused window found, falling back to first available window"
        );
    } else {
        tracing::debug!(
            app_name = %app_name,
            title = %title,
            "Capturing focused window screenshot"
        );
    }

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    // Scale down to 1x for efficiency
    let (final_image, width, height) = {
        let new_width = original_width / 2;
        let new_height = original_height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    let display_title = if title.is_empty() {
        app_name
    } else {
        format!("{} - {}", app_name, title)
    };

    tracing::debug!(
        width = width,
        height = height,
        file_size = png_data.len(),
        title = %display_title,
        used_fallback = used_fallback,
        "Focused window screenshot captured"
    );

    Ok(FocusedWindowCapture {
        png_data,
        width,
        height,
        window_title: display_title,
        used_fallback,
    })
}

/// Capture a screenshot of Script Kit's own visible panel window.
///
/// Unlike `capture_focused_window_screenshot()`, this helper intentionally
/// targets our own window so Tab AI can send the launcher state to the harness.
///
/// # Returns
/// A `FocusedWindowCapture` on success with `used_fallback = false`.
pub fn capture_script_kit_panel_screenshot(
) -> Result<FocusedWindowCapture, Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    let mut best: Option<(u32, xcap::Window)> = None;
    for window in windows {
        let app_name = window.app_name().unwrap_or_else(|_| String::new());
        let is_minimized = window.is_minimized().unwrap_or(true);
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);
        let area = width.saturating_mul(height);

        let is_our_app = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || app_name.contains("Script Kit");

        if !is_our_app || is_minimized || width < 100 || height < 100 {
            continue;
        }

        let should_replace = best
            .as_ref()
            .map(|(best_area, _)| area > *best_area)
            .unwrap_or(true);
        if should_replace {
            best = Some((area, window));
        }
    }

    let (_, window) = best.ok_or("No Script Kit panel window found")?;
    let title = window.title().unwrap_or_else(|_| "Panel".to_string());
    let app_name = window.app_name().unwrap_or_else(|_| "Script Kit".to_string());

    tracing::debug!(
        app_name = %app_name,
        title = %title,
        "Capturing Script Kit panel screenshot"
    );

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    // Scale down to 1x for efficiency (retina)
    let width = (original_width / 2).max(1);
    let height = (original_height / 2).max(1);
    let resized = image::imageops::resize(
        &image,
        width,
        height,
        image::imageops::FilterType::Lanczos3,
    );

    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&resized, width, height, image::ExtendedColorType::Rgba8)?;

    let display_title = format!("Script Kit - {}", title);

    tracing::debug!(
        width = width,
        height = height,
        file_size = png_data.len(),
        title = %display_title,
        "Script Kit panel screenshot captured"
    );

    Ok(FocusedWindowCapture {
        png_data,
        width,
        height,
        window_title: display_title,
        used_fallback: false,
    })
}

/// Get the URL of the currently focused browser tab.
///
/// Supports Safari, Google Chrome, Arc, Brave, Firefox, and Edge.
///
/// # Returns
/// The URL string on success.
#[cfg(target_os = "macos")]
pub fn get_focused_browser_tab_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::process::Command;

    // First, get the frontmost application name
    let frontmost_script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            return frontApp
        end tell
    "#;

    let frontmost_output = Command::new("osascript")
        .arg("-e")
        .arg(frontmost_script)
        .output()?;

    if !frontmost_output.status.success() {
        return Err("Failed to get frontmost application".into());
    }

    let frontmost_app = String::from_utf8_lossy(&frontmost_output.stdout)
        .trim()
        .to_string();

    tracing::debug!(app = %frontmost_app, "Detected frontmost browser");

    // Map process name to application name and the AppleScript to get URL
    let (app_name, url_script) = match frontmost_app.as_str() {
        "Safari" => (
            "Safari",
            r#"tell application "Safari" to return URL of front document"#,
        ),
        "Google Chrome" => (
            "Google Chrome",
            r#"tell application "Google Chrome" to return URL of active tab of front window"#,
        ),
        "Arc" => (
            "Arc",
            r#"tell application "Arc" to return URL of active tab of front window"#,
        ),
        "Brave Browser" => (
            "Brave Browser",
            r#"tell application "Brave Browser" to return URL of active tab of front window"#,
        ),
        "Firefox" => {
            // Firefox doesn't support AppleScript well - return an error with helpful message
            return Err("Firefox doesn't fully support AppleScript for URL retrieval. Try Safari or Chrome.".into());
        }
        "Microsoft Edge" => (
            "Microsoft Edge",
            r#"tell application "Microsoft Edge" to return URL of active tab of front window"#,
        ),
        "Chromium" => (
            "Chromium",
            r#"tell application "Chromium" to return URL of active tab of front window"#,
        ),
        "Vivaldi" => (
            "Vivaldi",
            r#"tell application "Vivaldi" to return URL of active tab of front window"#,
        ),
        "Opera" => (
            "Opera",
            r#"tell application "Opera" to return URL of active tab of front window"#,
        ),
        _ => {
            return Err(format!(
                "Frontmost app '{}' is not a supported browser. Supported: Safari, Chrome, Arc, Brave, Edge, Vivaldi, Opera",
                frontmost_app
            ).into());
        }
    };

    tracing::debug!(app = %app_name, "Getting URL from browser");

    let url_output = Command::new("osascript")
        .arg("-e")
        .arg(url_script)
        .output()?;

    if !url_output.status.success() {
        let stderr = String::from_utf8_lossy(&url_output.stderr);
        return Err(format!("Failed to get URL from {}: {}", app_name, stderr).into());
    }

    let url = String::from_utf8_lossy(&url_output.stdout)
        .trim()
        .to_string();

    if url.is_empty() {
        return Err(format!("No URL found in {}", app_name).into());
    }

    tracing::debug!(url = %url, app = %app_name, "Browser URL retrieved");

    Ok(url)
}

#[cfg(not(target_os = "macos"))]
pub fn get_focused_browser_tab_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Err("Browser URL retrieval is only supported on macOS".into())
}

// ============================================================================
// Cursor Visibility
// ============================================================================

/// Hide the mouse cursor until the mouse moves.
///
/// This is the standard macOS pattern used by text editors to hide the cursor
/// while typing. The cursor will automatically reappear when the user moves
/// the mouse, with no additional code needed.
///
/// # macOS Behavior
///
/// Calls `[NSCursor setHiddenUntilMouseMoves:YES]` which:
/// - Immediately hides the system cursor
/// - Automatically shows the cursor when the mouse moves
/// - Is idempotent (safe to call multiple times)
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn hide_cursor_until_mouse_moves() {
    // SAFETY: NSCursor.setHiddenUntilMouseMoves: is a class method that is
    // safe to call from any thread (it's one of the few AppKit methods that is).
    // It takes a BOOL value type and returns void.
    unsafe {
        // NSCursor.setHiddenUntilMouseMoves(YES) - hides cursor until mouse moves
        let _: () = msg_send![class!(NSCursor), setHiddenUntilMouseMoves: true];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn hide_cursor_until_mouse_moves() {
    // No-op on non-macOS platforms
}

