//! One place that knows which macOS privacy permissions Script Kit has.
//!
//! Every feature asks this module instead of calling the OS directly, so the
//! answer is the same everywhere and a change is noticed once:
//!
//! - [`status`] reads the live status and records it. Reads are cheap
//!   preflight calls that never show a prompt.
//! - A background watcher ([`start_watcher`]) re-reads every status every few
//!   seconds and publishes each change to [`subscribe`]rs, so a grant made in
//!   System Settings, or a revocation while Script Kit runs, reaches the
//!   features that depend on it (text expansion re-arms or stops).
//! - [`note_requested`] remembers that Script Kit asked for a permission, so
//!   [`grant_looks_stuck`] can spot the common case where System Settings shows
//!   Script Kit switched on but macOS still reports it as off (a rebuilt or
//!   moved app, or Screen Recording before a relaunch).
//! - [`features`] turns statuses plus live facts (is the snippet watcher
//!   running?) into per-feature readiness with the words to show the user.
//! - [`recovery`] restarts Script Kit or resets its entry in the privacy
//!   database when a grant doesn't take.

pub mod features;
pub mod recovery;

pub use crate::permissions_wizard::PermissionKind;
pub use crate::platform::permiso_detect::PermissionStatus;
pub use features::{FeatureReadiness, LiveFacts, Readiness, TextExpansionState};

use parking_lot::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How often the watcher re-reads statuses.
pub const WATCH_INTERVAL: Duration = Duration::from_secs(2);

/// A requested permission still off after this long probably needs a restart
/// or a reset of Script Kit's entry.
pub const STUCK_AFTER: Duration = Duration::from_secs(15);

/// One status transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionChange {
    pub kind: PermissionKind,
    /// `None` the first time a status is read.
    pub from: Option<PermissionStatus>,
    pub to: PermissionStatus,
}

impl PermissionChange {
    /// Granted before, not granted now.
    pub fn is_revocation(&self) -> bool {
        self.from == Some(PermissionStatus::Authorized) && self.to != PermissionStatus::Authorized
    }

    /// Not granted before (or never read), granted now.
    pub fn is_grant(&self) -> bool {
        self.from != Some(PermissionStatus::Authorized) && self.to == PermissionStatus::Authorized
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Entry {
    status: Option<PermissionStatus>,
    changed_at_unix_ms: Option<u64>,
    requested_at: Option<Instant>,
    requested_at_unix_ms: Option<u64>,
}

struct Registry {
    entries: [Entry; KIND_COUNT],
    subscribers: Vec<async_channel::Sender<PermissionChange>>,
    text_expansion: TextExpansionState,
}

const KIND_COUNT: usize = 5;

fn index(kind: PermissionKind) -> usize {
    match kind {
        PermissionKind::Accessibility => 0,
        PermissionKind::ScreenRecording => 1,
        PermissionKind::EventSynthesizing => 2,
        PermissionKind::InputMonitoring => 3,
        PermissionKind::Microphone => 4,
    }
}

fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        Mutex::new(Registry {
            entries: [Entry::default(); KIND_COUNT],
            subscribers: Vec::new(),
            text_expansion: TextExpansionState::default(),
        })
    })
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

/// Read the live status of `kind` (no prompt) and record it.
pub fn status(kind: PermissionKind) -> PermissionStatus {
    let status = crate::permissions_wizard::detect_permission(kind);
    record(kind, status);
    status
}

/// Live read: is `kind` granted?
pub fn is_granted(kind: PermissionKind) -> bool {
    status(kind) == PermissionStatus::Authorized
}

/// Live read of Accessibility trust, the permission most features need.
pub fn accessibility_granted() -> bool {
    is_granted(PermissionKind::Accessibility)
}

/// Can Script Kit post keystrokes (⌘V, Delete) to other apps?
///
/// macOS allows it with Accessibility, or with the narrower "post events"
/// grant. Neither a missing observation nor an unavailable detector authorizes native input.
pub fn can_post_keystrokes() -> bool {
    let accessibility = status(PermissionKind::Accessibility);
    if accessibility == PermissionStatus::Authorized {
        return true;
    }
    let post_events = status(PermissionKind::EventSynthesizing);
    features::post_access(accessibility, post_events) == Some(true)
}

