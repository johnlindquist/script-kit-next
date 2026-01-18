//! Window Control module using macOS Accessibility APIs
//!
//! This module provides window management functionality including:
//! - Listing all visible windows with their properties
//! - Moving, resizing, minimizing, maximizing, and closing windows
//! - Tiling windows to predefined positions (halves, quadrants, fullscreen)
//!
//! ## Architecture
//!
//! Uses macOS Accessibility APIs (AXUIElement) to control windows across applications.
//! The accessibility framework allows querying and modifying window properties for any
//! application, provided the user has granted accessibility permissions.
//!
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility
//!

// This entire module is macOS-only
#![cfg(target_os = "macos")]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use core_graphics::display::{CGDisplay, CGRect};
use macos_accessibility_client::accessibility;
use std::ffi::c_void;
use tracing::{debug, info, instrument, warn};

// ============================================================================
// CoreFoundation FFI bindings
// ============================================================================

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFRetain(cf: *const c_void) -> *const c_void;
}

// ============================================================================
// ApplicationServices (Accessibility) FFI bindings
// ============================================================================

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> i32;
    fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> i32;
    fn AXValueCreate(value_type: i32, value: *const c_void) -> AXValueRef;
    fn AXValueGetValue(value: AXValueRef, value_type: i32, value_out: *mut c_void) -> bool;
    fn AXValueGetType(value: AXValueRef) -> i32;
}

// AXValue types
const kAXValueTypeCGPoint: i32 = 1;
const kAXValueTypeCGSize: i32 = 2;

// AXError codes
const kAXErrorSuccess: i32 = 0;
const kAXErrorAPIDisabled: i32 = -25211;
const kAXErrorNoValue: i32 = -25212;

type AXUIElementRef = *const c_void;
type AXValueRef = *const c_void;
type CFTypeRef = *const c_void;
type CFStringRef = *const c_void;
type CFArrayRef = *const c_void;

// ============================================================================
// CoreFoundation String FFI bindings
// ============================================================================

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetCString(
        string: CFStringRef,
        buffer: *mut i8,
        buffer_size: i64,
        encoding: u32,
    ) -> bool;
    fn CFStringGetLength(string: CFStringRef) -> i64;
    fn CFArrayGetCount(array: CFArrayRef) -> i64;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: i64) -> CFTypeRef;
    fn CFGetTypeID(cf: CFTypeRef) -> u64;
    fn CFStringGetTypeID() -> u64;
    fn CFNumberGetValue(number: CFTypeRef, number_type: i32, value_ptr: *mut c_void) -> bool;
}

const kCFStringEncodingUTF8: u32 = 0x08000100;
const kCFNumberSInt32Type: i32 = 3;

// ============================================================================
// AppKit (NSWorkspace/NSRunningApplication) FFI bindings
// ============================================================================

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // We'll use objc crate for AppKit access instead of direct FFI
}

// ============================================================================
// Public Types
// ============================================================================

/// Represents the bounds (position and size) of a window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    /// Create a new Bounds
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create bounds from CoreGraphics CGRect
    fn from_cg_rect(rect: CGRect) -> Self {
        Self {
            x: rect.origin.x as i32,
            y: rect.origin.y as i32,
            width: rect.size.width as u32,
            height: rect.size.height as u32,
        }
    }
}

/// Information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Unique window identifier (process ID << 16 | window index)
    pub id: u32,
    /// Application name
    pub app: String,
    /// Window title
    pub title: String,
    /// Window position and size
    pub bounds: Bounds,
    /// Process ID of the owning application
    pub pid: i32,
    /// The AXUIElement reference (internal, for operations)
    #[doc(hidden)]
    ax_window: Option<usize>, // Store as usize to avoid lifetime issues
}

impl WindowInfo {
    /// Get the internal window reference for operations
    fn window_ref(&self) -> Option<AXUIElementRef> {
        self.ax_window.map(|ptr| ptr as AXUIElementRef)
    }
}

/// Tiling positions for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePosition {
    // Half positions
    /// Left half of the screen
    LeftHalf,
    /// Right half of the screen
    RightHalf,
    /// Top half of the screen
    TopHalf,
    /// Bottom half of the screen
    BottomHalf,

    // Quadrant positions
    /// Top-left quadrant
    TopLeft,
    /// Top-right quadrant
    TopRight,
    /// Bottom-left quadrant
    BottomLeft,
    /// Bottom-right quadrant
    BottomRight,

    // Horizontal thirds positions
    /// Left third of the screen
    LeftThird,
    /// Center third of the screen (horizontal)
    CenterThird,
    /// Right third of the screen
    RightThird,

    // Vertical thirds positions
    /// Top third of the screen
    TopThird,
    /// Middle third of the screen (vertical)
    MiddleThird,
    /// Bottom third of the screen
    BottomThird,

    // Horizontal two-thirds positions
    /// First two-thirds of the screen (left side)
    FirstTwoThirds,
    /// Last two-thirds of the screen (right side)
    LastTwoThirds,

    // Vertical two-thirds positions
    /// Top two-thirds of the screen
    TopTwoThirds,
    /// Bottom two-thirds of the screen
    BottomTwoThirds,

    // Centered positions
    /// Centered on screen (60% of screen dimensions)
    Center,
    /// Almost maximize (90% with margins)
    AlmostMaximize,

    /// Fullscreen (covers entire display)
    Fullscreen,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a CFString from a Rust string
