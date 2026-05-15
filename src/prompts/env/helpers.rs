use super::*;

/// Format a DateTime as relative time (e.g., "2 hours ago", "3 days ago")
pub(super) fn format_relative_time(dt: DateTime<Utc>) -> String {
    crate::formatting::format_relative_time_long_dt(dt)
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

pub(super) fn env_storage_hint_text(
    secret: bool,
    store_error: Option<&SecretStoreError>,
) -> String {
    if let Some(error) = store_error {
        error.user_message().to_string()
    } else if secret {
        "Stored securely in ~/.scriptkit/secrets.age".to_string()
    } else {
        "Value is provided to the script for this run only".to_string()
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
