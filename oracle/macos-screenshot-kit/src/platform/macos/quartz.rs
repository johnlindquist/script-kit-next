use super::cf::{cf_number_i64, dict_bool, dict_f64, dict_i64, dict_rect, dict_string, OwnedCf};
use super::ffi::*;
use super::image::NativeImage;
use crate::{
    CaptureOptions, CaptureTarget, DisplayInfo, Point, Rect, Result, ScreenshotError, WindowFrameMode,
    WindowId, WindowInfo, WindowListOptions,
};
use std::os::raw::c_void;
use std::ptr;

pub(super) fn displays() -> Result<Vec<DisplayInfo>> {
    let mut count = 0_u32;
    let err = unsafe { CGGetActiveDisplayList(0, ptr::null_mut(), &mut count) };
    if err != kCGErrorSuccess {
        return Err(ScreenshotError::CoreGraphics(format!("CGGetActiveDisplayList(count) failed: {err}")));
    }
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut ids = vec![0_u32; count as usize];
    let err = unsafe { CGGetActiveDisplayList(count, ids.as_mut_ptr(), &mut count) };
    if err != kCGErrorSuccess {
        return Err(ScreenshotError::CoreGraphics(format!("CGGetActiveDisplayList failed: {err}")));
    }
    ids.truncate(count as usize);

    let main = unsafe { CGMainDisplayID() };
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        let bounds = unsafe { CGDisplayBounds(id) };
        let pixel_width = unsafe { CGDisplayPixelsWide(id) };
        let pixel_height = unsafe { CGDisplayPixelsHigh(id) };
        let scale_x = if bounds.size.width > 0.0 { pixel_width as f64 / bounds.size.width } else { 1.0 };
        let scale_y = if bounds.size.height > 0.0 { pixel_height as f64 / bounds.size.height } else { scale_x };
        out.push(DisplayInfo {
            id,
            bounds: Rect::new(bounds.origin.x, bounds.origin.y, bounds.size.width, bounds.size.height),
            pixel_width,
            pixel_height,
            scale_factor: ((scale_x + scale_y) / 2.0).max(1.0),
            is_main: id == main,
            is_builtin: unsafe { CGDisplayIsBuiltin(id) } != 0,
        });
    }
    Ok(out)
}

pub(super) fn windows(options: WindowListOptions) -> Result<Vec<WindowInfo>> {
    let mut list_option = if options.onscreen_only {
        kCGWindowListOptionOnScreenOnly
    } else {
        kCGWindowListOptionAll
    };
    if options.exclude_desktop_elements {
        list_option |= kCGWindowListExcludeDesktopElements;
    }

    let array = unsafe { CGWindowListCopyWindowInfo(list_option, 0) };
    let array = OwnedCf::new_const(array)
        .ok_or_else(|| ScreenshotError::CoreGraphics("CGWindowListCopyWindowInfo returned null".into()))?;

    let count = unsafe { CFArrayGetCount(array.as_ptr() as CFArrayRef) };
    let mut out = Vec::new();
    for i in 0..count {
        let dict = unsafe { CFArrayGetValueAtIndex(array.as_ptr() as CFArrayRef, i) } as CFDictionaryRef;
        if dict.is_null() {
            continue;
        }
        let Some(id) = dict_i64(dict, "kCGWindowNumber").and_then(|v| u32::try_from(v).ok()) else {
            continue;
        };
        let owner_pid = dict_i64(dict, "kCGWindowOwnerPID").unwrap_or_default() as i32;
        let owner_name = dict_string(dict, "kCGWindowOwnerName").filter(|s| !s.is_empty());
        let title = dict_string(dict, "kCGWindowName").filter(|s| !s.is_empty());
        if !options.include_untitled && title.is_none() {
            continue;
        }
        let bounds = dict_rect(dict, "kCGWindowBounds").unwrap_or_else(|| Rect::new(0.0, 0.0, 0.0, 0.0));
        let layer = dict_i64(dict, "kCGWindowLayer").unwrap_or_default();
        let alpha = dict_f64(dict, "kCGWindowAlpha").unwrap_or(1.0);
        if let Some(min_alpha) = options.min_alpha {
            if alpha < min_alpha {
                continue;
            }
        }
        if let Some(layers) = &options.allowed_layers {
            if !layers.contains(&layer) {
                continue;
            }
        }
        let is_onscreen = dict_bool(dict, "kCGWindowIsOnscreen").unwrap_or(!options.onscreen_only);
        if options.onscreen_only && !is_onscreen {
            continue;
        }
        let window = WindowInfo {
            id,
            owner_pid,
            owner_name,
            title,
            bounds,
            layer,
            alpha,
            is_onscreen,
            sharing_state: dict_i64(dict, "kCGWindowSharingState"),
            memory_usage: dict_i64(dict, "kCGWindowMemoryUsage"),
        };
        out.push(window);
    }
    Ok(out)
}