fn create_cf_string(s: &str) -> CFStringRef {
    unsafe {
        let c_str = std::ffi::CString::new(s).unwrap();
        CFStringCreateWithCString(std::ptr::null(), c_str.as_ptr(), kCFStringEncodingUTF8)
    }
}

/// Convert a CFString to a Rust String
fn cf_string_to_string(cf_string: CFStringRef) -> Option<String> {
    if cf_string.is_null() {
        return None;
    }

    unsafe {
        let length = CFStringGetLength(cf_string);
        if length <= 0 {
            return Some(String::new());
        }

        // Allocate buffer with extra space for UTF-8 expansion
        let buffer_size = (length * 4 + 1) as usize;
        let mut buffer: Vec<i8> = vec![0; buffer_size];

        if CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr(),
            buffer_size as i64,
            kCFStringEncodingUTF8,
        ) {
            let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr());
            c_str.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }
}

/// Release a CoreFoundation object
fn cf_release(cf: CFTypeRef) {
    if !cf.is_null() {
        unsafe {
            CFRelease(cf);
        }
    }
}

/// Retain a CoreFoundation object (increment reference count)
/// Returns the same pointer for convenience
fn cf_retain(cf: CFTypeRef) -> CFTypeRef {
    if !cf.is_null() {
        unsafe { CFRetain(cf) }
    } else {
        cf
    }
}

/// Get an attribute value from an AXUIElement
fn get_ax_attribute(element: AXUIElementRef, attribute: &str) -> Result<CFTypeRef> {
    let attr_str = create_cf_string(attribute);
    let mut value: CFTypeRef = std::ptr::null();

    let result =
        unsafe { AXUIElementCopyAttributeValue(element, attr_str, &mut value as *mut CFTypeRef) };

    cf_release(attr_str);

    match result {
        kAXErrorSuccess => Ok(value),
        kAXErrorAPIDisabled => bail!("Accessibility API is disabled"),
        kAXErrorNoValue => bail!("No value for attribute: {}", attribute),
        _ => bail!("Failed to get attribute {}: error {}", attribute, result),
    }
}

/// Set an attribute value on an AXUIElement
fn set_ax_attribute(element: AXUIElementRef, attribute: &str, value: CFTypeRef) -> Result<()> {
    let attr_str = create_cf_string(attribute);

    let result = unsafe { AXUIElementSetAttributeValue(element, attr_str, value) };

    cf_release(attr_str);

    match result {
        kAXErrorSuccess => Ok(()),
        kAXErrorAPIDisabled => bail!("Accessibility API is disabled"),
        _ => bail!("Failed to set attribute {}: error {}", attribute, result),
    }
}

/// Perform an action on an AXUIElement
fn perform_ax_action(element: AXUIElementRef, action: &str) -> Result<()> {
    let action_str = create_cf_string(action);

    let result = unsafe { AXUIElementPerformAction(element, action_str) };

    cf_release(action_str);

    match result {
        kAXErrorSuccess => Ok(()),
        kAXErrorAPIDisabled => bail!("Accessibility API is disabled"),
        _ => bail!("Failed to perform action {}: error {}", action, result),
    }
}

/// Get the position of a window
fn get_window_position(window: AXUIElementRef) -> Result<(i32, i32)> {
    let value = get_ax_attribute(window, "AXPosition")?;

    let mut point = core_graphics::geometry::CGPoint::new(0.0, 0.0);
    let success = unsafe {
        AXValueGetValue(
            value,
            kAXValueTypeCGPoint,
            &mut point as *mut _ as *mut c_void,
        )
    };

    cf_release(value);

    if success {
        Ok((point.x as i32, point.y as i32))
    } else {
        bail!("Failed to extract position value")
    }
}

/// Get the size of a window
fn get_window_size(window: AXUIElementRef) -> Result<(u32, u32)> {
    let value = get_ax_attribute(window, "AXSize")?;

    let mut size = core_graphics::geometry::CGSize::new(0.0, 0.0);
    let success = unsafe {
        AXValueGetValue(
            value,
            kAXValueTypeCGSize,
            &mut size as *mut _ as *mut c_void,
        )
    };

    cf_release(value);

    if success {
        Ok((size.width as u32, size.height as u32))
    } else {
        bail!("Failed to extract size value")
    }
}

/// Set the position of a window
fn set_window_position(window: AXUIElementRef, x: i32, y: i32) -> Result<()> {
    let point = core_graphics::geometry::CGPoint::new(x as f64, y as f64);
    let value = unsafe { AXValueCreate(kAXValueTypeCGPoint, &point as *const _ as *const c_void) };

    if value.is_null() {
        bail!("Failed to create AXValue for position");
    }

    let result = set_ax_attribute(window, "AXPosition", value);
    cf_release(value);
    result
}

