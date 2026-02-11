//! Script scheduling module for cron-based and natural language script execution.
//!
//! This module provides functionality to schedule scripts for automatic execution
//! based on cron expressions or natural language schedules (e.g., "every tuesday at 2pm").
//!
//! # Metadata Keys
//! Scripts can specify schedules using two metadata formats:
//! - `// Cron: */5 * * * *` - Raw cron patterns (minute precision)
//! - `// Schedule: every tuesday at 2pm` - Natural language schedules
//!

// --- merged from part_000.rs ---
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use croner::Cron;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{debug, info, warn};
/// Indicates whether the schedule came from a raw cron expression or natural language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleSource {
    /// From `// Cron:` metadata
    Cron,
    /// From `// Schedule:` metadata (natural language converted to cron)
    NaturalLanguage,
}
/// A script with its parsed cron schedule and next execution time.
#[derive(Debug, Clone)]
pub struct ScheduledScript {
    /// Path to the script file
    pub path: PathBuf,
    /// The original cron expression string (after conversion if from natural language)
    pub cron_expr: String,
    /// Next scheduled execution time
    pub next_run: DateTime<Utc>,
    /// Whether this schedule came from Cron: or Schedule: metadata
    #[allow(dead_code)]
    pub source: ScheduleSource,
}
/// Events emitted by the scheduler.
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// A script is due to run
    RunScript(PathBuf),
    /// An error occurred during scheduling
    #[allow(dead_code)]
    Error(String),
}
/// Manages scheduled script execution.
///
/// The scheduler maintains a list of scripts with their cron schedules,
/// periodically checks which scripts are due, and sends events when
/// scripts should be executed.
pub struct Scheduler {
    /// Scheduled scripts keyed by path (protected by mutex for thread-safe access)
    scripts: Arc<Mutex<HashMap<PathBuf, ScheduledScript>>>,
    /// Sender for scheduler events
    tx: Sender<SchedulerEvent>,
    /// Handle to the background thread (if started)
    thread_handle: Option<JoinHandle<()>>,
    /// Flag to signal the background thread to stop.
    /// Atomic to avoid lock contention in the hot scheduler loop.
    running: Arc<AtomicBool>,
    /// One-shot signal to wake the scheduler immediately during stop.
    stop_tx: Option<Sender<()>>,
}
impl Scheduler {
    /// Create a new Scheduler.
    ///
    /// Returns a tuple of (scheduler, receiver) where the receiver will emit
    /// SchedulerEvent when scripts are due to run.
    pub fn new() -> (Self, Receiver<SchedulerEvent>) {
        let (tx, rx) = channel();
        let scheduler = Scheduler {
            scripts: Arc::new(Mutex::new(HashMap::new())),
            tx,
            thread_handle: None,
            running: Arc::new(AtomicBool::new(false)),
            stop_tx: None,
        };
        (scheduler, rx)
    }

    /// Add a script with a cron or natural language schedule.
    ///
    /// # Arguments
    /// * `path` - Path to the script file
    /// * `cron` - Optional raw cron expression (from `// Cron:` metadata)
    /// * `schedule` - Optional natural language schedule (from `// Schedule:` metadata)
    ///
    /// # Returns
    /// Returns `Ok(())` if the script was successfully added, or an error if
    /// the schedule could not be parsed.
    ///
    /// # Note
    /// If both `cron` and `schedule` are provided, `cron` takes precedence.
    /// If neither is provided, returns an error.
    pub fn add_script(
        &self,
        path: PathBuf,
        cron: Option<String>,
        schedule: Option<String>,
    ) -> Result<()> {
        let (cron_expr, source) = match (cron, schedule) {
            (Some(expr), _) => (expr, ScheduleSource::Cron),
            (None, Some(natural)) => {
                let expr = natural_to_cron(&natural).with_context(|| {
                    format!("Failed to parse natural language schedule: {}", natural)
                })?;
                (expr, ScheduleSource::NaturalLanguage)
            }
            (None, None) => {
                anyhow::bail!("Either cron or schedule must be provided");
            }
        };

        // Parse and validate the cron expression
        let parsed_cron = parse_cron(&cron_expr)
            .with_context(|| format!("Failed to parse cron expression: {}", cron_expr))?;

        // Calculate the next run time in local timezone semantics,
        // then normalize to UTC for storage/comparison.
        let now = Utc::now();
        let next_run = find_next_occurrence_utc_in_timezone(&parsed_cron, &now, &Local)
            .context("Failed to calculate next run time")?;

        let scheduled_script = ScheduledScript {
            path: path.clone(),
            cron_expr: cron_expr.clone(),
            next_run,
            source: source.clone(),
        };

        // Add to the list
        let mut scripts = self.scripts.lock().unwrap_or_else(|e| e.into_inner());

        // Insert or update by path
        if let Some(existing) = scripts.get_mut(&path) {
            *existing = scheduled_script;
            info!(
                path = %path.display(),
                cron = %cron_expr,
                source = ?source,
                next_run = %next_run,
                "Updated scheduled script"
            );
        } else {
            scripts.insert(path.clone(), scheduled_script);
            info!(
                path = %path.display(),
                cron = %cron_expr,
                source = ?source,
                next_run = %next_run,
                "Added scheduled script"
            );
        }

        Ok(())
    }

