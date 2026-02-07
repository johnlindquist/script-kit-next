//! Window capability detection via AXUIElementIsAttributeSettable

#![allow(non_upper_case_globals)]

use super::bounds::{SizeConstraints, WindowBounds};
use std::ffi::c_void;

/// Capabilities of a window, determined by querying Accessibility APIs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowCapabilities {
    /// Whether the window position can be changed (AXPosition is settable)
    pub can_move: bool,
    /// Whether the window size can be changed (AXSize is settable)
    pub can_resize: bool,
    /// Whether the window can be minimized
    pub can_minimize: bool,
    /// Whether the window can be closed
    pub can_close: bool,
    /// Whether the window can enter fullscreen mode
    pub can_fullscreen: bool,
    /// Whether moving to different Spaces is supported (requires external backend)
    pub supports_space_move: bool,
}

impl Default for WindowCapabilities {
    fn default() -> Self {
        Self {
            can_move: true,
            can_resize: true,
            can_minimize: true,
            can_close: true,
            can_fullscreen: true,
            supports_space_move: false, // Always false by default
        }
    }
}

/// Enhanced window information with capabilities
#[derive(Debug, Clone)]
pub struct EnhancedWindowInfo {
    /// Unique window identifier
    pub id: u32,
    /// Application name
    pub app: String,
    /// Bundle identifier of the owning application
    pub bundle_id: Option<String>,
    /// Window title
    pub title: String,
    /// Process ID of the owning application
    pub pid: i32,
    /// Window bounds in AX coordinates
    pub bounds: WindowBounds,
    /// Window capabilities
    pub capabilities: WindowCapabilities,
    /// Size constraints (if available)
    pub size_constraints: SizeConstraints,
    /// Internal AX element reference (stored as usize for Send/Sync)
    #[allow(dead_code)]
    ax_element: Option<usize>,
}

impl EnhancedWindowInfo {
    /// Create new enhanced window info
    pub fn new(
        id: u32,
        app: String,
        title: String,
        pid: i32,
        bounds: WindowBounds,
        capabilities: WindowCapabilities,
    ) -> Self {
        Self {
            id,
            app,
            bundle_id: None,
            title,
            pid,
            bounds,
            capabilities,
            size_constraints: SizeConstraints::default(),
            ax_element: None,
        }
    }

    /// Check if this window can be resized to the given dimensions
    pub fn can_resize_to(&self, width: f64, height: f64) -> bool {
        if !self.capabilities.can_resize {
            return false;
        }

        let (clamped_w, clamped_h) = self.size_constraints.clamp_size(width, height);
        (clamped_w - width).abs() < 1.0 && (clamped_h - height).abs() < 1.0
    }
}

// ============================================================================
// FFI Bindings for AXUIElementIsAttributeSettable
// ============================================================================

#[cfg(target_os = "macos")]
mod ax_ffi {
    use std::ffi::c_void;

    pub type AXUIElementRef = *const c_void;
    pub type CFStringRef = *const c_void;
    pub type CFTypeRef = *const c_void;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub fn AXUIElementIsAttributeSettable(
            element: AXUIElementRef,
            attribute: CFStringRef,
            settable: *mut bool,
        ) -> i32;

