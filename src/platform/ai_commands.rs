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

/// Get the URL of the currently focused browser tab on Windows.
///
/// Uses a two-phase approach:
/// 1. **Primary (Ctrl+L, Ctrl+C):** Sends keyboard shortcuts to focus the address bar
///    and copy the URL to the clipboard. Works with Chrome, Edge, Firefox, Brave, Arc, Vivaldi, Opera.
/// 2. **Fallback (window title):** If the clipboard approach fails, attempts to parse the URL
///    from the browser's window title (some browsers include the page title which is less useful,
///    but Edge/Chrome sometimes show the URL).
///
/// # Caveats
/// - The target browser must be the foreground window when this is called.
/// - Briefly disturbs the clipboard (saves and restores the original content).
/// - UAC-elevated browser windows cannot receive synthetic input from non-elevated processes.
/// - Some browsers with custom input handling may not respond to Ctrl+L.
#[cfg(target_os = "windows")]
pub fn get_focused_browser_tab_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    windows_browser_url::get_browser_url_impl()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn get_focused_browser_tab_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Err("Browser URL retrieval is only supported on macOS and Windows".into())
}

// ============================================================================
// Windows Browser URL Implementation
// ============================================================================

#[cfg(target_os = "windows")]
mod windows_browser_url {
    use std::os::raw::c_int;

    // ── Win32 FFI ──────────────────────────────────────────────────────
    //
    // Follows the codebase convention of raw extern blocks
    // (see selected_text.rs, windows_system_actions.rs, window_control_stub.rs).
    #[allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms)]
    mod ffi {
        use std::os::raw::c_int;

        pub type HWND = *mut std::ffi::c_void;
        pub type DWORD = u32;

        // ── SendInput types ────────────────────────────────────────
        pub const INPUT_KEYBOARD: u32 = 1;
        pub const KEYEVENTF_KEYUP: u32 = 0x0002;

        pub const VK_CONTROL: u16 = 0x11;
        pub const VK_ESCAPE: u16 = 0x1B;
        pub const VK_L: u16 = 0x4C;
        pub const VK_C: u16 = 0x43;

        #[repr(C)]
        pub struct KEYBDINPUT {
            pub wVk: u16,
            pub wScan: u16,
            pub dwFlags: u32,
            pub time: u32,
            pub dwExtraInfo: usize,
        }

        /// Padded union body — matches the largest union member (MOUSEINPUT = 28 bytes on x64).
        /// KEYBDINPUT is 20 bytes on x64, so 8 bytes of padding are needed.
        #[repr(C)]
        pub struct INPUT_UNION {
            pub ki: KEYBDINPUT,
            pub _pad: [u8; 8],
        }

        #[repr(C)]
        pub struct INPUT {
            pub r#type: u32,
            pub u: INPUT_UNION,
        }

