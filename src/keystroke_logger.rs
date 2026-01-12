//! Debounced keystroke logging for text expansion system
//!
//! This module provides consolidated logging for keystroke events to avoid
//! flooding logs with per-keystroke messages. Instead of logging every
//! keystroke, it accumulates events and flushes summaries periodically.

use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// How often to flush keystroke summaries (in seconds)
const FLUSH_INTERVAL_SECS: u64 = 5;

/// Accumulated keystroke statistics
#[derive(Debug, Default)]
struct KeystrokeStats {
    /// Total keystrokes since last flush
    keystroke_count: usize,
    /// Keystrokes that were skipped (modifier held)
    skipped_count: usize,
    /// Buffer-clearing characters (enter, escape, tab)
    clear_count: usize,
    /// Last few characters typed (for context, not full buffer)
    recent_chars: String,
    /// Current buffer length
    buffer_len: usize,
    /// Number of trigger checks performed
    trigger_checks: usize,
    /// Number of registered triggers
    trigger_count: usize,
}

impl KeystrokeStats {
    fn reset(&mut self) {
        self.keystroke_count = 0;
        self.skipped_count = 0;
        self.clear_count = 0;
        self.recent_chars.clear();
        self.trigger_checks = 0;
        // Don't reset buffer_len or trigger_count - those are current state
    }

    fn is_empty(&self) -> bool {
        self.keystroke_count == 0
    }
}

/// Thread-safe debounced keystroke logger
pub struct KeystrokeLogger {
    stats: Mutex<KeystrokeStats>,
    last_flush: Mutex<Instant>,
}

impl Default for KeystrokeLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl KeystrokeLogger {
    /// Create a new keystroke logger
    pub fn new() -> Self {
        Self {
            stats: Mutex::new(KeystrokeStats::default()),
            last_flush: Mutex::new(Instant::now()),
        }
    }

    /// Record a keystroke event
    pub fn record_keystroke(&self, c: char) {
        let mut stats = self.stats.lock().unwrap();
        stats.keystroke_count += 1;

        // Keep only last 10 chars for context
        if stats.recent_chars.len() >= 10 {
            stats.recent_chars.remove(0);
        }
        // Only add printable chars to recent_chars
        if c.is_ascii_graphic() || c == ' ' {
            stats.recent_chars.push(c);
        }

        drop(stats);
        self.maybe_flush();
    }

    /// Record a skipped keystroke (due to modifier key)
    pub fn record_skipped(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.skipped_count += 1;
        drop(stats);
        self.maybe_flush();
    }

    /// Record a buffer-clearing character
    pub fn record_buffer_clear(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.clear_count += 1;
        drop(stats);
        self.maybe_flush();
    }

    /// Update current buffer state
    pub fn update_buffer_state(&self, buffer_len: usize, trigger_count: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.buffer_len = buffer_len;
        stats.trigger_count = trigger_count;
        stats.trigger_checks += 1;
    }

    /// Log a trigger match immediately (important event)
    pub fn log_match(&self, trigger: &str, chars_to_delete: usize) {
        // Flush any pending stats first
        self.flush();

        info!(
            category = "KEYWORD",
            trigger = %trigger,
            chars_to_delete = chars_to_delete,
            "Trigger matched, performing expansion"
        );
    }

    /// Check if it's time to flush and do so if needed
    fn maybe_flush(&self) {
        let should_flush = {
            let last_flush = self.last_flush.lock().unwrap();
            last_flush.elapsed() >= Duration::from_secs(FLUSH_INTERVAL_SECS)
        };

        if should_flush {
            self.flush();
        }
    }

    /// Force flush accumulated stats to log
    pub fn flush(&self) {
        let mut stats = self.stats.lock().unwrap();

        if stats.is_empty() {
            return;
        }

        // Log consolidated summary
        debug!(
            category = "KEYWORD",
            keystrokes = stats.keystroke_count,
            skipped = stats.skipped_count,
            buffer_clears = stats.clear_count,
            buffer_len = stats.buffer_len,
            trigger_count = stats.trigger_count,
            recent = %stats.recent_chars,
            "Keystroke summary ({}s window)",
            FLUSH_INTERVAL_SECS
        );

        stats.reset();

        // Update last flush time
        let mut last_flush = self.last_flush.lock().unwrap();
        *last_flush = Instant::now();
    }
}

// Global singleton for the keystroke logger
use std::sync::OnceLock;

static KEYSTROKE_LOGGER: OnceLock<KeystrokeLogger> = OnceLock::new();

/// Get the global keystroke logger instance
pub fn keystroke_logger() -> &'static KeystrokeLogger {
    KEYSTROKE_LOGGER.get_or_init(KeystrokeLogger::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_logger() {
        let logger = KeystrokeLogger::new();
        let stats = logger.stats.lock().unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_record_keystroke() {
        let logger = KeystrokeLogger::new();
        logger.record_keystroke('a');
        logger.record_keystroke('b');
        logger.record_keystroke('c');

        let stats = logger.stats.lock().unwrap();
        assert_eq!(stats.keystroke_count, 3);
        assert_eq!(stats.recent_chars, "abc");
    }

    #[test]
    fn test_record_skipped() {
        let logger = KeystrokeLogger::new();
        logger.record_skipped();
        logger.record_skipped();

        let stats = logger.stats.lock().unwrap();
        assert_eq!(stats.skipped_count, 2);
    }

    #[test]
    fn test_record_buffer_clear() {
        let logger = KeystrokeLogger::new();
        logger.record_buffer_clear();

        let stats = logger.stats.lock().unwrap();
        assert_eq!(stats.clear_count, 1);
    }

    #[test]
    fn test_recent_chars_limit() {
        let logger = KeystrokeLogger::new();
        for c in "abcdefghijklmnop".chars() {
            logger.record_keystroke(c);
        }

        let stats = logger.stats.lock().unwrap();
        assert_eq!(stats.recent_chars.len(), 10);
        // Should keep last 10
        assert_eq!(stats.recent_chars, "ghijklmnop");
    }

    #[test]
    fn test_flush_resets_stats() {
        let logger = KeystrokeLogger::new();
        logger.record_keystroke('a');
        logger.record_skipped();
        logger.record_buffer_clear();

        logger.flush();

        let stats = logger.stats.lock().unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_update_buffer_state() {
        let logger = KeystrokeLogger::new();
        logger.update_buffer_state(5, 10);

        let stats = logger.stats.lock().unwrap();
        assert_eq!(stats.buffer_len, 5);
        assert_eq!(stats.trigger_count, 10);
        assert_eq!(stats.trigger_checks, 1);
    }
}
