//! What happened to each global hotkey Script Kit tried to register.
//!
//! macOS refuses a shortcut that another app (or Script Kit itself) already
//! holds, and a few combinations are reserved by the system. Before this
//! module such a failure was a log line and the hotkey simply did nothing.
//! Every registration attempt now records its outcome here so the launcher,
//! a notification and `getState` can say which shortcut failed and why.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;

/// Why a hotkey didn't register, in terms the user can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HotkeyFailureKind {
    /// Another app (or another Script Kit shortcut) already holds it.
    TakenByAnotherApp,
    /// macOS refused it, usually because the system reserves it.
    ReservedBySystem,
    /// The shortcut text couldn't be understood.
    Invalid,
    /// Any other operating-system error.
    Other,
}

impl HotkeyFailureKind {
    /// Short reason for the launcher and notifications (no trailing period).
    pub fn reason(self) -> &'static str {
        match self {
            Self::TakenByAnotherApp => "another app or shortcut already uses it",
            Self::ReservedBySystem => "macOS reserves it",
            Self::Invalid => "Script Kit couldn't read the shortcut",
            Self::Other => "macOS refused it",
        }
    }

    /// Stable wire code.
    pub fn code(self) -> &'static str {
        match self {
            Self::TakenByAnotherApp => "takenByAnotherApp",
            Self::ReservedBySystem => "reservedBySystem",
            Self::Invalid => "invalid",
            Self::Other => "other",
        }
    }
}

/// The latest registration outcome for one hotkey.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyRegistration {
    /// Stable identity: `builtin:<label>` or `script:<path or command id>`.
    pub key: String,
    /// What the hotkey opens ("Main launcher", "Notes", a script name).
    pub label: String,
    /// The shortcut as shown to the user.
    pub shortcut: String,
    /// `None` when registered; the reason when it failed.
    pub failure: Option<HotkeyFailureKind>,
    /// Operating-system detail, for diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl HotkeyRegistration {
    pub fn registered(&self) -> bool {
        self.failure.is_none()
    }

    /// One sentence naming the shortcut, what it opens and why it failed.
    pub fn user_message(&self) -> Option<String> {
        let kind = self.failure?;
        Some(format!(
            "{} ({}) doesn't work: {}",
            self.shortcut,
            self.label,
            kind.reason()
        ))
    }
}

