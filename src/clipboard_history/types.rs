//! App-owned clipboard grouping and root-search policy.
//!
//! Core value types live in `sk-clipboard` and remain re-exported here while
//! existing app callers migrate.

use chrono::{Datelike, Local, NaiveDate, TimeZone};
use itertools::Itertools;

#[allow(unused_imports)]
pub use sk_clipboard::{classify_content, ClipboardEntry, ClipboardEntryMeta, ContentType};

/// Time grouping for clipboard entries (like Raycast)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub enum TimeGroup {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    Older,
}

impl TimeGroup {
    /// Get display name for UI labels
    #[allow(dead_code)] // Used by downstream subtasks (UI)
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeGroup::Today => "Today",
            TimeGroup::Yesterday => "Yesterday",
            TimeGroup::ThisWeek => "This Week",
            TimeGroup::LastWeek => "Last Week",
            TimeGroup::ThisMonth => "This Month",
            TimeGroup::Older => "Older",
        }
    }

    /// Order for sorting groups (lower = earlier in list)
    #[allow(dead_code)] // Used by downstream subtasks (UI)
    pub fn sort_order(&self) -> u8 {
        match self {
            TimeGroup::Today => 0,
            TimeGroup::Yesterday => 1,
            TimeGroup::ThisWeek => 2,
            TimeGroup::LastWeek => 3,
            TimeGroup::ThisMonth => 4,
            TimeGroup::Older => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootClipboardHistorySectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub scan_limit: usize,
}

impl Default for RootClipboardHistorySectionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            max_results: 3,
            min_query_chars: 3,
            scan_limit: 200,
        }
    }
}

pub fn root_clipboard_history_query_is_eligible(
    query: &str,
    options: RootClipboardHistorySectionOptions,
) -> bool {
    let query = query.trim();
    options.enabled
        && !query.contains('\n')
        && crate::scripts::search::query_meets_min_query_chars(query, options.min_query_chars)
}

pub fn root_clipboard_entry_is_eligible(entry: &ClipboardEntryMeta) -> bool {
    matches!(
        entry.content_type,
        ContentType::Text | ContentType::Link | ContentType::File | ContentType::Color
    )
}

/// Classify a Unix timestamp (milliseconds) into a TimeGroup using local timezone
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub fn classify_timestamp(timestamp_ms: i64) -> TimeGroup {
    classify_timestamp_with_now(timestamp_ms, Local::now())
}

/// Internal testable version of classify_timestamp that accepts a "now" parameter
/// This avoids DST-related flakiness in tests by allowing fixed reference times
/// Timestamp is in MILLISECONDS (not seconds).
pub fn classify_timestamp_with_now<Tz: chrono::TimeZone>(
    timestamp_ms: i64,
    now: chrono::DateTime<Tz>,
) -> TimeGroup {
    let today = now.date_naive();
    // Convert milliseconds to seconds for chrono (which expects seconds)
    let timestamp_secs = timestamp_ms / 1000;
    let entry_date = match Local.timestamp_opt(timestamp_secs, 0) {
        chrono::LocalResult::Single(dt) => dt.date_naive(),
        _ => return TimeGroup::Older,
    };

    // Check Today
    if entry_date == today {
        return TimeGroup::Today;
    }

    // Check Yesterday
    if let Some(yesterday) = today.pred_opt() {
        if entry_date == yesterday {
            return TimeGroup::Yesterday;
        }
    }

    // Calculate week boundaries
    // Week starts on Monday (ISO 8601)
    let days_since_monday = today.weekday().num_days_from_monday();
    let this_week_start = today - chrono::Duration::days(days_since_monday as i64);
    let last_week_start = this_week_start - chrono::Duration::days(7);

    // Check This Week (not including today/yesterday which are handled above)
    if entry_date >= this_week_start && entry_date < today {
        return TimeGroup::ThisWeek;
    }

    // Check Last Week
    if entry_date >= last_week_start && entry_date < this_week_start {
        return TimeGroup::LastWeek;
    }

    // Check This Month
    let this_month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
    if entry_date >= this_month_start {
        return TimeGroup::ThisMonth;
    }

    TimeGroup::Older
}