/// Set the size of a window
fn set_window_size(window: AXUIElementRef, width: u32, height: u32) -> Result<()> {
    let size = core_graphics::geometry::CGSize::new(width as f64, height as f64);
    let value = unsafe { AXValueCreate(kAXValueTypeCGSize, &size as *const _ as *const c_void) };

    if value.is_null() {
        bail!("Failed to create AXValue for size");
    }

    let result = set_ax_attribute(window, "AXSize", value);
    cf_release(value);
    result
}

/// Get the string value of a window attribute
fn get_window_string_attribute(window: AXUIElementRef, attribute: &str) -> Option<String> {
    match get_ax_attribute(window, attribute) {
        Ok(value) => {
            // Check if it's a CFString
            let type_id = unsafe { CFGetTypeID(value) };
            let string_type_id = unsafe { CFStringGetTypeID() };

            let result = if type_id == string_type_id {
                cf_string_to_string(value as CFStringRef)
            } else {
                None
            };

            cf_release(value);
            result
        }
        Err(_) => None,
    }
}

/// Get the main display bounds
fn get_main_display_bounds() -> Bounds {
    let main_display = CGDisplay::main();
    let rect = main_display.bounds();
    Bounds::from_cg_rect(rect)
}

/// Get the display bounds for the display containing a point
fn get_display_bounds_at_point(_x: i32, _y: i32) -> Bounds {
    // For simplicity, we'll use the main display
    // A more complete implementation would find the display containing the point
    get_main_display_bounds()
}

// ============================================================================
// Window Cache for lookups
// ============================================================================

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Global window cache using OnceLock (std alternative to lazy_static)
static WINDOW_CACHE: OnceLock<Mutex<HashMap<u32, usize>>> = OnceLock::new();

/// Get or initialize the window cache
fn get_cache() -> &'static Mutex<HashMap<u32, usize>> {
    WINDOW_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_window(id: u32, window_ref: AXUIElementRef) {
    if let Ok(mut cache) = get_cache().lock() {
        cache.insert(id, window_ref as usize);
    }
}

fn get_cached_window(id: u32) -> Option<AXUIElementRef> {
    get_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(&id).map(|&ptr| ptr as AXUIElementRef))
}

fn clear_window_cache() {
    if let Ok(mut cache) = get_cache().lock() {
        // Release all retained window refs before clearing
        for &window_ptr in cache.values() {
            cf_release(window_ptr as CFTypeRef);
        }
        cache.clear();
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Check if accessibility permissions are granted.
///
/// Window control operations require the application to have accessibility
/// permissions granted by the user.
///
/// # Returns
/// `true` if permission is granted, `false` otherwise.
#[instrument]
pub fn has_accessibility_permission() -> bool {
    let result = accessibility::application_is_trusted();
    debug!(granted = result, "Checked accessibility permission");
    result
}

/// Request accessibility permissions (opens System Preferences).
///
/// # Returns
/// `true` if permission is granted after the request, `false` otherwise.
#[instrument]
pub fn request_accessibility_permission() -> bool {
    info!("Requesting accessibility permission for window control");
    accessibility::application_is_trusted_with_prompt()
}

/// List all visible windows across all applications.
///
/// Returns a vector of `WindowInfo` structs containing window metadata.
/// Windows are filtered to only include visible, standard windows.
///
/// # Returns
/// A vector of window information structs.
///
/// # Errors
/// Returns error if accessibility permission is not granted.
///
#[instrument]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    if !has_accessibility_permission() {
        bail!("Accessibility permission required for window control");
    }

    // Clear the cache before listing
    clear_window_cache();

    let mut windows = Vec::new();

    // Get list of running applications using objc
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        let app_count: usize = msg_send![running_apps, count];

        for i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];

            // Check activation policy (skip background apps)
            let activation_policy: i64 = msg_send![app, activationPolicy];
            if activation_policy != 0 {
                // 0 = NSApplicationActivationPolicyRegular
                continue;
            }

            // Get app name
            let app_name: *mut Object = msg_send![app, localizedName];
            let app_name_str = if !app_name.is_null() {
                let utf8: *const i8 = msg_send![app_name, UTF8String];
                if !utf8.is_null() {
                    std::ffi::CStr::from_ptr(utf8)
                        .to_str()
                        .unwrap_or("Unknown")
                        .to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            // Get PID
            let pid: i32 = msg_send![app, processIdentifier];

            // Create AXUIElement for this application
            let ax_app = AXUIElementCreateApplication(pid);
            if ax_app.is_null() {
                continue;
            }

            // Get windows for this app
            if let Ok(windows_value) = get_ax_attribute(ax_app, "AXWindows") {
                let window_count = CFArrayGetCount(windows_value as CFArrayRef);

                for j in 0..window_count {
                    // CFArrayGetValueAtIndex returns a borrowed reference - we must retain
                    // if we want to keep it beyond the array's lifetime
                    let ax_window = CFArrayGetValueAtIndex(windows_value as CFArrayRef, j);

                    // Get window title
                    let title = get_window_string_attribute(ax_window as AXUIElementRef, "AXTitle")
                        .unwrap_or_default();

                    // Skip windows without titles (often utility windows)
                    if title.is_empty() {
                        continue;
                    }

                    // Get window position and size
                    let (x, y) = get_window_position(ax_window as AXUIElementRef).unwrap_or((0, 0));
                    let (width, height) =
                        get_window_size(ax_window as AXUIElementRef).unwrap_or((0, 0));

                    // Skip very small windows (likely invisible or popups)
                    if width < 50 || height < 50 {
                        continue;
                    }

                    // Create a unique window ID: (pid << 16) | window_index
                    let window_id = ((pid as u32) << 16) | (j as u32);

                    // Retain the window ref before caching - CFArrayGetValueAtIndex returns
                    // a borrowed reference, so we need to retain it to extend its lifetime
                    // beyond when we release windows_value
                    let retained_window = cf_retain(ax_window);
                    cache_window(window_id, retained_window as AXUIElementRef);

                    windows.push(WindowInfo {
                        id: window_id,
                        app: app_name_str.clone(),
                        title,
                        bounds: Bounds::new(x, y, width, height),
                        pid,
                        ax_window: Some(retained_window as usize),
                    });
                }

                // Release windows_value - AXUIElementCopyAttributeValue returns an owned
                // CF object that we must release (the "Copy" in the name means we own it)
                cf_release(windows_value);
            }

            // Release ax_app - AXUIElementCreateApplication returns an owned CF object
            cf_release(ax_app);
        }
    }

    info!(window_count = windows.len(), "Listed windows");
    Ok(windows)
}