/// The last recorded status, without reading the OS.
pub fn cached(kind: PermissionKind) -> Option<PermissionStatus> {
    registry().lock().entries[index(kind)].status
}

/// Record a status read elsewhere. Returns the change, if it is one, after
/// sending it to subscribers.
pub fn record(kind: PermissionKind, status: PermissionStatus) -> Option<PermissionChange> {
    let mut registry = registry().lock();
    let entry = &mut registry.entries[index(kind)];
    if entry.status == Some(status) {
        return None;
    }
    let change = PermissionChange {
        kind,
        from: entry.status,
        to: status,
    };
    entry.status = Some(status);
    entry.changed_at_unix_ms = Some(unix_ms_now());
    if status == PermissionStatus::Authorized {
        entry.requested_at = None;
        entry.requested_at_unix_ms = None;
    }
    // The first read of a status is not news; only transitions are.
    if change.from.is_some() {
        tracing::info!(
            category = "PERMISSIONS",
            event = "permission_changed",
            permission = %kind,
            from = ?change.from,
            to = ?change.to,
            "Permission status changed"
        );
        registry
            .subscribers
            .retain(|subscriber| subscriber.try_send(change).is_ok() || !subscriber.is_closed());
    }
    Some(change)
}

/// Re-read every status. Returns the transitions found.
pub fn refresh_all() -> Vec<PermissionChange> {
    PermissionKind::all()
        .iter()
        .filter_map(|&kind| {
            let status = crate::permissions_wizard::detect_permission(kind);
            record(kind, status).filter(|change| change.from.is_some())
        })
        .collect()
}

/// Receive every later status transition.
pub fn subscribe() -> async_channel::Receiver<PermissionChange> {
    let (sender, receiver) = async_channel::unbounded();
    registry().lock().subscribers.push(sender);
    receiver
}

/// Remember that Script Kit just asked macOS (or the user) for `kind`.
pub fn note_requested(kind: PermissionKind) {
    let mut registry = registry().lock();
    let entry = &mut registry.entries[index(kind)];
    entry.requested_at = Some(Instant::now());
    entry.requested_at_unix_ms = Some(unix_ms_now());
}

/// Script Kit asked for `kind` a while ago and it is still off.
///
/// The usual cause is a stale entry: System Settings shows Script Kit switched
/// on, but the entry belongs to an older copy of the app, or (for Screen
/// Recording) macOS only applies the grant after a relaunch.
pub fn grant_looks_stuck(kind: PermissionKind) -> bool {
    let entry = registry().lock().entries[index(kind)];
    stuck(entry.requested_at, entry.status, Instant::now())
}

fn stuck(requested_at: Option<Instant>, status: Option<PermissionStatus>, now: Instant) -> bool {
    matches!(
        status,
        Some(PermissionStatus::Denied | PermissionStatus::NotDetermined)
    ) && requested_at.is_some_and(|at| now.saturating_duration_since(at) >= STUCK_AFTER)
}

/// Text expansion publishes its live state here.
pub fn set_text_expansion_state(state: TextExpansionState) {
    let mut registry = registry().lock();
    if registry.text_expansion != state {
        tracing::info!(
            category = "PERMISSIONS",
            event = "text_expansion_state",
            state = state.code(),
            "Text expansion state changed"
        );
        registry.text_expansion = state;
    }
}

pub fn text_expansion_state() -> TextExpansionState {
    registry().lock().text_expansion.clone()
}

/// Statuses and live facts as they are now.
pub fn live_facts() -> LiveFacts {
    #[cfg(target_os = "macos")]
    let keyword_trigger_count = crate::keyword_manager::keyword_trigger_count();
    #[cfg(not(target_os = "macos"))]
    let keyword_trigger_count = 0;
    LiveFacts {
        text_expansion: text_expansion_state(),
        keyword_trigger_count,
        dictation_hotkey_enabled: crate::config::load_config().is_dictation_hotkey_enabled(),
    }
}

/// Readiness of every feature, from live statuses.
pub fn feature_readiness() -> Vec<FeatureReadiness> {
    let facts = live_facts();
    features::readiness(&|kind| status(kind), &facts)
}

