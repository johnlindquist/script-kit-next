//! Stderr log observer: background thread that captures + parses app log lines.
//!
//! The app emits compact logs to stderr when `SCRIPT_KIT_AI_LOG=1`:
//!   `SS.mmm|L|C|message`
//! where L = level (i/w/e/d/t), C = category code.

#![allow(dead_code)]
//!
//! The observer buffers all lines and supports blocking waits for pattern matches.

use std::io::{BufRead, BufReader};
use std::process::ChildStderr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// A parsed log line from the app's stderr.
#[derive(Debug, Clone)]
pub struct LogLine {
    /// Raw line text.
    pub raw: String,
    /// Parsed message portion (after the `SS.mmm|L|C|` prefix), or the full line if unparseable.
    pub message: String,
    /// Level character: i, w, e, d, t (or '?' if unparseable).
    pub level: char,
    /// Category character: A, S, U, etc. (or '?' if unparseable).
    pub category: char,
}

impl LogLine {
    /// Parse a compact log line. Falls back gracefully for non-conforming lines.
    fn parse(raw: String) -> Self {
        // Format: SS.mmm|L|C|message
        // Example: 12.345|i|S|Processing external command type=show
        let parts: Vec<&str> = raw.splitn(4, '|').collect();
        if parts.len() == 4 {
            let level = parts[1].chars().next().unwrap_or('?');
            let category = parts[2].chars().next().unwrap_or('?');
            let message = parts[3].to_string();
            Self {
                raw,
                message,
                level,
                category,
            }
        } else {
            // Not in compact format — keep as-is
            Self {
                message: raw.clone(),
                raw,
                level: '?',
                category: '?',
            }
        }
    }
}

/// Shared state between the reader thread and test code.
struct ObserverState {
    lines: Vec<LogLine>,
    /// Set to true when stderr EOF is reached.
    done: bool,
}

/// Background observer that captures stderr into a searchable buffer.
pub struct LogObserver {
    state: Arc<(Mutex<ObserverState>, Condvar)>,
    /// Join handle for the reader thread.
    _handle: std::thread::JoinHandle<()>,
}

impl LogObserver {
    /// Start observing stderr from a child process.
    ///
    /// Spawns a background thread that reads lines until EOF.
    pub fn new(stderr: ChildStderr) -> Self {
        let state = Arc::new((
            Mutex::new(ObserverState {
                lines: Vec::new(),
                done: false,
            }),
            Condvar::new(),
        ));

        let state_clone = Arc::clone(&state);
        let handle = std::thread::Builder::new()
            .name("integration-test-log-observer".into())
            .spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(text) => {
                            // Also print to test stdout for debugging when tests fail
                            eprintln!("[app] {}", text);

                            let parsed = LogLine::parse(text);
                            let (lock, cvar) = &*state_clone;
                            let mut state = lock.lock().unwrap();
                            state.lines.push(parsed);
                            cvar.notify_all();
                        }
                        Err(_) => break,
                    }
                }
                let (lock, cvar) = &*state_clone;
                let mut state = lock.lock().unwrap();
                state.done = true;
                cvar.notify_all();
            })
            .expect("failed to spawn log observer thread");

        Self {
            state,
            _handle: handle,
        }
    }

    /// Wait for a log line whose message contains `pattern` (substring match).
    ///
    /// Searches existing buffered lines first, then blocks until a match
    /// arrives or `timeout` elapses.
    ///
    /// Returns the matching `LogLine`, or an error on timeout.
    pub fn wait_for_log(&self, pattern: &str, timeout: Duration) -> anyhow::Result<LogLine> {
        let deadline = std::time::Instant::now() + timeout;
        let (lock, cvar) = &*self.state;

        let mut checked_up_to = 0;

        loop {
            let state = lock.lock().unwrap();

            // Check any new lines since last scan
            for line in &state.lines[checked_up_to..] {
                if line.message.contains(pattern) || line.raw.contains(pattern) {
                    return Ok(line.clone());
                }
            }
            checked_up_to = state.lines.len();

            if state.done {
                anyhow::bail!(
                    "stderr closed without matching pattern: {:?}\n\
                     Total lines captured: {}\n\
                     Last 10 lines:\n{}",
                    pattern,
                    state.lines.len(),
                    state
                        .lines
                        .iter()
                        .rev()
                        .take(10)
                        .rev()
                        .map(|l| format!("  {}", l.raw))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }

            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                anyhow::bail!(
                    "timed out after {:?} waiting for log pattern: {:?}\n\
                     Total lines captured: {}\n\
                     Last 10 lines:\n{}",
                    timeout,
                    pattern,
                    state.lines.len(),
                    state
                        .lines
                        .iter()
                        .rev()
                        .take(10)
                        .rev()
                        .map(|l| format!("  {}", l.raw))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }

            // Wait for new lines or EOF
            let _state = cvar.wait_timeout(state, remaining).unwrap().0;
            // Loop will re-lock and re-check
        }
    }

    /// Get all captured lines so far (snapshot).
    pub fn lines(&self) -> Vec<LogLine> {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        state.lines.clone()
    }

    /// Check if stderr has been closed (process exited or pipe broken).
    pub fn is_done(&self) -> bool {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        state.done
    }
}