/// Get the PID of the application that owns the menu bar.
///
/// When Script Kit (an accessory/LSUIElement app) is active, it does NOT take
/// menu bar ownership. The previously active "regular" app still owns the menu bar.
/// This is exactly what we need for window actions - we want to act on the
/// window that was focused before Script Kit was shown.
///
/// # Returns
/// The process identifier (PID) of the menu bar owning application.
///
/// # Errors
/// Returns error if no menu bar owner is found or if the PID is invalid.
#[instrument]
pub fn get_menu_bar_owner_pid() -> Result<i32> {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let menu_owner: *mut Object = msg_send![workspace, menuBarOwningApplication];

        if menu_owner.is_null() {
            bail!("No menu bar owning application found");
        }

        let pid: i32 = msg_send![menu_owner, processIdentifier];

        if pid <= 0 {
            bail!("Invalid process identifier for menu bar owner");
        }

        // Log for debugging
        let name: *mut Object = msg_send![menu_owner, localizedName];
        let name_str = if !name.is_null() {
            let utf8: *const i8 = msg_send![name, UTF8String];
            if !utf8.is_null() {
                std::ffi::CStr::from_ptr(utf8).to_str().unwrap_or("unknown")
            } else {
                "unknown"
            }
        } else {
            "unknown"
        };

        info!(pid, app_name = name_str, "Got menu bar owner");
        Ok(pid)
    }
}

/// Get the frontmost window of the menu bar owning application.
///
/// This is the key function for window actions from Script Kit. When the user
/// executes "Tile Window Left" etc., we want to act on the window they were
/// using before invoking Script Kit, not Script Kit's own window.
///
/// Since Script Kit is an LSUIElement (accessory app), it doesn't take menu bar
/// ownership. The menu bar owner is the previously active app.
///
/// # Window Selection Strategy
///
/// 1. First try AXFocusedWindow - the actual focused window of the app
/// 2. If that fails, try AXMainWindow - the app's designated "main" window
/// 3. Fall back to first window in AXWindows array if neither works
///
/// This is more accurate than just picking the first window with matching pid,
/// which can return the wrong window if the app has multiple windows open.
///
/// # Returns
/// The focused/main window of the menu bar owning application, or None if
/// no windows are found.
#[instrument]
pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
    let target_pid = get_menu_bar_owner_pid()?;

    unsafe {
        // Create AX element for the target application
        let ax_app = AXUIElementCreateApplication(target_pid);
        if ax_app.is_null() {
            warn!(target_pid, "Failed to create AXUIElement for app");
            return Ok(None);
        }

        // Strategy 1: Try to get the focused window (most accurate)
        let focused_window = get_ax_attribute(ax_app as AXUIElementRef, "AXFocusedWindow")
            .ok()
            .filter(|&w| !w.is_null());

        // Strategy 2: Fall back to main window
        let target_window = focused_window.or_else(|| {
            get_ax_attribute(ax_app as AXUIElementRef, "AXMainWindow")
                .ok()
                .filter(|&w| !w.is_null())
        });

        // Strategy 3: Fall back to first window in AXWindows list
        let target_window = target_window.or_else(|| {
            if let Ok(windows_value) = get_ax_attribute(ax_app as AXUIElementRef, "AXWindows") {
                let count = CFArrayGetCount(windows_value as CFArrayRef);
                if count > 0 {
                    let window = CFArrayGetValueAtIndex(windows_value as CFArrayRef, 0);
                    // Retain the window ref since CFArrayGetValueAtIndex returns borrowed
                    let retained = cf_retain(window);
                    cf_release(windows_value);
                    Some(retained)
                } else {
                    cf_release(windows_value);
                    None
                }
            } else {
                None
            }
        });

        // Release ax_app - we're done with it
        cf_release(ax_app);

        // If we found a target window, build WindowInfo for it
        if let Some(window_ref) = target_window {
            let ax_window = window_ref as AXUIElementRef;

            // Get window attributes
            let title = get_window_string_attribute(ax_window, "AXTitle").unwrap_or_default();
            let (x, y) = get_window_position(ax_window).unwrap_or((0, 0));
            let (width, height) = get_window_size(ax_window).unwrap_or((0, 0));

            // Get app name for logging
            let app_name = get_app_name_for_pid(target_pid);

            // Create a window ID (focused window uses index 0)
            let window_id = (target_pid as u32) << 16;

            // Cache the window reference for subsequent operations
            cache_window(window_id, ax_window);

            let window_info = WindowInfo {
                id: window_id,
                app: app_name.clone(),
                title: title.clone(),
                bounds: Bounds::new(x, y, width, height),
                pid: target_pid,
                ax_window: Some(window_ref as usize),
            };

            info!(
                window_id = window_info.id,
                app = %app_name,
                title = %title,
                "Found focused/main window of previous app via AX"
            );

            Ok(Some(window_info))
        } else {
            warn!(
                target_pid,
                "No focused or main window found for menu bar owner"
            );
            Ok(None)
        }
    }
}