pub(super) fn frontmost_window() -> Result<WindowInfo> {
    windows(WindowListOptions::default())?
        .into_iter()
        .find(WindowInfo::is_probably_user_window)
        .ok_or_else(|| ScreenshotError::NotFound("no visible layer-0 window found".into()))
}

pub(super) fn window_at_point(point: Point) -> Result<WindowInfo> {
    windows(WindowListOptions::default())?
        .into_iter()
        .find(|w| w.is_probably_user_window() && w.bounds.contains(point))
        .ok_or_else(|| ScreenshotError::NotFound(format!("no visible layer-0 window at {},{}", point.x, point.y)))
}

pub(super) fn mouse_location() -> Result<Point> {
    let event = unsafe { CGEventCreate(ptr::null()) };
    let event = OwnedCf::new_mut(event).ok_or_else(|| ScreenshotError::CoreGraphics("CGEventCreate failed".into()))?;
    let p = unsafe { CGEventGetLocation(event.as_mut_ptr() as CGEventRef) };
    Ok(Point::new(p.x, p.y))
}

pub(super) fn capture_core_graphics(target: CaptureTarget, options: &CaptureOptions) -> Result<(NativeImage, CaptureTarget)> {
    if options.include_cursor {
        return Err(ScreenshotError::UnsupportedBackend("CoreGraphics still screenshots do not include the cursor; use CaptureBackend::Auto or SystemScreencapture"));
    }
    let image_options = cg_image_options(options);
    let target = resolve_target(target)?;
    let image = match &target {
        CaptureTarget::AllDisplays => unsafe {
            CGWindowListCreateImage(
                CGRectInfinite,
                window_list_options_for_capture(options),
                0,
                image_options,
            )
        },
        CaptureTarget::MainDisplay => unsafe { CGDisplayCreateImage(CGMainDisplayID()) },
        CaptureTarget::Display(display_id) => unsafe { CGDisplayCreateImage(*display_id) },
        CaptureTarget::DisplayOrdinal(_) => {
            return Err(ScreenshotError::UnsupportedBackend("display ordinals are a /usr/sbin/screencapture concept; use CaptureTarget::Display(display_id) for CoreGraphics"));
        }
        CaptureTarget::Region(rect) => {
            validate_rect(*rect)?;
            unsafe {
                CGWindowListCreateImage(
                    rect_to_cg(*rect),
                    window_list_options_for_capture(options),
                    0,
                    image_options,
                )
            }
        }
        CaptureTarget::DisplayRegion { display_id, rect } => {
            validate_rect(*rect)?;
            unsafe { CGDisplayCreateImageForRect(*display_id, rect_to_cg(*rect)) }
        }
        CaptureTarget::Window(window_id) => unsafe {
            CGWindowListCreateImage(
                CGRectNull,
                kCGWindowListOptionIncludingWindow,
                *window_id,
                image_options,
            )
        },
        CaptureTarget::Windows(window_ids) => capture_windows_array(window_ids, image_options)?,
        CaptureTarget::Application(pid) => {
            let ids: Vec<WindowId> = windows(WindowListOptions::default())?
                .into_iter()
                .filter(|w| w.owner_pid == *pid && w.is_probably_user_window())
                .map(|w| w.id)
                .collect();
            if ids.is_empty() {
                return Err(ScreenshotError::NotFound(format!("no visible windows for pid {pid}")));
            }
            capture_windows_array(&ids, image_options)?
        }
        CaptureTarget::VisibleWindows { exclude_window_ids, exclude_pids, include_all_layers } => {
            let list_options = WindowListOptions {
                allowed_layers: if *include_all_layers { None } else { Some(vec![0]) },
                exclude_desktop_elements: !options.include_desktop_elements,
                ..WindowListOptions::default()
            };
            let ids: Vec<WindowId> = windows(list_options)?
                .into_iter()
                .filter(|w| w.is_onscreen && w.alpha > 0.0 && !w.bounds.is_empty())
                .filter(|w| !exclude_window_ids.contains(&w.id))
                .filter(|w| !exclude_pids.contains(&w.owner_pid))
                .map(|w| w.id)
                .collect();
            if ids.is_empty() {
                return Err(ScreenshotError::NotFound("no windows left after applying exclusions".into()));
            }
            capture_windows_array(&ids, image_options)?
        }
        CaptureTarget::Interactive
        | CaptureTarget::InteractiveSelection
        | CaptureTarget::InteractiveWindow
        | CaptureTarget::InteractiveToolbar
        | CaptureTarget::TouchBar => {
            return Err(ScreenshotError::UnsupportedBackend("interactive, toolbar, and Touch Bar captures require the system screencapture backend"));
        }
        CaptureTarget::FrontmostWindow
        | CaptureTarget::WindowUnderCursor
        | CaptureTarget::WindowAtPoint(_) => unreachable!("resolve_target converts dynamic window targets"),
    };

    Ok((NativeImage::from_create_rule(image)?, target))
}

