//! Running scheduled scripts and remembering how each run went.
//!
//! A scheduled script runs with nobody watching, so this module owns what
//! the user would otherwise never see:
//!
//! - **The same launch rules as an interactive run** (runtime fallback,
//!   environment allow-list, its own process group), through
//!   [`crate::executor::spawn_unattended_script`].
//! - **A time and memory limit.** `processLimits.maxRuntimeSeconds` and
//!   `processLimits.maxMemoryMb` from config.ts apply; without them a
//!   scheduled run gets [`DEFAULT_SCHEDULED_MAX_RUNTIME`]. A run over the
//!   limit is asked to stop, then stopped, with its whole process group.
//! - **No silent drops.** A run due while four are already going waits its
//!   turn; a run due while the same script is still going is skipped and the
//!   skip is recorded.
//! - **A record on disk** (`~/.scriptkit/db/automation-runs.json`): when each
//!   automation last ran, how it ended, the end of its output and how many
//!   times in a row it has failed. Runs missed while Script Kit was quit or
//!   the Mac slept are caught up once ([`catch_up_due`]).
//! - **A notification when a run fails** (the first failure, then the 3rd
//!   and every 10th in a row), and a pause after
//!   [`PAUSE_AFTER_FAILURES`] failures in a row that backs off up to a day.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

/// Limit for a scheduled run when config.ts doesn't set one.
pub const DEFAULT_SCHEDULED_MAX_RUNTIME: Duration = Duration::from_secs(30 * 60);
/// Time between asking a run to stop and stopping it.
pub const STOP_GRACE: Duration = Duration::from_secs(5);
/// Failures in a row before the schedule pauses (and backs off).
pub const PAUSE_AFTER_FAILURES: u32 = 5;
/// Longest pause between attempts of a failing automation.
pub const MAX_PAUSE: Duration = Duration::from_secs(24 * 60 * 60);
/// Bytes of output kept from the end of a run.
pub const OUTPUT_TAIL_BYTES: usize = 4 * 1024;
/// How late a run may be before it counts as missed (the scheduler checks
/// every 30 s).
const MISSED_GRACE: chrono::Duration = chrono::Duration::seconds(90);

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RunOutcome {
    Succeeded,
    Failed {
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
    },
    TimedOut {
        limit_secs: u64,
    },
    OverMemory {
        limit_mb: u64,
    },
    CouldNotStart,
}

impl RunOutcome {
    pub fn is_failure(&self) -> bool {
        !matches!(self, Self::Succeeded)
    }

    /// A few words for lists and notifications.
    pub fn words(&self) -> String {
        match self {
            Self::Succeeded => "finished".to_string(),
            Self::Failed {
                exit_code: Some(code),
            } => format!("failed (exit {code})"),
            Self::Failed { exit_code: None } => "failed".to_string(),
            Self::TimedOut { limit_secs } => {
                format!("stopped after {}", duration_words(*limit_secs))
            }
            Self::OverMemory { limit_mb } => format!("stopped at {limit_mb} MB of memory"),
            Self::CouldNotStart => "couldn't start".to_string(),
        }
    }
}

fn duration_words(secs: u64) -> String {
    if secs >= 3600 && secs % 3600 == 0 {
        format!("{} h", secs / 3600)
    } else if secs >= 60 && secs % 60 == 0 {
        format!("{} min", secs / 60)
    } else {
        format!("{secs} s")
    }
}

/// One finished run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub outcome: RunOutcome,
    /// End of stdout and stderr (bounded).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub output_tail: String,
    /// Why it failed, when known (last stderr line or the spawn error).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Everything remembered about one automation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last: Option<RunRecord>,
    /// Set while a run is going; also the start of the latest run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub consecutive_failures: u32,
    /// The latest time a due run was skipped or delayed, and why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_skip: Option<SkipRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipRecord {
    pub at: DateTime<Utc>,
    pub reason: String,
}

