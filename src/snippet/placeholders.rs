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

/// Expands simple built-in snippet placeholder tokens in `text`.
///
/// Supported `{token}` values and output formats:
/// - `{clipboard}`: current clipboard text (or empty string if unavailable)
/// - `{date}`: local date formatted as `%Y-%m-%d` (example: `2026-02-07`)
/// - `{time}`: local time formatted as `%H:%M:%S` (example: `23:59:58`)
/// - `{datetime}`: local date/time formatted as `%Y-%m-%d %H:%M:%S`
///   (example: `2026-02-07 23:59:58`)
/// - `{uuid}`: newly generated UUID v4 string
///   (example: `123e4567-e89b-42d3-a456-426614174000`)
///
/// Example:
/// - Before: `Log {datetime} | id={uuid} | clip={clipboard}`
/// - After: `Log 2026-02-07 23:59:58 | id=123e4567-e89b-42d3-a456-426614174000 | clip=hello`
///
/// This `{token}` system is separate from template variables like `${var}` and `{{var}}`.
/// Use `expand_placeholders()` only for the fixed built-ins above; use template-variable
/// expansion for named variables provided by template data.
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