fn resolve_target(target: CaptureTarget) -> Result<CaptureTarget> {
    match target {
        CaptureTarget::FrontmostWindow => Ok(CaptureTarget::Window(frontmost_window()?.id)),
        CaptureTarget::WindowUnderCursor => Ok(CaptureTarget::Window(window_at_point(mouse_location()?)?.id)),
        CaptureTarget::WindowAtPoint(point) => Ok(CaptureTarget::Window(window_at_point(point)?.id)),
        other => Ok(other),
    }
}

fn capture_windows_array(window_ids: &[WindowId], image_options: CGWindowImageOption) -> Result<CGImageRef> {
    if window_ids.is_empty() {
        return Err(ScreenshotError::InvalidInput("window array capture requires at least one window ID".into()));
    }

    let mut numbers = Vec::with_capacity(window_ids.len());
    let mut values: Vec<*const c_void> = Vec::with_capacity(window_ids.len());
    for id in window_ids {
        let number = cf_number_i64(*id as i64)?;
        values.push(number.as_ptr());
        numbers.push(number);
    }

    let array = unsafe {
        CFArrayCreate(
            null_allocator(),
            values.as_ptr(),
            values.len() as CFIndex,
            ptr::null(),
        )
    };
    let array = OwnedCf::new_const(array)
        .ok_or_else(|| ScreenshotError::CoreGraphics("CFArrayCreate for window IDs failed".into()))?;

    let image = unsafe { CGWindowListCreateImageFromArray(CGRectNull, array.as_ptr() as CFArrayRef, image_options) };
    Ok(image)
}

fn cg_image_options(options: &CaptureOptions) -> CGWindowImageOption {
    let mut out = kCGWindowImageDefault;
    if options.best_resolution {
        out |= kCGWindowImageBestResolution;
    } else {
        out |= kCGWindowImageNominalResolution;
    }
    if options.opaque {
        out |= kCGWindowImageShouldBeOpaque;
    }
    match options.window_frame {
        WindowFrameMode::Default => {}
        WindowFrameMode::WithoutShadow => out |= kCGWindowImageBoundsIgnoreFraming,
        WindowFrameMode::ShadowOnly => out |= kCGWindowImageOnlyShadows,
    }
    out
}

fn window_list_options_for_capture(options: &CaptureOptions) -> CGWindowListOption {
    let mut out = kCGWindowListOptionOnScreenOnly;
    if !options.include_desktop_elements {
        out |= kCGWindowListExcludeDesktopElements;
    }
    out
}

fn rect_to_cg(rect: Rect) -> CGRect {
    cg_rect(rect.x, rect.y, rect.width, rect.height)
}

fn validate_rect(rect: Rect) -> Result<()> {
    if !rect.x.is_finite() || !rect.y.is_finite() || !rect.width.is_finite() || !rect.height.is_finite() || rect.is_empty() {
        Err(ScreenshotError::InvalidInput(format!("invalid capture rectangle: {rect:?}")))
    } else {
        Ok(())
    }
}