    /// Remove a script from the scheduler.
    #[allow(dead_code)]
    pub fn remove_script(&self, path: &PathBuf) -> bool {
        let mut scripts = self.scripts.lock().unwrap_or_else(|e| e.into_inner());
        let removed = scripts.remove(path).is_some();
        if removed {
            info!(path = %path.display(), "Removed scheduled script");
        }
        removed
    }

    /// Get a list of all scheduled scripts (for debugging/display).
    #[allow(dead_code)]
    pub fn list_scripts(&self) -> Vec<ScheduledScript> {
        let mut scripts: Vec<ScheduledScript> = self
            .scripts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect();
        scripts.sort_by(|a, b| a.path.cmp(&b.path));
        scripts
    }

    /// Start the background scheduler loop.
    ///
    /// This spawns a background thread that checks every minute for scripts
    /// that are due to run, sending RunScript events for each.
    ///
    /// # Returns
    /// Returns a JoinHandle for the spawned thread.
    pub fn start(&mut self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            anyhow::bail!("Scheduler already running");
        }

        let scripts = Arc::clone(&self.scripts);
        let tx = self.tx.clone();
        let running = Arc::clone(&self.running);
        let (stop_tx, stop_rx) = channel();
        self.stop_tx = Some(stop_tx);

        let handle = thread::spawn(move || {
            Self::scheduler_loop(scripts, tx, running, stop_rx);
        });

