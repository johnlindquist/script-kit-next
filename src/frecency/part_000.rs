use crate::config::{SuggestedConfig, DEFAULT_SUGGESTED_HALF_LIFE_DAYS};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, instrument, warn};
/// Re-export for tests that need the half-life constant
#[allow(dead_code)]
pub const HALF_LIFE_DAYS: f64 = DEFAULT_SUGGESTED_HALF_LIFE_DAYS;
/// Seconds in a day for timestamp calculations
const SECONDS_PER_DAY: f64 = 86400.0;
/// A single frecency entry tracking usage of a script
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrecencyEntry {
    /// Number of times this script has been used
    pub count: u32,
    /// Unix timestamp (seconds) of last use
    pub last_used: u64,
    /// Cached score (recalculated on load, not persisted)
    /// This is a derived value computed from count, last_used, and half_life.
    /// We skip serializing it because it's always recalculated on load.
    #[serde(skip_serializing, default)]
    pub score: f64,
}
impl FrecencyEntry {
    /// Create a new entry with initial use
    pub fn new() -> Self {
        let now = current_timestamp();
        FrecencyEntry {
            count: 1,
            last_used: now,
            score: 1.0, // Initial score is just the count (no decay yet)
        }
    }

    /// Calculate the decay factor for a given time elapsed
    fn decay_factor(seconds_elapsed: u64, half_life_days: f64) -> f64 {
        // Guard against nonsense config (zero or negative half-life)
        let hl = half_life_days.max(0.001);
        let days_elapsed = seconds_elapsed as f64 / SECONDS_PER_DAY;
        // True half-life decay: 2^(-days/hl) == e^(-ln(2) * days/hl)
        (-std::f64::consts::LN_2 * days_elapsed / hl).exp()
    }

    /// Compute the score at a given timestamp (live computation with decay)
    ///
    /// This decays the stored score by the time elapsed since last_used.
    /// Use this for ranking to avoid stale scores.
    pub fn score_at(&self, now: u64, half_life_days: f64) -> f64 {
        let dt = now.saturating_sub(self.last_used);
        self.score * Self::decay_factor(dt, half_life_days)
    }

    /// Record a new use with explicit timestamp (incremental frecency model)
    ///
    /// Uses the incremental model: new_score = old_score * decay(elapsed_time) + 1
    /// This prevents "rich get richer" by decaying historical usage.
    pub fn record_use_with_timestamp(&mut self, now: u64, half_life_days: f64) {
        // Compute current score with decay
        let current_score = self.score_at(now, half_life_days);
        // Add 1 for this new use
        self.score = current_score + 1.0;
        self.last_used = now;
        self.count = self.count.saturating_add(1);
    }

    /// Record a new use of this script using the default half-life
    ///
    /// NOTE: Prefer using FrecencyStore::record_use() which uses the store's
    /// configured half-life instead of the default.
    #[allow(dead_code)]
    pub fn record_use(&mut self) {
        self.record_use_with_timestamp(current_timestamp(), DEFAULT_SUGGESTED_HALF_LIFE_DAYS);
    }

    /// Recalculate the frecency score based on current time using default half-life
    ///
    /// NOTE: Prefer using recalculate_score_with_half_life() with the store's
    /// configured half-life.
    #[allow(dead_code)]
    pub fn recalculate_score(&mut self) {
        self.score = calculate_score(self.count, self.last_used, DEFAULT_SUGGESTED_HALF_LIFE_DAYS);
    }

    /// Recalculate the frecency score with a custom half-life
    pub fn recalculate_score_with_half_life(&mut self, half_life_days: f64) {
        self.score = calculate_score(self.count, self.last_used, half_life_days);
    }
}
impl Default for FrecencyEntry {
    fn default() -> Self {
        Self::new()
    }
}
/// Calculate frecency score using exponential decay with true half-life
///
/// Formula: score = count * 2^(-days_since_use / half_life_days)
///        = count * e^(-ln(2) * days_since_use / half_life_days)
///
/// This means (with default 7-day half-life):
/// - After 7 days (half_life), the score is reduced to exactly 50%
/// - After 14 days, the score is reduced to exactly 25%
/// - After 21 days, the score is reduced to exactly 12.5%
///
/// With a shorter half-life (e.g., 1 day), recent items dominate.
/// With a longer half-life (e.g., 30 days), frequently used items dominate.
fn calculate_score(count: u32, last_used: u64, half_life_days: f64) -> f64 {
    let now = current_timestamp();
    let seconds_since_use = now.saturating_sub(last_used);
    let days_since_use = seconds_since_use as f64 / SECONDS_PER_DAY;

    // Guard against nonsense config (zero or negative half-life)
    let hl = half_life_days.max(0.001);

    // True half-life decay: 2^(-days/hl) == e^(-ln(2) * days/hl)
    // At days == hl: decay_factor = 2^(-1) = 0.5 (exactly 50%)
    let decay_factor = (-std::f64::consts::LN_2 * days_since_use / hl).exp();
    count as f64 * decay_factor
}
/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
/// Store for frecency data with persistence
///
/// NOTE: Clone is intentionally NOT derived to prevent accidental data loss
/// in multi-window contexts. If you need to share FrecencyStore across
/// multiple owners, use `Arc<Mutex<FrecencyStore>>` explicitly.
#[derive(Debug)]
pub struct FrecencyStore {
    /// Map of script path to frecency entry
    entries: HashMap<String, FrecencyEntry>,
    /// Path to the frecency data file
    file_path: PathBuf,
    /// Whether there are unsaved changes
    dirty: bool,
    /// Half-life in days for score decay (from config)
    half_life_days: f64,
    /// Revision counter for cache invalidation
    /// Incremented on any change affecting ranking
    revision: u64,
}
/// Raw data format for JSON serialization (owned, for deserialization)
#[derive(Debug, Serialize, Deserialize)]
struct FrecencyData {
    entries: HashMap<String, FrecencyEntry>,
}
/// Raw data format for JSON serialization (borrowed, for serialization without cloning)
#[derive(Serialize)]
struct FrecencyDataRef<'a> {
    entries: &'a HashMap<String, FrecencyEntry>,
}
