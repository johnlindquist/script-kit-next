//! Core types for clipboard history
//!
//! Contains the main data types: ContentType, TimeGroup, and ClipboardEntry.

use chrono::{Datelike, Local, NaiveDate, TimeZone};

/// Content types for clipboard entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Image,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "image" => ContentType::Image,
            _ => ContentType::Text,
        }
    }
}

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

/// A single clipboard history entry (full, includes content)
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: ContentType,
    pub timestamp: i64,
    pub pinned: bool,
    /// OCR text extracted from images (None for text entries or pending OCR)
    #[allow(dead_code)] // Used by downstream subtasks (OCR, UI)
    pub ocr_text: Option<String>,
}

/// Lightweight clipboard entry metadata for list views (no payload)
///
/// This struct contains everything needed for displaying entries in a list
/// without loading the full content (which can be megabytes for images).
/// Use `get_entry_content()` to fetch the full content when needed.
#[derive(Debug, Clone)]
pub struct ClipboardEntryMeta {
    pub id: String,
    pub content_type: ContentType,
    pub timestamp: i64,
    pub pinned: bool,
    /// First 100 chars of text content (for list preview), or "[Image]" for images
    pub text_preview: String,
    /// Image width in pixels (None for text)
    pub image_width: Option<u32>,
    /// Image height in pixels (None for text)
    pub image_height: Option<u32>,
    /// Content size in bytes (useful for displaying file sizes)
    #[allow(dead_code)]
    pub byte_size: usize,
    /// OCR text extracted from images (None for text entries or pending OCR)
    #[allow(dead_code)]
    pub ocr_text: Option<String>,
}

impl ClipboardEntryMeta {
    /// Get a display-friendly preview string for list items
    pub fn display_preview(&self) -> String {
        match self.content_type {
            ContentType::Image => {
                if let (Some(w), Some(h)) = (self.image_width, self.image_height) {
                    format!("{}×{} image", w, h)
                } else {
                    "[Image]".to_string()
                }
            }
            ContentType::Text => {
                // Replace newlines with spaces for single-line display
                let sanitized = self.text_preview.replace(['\n', '\r'], " ");
                if sanitized.len() > 50 {
                    format!("{}…", sanitized.chars().take(50).collect::<String>())
                } else {
                    sanitized
                }
            }
        }
    }
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
    use std::collections::HashMap;

    let mut groups: HashMap<TimeGroup, Vec<ClipboardEntry>> = HashMap::new();

    for entry in entries {
        let group = classify_timestamp(entry.timestamp);
        groups.entry(group).or_default().push(entry);
    }

    // Sort groups by their display order
    let mut result: Vec<(TimeGroup, Vec<ClipboardEntry>)> = groups.into_iter().collect();
    result.sort_by_key(|(group, _)| group.sort_order());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(ContentType::Text.as_str(), "text");
        assert_eq!(ContentType::Image.as_str(), "image");
        assert_eq!(ContentType::from_str("text"), ContentType::Text);
        assert_eq!(ContentType::from_str("image"), ContentType::Image);
        assert_eq!(ContentType::from_str("unknown"), ContentType::Text);
    }

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
            },
            ClipboardEntry {
                id: "2".to_string(),
                content: "yesterday".to_string(),
                content_type: ContentType::Text,
                timestamp: yesterday_ts_ms,
                pinned: false,
                ocr_text: None,
            },
            ClipboardEntry {
                id: "3".to_string(),
                content: "old".to_string(),
                content_type: ContentType::Text,
                timestamp: old_ts_ms,
                pinned: false,
                ocr_text: None,
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
}
