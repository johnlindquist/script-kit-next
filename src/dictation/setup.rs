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

/// Dictation Setup row: start a microphone test.
pub const DICTATION_SETUP_TEST_MIC: &str = "test-microphone";
/// Dictation Setup row: ask for Accessibility so dictation can type.
pub const DICTATION_SETUP_GRANT_ACCESSIBILITY: &str = "allow-pasting";
/// Dictation Setup row: transcribe the newest kept recording again.
pub const DICTATION_SETUP_RETRY_LAST: &str = "retry-last-recording";

/// Dictation Setup facts beyond model, microphone permission and hotkey.
#[derive(Debug, Clone, PartialEq)]
pub struct DictationSetupExtras {
    /// Script Kit can post ⌘V, so dictated words are typed in place.
    pub paste_ready: bool,
    /// The selected microphone is the built-in one and the lid is closed.
    pub lid_closed_on_builtin_microphone: bool,
    /// The last microphone test in this session.
    pub microphone_check: Option<crate::dictation::MicrophoneCheck>,
    /// A microphone test is recording now.
    pub microphone_check_running: bool,
    /// The newest kept recording: its length and when it was made ("2:05 PM").
    pub last_take: Option<(std::time::Duration, String)>,
    /// A retry of the newest recording is transcribing now.
    pub retry_running: bool,
}

impl Default for DictationSetupExtras {
    fn default() -> Self {
        Self {
            paste_ready: true,
            lid_closed_on_builtin_microphone: false,
            microphone_check: None,
            microphone_check_running: false,
            last_take: None,
            retry_running: false,
        }
    }
}

/// One Dictation Setup row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationSetupRow {
    pub name: String,
    pub value: &'static str,
    pub description: String,
}

/// The rows that say whether dictation will really work: a closed-lid
/// warning, the microphone test, and whether dictated words can be typed
/// into other apps (Accessibility) or will be copied instead.
pub fn dictation_setup_extra_rows(
    state: &DictationSetupState,
    extras: &DictationSetupExtras,
) -> Vec<DictationSetupRow> {
    let mut rows = Vec::new();
    if extras.lid_closed_on_builtin_microphone {
        rows.push(DictationSetupRow {
            name: "The lid is closed, so the built-in microphone can't hear you".to_string(),
            value: "microphone-lid-closed",
            description: "Open the lid or choose another microphone".to_string(),
        });
    }

    if matches!(
        state.microphone_status,
        DictationMicrophoneStatus::Ready { .. }
            | DictationMicrophoneStatus::SavedDeviceMissing { .. }
    ) {
        let (name, description) = if extras.microphone_check_running {
            (
                "Microphone test: listening…".to_string(),
                "Say something now".to_string(),
            )
        } else if let Some(check) = &extras.microphone_check {
            check.summary()
        } else {
            (
                "Test microphone".to_string(),
                format!(
                    "Records {} seconds and checks that sound comes through",
                    crate::dictation::MICROPHONE_CHECK_DURATION.as_secs()
                ),
            )
        };
        rows.push(DictationSetupRow {
            name,
            value: DICTATION_SETUP_TEST_MIC,
            description,
        });
    }

    if extras.retry_running {
        rows.push(DictationSetupRow {
            name: "Transcribing your last recording…".to_string(),
            value: DICTATION_SETUP_RETRY_LAST,
            description: "The words will be copied to the clipboard".to_string(),
        });
    } else if let Some((duration, when)) = &extras.last_take {
        rows.push(DictationSetupRow {
            name: format!(
                "Retry last recording ({} s, {when})",
                duration.as_secs_f32().round().max(1.0) as u64
            ),
            value: DICTATION_SETUP_RETRY_LAST,
            description: "Transcribes it again and copies the words to the clipboard".to_string(),
        });
    }

    rows.push(if extras.paste_ready {
        DictationSetupRow {
            name: "Typing into other apps: ready".to_string(),
            value: "paste-ready",
            description: "Dictated words go where you're working".to_string(),
        }
    } else {
        DictationSetupRow {
            name: "Allow typing into other apps".to_string(),
            value: DICTATION_SETUP_GRANT_ACCESSIBILITY,
            description:
                "Needs Accessibility. Without it, dictated words are copied to the clipboard instead"
                    .to_string(),
        }
    });
    rows
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

    fn ready_state() -> DictationSetupState {
        DictationSetupState {
            model_status: DictationModelStatus::Available,
            microphone_status: DictationMicrophoneStatus::Ready {
                name: "MacBook Microphone".to_string(),
                using_system_default: true,
            },
            hotkey_status: DictationHotkeyStatus::Disabled,
            ready: true,
        }
    }

    #[test]
    fn setup_offers_accessibility_when_dictation_would_only_copy() {
        let rows = dictation_setup_extra_rows(
            &ready_state(),
            &DictationSetupExtras {
                paste_ready: false,
                ..DictationSetupExtras::default()
            },
        );
        let grant = rows
            .iter()
            .find(|row| row.value == DICTATION_SETUP_GRANT_ACCESSIBILITY)
            .expect("missing Accessibility is offered in setup");
        assert!(grant.description.contains("copied to the clipboard"));

        let ready = dictation_setup_extra_rows(&ready_state(), &DictationSetupExtras::default());
        assert!(ready.iter().any(|row| row.value == "paste-ready"));
        assert!(!ready
            .iter()
            .any(|row| row.value == DICTATION_SETUP_GRANT_ACCESSIBILITY));
    }

    #[test]
    fn setup_offers_a_microphone_test_and_shows_its_result() {
        let rows = dictation_setup_extra_rows(&ready_state(), &DictationSetupExtras::default());
        assert!(rows
            .iter()
            .any(|row| row.value == DICTATION_SETUP_TEST_MIC && row.name == "Test microphone"));

        let rows = dictation_setup_extra_rows(
            &ready_state(),
            &DictationSetupExtras {
                microphone_check: Some(crate::dictation::assess_samples(&[0.0; 1600], 16_000)),
                lid_closed_on_builtin_microphone: true,
                ..DictationSetupExtras::default()
            },
        );
        assert_eq!(
            rows[0].value, "microphone-lid-closed",
            "the lid warning comes first"
        );
        assert!(rows
            .iter()
            .any(|row| row.name == "Microphone test: no sound came through"));
    }

    #[test]
    fn no_microphone_test_until_permission_is_granted() {
        let state = DictationSetupState {
            microphone_status: DictationMicrophoneStatus::PermissionNeeded(
                DictationMicrophonePermissionStatus::NotDetermined,
            ),
            ready: false,
            ..ready_state()
        };
        let rows = dictation_setup_extra_rows(&state, &DictationSetupExtras::default());
        assert!(!rows.iter().any(|row| row.value == DICTATION_SETUP_TEST_MIC));
    }

    #[test]
    fn setup_offers_to_retry_the_last_recording() {
        let none = dictation_setup_extra_rows(&ready_state(), &DictationSetupExtras::default());
        assert!(!none
            .iter()
            .any(|row| row.value == DICTATION_SETUP_RETRY_LAST));
        let rows = dictation_setup_extra_rows(
            &ready_state(),
            &DictationSetupExtras {
                last_take: Some((
                    std::time::Duration::from_millis(4_400),
                    "2:05 PM".to_string(),
                )),
                ..DictationSetupExtras::default()
            },
        );
        let retry = rows
            .iter()
            .find(|row| row.value == DICTATION_SETUP_RETRY_LAST)
            .expect("retry row");
        assert_eq!(retry.name, "Retry last recording (4 s, 2:05 PM)");
    }
}