/// Get the localized app name for a given PID.
fn get_app_name_for_pid(pid: i32) -> String {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        if let Some(workspace_class) = Class::get("NSWorkspace") {
            let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
            let running_apps: *mut Object = msg_send![workspace, runningApplications];
            let app_count: usize = msg_send![running_apps, count];

            for i in 0..app_count {
                let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
                let app_pid: i32 = msg_send![app, processIdentifier];

                if app_pid == pid {
                    let app_name: *mut Object = msg_send![app, localizedName];
                    if !app_name.is_null() {
                        let utf8: *const i8 = msg_send![app_name, UTF8String];
                        if !utf8.is_null() {
                            return std::ffi::CStr::from_ptr(utf8)
                                .to_str()
                                .unwrap_or("Unknown")
                                .to_string();
                        }
                    }
                    break;
                }
            }
        }

        "Unknown".to_string()
    }
}

/// Move a window to a new position.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `x` - The new X coordinate (screen pixels from left)
/// * `y` - The new Y coordinate (screen pixels from top)
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn move_window(window_id: u32, x: i32, y: i32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            // Try to refresh the cache
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    set_window_position(window, x, y)?;
    info!(window_id, x, y, "Moved window");
    Ok(())
}

/// Resize a window to new dimensions.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `width` - The new width in pixels
/// * `height` - The new height in pixels
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn resize_window(window_id: u32, width: u32, height: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    set_window_size(window, width, height)?;
    info!(window_id, width, height, "Resized window");
    Ok(())
}

/// Set the complete bounds (position and size) of a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `bounds` - The new bounds for the window
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn set_window_bounds(window_id: u32, bounds: Bounds) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Set position first, then size
    set_window_position(window, bounds.x, bounds.y)?;
    set_window_size(window, bounds.width, bounds.height)?;

    info!(
        window_id,
        x = bounds.x,
        y = bounds.y,
        width = bounds.width,
        height = bounds.height,
        "Set window bounds"
    );
    Ok(())
}

/// Minimize a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn minimize_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Use AXMinimized attribute to minimize
    let true_value = create_cf_string("1");
    let minimize_attr = create_cf_string("AXMinimized");

    // AXMinimized expects a CFBoolean, so we need to use the attribute differently
    // Actually, we should perform the press action on the minimize button
    // or set the AXMinimized attribute to true

    // Try setting AXMinimized directly with a boolean value
    unsafe {
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            static kCFBooleanTrue: CFTypeRef;
        }

        let result = AXUIElementSetAttributeValue(window, minimize_attr, kCFBooleanTrue);

        cf_release(minimize_attr);
        cf_release(true_value);

        if result != kAXErrorSuccess {
            bail!("Failed to minimize window: error {}", result);
        }
    }

    info!(window_id, "Minimized window");
    Ok(())
}

/// Maximize a window (fills the display without entering fullscreen mode).
///
/// This positions the window to fill the available display area (excluding
/// dock and menu bar) without entering macOS fullscreen mode.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn maximize_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get current position to determine which display the window is on
    let (current_x, current_y) = get_window_position(window).unwrap_or((0, 0));

    // Get the display bounds (accounting for menu bar and dock)
    let display_bounds = get_visible_display_bounds(current_x, current_y);

    // Set the window to fill the visible area
    set_window_position(window, display_bounds.x, display_bounds.y)?;
    set_window_size(window, display_bounds.width, display_bounds.height)?;

    info!(window_id, "Maximized window");
    Ok(())
}

