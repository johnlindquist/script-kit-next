use anyhow::{bail, Context, Result};
use bitflags::bitflags;
use std::ffi::c_void;
use std::time::{Duration, Instant};
use tracing::{debug, instrument, warn};
// Import shared FFI from window_control where possible
use crate::window_control::has_accessibility_permission;
// ============================================================================
// CoreFoundation FFI bindings
// ============================================================================

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}
// ============================================================================
// ApplicationServices (Accessibility) FFI bindings
// ============================================================================

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
}
// AXError codes
const kAXErrorSuccess: i32 = 0;
const kAXErrorAPIDisabled: i32 = -25211;
const kAXErrorNoValue: i32 = -25212;
type AXUIElementRef = *const c_void;
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
    fn CFNumberGetTypeID() -> u64;
    fn CFBooleanGetValue(boolean: CFTypeRef) -> bool;
    fn CFBooleanGetTypeID() -> u64;
}
const kCFStringEncodingUTF8: u32 = 0x08000100;
const kCFNumberSInt32Type: i32 = 3;
const kCFNumberSInt64Type: i32 = 4;
// ============================================================================
// Menu-specific AX attribute constants
// ============================================================================

/// AX attribute names for menu bar traversal
const AX_MENU_BAR: &str = "AXMenuBar";
const AX_CHILDREN: &str = "AXChildren";
const AX_TITLE: &str = "AXTitle";
const AX_ROLE: &str = "AXRole";
const AX_ENABLED: &str = "AXEnabled";
const AX_MENU_ITEM_CMD_CHAR: &str = "AXMenuItemCmdChar";
const AX_MENU_ITEM_CMD_MODIFIERS: &str = "AXMenuItemCmdModifiers";
/// AX role values
const AX_ROLE_MENU_BAR_ITEM: &str = "AXMenuBarItem";
const AX_ROLE_MENU_ITEM: &str = "AXMenuItem";
const AX_ROLE_MENU: &str = "AXMenu";
/// macOS modifier key masks (from Carbon HIToolbox)
const CMD_KEY_MASK: u32 = 256;
const SHIFT_KEY_MASK: u32 = 512;
const OPTION_KEY_MASK: u32 = 2048;
const CONTROL_KEY_MASK: u32 = 4096;
/// Maximum depth for menu traversal (to prevent infinite recursion)
const MAX_MENU_DEPTH: usize = 3;
/// Separator title marker
const SEPARATOR_TITLE: &str = "---";
// ============================================================================
// Public Types
// ============================================================================

bitflags! {
    /// Modifier key flags for keyboard shortcuts
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModifierFlags: u32 {
        /// Command key (Cmd/⌘)
        const COMMAND = CMD_KEY_MASK;
        /// Shift key (⇧)
        const SHIFT = SHIFT_KEY_MASK;
        /// Option key (Alt/⌥)
        const OPTION = OPTION_KEY_MASK;
        /// Control key (⌃)
        const CONTROL = CONTROL_KEY_MASK;
    }
}
/// A keyboard shortcut with key and modifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardShortcut {
    /// The key character (e.g., "S", "N", "Q")
    pub key: String,
    /// Modifier keys required
    pub modifiers: ModifierFlags,
}
impl KeyboardShortcut {
    /// Create a new keyboard shortcut
    pub fn new(key: String, modifiers: ModifierFlags) -> Self {
        Self { key, modifiers }
    }

    /// Create a keyboard shortcut from AX accessibility values
    ///
    /// # Arguments
    /// * `cmd_char` - The AXMenuItemCmdChar value (the key)
    /// * `cmd_modifiers` - The AXMenuItemCmdModifiers value (bitmask)
    pub fn from_ax_values(cmd_char: &str, cmd_modifiers: u32) -> Self {
        Self {
            key: cmd_char.to_string(),
            modifiers: ModifierFlags::from_bits_truncate(cmd_modifiers),
        }
    }

    /// Convert to a human-readable display string (e.g., "⌘⇧S")
    pub fn to_display_string(&self) -> String {
        let mut result = String::new();

        // Order: Control, Option, Shift, Command (standard macOS order)
        if self.modifiers.contains(ModifierFlags::CONTROL) {
            result.push('⌃');
        }
        if self.modifiers.contains(ModifierFlags::OPTION) {
            result.push('⌥');
        }
        if self.modifiers.contains(ModifierFlags::SHIFT) {
            result.push('⇧');
        }
        if self.modifiers.contains(ModifierFlags::COMMAND) {
            result.push('⌘');
        }

        result.push_str(&self.key);
        result
    }
}
/// A menu bar item with its children and metadata
#[derive(Debug, Clone)]
pub struct MenuBarItem {
    /// The display title of the menu item
    pub title: String,
    /// Whether the menu item is enabled (clickable)
    pub enabled: bool,
    /// Keyboard shortcut, if any
    pub shortcut: Option<KeyboardShortcut>,
    /// Child menu items (for submenus)
    pub children: Vec<MenuBarItem>,
    /// Path of indices to reach this element in the AX hierarchy
    /// Used for executing menu actions later
    pub ax_element_path: Vec<usize>,
}
impl MenuBarItem {
    /// Create a separator menu item
    pub fn separator(path: Vec<usize>) -> Self {
        Self {
            title: SEPARATOR_TITLE.to_string(),
            enabled: false,
            shortcut: None,
            children: vec![],
            ax_element_path: path,
        }
    }