        pub fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const i8,
            encoding: u32,
        ) -> CFStringRef;
        pub fn CFRelease(cf: *const c_void);
    }

    pub const kCFStringEncodingUTF8: u32 = 0x08000100;
    pub const kAXErrorSuccess: i32 = 0;

    /// Create a CFString from a Rust string.
    pub fn try_create_cf_string(s: &str) -> Option<CFStringRef> {
        let c_str = std::ffi::CString::new(s).ok()?;
        let cf_string = unsafe {
            CFStringCreateWithCString(std::ptr::null(), c_str.as_ptr(), kCFStringEncodingUTF8)
        };
        (!cf_string.is_null()).then_some(cf_string)
    }

    /// Release a CoreFoundation object
    pub fn cf_release(cf: CFTypeRef) {
        if !cf.is_null() {
            unsafe {
                CFRelease(cf);
            }
        }
    }

    /// Check if an AX attribute is settable
    pub fn is_attribute_settable(element: AXUIElementRef, attribute: &str) -> bool {
        if element.is_null() {
            return false;
        }

        let Some(attr_str) = try_create_cf_string(attribute) else {
            return false;
        };
        let mut settable = false;

        let result = unsafe {
            AXUIElementIsAttributeSettable(element, attr_str, &mut settable as *mut bool)
        };

        cf_release(attr_str);

        result == kAXErrorSuccess && settable
    }

    /// Check if an AX attribute exists (has a value)
    pub fn has_attribute(element: AXUIElementRef, attribute: &str) -> bool {
        if element.is_null() {
            return false;
        }

        let Some(attr_str) = try_create_cf_string(attribute) else {
            return false;
        };
        let mut value: CFTypeRef = std::ptr::null();

        let result = unsafe {
            AXUIElementCopyAttributeValue(element, attr_str, &mut value as *mut CFTypeRef)
        };

        cf_release(attr_str);

        if result == kAXErrorSuccess && !value.is_null() {
            cf_release(value);
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Public API - Capability Detection (macOS)
// ============================================================================

/// Check if a window's position can be changed
#[cfg(target_os = "macos")]
pub fn can_move_window(ax_element: *const c_void) -> bool {
    ax_ffi::is_attribute_settable(ax_element, "AXPosition")
}

/// Check if a window's size can be changed
#[cfg(target_os = "macos")]
pub fn can_resize_window(ax_element: *const c_void) -> bool {
    ax_ffi::is_attribute_settable(ax_element, "AXSize")
}

/// Check if a window has a minimize button
#[cfg(target_os = "macos")]
pub fn can_minimize_window(ax_element: *const c_void) -> bool {
    ax_ffi::has_attribute(ax_element, "AXMinimizeButton")
}

/// Check if a window has a close button
#[cfg(target_os = "macos")]
pub fn can_close_window(ax_element: *const c_void) -> bool {
    ax_ffi::has_attribute(ax_element, "AXCloseButton")
}

/// Check if a window has a fullscreen button
#[cfg(target_os = "macos")]
pub fn can_fullscreen_window(ax_element: *const c_void) -> bool {
    ax_ffi::has_attribute(ax_element, "AXFullScreenButton")
}

/// Detect all capabilities for a window
#[cfg(target_os = "macos")]
pub fn detect_window_capabilities(ax_element: *const c_void) -> WindowCapabilities {
    WindowCapabilities {
        can_move: can_move_window(ax_element),
        can_resize: can_resize_window(ax_element),
        can_minimize: can_minimize_window(ax_element),
        can_close: can_close_window(ax_element),
        can_fullscreen: can_fullscreen_window(ax_element),
        supports_space_move: false, // Always false without external backend
    }
}

// ============================================================================
// Non-macOS stubs
// ============================================================================

#[cfg(not(target_os = "macos"))]
pub fn can_move_window(_ax_element: *const c_void) -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn can_resize_window(_ax_element: *const c_void) -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn can_minimize_window(_ax_element: *const c_void) -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn can_close_window(_ax_element: *const c_void) -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn can_fullscreen_window(_ax_element: *const c_void) -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn detect_window_capabilities(_ax_element: *const c_void) -> WindowCapabilities {
    WindowCapabilities::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capabilities() {
        let caps = WindowCapabilities::default();
        assert!(caps.can_move);
        assert!(caps.can_resize);
        assert!(caps.can_minimize);
        assert!(caps.can_close);
        assert!(caps.can_fullscreen);
        assert!(!caps.supports_space_move);
    }

    #[test]
    fn test_enhanced_window_info_can_resize_to() {
        let mut info = EnhancedWindowInfo::new(
            1,
            "Test".to_string(),
            "Window".to_string(),
            123,
            WindowBounds::new(0.0, 0.0, 800.0, 600.0),
            WindowCapabilities::default(),
        );

        // No constraints, should always allow
        assert!(info.can_resize_to(500.0, 400.0));

        // With constraints
        info.size_constraints = SizeConstraints {
            min_width: Some(200.0),
            min_height: Some(200.0),
            max_width: Some(1000.0),
            max_height: Some(800.0),
        };

        assert!(info.can_resize_to(500.0, 400.0));
        assert!(!info.can_resize_to(100.0, 100.0)); // Below min
        assert!(!info.can_resize_to(2000.0, 1000.0)); // Above max

        // Can't resize if capability says no
        info.capabilities.can_resize = false;
        assert!(!info.can_resize_to(500.0, 400.0));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_try_create_cf_string_rejects_interior_nul() {
        assert!(
            ax_ffi::try_create_cf_string("AX\0Title").is_none(),
            "interior NUL should not produce CFString"
        );
    }
}
