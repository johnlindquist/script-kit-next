use std::fmt;

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, ScreenshotError>;

/// Errors returned by screenshot operations.
#[derive(Debug)]
pub enum ScreenshotError {
    /// The current platform is not supported by this crate.
    UnsupportedPlatform,
    /// The requested capture backend is not available for this target/options pair.
    UnsupportedBackend(&'static str),
    /// macOS Screen Recording permission is not granted, or the system refused capture.
    PermissionDenied,
    /// No display/window/target matched the request.
    NotFound(String),
    /// A CoreGraphics or CoreFoundation call failed.
    CoreGraphics(String),
    /// Image decoding, encoding, or pixel conversion failed.
    Image(String),
    /// The system `/usr/sbin/screencapture` tool failed.
    SystemCapture(String),
    /// An I/O operation failed.
    Io(std::io::Error),
    /// The caller passed invalid options or an impossible rectangle.
    InvalidInput(String),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "screen capture is only implemented on macOS"),
            Self::UnsupportedBackend(msg) => write!(f, "unsupported capture backend: {msg}"),
            Self::PermissionDenied => write!(f, "screen recording permission is not granted"),
            Self::NotFound(msg) => write!(f, "capture target was not found: {msg}"),
            Self::CoreGraphics(msg) => write!(f, "CoreGraphics error: {msg}"),
            Self::Image(msg) => write!(f, "image error: {msg}"),
            Self::SystemCapture(msg) => write!(f, "screencapture failed: {msg}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::InvalidInput(msg) => write!(f, "invalid screenshot request: {msg}"),
        }
    }
}

impl std::error::Error for ScreenshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ScreenshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