impl AutomationRunState {
    /// When a paused automation may run again, if it is paused.
    pub fn paused_until(&self) -> Option<DateTime<Utc>> {
        if self.consecutive_failures < PAUSE_AFTER_FAILURES {
            return None;
        }
        let last = self.last.as_ref()?;
        let doublings = (self.consecutive_failures - PAUSE_AFTER_FAILURES).min(16);
        let pause = Duration::from_secs(3600)
            .saturating_mul(1u32 << doublings)
            .min(MAX_PAUSE);
        Some(last.finished_at + chrono::Duration::from_std(pause).ok()?)
    }

    /// "last ran 2:05 PM · failed (exit 1)" for the Automations list.
    pub fn summary(&self, now: DateTime<Utc>) -> Option<String> {
        let last = self.last.as_ref()?;
        let local = last.started_at.with_timezone(&Local);
        let when = if local.date_naive() == now.with_timezone(&Local).date_naive() {
            local.format("%-I:%M %p").to_string()
        } else {
            local.format("%a %-I:%M %p").to_string()
        };
        let mut text = format!("last ran {when} · {}", last.outcome.words());
        if let Some(until) = self.paused_until().filter(|until| *until > now) {
            text.push_str(&format!(
                " · paused until {}",
                until.with_timezone(&Local).format("%a %-I:%M %p")
            ));
        }
        Some(text)
    }
}