/// Tile a window to a predefined position on the screen.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `position` - The tiling position (half, quadrant, or fullscreen)
///
/// # Errors
/// Returns error if window not found or operation fails.
///
#[instrument]
pub fn tile_window(window_id: u32, position: TilePosition) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get current position to determine which display the window is on
    let (current_x, current_y) = get_window_position(window).unwrap_or((0, 0));

    // Get the visible display bounds (accounting for menu bar and dock)
    let display = get_visible_display_bounds(current_x, current_y);

    let bounds = calculate_tile_bounds(&display, position);

    set_window_position(window, bounds.x, bounds.y)?;
    set_window_size(window, bounds.width, bounds.height)?;

    info!(window_id, ?position, "Tiled window");
    Ok(())
}

/// Close a window.
///
/// Note: This performs the close action on the window, which may prompt
/// the user to save unsaved changes depending on the application.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn close_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get the close button and press it
    if let Ok(close_button) = get_ax_attribute(window, "AXCloseButton") {
        perform_ax_action(close_button as AXUIElementRef, "AXPress")?;
        cf_release(close_button);
    } else {
        bail!("Window does not have a close button");
    }

    info!(window_id, "Closed window");
    Ok(())
}

/// Focus (bring to front) a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn focus_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Raise the window
    perform_ax_action(window, "AXRaise")?;

    // Also activate the owning application
    let pid = (window_id >> 16) as i32;

    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        let app_count: usize = msg_send![running_apps, count];

        for i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
            let app_pid: i32 = msg_send![app, processIdentifier];

            if app_pid == pid {
                let _: bool = msg_send![app, activateWithOptions: 1u64]; // NSApplicationActivateIgnoringOtherApps
                break;
            }
        }
    }

    info!(window_id, "Focused window");
    Ok(())
}

/// Move a window to the next display (cycles through available displays).
#[instrument]
pub fn move_to_next_display(window_id: u32) -> Result<()> {
    move_to_adjacent_display(window_id, true)
}

/// Move a window to the previous display (cycles through available displays).
#[instrument]
pub fn move_to_previous_display(window_id: u32) -> Result<()> {
    move_to_adjacent_display(window_id, false)
}

/// Internal helper to move window to adjacent display
fn move_to_adjacent_display(window_id: u32, next: bool) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    let (current_x, current_y) = get_window_position(window).unwrap_or((0, 0));
    let (current_width, current_height) = get_window_size(window).unwrap_or((800, 600));

    let displays = get_all_display_bounds()?;
    if displays.len() <= 1 {
        info!(window_id, "Only one display, cannot move to adjacent");
        return Ok(());
    }

    let current_display_idx = displays
        .iter()
        .position(|d| {
            current_x >= d.x
                && current_x < d.x + d.width as i32
                && current_y >= d.y
                && current_y < d.y + d.height as i32
        })
        .unwrap_or(0);

    let target_idx = if next {
        (current_display_idx + 1) % displays.len()
    } else if current_display_idx == 0 {
        displays.len() - 1
    } else {
        current_display_idx - 1
    };

    let current_display = &displays[current_display_idx];
    let target_display = &displays[target_idx];

    let rel_x = (current_x - current_display.x) as f64 / current_display.width as f64;
    let rel_y = (current_y - current_display.y) as f64 / current_display.height as f64;

    let new_x = target_display.x + (rel_x * target_display.width as f64) as i32;
    let new_y = target_display.y + (rel_y * target_display.height as f64) as i32;

    let scale_x = target_display.width as f64 / current_display.width as f64;
    let scale_y = target_display.height as f64 / current_display.height as f64;
    let new_width = (current_width as f64 * scale_x).min(target_display.width as f64) as u32;
    let new_height = (current_height as f64 * scale_y).min(target_display.height as f64) as u32;

    set_window_position(window, new_x, new_y)?;
    set_window_size(window, new_width, new_height)?;

    info!(
        window_id,
        from_display = current_display_idx,
        to_display = target_idx,
        "Moved window to {} display",
        if next { "next" } else { "previous" }
    );
    Ok(())
}

/// Get bounds for all available displays
fn get_all_display_bounds() -> Result<Vec<Bounds>> {
    let mut displays = Vec::new();

    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen_class = Class::get("NSScreen").context("Failed to get NSScreen class")?;
        let screens: *mut Object = msg_send![nsscreen_class, screens];
        if screens.is_null() {
            bail!("Failed to get screens");
        }

        let screen_count: usize = msg_send![screens, count];
        if screen_count == 0 {
            bail!("No screens found");
        }

        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        let primary_frame: CGRect = msg_send![primary_screen, frame];
        let primary_height = primary_frame.size.height;

        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            let visible_frame: CGRect = msg_send![screen, visibleFrame];
            let cg_y = primary_height - (visible_frame.origin.y + visible_frame.size.height);

            displays.push(Bounds {
                x: visible_frame.origin.x as i32,
                y: cg_y as i32,
                width: visible_frame.size.width as u32,
                height: visible_frame.size.height as u32,
            });
        }
    }

    Ok(displays)
}

// ============================================================================
// Helper Functions for Display Bounds
// ============================================================================

