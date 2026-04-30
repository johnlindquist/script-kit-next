use crate::{
    CaptureBackend, CaptureOptions, CaptureTarget, CapturedImage, DisplayInfo, PermissionStatus,
    Point, Result, RgbaImage, ScreenshotError, WindowInfo, WindowListOptions,
};
use std::path::Path;

#[derive(Debug)]
pub(crate) struct NativeImage;

impl NativeImage {
    pub fn width(&self) -> usize { 0 }
    pub fn height(&self) -> usize { 0 }
    pub fn to_bytes(&self, _format: crate::ImageFormat) -> Result<Vec<u8>> { Err(ScreenshotError::UnsupportedPlatform) }
    pub fn save_as(&self, _path: &Path, _format: crate::ImageFormat) -> Result<()> { Err(ScreenshotError::UnsupportedPlatform) }
    pub fn to_rgba8(&self) -> Result<RgbaImage> { Err(ScreenshotError::UnsupportedPlatform) }
}

pub(crate) fn permission_status() -> PermissionStatus { PermissionStatus::DeniedOrNotDetermined }
pub(crate) fn request_permission() -> bool { false }
pub(crate) fn displays() -> Result<Vec<DisplayInfo>> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn windows(_options: WindowListOptions) -> Result<Vec<WindowInfo>> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn frontmost_window() -> Result<WindowInfo> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn window_at_point(_point: Point) -> Result<WindowInfo> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn mouse_location() -> Result<Point> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn capture(_target: CaptureTarget, _options: CaptureOptions) -> Result<CapturedImage> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn capture_to_clipboard(_target: CaptureTarget, _options: CaptureOptions) -> Result<()> { Err(ScreenshotError::UnsupportedPlatform) }
pub(crate) fn open_screen_recording_settings() -> Result<()> { Err(ScreenshotError::UnsupportedPlatform) }

#[allow(dead_code)]
fn _keep_imports(_: CaptureBackend) {}
