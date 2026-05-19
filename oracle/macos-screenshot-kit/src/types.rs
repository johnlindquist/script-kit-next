use std::path::{Path, PathBuf};
use std::time::Duration;

pub type DisplayId = u32;
pub type WindowId = u32;
pub type ProcessId = i32;

/// A point in global macOS screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A size in points or pixels, depending on context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

/// A rectangle in global macOS screen coordinates unless documented otherwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn origin(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn union(self, other: Rect) -> Rect {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn inset(self, amount: f64) -> Rect {
        Rect::new(
            self.x + amount,
            self.y + amount,
            (self.width - amount * 2.0).max(0.0),
            (self.height - amount * 2.0).max(0.0),
        )
    }
}

/// Screen Recording permission state as exposed by CoreGraphics.
///
/// macOS intentionally does not distinguish "not requested" from "denied" for
/// this API surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    Granted,
    DeniedOrNotDetermined,
}

impl PermissionStatus {
    pub fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Capture implementation preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBackend {
    /// Pick the safest backend for the target and options.
    Auto,
    /// Native CoreGraphics window/display capture. Fast and returns CGImage-backed pixels.
    CoreGraphics,
    /// Use `/usr/sbin/screencapture`. Good for interactive capture, cursor capture, and parity
    /// with macOS's built-in screenshot UI.
    SystemScreencapture,
    /// Reserved for consumers that enable the `screen-capture-kit` feature and call the re-exported
    /// ScreenCaptureKit API directly for streaming/HDR/latest OS functionality.
    ScreenCaptureKit,
}

/// File/byte output format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg { quality: f32 },
    Tiff,
    Heic { quality: f32 },
    Pdf,
    Bmp,
}

impl ImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg { .. } => "jpg",
            Self::Tiff => "tiff",
            Self::Heic { .. } => "heic",
            Self::Pdf => "pdf",
            Self::Bmp => "bmp",
        }
    }

    pub fn uti(self) -> &'static str {
        match self {
            Self::Png => "public.png",
            Self::Jpeg { .. } => "public.jpeg",
            Self::Tiff => "public.tiff",
            Self::Heic { .. } => "public.heic",
            Self::Pdf => "com.adobe.pdf",
            Self::Bmp => "com.microsoft.bmp",
        }
    }

    pub fn quality(self) -> Option<f32> {
        match self {
            Self::Jpeg { quality } | Self::Heic { quality } => Some(quality.clamp(0.0, 1.0)),
            _ => None,
        }
    }
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::Png
    }
}

/// How to handle the non-content framing around a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowFrameMode {
    /// Preserve the platform default, generally including shadow/framing for window captures.
    Default,
    /// Request a window image without the surrounding frame/shadow where the backend supports it.
    WithoutShadow,
    /// Capture only the window shadow.
    ShadowOnly,
}

/// A screenshot target.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureTarget {
    /// Composite image of all visible displays in global desktop coordinates.
    AllDisplays,
    /// Main display only.
    MainDisplay,
    /// A display by macOS screencapture ordinal. The system tool treats 1 as main, 2 as secondary, etc.
    DisplayOrdinal(u32),
    /// A specific display.
    Display(DisplayId),
    /// A global desktop rectangle, possibly spanning displays.
    Region(Rect),
    /// A rectangle relative to a display's coordinate space.
    DisplayRegion { display_id: DisplayId, rect: Rect },
    /// A specific CoreGraphics window ID.
    Window(WindowId),
    /// Multiple CoreGraphics window IDs composited together.
    Windows(Vec<WindowId>),
    /// First eligible frontmost/topmost window according to the window server list.
    FrontmostWindow,
    /// The first eligible window containing the current mouse pointer.
    WindowUnderCursor,
    /// The first eligible window containing a point.
    WindowAtPoint(Point),
    /// Composite all visible windows owned by a process.
    Application(ProcessId),
    /// Composite currently visible windows while excluding known windows or processes.
    /// Useful for screenshot overlays that need to hide their own UI.
    VisibleWindows {
        exclude_window_ids: Vec<WindowId>,
        exclude_pids: Vec<ProcessId>,
        include_all_layers: bool,
    },
    /// Let the user choose selection/window mode using macOS's interactive screenshot UI.
    Interactive,
    /// Let the user drag a rectangle using macOS's interactive screenshot UI.
    InteractiveSelection,
    /// Let the user pick a window using macOS's interactive screenshot UI.
    InteractiveWindow,
    /// Show macOS's screenshot toolbar UI, similar to Shift-Command-5.
    InteractiveToolbar,
    /// Capture the Touch Bar on supported Macs. Uses the system screenshot backend.
    TouchBar,
}