/// Get the visible display bounds (excluding menu bar and dock) for the display
/// containing the given point.
///
/// Uses NSScreen.visibleFrame to get accurate bounds that account for:
/// - Menu bar (on main display)
/// - Dock (on any edge, any display)
/// - Notch area (on newer MacBooks)
fn get_visible_display_bounds(x: i32, y: i32) -> Bounds {
    // Use NSScreen to get accurate visible frame
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen_class = match Class::get("NSScreen") {
            Some(c) => c,
            None => return get_visible_display_bounds_fallback(x, y),
        };

        // Get all screens
        let screens: *mut Object = msg_send![nsscreen_class, screens];
        if screens.is_null() {
            return get_visible_display_bounds_fallback(x, y);
        }

        let screen_count: usize = msg_send![screens, count];
        if screen_count == 0 {
            return get_visible_display_bounds_fallback(x, y);
        }

        // Get the PRIMARY screen height for coordinate conversion (do this once outside the loop)
        // IMPORTANT: We MUST use screens[0] (the primary screen with the menu bar),
        // NOT mainScreen (which is the screen with the key window).
        // Cocoa coordinates have their origin at bottom-left of the PRIMARY screen,
        // and CoreGraphics coordinates have origin at top-left of PRIMARY screen.
        // Using mainScreen causes incorrect coordinate conversion on multi-monitor setups
        // when the focused window is on a secondary display.
        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        let primary_frame: CGRect = msg_send![primary_screen, frame];
        let primary_height = primary_frame.size.height;

        // Convert CG y to Cocoa y (once, outside the loop)
        let cocoa_y = primary_height - y as f64;

        // Find the screen containing the point
        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            // Get the full frame (in Cocoa coordinates - origin at bottom-left)
            let frame: CGRect = msg_send![screen, frame];

            // Check if point is within this screen's frame
            if (x as f64) >= frame.origin.x
                && (x as f64) < frame.origin.x + frame.size.width
                && cocoa_y >= frame.origin.y
                && cocoa_y < frame.origin.y + frame.size.height
            {
                // Get the visible frame (excludes menu bar and dock)
                let visible_frame: CGRect = msg_send![screen, visibleFrame];

                // Convert Cocoa coordinates back to CoreGraphics coordinates
                // CG origin is at top-left of primary screen
                // Cocoa origin.y is distance from bottom of primary screen
                // CG y = primary_height - (cocoa_y + height)
                let cg_y = primary_height - (visible_frame.origin.y + visible_frame.size.height);

                debug!(
                    screen_index = i,
                    frame_x = frame.origin.x,
                    frame_y = frame.origin.y,
                    frame_w = frame.size.width,
                    frame_h = frame.size.height,
                    visible_x = visible_frame.origin.x,
                    visible_y = visible_frame.origin.y,
                    visible_w = visible_frame.size.width,
                    visible_h = visible_frame.size.height,
                    cg_y = cg_y,
                    "Found screen for point ({}, {})",
                    x,
                    y
                );

                return Bounds {
                    x: visible_frame.origin.x as i32,
                    y: cg_y as i32,
                    width: visible_frame.size.width as u32,
                    height: visible_frame.size.height as u32,
                };
            }
        }
    }

    // Fallback if no screen found
    get_visible_display_bounds_fallback(x, y)
}

/// Fallback method using CGDisplay when NSScreen is unavailable
fn get_visible_display_bounds_fallback(x: i32, y: i32) -> Bounds {
    // Get all displays
    if let Ok(display_ids) = CGDisplay::active_displays() {
        for display_id in display_ids {
            let display = CGDisplay::new(display_id);
            let frame = display.bounds();

            // Check if point is within this display
            if x >= frame.origin.x as i32
                && x < (frame.origin.x + frame.size.width) as i32
                && y >= frame.origin.y as i32
                && y < (frame.origin.y + frame.size.height) as i32
            {
                let is_main = display_id == CGDisplay::main().id;

                // Conservative estimates for menu bar and dock
                let menu_bar_height = if is_main { 25 } else { 0 };
                let dock_height = if is_main { 70 } else { 0 };

                return Bounds {
                    x: frame.origin.x as i32,
                    y: frame.origin.y as i32 + menu_bar_height,
                    width: frame.size.width as u32,
                    height: (frame.size.height as i32 - menu_bar_height - dock_height) as u32,
                };
            }
        }
    }

    // Final fallback to main display
    let main = CGDisplay::main();
    let frame = main.bounds();
    Bounds {
        x: frame.origin.x as i32,
        y: frame.origin.y as i32 + 25,
        width: frame.size.width as u32,
        height: (frame.size.height - 95.0) as u32,
    }
}

