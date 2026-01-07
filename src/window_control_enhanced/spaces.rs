//! Space (virtual desktop) management backend

use std::sync::{Arc, RwLock};

/// Information about a macOS Space (virtual desktop)
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    /// Space identifier
    pub id: u64,
    /// Space index (1-based, as shown in Mission Control)
    pub index: u32,
    /// Whether this is the currently active space
    pub is_active: bool,
    /// Space type (regular desktop, fullscreen app, etc.)
    pub space_type: SpaceType,
}

/// Type of Space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceType {
    /// Regular desktop space
    Desktop,
    /// Fullscreen application space
    Fullscreen,
    /// Unknown type
    Unknown,
}

/// Error type for Space operations
#[derive(Debug, Clone)]
pub enum SpaceError {
    /// Space operations are not supported on this system/configuration
    NotSupported(String),
    /// The requested space was not found
    SpaceNotFound(u64),
    /// The window cannot be moved to a space
    WindowNotMovable(u32),
    /// External tool (e.g., yabai) is not available
    ExternalToolNotAvailable(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for SpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpaceError::NotSupported(msg) => write!(f, "Space operations not supported: {}", msg),
            SpaceError::SpaceNotFound(id) => write!(f, "Space not found: {}", id),
            SpaceError::WindowNotMovable(id) => {
                write!(f, "Window {} cannot be moved to a space", id)
            }
            SpaceError::ExternalToolNotAvailable(tool) => {
                write!(f, "External tool not available: {}", tool)
            }
            SpaceError::Other(msg) => write!(f, "Space error: {}", msg),
        }
    }
}

impl std::error::Error for SpaceError {}

/// Backend trait for Space management operations
///
/// Default implementation returns "unsupported" for all operations.
/// Alternative implementations can integrate with external tools (yabai)
/// or private APIs (experimental, not recommended).
pub trait SpaceManager: Send + Sync {
    /// Get all spaces on all displays
    fn get_all_spaces(&self) -> Result<Vec<SpaceInfo>, SpaceError>;

    /// Get the currently active space
    fn get_active_space(&self) -> Result<SpaceInfo, SpaceError>;

    /// Move a window to a specific space
    fn move_window_to_space(&self, window_id: u32, space_id: u64) -> Result<(), SpaceError>;

    /// Check if space operations are supported
    fn is_supported(&self) -> bool;

    /// Get a description of why spaces aren't supported (if not supported)
    fn unsupported_reason(&self) -> Option<String>;
}

/// Default "unsupported" Space backend
///
/// This is the default backend that returns clear "not supported" errors
/// for all space operations. This is the safe default since moving windows
/// between Spaces requires either:
/// - Private WindowServer/Dock APIs (fragile, requires SIP disable)
/// - External tools like yabai (requires user setup)
pub struct UnsupportedSpaceBackend;

impl SpaceManager for UnsupportedSpaceBackend {
    fn get_all_spaces(&self) -> Result<Vec<SpaceInfo>, SpaceError> {
        Err(SpaceError::NotSupported(
            "Space enumeration requires private macOS APIs or external tools like yabai. \
             This feature is not enabled by default."
                .to_string(),
        ))
    }

    fn get_active_space(&self) -> Result<SpaceInfo, SpaceError> {
        Err(SpaceError::NotSupported(
            "Getting active space requires private macOS APIs. \
             This feature is not enabled by default."
                .to_string(),
        ))
    }

    fn move_window_to_space(&self, _window_id: u32, _space_id: u64) -> Result<(), SpaceError> {
        Err(SpaceError::NotSupported(
            "Moving windows between Spaces requires private macOS APIs or external tools. \
             Consider using yabai (https://github.com/koekeishiya/yabai) if you need this feature."
                .to_string(),
        ))
    }

    fn is_supported(&self) -> bool {
        false
    }

    fn unsupported_reason(&self) -> Option<String> {
        Some(
            "Space management requires private macOS APIs or external tools like yabai. \
             This feature is not enabled by default for stability and compatibility."
                .to_string(),
        )
    }
}

// ============================================================================
// Global Space Manager
// ============================================================================

/// Global space manager instance
static SPACE_MANAGER: RwLock<Option<Arc<dyn SpaceManager>>> = RwLock::new(None);

/// Get the current space manager, or create the default unsupported backend
pub fn get_space_manager() -> Arc<dyn SpaceManager> {
    let read_guard = SPACE_MANAGER.read().unwrap();
    if let Some(ref manager) = *read_guard {
        return Arc::clone(manager);
    }
    drop(read_guard);

    // Create default backend
    let mut write_guard = SPACE_MANAGER.write().unwrap();
    if write_guard.is_none() {
        *write_guard = Some(Arc::new(UnsupportedSpaceBackend));
    }
    Arc::clone(write_guard.as_ref().unwrap())
}

/// Set a custom space manager backend
pub fn set_space_manager(manager: Arc<dyn SpaceManager>) {
    let mut write_guard = SPACE_MANAGER.write().unwrap();
    *write_guard = Some(manager);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_backend() {
        let backend = UnsupportedSpaceBackend;

        assert!(!backend.is_supported());
        assert!(backend.unsupported_reason().is_some());

        let result = backend.get_all_spaces();
        assert!(matches!(result, Err(SpaceError::NotSupported(_))));

        let result = backend.get_active_space();
        assert!(matches!(result, Err(SpaceError::NotSupported(_))));

        let result = backend.move_window_to_space(123, 1);
        assert!(matches!(result, Err(SpaceError::NotSupported(_))));
    }

    #[test]
    fn test_space_error_display() {
        let err = SpaceError::NotSupported("test reason".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("not supported"));
        assert!(msg.contains("test reason"));

        let err = SpaceError::SpaceNotFound(42);
        let msg = format!("{}", err);
        assert!(msg.contains("42"));

        let err = SpaceError::WindowNotMovable(123);
        let msg = format!("{}", err);
        assert!(msg.contains("123"));
    }

    #[test]
    fn test_global_space_manager() {
        let manager = get_space_manager();
        assert!(!manager.is_supported());

        let result = manager.get_all_spaces();
        assert!(result.is_err());
    }
}
