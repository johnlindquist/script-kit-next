// ============================================================================
// Screenshot Capture
// ============================================================================

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use xcap::Window;

fn encode_rgba_to_png(
    image: &image::RgbaImage,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    const DOWNSCALE_DIVISOR: u32 = 2;

    let original_width = image.width();
    let original_height = image.height();

    let (final_image, width, height) = if hi_dpi {
        (image.clone(), original_width, original_height)
    } else {
        let new_width = (original_width / DOWNSCALE_DIVISOR).max(1);
        let new_height = (original_height / DOWNSCALE_DIVISOR).max(1);
        let resized = image::imageops::resize(
            image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        tracing::debug!(
            original_width = original_width,
            original_height = original_height,
            new_width = new_width,
            new_height = new_height,
            downscale_divisor = DOWNSCALE_DIVISOR,
            "Scaled screenshot to 1x resolution"
        );
        (resized, new_width, new_height)
    };

    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    Ok((png_data, width, height))
}

fn capture_and_encode_png(
    window: &xcap::Window,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let image = window.capture_image()?;
    encode_rgba_to_png(&image, hi_dpi)
}

// ── Windows: direct HWND capture via Win32 PrintWindow ────────────────────
// xcap's Window::all() explicitly skips windows owned by the current process
// (to avoid GetWindowText deadlocks). Since we ARE the current process, we
// must capture our own window directly using the stored HWND.
#[cfg(target_os = "windows")]
mod win32_capture {
    use image::RgbaImage;
    use std::ffi::c_void;

    // Win32 type aliases
    #[allow(clippy::upper_case_acronyms)]
    type HWND = isize;
    #[allow(clippy::upper_case_acronyms)]
    type HDC = isize;
    #[allow(clippy::upper_case_acronyms)]
    type HBITMAP = isize;
    #[allow(clippy::upper_case_acronyms)]
    type HGDIOBJ = isize;
    #[allow(clippy::upper_case_acronyms)]
    type BOOL = i32;

    #[repr(C)]
    #[allow(clippy::upper_case_acronyms)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct BITMAPINFOHEADER {
        biSize: u32,
        biWidth: i32,
        biHeight: i32,
        biPlanes: u16,
        biBitCount: u16,
        biCompression: u32,
        biSizeImage: u32,
        biXPelsPerMeter: i32,
        biYPelsPerMeter: i32,
        biClrUsed: u32,
        biClrImportant: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct RGBQUAD {
        rgbBlue: u8,
        rgbGreen: u8,
        rgbRed: u8,
        rgbReserved: u8,
    }

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER,
        bmiColors: [RGBQUAD; 1],
    }

    const DIB_RGB_COLORS: u32 = 0;
    const SRCCOPY: u32 = 0x00CC0020;
    const PW_RENDERFULLCONTENT: u32 = 2;

    extern "system" {
        fn GetWindowRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
        fn GetClientRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
        fn GetWindowDC(hwnd: HWND) -> HDC;
        fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
        fn CreateCompatibleDC(hdc: HDC) -> HDC;
        fn CreateCompatibleBitmap(hdc: HDC, width: i32, height: i32) -> HBITMAP;
        fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
        fn DeleteObject(obj: HGDIOBJ) -> BOOL;
        fn DeleteDC(hdc: HDC) -> BOOL;
        fn BitBlt(
            dest: HDC,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            src: HDC,
            src_x: i32,
            src_y: i32,
            rop: u32,
        ) -> BOOL;
        fn PrintWindow(hwnd: HWND, hdc: HDC, flags: u32) -> BOOL;
        fn GetDIBits(
            hdc: HDC,
            bitmap: HBITMAP,
            start: u32,
            lines: u32,
            bits: *mut c_void,
            info: *mut BITMAPINFO,
            usage: u32,
        ) -> i32;
        fn IsWindow(hwnd: HWND) -> BOOL;
        fn IsWindowVisible(hwnd: HWND) -> BOOL;
        fn IsIconic(hwnd: HWND) -> BOOL;
    }

    /// Capture a window by its HWND using PrintWindow (works for own-process windows).
    /// Returns an RGBA image on success.
    pub fn capture_hwnd(
        hwnd: isize,
    ) -> Result<RgbaImage, Box<dyn std::error::Error + Send + Sync>> {
        unsafe {
            if IsWindow(hwnd) == 0 {
                return Err("HWND is not a valid window".into());
            }
            if IsWindowVisible(hwnd) == 0 {
                return Err("Window is not visible".into());
            }
            if IsIconic(hwnd) != 0 {
                return Err("Window is minimized".into());
            }

            let mut window_rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut window_rect) == 0 {
                return Err("GetWindowRect failed".into());
            }

            let width = window_rect.right - window_rect.left;
            let height = window_rect.bottom - window_rect.top;
            if width <= 0 || height <= 0 {
                return Err(format!("Invalid window size: {}x{}", width, height).into());
            }

            // Get the window DC
            let hdc_window = GetWindowDC(hwnd);
            if hdc_window == 0 {
                return Err("GetWindowDC failed".into());
            }

            // Create compatible DC and bitmap
            let hdc_mem = CreateCompatibleDC(hdc_window);
            if hdc_mem == 0 {
                ReleaseDC(hwnd, hdc_window);
                return Err("CreateCompatibleDC failed".into());
            }

            let h_bitmap = CreateCompatibleBitmap(hdc_window, width, height);
            if h_bitmap == 0 {
                DeleteDC(hdc_mem);
                ReleaseDC(hwnd, hdc_window);
                return Err("CreateCompatibleBitmap failed".into());
            }

            let prev_obj = SelectObject(hdc_mem, h_bitmap);

            // Try PrintWindow with PW_RENDERFULLCONTENT first (Win8+),
            // fall back to BitBlt
            let mut captured = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT) != 0;
            if !captured {
                captured = PrintWindow(hwnd, hdc_mem, 0) != 0;
            }
            if !captured {
                captured = BitBlt(hdc_mem, 0, 0, width, height, hdc_window, 0, 0, SRCCOPY) != 0;
            }

            if !captured {
                SelectObject(hdc_mem, prev_obj);
                DeleteObject(h_bitmap);
                DeleteDC(hdc_mem);
                ReleaseDC(hwnd, hdc_window);
                return Err("All capture methods failed (PrintWindow + BitBlt)".into());
            }

            // Read the bitmap data
            let buffer_size = (width * height * 4) as usize;
            let mut buffer = vec![0u8; buffer_size];

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0,
                    biSizeImage: buffer_size as u32,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };

            let lines = GetDIBits(
                hdc_mem,
                h_bitmap,
                0,
                height as u32,
                buffer.as_mut_ptr() as *mut c_void,
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Cleanup GDI resources
            SelectObject(hdc_mem, prev_obj);
            DeleteObject(h_bitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);

            if lines == 0 {
                return Err("GetDIBits failed".into());
            }

            // Convert BGRA to RGBA
            for chunk in buffer.chunks_exact_mut(4) {
                chunk.swap(0, 2); // swap B and R
            }

            // Crop to client area (removes title bar and borders)
            let mut client_rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            GetClientRect(hwnd, &mut client_rect);

            let border_left = (client_rect.left - window_rect.left).max(0) as u32;
            let border_top = (client_rect.top - window_rect.top).max(0) as u32;
            let client_width = (client_rect.right - client_rect.left).max(1) as u32;
            let client_height = (client_rect.bottom - client_rect.top).max(1) as u32;

            let full_image = RgbaImage::from_raw(width as u32, height as u32, buffer)
                .ok_or("Failed to create RgbaImage from buffer")?;

            // If client area equals window area (borderless), return as-is
            if border_left == 0
                && border_top == 0
                && client_width == width as u32
                && client_height == height as u32
            {
                Ok(full_image)
            } else {
                // GetClientRect returns client coords relative to client area (always 0,0).
                // We need the offset from window rect to client area.
                // On Windows, client rect from GetClientRect is in client coords.
                // The border offset is the difference between window and client rects
                // in screen coordinates. We need ClientToScreen for the origin.
                // But a simpler approach: the non-client area (borders + title bar)
                // is typically symmetric left/right, with a larger top for the title bar.
                // Use DwmGetWindowAttribute for accurate frame bounds if available.
                // For now, just return the full window capture (GPUI windows are borderless).
                Ok(full_image)
            }
        }
    }
}