/// Calculate the bounds for a tiling position within a display.
fn calculate_tile_bounds(display: &Bounds, position: TilePosition) -> Bounds {
    let half_width = display.width / 2;
    let half_height = display.height / 2;
    let third_width = display.width / 3;
    let third_height = display.height / 3;
    let two_thirds_width = (display.width * 2) / 3;
    let two_thirds_height = (display.height * 2) / 3;

    match position {
        // Half positions
        TilePosition::LeftHalf => Bounds {
            x: display.x,
            y: display.y,
            width: half_width,
            height: display.height,
        },
        TilePosition::RightHalf => Bounds {
            x: display.x + half_width as i32,
            y: display.y,
            width: half_width,
            height: display.height,
        },
        TilePosition::TopHalf => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: half_height,
        },
        TilePosition::BottomHalf => Bounds {
            x: display.x,
            y: display.y + half_height as i32,
            width: display.width,
            height: half_height,
        },

        // Quadrant positions
        TilePosition::TopLeft => Bounds {
            x: display.x,
            y: display.y,
            width: half_width,
            height: half_height,
        },
        TilePosition::TopRight => Bounds {
            x: display.x + half_width as i32,
            y: display.y,
            width: half_width,
            height: half_height,
        },
        TilePosition::BottomLeft => Bounds {
            x: display.x,
            y: display.y + half_height as i32,
            width: half_width,
            height: half_height,
        },
        TilePosition::BottomRight => Bounds {
            x: display.x + half_width as i32,
            y: display.y + half_height as i32,
            width: half_width,
            height: half_height,
        },

        // Horizontal thirds positions
        TilePosition::LeftThird => Bounds {
            x: display.x,
            y: display.y,
            width: third_width,
            height: display.height,
        },
        TilePosition::CenterThird => Bounds {
            x: display.x + third_width as i32,
            y: display.y,
            width: third_width,
            height: display.height,
        },
        TilePosition::RightThird => Bounds {
            x: display.x + (two_thirds_width) as i32,
            y: display.y,
            width: third_width,
            height: display.height,
        },

        // Vertical thirds positions
        TilePosition::TopThird => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: third_height,
        },
        TilePosition::MiddleThird => Bounds {
            x: display.x,
            y: display.y + third_height as i32,
            width: display.width,
            height: third_height,
        },
        TilePosition::BottomThird => Bounds {
            x: display.x,
            y: display.y + two_thirds_height as i32,
            width: display.width,
            height: third_height,
        },

        // Horizontal two-thirds positions
        TilePosition::FirstTwoThirds => Bounds {
            x: display.x,
            y: display.y,
            width: two_thirds_width,
            height: display.height,
        },
        TilePosition::LastTwoThirds => Bounds {
            x: display.x + third_width as i32,
            y: display.y,
            width: two_thirds_width,
            height: display.height,
        },

        // Vertical two-thirds positions
        TilePosition::TopTwoThirds => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: two_thirds_height,
        },
        TilePosition::BottomTwoThirds => Bounds {
            x: display.x,
            y: display.y + third_height as i32,
            width: display.width,
            height: two_thirds_height,
        },

        // Centered positions
        TilePosition::Center => {
            // 60% of screen, centered
            let width = (display.width * 60) / 100;
            let height = (display.height * 60) / 100;
            let x_offset = (display.width - width) / 2;
            let y_offset = (display.height - height) / 2;
            Bounds {
                x: display.x + x_offset as i32,
                y: display.y + y_offset as i32,
                width,
                height,
            }
        }
        TilePosition::AlmostMaximize => {
            // 90% of screen with margins
            let margin_x = (display.width * 5) / 100; // 5% margin on each side
            let margin_y = (display.height * 5) / 100;
            Bounds {
                x: display.x + margin_x as i32,
                y: display.y + margin_y as i32,
                width: display.width - (margin_x * 2),
                height: display.height - (margin_y * 2),
            }
        }

        TilePosition::Fullscreen => *display,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_new() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 20);
        assert_eq!(bounds.width, 100);
        assert_eq!(bounds.height, 200);
    }

    #[test]
    fn test_calculate_tile_bounds_left_half() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::LeftHalf);

        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 1055);
    }

    #[test]
    fn test_calculate_tile_bounds_right_half() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::RightHalf);

        assert_eq!(bounds.x, 960);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 1055);
    }

    #[test]
    fn test_calculate_tile_bounds_top_left() {
        let display = Bounds::new(0, 25, 1920, 1080);
        let bounds = calculate_tile_bounds(&display, TilePosition::TopLeft);

        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 540);
    }

    #[test]
    fn test_calculate_tile_bounds_fullscreen() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::Fullscreen);

        assert_eq!(bounds, display);
    }

    #[test]
    fn test_tile_position_equality() {
        assert_eq!(TilePosition::LeftHalf, TilePosition::LeftHalf);
        assert_ne!(TilePosition::LeftHalf, TilePosition::RightHalf);
    }

    #[test]
    fn test_permission_check_does_not_panic() {
        // This test verifies the permission check doesn't panic
        let _has_permission = has_accessibility_permission();
    }

    #[test]
    #[ignore] // Requires accessibility permission
    fn test_list_windows() {
        let windows = list_windows().expect("Should list windows");
        println!("Found {} windows:", windows.len());
        for window in &windows {
            println!(
                "  [{:08x}] {}: {} ({:?})",
                window.id, window.app, window.title, window.bounds
            );
        }
    }

    #[test]
    #[ignore] // Requires accessibility permission and a visible window
    fn test_tile_window_left_half() {
        let windows = list_windows().expect("Should list windows");
        if let Some(window) = windows.first() {
            tile_window(window.id, TilePosition::LeftHalf).expect("Should tile window");
            println!("Tiled '{}' to left half", window.title);
        } else {
            panic!("No windows found to test with");
        }
    }
}
