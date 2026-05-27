//! Permissions Wizard Module
//!
//! Unified permission model for Script Kit GPUI. Centralizes permission
//! checking across Accessibility, Screen Recording, Event Synthesizing,
//! Input Monitoring, and Microphone.

use tracing::{debug, info, instrument};

// ============================================================================
// Permission Kinds
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PermissionKind {
    Accessibility,
    ScreenRecording,
    EventSynthesizing,
    InputMonitoring,
    Microphone,
}

impl PermissionKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Accessibility => "Accessibility",
            Self::ScreenRecording => "Screen Recording",
            Self::EventSynthesizing => "Event Synthesizing",
            Self::InputMonitoring => "Input Monitoring",
            Self::Microphone => "Microphone",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Self::Accessibility => "Read selected text, control windows, run text expansion",
            Self::ScreenRecording => "Capture screenshots for AI context and visual tools",
            Self::EventSynthesizing => "Paste text and simulate keypresses in other apps",
            Self::InputMonitoring => "Global keyboard shortcuts and text expansion triggers",
            Self::Microphone => "Dictation and voice input",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Accessibility => "accessibility",
            Self::ScreenRecording => "monitor",
            Self::EventSynthesizing => "keyboard",
            Self::InputMonitoring => "ear",
            Self::Microphone => "mic",
        }
    }

    pub fn settings_url(&self) -> &'static str {
        match self {
            Self::Accessibility => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
            Self::ScreenRecording => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
            }
            Self::EventSynthesizing => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
            Self::InputMonitoring => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
            }
            Self::Microphone => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
            }
        }
    }

    pub fn requirement(&self) -> PermissionRequirement {
        match self {
            Self::Accessibility => PermissionRequirement::Required,
            Self::ScreenRecording => PermissionRequirement::Required,
            Self::EventSynthesizing => PermissionRequirement::Recommended,
            Self::InputMonitoring => PermissionRequirement::Recommended,
            Self::Microphone => PermissionRequirement::Optional,
        }
    }

    pub fn all() -> &'static [PermissionKind] {
        &[
            Self::Accessibility,
            Self::ScreenRecording,
            Self::EventSynthesizing,
            Self::InputMonitoring,
            Self::Microphone,
        ]
    }
}

impl std::fmt::Display for PermissionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionRequirement {
    Required,
    Recommended,
    Optional,
}

// ============================================================================
// Permission Card State
// ============================================================================

#[derive(Debug, Clone)]
pub struct PermissionCardState {
    pub kind: PermissionKind,
    pub status: crate::platform::permiso_detect::PermissionStatus,
}

// ============================================================================
// Permission Snapshot
// ============================================================================

#[derive(Debug, Clone)]
pub struct PermissionSnapshot {
    pub cards: Vec<PermissionCardState>,
}

impl PermissionSnapshot {
    pub fn current() -> Self {
        Self {
            cards: PermissionKind::all()
                .iter()
                .map(|&kind| PermissionCardState {
                    kind,
                    status: detect_permission(kind),
                })
                .collect(),
        }
    }

    pub fn all_required_granted(&self) -> bool {
        self.cards.iter().all(|card| {
            card.kind.requirement() != PermissionRequirement::Required
                || card.status == crate::platform::permiso_detect::PermissionStatus::Authorized
        })
    }

    pub fn missing_required(&self) -> Vec<PermissionKind> {
        self.cards
            .iter()
            .filter(|card| {
                card.kind.requirement() == PermissionRequirement::Required
                    && card.status != crate::platform::permiso_detect::PermissionStatus::Authorized
            })
            .map(|card| card.kind)
            .collect()
    }
}

pub fn detect_permission(
    kind: PermissionKind,
) -> crate::platform::permiso_detect::PermissionStatus {
    use crate::platform::permiso_detect;
    match kind {
        PermissionKind::Accessibility => permiso_detect::ax_is_trusted(),
        PermissionKind::ScreenRecording => permiso_detect::screen_capture_authorized(),
        PermissionKind::Microphone => permiso_detect::microphone_authorized(),
        PermissionKind::EventSynthesizing => permiso_detect::event_synthesizing_authorized(),
        PermissionKind::InputMonitoring => permiso_detect::input_monitoring_authorized(),
    }
}

// ============================================================================
// Startup Intent
// ============================================================================

#[derive(Debug, Clone)]
pub enum PermissionStartupIntent {
    OpenFullWizard,
    ShowReminder { missing: Vec<PermissionKind> },
    None,
}

pub fn startup_intent(is_fresh_install: bool) -> PermissionStartupIntent {
    let snapshot = PermissionSnapshot::current();
    let missing = snapshot.missing_required();

    if missing.is_empty() {
        return PermissionStartupIntent::None;
    }

    if is_fresh_install || !onboarding_state_exists() {
        return PermissionStartupIntent::OpenFullWizard;
    }

    PermissionStartupIntent::ShowReminder { missing }
}

fn onboarding_state_exists() -> bool {
    let path = crate::setup::get_kit_path().join("permission-onboarding.json");
    path.exists()
}

pub fn mark_onboarding_completed() {
    let path = crate::setup::get_kit_path().join("permission-onboarding.json");
    let state = serde_json::json!({
        "schemaVersion": 1,
        "completedAt": chrono::Utc::now().to_rfc3339(),
    });
    if let Ok(content) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(path, content);
    }
}

// ============================================================================
// Backward-compatible public API
// ============================================================================

/// Legacy alias — preserved for existing callers.
pub type PermissionType = PermissionKind;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PermissionInfo {
    pub permission_type: PermissionKind,
    pub granted: bool,
    pub description: String,
    pub instructions: String,
    pub features: Vec<String>,
}