/// Whether a run was missed since `last_started` (Script Kit was quit or
/// the Mac slept through it). The scheduler runs a missed automation once.
/// Without any earlier run there is nothing to catch up.
pub fn catch_up_due(
    cron: &croner::Cron,
    last_started: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    let Some(last) = last_started else {
        return false;
    };
    let last_local = last.with_timezone(&Local);
    match cron.find_next_occurrence(&last_local, false) {
        Ok(next) => next.with_timezone(&Utc) + MISSED_GRACE < now,
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn runs_path() -> PathBuf {
    crate::setup::get_kit_path().join("db/automation-runs.json")
}

fn store() -> &'static Mutex<BTreeMap<String, AutomationRunState>> {
    static STORE: OnceLock<Mutex<BTreeMap<String, AutomationRunState>>> = OnceLock::new();
    STORE.get_or_init(|| {
        let loaded = if cfg!(test) {
            BTreeMap::new()
        } else {
            std::fs::read_to_string(runs_path())
                .ok()
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or_default()
        };
        Mutex::new(loaded)
    })
}

fn key(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn update(path: &Path, change: impl FnOnce(&mut AutomationRunState)) -> AutomationRunState {
    let mut guard = store().lock().unwrap_or_else(|e| e.into_inner());
    let state = guard.entry(key(path)).or_default();
    change(state);
    let snapshot = state.clone();
    if !cfg!(test) {
        match serde_json::to_vec_pretty(&*guard) {
            Ok(bytes) => {
                if let Err(error) = crate::atomic_file::write_atomic(&runs_path(), &bytes) {
                    tracing::warn!(%error, "automation_runs_save_failed");
                }
            }
            Err(error) => tracing::warn!(%error, "automation_runs_serialize_failed"),
        }
    }
    snapshot
}

/// What is remembered about the automation at `path`.
pub fn state(path: &Path) -> Option<AutomationRunState> {
    store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&key(path))
        .cloned()
}

/// Start of the latest run of `path`, for missed-run detection.
pub fn last_started(path: &Path) -> Option<DateTime<Utc>> {
    state(path).and_then(|state| state.last_started_at)
}

/// Every remembered automation (for agents).
pub fn all_states() -> BTreeMap<String, AutomationRunState> {
    store().lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn record_skip(path: &Path, reason: &str) {
    update(path, |state| {
        state.last_skip = Some(SkipRecord {
            at: Utc::now(),
            reason: reason.to_string(),
        });
    });
}

/// Stores a finished run; returns the new failure streak.
fn record_finished(path: &Path, record: RunRecord) -> u32 {
    update(path, |state| {
        if record.outcome.is_failure() {
            state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        } else {
            state.consecutive_failures = 0;
        }
        state.last = Some(record);
    })
    .consecutive_failures
}

/// Whether a failure streak of `streak` deserves a notification.
pub fn should_notify_failure(streak: u32) -> bool {
    streak == 1 || streak == 3 || streak == PAUSE_AFTER_FAILURES || streak % 10 == 0
}

/// The failure notification for a run.
pub fn failure_message(name: &str, record: &RunRecord, streak: u32) -> (String, String) {
    let title = if streak >= PAUSE_AFTER_FAILURES {
        format!("{name} keeps failing")
    } else {
        format!("{name} {}", record.outcome.words())
    };
    let mut body = match &record.error {
        Some(error) if !error.trim().is_empty() => error.trim().to_string(),
        _ => format!("The scheduled run {}.", record.outcome.words()),
    };
    if streak >= PAUSE_AFTER_FAILURES {
        body.push_str(&format!(
            " It failed {streak} times in a row, so its schedule is paused for a while."
        ));
    } else if streak > 1 {
        body.push_str(&format!(" ({streak} failures in a row)"));
    }
    (title, body)
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// What happened when a due run was handed over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dispatch {
    Started,
    /// Too many runs at once; it starts when one finishes.
    Waiting,
    /// The same automation is still running.
    SkippedStillRunning,
    /// Paused after repeated failures.
    SkippedPaused,
}

#[derive(Default)]
struct Dispatcher {
    running: HashSet<PathBuf>,
    waiting: VecDeque<PathBuf>,
}

fn dispatcher() -> &'static Mutex<Dispatcher> {
    static DISPATCHER: OnceLock<Mutex<Dispatcher>> = OnceLock::new();
    DISPATCHER.get_or_init(|| Mutex::new(Dispatcher::default()))
}

/// Pure admission rule shared by [`dispatch`] and its tests.
fn admit(dispatcher: &mut Dispatcher, path: &Path, limit: usize) -> Dispatch {
    if dispatcher.running.contains(path) {
        return Dispatch::SkippedStillRunning;
    }
    if dispatcher.running.len() >= limit {
        if !dispatcher.waiting.iter().any(|waiting| waiting == path) {
            dispatcher.waiting.push_back(path.to_path_buf());
        }
        return Dispatch::Waiting;
    }
    dispatcher.running.insert(path.to_path_buf());
    Dispatch::Started
}

/// Hands a due automation over: starts it now, queues it, or records why it
/// was skipped. Never blocks.
pub fn dispatch(path: PathBuf) -> Dispatch {
    if let Some(until) = state(&path).and_then(|state| state.paused_until()) {
        if until > Utc::now() {
            record_skip(&path, "paused after repeated failures");
            return Dispatch::SkippedPaused;
        }
    }
    let admitted = {
        let mut guard = dispatcher().lock().unwrap_or_else(|e| e.into_inner());
        admit(
            &mut guard,
            &path,
            super::SCHEDULED_SCRIPT_MAX_CONCURRENT_RUNS,
        )
    };
    match admitted {
        Dispatch::Started => start(path),
        Dispatch::Waiting => {
            tracing::info!(path = %path.display(), "scheduled_run_waiting_for_slot");
            crate::executor::telemetry::log_scheduler_job_skipped(
                "scheduler::runs::dispatch",
                &path.display().to_string(),
                "waiting_for_slot",
            );
            record_skip(&path, "waited for other scheduled runs to finish");
        }
        Dispatch::SkippedStillRunning => {
            tracing::info!(path = %path.display(), "scheduled_run_skipped_still_running");
            crate::executor::telemetry::log_scheduler_job_skipped(
                "scheduler::runs::dispatch",
                &path.display().to_string(),
                "still_running",
            );
            record_skip(&path, "the previous run was still going");
        }
        Dispatch::SkippedPaused => {}
    }
    admitted
}

/// Number of scheduled runs going now.
pub fn running_count() -> usize {
    dispatcher()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .running
        .len()
}

fn start(path: PathBuf) {
    update(&path, |state| state.last_started_at = Some(Utc::now()));
    let spawned = std::thread::Builder::new()
        .name("scheduled-run".to_string())
        .spawn({
            let path = path.clone();
            move || {
                let record = run_once(&path, &limits());
                finish(&path, record);
            }
        });
    if let Err(error) = spawned {
        finish(
            &path,
            RunRecord {
                started_at: Utc::now(),
                finished_at: Utc::now(),
                outcome: RunOutcome::CouldNotStart,
                output_tail: String::new(),
                error: Some(format!("Couldn't start a thread for the run: {error}")),
            },
        );
    }
}

fn finish(path: &Path, record: RunRecord) {
    let failed = record.outcome.is_failure();
    let streak = record_finished(path, record.clone());
    tracing::info!(
        path = %path.display(),
        outcome = ?record.outcome,
        streak,
        "scheduled_run_finished"
    );
    if failed && should_notify_failure(streak) {
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("A scheduled script");
        let (title, body) = failure_message(name, &record, streak);
        crate::permissions::notify_user(&title, &body);
    }
    let next = {
        let mut guard = dispatcher().lock().unwrap_or_else(|e| e.into_inner());
        guard.running.remove(path);
        guard.waiting.pop_front()
    };
    if let Some(next) = next {
        dispatch(next);
    }
}

/// Limits for a scheduled run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunLimits {
    pub max_runtime: Duration,
    pub max_memory_mb: Option<u64>,
}

