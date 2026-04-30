//! Window-based per-key rate limiter for log sites that might emit
//! untrusted values at high frequency.
//!
//! Oracle-Session `logging-observability-next-pass` PR1 (C): the existing
//! `protocol_stats::should_log_occurrence(total)` gate is occurrence-count
//! based (1st, 100th, 200th, …). It does not defend against same-key
//! bursts inside a short wall-clock window — e.g. a stuck automation
//! client looping `triggerBuiltin unknownName` 500×/sec still produces
//! 5 warn lines per second once the counter crosses 100. This module
//! adds a complementary time-window gate keyed on
//! `(category, key.len(), hash(key))` so log spam from one hostile
//! payload is bounded, without ever storing the raw string.
//!
//! Usage:
//!
//! ```ignore
//! let rate = logging::log_rate_limit("trigger_builtin_unknown", name);
//! if !rate.emit {
//!     return;
//! }
//! tracing::warn!(
//!     suppressed = rate.suppressed,
//!     /* … */
//! );
//! ```

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Emit at most once per `category+key` inside this wall-clock window.
pub const LOG_RL_WINDOW: Duration = Duration::from_secs(30);

/// Buckets older than this are treated as expired during GC.
pub const LOG_RL_STALE_AFTER: Duration = Duration::from_secs(120);

/// Soft cap on retained buckets. When exceeded we first prune stale
/// entries; if still over, we clear the whole map rather than let it
/// grow unbounded.
pub const LOG_RL_MAX_KEYS: usize = 2048;

/// Decision returned by [`log_rate_limit`] — the caller should log iff
/// `emit` is `true`, and should include `suppressed` as a structured
/// field so operators can see bursts even when individual entries drop.
#[derive(Debug, Clone, Copy)]
pub struct LogRateDecision {
    /// `true` → the caller should emit the log line.
    pub emit: bool,
    /// Number of previously-suppressed events on this key since the
    /// last emit (reset to zero when `emit` is `true`).
    pub suppressed: u64,
}

#[derive(Debug, Clone, Copy)]
struct LogBucket {
    last_emit: Instant,
    suppressed: u64,
}

/// Composite key: `(category, key.len(), hash(key))`. We intentionally
/// never store the raw untrusted key — the hash+len collision-resistance
/// is adequate for telemetry and avoids turning the rate limiter into a
/// second log-spam memory sink.
type RateLimitKey = (&'static str, usize, u64);

static LOG_RATE_LIMITS: OnceLock<Mutex<HashMap<RateLimitKey, LogBucket>>> = OnceLock::new();

fn hash_key(key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

/// Decide whether a log entry for `(category, key)` should be emitted.
///
/// `category` is a `&'static str` so the caller discriminates on compile-
/// time constants — there is no caller-facing interning. `key` is the
/// untrusted value; it is hashed + length-summarized, never stored.
pub fn log_rate_limit(category: &'static str, key: &str) -> LogRateDecision {
    decide(category, key, Instant::now())
}

fn decide(category: &'static str, key: &str, now: Instant) -> LogRateDecision {
    let id: RateLimitKey = (category, key.len(), hash_key(key));
    let limits = LOG_RATE_LIMITS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = limits
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if map.len() > LOG_RL_MAX_KEYS {
        map.retain(|_, bucket| now.duration_since(bucket.last_emit) <= LOG_RL_STALE_AFTER);
        if map.len() > LOG_RL_MAX_KEYS {
            map.clear();
        }
    }

    match map.get_mut(&id) {
        Some(bucket) if now.duration_since(bucket.last_emit) < LOG_RL_WINDOW => {
            bucket.suppressed = bucket.suppressed.saturating_add(1);
            LogRateDecision {
                emit: false,
                suppressed: 0,
            }
        }
        Some(bucket) => {
            let suppressed = bucket.suppressed;
            bucket.last_emit = now;
            bucket.suppressed = 0;
            LogRateDecision {
                emit: true,
                suppressed,
            }
        }
        None => {
            map.insert(
                id,
                LogBucket {
                    last_emit: now,
                    suppressed: 0,
                },
            );
            LogRateDecision {
                emit: true,
                suppressed: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_emission_passes_and_reports_zero_suppressed() {
        let now = Instant::now();
        let decision = decide("test_first_emit", "abc", now);
        assert!(decision.emit);
        assert_eq!(decision.suppressed, 0);
    }

    #[test]
    fn repeat_inside_window_is_suppressed() {
        let t0 = Instant::now();
        assert!(decide("test_repeat", "same", t0).emit);

        let t1 = t0 + Duration::from_secs(1);
        let second = decide("test_repeat", "same", t1);
        assert!(!second.emit);
        assert_eq!(
            second.suppressed, 0,
            "suppressed is always 0 for dropped entries"
        );
    }

    #[test]
    fn second_emit_after_window_reports_suppressed_count() {
        let t0 = Instant::now();
        assert!(decide("test_window", "burst", t0).emit);

        // Ten suppressed attempts inside the window.
        for i in 1..=10 {
            let drop = decide("test_window", "burst", t0 + Duration::from_secs(i));
            assert!(!drop.emit);
        }

        // Past the window — emits with suppressed = 10.
        let after = decide(
            "test_window",
            "burst",
            t0 + LOG_RL_WINDOW + Duration::from_secs(1),
        );
        assert!(after.emit);
        assert_eq!(after.suppressed, 10);
    }

    #[test]
    fn different_keys_do_not_block_each_other() {
        let now = Instant::now();
        assert!(decide("test_multi", "one", now).emit);
        assert!(decide("test_multi", "two", now).emit);
        assert!(decide("test_multi", "three", now).emit);
    }

    #[test]
    fn different_categories_do_not_block_each_other() {
        let now = Instant::now();
        assert!(decide("test_catA", "same_key", now).emit);
        assert!(decide("test_catB", "same_key", now).emit);
    }

    #[test]
    fn same_string_same_category_is_deduped() {
        let t0 = Instant::now();
        assert!(decide("test_dedupe", "twice", t0).emit);
        assert!(
            !decide("test_dedupe", "twice", t0 + Duration::from_millis(1)).emit,
            "same key within window must be suppressed"
        );
    }
}