    /// Check if this item is a separator
    pub fn is_separator(&self) -> bool {
        self.title == SEPARATOR_TITLE
    }
}
/// Cache for scanned menu data
#[derive(Debug)]
pub struct MenuCache {
    /// The bundle identifier of the application
    pub bundle_id: String,
    /// Serialized menu JSON (for SDK transmission)
    pub menu_json: Option<String>,
    /// When the menu was last scanned
    pub last_scanned: Option<Instant>,
}
impl MenuCache {
    /// Create a new empty cache for an application
    pub fn new(bundle_id: String) -> Self {
        Self {
            bundle_id,
            menu_json: None,
            last_scanned: None,
        }
    }

    /// Check if the cache is stale
    pub fn is_stale(&self, max_age: Duration) -> bool {
        match self.last_scanned {
            None => true,
            Some(scanned) => scanned.elapsed() > max_age,
        }
    }
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
/// Get a string attribute from an AXUIElement
fn get_ax_string_attribute(element: AXUIElementRef, attribute: &str) -> Option<String> {
    match get_ax_attribute(element, attribute) {
        Ok(value) => {
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
/// Get a boolean attribute from an AXUIElement
fn get_ax_bool_attribute(element: AXUIElementRef, attribute: &str) -> Option<bool> {
    match get_ax_attribute(element, attribute) {
        Ok(value) => {
            let type_id = unsafe { CFGetTypeID(value) };
            let bool_type_id = unsafe { CFBooleanGetTypeID() };

            let result = if type_id == bool_type_id {
                Some(unsafe { CFBooleanGetValue(value) })
            } else {
                None
            };

            cf_release(value);
            result
        }
        Err(_) => None,
    }
}
/// Get a number attribute from an AXUIElement as i32
fn get_ax_number_attribute(element: AXUIElementRef, attribute: &str) -> Option<i32> {
    match get_ax_attribute(element, attribute) {
        Ok(value) => {
            let type_id = unsafe { CFGetTypeID(value) };
            let number_type_id = unsafe { CFNumberGetTypeID() };

            let result = if type_id == number_type_id {
                let mut num_value: i32 = 0;
                if unsafe {
                    CFNumberGetValue(
                        value,
                        kCFNumberSInt32Type,
                        &mut num_value as *mut _ as *mut c_void,
                    )
                } {
                    Some(num_value)
                } else {
                    // Try 64-bit
                    let mut num_value_64: i64 = 0;
                    if unsafe {
                        CFNumberGetValue(
                            value,
                            kCFNumberSInt64Type,
                            &mut num_value_64 as *mut _ as *mut c_void,
                        )
                    } {
                        Some(num_value_64 as i32)
                    } else {
                        None
                    }
                }
            } else {
                None
            };

            cf_release(value);
            result
        }
        Err(_) => None,
    }
}
/// Get the children array from an AXUIElement
fn get_ax_children(element: AXUIElementRef) -> Result<(CFArrayRef, i64)> {
    let children_value = get_ax_attribute(element, AX_CHILDREN)?;
    let count = unsafe { CFArrayGetCount(children_value as CFArrayRef) };
    Ok((children_value as CFArrayRef, count))
}
/// Check if an element is a separator
fn is_menu_separator(element: AXUIElementRef) -> bool {
    // Separators have empty titles or specific role
    let title = get_ax_string_attribute(element, AX_TITLE);
    let role = get_ax_string_attribute(element, AX_ROLE);

    // Check for separator role or empty/whitespace title with disabled state
    if let Some(role_str) = role {
        // Some apps use a specific separator role
        if role_str.contains("Separator") {
            return true;
        }
    }

    // Also check for empty title + disabled
    if let Some(title_str) = title {
        if title_str.is_empty() || title_str.chars().all(|c| c.is_whitespace()) {
            return true;
        }
    } else {
        // No title at all - likely separator
        return true;
    }

    false
}
/// Parse a single menu item from an AXUIElement
fn parse_menu_item(element: AXUIElementRef, path: Vec<usize>, depth: usize) -> Option<MenuBarItem> {
    // Check for separator first
    if is_menu_separator(element) {
        return Some(MenuBarItem::separator(path));
    }

    // Get title
    let title = get_ax_string_attribute(element, AX_TITLE).unwrap_or_default();

    // Get enabled state
    let enabled = get_ax_bool_attribute(element, AX_ENABLED).unwrap_or(true);

    // Get keyboard shortcut
    let shortcut = {
        let cmd_char = get_ax_string_attribute(element, AX_MENU_ITEM_CMD_CHAR);
        let cmd_modifiers = get_ax_number_attribute(element, AX_MENU_ITEM_CMD_MODIFIERS);

        match (cmd_char, cmd_modifiers) {
            (Some(key), Some(mods)) if !key.is_empty() => {
                Some(KeyboardShortcut::from_ax_values(&key, mods as u32))
            }
            (Some(key), None) if !key.is_empty() => {
                // Has key but no modifiers - unusual but possible
                Some(KeyboardShortcut::new(key, ModifierFlags::empty()))
            }
            _ => None,
        }
    };

    // Get children (submenu items) if not at max depth
    let children = if depth < MAX_MENU_DEPTH {
        parse_submenu_children(element, &path, depth)
    } else {
        vec![]
    };

    Some(MenuBarItem {
        title,
        enabled,
        shortcut,
        children,
        ax_element_path: path,
    })
}