        self.thread_handle = Some(handle);
        info!("Scheduler started");
        Ok(())
    }

    /// Stop the scheduler.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        info!("Scheduler stopped");
    }

    /// Internal scheduler loop running in background thread.
    fn scheduler_loop(
        scripts: Arc<Mutex<HashMap<PathBuf, ScheduledScript>>>,
        tx: Sender<SchedulerEvent>,
        running: Arc<AtomicBool>,
        stop_rx: Receiver<()>,
    ) {
        // Check every 30 seconds (balance between responsiveness and CPU usage)
        let check_interval = Duration::from_secs(30);

        info!(check_interval_secs = 30, "Scheduler loop started");

        loop {
            // Check if we should stop
            if !running.load(Ordering::SeqCst) {
                info!("Scheduler loop stopping");
                break;
            }

            // Check for due scripts
            let now = Utc::now();
            let mut scripts_to_run: Vec<PathBuf> = Vec::new();
            let mut updates: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

            {
                let scripts = scripts.lock().unwrap_or_else(|e| e.into_inner());
                for script in scripts.values() {
                    if now >= script.next_run {
                        scripts_to_run.push(script.path.clone());

                        // Calculate next run time
                        if let Ok(cron) = parse_cron(&script.cron_expr) {
                            if let Ok(next) =
                                find_next_occurrence_utc_in_timezone(&cron, &now, &Local)
                            {
                                updates.push((script.path.clone(), next));
                            }
                        }
                    }
                }
            }

            // Send run events and update next_run times
            for path in scripts_to_run {
                debug!(path = %path.display(), "Script due to run");
                if tx.send(SchedulerEvent::RunScript(path.clone())).is_err() {
                    warn!("Failed to send RunScript event, receiver dropped");
                    return;
                }
            }

            // Update next_run times
            if !updates.is_empty() {
                let mut scripts = scripts.lock().unwrap_or_else(|e| e.into_inner());
                for (path, next_run) in updates {
                    if let Some(script) = scripts.get_mut(&path) {
                        script.next_run = next_run;
                        debug!(
                            path = %path.display(),
                            next_run = %next_run,
                            "Updated next run time"
                        );
                    }
                }
            }

            // Wait for the next interval, but wake immediately on stop.
            match stop_rx.recv_timeout(check_interval) {
                Ok(()) => {
                    info!("Scheduler loop received stop signal");
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    if !running.load(Ordering::SeqCst) {
                        info!("Scheduler loop stopping (stop signal disconnected)");
                        break;
                    }
                    warn!("Scheduler stop signal disconnected unexpectedly");
                }
            }
        }
    }
}
impl Default for Scheduler {
    fn default() -> Self {
        Self::new().0
    }
}
impl Drop for Scheduler {
    fn drop(&mut self) {
        self.stop();
    }
}
/// Parse a cron expression string into a Cron object.
///
/// # Arguments
/// * `expr` - A cron expression string (e.g., "*/5 * * * *" for every 5 minutes)
///
/// # Returns
/// Returns the parsed Cron object or an error if the expression is invalid.
///
/// # Supported Formats
/// The croner crate supports standard 5-field cron (minute, hour, day, month, weekday)
/// as well as 6-field cron with seconds.
pub fn parse_cron(expr: &str) -> Result<Cron> {
    Cron::from_str(expr).map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", expr, e))
}
/// Convert a natural language schedule to a cron expression.
///
/// # Arguments
/// * `text` - A natural language schedule (e.g., "every tuesday at 2pm")
///
/// # Returns
/// Returns the equivalent cron expression string or an error if parsing fails.
///
/// # Examples
/// - "every minute" -> "* * * * *"
/// - "every tuesday at 2pm" -> "0 14 * * 2"
/// - "every day at 9am" -> "0 9 * * *"
pub fn natural_to_cron(text: &str) -> Result<String> {
    english_to_cron::str_cron_syntax(text)
        .map_err(|e| anyhow::anyhow!("Failed to convert '{}' to cron: {:?}", text, e))
}
/// Find the next occurrence after the given time for a cron schedule.
fn find_next_occurrence<Tz>(cron: &Cron, after: &DateTime<Tz>) -> Result<DateTime<Tz>>
where
    Tz: TimeZone,
    Tz::Offset: Send + Sync,
{
    cron.find_next_occurrence(after, false)
        .map_err(|e| anyhow::anyhow!("Failed to find next occurrence: {:?}", e))
}
/// Find next occurrence using local-time cron semantics in a specific timezone,
/// then convert the result to UTC for scheduler storage/comparison.
fn find_next_occurrence_utc_in_timezone<Tz>(
    cron: &Cron,
    after_utc: &DateTime<Utc>,
    timezone: &Tz,
) -> Result<DateTime<Utc>>
where
    Tz: TimeZone,
    Tz::Offset: Send + Sync,
{
    let localized_after = after_utc.with_timezone(timezone);
    let localized_next = find_next_occurrence(cron, &localized_after)?;
    Ok(localized_next.with_timezone(&Utc))
}

