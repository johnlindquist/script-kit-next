//! Shared formatting helpers for human-readable file sizes, relative times, and durations.
//!
//! Consolidates duplicate implementations from across the codebase into a single source of truth.

use chrono::{DateTime, Utc};

/// Format a byte count as a human-readable file size (e.g. "1.5 MB", "456 KB", "12 B").
///
/// Uses binary (1024-based) thresholds with conventional suffixes (KB, MB, GB).
/// Delegates to `humansize` with the BINARY preset and normalises "KiB"→"KB" etc.
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        // Small files: no decimal places, matches existing "X B" output.
        format!("{bytes} B")
    } else {
        // BINARY: powers of 1024 with "KiB", "MiB", "GiB" suffixes.
        // decimal_zeroes(1) keeps trailing ".0" to match legacy "{:.1}" formatting.
        let opts = humansize::FormatSizeOptions::from(humansize::BINARY)
            .decimal_places(1)
            .decimal_zeroes(1);
        // Normalise IEC suffixes to conventional ones: "KiB"→"KB", "MiB"→"MB", etc.
        humansize::format_size(bytes, opts).replace("iB", "B")
    }
}

/// Format a Unix timestamp (seconds since epoch) as a long-form relative time string.
///
/// Output examples: "Just now", "3 mins ago", "1 hour ago", "2 weeks ago".
/// Used in file search results and environment variable prompts.
pub fn format_relative_time_long(unix_timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if unix_timestamp == 0 {
        return "Unknown".to_string();
    }

    let diff = now.saturating_sub(unix_timestamp);
    format_seconds_long(diff)
}

/// Format a `chrono::DateTime<Utc>` as a long-form relative time string.
///
/// Same output style as [`format_relative_time_long`] but accepts a chrono DateTime.
pub fn format_relative_time_long_dt(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(dt);

    let seconds = diff.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }

    format_seconds_long(seconds as u64)
}