fn limits() -> RunLimits {
    let configured = crate::config::load_config().get_process_limits();
    limits_from(configured.max_runtime_seconds, configured.max_memory_mb)
}

/// `processLimits` (seconds and megabytes) as run limits.
pub fn limits_from(max_runtime_seconds: Option<u64>, max_memory_mb: Option<u64>) -> RunLimits {
    RunLimits {
        max_runtime: max_runtime_seconds
            .filter(|secs| *secs > 0)
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_SCHEDULED_MAX_RUNTIME),
        max_memory_mb: max_memory_mb.filter(|mb| *mb > 0),
    }
}

/// Keeps the last `max` bytes written to it (on a char boundary).
#[derive(Clone, Default)]
struct Tail(Arc<Mutex<Vec<u8>>>);

impl Tail {
    fn drain(&self, mut reader: impl Read + Send + 'static) -> std::thread::JoinHandle<()> {
        let tail = self.clone();
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            while let Ok(read) = reader.read(&mut buffer) {
                if read == 0 {
                    break;
                }
                let mut guard = tail.0.lock().unwrap_or_else(|e| e.into_inner());
                guard.extend_from_slice(&buffer[..read]);
                let len = guard.len();
                if len > OUTPUT_TAIL_BYTES * 2 {
                    guard.drain(..len - OUTPUT_TAIL_BYTES);
                }
            }
        })
    }

    fn text(&self) -> String {
        let guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let start = guard.len().saturating_sub(OUTPUT_TAIL_BYTES);
        String::from_utf8_lossy(&guard[start..]).trim().to_string()
    }
}

fn last_line(text: &str) -> Option<String> {
    text.lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(300).collect())
}

fn resident_mb(pid: u32) -> Option<u64> {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
    let mut system = System::new();
    let pid = Pid::from_u32(pid);
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );
    system
        .process(pid)
        .map(|process| process.memory() / (1024 * 1024))
}