/// Options for a screenshot request.
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    pub backend: CaptureBackend,
    pub format: ImageFormat,
    pub include_cursor: bool,
    pub include_desktop_elements: bool,
    pub best_resolution: bool,
    pub opaque: bool,
    pub window_frame: WindowFrameMode,
    pub delay: Duration,
    /// When the system `screencapture` backend is selected, this controls whether macOS plays the
    /// screenshot sound. CoreGraphics captures are always silent.
    pub play_sound: bool,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            backend: CaptureBackend::Auto,
            format: ImageFormat::Png,
            include_cursor: false,
            include_desktop_elements: true,
            best_resolution: true,
            opaque: false,
            window_frame: WindowFrameMode::Default,
            delay: Duration::ZERO,
            play_sound: false,
        }
    }
}

impl CaptureOptions {
    pub fn png() -> Self {
        Self::default()
    }

    pub fn jpeg(quality: f32) -> Self {
        Self {
            format: ImageFormat::Jpeg { quality },
            ..Self::default()
        }
    }

    pub fn with_backend(mut self, backend: CaptureBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_cursor(mut self, include_cursor: bool) -> Self {
        self.include_cursor = include_cursor;
        self
    }

    pub fn without_window_shadow(mut self) -> Self {
        self.window_frame = WindowFrameMode::WithoutShadow;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

/// Options for querying the window server.
#[derive(Debug, Clone)]
pub struct WindowListOptions {
    pub onscreen_only: bool,
    pub exclude_desktop_elements: bool,
    pub min_alpha: Option<f64>,
    pub allowed_layers: Option<Vec<i64>>,
    pub include_untitled: bool,
}

impl Default for WindowListOptions {
    fn default() -> Self {
        Self {
            onscreen_only: true,
            exclude_desktop_elements: true,
            min_alpha: Some(0.01),
            allowed_layers: Some(vec![0]),
            include_untitled: true,
        }
    }
}

impl WindowListOptions {
    /// Query visible windows at every layer. Useful for menus, popovers, tooltips, and overlays.
    pub fn visible_all_layers() -> Self {
        Self {
            allowed_layers: None,
            ..Self::default()
        }
    }

    /// Query the broadest window list CoreGraphics exposes.
    pub fn all_windows() -> Self {
        Self {
            onscreen_only: false,
            exclude_desktop_elements: false,
            min_alpha: None,
            allowed_layers: None,
            include_untitled: true,
        }
    }

    pub fn include_all_layers(mut self) -> Self {
        self.allowed_layers = None;
        self
    }

    pub fn include_offscreen(mut self) -> Self {
        self.onscreen_only = false;
        self
    }

    pub fn include_desktop_elements(mut self) -> Self {
        self.exclude_desktop_elements = false;
        self
    }

    pub fn titled_only(mut self) -> Self {
        self.include_untitled = false;
        self
    }
}

/// Information about a display.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    pub id: DisplayId,
    pub bounds: Rect,
    pub pixel_width: usize,
    pub pixel_height: usize,
    pub scale_factor: f64,
    pub is_main: bool,
    pub is_builtin: bool,
}

/// Information about a CoreGraphics window.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub id: WindowId,
    pub owner_pid: ProcessId,
    pub owner_name: Option<String>,
    pub title: Option<String>,
    pub bounds: Rect,
    pub layer: i64,
    pub alpha: f64,
    pub is_onscreen: bool,
    pub sharing_state: Option<i64>,
    pub memory_usage: Option<i64>,
}

impl WindowInfo {
    pub fn display_title(&self) -> String {
        match (&self.owner_name, &self.title) {
            (Some(owner), Some(title)) if !title.is_empty() => format!("{owner} — {title}"),
            (Some(owner), _) => owner.clone(),
            (_, Some(title)) if !title.is_empty() => title.clone(),
            _ => format!("Window {}", self.id),
        }
    }

    pub fn is_probably_user_window(&self) -> bool {
        self.is_onscreen && self.layer == 0 && self.alpha > 0.0 && !self.bounds.is_empty()
    }
}

/// Raw 8-bit RGBA pixels.
#[derive(Debug, Clone)]
pub struct RgbaImage {
    pub width: usize,
    pub height: usize,
    pub bytes_per_row: usize,
    /// Premultiplied RGBA8 bytes. The buffer length is `bytes_per_row * height`.
    pub data: Vec<u8>,
}

/// Where to write a screenshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenshotDestination {
    Bytes,
    File(PathBuf),
}

impl ScreenshotDestination {
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self::File(path.as_ref().to_path_buf())
    }
}