/// Format a `chrono::DateTime<Utc>` as a short-form relative time string.
///
/// Output examples: "just now", "5s ago", "3m ago", "2h ago", "1d ago", "Mar 05".
/// Used in notes, clipboard history, and AI chat timestamps.
pub fn format_relative_time_short_dt(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now - dt;

    if diff.num_seconds() < 5 {
        "just now".to_string()
    } else if diff.num_seconds() < 60 {
        format!("{}s ago", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_days() < 7 {
        format!("{}d ago", diff.num_days())
    } else {
        dt.format("%b %d").to_string()
    }
}

/// Format a millisecond timestamp (e.g. clipboard entry timestamps) as a short relative time.
///
/// Output examples: "just now", "3m ago", "2h ago", "5d ago".
#[allow(dead_code)]
pub fn format_relative_time_short_millis(timestamp_ms: i64) -> String {
    let now_ms = Utc::now().timestamp_millis();
    let age_secs = (now_ms - timestamp_ms) / 1000;

    if age_secs < 60 {
        "just now".to_string()
    } else if age_secs < 3600 {
        format!("{}m ago", age_secs / 60)
    } else if age_secs < 86400 {
        format!("{}h ago", age_secs / 3600)
    } else {
        format!("{}d ago", age_secs / 86400)
    }
}

/// Format a `std::time::Duration` as a compact human-readable label.
///
/// Output: "345ms" for sub-second, "5s" for whole seconds.
/// Uses `humantime::format_duration` for consistent output.
pub fn format_duration_compact(dur: std::time::Duration) -> String {
    if dur.as_secs() < 1 {
        humantime::format_duration(std::time::Duration::from_millis(dur.as_millis() as u64))
            .to_string()
    } else {
        humantime::format_duration(std::time::Duration::from_secs(dur.as_secs())).to_string()
    }
}

// ── Internal ──────────────────────────────────────────────────────────────────

/// Shared long-form relative time formatting from a seconds delta.
fn format_seconds_long(diff: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;

    if diff < MINUTE {
        "Just now".to_string()
    } else if diff < HOUR {
        let mins = diff / MINUTE;
        format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < DAY {
        let hours = diff / HOUR;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < WEEK {
        let days = diff / DAY;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < MONTH {
        let weeks = diff / WEEK;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if diff < YEAR {
        let months = diff / MONTH;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = diff / YEAR;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use std::time::Duration;

    // ── format_file_size ──────────────────────────────────────────────────

    #[test]
    fn test_format_file_size_zero() {
        assert_eq!(format_file_size(0), "0 B");
    }

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1023), "1023 B");
    }

    #[test]
    fn test_format_file_size_kilobytes() {
        let result = format_file_size(1024);
        assert!(result.contains("KB"), "expected KB, got: {result}");
    }

    #[test]
    fn test_format_file_size_megabytes() {
        let result = format_file_size(1024 * 1024 * 3);
        assert!(result.contains("MB"), "expected MB, got: {result}");
    }

    #[test]
    fn test_format_file_size_gigabytes() {
        let result = format_file_size(1024 * 1024 * 1024 * 2);
        assert!(result.contains("GB"), "expected GB, got: {result}");
    }

    // ── format_relative_time_long ─────────────────────────────────────────

    #[test]
    fn test_relative_time_long_zero_timestamp() {
        assert_eq!(format_relative_time_long(0), "Unknown");
    }

    #[test]
    fn test_relative_time_long_just_now() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time_long(now), "Just now");
    }

    #[test]
    fn test_relative_time_long_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time_long(now - 120), "2 mins ago");
    }

    #[test]
    fn test_relative_time_long_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time_long(now - 7200), "2 hours ago");
    }

    #[test]
    fn test_relative_time_long_singular() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time_long(now - 60), "1 min ago");
        assert_eq!(format_relative_time_long(now - 3600), "1 hour ago");
    }

    // ── format_relative_time_long_dt ──────────────────────────────────────

    #[test]
    fn test_relative_time_long_dt_future() {
        let future = Utc::now() + TimeDelta::hours(1);
        assert_eq!(format_relative_time_long_dt(future), "just now");
    }

    #[test]
    fn test_relative_time_long_dt_days() {
        let past = Utc::now() - TimeDelta::days(3);
        assert_eq!(format_relative_time_long_dt(past), "3 days ago");
    }

    // ── format_relative_time_short_dt ─────────────────────────────────────

    #[test]
    fn test_relative_time_short_just_now() {
        let now = Utc::now();
        assert_eq!(format_relative_time_short_dt(now), "just now");
    }

    #[test]
    fn test_relative_time_short_seconds() {
        let past = Utc::now() - TimeDelta::seconds(30);
        assert_eq!(format_relative_time_short_dt(past), "30s ago");
    }

    #[test]
    fn test_relative_time_short_minutes() {
        let past = Utc::now() - TimeDelta::minutes(5);
        assert_eq!(format_relative_time_short_dt(past), "5m ago");
    }

    #[test]
    fn test_relative_time_short_hours() {
        let past = Utc::now() - TimeDelta::hours(3);
        assert_eq!(format_relative_time_short_dt(past), "3h ago");
    }

    #[test]
    fn test_relative_time_short_days() {
        let past = Utc::now() - TimeDelta::days(2);
        assert_eq!(format_relative_time_short_dt(past), "2d ago");
    }

    #[test]
    fn test_relative_time_short_old_shows_date() {
        let past = Utc::now() - TimeDelta::days(14);
        let result = format_relative_time_short_dt(past);
        // Should be a date like "Mar 01" not "2w ago"
        assert!(!result.contains("ago"), "expected date, got: {result}");
    }

    // ── format_relative_time_short_millis ─────────────────────────────────

    #[test]
    fn test_relative_time_short_millis_just_now() {
        let now_ms = Utc::now().timestamp_millis();
        assert_eq!(format_relative_time_short_millis(now_ms), "just now");
    }

    #[test]
    fn test_relative_time_short_millis_minutes() {
        let now_ms = Utc::now().timestamp_millis();
        let five_min_ago = now_ms - 5 * 60 * 1000;
        assert_eq!(format_relative_time_short_millis(five_min_ago), "5m ago");
    }

    // ── format_duration_compact ───────────────────────────────────────────

    #[test]
    fn test_duration_compact_millis() {
        let result = format_duration_compact(Duration::from_millis(345));
        assert_eq!(result, "345ms");
    }

    #[test]
    fn test_duration_compact_seconds() {
        let result = format_duration_compact(Duration::from_secs(5));
        assert_eq!(result, "5s");
    }

    #[test]
    fn test_duration_compact_zero() {
        let result = format_duration_compact(Duration::ZERO);
        assert_eq!(result, "0s");
    }
}