fn registry() -> &'static Mutex<BTreeMap<String, HotkeyRegistration>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, HotkeyRegistration>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Set once the startup summary has been shown; later failures (a config
/// reload, a new script shortcut) notify on their own.
static STARTUP_REPORTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Marks the startup summary as shown.
pub fn mark_startup_reported() {
    STARTUP_REPORTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Stores an outcome; returns true when it is a new failure (the shortcut
/// wasn't already failing).
fn store(registration: HotkeyRegistration) -> bool {
    let mut guard = registry().lock().unwrap_or_else(|e| e.into_inner());
    let newly_failed = registration.failure.is_some()
        && guard.get(&registration.key).is_none_or(|previous| {
            previous.registered() || previous.shortcut != registration.shortcut
        });
    guard.insert(registration.key.clone(), registration);
    newly_failed
}

/// Records a successful registration (clearing an earlier failure).
pub fn record_registered(key: &str, label: &str, shortcut: &str) {
    let _ = store(HotkeyRegistration {
        key: key.to_string(),
        label: label.to_string(),
        shortcut: shortcut.to_string(),
        failure: None,
        detail: None,
    });
}

/// Records a failed registration.
pub fn record_failed(
    key: &str,
    label: &str,
    shortcut: &str,
    kind: HotkeyFailureKind,
    detail: impl Into<String>,
) {
    let registration = HotkeyRegistration {
        key: key.to_string(),
        label: label.to_string(),
        shortcut: shortcut.to_string(),
        failure: Some(kind),
        detail: Some(detail.into()),
    };
    let message = failure_notice(std::slice::from_ref(&registration));
    if store(registration) && STARTUP_REPORTED.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(message) = message {
            crate::permissions::notify_user("Shortcut not working", &message);
        }
    }
}

/// Forgets a hotkey that was removed on purpose.
pub fn forget(key: &str) {
    let mut guard = registry().lock().unwrap_or_else(|e| e.into_inner());
    guard.remove(key);
}

/// Every recorded hotkey, failures first.
pub fn snapshot() -> Vec<HotkeyRegistration> {
    let guard = registry().lock().unwrap_or_else(|e| e.into_inner());
    let mut all: Vec<HotkeyRegistration> = guard.values().cloned().collect();
    all.sort_by_key(|registration| registration.registered());
    all
}

/// Hotkeys that failed to register.
pub fn failures() -> Vec<HotkeyRegistration> {
    snapshot()
        .into_iter()
        .filter(|registration| !registration.registered())
        .collect()
}

/// The launcher/notification text for a set of failures, or `None` when all
/// hotkeys work. Names up to two shortcuts, then counts the rest.
pub fn failure_notice(failures: &[HotkeyRegistration]) -> Option<String> {
    let messages: Vec<String> = failures
        .iter()
        .filter_map(HotkeyRegistration::user_message)
        .collect();
    match messages.len() {
        0 => None,
        1 => Some(format!(
            "{}. Choose another shortcut in config.ts.",
            messages[0]
        )),
        2 => Some(format!(
            "{}; {}. Choose other shortcuts in config.ts.",
            messages[0], messages[1]
        )),
        n => Some(format!(
            "{}; {}; and {} more. Choose other shortcuts in config.ts.",
            messages[0],
            messages[1],
            n - 2
        )),
    }
}

/// JSON for `getState` (`permissions.hotkeys`).
pub fn automation_state() -> serde_json::Value {
    let all = snapshot();
    let failed = all.iter().filter(|r| !r.registered()).count();
    serde_json::json!({
        "registeredCount": all.len() - failed,
        "failedCount": failed,
        "hotkeys": all
            .iter()
            .map(|registration| serde_json::json!({
                "key": registration.key,
                "label": registration.label,
                "shortcut": registration.shortcut,
                "registered": registration.registered(),
                "failure": registration.failure.map(HotkeyFailureKind::code),
                "message": registration.user_message(),
            }))
            .collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed(label: &str, shortcut: &str, kind: HotkeyFailureKind) -> HotkeyRegistration {
        HotkeyRegistration {
            key: format!("builtin:{label}"),
            label: label.to_string(),
            shortcut: shortcut.to_string(),
            failure: Some(kind),
            detail: None,
        }
    }

    #[test]
    fn a_failed_registration_names_the_shortcut_and_the_reason() {
        let registration = failed("Notes", "⌘⌃N", HotkeyFailureKind::TakenByAnotherApp);
        assert_eq!(
            registration.user_message().as_deref(),
            Some("⌘⌃N (Notes) doesn't work: another app or shortcut already uses it")
        );
    }

    #[test]
    fn the_notice_names_two_failures_and_counts_the_rest() {
        assert_eq!(failure_notice(&[]), None);
        let one = [failed("Notes", "⌘⌃N", HotkeyFailureKind::ReservedBySystem)];
        assert_eq!(
            failure_notice(&one).as_deref(),
            Some("⌘⌃N (Notes) doesn't work: macOS reserves it. Choose another shortcut in config.ts.")
        );
        let many = [
            failed("Notes", "⌘⌃N", HotkeyFailureKind::Other),
            failed("Dictation", "⌘⇧;", HotkeyFailureKind::Other),
            failed("Logs", "⌘⇧L", HotkeyFailureKind::Other),
        ];
        let notice = failure_notice(&many).expect("notice");
        assert!(notice.contains("and 1 more"), "{notice}");
        assert!(notice.contains("⌘⇧;"), "{notice}");
    }

    #[test]
    fn registering_again_clears_the_failure() {
        let key = "builtin:hotkey-health-test";
        record_failed(
            key,
            "Test",
            "⌘⌥⌃T",
            HotkeyFailureKind::TakenByAnotherApp,
            "AlreadyRegistered",
        );
        assert!(failures().iter().any(|r| r.key == key));
        record_registered(key, "Test", "⌘⌥⌃T");
        assert!(!failures().iter().any(|r| r.key == key));
        let state = automation_state();
        assert!(state["hotkeys"].as_array().is_some_and(|all| all
            .iter()
            .any(|h| h["key"] == key && h["registered"] == true)));
        forget(key);
        assert!(!snapshot().iter().any(|r| r.key == key));
    }
}
