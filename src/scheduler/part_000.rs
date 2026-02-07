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