/// Runs `path` to completion (or its limit) and reports how it went.
pub fn run_once(path: &Path, limits: &RunLimits) -> RunRecord {
    let started_at = Utc::now();
    let mut spawned = match crate::executor::spawn_unattended_script(path, &[]) {
        Ok(spawned) => spawned,
        Err(error) => {
            return RunRecord {
                started_at,
                finished_at: Utc::now(),
                outcome: RunOutcome::CouldNotStart,
                output_tail: String::new(),
                error: Some(error),
            };
        }
    };
    let pid = spawned.child.id();
    let path_text = path.to_string_lossy().to_string();
    crate::process_manager::PROCESS_MANAGER.register_process(pid, &path_text);

    let stdout_tail = Tail::default();
    let stderr_tail = Tail::default();
    let mut readers = Vec::new();
    if let Some(stdout) = spawned.child.stdout.take() {
        readers.push(stdout_tail.drain(stdout));
    }
    if let Some(stderr) = spawned.child.stderr.take() {
        readers.push(stderr_tail.drain(stderr));
    }

    let clock = Instant::now();
    let mut last_memory_check = Instant::now();
    let mut stopping: Option<(Instant, RunOutcome)> = None;
    let status = loop {
        match spawned.child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(%error, pid, "scheduled_run_wait_failed");
                break None;
            }
        }
        match &stopping {
            None => {
                let over = if clock.elapsed() >= limits.max_runtime {
                    Some(RunOutcome::TimedOut {
                        limit_secs: limits.max_runtime.as_secs(),
                    })
                } else if let Some(limit_mb) = limits
                    .max_memory_mb
                    .filter(|_| last_memory_check.elapsed() >= Duration::from_secs(2))
                {
                    last_memory_check = Instant::now();
                    resident_mb(pid)
                        .filter(|mb| *mb > limit_mb)
                        .map(|_| RunOutcome::OverMemory { limit_mb })
                } else {
                    None
                };
                if let Some(outcome) = over {
                    let _ = crate::executor::signal_process_group(spawned.pgid, false);
                    stopping = Some((Instant::now(), outcome));
                }
            }
            Some((asked_at, _)) if asked_at.elapsed() >= STOP_GRACE => {
                let _ = crate::executor::signal_process_group(spawned.pgid, true);
            }
            Some(_) => {}
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    if stopping.is_some() {
        // Stop anything the script left in its group as well.
        let _ = crate::executor::signal_process_group(spawned.pgid, true);
    }
    crate::process_manager::PROCESS_MANAGER.unregister_process(pid);
    for reader in readers {
        let _ = reader.join();
    }

    let stdout = stdout_tail.text();
    let stderr = stderr_tail.text();
    let outcome = match (stopping, status) {
        (Some((_, outcome)), _) => outcome,
        (None, Some(status)) if status.success() => RunOutcome::Succeeded,
        (None, Some(status)) => RunOutcome::Failed {
            exit_code: status.code(),
        },
        (None, None) => RunOutcome::Failed { exit_code: None },
    };
    let error = outcome
        .is_failure()
        .then(|| last_line(&stderr).or_else(|| last_line(&stdout)))
        .flatten();
    let output_tail = match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stdout}\n--- stderr ---\n{stderr}"),
    };
    RunRecord {
        started_at,
        finished_at: Utc::now(),
        outcome,
        output_tail,
        error,
    }
}