// --- merged from part_001.rs ---
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone, Timelike};
    use std::time::Instant;

    #[test]
    fn test_parse_cron_valid() {
        // Every minute
        let cron = parse_cron("* * * * *");
        assert!(cron.is_ok());

        // Every 5 minutes
        let cron = parse_cron("*/5 * * * *");
        assert!(cron.is_ok());

        // Every hour at minute 0
        let cron = parse_cron("0 * * * *");
        assert!(cron.is_ok());

        // Every day at 9:00 AM
        let cron = parse_cron("0 9 * * *");
        assert!(cron.is_ok());

        // Every Monday at 2:30 PM
        let cron = parse_cron("30 14 * * 1");
        assert!(cron.is_ok());
    }

    #[test]
    fn test_parse_cron_supports_standard_variants() {
        for expr in [
            "*/15 9-17 * * MON-FRI",
            "0 0 1 JAN *",
            "0 0 * * 7",     // 7 == Sunday
            "0 30 14 * * *", // seconds precision
        ] {
            assert!(
                parse_cron(expr).is_ok(),
                "expected cron variant to parse: {expr}"
            );
        }
    }

    #[test]
    fn test_parse_cron_invalid() {
        // Invalid: not enough fields
        let cron = parse_cron("* * *");
        assert!(cron.is_err());

        // Invalid: bad range
        let cron = parse_cron("60 * * * *");
        assert!(cron.is_err());
    }

    #[test]
    fn test_natural_to_cron_basic() {
        // Test basic conversions
        let result = natural_to_cron("every minute");
        assert!(
            result.is_ok(),
            "Failed to parse 'every minute': {:?}",
            result.err()
        );

        let result = natural_to_cron("every hour");
        assert!(
            result.is_ok(),
            "Failed to parse 'every hour': {:?}",
            result.err()
        );
    }

    #[test]
    fn test_natural_to_cron_specific_time() {
        // Test specific time parsing
        let result = natural_to_cron("every day at 9am");
        assert!(
            result.is_ok(),
            "Failed to parse 'every day at 9am': {:?}",
            result.err()
        );

        if let Ok(cron_str) = result {
            // Should contain hour=9
            assert!(
                cron_str.contains("9"),
                "Expected hour 9 in cron: {}",
                cron_str
            );
        }
    }

    #[test]
    fn test_natural_to_cron_weekday() {
        // Test weekday parsing
        let result = natural_to_cron("every tuesday at 2pm");
        assert!(
            result.is_ok(),
            "Failed to parse 'every tuesday at 2pm': {:?}",
            result.err()
        );

        if let Ok(cron_str) = result {
            // Should contain hour=14 (2pm)
            assert!(
                cron_str.contains("14") || cron_str.contains("2"),
                "Expected hour 14 or 2 in cron: {}",
                cron_str
            );
        }
    }

    #[test]
    fn test_natural_to_cron_output_is_parseable_by_croner() {
        let cron = natural_to_cron("every day at 9am").expect("natural language parse");
        assert!(
            parse_cron(&cron).is_ok(),
            "english-to-cron output should parse in croner: {cron}"
        );
    }

    #[test]
    fn test_scheduler_creation() {
        let (scheduler, _rx) = Scheduler::new();
        assert!(scheduler.list_scripts().is_empty());
    }

    #[test]
    fn test_scheduler_add_script_with_cron() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(
            PathBuf::from("/test/script.ts"),
            Some("*/5 * * * *".to_string()),
            None,
        );

        assert!(result.is_ok(), "Failed to add script: {:?}", result.err());

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].path, PathBuf::from("/test/script.ts"));
        assert_eq!(scripts[0].source, ScheduleSource::Cron);
    }

    #[test]
    fn test_scheduler_add_script_with_natural_language() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(
            PathBuf::from("/test/script.ts"),
            None,
            Some("every hour".to_string()),
        );

        assert!(result.is_ok(), "Failed to add script: {:?}", result.err());

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].source, ScheduleSource::NaturalLanguage);
    }

    #[test]
    fn test_scheduler_add_script_cron_takes_precedence() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(
            PathBuf::from("/test/script.ts"),
            Some("0 9 * * *".to_string()),
            Some("every hour".to_string()), // Should be ignored
        );

        assert!(result.is_ok());

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].source, ScheduleSource::Cron);
        assert_eq!(scripts[0].cron_expr, "0 9 * * *");
    }

    #[test]
    fn test_scheduler_add_script_no_schedule() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(PathBuf::from("/test/script.ts"), None, None);

        assert!(result.is_err(), "Should fail when no schedule provided");
    }

    #[test]
    fn test_scheduler_remove_script() {
        let (scheduler, _rx) = Scheduler::new();

        scheduler
            .add_script(
                PathBuf::from("/test/script.ts"),
                Some("* * * * *".to_string()),
                None,
            )
            .unwrap();

        assert_eq!(scheduler.list_scripts().len(), 1);

        let removed = scheduler.remove_script(&PathBuf::from("/test/script.ts"));
        assert!(removed);
        assert!(scheduler.list_scripts().is_empty());
    }

    #[test]
    fn test_scheduler_update_existing_script() {
        let (scheduler, _rx) = Scheduler::new();
        let path = PathBuf::from("/test/script.ts");

        // Add initial script
        scheduler
            .add_script(path.clone(), Some("* * * * *".to_string()), None)
            .unwrap();

        // Update with new schedule
        scheduler
            .add_script(path.clone(), Some("0 9 * * *".to_string()), None)
            .unwrap();

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1); // Should still be 1, not 2
        assert_eq!(scripts[0].cron_expr, "0 9 * * *");
    }

    #[test]
    fn test_scheduler_list_scripts_returns_paths_in_sorted_order() {
        let (scheduler, _rx) = Scheduler::new();

        scheduler
            .add_script(
                PathBuf::from("/test/z-last.ts"),
                Some("* * * * *".to_string()),
                None,
            )
            .unwrap();
        scheduler
            .add_script(
                PathBuf::from("/test/a-first.ts"),
                Some("* * * * *".to_string()),
                None,
            )
            .unwrap();

        let scripts = scheduler.list_scripts();
        let paths: Vec<_> = scripts
            .iter()
            .map(|script| script.path.to_string_lossy().into_owned())
            .collect();

        assert_eq!(paths, vec!["/test/a-first.ts", "/test/z-last.ts"]);
    }

    #[test]
    fn test_scheduler_event_clone() {
        let event = SchedulerEvent::RunScript(PathBuf::from("/test.ts"));
        let _cloned = event.clone();

        let error_event = SchedulerEvent::Error("test error".to_string());
        let _cloned = error_event.clone();
    }

    #[test]
    fn test_schedule_source_equality() {
        assert_eq!(ScheduleSource::Cron, ScheduleSource::Cron);
        assert_eq!(
            ScheduleSource::NaturalLanguage,
            ScheduleSource::NaturalLanguage
        );
        assert_ne!(ScheduleSource::Cron, ScheduleSource::NaturalLanguage);
    }

    #[test]
    fn test_find_next_occurrence() {
        let cron = parse_cron("0 9 * * *").unwrap(); // Every day at 9 AM
        let now = Utc::now();

        let next = find_next_occurrence(&cron, &now);
        assert!(
            next.is_ok(),
            "Failed to find next occurrence: {:?}",
            next.err()
        );

        let next = next.unwrap();
        assert!(next > now, "Next occurrence should be in the future");
    }

    #[test]
    fn test_find_next_occurrence_utc_in_timezone_keeps_local_hour() {
        let cron = parse_cron("0 9 * * *").unwrap();
        let tz = FixedOffset::west_opt(8 * 3600).expect("valid timezone");
        // 2025-01-15 16:00:00Z = 08:00 local in UTC-8.
        let after_utc = Utc
            .with_ymd_and_hms(2025, 1, 15, 16, 0, 0)
            .single()
            .expect("valid timestamp");

        let next_utc =
            find_next_occurrence_utc_in_timezone(&cron, &after_utc, &tz).expect("next run");
        let local_next = next_utc.with_timezone(&tz);

        assert_eq!(local_next.hour(), 9, "should run at 9am local time");
        assert_eq!(local_next.minute(), 0, "should run at minute 0");
    }

    #[test]
    fn test_scheduler_stop_returns_quickly_when_idle() {
        let (mut scheduler, _rx) = Scheduler::new();
        scheduler.start().expect("start scheduler");

        // Give the background thread a moment to enter the wait state.
        thread::sleep(Duration::from_millis(20));

        let start = Instant::now();
        scheduler.stop();
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(500),
            "scheduler stop took too long: {elapsed:?}"
        );
    }
}
