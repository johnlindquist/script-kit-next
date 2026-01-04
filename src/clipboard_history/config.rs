//! Clipboard history configuration
//!
//! Retention settings and text length limits.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Default retention period in days (entries older than this are pruned)
pub const DEFAULT_RETENTION_DAYS: u32 = 30;

/// Default maximum number of bytes allowed for text clipboard entries.
pub const DEFAULT_MAX_TEXT_CONTENT_LEN: usize = 100_000;

/// Configured retention days (loaded from config, defaults to DEFAULT_RETENTION_DAYS)
static RETENTION_DAYS: OnceLock<u32> = OnceLock::new();

/// Configured maximum text entry length (bytes). usize::MAX means no limit.
static MAX_TEXT_CONTENT_LEN: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_TEXT_CONTENT_LEN);

/// Get the configured retention period in days
pub fn get_retention_days() -> u32 {
    *RETENTION_DAYS.get().unwrap_or(&DEFAULT_RETENTION_DAYS)
}

/// Get the configured max text length (bytes).
pub fn get_max_text_content_len() -> usize {
    MAX_TEXT_CONTENT_LEN.load(Ordering::Relaxed)
}

/// Set the retention period (call before init_clipboard_history)
#[allow(dead_code)] // Used by downstream subtasks (config)
pub fn set_retention_days(days: u32) {
    let _ = RETENTION_DAYS.set(days);
}

/// Set the max text length (bytes). Use 0 to disable the limit.
pub fn set_max_text_content_len(max_len: usize) {
    let value = if max_len == 0 { usize::MAX } else { max_len };
    MAX_TEXT_CONTENT_LEN.store(value, Ordering::Relaxed);
}

/// Check if text exceeds the configured limit
pub fn is_text_over_limit(text: &str) -> bool {
    text.len() > get_max_text_content_len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_days_default() {
        assert_eq!(DEFAULT_RETENTION_DAYS, 30);
    }

    #[test]
    fn test_text_length_limit() {
        let ok_text = "a".repeat(DEFAULT_MAX_TEXT_CONTENT_LEN);
        assert!(!is_text_over_limit(&ok_text));

        let long_text = "a".repeat(DEFAULT_MAX_TEXT_CONTENT_LEN + 1);
        assert!(is_text_over_limit(&long_text));
    }
}
