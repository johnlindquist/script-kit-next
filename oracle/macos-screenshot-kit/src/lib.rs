//! Practical macOS screenshot capture for apps that need the same primitives as
//! screenshot tools, launchers, window managers, QA automation, visual agents, and overlays.
//!
//! The default API uses CoreGraphics plus ImageIO and can capture displays, windows,
//! regions, and pixels. For parity with macOS's built-in interactive screenshot UI and
//! cursor capture, the crate also exposes a backend that calls `/usr/sbin/screencapture`
//! without a shell. Consumers that need real-time capture, IOSurface, HDR, or system
//! picker workflows can enable the `screen-capture-kit` feature and use the re-exported
//! ScreenCaptureKit bindings.

mod error;
mod types;

#[cfg(target_os = "macos")]
mod platform;
#[cfg(not(target_os = "macos"))]
mod platform;

pub use error::{Result, ScreenshotError};
pub use types::*;

#[cfg(feature = "screen-capture-kit")]
pub mod screen_capture_kit {
    //! Re-export of the community `screencapturekit` crate for advanced consumers.
    //!
    //! Use this when you need streaming, IOSurface, HDR, ScreenCaptureKit's system picker,
    //! or the newest macOS capture controls. The high-level [`ScreenshotClient`] stays
    //! synchronous and focused on still screenshots.
    pub use screencapturekit::*;
}

/// High-level synchronous screenshot client.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScreenshotClient;

impl ScreenshotClient {
    pub fn new() -> Self {
        Self
    }

    /// Check Screen Recording permission using CoreGraphics.
    pub fn permission_status(&self) -> PermissionStatus {
        platform::permission_status()
    }

    /// Request Screen Recording permission using CoreGraphics.
    ///
    /// macOS may require the process to be restarted after the user changes permission.
    pub fn request_permission(&self) -> bool {
        platform::request_permission()
    }

    /// Return all active displays with global bounds and scale factor estimates.
    pub fn displays(&self) -> Result<Vec<DisplayInfo>> {
        platform::displays()
    }

    /// Return windows from the window server.
    pub fn windows(&self, options: WindowListOptions) -> Result<Vec<WindowInfo>> {
        platform::windows(options)
    }

    /// Return the first visible layer-0 window from the front-to-back window list.
    pub fn frontmost_window(&self) -> Result<WindowInfo> {
        platform::frontmost_window()
    }

    /// Return the first visible layer-0 window containing the supplied point.
    pub fn window_at_point(&self, point: Point) -> Result<WindowInfo> {
        platform::window_at_point(point)
    }

    /// Return all queried windows containing a point, preserving the window-server order.
    /// Use `WindowListOptions::visible_all_layers()` to include menus, popovers, and tooltips.
    pub fn windows_at_point(&self, point: Point, options: WindowListOptions) -> Result<Vec<WindowInfo>> {
        Ok(self
            .windows(options)?
            .into_iter()
            .filter(|window| window.bounds.contains(point))
            .collect())
    }

    /// Return the current global mouse location.
    pub fn mouse_location(&self) -> Result<Point> {
        platform::mouse_location()
    }

    /// Capture a target into an image object.
    pub fn capture(&self, target: CaptureTarget, options: CaptureOptions) -> Result<CapturedImage> {
        platform::capture(target, options)
    }

    /// Capture a target and encode it to bytes.
    pub fn capture_bytes(&self, target: CaptureTarget, options: CaptureOptions) -> Result<Vec<u8>> {
        let format = options.format;
        let image = self.capture(target, options)?;
        image.to_bytes(format)
    }

    /// Capture a target and write the requested format to disk.
    pub fn capture_to_file(
        &self,
        target: CaptureTarget,
        options: CaptureOptions,
        path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let format = options.format;
        let image = self.capture(target, options)?;
        image.save_as(path, format)
    }


    /// Capture directly to the macOS clipboard using the system screenshot backend.
    ///
    /// This is useful for apps that mirror Shift-Control-Command screenshot workflows.
    pub fn capture_to_clipboard(&self, target: CaptureTarget, options: CaptureOptions) -> Result<()> {
        platform::capture_to_clipboard(target, options)
    }

    /// Open System Settings near Screen Recording permission.
    pub fn open_screen_recording_settings(&self) -> Result<()> {
        platform::open_screen_recording_settings()
    }

    /// Capture each active display separately. This is the safest path for multi-monitor
    /// workflows that do not want one giant virtual-desktop image.
    pub fn capture_each_display(&self, options: CaptureOptions) -> Result<Vec<(DisplayInfo, CapturedImage)>> {
        let displays = self.displays()?;
        let mut out = Vec::with_capacity(displays.len());
        for display in displays {
            let image = self.capture(CaptureTarget::Display(display.id), options.clone())?;
            out.push((display, image));
        }
        Ok(out)
    }
}

/// Captured image with native backing on macOS.
pub struct CapturedImage {
    inner: platform::NativeImage,
    target: CaptureTarget,
    backend: CaptureBackend,
}

impl CapturedImage {
    pub(crate) fn new(inner: platform::NativeImage, target: CaptureTarget, backend: CaptureBackend) -> Self {
        Self { inner, target, backend }
    }

    /// Width in pixels.
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    /// Height in pixels.
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    /// Original target requested by the caller.
    pub fn target(&self) -> &CaptureTarget {
        &self.target
    }

    /// Backend that produced the image.
    pub fn backend(&self) -> CaptureBackend {
        self.backend
    }

    /// Encode to PNG/JPEG/TIFF/HEIC bytes using ImageIO.
    pub fn to_bytes(&self, format: ImageFormat) -> Result<Vec<u8>> {
        self.inner.to_bytes(format)
    }

    /// Save PNG/JPEG/TIFF/HEIC using ImageIO.
    pub fn save_as(&self, path: impl AsRef<std::path::Path>, format: ImageFormat) -> Result<()> {
        self.inner.save_as(path.as_ref(), format)
    }

    /// Save using the image format implied by the extension. Defaults to PNG if the extension is unknown.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let path = path.as_ref();
        let format = match path.extension().and_then(|ext| ext.to_str()).map(|s| s.to_ascii_lowercase()) {
            Some(ext) if ext == "jpg" || ext == "jpeg" => ImageFormat::Jpeg { quality: 0.92 },
            Some(ext) if ext == "tif" || ext == "tiff" => ImageFormat::Tiff,
            Some(ext) if ext == "heic" || ext == "heif" => ImageFormat::Heic { quality: 0.92 },
            Some(ext) if ext == "pdf" => ImageFormat::Pdf,
            Some(ext) if ext == "bmp" => ImageFormat::Bmp,
            _ => ImageFormat::Png,
        };
        self.save_as(path, format)
    }

    /// Convert to premultiplied RGBA8 pixels.
    pub fn to_rgba8(&self) -> Result<RgbaImage> {
        self.inner.to_rgba8()
    }
}

impl std::fmt::Debug for CapturedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedImage")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("target", &self.target)
            .field("backend", &self.backend)
            .finish()
    }
}

/// Convenience capture using default options.
pub fn capture(target: CaptureTarget) -> Result<CapturedImage> {
    ScreenshotClient::new().capture(target, CaptureOptions::default())
}

/// Convenience permission check.
pub fn has_screen_recording_permission() -> bool {
    ScreenshotClient::new().permission_status().is_granted()
}
