// ============================================================================
// Screenshot Capture
// ============================================================================

/// Capture a screenshot of the app window using xcap for cross-platform support.
///
/// Returns a tuple of (png_data, width, height) on success.
/// The function:
/// 1. Uses xcap::Window::all() to enumerate windows
/// 2. Finds the Script Kit window by app name or title
/// 3. Captures the window directly to an image buffer
/// 4. Optionally scales down to 1x resolution if hi_dpi is false
/// 5. Encodes to PNG in memory (no temp files)
///
/// # Arguments
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
pub fn capture_app_screenshot(
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    struct Candidate {
        window: Window,
        title: String,
        width: u32,
        height: u32,
    }

    let mut candidates = Vec::new();
    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match our app window by name
        let is_our_window = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || title.contains("Script Kit");

        let is_minimized = window.is_minimized().unwrap_or(true);

        // Get window dimensions to filter out tiny windows (tooltips, list items, etc.)
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);

        // Only consider windows that are reasonably sized
        // Width >= 200 filters out small UI elements
        // Height >= 50 allows compact prompts (arg prompt without choices is ~76px)
        let is_reasonable_size = width >= 200 && height >= 50;

        if is_our_window && !is_minimized && is_reasonable_size {
            candidates.push(Candidate {
                window,
                title,
                width,
                height,
            });
        }
    }

    // Sort by size (largest first) - the main window is typically the largest
    candidates.sort_by(|a, b| {
        let area_a = a.width as u64 * a.height as u64;
        let area_b = b.width as u64 * b.height as u64;
        area_b.cmp(&area_a)
    });

    let mut target = candidates
        .iter()
        .filter(|candidate| candidate.title.contains("Notes") || candidate.title.contains("AI"))
        .find(|candidate| candidate.window.is_focused().unwrap_or(false))
        .map(|candidate| candidate.window.clone());

    if target.is_none() {
        target = candidates
            .iter()
            .find(|candidate| candidate.title.contains("Notes") || candidate.title.contains("AI"))
            .map(|candidate| candidate.window.clone());
    }

    if target.is_none() {
        target = candidates
            .iter()
            .find(|candidate| candidate.window.is_focused().unwrap_or(false))
            .map(|candidate| candidate.window.clone());
    }

    let Some(window) =
        target.or_else(|| candidates.first().map(|candidate| candidate.window.clone()))
    else {
        return Err("Script Kit window not found".into());
    };

    let title = window.title().unwrap_or_else(|_| String::new());
    let app_name = window.app_name().unwrap_or_else(|_| String::new());

    tracing::debug!(
        app_name = %app_name,
        title = %title,
        hi_dpi = hi_dpi,
        "Found Script Kit window for screenshot"
    );

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    // Scale down to 1x if not hi_dpi mode (xcap captures at retina resolution on macOS)
    let (final_image, width, height) = if hi_dpi {
        (image, original_width, original_height)
    } else {
        // Scale down by 2x for 1x resolution
        let new_width = original_width / 2;
        let new_height = original_height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        tracing::debug!(
            original_width = original_width,
            original_height = original_height,
            new_width = new_width,
            new_height = new_height,
            "Scaled screenshot to 1x resolution"
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory (no temp files needed)
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    tracing::debug!(
        width = width,
        height = height,
        hi_dpi = hi_dpi,
        file_size = png_data.len(),
        "Screenshot captured with xcap"
    );

    Ok((png_data, width, height))
}

/// Capture a screenshot of a window by its title pattern.
///
/// Similar to `capture_app_screenshot` but allows specifying which window to capture
/// by matching the title. This is useful for secondary windows like the AI Chat window.
///
/// # Arguments
/// * `title_pattern` - A string that the window title must contain
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
///
/// # Returns
/// A tuple of (png_data, width, height) on success.
pub fn capture_window_by_title(
    title_pattern: &str,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match window by title pattern (must also be our app)
        let is_our_app = app_name.contains("script-kit-gpui") || app_name == "Script Kit";
        let title_matches = title.contains(title_pattern);
        let is_minimized = window.is_minimized().unwrap_or(true);
        // Skip tiny windows (e.g. tray icon) when using empty title pattern
        let win_width = window.width().unwrap_or(0);
        let win_height = window.height().unwrap_or(0);
        let is_too_small = win_width < 100 || win_height < 100;

        if is_our_app && title_matches && !is_minimized && !is_too_small {
            tracing::debug!(
                app_name = %app_name,
                title = %title,
                title_pattern = %title_pattern,
                hi_dpi = hi_dpi,
                "Found window matching title pattern for screenshot"
            );

            let image = window.capture_image()?;
            let original_width = image.width();
            let original_height = image.height();

            // Scale down to 1x if not hi_dpi mode
            let (final_image, width, height) = if hi_dpi {
                (image, original_width, original_height)
            } else {
                let new_width = original_width / 2;
                let new_height = original_height / 2;
                let resized = image::imageops::resize(
                    &image,
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,
                );
                tracing::debug!(
                    original_width = original_width,
                    original_height = original_height,
                    new_width = new_width,
                    new_height = new_height,
                    "Scaled screenshot to 1x resolution"
                );
                (resized, new_width, new_height)
            };

            // Encode to PNG in memory
            let mut png_data = Vec::new();
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

            tracing::debug!(
                width = width,
                height = height,
                hi_dpi = hi_dpi,
                file_size = png_data.len(),
                title_pattern = %title_pattern,
                "Screenshot captured for window by title"
            );

            return Ok((png_data, width, height));
        }
    }

    Err(format!("Window with title containing '{}' not found", title_pattern).into())
}

// ============================================================================
// Open Path with System Default
// ============================================================================

/// Open a path (file or folder) with the system default application.
/// On macOS: uses `open` command
/// On Linux: uses `xdg-open` command
/// On Windows: uses `cmd /C start` command
///
/// This can be used to open files, folders, URLs, or any path that the
/// system knows how to handle.
#[allow(dead_code)]
pub fn open_path_with_system_default(path: &str) {
    logging::log("UI", &format!("Opening path with system default: {}", path));
    let path_owned = path.to_string();

    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        {
            match std::process::Command::new("open").arg(&path_owned).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }

        #[cfg(target_os = "linux")]
        {
            match std::process::Command::new("xdg-open")
                .arg(&path_owned)
                .spawn()
            {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }

        #[cfg(target_os = "windows")]
        {
            match std::process::Command::new("cmd")
                .args(["/C", "start", "", &path_owned])
                .spawn()
            {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }
    });
}