impl PermissionInfo {
    fn accessibility(granted: bool) -> Self {
        Self {
            permission_type: PermissionKind::Accessibility,
            granted,
            description:
                "Accessibility permission allows Script Kit to monitor keyboard input \
                for text expansion, control windows, and get selected text from other applications."
                    .to_string(),
            instructions: "1. Open System Settings\n\
                 2. Go to Privacy & Security > Accessibility\n\
                 3. Click the + button\n\
                 4. Find and select Script Kit\n\
                 5. Enable the toggle next to Script Kit"
                .to_string(),
            features: PermissionKind::Accessibility
                .subtitle()
                .split(", ")
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionStatus {
    pub accessibility: PermissionInfo,
}

impl PermissionStatus {
    pub fn all_granted(&self) -> bool {
        self.accessibility.granted
    }

    pub fn missing_permissions(&self) -> Vec<&PermissionInfo> {
        let mut missing = Vec::new();
        if !self.accessibility.granted {
            missing.push(&self.accessibility);
        }
        missing
    }
}

#[instrument]
pub fn check_all_permissions() -> PermissionStatus {
    let accessibility_granted = check_accessibility_permission();
    let status = PermissionStatus {
        accessibility: PermissionInfo::accessibility(accessibility_granted),
    };
    info!(
        all_granted = status.all_granted(),
        accessibility = accessibility_granted,
        "Checked all permissions"
    );
    status
}

#[instrument]
pub fn check_accessibility_permission() -> bool {
    let granted = macos_accessibility_client::accessibility::application_is_trusted();
    debug!(granted, "Checked accessibility permission");
    granted
}

#[instrument]
pub fn request_accessibility_permission() -> bool {
    info!("Requesting accessibility permission");
    let granted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    info!(granted, "Accessibility permission request completed");
    granted
}

pub fn open_accessibility_settings() -> std::io::Result<()> {
    info!("Opening accessibility settings");
    std::process::Command::new("open")
        .arg(PermissionKind::Accessibility.settings_url())
        .spawn()?;
    Ok(())
}

pub fn open_permission_settings(kind: PermissionKind) -> std::io::Result<()> {
    info!(permission = %kind, "Opening permission settings");
    std::process::Command::new("open")
        .arg(kind.settings_url())
        .spawn()?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_kind_name() {
        assert_eq!(PermissionKind::Accessibility.name(), "Accessibility");
        assert_eq!(PermissionKind::ScreenRecording.name(), "Screen Recording");
    }

    #[test]
    fn test_permission_kind_display() {
        assert_eq!(
            format!("{}", PermissionKind::Accessibility),
            "Accessibility"
        );
    }

    #[test]
    fn test_permission_kind_all() {
        assert_eq!(PermissionKind::all().len(), 5);
    }

    #[test]
    fn test_permission_requirements() {
        assert_eq!(
            PermissionKind::Accessibility.requirement(),
            PermissionRequirement::Required
        );
        assert_eq!(
            PermissionKind::ScreenRecording.requirement(),
            PermissionRequirement::Required
        );
        assert_eq!(
            PermissionKind::Microphone.requirement(),
            PermissionRequirement::Optional
        );
    }

    #[test]
    fn test_settings_urls() {
        assert!(PermissionKind::Accessibility
            .settings_url()
            .contains("Privacy_Accessibility"));
        assert!(PermissionKind::ScreenRecording
            .settings_url()
            .contains("Privacy_ScreenCapture"));
        assert!(PermissionKind::InputMonitoring
            .settings_url()
            .contains("Privacy_ListenEvent"));
        assert!(PermissionKind::Microphone
            .settings_url()
            .contains("Privacy_Microphone"));
    }

    #[test]
    fn test_snapshot_current_does_not_panic() {
        let snapshot = PermissionSnapshot::current();
        assert_eq!(snapshot.cards.len(), 5);
    }

    #[test]
    fn test_snapshot_all_required_granted_logic() {
        use crate::platform::permiso_detect::PermissionStatus as PS;
        let snapshot = PermissionSnapshot {
            cards: vec![
                PermissionCardState {
                    kind: PermissionKind::Accessibility,
                    status: PS::Authorized,
                },
                PermissionCardState {
                    kind: PermissionKind::ScreenRecording,
                    status: PS::Authorized,
                },
                PermissionCardState {
                    kind: PermissionKind::EventSynthesizing,
                    status: PS::Denied,
                },
                PermissionCardState {
                    kind: PermissionKind::InputMonitoring,
                    status: PS::Denied,
                },
                PermissionCardState {
                    kind: PermissionKind::Microphone,
                    status: PS::NotDetermined,
                },
            ],
        };
        assert!(
            snapshot.all_required_granted(),
            "Required (Accessibility, ScreenRecording) are Authorized; non-required can be Denied"
        );
        assert!(snapshot.missing_required().is_empty());
    }

    #[test]
    fn test_snapshot_missing_required() {
        use crate::platform::permiso_detect::PermissionStatus as PS;
        let snapshot = PermissionSnapshot {
            cards: vec![
                PermissionCardState {
                    kind: PermissionKind::Accessibility,
                    status: PS::Denied,
                },
                PermissionCardState {
                    kind: PermissionKind::ScreenRecording,
                    status: PS::Authorized,
                },
            ],
        };
        let missing = snapshot.missing_required();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], PermissionKind::Accessibility);
    }

    #[test]
    fn test_backward_compat_check_all_permissions() {
        let status = check_all_permissions();
        assert_eq!(
            status.accessibility.permission_type,
            PermissionKind::Accessibility
        );
    }
}
