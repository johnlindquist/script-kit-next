mod cf;
mod ffi;
mod image;
mod quartz;
mod system_capture;

pub(crate) use image::NativeImage;

use crate::{
    CaptureBackend, CaptureOptions, CaptureTarget, CapturedImage, DisplayInfo, PermissionStatus,
    Point, Result, ScreenshotError, WindowInfo, WindowListOptions,
};

pub(crate) fn permission_status() -> PermissionStatus {
    if unsafe { ffi::CGPreflightScreenCaptureAccess() } {
        PermissionStatus::Granted
    } else {
        PermissionStatus::DeniedOrNotDetermined
    }
}

pub(crate) fn request_permission() -> bool {
    unsafe { ffi::CGRequestScreenCaptureAccess() }
}

pub(crate) fn displays() -> Result<Vec<DisplayInfo>> {
    quartz::displays()
}

pub(crate) fn windows(options: WindowListOptions) -> Result<Vec<WindowInfo>> {
    quartz::windows(options)
}

pub(crate) fn frontmost_window() -> Result<WindowInfo> {
    quartz::frontmost_window()
}

pub(crate) fn window_at_point(point: Point) -> Result<WindowInfo> {
    quartz::window_at_point(point)
}

pub(crate) fn mouse_location() -> Result<Point> {
    quartz::mouse_location()
}

pub(crate) fn capture(target: CaptureTarget, options: CaptureOptions) -> Result<CapturedImage> {
    if options.delay.as_millis() > 0
        && !matches!(
            selected_backend(&target, &options),
            CaptureBackend::SystemScreencapture
        )
    {
        std::thread::sleep(options.delay);
    }

    let backend = selected_backend(&target, &options);
    let (image, resolved_target) = match backend {
        CaptureBackend::CoreGraphics => quartz::capture_core_graphics(target, &options)?,
        CaptureBackend::SystemScreencapture => system_capture::capture_system(target, &options)?,
        CaptureBackend::ScreenCaptureKit => {
            return Err(ScreenshotError::UnsupportedBackend(
                "the high-level still-image API does not wrap ScreenCaptureKit; enable the screen-capture-kit feature and use the re-exported bindings directly",
            ));
        }
        CaptureBackend::Auto => unreachable!("selected_backend never returns Auto"),
    };

    Ok(CapturedImage::new(image, resolved_target, backend))
}

pub(crate) fn capture_to_clipboard(target: CaptureTarget, options: CaptureOptions) -> Result<()> {
    system_capture::capture_clipboard(target, &options)
}

pub(crate) fn open_screen_recording_settings() -> Result<()> {
    system_capture::open_screen_recording_settings()
}

fn selected_backend(target: &CaptureTarget, options: &CaptureOptions) -> CaptureBackend {
    match options.backend {
        CaptureBackend::Auto => {
            if options.include_cursor
                || matches!(
                    target,
                    CaptureTarget::Interactive
                        | CaptureTarget::InteractiveSelection
                        | CaptureTarget::InteractiveWindow
                        | CaptureTarget::InteractiveToolbar
                        | CaptureTarget::TouchBar
                        | CaptureTarget::DisplayOrdinal(_)
                )
            {
                CaptureBackend::SystemScreencapture
            } else {
                CaptureBackend::CoreGraphics
            }
        }
        explicit => explicit,
    }
}
