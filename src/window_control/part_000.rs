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

    // Sixth positions (top/bottom row split into thirds)
    /// Top-left sixth (left third of top half)
    TopLeftSixth,
    /// Top-center sixth (center third of top half)
    TopCenterSixth,
    /// Top-right sixth (right third of top half)
    TopRightSixth,
    /// Bottom-left sixth (left third of bottom half)
    BottomLeftSixth,
    /// Bottom-center sixth (center third of bottom half)
    BottomCenterSixth,
    /// Bottom-right sixth (right third of bottom half)
    BottomRightSixth,

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
    /// Move to the next display (multi-display routing handled elsewhere)
    NextDisplay,
    /// Move to the previous display (multi-display routing handled elsewhere)
    PreviousDisplay,
}
// ============================================================================
// Helper Functions
// ============================================================================

/// Create a CFString from a Rust string.
fn try_create_cf_string(s: &str) -> Result<CFStringRef> {
    let c_str = std::ffi::CString::new(s)
        .with_context(|| format!("CFString input contains interior NUL: {:?}", s))?;
    let cf_string = unsafe {
        CFStringCreateWithCString(std::ptr::null(), c_str.as_ptr(), kCFStringEncodingUTF8)
    };
    if cf_string.is_null() {
        bail!("CFStringCreateWithCString returned null for input: {:?}", s);
    }
    Ok(cf_string)
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
    let attr_str = try_create_cf_string(attribute)?;
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
    let attr_str = try_create_cf_string(attribute)?;

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
    let action_str = try_create_cf_string(action)?;

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
/// An owned cached window reference retained while in use.
struct OwnedCachedWindowRef {
    window_ref: AXUIElementRef,
}
impl OwnedCachedWindowRef {
    fn as_ptr(&self) -> AXUIElementRef {
        self.window_ref
    }
}
impl Drop for OwnedCachedWindowRef {
    fn drop(&mut self) {
        cf_release(self.window_ref as CFTypeRef);
    }
}
/// Get or initialize the window cache
fn get_cache() -> &'static Mutex<HashMap<u32, usize>> {
    WINDOW_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
fn cache_window(id: u32, window_ref: AXUIElementRef) {
    if let Ok(mut cache) = get_cache().lock() {
        if let Some(previous) = cache.insert(id, window_ref as usize) {
            // The cache owns retained window references. Replacing an entry must
            // release the previous retained pointer to avoid leaks.
            cf_release(previous as CFTypeRef);
        }
    }
}
fn get_cached_window(id: u32) -> Option<OwnedCachedWindowRef> {
    let cache = get_cache().lock().ok()?;
    let ptr = *cache.get(&id)?;
    let retained = cf_retain(ptr as CFTypeRef) as AXUIElementRef;
    if retained.is_null() {
        None
    } else {
        Some(OwnedCachedWindowRef {
            window_ref: retained,
        })
    }
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