/// Start the background watcher once. It re-reads every status every
/// [`WATCH_INTERVAL`] so grants and revocations reach subscribers.
pub fn start_watcher() {
    static STARTED: OnceLock<()> = OnceLock::new();
    // Owned evaluation runs refuse native status reads; polling them would
    // only log refusals.
    if crate::runtime_policy::is_owned_evaluation() || STARTED.set(()).is_err() {
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("permission-watcher".to_string())
        .spawn(|| {
            refresh_all();
            loop {
                std::thread::sleep(WATCH_INTERVAL);
                refresh_all();
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(
            category = "PERMISSIONS",
            %error,
            "Could not start the permission watcher; changes are noticed only on use"
        );
    }
}

/// Show a system notification from a background path the user can't see
/// (the main window is hidden). Best effort; failures are logged.
pub fn notify_user(title: &str, body: &str) {
    if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Notification).is_err() {
        return;
    }
    let title = title.to_string();
    let body = body.to_string();
    let _ = std::thread::Builder::new()
        .name("permission-notify".to_string())
        .spawn(move || {
            if let Err(error) = notify_rust::Notification::new()
                .summary(&title)
                .body(&body)
                .show()
            {
                tracing::warn!(
                    category = "PERMISSIONS",
                    %error,
                    "Could not show a system notification"
                );
            }
        });
}

/// Wire name of a status, used in agent state.
pub fn status_code(status: PermissionStatus) -> &'static str {
    match status {
        PermissionStatus::Authorized => "granted",
        PermissionStatus::Denied => "denied",
        PermissionStatus::NotDetermined => "notAsked",
        PermissionStatus::Unknown => "unknown",
    }
}

/// What the OS established, not an inference about a user's choice. CG and AX expose
/// boolean preflight results, while AVFoundation distinguishes not-determined vs denied.
fn observation_code(kind: PermissionKind, status: PermissionStatus) -> &'static str {
    match status {
        PermissionStatus::Authorized => "authorizedBySystem",
        PermissionStatus::Unknown => "unavailable",
        PermissionStatus::NotDetermined => "notDeterminedBySystem",
        PermissionStatus::Denied if kind == PermissionKind::Microphone => {
            "deniedOrRestrictedBySystem"
        }
        PermissionStatus::Denied => "notGrantedByPreflight",
    }
}

/// Wire name of a permission, used in agent state.
pub fn kind_code(kind: PermissionKind) -> &'static str {
    match kind {
        PermissionKind::Accessibility => "accessibility",
        PermissionKind::ScreenRecording => "screenRecording",
        PermissionKind::EventSynthesizing => "postEvents",
        PermissionKind::InputMonitoring => "inputMonitoring",
        PermissionKind::Microphone => "microphone",
    }
}