/// Group entries by time period
///
/// Returns a vector of (TimeGroup, Vec<ClipboardEntry>) tuples,
/// sorted by time group order (Today first, Older last).
/// Entries within each group maintain their original order.
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub fn group_entries_by_time(
    entries: Vec<ClipboardEntry>,
) -> Vec<(TimeGroup, Vec<ClipboardEntry>)> {
    entries
        .into_iter()
        .into_group_map_by(|entry| classify_timestamp(entry.timestamp))
        .into_iter()
        .sorted_by_key(|(group, _)| group.sort_order())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_group_display_names() {
        assert_eq!(TimeGroup::Today.display_name(), "Today");
        assert_eq!(TimeGroup::Yesterday.display_name(), "Yesterday");
        assert_eq!(TimeGroup::ThisWeek.display_name(), "This Week");
        assert_eq!(TimeGroup::LastWeek.display_name(), "Last Week");
        assert_eq!(TimeGroup::ThisMonth.display_name(), "This Month");
        assert_eq!(TimeGroup::Older.display_name(), "Older");
    }

    #[test]
    fn test_time_group_sort_order() {
        assert!(TimeGroup::Today.sort_order() < TimeGroup::Yesterday.sort_order());
        assert!(TimeGroup::Yesterday.sort_order() < TimeGroup::ThisWeek.sort_order());
        assert!(TimeGroup::ThisWeek.sort_order() < TimeGroup::LastWeek.sort_order());
        assert!(TimeGroup::LastWeek.sort_order() < TimeGroup::ThisMonth.sort_order());
        assert!(TimeGroup::ThisMonth.sort_order() < TimeGroup::Older.sort_order());
    }

    #[test]
    fn test_classify_timestamp_today() {
        // Use a fixed reference date (Wed, Jan 15, 2025 at noon UTC) to avoid DST flakiness
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        // Timestamps are now in milliseconds
        let same_day_timestamp_ms = fixed_now.timestamp_millis();

        assert_eq!(
            classify_timestamp_with_now(same_day_timestamp_ms, fixed_now),
            TimeGroup::Today
        );
    }

    #[test]
    fn test_classify_timestamp_yesterday() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let yesterday_timestamp_ms = chrono::Utc
            .with_ymd_and_hms(2025, 1, 14, 12, 0, 0)
            .unwrap()
            .timestamp_millis();

        assert_eq!(
            classify_timestamp_with_now(yesterday_timestamp_ms, fixed_now),
            TimeGroup::Yesterday
        );
    }

    #[test]
    fn test_classify_timestamp_very_old() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let old_timestamp_ms = chrono::Utc
            .with_ymd_and_hms(2024, 10, 7, 12, 0, 0)
            .unwrap()
            .timestamp_millis();

        assert_eq!(
            classify_timestamp_with_now(old_timestamp_ms, fixed_now),
            TimeGroup::Older
        );
    }

    #[test]
    fn test_classify_timestamp_this_week() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 17, 12, 0, 0).unwrap();
        let this_week_timestamp_ms = chrono::Utc
            .with_ymd_and_hms(2025, 1, 15, 12, 0, 0)
            .unwrap()
            .timestamp_millis();

        assert_eq!(
            classify_timestamp_with_now(this_week_timestamp_ms, fixed_now),
            TimeGroup::ThisWeek
        );
    }

    #[test]
    fn test_classify_timestamp_last_week() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let last_week_timestamp_ms = chrono::Utc
            .with_ymd_and_hms(2025, 1, 8, 12, 0, 0)
            .unwrap()
            .timestamp_millis();

        assert_eq!(
            classify_timestamp_with_now(last_week_timestamp_ms, fixed_now),
            TimeGroup::LastWeek
        );
    }

    #[test]
    fn test_classify_timestamp_this_month() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let this_month_timestamp_ms = chrono::Utc
            .with_ymd_and_hms(2025, 1, 2, 12, 0, 0)
            .unwrap()
            .timestamp_millis();

        assert_eq!(
            classify_timestamp_with_now(this_month_timestamp_ms, fixed_now),
            TimeGroup::ThisMonth
        );
    }

    #[test]
    fn test_group_entries_by_time() {
        // Timestamps are now in milliseconds
        let today_ts_ms = chrono::Utc
            .with_ymd_and_hms(2025, 1, 15, 12, 0, 0)
            .unwrap()
            .timestamp_millis();
        let yesterday_ts_ms = chrono::Utc
            .with_ymd_and_hms(2025, 1, 14, 12, 0, 0)
            .unwrap()
            .timestamp_millis();
        let old_ts_ms = chrono::Utc
            .with_ymd_and_hms(2024, 10, 7, 12, 0, 0)
            .unwrap()
            .timestamp_millis();

        let entries = vec![
            ClipboardEntry {
                id: "1".to_string(),
                content: "today".to_string(),
                content_type: ContentType::Text,
                timestamp: today_ts_ms,
                pinned: false,
                ocr_text: None,
                source_app_name: None,
                source_app_bundle_id: None,
            },
            ClipboardEntry {
                id: "2".to_string(),
                content: "yesterday".to_string(),
                content_type: ContentType::Text,
                timestamp: yesterday_ts_ms,
                pinned: false,
                ocr_text: None,
                source_app_name: None,
                source_app_bundle_id: None,
            },
            ClipboardEntry {
                id: "3".to_string(),
                content: "old".to_string(),
                content_type: ContentType::Text,
                timestamp: old_ts_ms,
                pinned: false,
                ocr_text: None,
                source_app_name: None,
                source_app_bundle_id: None,
            },
        ];

        let grouped = group_entries_by_time(entries);

        assert!(!grouped.is_empty(), "Should have at least one group");

        for i in 1..grouped.len() {
            assert!(
                grouped[i - 1].0.sort_order() <= grouped[i].0.sort_order(),
                "Groups should be sorted by sort_order"
            );
        }

        let total_entries: usize = grouped.iter().map(|(_, entries)| entries.len()).sum();
        assert_eq!(total_entries, 3, "All entries should be grouped");
    }

    #[test]
    fn clipboard_history_root_query_requires_opt_in_and_minimum_length() {
        let options = RootClipboardHistorySectionOptions {
            enabled: true,
            max_results: 3,
            min_query_chars: 3,
            scan_limit: 200,
        };

        assert!(root_clipboard_history_query_is_eligible("fix", options));
        assert!(!root_clipboard_history_query_is_eligible("fi", options));
        assert!(!root_clipboard_history_query_is_eligible(
            "fix\ncase",
            options
        ));
        assert!(!root_clipboard_history_query_is_eligible(
            "fix",
            RootClipboardHistorySectionOptions {
                enabled: false,
                ..options
            }
        ));
    }

    #[test]
    fn clipboard_history_root_rows_exclude_images() {
        let mut entry = ClipboardEntryMeta {
            id: "clip-1".to_string(),
            content_type: ContentType::Text,
            timestamp: 1_778_000_000_000,
            pinned: false,
            text_preview: "fix spelling".to_string(),
            image_width: None,
            image_height: None,
            byte_size: 12,
            ocr_text: None,
        };

        assert!(root_clipboard_entry_is_eligible(&entry));
        entry.content_type = ContentType::Image;
        assert!(!root_clipboard_entry_is_eligible(&entry));
    }
}
