use anyhow::{bail, Result};
use core_graphics::geometry::{CGPoint, CGSize};
use std::ffi::c_void;

use super::cf::*;
use super::ffi::*;

/// Get an attribute value from an AXUIElement
pub(super) fn get_ax_attribute(element: AXUIElementRef, attribute: &str) -> Result<CFTypeRef> {
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
pub(super) fn set_ax_attribute(
    element: AXUIElementRef,
    attribute: &str,
    value: CFTypeRef,
) -> Result<()> {
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
pub(super) fn perform_ax_action(element: AXUIElementRef, action: &str) -> Result<()> {
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
pub(super) fn get_window_position(window: AXUIElementRef) -> Result<(i32, i32)> {
    let value = get_ax_attribute(window, "AXPosition")?;

    let mut point = CGPoint::new(0.0, 0.0);
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
pub(super) fn get_window_size(window: AXUIElementRef) -> Result<(u32, u32)> {
    let value = get_ax_attribute(window, "AXSize")?;

    let mut size = CGSize::new(0.0, 0.0);
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
pub(super) fn set_window_position(window: AXUIElementRef, x: i32, y: i32) -> Result<()> {
    let point = CGPoint::new(x as f64, y as f64);
    let value = unsafe { AXValueCreate(kAXValueTypeCGPoint, &point as *const _ as *const c_void) };

    if value.is_null() {
        bail!("Failed to create AXValue for position");
    }

    let result = set_ax_attribute(window, "AXPosition", value);
    cf_release(value);
    result
}

/// Set the size of a window
pub(super) fn set_window_size(window: AXUIElementRef, width: u32, height: u32) -> Result<()> {
    let size = CGSize::new(width as f64, height as f64);
    let value = unsafe { AXValueCreate(kAXValueTypeCGSize, &size as *const _ as *const c_void) };

    if value.is_null() {
        bail!("Failed to create AXValue for size");
    }

    let result = set_ax_attribute(window, "AXSize", value);
    cf_release(value);
    result
}

/// Get the string value of a window attribute
pub(super) fn get_window_string_attribute(
    window: AXUIElementRef,
    attribute: &str,
) -> Option<String> {
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