        extern "system" {
            pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: c_int) -> u32;
            pub fn GetForegroundWindow() -> HWND;
            pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: c_int) -> c_int;
            pub fn GetWindowTextLengthW(hWnd: HWND) -> c_int;
            pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
        }

        // Process name lookup
        extern "system" {
            pub fn OpenProcess(
                dwDesiredAccess: DWORD,
                bInheritHandle: i32,
                dwProcessId: DWORD,
            ) -> *mut std::ffi::c_void;
            pub fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        }

        #[link(name = "psapi")]
        extern "system" {
            pub fn GetProcessImageFileNameW(
                hProcess: *mut std::ffi::c_void,
                lpImageFileName: *mut u16,
                nSize: DWORD,
            ) -> DWORD;
        }

        pub const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
    }

    // ── Known browser executable names ─────────────────────────────────
    const KNOWN_BROWSERS: &[&str] = &[
        "chrome", "msedge", "firefox", "brave", "arc", "vivaldi", "opera", "chromium",
    ];

    /// Build a SendInput keyboard event.
    fn make_key_input(vk: u16, flags: u32) -> ffi::INPUT {
        ffi::INPUT {
            r#type: ffi::INPUT_KEYBOARD,
            u: ffi::INPUT_UNION {
                ki: ffi::KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
                _pad: [0u8; 8],
            },
        }
    }

    /// Send a sequence of keyboard inputs via Win32 SendInput.
    fn send_inputs(inputs: &[ffi::INPUT]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sent = unsafe {
            ffi::SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<ffi::INPUT>() as c_int,
            )
        };
        if sent != inputs.len() as u32 {
            return Err(format!(
                "SendInput failed: sent {sent} of {} inputs (OS error: {})",
                inputs.len(),
                std::io::Error::last_os_error()
            )
            .into());
        }
        Ok(())
    }

    /// Get the foreground window's process executable name (lowercase, no extension).
    fn get_foreground_process_name() -> Option<String> {
        unsafe {
            let hwnd = ffi::GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }

            let mut pid: ffi::DWORD = 0;
            ffi::GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return None;
            }

            let handle = ffi::OpenProcess(ffi::PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return None;
            }

            let mut buf = vec![0u16; 1024];
            let len = ffi::GetProcessImageFileNameW(handle, buf.as_mut_ptr(), buf.len() as u32);
            ffi::CloseHandle(handle);

            if len == 0 {
                return None;
            }

            let full_path = String::from_utf16_lossy(&buf[..len as usize]);
            let name = full_path
                .rsplit('\\')
                .next()
                .unwrap_or(&full_path)
                .trim_end_matches(".exe")
                .trim_end_matches(".EXE")
                .to_lowercase();
            Some(name)
        }
    }

    /// Get the foreground window's title text.
    fn get_foreground_window_title() -> Option<String> {
        unsafe {
            let hwnd = ffi::GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let len = ffi::GetWindowTextLengthW(hwnd);
            if len == 0 {
                return None;
            }
            let mut buf = vec![0u16; (len + 1) as usize];
            let copied = ffi::GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as c_int);
            if copied > 0 {
                Some(String::from_utf16_lossy(&buf[..copied as usize]))
            } else {
                None
            }
        }
    }

    /// Check if the foreground window belongs to a known browser.
    fn is_foreground_browser() -> Option<String> {
        let name = get_foreground_process_name()?;
        if KNOWN_BROWSERS.iter().any(|b| name.contains(b)) {
            Some(name)
        } else {
            None
        }
    }

    /// Validate that a string looks like a URL.
    pub(super) fn looks_like_url(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        // Must start with a scheme or be a bare domain
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("file://")
            || trimmed.starts_with("ftp://")
            || trimmed.starts_with("chrome://")
            || trimmed.starts_with("edge://")
            || trimmed.starts_with("about:")
            || trimmed.starts_with("chrome-extension://")
            || trimmed.starts_with("moz-extension://")
        {
            return true;
        }
        // Bare domain heuristic: contains a dot and no spaces
        if !trimmed.contains(' ') && trimmed.contains('.') {
            // Basic check: at least one character before and after the dot
            let parts: Vec<&str> = trimmed.splitn(2, '.').collect();
            if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                return true;
            }
        }
        false
    }

    /// Primary approach: Ctrl+L → Ctrl+C → read clipboard → Escape.
    ///
    /// Saves and restores the clipboard content to avoid data loss.
    fn capture_url_via_keyboard() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use arboard::Clipboard;

        // 1. Save current clipboard text
        let mut clipboard = Clipboard::new()?;
        let original_clipboard = clipboard.get_text().ok();

        // Clear clipboard so we can detect if Ctrl+C actually copied something
        let _ = clipboard.set_text(String::new());

        // 2. Send Ctrl+L to focus the address bar
        let ctrl_l = [
            make_key_input(ffi::VK_CONTROL, 0),
            make_key_input(ffi::VK_L, 0),
            make_key_input(ffi::VK_L, ffi::KEYEVENTF_KEYUP),
            make_key_input(ffi::VK_CONTROL, ffi::KEYEVENTF_KEYUP),
        ];
        send_inputs(&ctrl_l)?;

        // 3. Brief delay for address bar to focus and select its content
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 4. Send Ctrl+C to copy the URL
        let ctrl_c = [
            make_key_input(ffi::VK_CONTROL, 0),
            make_key_input(ffi::VK_C, 0),
            make_key_input(ffi::VK_C, ffi::KEYEVENTF_KEYUP),
            make_key_input(ffi::VK_CONTROL, ffi::KEYEVENTF_KEYUP),
        ];
        send_inputs(&ctrl_c)?;

        // 5. Brief delay for clipboard to be populated
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 6. Read the clipboard
        let mut clipboard = Clipboard::new()?;
        let url = clipboard.get_text().unwrap_or_default().trim().to_string();

        // 7. Send Escape to unfocus the address bar (restore normal browsing state)
        let escape = [
            make_key_input(ffi::VK_ESCAPE, 0),
            make_key_input(ffi::VK_ESCAPE, ffi::KEYEVENTF_KEYUP),
        ];
        send_inputs(&escape)?;

        // 8. Restore original clipboard content
        if let Some(original) = original_clipboard {
            // Small delay before restoring to avoid race with Escape handling
            std::thread::sleep(std::time::Duration::from_millis(50));
            let mut clipboard = Clipboard::new()?;
            let _ = clipboard.set_text(original);
        }

        // 9. Validate the result
        if url.is_empty() {
            return Err("Clipboard was empty after Ctrl+L, Ctrl+C — no URL captured".into());
        }
        if !looks_like_url(&url) {
            return Err(format!(
                "Clipboard content doesn't look like a URL: {:?}",
                if url.len() > 80 { &url[..80] } else { &url }
            )
            .into());
        }

        tracing::debug!(url = %url, "Browser URL captured via keyboard shortcut");
        Ok(url)
    }

    /// Fallback: attempt to extract URL from the browser window title.
    ///
    /// Most Chromium browsers set the title to "<page_title> - <browser_name>".
    /// This doesn't give us the URL directly, but we return the title as context.
    /// Some browsers in minimal/kiosk mode may show the URL in the title.
    fn capture_url_from_window_title() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let title = get_foreground_window_title().ok_or("No foreground window title available")?;

        // Some browsers show URLs in the title bar (rare, but possible in kiosk mode)
        if looks_like_url(&title) {
            tracing::debug!(url = %title, "Browser URL found in window title");
            return Ok(title);
        }

        // Chromium browsers: "Page Title - Google Chrome" / "Page Title - Microsoft Edge"
        // Try to find a URL embedded in the title
        for part in title.split(&['-', '—', '|'][..]) {
            let trimmed = part.trim();
            if looks_like_url(trimmed) {
                tracing::debug!(url = %trimmed, "Browser URL extracted from window title segment");
                return Ok(trimmed.to_string());
            }
        }

        Err(format!(
            "Window title does not contain a URL: {:?}",
            if title.len() > 100 {
                &title[..100]
            } else {
                &title
            }
        )
        .into())
    }

    /// Main entry point: detect browser → try keyboard capture → fallback to title.
    pub(super) fn get_browser_url_impl() -> Result<String, Box<dyn std::error::Error + Send + Sync>>
    {
        // Step 1: Verify the foreground window is a browser
        let browser_name = is_foreground_browser().ok_or_else(|| {
            let proc_name = get_foreground_process_name().unwrap_or_else(|| "<unknown>".into());
            format!(
                "Foreground application '{}' is not a supported browser. \
                 Supported: Chrome, Edge, Firefox, Brave, Arc, Vivaldi, Opera, Chromium",
                proc_name
            )
        })?;

        tracing::debug!(browser = %browser_name, "Detected foreground browser, capturing URL");

        // Step 2: Try keyboard shortcut approach (Ctrl+L → Ctrl+C)
        match capture_url_via_keyboard() {
            Ok(url) => return Ok(url),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    browser = %browser_name,
                    "Keyboard URL capture failed, trying window title fallback"
                );
            }
        }

        // Step 3: Fallback to window title parsing
        capture_url_from_window_title()
    }

    // ========================================================================
    // Tests
    // ========================================================================

    #[cfg(test)]
    mod tests {
        use super::*;

        // ── URL validation tests ───────────────────────────────────────

        #[test]
        fn test_looks_like_url_http() {
            assert!(looks_like_url("https://www.google.com"));
            assert!(looks_like_url("http://localhost:3000"));
            assert!(looks_like_url("https://example.com/path?q=1&r=2#section"));
        }

        #[test]
        fn test_looks_like_url_ftp() {
            assert!(looks_like_url("ftp://files.example.com/pub/"));
            assert!(looks_like_url("ftp://192.168.1.1/share"));
        }

        #[test]
        fn test_looks_like_url_special_schemes() {
            assert!(looks_like_url("chrome://settings"));
            assert!(looks_like_url("edge://flags"));
            assert!(looks_like_url("about:blank"));
            assert!(looks_like_url("file:///C:/Users/test/file.html"));
            assert!(looks_like_url("chrome-extension://abcdef/popup.html"));
            assert!(looks_like_url("moz-extension://abcdef/popup.html"));
        }

        #[test]
        fn test_looks_like_url_bare_domains() {
            assert!(looks_like_url("google.com"));
            assert!(looks_like_url("www.github.com"));
            assert!(looks_like_url("localhost.test"));
        }

        #[test]
        fn test_looks_like_url_rejects_non_urls() {
            assert!(!looks_like_url(""));
            assert!(!looks_like_url("   "));
            assert!(!looks_like_url("hello world"));
            assert!(!looks_like_url("just some text"));
            assert!(!looks_like_url("Google Chrome"));
            assert!(!looks_like_url(".hidden"));
            assert!(!looks_like_url("noextension"));
        }

        #[test]
        fn test_looks_like_url_rejects_html_fragments() {
            assert!(!looks_like_url("<div>hello</div>"));
            assert!(!looks_like_url("<a href='http://example.com'>link</a>"));
            assert!(!looks_like_url("Some paragraph text with no URL"));
        }

        #[test]
        fn test_looks_like_url_edge_cases() {
            // Whitespace around valid URLs
            assert!(looks_like_url("  https://example.com  "));
            assert!(looks_like_url("\thttps://example.com\n"));
            // Single dot is not a URL
            assert!(!looks_like_url("."));
            // Just a dot with no TLD
            assert!(!looks_like_url("foo."));
            // Starting with dot
            assert!(!looks_like_url(".foo"));
        }

        #[test]
        fn test_looks_like_url_complex_paths() {
            assert!(looks_like_url(
                "https://github.com/user/repo/blob/main/src/lib.rs#L42"
            ));
            assert!(looks_like_url(
                "http://localhost:8080/api/v2/users?page=1&limit=50"
            ));
            assert!(looks_like_url(
                "https://example.com/path/to/page.html?foo=bar&baz=qux#anchor"
            ));
        }

        // ── SendInput structure layout tests ───────────────────────────

        #[test]
        fn test_input_struct_size() {
            // On x64, INPUT should be 4 (type) + 4 (padding) + 28 (union) = 40 bytes
            // This ensures we don't get SendInput failures from misaligned structs.
            let size = std::mem::size_of::<ffi::INPUT>();
            // The exact size depends on architecture. On x64 it's 40.
            // On x86 it would be 28. Just ensure it's reasonable.
            assert!(
                (28..=48).contains(&size),
                "INPUT struct size {size} is outside expected range [28, 48]"
            );
        }

        #[test]
        fn test_keybdinput_struct_size() {
            // KEYBDINPUT: u16 + u16 + u32 + u32 + usize = 4 + 4 + 8 + 8(pad) = 24 on x64
            let size = std::mem::size_of::<ffi::KEYBDINPUT>();
            assert!(
                (16..=32).contains(&size),
                "KEYBDINPUT struct size {size} is outside expected range"
            );
        }

        #[test]
        fn test_input_union_includes_padding() {
            let union_size = std::mem::size_of::<ffi::INPUT_UNION>();
            let ki_size = std::mem::size_of::<ffi::KEYBDINPUT>();
            // INPUT_UNION = KEYBDINPUT + 8 bytes padding
            assert!(
                union_size >= ki_size,
                "INPUT_UNION ({union_size}) must be >= KEYBDINPUT ({ki_size})"
            );
        }

        #[test]
        fn test_input_alignment() {
            let align = std::mem::align_of::<ffi::INPUT>();
            // Must be at least pointer-aligned for SendInput
            assert!(
                align >= std::mem::size_of::<usize>(),
                "INPUT alignment {align} must be at least pointer size"
            );
        }

        #[test]
        fn test_make_key_input_ctrl_l() {
            let input = make_key_input(ffi::VK_L, 0);
            assert_eq!(input.r#type, ffi::INPUT_KEYBOARD);
            assert_eq!(input.u.ki.wVk, ffi::VK_L);
            assert_eq!(input.u.ki.dwFlags, 0);
        }

        #[test]
        fn test_make_key_input_keyup() {
            let input = make_key_input(ffi::VK_CONTROL, ffi::KEYEVENTF_KEYUP);
            assert_eq!(input.r#type, ffi::INPUT_KEYBOARD);
            assert_eq!(input.u.ki.wVk, ffi::VK_CONTROL);
            assert_eq!(input.u.ki.dwFlags, ffi::KEYEVENTF_KEYUP);
        }

        #[test]
        fn test_make_key_input_escape() {
            let input = make_key_input(ffi::VK_ESCAPE, 0);
            assert_eq!(input.r#type, ffi::INPUT_KEYBOARD);
            assert_eq!(input.u.ki.wVk, ffi::VK_ESCAPE);
            assert_eq!(input.u.ki.dwFlags, 0);
            assert_eq!(input.u.ki.wScan, 0);
            assert_eq!(input.u.ki.time, 0);
            assert_eq!(input.u.ki.dwExtraInfo, 0);
        }

        #[test]
        fn test_make_key_input_c_key() {
            let input = make_key_input(ffi::VK_C, 0);
            assert_eq!(input.u.ki.wVk, ffi::VK_C);
            assert_eq!(input.u.ki.wVk, 0x43);
        }

        // ── Virtual key constant correctness ───────────────────────────

        #[test]
        fn test_vk_constants_match_win32_spec() {
            assert_eq!(ffi::VK_CONTROL, 0x11);
            assert_eq!(ffi::VK_ESCAPE, 0x1B);
            assert_eq!(ffi::VK_L, 0x4C);
            assert_eq!(ffi::VK_C, 0x43);
            assert_eq!(ffi::INPUT_KEYBOARD, 1);
            assert_eq!(ffi::KEYEVENTF_KEYUP, 0x0002);
        }

        #[test]
        fn test_process_query_limited_information() {
            assert_eq!(ffi::PROCESS_QUERY_LIMITED_INFORMATION, 0x1000);
        }

        // ── Browser detection tests ────────────────────────────────────

        #[test]
        fn test_known_browsers_list_is_not_empty() {
            // Use direct length check to avoid clippy::const_is_empty (is_empty on const)
            // and clippy::len_zero (len() > 0)
            assert_ne!(KNOWN_BROWSERS.len(), 0);
        }

        #[test]
        fn test_known_browsers_contains_major_browsers() {
            assert!(KNOWN_BROWSERS.contains(&"chrome"));
            assert!(KNOWN_BROWSERS.contains(&"msedge"));
            assert!(KNOWN_BROWSERS.contains(&"firefox"));
            assert!(KNOWN_BROWSERS.contains(&"brave"));
        }

        #[test]
        fn test_known_browsers_contains_all_expected() {
            // All browsers listed in the doc comment for get_focused_browser_tab_url
            let expected = [
                "chrome", "msedge", "firefox", "brave", "arc", "vivaldi", "opera", "chromium",
            ];
            for browser in &expected {
                assert!(
                    KNOWN_BROWSERS.contains(browser),
                    "KNOWN_BROWSERS should contain '{browser}'"
                );
            }
        }

        #[test]
        fn test_known_browsers_are_lowercase() {
            for browser in KNOWN_BROWSERS {
                assert_eq!(
                    *browser,
                    browser.to_lowercase(),
                    "Browser name '{browser}' should be lowercase"
                );
            }
        }

        // ── Error resilience tests ─────────────────────────────────────

        #[test]
        fn test_get_browser_url_returns_error_when_no_browser() {
            // When running in a test harness, there's likely no foreground browser.
            // Verify we get a clear error rather than a panic.
            let result = get_browser_url_impl();
            // Should be Err (no browser focused during test) — the important thing
            // is that it doesn't panic.
            assert!(
                result.is_err(),
                "Expected error when no browser is focused, got: {:?}",
                result
            );
        }

        #[test]
        fn test_get_browser_url_error_mentions_supported_browsers() {
            let result = get_browser_url_impl();
            if let Err(e) = result {
                let msg = e.to_string();
                // The error should be descriptive — either "not a supported browser",
                // "Supported", or "does not contain a URL" (when a browser is focused
                // but the title lacks a parseable URL).
                assert!(
                    msg.contains("not a supported browser")
                        || msg.contains("Supported")
                        || msg.contains("does not contain a URL"),
                    "Error should mention supported browsers or URL parsing issue, got: {msg}"
                );
            }
        }

        #[test]
        fn test_is_foreground_browser_in_test_context() {
            // In test context, the foreground window is likely the terminal/IDE,
            // not a browser. This should return None without panicking.
            let result = is_foreground_browser();
            // We don't assert the value (it depends on what's focused),
            // but it must not panic.
            let _ = result;
        }

        #[test]
        fn test_get_foreground_process_name_does_not_panic() {
            // Should return Some(name) or None, never panic
            let result = get_foreground_process_name();
            if let Some(ref name) = result {
                assert!(!name.is_empty(), "Process name should not be empty if Some");
            }
        }

        #[test]
        fn test_get_foreground_window_title_does_not_panic() {
            // Should return Some(title) or None, never panic
            let result = get_foreground_window_title();
            let _ = result;
        }
    }
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
