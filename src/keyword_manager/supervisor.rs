//! Keeps text expansion running whenever macOS allows it, and says so when it
//! can't.
//!
//! One background thread checks every [`CHECK_INTERVAL`]:
//!
//! - Accessibility off: stop watching typing (a revocation while running) and
//!   wait. Granting it later starts text expansion without a restart.
//! - Accessibility on, nothing running: start. If macOS refuses the event tap
//!   (usually a stale privacy entry), report the failure and retry with
//!   back-off instead of pretending to run.
//! - Running, but the event tap's thread has ended: report it and restart.
//! - Running: publish whether Secure Input currently hides typing.
//!
//! Every state goes to [`crate::permissions::set_text_expansion_state`], which
//! feeds the setup list, startup reminders and agent state. Losing text
//! expansion while the user has snippets also shows a system notification,
//! because the main window is usually hidden when it happens.

use crate::permissions::{self, PermissionKind, PermissionStatus, TextExpansionState};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// How often the supervisor checks.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(2);

/// What the supervisor does on one check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SupervisorStep {
    /// Accessibility is off. `stop` when text expansion is still on.
    WaitForAccessibility { stop: bool },
    /// Accessibility is on and nothing runs: start now.
    Start,
    /// Started earlier but the event tap's thread has ended.
    TapLost,
    /// Running normally.
    Running,
    /// A start failed; wait for the retry time.
    WaitToRetry,
}

pub(crate) fn decide(
    accessibility: PermissionStatus,
    enabled: bool,
    monitor_running: bool,
    retry_due: bool,
) -> SupervisorStep {
    if matches!(
        accessibility,
        PermissionStatus::Denied | PermissionStatus::NotDetermined
    ) {
        return SupervisorStep::WaitForAccessibility { stop: enabled };
    }
    // A status that can't be read never starts text expansion (starting
    // checks trust itself), but doesn't stop a running watcher either.
    if accessibility == PermissionStatus::Unknown && !enabled {
        return SupervisorStep::WaitForAccessibility { stop: false };
    }
    match (enabled, monitor_running) {
        (true, true) => SupervisorStep::Running,
        (true, false) => SupervisorStep::TapLost,
        (false, _) if retry_due => SupervisorStep::Start,
        (false, _) => SupervisorStep::WaitToRetry,
    }
}

/// Wait before retry number `failures` (1-based).
pub(crate) fn retry_delay(failures: u32) -> Duration {
    match failures {
        0 => Duration::ZERO,
        1 => Duration::from_secs(10),
        2 => Duration::from_secs(30),
        _ => Duration::from_secs(60),
    }
}

/// The words for a failed start.
pub(crate) fn failure_reason(error: &anyhow::Error) -> String {
    match error.downcast_ref::<crate::keyboard_monitor::KeyboardMonitorError>() {
        Some(
            crate::keyboard_monitor::KeyboardMonitorError::EventTapCreationFailed
            | crate::keyboard_monitor::KeyboardMonitorError::RunLoopSourceCreationFailed,
        ) => "macOS wouldn't let Script Kit watch typing".to_string(),
        Some(crate::keyboard_monitor::KeyboardMonitorError::AccessibilityNotGranted) => {
            "Script Kit doesn't have Accessibility".to_string()
        }
        _ => format!("{error:#}"),
    }
}

