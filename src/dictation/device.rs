use crate::dictation::types::{DictationDeviceId, DictationDeviceInfo};
#[cfg(target_os = "macos")]
use anyhow::Context;
use anyhow::Result;

// ---------------------------------------------------------------------------
// Device selection types
// ---------------------------------------------------------------------------

/// Action produced by the microphone picker UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationDeviceSelectionAction {
    UseSystemDefault,
    UseDevice(DictationDeviceId),
}

/// A single row in the microphone picker list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationDeviceMenuItem {
    pub title: String,
    pub subtitle: String,
    pub action: DictationDeviceSelectionAction,
    pub is_selected: bool,
}

// ---------------------------------------------------------------------------
// Pure selection helpers (no I/O — easy to test)
// ---------------------------------------------------------------------------

/// Resolve a microphone from the device list using a saved preference.
///
/// Returns the device matching `selected_device_id` when present, otherwise
/// falls back to the system default (the device with `is_default == true`).
pub fn resolve_selected_input_device(
    devices: &[DictationDeviceInfo],
    selected_device_id: Option<&str>,
) -> Option<DictationDeviceInfo> {
    if let Some(saved) = selected_device_id {
        if let Some(device) = devices.iter().find(|d| d.id.0 == saved) {
            return Some(device.clone());
        }
    }
    devices.iter().find(|d| d.is_default).cloned()
}

/// Build the full menu item list for the microphone picker.
///
/// Always includes a "System Default" row at the top, followed by one row per
/// enumerated device.  Exactly one row will have `is_selected == true`.
pub fn build_device_menu_items(
    devices: &[DictationDeviceInfo],
    selected_device_id: Option<&str>,
) -> Vec<DictationDeviceMenuItem> {
    let saved_exists = selected_device_id
        .map(|saved| devices.iter().any(|d| d.id.0 == saved))
        .unwrap_or(false);

    let mut items = Vec::with_capacity(devices.len() + 1);

    items.push(DictationDeviceMenuItem {
        title: "System Default".to_string(),
        subtitle: devices
            .iter()
            .find(|d| d.is_default)
            .map(|d| format!("Use macOS default microphone ({})", d.name))
            .unwrap_or_else(|| "Use macOS default microphone".to_string()),
        action: DictationDeviceSelectionAction::UseSystemDefault,
        is_selected: selected_device_id.is_none() || !saved_exists,
    });

    for device in devices {
        items.push(DictationDeviceMenuItem {
            title: if device.is_default {
                format!("{} \u{00b7} default", device.name)
            } else {
                device.name.clone()
            },
            subtitle: if device.is_default {
                "Currently the macOS default input".to_string()
            } else {
                "Use this microphone for dictation".to_string()
            },
            action: DictationDeviceSelectionAction::UseDevice(device.id.clone()),
            is_selected: selected_device_id == Some(device.id.0.as_str()),
        });
    }

    items
}

// ---------------------------------------------------------------------------
// I/O wrappers (thin shells over the pure helpers + persistence)
// ---------------------------------------------------------------------------

/// Enumerate input devices and build a ready-to-render menu item list.
pub fn list_input_device_menu_items(
    selected_device_id: Option<&str>,
) -> Result<Vec<DictationDeviceMenuItem>> {
    let devices = list_input_devices().context("failed to enumerate microphones")?;
    Ok(build_device_menu_items(&devices, selected_device_id))
}

/// Persist a picker selection to user preferences.
pub fn apply_device_selection(action: &DictationDeviceSelectionAction) -> Result<()> {
    match action {
        DictationDeviceSelectionAction::UseSystemDefault => save_dictation_device_id(None),
        DictationDeviceSelectionAction::UseDevice(device_id) => {
            save_dictation_device_id(Some(device_id.0.as_str()))
        }
    }
}

/// Persist the selected microphone device ID to user preferences.
///
/// Pass `None` to clear the preference and revert to the system default.
pub fn save_dictation_device_id(device_id: Option<&str>) -> Result<()> {
    let mut prefs = crate::config::load_user_preferences();
    prefs.dictation.selected_device_id = device_id.map(str::to_owned);
    crate::config::save_user_preferences(&prefs)?;
    tracing::info!(
        category = "DICTATION",
        device_id = ?device_id,
        "Saved microphone preference"
    );
    Ok(())
}

#[cfg(target_os = "macos")]
use objc::runtime::Object;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
pub fn list_input_devices() -> Result<Vec<DictationDeviceInfo>> {
    unsafe {
        // SAFETY: AVFoundation device enumeration is performed via Objective-C messaging
        // with selectors documented for AVCaptureDevice. All returned objects are treated
        // as borrowed/autoreleased and not retained by Rust.
        let default_device: *mut Object =
            msg_send![class!(AVCaptureDevice), defaultDeviceWithMediaType: av_media_type_audio()];
        let default_id = nsstring_to_string(if default_device.is_null() {
            std::ptr::null_mut()
        } else {
            let value: *mut Object = msg_send![default_device, uniqueID];
            value
        });

        let devices: *mut Object =
            msg_send![class!(AVCaptureDevice), devicesWithMediaType: av_media_type_audio()];
        if devices.is_null() {
            return Ok(Vec::new());
        }

        let count: usize = msg_send![devices, count];
        let mut items = Vec::with_capacity(count);

        for index in 0..count {
            let device: *mut Object = msg_send![devices, objectAtIndex: index];
            if device.is_null() {
                continue;
            }

            let id_obj: *mut Object = msg_send![device, uniqueID];
            let name_obj: *mut Object = msg_send![device, localizedName];

            let id = match nsstring_to_string(id_obj) {
                Some(value) => value,
                None => continue,
            };

            let name = nsstring_to_string(name_obj)
                .with_context(|| format!("missing localizedName for audio input device {id}"))?;

            items.push(DictationDeviceInfo {
                is_default: default_id.as_deref() == Some(id.as_str()),
                id: DictationDeviceId(id),
                name,
            });
        }

        Ok(items)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn list_input_devices() -> Result<Vec<DictationDeviceInfo>> {
    Ok(Vec::new())
}

#[cfg(target_os = "macos")]
pub fn default_input_device() -> Result<Option<DictationDeviceInfo>> {
    let devices = list_input_devices()?;
    Ok(devices.into_iter().find(|device| device.is_default))
}

#[cfg(not(target_os = "macos"))]
pub fn default_input_device() -> Result<Option<DictationDeviceInfo>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
fn av_media_type_audio() -> *mut Object {
    unsafe {
        // SAFETY: The UTF-8 string literal is NUL-terminated and valid for NSString construction.
        let media_type: *mut Object =
            msg_send![class!(NSString), stringWithUTF8String: c"soun".as_ptr()];
        media_type
    }
}

#[cfg(target_os = "macos")]
unsafe fn nsstring_to_string(value: *mut Object) -> Option<String> {
    if value.is_null() {
        return None;
    }

    // SAFETY: `value` is expected to be an NSString-compatible object and UTF8String
    // returns a borrowed NUL-terminated buffer for the lifetime of the NSString.
    let utf8: *const i8 = msg_send![value, UTF8String];
    if utf8.is_null() {
        return None;
    }

    Some(
        std::ffi::CStr::from_ptr(utf8)
            .to_string_lossy()
            .into_owned(),
    )
}