/// JSON for agents: every automation's last result, streak and pause.
pub fn automation_state() -> serde_json::Value {
    let now = Utc::now();
    let states = all_states();
    serde_json::json!({
        "schemaVersion": 1,
        "running": running_count(),
        "limit": super::SCHEDULED_SCRIPT_MAX_CONCURRENT_RUNS,
        "automations": states.iter().map(|(path, state)| serde_json::json!({
            "path": path,
            "lastStartedAt": state.last_started_at,
            "last": state.last,
            "consecutiveFailures": state.consecutive_failures,
            "pausedUntil": state.paused_until().filter(|until| *until > now),
            "lastSkip": state.last_skip,
            "summary": state.summary(now),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(outcome: RunOutcome, minutes_ago: i64) -> RunRecord {
        let at = Utc::now() - chrono::Duration::minutes(minutes_ago);
        RunRecord {
            started_at: at,
            finished_at: at,
            outcome,
            output_tail: String::new(),
            error: None,
        }
    }

    #[test]
    fn a_run_missed_while_quit_is_caught_up_once() {
        let every_five = super::super::parse_cron("*/5 * * * *").expect("cron");
        let now = Utc::now();
        assert!(catch_up_due(
            &every_five,
            Some(now - chrono::Duration::minutes(20)),
            now
        ));
        assert!(!catch_up_due(
            &every_five,
            Some(now - chrono::Duration::seconds(30)),
            now
        ));
        assert!(
            !catch_up_due(&every_five, None, now),
            "nothing to catch up yet"
        );
    }

    #[test]
    fn a_busy_scheduler_queues_instead_of_dropping() {
        let mut dispatcher = Dispatcher::default();
        let a = PathBuf::from("/a.ts");
        let b = PathBuf::from("/b.ts");
        let c = PathBuf::from("/c.ts");
        assert_eq!(admit(&mut dispatcher, &a, 2), Dispatch::Started);
        assert_eq!(admit(&mut dispatcher, &a, 2), Dispatch::SkippedStillRunning);
        assert_eq!(admit(&mut dispatcher, &b, 2), Dispatch::Started);
        assert_eq!(admit(&mut dispatcher, &c, 2), Dispatch::Waiting);
        assert_eq!(admit(&mut dispatcher, &c, 2), Dispatch::Waiting);
        assert_eq!(dispatcher.waiting.len(), 1, "queued once");
    }

    #[test]
    fn repeated_failures_pause_with_growing_waits() {
        let mut state = AutomationRunState {
            last: Some(record(RunOutcome::Failed { exit_code: Some(1) }, 0)),
            consecutive_failures: PAUSE_AFTER_FAILURES - 1,
            ..Default::default()
        };
        assert_eq!(state.paused_until(), None);
        state.consecutive_failures = PAUSE_AFTER_FAILURES;
        let first = state.paused_until().expect("paused");
        state.consecutive_failures = PAUSE_AFTER_FAILURES + 2;
        let later = state.paused_until().expect("paused");
        assert!(later > first);
        state.consecutive_failures = 60;
        let capped = state.paused_until().expect("paused");
        let finished = state.last.as_ref().expect("last").finished_at;
        assert!(capped - finished <= chrono::Duration::hours(24));
    }

    #[test]
    fn failures_notify_first_then_sparingly() {
        let notified: Vec<u32> = (1..=25).filter(|n| should_notify_failure(*n)).collect();
        assert_eq!(notified, vec![1, 3, 5, 10, 20]);
        let (title, body) = failure_message(
            "backup",
            &RunRecord {
                error: Some("ENOENT: no such file".to_string()),
                ..record(RunOutcome::Failed { exit_code: Some(1) }, 0)
            },
            1,
        );
        assert_eq!(title, "backup failed (exit 1)");
        assert_eq!(body, "ENOENT: no such file");
        let (title, body) = failure_message(
            "backup",
            &record(RunOutcome::TimedOut { limit_secs: 1800 }, 0),
            PAUSE_AFTER_FAILURES,
        );
        assert_eq!(title, "backup keeps failing");
        assert!(body.contains("stopped after 30 min"), "{body}");
        assert!(body.contains("paused"), "{body}");
    }

    #[test]
    fn limits_come_from_config_in_seconds_and_megabytes() {
        assert_eq!(
            limits_from(None, None),
            RunLimits {
                max_runtime: DEFAULT_SCHEDULED_MAX_RUNTIME,
                max_memory_mb: None
            }
        );
        let limits = limits_from(Some(90), Some(512));
        assert_eq!(
            limits.max_runtime,
            Duration::from_secs(90),
            "seconds, not ms"
        );
        assert_eq!(limits.max_memory_mb, Some(512));
        assert_eq!(limits_from(Some(0), Some(0)), limits_from(None, None));
    }

    #[test]
    fn summary_says_when_and_how_it_went() {
        let state = AutomationRunState {
            last: Some(record(RunOutcome::Succeeded, 1)),
            ..Default::default()
        };
        let summary = state.summary(Utc::now()).expect("summary");
        assert!(summary.starts_with("last ran "), "{summary}");
        assert!(summary.ends_with("finished"), "{summary}");
    }

    #[test]
    fn the_output_tail_keeps_the_end() {
        let tail = Tail::default();
        let data = format!("{}END", "x".repeat(OUTPUT_TAIL_BYTES * 3));
        tail.drain(std::io::Cursor::new(data.into_bytes()))
            .join()
            .expect("drain");
        let text = tail.text();
        assert!(text.ends_with("END"));
        assert!(text.len() <= OUTPUT_TAIL_BYTES);
    }

    #[cfg(unix)]
    #[test]
    fn a_run_over_its_limit_is_stopped_with_its_children() {
        if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process).is_err() {
            return;
        }
        let dir = tempfile::tempdir().expect("tempdir");
        let script = dir.path().join("sleepy.js");
        std::fs::write(
            &script,
            "console.log('started'); setInterval(() => {}, 1000);",
        )
        .expect("write");
        let record = run_once(
            &script,
            &RunLimits {
                max_runtime: Duration::from_millis(500),
                max_memory_mb: None,
            },
        );
        if record.outcome == RunOutcome::CouldNotStart {
            return; // no node or bun on this machine
        }
        assert_eq!(record.outcome, RunOutcome::TimedOut { limit_secs: 0 });
        assert!(record.output_tail.contains("started"), "{record:?}");
    }
}
