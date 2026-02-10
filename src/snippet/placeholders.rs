#[allow(dead_code)]
use arboard::Clipboard;
#[allow(dead_code)]
use chrono::Local;
#[allow(dead_code)]
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SnippetPlaceholderValues {
    clipboard: String,
    date: String,
    time: String,
    datetime: String,
    uuid: String,
}

/// Expands supported snippet placeholders in `text`.
///
/// Supported placeholders:
/// - `{clipboard}`
/// - `{date}` as `%Y-%m-%d`
/// - `{time}` as `%H:%M:%S`
/// - `{datetime}` as `%Y-%m-%d %H:%M:%S`
/// - `{uuid}` as a v4 UUID string
#[allow(dead_code)]
pub fn expand_placeholders(text: &str) -> String {
    let now = Local::now();
    let values = SnippetPlaceholderValues {
        clipboard: read_clipboard_text(),
        date: now.format("%Y-%m-%d").to_string(),
        time: now.format("%H:%M:%S").to_string(),
        datetime: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        uuid: Uuid::new_v4().to_string(),
    };

    expand_placeholders_with_values(text, &values)
}

#[allow(dead_code)]
fn read_clipboard_text() -> String {
    Clipboard::new()
        .and_then(|mut clipboard| clipboard.get_text())
        .unwrap_or_default()
}

#[allow(dead_code)]
fn expand_placeholders_with_values(text: &str, values: &SnippetPlaceholderValues) -> String {
    text.replace("{clipboard}", &values.clipboard)
        .replace("{datetime}", &values.datetime)
        .replace("{date}", &values.date)
        .replace("{time}", &values.time)
        .replace("{uuid}", &values.uuid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_values() -> SnippetPlaceholderValues {
        SnippetPlaceholderValues {
            clipboard: "clipboard-text".to_string(),
            date: "2026-02-07".to_string(),
            time: "23:59:58".to_string(),
            datetime: "2026-02-07 23:59:58".to_string(),
            uuid: "123e4567-e89b-42d3-a456-426614174000".to_string(),
        }
    }

    #[test]
    fn test_expand_placeholders_replaces_clipboard_when_token_present() {
        let output = expand_placeholders_with_values("before {clipboard} after", &test_values());
        assert_eq!(output, "before clipboard-text after");
    }

    #[test]
    fn test_expand_placeholders_replaces_date_when_token_present() {
        let output = expand_placeholders_with_values("{date}", &test_values());
        assert_eq!(output, "2026-02-07");
    }

    #[test]
    fn test_expand_placeholders_replaces_time_when_token_present() {
        let output = expand_placeholders_with_values("{time}", &test_values());
        assert_eq!(output, "23:59:58");
    }

    #[test]
    fn test_expand_placeholders_replaces_datetime_when_token_present() {
        let output = expand_placeholders_with_values("{datetime}", &test_values());
        assert_eq!(output, "2026-02-07 23:59:58");
    }

    #[test]
    fn test_expand_placeholders_replaces_uuid_when_token_present() {
        let output = expand_placeholders_with_values("{uuid}", &test_values());
        assert_eq!(output, "123e4567-e89b-42d3-a456-426614174000");
    }

    #[test]
    fn test_expand_placeholders_replaces_all_tokens_when_all_present() {
        let output = expand_placeholders_with_values(
            "{clipboard} {date} {time} {datetime} {uuid}",
            &test_values(),
        );

        assert_eq!(
            output,
            "clipboard-text 2026-02-07 23:59:58 2026-02-07 23:59:58 123e4567-e89b-42d3-a456-426614174000"
        );
    }

    #[test]
    fn test_expand_placeholders_keeps_plain_text_when_no_tokens_present() {
        let output = expand_placeholders_with_values("plain text", &test_values());
        assert_eq!(output, "plain text");
    }
}
