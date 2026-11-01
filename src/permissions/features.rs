//! Which features can run, from permission statuses and live facts.
//!
//! A permission matters to the user only through the features it affects, so
//! everything user-facing (reminders, the setup list, agent state) is phrased
//! per feature: what doesn't work, what Script Kit does instead, and what fixes
//! it. Pure functions; the live inputs come from [`super::live_facts`].

use super::{PermissionKind, PermissionStatus};
use serde::Serialize;

/// The live state of text expansion (snippet keywords).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum TextExpansionState {
    /// Startup hasn't reached text expansion yet.
    #[default]
    NotStarted,
    /// Waiting for Accessibility; starts on its own once granted.
    WaitingForAccessibility,
    /// Watching typing.
    Running { triggers: usize },
    /// Watching, but macOS hides typing from every app while a password field
    /// (or an app that asked for it) has Secure Input on.
    SecureInput { owner: Option<String> },
    /// macOS refused to let Script Kit watch typing.
    Failed { reason: String },
    /// Not available on this platform.
    Unsupported,
}

impl TextExpansionState {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotStarted => "notStarted",
            Self::WaitingForAccessibility => "waitingForAccessibility",
            Self::Running { .. } => "running",
            Self::SecureInput { .. } => "secureInput",
            Self::Failed { .. } => "failed",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Feature {
    TextExpansion,
    Dictation,
    PasteIntoApps,
    SelectedText,
    WindowControl,
    ScreenCapture,
}

impl Feature {
    pub fn label(self) -> &'static str {
        match self {
            Self::TextExpansion => "Snippets",
            Self::Dictation => "Dictation",
            Self::PasteIntoApps => "Paste into other apps",
            Self::SelectedText => "Selected text",
            Self::WindowControl => "Window commands",
            Self::ScreenCapture => "Screenshots and screen context",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Readiness {
    Ready,
    /// Works in a reduced form (for example: copies instead of pasting).
    Degraded,
    /// Can't run until something is fixed.
    Blocked,
    /// macOS asks for the permission the first time the feature runs.
    AskOnUse,
    /// Not known yet (starting, or the status can't be read here).
    Unknown,
}

/// What fixes a feature that isn't ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FeatureFix {
    Grant {
        #[serde(serialize_with = "serialize_kind")]
        permission: PermissionKind,
    },
    RestartScriptKit,
}

fn serialize_kind<S: serde::Serializer>(
    kind: &PermissionKind,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(super::kind_code(*kind))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureReadiness {
    pub feature: Feature,
    pub label: &'static str,
    pub readiness: Readiness,
    /// Permission codes this feature is missing.
    pub missing: Vec<&'static str>,
    /// What the user would be told; `None` when ready.
    pub message: Option<String>,
    pub fix: Option<FeatureFix>,
}

/// Facts that aren't permissions.
#[derive(Debug, Clone, Default)]
pub struct LiveFacts {
    pub text_expansion: TextExpansionState,
    pub keyword_trigger_count: usize,
    pub dictation_hotkey_enabled: bool,
}

const RESTART_HINT: &str =
    " If Script Kit is already switched on in System Settings, restart Script Kit.";

fn granted(status: PermissionStatus) -> bool {
    status == PermissionStatus::Authorized
}

fn unknown(status: PermissionStatus) -> bool {
    status == PermissionStatus::Unknown
}

/// Either native grant can authorize posting a keystroke. An unreadable observation
/// proves neither authorization nor denial. It must never authorize a real dispatch.
pub(super) fn post_access(ax: PermissionStatus, post: PermissionStatus) -> Option<bool> {
    if granted(ax) || granted(post) {
        Some(true)
    } else if unknown(ax) || unknown(post) {
        None
    } else {
        Some(false)
    }
}

fn entry(
    feature: Feature,
    readiness: Readiness,
    missing: &[PermissionKind],
    message: Option<String>,
    fix: Option<FeatureFix>,
) -> FeatureReadiness {
    FeatureReadiness {
        feature,
        label: feature.label(),
        readiness,
        missing: missing.iter().map(|&kind| super::kind_code(kind)).collect(),
        message,
        fix,
    }
}

fn grant(permission: PermissionKind) -> Option<FeatureFix> {
    Some(FeatureFix::Grant { permission })
}

/// Readiness of every feature.
pub fn readiness(
    status_of: &dyn Fn(PermissionKind) -> PermissionStatus,
    facts: &LiveFacts,
) -> Vec<FeatureReadiness> {
    let ax = status_of(PermissionKind::Accessibility);
    let post = status_of(PermissionKind::EventSynthesizing);
    let mic = status_of(PermissionKind::Microphone);
    let screen = status_of(PermissionKind::ScreenRecording);
    let can_paste = post_access(ax, post);
    let ax_missing = !granted(ax) && !unknown(ax);
    let accessibility = [PermissionKind::Accessibility];

    let text_expansion = if ax_missing {
        entry(
            Feature::TextExpansion,
            Readiness::Blocked,
            &accessibility,
            Some("Snippets can't expand until Script Kit has Accessibility.".to_string()),
            grant(PermissionKind::Accessibility),
        )
    } else {
        match &facts.text_expansion {
            TextExpansionState::Running { .. } => {
                entry(Feature::TextExpansion, Readiness::Ready, &[], None, None)
            }
            TextExpansionState::SecureInput { owner } => entry(
                Feature::TextExpansion,
                Readiness::Degraded,
                &[],
                Some(format!(
                    "Snippets are paused while {} has secure typing on (usually a password field).",
                    owner.as_deref().unwrap_or("another app")
                )),
                None,
            ),
            TextExpansionState::Failed { reason } => entry(
                Feature::TextExpansion,
                Readiness::Blocked,
                &[],
                Some(format!("Snippets stopped: {reason}.{RESTART_HINT}")),
                Some(FeatureFix::RestartScriptKit),
            ),
            TextExpansionState::Unsupported => entry(
                Feature::TextExpansion,
                Readiness::Blocked,
                &[],
                Some("Snippets need macOS.".to_string()),
                None,
            ),
            TextExpansionState::NotStarted | TextExpansionState::WaitingForAccessibility => entry(
                Feature::TextExpansion,
                Readiness::Unknown,
                &[],
                Some("Snippets are starting.".to_string()),
                None,
            ),
        }
    };

    let dictation = match mic {
        PermissionStatus::Denied => entry(
            Feature::Dictation,
            Readiness::Blocked,
            &[PermissionKind::Microphone],
            Some("Dictation can't hear you until Script Kit may use the microphone.".to_string()),
            grant(PermissionKind::Microphone),
        ),
        PermissionStatus::NotDetermined => entry(
            Feature::Dictation,
            Readiness::AskOnUse,
            &[PermissionKind::Microphone],
            Some("macOS asks for the microphone the first time you dictate.".to_string()),
            grant(PermissionKind::Microphone),
        ),
        PermissionStatus::Unknown => entry(Feature::Dictation, Readiness::Unknown, &[], Some("Kit couldn't check microphone access.".into()), None),
        PermissionStatus::Authorized if unknown(ax) => entry(Feature::Dictation, Readiness::Unknown, &[], Some("Kit couldn't verify access to an external cursor.".into()), None),
        PermissionStatus::Authorized if !granted(ax) => entry(
            Feature::Dictation,
            Readiness::Degraded,
            &accessibility,
            Some(
                "Dictation works in Kit. To type into another app, Kit needs Accessibility to verify the original field. If delivery is withheld, review the text in Kit."
                    .to_string(),
            ),
            grant(PermissionKind::Accessibility),
        ),
        PermissionStatus::Authorized => entry(Feature::Dictation, Readiness::Ready, &[], None, None),
    };

    let paste = match can_paste {
        Some(true) => entry(Feature::PasteIntoApps, Readiness::Ready, &[], None, None),
        None => entry(Feature::PasteIntoApps, Readiness::Unknown, &[], Some("Kit couldn't verify permission to paste into other apps.".into()), None),
        Some(false) => entry(
            Feature::PasteIntoApps,
            Readiness::Degraded,
            &accessibility,
            Some(
                "Script Kit copies text to the clipboard instead of pasting it into other apps until it has Accessibility."
                    .to_string(),
            ),
            grant(PermissionKind::Accessibility),
        ),
    };

    let needs_accessibility = |feature: Feature, message: &str| {
        if ax_missing {
            entry(
                feature,
                Readiness::Blocked,
                &accessibility,
                Some(message.to_string()),
                grant(PermissionKind::Accessibility),
            )
        } else if unknown(ax) {
            entry(feature, Readiness::Unknown, &[], None, None)
        } else {
            entry(feature, Readiness::Ready, &[], None, None)
        }
    };
    let selected_text = needs_accessibility(
        Feature::SelectedText,
        "Script Kit can't read or replace selected text without Accessibility.",
    );
    let window_control = needs_accessibility(
        Feature::WindowControl,
        "Window commands can't move or resize windows without Accessibility.",
    );

    let screen_capture = match screen {
        PermissionStatus::Authorized => {
            entry(Feature::ScreenCapture, Readiness::Ready, &[], None, None)
        }
        PermissionStatus::Unknown => {
            entry(Feature::ScreenCapture, Readiness::Unknown, &[], None, None)
        }
        _ => entry(
            Feature::ScreenCapture,
            Readiness::AskOnUse,
            &[PermissionKind::ScreenRecording],
            Some(
                "Screenshots and screen context need Screen Recording. macOS may ask on first use; if not, enable it in Privacy & Security."
                    .to_string(),
            ),
            grant(PermissionKind::ScreenRecording),
        ),
    };

    vec![
        text_expansion,
        dictation,
        paste,
        selected_text,
        window_control,
        screen_capture,
    ]
}

/// A startup reminder: only for permissions the user's own setup depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupReminder {
    pub title: String,
    pub body: String,
    pub missing: Vec<PermissionKind>,
}

/// What to remind about at startup, if anything.
///
/// Accessibility is core (pasting, snippets, selected text, windows), so its
/// absence is always worth a reminder. A denied microphone is mentioned only
/// when the user set up a dictation hotkey. Screen Recording is never a
/// startup reminder: macOS asks for it when a screenshot feature first runs.
pub fn startup_reminder(
    status_of: &dyn Fn(PermissionKind) -> PermissionStatus,
    facts: &LiveFacts,
) -> Option<StartupReminder> {
    let ax = status_of(PermissionKind::Accessibility);
    let mic = status_of(PermissionKind::Microphone);
    let mut missing = Vec::new();
    let mut effects = Vec::new();

    if ax == PermissionStatus::Denied || ax == PermissionStatus::NotDetermined {
        missing.push(PermissionKind::Accessibility);
        if facts.keyword_trigger_count > 0 {
            let noun = if facts.keyword_trigger_count == 1 {
                "snippet"
            } else {
                "snippets"
            };
            effects.push(format!(
                "your {} {noun} can't expand",
                facts.keyword_trigger_count
            ));
        }
        effects.push("text is copied instead of pasted".to_string());
        effects.push("window commands and selected text are off".to_string());
    }
    if mic == PermissionStatus::Denied && facts.dictation_hotkey_enabled {
        missing.push(PermissionKind::Microphone);
        effects.push("dictation can't hear you".to_string());
    }
    if missing.is_empty() {
        return None;
    }

    let names = missing
        .iter()
        .map(|kind| kind.name())
        .collect::<Vec<_>>()
        .join(" and ");
    Some(StartupReminder {
        title: format!("Script Kit needs {names}"),
        body: format!(
            "Until it's allowed, {}. Open Script Kit and run \u{201c}Set Up Permissions\u{201d}.",
            join_sentence(&effects)
        ),
        missing,
    })
}

fn join_sentence(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{}, and {last}", init.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PermissionStatus as S;

    fn statuses(ax: S, post: S, mic: S, screen: S) -> impl Fn(PermissionKind) -> PermissionStatus {
        move |kind| match kind {
            PermissionKind::Accessibility => ax,
            PermissionKind::EventSynthesizing => post,
            PermissionKind::Microphone => mic,
            PermissionKind::ScreenRecording => screen,
            PermissionKind::InputMonitoring => S::Denied,
        }
    }

    fn find(list: &[FeatureReadiness], feature: Feature) -> &FeatureReadiness {
        list.iter()
            .find(|entry| entry.feature == feature)
            .expect("every feature is reported")
    }

    fn running() -> LiveFacts {
        LiveFacts {
            text_expansion: TextExpansionState::Running { triggers: 3 },
            keyword_trigger_count: 3,
            dictation_hotkey_enabled: true,
        }
    }

    #[test]
    fn everything_granted_and_running_is_ready() {
        let list = readiness(
            &statuses(S::Authorized, S::Authorized, S::Authorized, S::Authorized),
            &running(),
        );
        assert_eq!(list.len(), 6);
        assert!(
            list.iter().all(|entry| entry.readiness == Readiness::Ready),
            "{list:?}"
        );
        assert!(list.iter().all(|entry| entry.message.is_none()));
    }

    #[test]
    fn missing_accessibility_blocks_external_cursor_delivery_without_promising_a_paste() {
        let list = readiness(
            &statuses(S::Denied, S::Denied, S::Authorized, S::Authorized),
            &running(),
        );
        let snippets = find(&list, Feature::TextExpansion);
        assert_eq!(snippets.readiness, Readiness::Blocked);
        assert_eq!(snippets.missing, vec!["accessibility"]);
        let dictation = find(&list, Feature::Dictation);
        assert_eq!(dictation.readiness, Readiness::Degraded);
        assert!(dictation
            .message
            .as_deref()
            .unwrap_or("")
            .contains("verify the original field"));
        assert_eq!(
            find(&list, Feature::PasteIntoApps).readiness,
            Readiness::Degraded
        );
        assert_eq!(
            find(&list, Feature::WindowControl).readiness,
            Readiness::Blocked
        );
    }

    #[test]
    fn post_event_grant_allows_generic_paste_but_cannot_verify_an_external_cursor() {
        let list = readiness(
            &statuses(S::Denied, S::Authorized, S::Authorized, S::Authorized),
            &running(),
        );
        assert_eq!(
            find(&list, Feature::Dictation).readiness,
            Readiness::Degraded
        );
        assert_eq!(
            find(&list, Feature::PasteIntoApps).readiness,
            Readiness::Ready
        );
    }

    #[test]
    fn microphone_not_asked_is_ask_on_use_and_denied_is_blocked() {
        let asked = readiness(
            &statuses(
                S::Authorized,
                S::Authorized,
                S::NotDetermined,
                S::Authorized,
            ),
            &running(),
        );
        assert_eq!(
            find(&asked, Feature::Dictation).readiness,
            Readiness::AskOnUse
        );
        let denied = readiness(
            &statuses(S::Authorized, S::Authorized, S::Denied, S::Authorized),
            &running(),
        );
        let dictation = find(&denied, Feature::Dictation);
        assert_eq!(dictation.readiness, Readiness::Blocked);
        assert_eq!(
            dictation.fix,
            Some(FeatureFix::Grant {
                permission: PermissionKind::Microphone
            })
        );
    }

    #[test]
    fn a_failed_snippet_watcher_is_blocked_even_with_accessibility() {
        let facts = LiveFacts {
            text_expansion: TextExpansionState::Failed {
                reason: "macOS wouldn't let Script Kit watch typing".to_string(),
            },
            ..running()
        };
        let list = readiness(
            &statuses(S::Authorized, S::Authorized, S::Authorized, S::Authorized),
            &facts,
        );
        let snippets = find(&list, Feature::TextExpansion);
        assert_eq!(snippets.readiness, Readiness::Blocked);
        assert_eq!(snippets.fix, Some(FeatureFix::RestartScriptKit));
    }

    #[test]
    fn secure_input_names_the_app() {
        let facts = LiveFacts {
            text_expansion: TextExpansionState::SecureInput {
                owner: Some("Terminal".to_string()),
            },
            ..running()
        };
        let list = readiness(
            &statuses(S::Authorized, S::Authorized, S::Authorized, S::Authorized),
            &facts,
        );
        let snippets = find(&list, Feature::TextExpansion);
        assert_eq!(snippets.readiness, Readiness::Degraded);
        assert!(snippets
            .message
            .as_deref()
            .unwrap_or("")
            .contains("Terminal"));
    }

    #[test]
    fn unreadable_statuses_are_unknown_not_blocked() {
        let list = readiness(
            &statuses(S::Unknown, S::Unknown, S::Unknown, S::Unknown),
            &LiveFacts::default(),
        );
        assert_eq!(
            find(&list, Feature::Dictation).readiness,
            Readiness::Unknown
        );
        assert_eq!(
            find(&list, Feature::PasteIntoApps).readiness,
            Readiness::Unknown
        );
        assert_eq!(
            find(&list, Feature::SelectedText).readiness,
            Readiness::Unknown
        );
        assert_eq!(
            find(&list, Feature::ScreenCapture).readiness,
            Readiness::Unknown
        );
    }

    #[test]
    fn missing_facts_never_authorize_native_input_or_mark_the_mic_as_ready() {
        for (ax, post, expected) in [
            (S::Unknown, S::Unknown, None),
            (S::Denied, S::Unknown, None),
            (S::Unknown, S::Denied, None),
            (S::Denied, S::Denied, Some(false)),
            (S::Authorized, S::Unknown, Some(true)),
            (S::Unknown, S::Authorized, Some(true)),
        ] {
            assert_eq!(post_access(ax, post), expected);
            let rows = readiness(&statuses(ax, post, S::Unknown, S::Denied), &running());
            assert_eq!(
                find(&rows, Feature::Dictation).readiness,
                Readiness::Unknown
            );
        }
    }

    #[test]
    fn startup_reminder_skips_screen_recording_and_names_snippets() {
        let reminder = startup_reminder(
            &statuses(S::Denied, S::Denied, S::Authorized, S::Denied),
            &running(),
        )
        .expect("accessibility missing is reminded");
        assert_eq!(reminder.missing, vec![PermissionKind::Accessibility]);
        assert_eq!(reminder.title, "Script Kit needs Accessibility");
        assert!(
            reminder.body.contains("your 3 snippets can't expand"),
            "{}",
            reminder.body
        );
        assert!(!reminder.body.contains("Screen"));

        assert_eq!(
            startup_reminder(
                &statuses(S::Authorized, S::Authorized, S::Authorized, S::Denied),
                &running()
            ),
            None,
            "Screen Recording alone never nags at startup"
        );
    }

    #[test]
    fn startup_reminder_mentions_microphone_only_with_a_dictation_hotkey() {
        let mut facts = running();
        let with_hotkey = startup_reminder(
            &statuses(S::Authorized, S::Authorized, S::Denied, S::Authorized),
            &facts,
        )
        .expect("denied microphone with a hotkey is reminded");
        assert_eq!(with_hotkey.missing, vec![PermissionKind::Microphone]);
        facts.dictation_hotkey_enabled = false;
        assert_eq!(
            startup_reminder(
                &statuses(S::Authorized, S::Authorized, S::Denied, S::Authorized),
                &facts
            ),
            None
        );
    }

    #[test]
    fn readiness_serializes_with_wire_codes() {
        let list = readiness(
            &statuses(S::Denied, S::Denied, S::Authorized, S::Authorized),
            &running(),
        );
        let json = serde_json::to_value(find(&list, Feature::TextExpansion)).expect("serializes");
        assert_eq!(json["feature"], "textExpansion");
        assert_eq!(json["readiness"], "blocked");
        assert_eq!(json["fix"]["kind"], "grant");
        assert_eq!(json["fix"]["permission"], "accessibility");
        let state =
            serde_json::to_value(TextExpansionState::Running { triggers: 2 }).expect("serializes");
        assert_eq!(state["state"], "running");
        assert_eq!(state["triggers"], 2);
    }
}
