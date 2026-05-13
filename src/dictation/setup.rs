use crate::config::HotkeyConfig;
use crate::dictation::types::{DictationDeviceInfo, DictationModelStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationMicrophonePermissionStatus {
    Granted,
    NotDetermined,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationMicrophoneStatus {
    Ready {
        name: String,
        using_system_default: bool,
    },
    SavedDeviceMissing {
        fallback_name: Option<String>,
    },
    PermissionNeeded(DictationMicrophonePermissionStatus),
    NoDevices,
    EnumerationFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationHotkeyStatus {
    Ready(String),
    NotConfigured,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationSetupState {
    pub model_status: DictationModelStatus,
    pub microphone_status: DictationMicrophoneStatus,
    pub hotkey_status: DictationHotkeyStatus,
    pub ready: bool,
}

pub fn build_dictation_setup_state(
    model_status: DictationModelStatus,
    microphone_permission: DictationMicrophonePermissionStatus,
    devices: Result<Vec<DictationDeviceInfo>, String>,
    selected_device_id: Option<&str>,
    dictation_hotkey: Option<&HotkeyConfig>,
    dictation_hotkey_enabled: bool,
) -> DictationSetupState {
    let microphone_status =
        build_microphone_status(microphone_permission, devices, selected_device_id);
    let hotkey_status = build_hotkey_status(dictation_hotkey, dictation_hotkey_enabled);
    let ready = matches!(model_status, DictationModelStatus::Available)
        && matches!(microphone_status, DictationMicrophoneStatus::Ready { .. });

    DictationSetupState {
        model_status,
        microphone_status,
        hotkey_status,
        ready,
    }
}

fn build_microphone_status(
    permission: DictationMicrophonePermissionStatus,
    devices: Result<Vec<DictationDeviceInfo>, String>,
    selected_device_id: Option<&str>,
) -> DictationMicrophoneStatus {
    if matches!(
        permission,
        DictationMicrophonePermissionStatus::Denied
            | DictationMicrophonePermissionStatus::NotDetermined
    ) {
        return DictationMicrophoneStatus::PermissionNeeded(permission);
    }

    let devices = match devices {
        Ok(devices) => devices,
        Err(error) => return DictationMicrophoneStatus::EnumerationFailed(error),
    };

    if devices.is_empty() {
        return DictationMicrophoneStatus::NoDevices;
    }

    let resolution = crate::dictation::resolve_selected_input_device(&devices, selected_device_id);

    match resolution {
        Some(resolution) if resolution.fell_back => DictationMicrophoneStatus::SavedDeviceMissing {
            fallback_name: Some(resolution.device.name),
        },
        Some(resolution) => DictationMicrophoneStatus::Ready {
            name: resolution.device.name,
            using_system_default: selected_device_id.is_none(),
        },
        None => DictationMicrophoneStatus::NoDevices,
    }
}

fn build_hotkey_status(
    dictation_hotkey: Option<&HotkeyConfig>,
    dictation_hotkey_enabled: bool,
) -> DictationHotkeyStatus {
    if !dictation_hotkey_enabled {
        return DictationHotkeyStatus::Disabled;
    }

    match dictation_hotkey {
        Some(hotkey) => DictationHotkeyStatus::Ready(format_hotkey(hotkey)),
        None => DictationHotkeyStatus::NotConfigured,
    }
}

fn format_hotkey(hotkey: &HotkeyConfig) -> String {
    format!(
        "{}{}{}",
        hotkey.modifiers.join("+"),
        if hotkey.modifiers.is_empty() { "" } else { "+" },
        hotkey.key
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictation::types::{
        DictationDeviceId, DictationDeviceInfo, DictationDeviceTransport,
    };

    fn device(id: &str, name: &str, is_default: bool) -> DictationDeviceInfo {
        DictationDeviceInfo {
            id: DictationDeviceId(id.to_string()),
            name: name.to_string(),
            is_default,
            transport: DictationDeviceTransport::BuiltIn,
        }
    }

    #[test]
    fn setup_ready_does_not_require_hotkey() {
        let state = build_dictation_setup_state(
            DictationModelStatus::Available,
            DictationMicrophonePermissionStatus::Granted,
            Ok(vec![device("default", "MacBook Microphone", true)]),
            None,
            None,
            true,
        );

        assert!(state.ready);
        assert_eq!(state.hotkey_status, DictationHotkeyStatus::NotConfigured);
    }

    #[test]
    fn missing_model_blocks_readiness_even_with_microphone() {
        let state = build_dictation_setup_state(
            DictationModelStatus::NotDownloaded,
            DictationMicrophonePermissionStatus::Granted,
            Ok(vec![device("default", "MacBook Microphone", true)]),
            None,
            None,
            true,
        );

        assert!(!state.ready);
    }

    #[test]
    fn denied_microphone_permission_blocks_readiness_without_capture() {
        let state = build_dictation_setup_state(
            DictationModelStatus::Available,
            DictationMicrophonePermissionStatus::Denied,
            Ok(vec![device("default", "MacBook Microphone", true)]),
            None,
            None,
            true,
        );

        assert!(!state.ready);
        assert_eq!(
            state.microphone_status,
            DictationMicrophoneStatus::PermissionNeeded(
                DictationMicrophonePermissionStatus::Denied
            )
        );
    }

    #[test]
    fn stale_saved_microphone_is_called_out_without_exposing_id() {
        let state = build_dictation_setup_state(
            DictationModelStatus::Available,
            DictationMicrophonePermissionStatus::Granted,
            Ok(vec![device("default", "MacBook Microphone", true)]),
            Some("missing-usb-id"),
            None,
            true,
        );

        assert!(!state.ready);
        assert_eq!(
            state.microphone_status,
            DictationMicrophoneStatus::SavedDeviceMissing {
                fallback_name: Some("MacBook Microphone".to_string())
            }
        );
    }
}