/// Capture a screenshot of the app window.
///
/// On Windows, uses the stored HWND with Win32 PrintWindow (bypasses xcap's
/// own-process exclusion). On other platforms, uses xcap window enumeration.
///
/// Returns a tuple of (png_data, width, height) on success.
///
/// # Arguments
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
pub fn capture_app_screenshot(
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    // On Windows, try direct HWND capture first (xcap skips own-process windows).
    #[cfg(target_os = "windows")]
    {
        let hwnd = win32_get_main_hwnd();
        if hwnd != 0 {
            tracing::debug!(
                hwnd = format_args!("{:#x}", hwnd),
                "Capturing own window via stored HWND"
            );
            match win32_capture::capture_hwnd(hwnd) {
                Ok(image) => {
                    let (png_data, width, height) = encode_rgba_to_png(&image, hi_dpi)?;
                    tracing::debug!(
                        width = width,
                        height = height,
                        hi_dpi = hi_dpi,
                        file_size = png_data.len(),
                        "Screenshot captured via Win32 PrintWindow"
                    );
                    return Ok((png_data, width, height));
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Win32 direct capture failed, falling back to xcap");
                }
            }
        } else {
            tracing::warn!("No stored HWND available, falling back to xcap enumeration");
        }
    }

    // Fallback: xcap window enumeration (works on macOS/Linux, and as safety net on Windows)
    capture_app_screenshot_via_xcap(hi_dpi)
}

