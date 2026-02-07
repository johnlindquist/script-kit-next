use super::*;

/// Format a DateTime as relative time (e.g., "2 hours ago", "3 days ago")
pub(super) fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(dt);

    let seconds = diff.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }
    let seconds = seconds as u64;

    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;

    if seconds < MINUTE {
        "just now".to_string()
    } else if seconds < HOUR {
        let mins = seconds / MINUTE;
        format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if seconds < DAY {
        let hours = seconds / HOUR;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if seconds < WEEK {
        let days = seconds / DAY;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if seconds < MONTH {
        let weeks = seconds / WEEK;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if seconds < YEAR {
        let months = seconds / MONTH;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = seconds / YEAR;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

pub(super) fn env_input_placeholder(key: &str, exists_in_keyring: bool) -> String {
    if exists_in_keyring {
        format!("Paste a replacement value for {key}")
    } else {
        format!("Paste value for {key}")
    }
}

pub(super) fn env_default_description(key: &str, exists_in_keyring: bool) -> String {
    if exists_in_keyring {
        format!("Update the saved value for {key}")
    } else {
        format!("Enter the value for {key}")
    }
}

pub(super) fn env_running_status(key: &str) -> String {
    format!("Script is running and waiting for {key}")
}

pub(super) fn env_input_label(secret: bool) -> &'static str {
    if secret {
        "Secret value"
    } else {
        "Value"
    }
}

pub(super) fn masked_secret_value_for_display(value: &str) -> String {
    "•".repeat(value.chars().count())
}

pub(super) fn env_storage_hint_text(secret: bool) -> &'static str {
    if secret {
        "Stored securely in ~/.scriptkit/secrets.age"
    } else {
        "Value is provided to the script for this run only"
    }
}

pub(super) fn env_submit_validation_error(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        Some("Value cannot be empty")
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnvKeyAction {
    Submit,
    Cancel,
}

#[inline]
pub(super) fn env_key_action(key: &str) -> Option<EnvKeyAction> {
    if is_key_enter(key) {
        return Some(EnvKeyAction::Submit);
    }
    if is_key_escape(key) {
        return Some(EnvKeyAction::Cancel);
    }
    None
}

pub(super) fn env_prompt_correlation_id(id: &str, key: &str) -> String {
    format!("env_prompt:{id}:{key}")
}