/// Everything an agent needs to know about permissions, read live.
///
/// Statuses, whether Script Kit asked and when, whether a grant looks stuck,
/// and the readiness of each feature with the words the user would see.
pub fn automation_state() -> serde_json::Value {
    let facts = live_facts();
    let mut statuses = Vec::new();
    let mut observed = [PermissionStatus::Unknown; KIND_COUNT];
    for &kind in PermissionKind::all() {
        let live = status(kind);
        observed[index(kind)] = live;
        let entry = registry().lock().entries[index(kind)];
        statuses.push(serde_json::json!({
            "permission": kind_code(kind),
            "name": kind.name(),
            "status": status_code(live), // compatibility for existing clients
            "observation": observation_code(kind, live),
            "requirement": kind.requirement(),
            "requestedAtUnixMs": entry.requested_at_unix_ms,
            "changedAtUnixMs": entry.changed_at_unix_ms,
            "grantLooksStuck": stuck(entry.requested_at, entry.status, Instant::now()),
            "settingsUrl": kind.settings_url(),
        }));
    }
    let features = features::readiness(&|kind| observed[index(kind)], &facts);
    serde_json::json!({
        "schemaVersion": 1,
        "source": "permissions.service",
        "readinessScope": "privacyPermissions",
        "watcherIntervalMs": WATCH_INTERVAL.as_millis() as u64,
        "permissions": statuses,
        "features": features,
        "textExpansion": facts.text_expansion,
        "canRestart": recovery::restart_supported(),
        "bundleIdentifier": recovery::host_bundle_identifier(),
        "hotkeys": crate::hotkey_health::automation_state(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn false_preflight_is_not_evidence_that_a_person_declined_a_prompt() {
        assert_eq!(
            observation_code(PermissionKind::Accessibility, PermissionStatus::Denied),
            "notGrantedByPreflight"
        );
        assert_eq!(
            observation_code(PermissionKind::Microphone, PermissionStatus::NotDetermined),
            "notDeterminedBySystem"
        );
        assert_eq!(
            observation_code(PermissionKind::Microphone, PermissionStatus::Denied),
            "deniedOrRestrictedBySystem"
        );
    }
    #[test]
    fn stuck_needs_a_request_and_time_and_a_status_that_is_still_off() {
        let now = Instant::now();
        let long_ago = now - STUCK_AFTER - Duration::from_secs(1);
        let just_now = now - Duration::from_secs(1);
        assert!(stuck(Some(long_ago), Some(PermissionStatus::Denied), now));
        assert!(stuck(
            Some(long_ago),
            Some(PermissionStatus::NotDetermined),
            now
        ));
        assert!(!stuck(Some(just_now), Some(PermissionStatus::Denied), now));
        assert!(!stuck(None, Some(PermissionStatus::Denied), now));
        assert!(!stuck(
            Some(long_ago),
            Some(PermissionStatus::Authorized),
            now
        ));
        assert!(!stuck(Some(long_ago), Some(PermissionStatus::Unknown), now));
    }

    #[test]
    fn grants_and_revocations_are_named() {
        let revoke = PermissionChange {
            kind: PermissionKind::Accessibility,
            from: Some(PermissionStatus::Authorized),
            to: PermissionStatus::Denied,
        };
        assert!(revoke.is_revocation());
        assert!(!revoke.is_grant());
        let grant = PermissionChange {
            kind: PermissionKind::Accessibility,
            from: Some(PermissionStatus::Denied),
            to: PermissionStatus::Authorized,
        };
        assert!(grant.is_grant());
        assert!(!grant.is_revocation());
    }

    #[test]
    fn recorded_transitions_reach_subscribers_and_repeats_do_not() {
        // Microphone is the kind no other test in this module records.
        let receiver = subscribe();
        let kind = PermissionKind::Microphone;
        record(kind, PermissionStatus::NotDetermined);
        record(kind, PermissionStatus::NotDetermined);
        record(kind, PermissionStatus::Authorized);
        record(kind, PermissionStatus::Denied);
        let mut seen = Vec::new();
        while let Ok(change) = receiver.try_recv() {
            if change.kind == kind {
                seen.push((change.from, change.to));
            }
        }
        assert!(seen.contains(&(
            Some(PermissionStatus::NotDetermined),
            PermissionStatus::Authorized
        )));
        assert!(seen.contains(&(Some(PermissionStatus::Authorized), PermissionStatus::Denied)));
        assert_eq!(cached(kind), Some(PermissionStatus::Denied));
    }

    /// Reads this Mac's real statuses (preflight calls only: no prompts).
    /// Run with `--ignored --nocapture` to see the agent state.
    #[test]
    #[ignore = "reads the real privacy database of the machine running it"]
    fn live_state_reads_every_permission_without_prompting() {
        let state = automation_state();
        println!(
            "{}",
            serde_json::to_string_pretty(&state).unwrap_or_default()
        );
        let permissions = state["permissions"].as_array().cloned().unwrap_or_default();
        assert_eq!(permissions.len(), 5);
        for entry in &permissions {
            let status = entry["status"].as_str().unwrap_or_default();
            assert!(
                ["granted", "denied", "notAsked", "unknown"].contains(&status),
                "{entry}"
            );
        }
        assert_eq!(state["features"].as_array().map(Vec::len), Some(6));
        println!("lid closed: {:?}", crate::dictation::laptop_lid_closed());
    }

    #[test]
    fn wire_codes_are_stable() {
        assert_eq!(
            kind_code(PermissionKind::ScreenRecording),
            "screenRecording"
        );
        assert_eq!(status_code(PermissionStatus::NotDetermined), "notAsked");
    }
}