/// Start the supervisor thread once.
pub fn start_text_expansion_supervisor() {
    static STARTED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    // Owned evaluation runs never watch real typing.
    if crate::runtime_policy::is_owned_evaluation() || STARTED.set(()).is_err() {
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("text-expansion-supervisor".to_string())
        .spawn(run);
    if let Err(error) = spawned {
        warn!(
            category = "KEYWORD",
            %error,
            "Could not start the text expansion supervisor"
        );
        permissions::set_text_expansion_state(TextExpansionState::Failed {
            reason: "Script Kit couldn't start its snippet watcher".to_string(),
        });
    }
}

fn run() {
    let mut failures: u32 = 0;
    let mut next_retry = Instant::now();
    let mut notified_failure = false;
    let mut loaded = false;
    loop {
        let accessibility = permissions::status(PermissionKind::Accessibility);
        let (enabled, monitor_running, triggers) = super::keyword_manager_activity();
        let step = decide(
            accessibility,
            enabled,
            monitor_running,
            Instant::now() >= next_retry,
        );
        let previous = permissions::text_expansion_state();
        if step == SupervisorStep::Start {
            loaded = true;
        }
        match step {
            SupervisorStep::WaitForAccessibility { stop } => {
                if stop {
                    super::disable_keyword_manager();
                    warn!(
                        category = "KEYWORD",
                        event = "text_expansion_stopped",
                        reason = "accessibility_revoked",
                        "Accessibility was turned off: text expansion stopped"
                    );
                    if triggers > 0 {
                        permissions::notify_user(
                            "Snippets stopped",
                            "Script Kit lost Accessibility, so snippets can't expand. Turn Script Kit back on in System Settings \u{2192} Privacy & Security \u{2192} Accessibility.",
                        );
                    }
                }
                if !loaded {
                    // Load keywords anyway, so reminders can count them.
                    loaded = true;
                    if let Err(error) = super::init_keyword_manager() {
                        warn!(category = "KEYWORD", %error, "Could not load snippet keywords");
                    }
                }
                failures = 0;
                notified_failure = false;
                next_retry = Instant::now();
                permissions::set_text_expansion_state(TextExpansionState::WaitingForAccessibility);
            }
            SupervisorStep::Start => match super::init_keyword_manager() {
                Ok(Some(count)) => {
                    if !matches!(previous, TextExpansionState::NotStarted) {
                        info!(
                            category = "KEYWORD",
                            event = "text_expansion_started",
                            triggers = count,
                            "Text expansion is running again"
                        );
                    }
                    failures = 0;
                    notified_failure = false;
                    permissions::set_text_expansion_state(TextExpansionState::Running {
                        triggers: count,
                    });
                }
                Ok(None) => {
                    permissions::set_text_expansion_state(
                        TextExpansionState::WaitingForAccessibility,
                    );
                }
                Err(error) => {
                    failures = failures.saturating_add(1);
                    next_retry = Instant::now() + retry_delay(failures);
                    let reason = failure_reason(&error);
                    warn!(
                        category = "KEYWORD",
                        event = "text_expansion_start_failed",
                        failures,
                        error = %error,
                        "Text expansion could not start"
                    );
                    report_failure(&reason, triggers, &mut notified_failure);
                    permissions::set_text_expansion_state(TextExpansionState::Failed { reason });
                }
            },
            SupervisorStep::TapLost => {
                super::disable_keyword_manager();
                failures = failures.saturating_add(1);
                next_retry = Instant::now() + retry_delay(failures);
                let reason = "macOS stopped Script Kit from watching typing".to_string();
                warn!(
                    category = "KEYWORD",
                    event = "text_expansion_tap_lost",
                    failures,
                    "The snippet watcher stopped; restarting it"
                );
                report_failure(&reason, triggers, &mut notified_failure);
                permissions::set_text_expansion_state(TextExpansionState::Failed { reason });
            }
            SupervisorStep::Running => {
                let secure = crate::keyboard_monitor::secure_input_status();
                let state = if secure.enabled {
                    TextExpansionState::SecureInput {
                        owner: secure.owner_name,
                    }
                } else {
                    TextExpansionState::Running { triggers }
                };
                permissions::set_text_expansion_state(state);
            }
            SupervisorStep::WaitToRetry => {}
        }
        std::thread::sleep(CHECK_INTERVAL);
    }
}

/// Tell the user once per failure streak, and only if they have snippets.
fn report_failure(reason: &str, triggers: usize, notified: &mut bool) {
    if *notified || triggers == 0 {
        return;
    }
    *notified = true;
    permissions::notify_user(
        "Snippets can't run",
        &format!(
            "{reason}. Open Script Kit and run \u{201c}Set Up Permissions\u{201d}; if Accessibility is already on, restart Script Kit."
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use PermissionStatus as S;

    #[test]
    fn accessibility_off_stops_a_running_watcher_and_waits() {
        assert_eq!(
            decide(S::Denied, true, true, true),
            SupervisorStep::WaitForAccessibility { stop: true }
        );
        assert_eq!(
            decide(S::NotDetermined, false, false, true),
            SupervisorStep::WaitForAccessibility { stop: false }
        );
    }

    #[test]
    fn accessibility_on_starts_or_keeps_running() {
        assert_eq!(
            decide(S::Authorized, false, false, true),
            SupervisorStep::Start
        );
        assert_eq!(
            decide(S::Authorized, true, true, false),
            SupervisorStep::Running
        );
        assert_eq!(
            decide(S::Unknown, false, false, true),
            SupervisorStep::WaitForAccessibility { stop: false },
            "an unreadable status waits instead of reloading snippets every check"
        );
        assert_eq!(
            decide(S::Unknown, true, true, true),
            SupervisorStep::Running
        );
    }

    #[test]
    fn a_dead_tap_is_noticed_and_a_failed_start_waits_for_its_retry() {
        assert_eq!(
            decide(S::Authorized, true, false, true),
            SupervisorStep::TapLost
        );
        assert_eq!(
            decide(S::Authorized, false, false, false),
            SupervisorStep::WaitToRetry
        );
    }

    #[test]
    fn retries_back_off_to_a_minute() {
        assert_eq!(retry_delay(1), Duration::from_secs(10));
        assert_eq!(retry_delay(2), Duration::from_secs(30));
        assert_eq!(retry_delay(9), Duration::from_secs(60));
    }

    #[test]
    fn a_refused_tap_reads_as_plain_words() {
        let error = anyhow::Error::new(
            crate::keyboard_monitor::KeyboardMonitorError::EventTapCreationFailed,
        );
        assert_eq!(
            failure_reason(&error),
            "macOS wouldn't let Script Kit watch typing"
        );
    }
}