/// xcap-based window enumeration fallback.
fn capture_app_screenshot_via_xcap(
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let windows = Window::all()?;

    struct Candidate {
        window: Window,
        title: String,
        width: u32,
        height: u32,
    }

    let our_pid = std::process::id();
    let mut candidates = Vec::new();
    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());
        let window_pid = window.pid().unwrap_or(0);

        // On Windows, xcap app_name() returns the executable filename (e.g. "script-kit-gpui.exe").
        // On macOS it returns the bundle name (e.g. "Script Kit").
        // Match by executable name, bundle name, title, or process ID.
        let is_our_window = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || title.contains("Script Kit")
            || window_pid == our_pid;

        let is_minimized = window.is_minimized().unwrap_or(true);

        // Get window dimensions to filter out tiny windows (tooltips, list items, etc.)
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);

        // Only consider windows that are reasonably sized
        // Width >= 200 filters out small UI elements
        // Height >= 50 allows compact prompts (arg prompt without choices is ~76px)
        let is_reasonable_size = width >= 200 && height >= 50;

        tracing::trace!(
            app_name = %app_name,
            title = %title,
            window_pid = window_pid,
            our_pid = our_pid,
            is_our_window = is_our_window,
            is_minimized = is_minimized,
            width = width,
            height = height,
            "Screenshot: evaluating window"
        );

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

    let (png_data, width, height) = capture_and_encode_png(&window, hi_dpi)?;

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
    let windows = Window::all()?;

    let our_pid = std::process::id();
    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());
        let window_pid = window.pid().unwrap_or(0);

        // Match window by title pattern (must also be our app)
        let is_our_app = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || window_pid == our_pid;
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

            let (png_data, width, height) = capture_and_encode_png(&window, hi_dpi)?;

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
