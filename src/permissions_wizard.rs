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
            Self::Accessibility => {
                "Snippets, pasting into other apps, selected text and window commands"
            }
            Self::ScreenRecording => "Screenshots and screen context for AI",
            Self::EventSynthesizing => "Pasting into other apps; Accessibility already covers this",
            Self::InputMonitoring => {
                "Watching typing for snippets; Accessibility already covers this"
            }
            Self::Microphone => "Dictation and voice input",
        }
    }

    /// What stops working while this permission is missing.
    pub fn missing_consequence(&self) -> &'static str {
        match self {
            Self::Accessibility => {
                "Until it's on, snippets can't expand, text is copied instead of pasted, and window commands are off"
            }
            Self::ScreenRecording => {
                "Until it's on, screenshots and screen context for AI don't work"
            }
            Self::EventSynthesizing | Self::InputMonitoring => {
                "Only needed when Accessibility is off"
            }
            Self::Microphone => "Until it's on, dictation can't hear you",
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

    /// How much Script Kit depends on this permission. Only Accessibility is
    /// required: most core features need it. Screen Recording is recommended
    /// and macOS asks for it when a screenshot feature first runs. The
    /// microphone matters only for dictation, and the other two are covered by
    /// Accessibility.
    pub fn requirement(&self) -> PermissionRequirement {
        match self {
            Self::Accessibility => PermissionRequirement::Required,
            Self::ScreenRecording => PermissionRequirement::Recommended,
            Self::EventSynthesizing => PermissionRequirement::Optional,
            Self::InputMonitoring => PermissionRequirement::Optional,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionRequirement {
    Required,
    Recommended,
    Optional,
}

/// Canonical actions exposed by the Permissions wizard across keyboard,
/// fallback hint-strip, and native-footer entry points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionsWizardAction {
    GrantSelected,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionsWizardActionSpec {
    pub action: PermissionsWizardAction,
    pub key: &'static str,
    pub label: &'static str,
}

pub struct PermissionsWizardActions;

impl PermissionsWizardActions {
    pub const GRANT: PermissionsWizardActionSpec = PermissionsWizardActionSpec {
        action: PermissionsWizardAction::GrantSelected,
        key: "↵",
        label: "Grant",
    };

    pub const DONE: PermissionsWizardActionSpec = PermissionsWizardActionSpec {
        action: PermissionsWizardAction::Done,
        key: "Esc",
        label: "Done",
    };

    pub const ALL: [PermissionsWizardActionSpec; 2] = [Self::GRANT, Self::DONE];

    pub fn for_key(key: &str, has_platform_modifier: bool) -> Option<PermissionsWizardAction> {
        if crate::ui_foundation::is_key_escape(key)
            || (has_platform_modifier && key.eq_ignore_ascii_case("w"))
        {
            Some(PermissionsWizardAction::Done)
        } else if crate::ui_foundation::is_key_enter(key) {
            Some(PermissionsWizardAction::GrantSelected)
        } else {
            None
        }
    }
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

/// Fire the native macOS grant flow for `kind` and return the resulting status.
///
/// Accessibility and the CG-based permissions show the one-time OS prompt
/// (or are silently denied if the user already refused once — callers should
/// fall back to [`open_permission_settings`] when the result is not
/// `Authorized`). The microphone prompt is asynchronous: when macOS hasn't
/// asked yet, this shows it and returns `NotDetermined` while it is up, and
/// the permission watcher picks up the answer. System Settings doesn't list
/// Script Kit under Microphone until that prompt has appeared once.
///
/// Every request is noted, so [`crate::permissions::grant_looks_stuck`] can
/// tell when a grant didn't take.
pub fn request_permission(
    kind: PermissionKind,
) -> crate::platform::permiso_detect::PermissionStatus {
    use crate::platform::permiso_detect;
    info!(permission = %kind, "Requesting permission via native prompt");
    crate::permissions::note_requested(kind);
    let status = match kind {
        PermissionKind::Accessibility => {
            if request_accessibility_permission() {
                permiso_detect::PermissionStatus::Authorized
            } else {
                permiso_detect::PermissionStatus::Denied
            }
        }
        PermissionKind::ScreenRecording => {
            crate::platform::permission_requests::request_screen_capture_access()
        }
        PermissionKind::InputMonitoring => {
            crate::platform::permission_requests::request_input_monitoring_access()
        }
        PermissionKind::EventSynthesizing => {
            crate::platform::permission_requests::request_event_synthesizing_access()
        }
        PermissionKind::Microphone => {
            let current = permiso_detect::microphone_authorized();
            if current == permiso_detect::PermissionStatus::NotDetermined
                && crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Device)
                    .is_ok()
            {
                crate::dictation::request_microphone_permission_nonblocking(|answer| {
                    info!(answer = ?answer, "Microphone prompt answered");
                    crate::permissions::refresh_all();
                });
            }
            current
        }
    };
    crate::permissions::record(kind, status);
    status
}

// ============================================================================
// Startup Intent
// ============================================================================

#[derive(Debug, Clone)]
pub enum PermissionStartupIntent {
    OpenFullWizard,
    ShowReminder {
        reminder: crate::permissions::features::StartupReminder,
    },
    None,
}

/// What to do about permissions at startup.
///
/// A first run without Accessibility opens the full setup list. Later runs
/// remind only about permissions the user's own setup depends on (see
/// [`crate::permissions::features::startup_reminder`]): Screen Recording
/// never nags at startup.
pub fn startup_intent(is_fresh_install: bool) -> PermissionStartupIntent {
    let facts = crate::permissions::live_facts();
    let status_of = |kind| crate::permissions::status(kind);
    decide_startup_intent(
        &status_of,
        &facts,
        is_fresh_install || !onboarding_state_exists(),
    )
}

fn decide_startup_intent(
    status_of: &dyn Fn(PermissionKind) -> crate::platform::permiso_detect::PermissionStatus,
    facts: &crate::permissions::LiveFacts,
    first_run: bool,
) -> PermissionStartupIntent {
    let accessibility_missing = !matches!(
        status_of(PermissionKind::Accessibility),
        crate::platform::permiso_detect::PermissionStatus::Authorized
            | crate::platform::permiso_detect::PermissionStatus::Unknown
    );
    if first_run && accessibility_missing {
        return PermissionStartupIntent::OpenFullWizard;
    }
    match crate::permissions::features::startup_reminder(status_of, facts) {
        Some(reminder) => PermissionStartupIntent::ShowReminder { reminder },
        None => PermissionStartupIntent::None,
    }
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
    let granted = crate::permissions::accessibility_granted();
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
            PermissionRequirement::Recommended,
            "Screen Recording is asked for when a screenshot feature first runs"
        );
        assert_eq!(
            PermissionKind::Microphone.requirement(),
            PermissionRequirement::Optional
        );
    }

    #[test]
    fn wizard_actions_keep_keyboard_and_surface_labels_in_one_contract() {
        assert_eq!(PermissionsWizardActions::GRANT.key, "↵");
        assert_eq!(PermissionsWizardActions::GRANT.label, "Grant");
        assert_eq!(PermissionsWizardActions::DONE.key, "Esc");
        assert_eq!(PermissionsWizardActions::DONE.label, "Done");
        assert_eq!(PermissionsWizardActions::ALL.len(), 2);
        assert_eq!(
            PermissionsWizardActions::for_key("enter", false),
            Some(PermissionsWizardAction::GrantSelected)
        );
        assert_eq!(
            PermissionsWizardActions::for_key("escape", false),
            Some(PermissionsWizardAction::Done)
        );
        assert_eq!(
            PermissionsWizardActions::for_key("w", true),
            Some(PermissionsWizardAction::Done)
        );
        assert_eq!(PermissionsWizardActions::for_key("down", false), None);
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
            "Required (Accessibility) is Authorized; non-required can be Denied"
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
    fn first_run_without_accessibility_opens_the_setup_list() {
        use crate::platform::permiso_detect::PermissionStatus as PS;
        let facts = crate::permissions::LiveFacts::default();
        let denied = |_kind: PermissionKind| PS::Denied;
        assert!(matches!(
            decide_startup_intent(&denied, &facts, true),
            PermissionStartupIntent::OpenFullWizard
        ));
        assert!(matches!(
            decide_startup_intent(&denied, &facts, false),
            PermissionStartupIntent::ShowReminder { .. }
        ));
    }

    #[test]
    fn missing_screen_recording_alone_never_opens_or_reminds_at_startup() {
        use crate::platform::permiso_detect::PermissionStatus as PS;
        let facts = crate::permissions::LiveFacts::default();
        let only_screen_missing = |kind: PermissionKind| match kind {
            PermissionKind::ScreenRecording => PS::Denied,
            _ => PS::Authorized,
        };
        assert!(matches!(
            decide_startup_intent(&only_screen_missing, &facts, true),
            PermissionStartupIntent::None
        ));
        assert!(matches!(
            decide_startup_intent(&only_screen_missing, &facts, false),
            PermissionStartupIntent::None
        ));
    }

    #[test]
    fn every_permission_says_what_stops_working() {
        for kind in PermissionKind::all() {
            assert!(!kind.missing_consequence().is_empty(), "{kind}");
        }
        assert!(PermissionKind::Accessibility
            .missing_consequence()
            .contains("snippets"));
        assert!(!PermissionKind::InputMonitoring
            .subtitle()
            .contains("keyboard shortcuts"));
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
