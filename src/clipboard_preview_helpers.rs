use crate::clipboard_history;

pub(crate) fn content_type_label(content_type: &clipboard_history::ContentType) -> &'static str {
    match content_type {
        clipboard_history::ContentType::Text => "Text",
        clipboard_history::ContentType::Link => "Link",
        clipboard_history::ContentType::File => "File",
        clipboard_history::ContentType::Color => "Color",
        clipboard_history::ContentType::Image => "Image",
    }
}

pub(crate) fn relative_time(now_ts: i64, entry_ts: i64) -> String {
    let age_secs = now_ts.saturating_sub(entry_ts);
    if age_secs < 60 {
        "just now".to_string()
    } else if age_secs < 3600 {
        format!("{} minutes ago", age_secs / 60)
    } else if age_secs < 86400 {
        format!("{} hours ago", age_secs / 3600)
    } else {
        format!("{} days ago", age_secs / 86400)
    }
}

pub(crate) fn absolute_time(entry_ts: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(entry_ts, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "unknown time".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_labels_are_specific() {
        assert_eq!(
            content_type_label(&clipboard_history::ContentType::Link),
            "Link"
        );
        assert_eq!(
            content_type_label(&clipboard_history::ContentType::Color),
            "Color"
        );
        assert_eq!(
            content_type_label(&clipboard_history::ContentType::File),
            "File"
        );
        assert_eq!(
            content_type_label(&clipboard_history::ContentType::Image),
            "Image"
        );
        assert_eq!(
            content_type_label(&clipboard_history::ContentType::Text),
            "Text"
        );
    }

    #[test]
    fn relative_time_uses_human_scale() {
        assert_eq!(relative_time(1_000, 995), "just now");
        assert_eq!(relative_time(4_000, 3_880), "2 minutes ago");
        assert_eq!(relative_time(10_000, 2_800), "2 hours ago");
        assert_eq!(relative_time(200_000, 100_000), "1 days ago");
    }

    #[test]
    fn relative_time_handles_equal_timestamps() {
        assert_eq!(relative_time(1_000, 1_000), "just now");
    }

    #[test]
    fn relative_time_handles_future_entry() {
        // saturating_sub prevents underflow
        assert_eq!(relative_time(100, 200), "just now");
    }

    #[test]
    fn absolute_time_formats_valid_timestamp() {
        let result = absolute_time(0);
        // Should produce a date string, not "unknown time"
        assert_ne!(result, "unknown time");
        assert!(result.contains("-"), "expected date format, got: {result}");
    }

    #[test]
    fn absolute_time_handles_invalid_timestamp() {
        // Very negative timestamp — chrono should still handle it or return None
        let result = absolute_time(i64::MIN);
        // Either a valid date or "unknown time" — both are acceptable
        assert!(
            result == "unknown time" || result.contains("-"),
            "unexpected result: {result}"
        );
    }
}
