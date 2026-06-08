//! Byte-capped, UTF-8-safe previews for untrusted user values in logs.
//!
//! Oracle-Session `logging-observability-next-pass` PR1 (A): every log
//! site that interpolates untrusted input (stdin text, chat titles,
//! dictation queries, triggerBuiltin names, Agent Chat command display strings,
//! …) must route through [`log_user_value`] so the preview can never
//! exceed [`SAFE_USER_VALUE_MAX_BYTES`].
//!
//! The cap is **bytes, not chars**. Log budget is disk + JSONL bytes, and
//! a 120-char value mixing emoji + combining marks can still exceed
//! 480 bytes. Rust `&str` is valid UTF-8, so we cap by byte budget, walk
//! back to the nearest char boundary, trim trailing whitespace, and
//! append an ellipsis inside the budget.
//!
//! Usage:
//!
//! ```ignore
//! let name_safe = logging::log_user_value(name);
//! tracing::warn!(
//!     category = "STDIN",
//!     event_type = "trigger_builtin_unknown",
//!     name_preview = %name_safe,
//!     name_bytes = name_safe.raw_bytes,
//!     name_safe_bytes = name_safe.safe_bytes,
//!     name_truncated = name_safe.truncated,
//!     "triggerBuiltin unknown name — dispatch no-op"
//! );
//! ```

use std::borrow::Cow;
use std::fmt;

/// Default byte cap for a single log preview.
pub const SAFE_USER_VALUE_MAX_BYTES: usize = 200;

/// Ellipsis marker appended to truncated previews (3 bytes in UTF-8).
const ELLIPSIS: &str = "…";

/// Byte-capped preview of an untrusted value plus byte-level metadata.
///
/// `Display` emits only the preview. The `raw_bytes`, `safe_bytes`, and
/// `truncated` fields are intended to be logged as separate structured
/// fields alongside the preview so downstream budget accounting can key
/// off them without re-measuring the string.
#[derive(Clone, Debug)]
pub struct LogSafe<'a> {
    value: Cow<'a, str>,
    /// Byte length of the original (untrimmed) input.
    pub raw_bytes: usize,
    /// Byte length of the emitted preview (always ≤ the byte limit).
    pub safe_bytes: usize,
    /// `true` when the original overflowed the byte budget and the
    /// preview has the ellipsis suffix.
    pub truncated: bool,
}

impl<'a> LogSafe<'a> {
    /// Borrow the preview as `&str`.
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for LogSafe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// Preview `raw` with the default byte cap.
pub fn log_user_value(raw: &str) -> LogSafe<'_> {
    log_user_value_with_limit(raw, SAFE_USER_VALUE_MAX_BYTES)
}

/// Preview `raw` with a caller-chosen byte cap. The budget includes the
/// ellipsis suffix, so the emitted preview is always ≤ `max_bytes`.
pub fn log_user_value_with_limit(raw: &str, max_bytes: usize) -> LogSafe<'_> {
    let trimmed = raw.trim();
    let raw_bytes = raw.len();

    if trimmed.len() <= max_bytes {
        let value = if trimmed.len() == raw.len() {
            Cow::Borrowed(raw)
        } else {
            Cow::Owned(trimmed.to_string())
        };
        let safe_bytes = value.len();
        return LogSafe {
            value,
            raw_bytes,
            safe_bytes,
            truncated: false,
        };
    }

    let budget = max_bytes.saturating_sub(ELLIPSIS.len());
    let mut end = budget.min(trimmed.len());
    while end > 0 && !trimmed.is_char_boundary(end) {
        end -= 1;
    }

    let mut out = trimmed[..end].trim_end().to_string();
    if max_bytes >= ELLIPSIS.len() {
        out.push_str(ELLIPSIS);
    }
    let safe_bytes = out.len();
    LogSafe {
        value: Cow::Owned(out),
        raw_bytes,
        safe_bytes,
        truncated: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_ascii_is_borrowed_unchanged() {
        let safe = log_user_value("hello");
        assert_eq!(safe.as_str(), "hello");
        assert_eq!(safe.raw_bytes, 5);
        assert_eq!(safe.safe_bytes, 5);
        assert!(!safe.truncated);
    }

    #[test]
    fn trims_whitespace_without_marking_truncated() {
        let safe = log_user_value("   hi   ");
        assert_eq!(safe.as_str(), "hi");
        assert_eq!(safe.raw_bytes, 8);
        assert_eq!(safe.safe_bytes, 2);
        assert!(!safe.truncated);
    }

    #[test]
    fn long_ascii_is_byte_capped_with_ellipsis() {
        let long: String = "x".repeat(1024);
        let safe = log_user_value(&long);
        assert!(safe.truncated);
        assert_eq!(safe.raw_bytes, 1024);
        assert!(safe.safe_bytes <= SAFE_USER_VALUE_MAX_BYTES);
        assert!(safe.as_str().ends_with('…'));
    }

    #[test]
    fn emoji_cap_walks_back_to_char_boundary() {
        // 🙂 = 4 bytes. Pack 60 of them (240 bytes) over the default 200-byte cap.
        let emoji = "🙂".repeat(60);
        let safe = log_user_value(&emoji);
        assert!(safe.truncated);
        assert!(safe.safe_bytes <= SAFE_USER_VALUE_MAX_BYTES);
        // The preview must still be valid UTF-8 after truncation — any
        // slice past a mid-char boundary would have panicked by now.
        let preview = safe.as_str().trim_end_matches('…');
        assert!(
            preview.chars().all(|c| c == '🙂'),
            "truncated preview should only contain whole 🙂 chars: {preview:?}"
        );
    }

    #[test]
    fn combining_marks_stay_on_boundary() {
        // "e" + combining acute accent = 3 bytes per visual char.
        let combining = "e\u{0301}".repeat(120);
        let safe = log_user_value(&combining);
        assert!(safe.truncated);
        assert!(safe.safe_bytes <= SAFE_USER_VALUE_MAX_BYTES);
        assert!(safe.as_str().is_char_boundary(safe.as_str().len()));
    }

    #[test]
    fn tiny_budget_drops_ellipsis_when_impossible() {
        let safe = log_user_value_with_limit("long value", 2);
        assert!(safe.truncated);
        assert_eq!(safe.safe_bytes, 0);
    }

    #[test]
    fn exactly_on_budget_not_truncated() {
        let payload = "a".repeat(SAFE_USER_VALUE_MAX_BYTES);
        let safe = log_user_value(&payload);
        assert!(!safe.truncated);
        assert_eq!(safe.safe_bytes, SAFE_USER_VALUE_MAX_BYTES);
        assert_eq!(safe.raw_bytes, SAFE_USER_VALUE_MAX_BYTES);
    }

    #[test]
    fn one_byte_over_budget_is_truncated() {
        let payload = "a".repeat(SAFE_USER_VALUE_MAX_BYTES + 1);
        let safe = log_user_value(&payload);
        assert!(safe.truncated);
        assert!(safe.safe_bytes <= SAFE_USER_VALUE_MAX_BYTES);
        assert!(safe.as_str().ends_with('…'));
    }

    #[test]
    fn display_writes_preview_string_only() {
        let safe = log_user_value("preview");
        assert_eq!(format!("{safe}"), "preview");
    }
}
