//! Text Injection Module for macOS
//!
//! Deletes a typed keyword and inserts its replacement in the app the user is
//! typing in:
//! 1. Delete the keyword with Delete key presses posted to that app
//! 2. Paste the replacement through the clipboard (⌘V), then give the user's
//!    clipboard back
//!
//! ## Architecture
//!
//! - `delete_chars_in()`: Delete presses aimed at one app, no modifiers, tagged
//!   so our own keyboard monitor ignores them (`platform::synthetic_keys`)
//! - `paste_text_in()`: promised-pasteboard paste that restores the full
//!   clipboard once the app has read the text (`platform::promised_paste`)
//! - `delete_chars()` / `paste_text()`: the same, aimed at the focused app
//!
//! ## Configurable Delays
//!
//! - `key_delay_ms`: Delay between Delete presses (default: 2ms)
//! - `pre_paste_delay_ms`: Delay between deleting and pasting in
//!   `inject_text()` (default: 50ms)
//!
//! Clipboard restore timing is not configured here: it follows the target
//! app's request for the pasted text.
//!
//! ## Permissions and honest results
//!
//! Posting ⌘V to another app needs Accessibility (or the narrower "post
//! events" grant). Without it the keystroke would be dropped without a trace,
//! so the text goes on the clipboard instead and the result says so
//! ([`PasteResult::CopiedInstead`]). When the app never reads a paste, the
//! result says that too, and by default the text stays on the clipboard.

use crate::platform::promised_paste::{PasteConsumption, PasteOptions, RestoreOutcome};
use crate::platform::synthetic_keys::{self, KeyTarget};
use anyhow::Result;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

// ============================================================================
// Results
// ============================================================================

/// Why text went to the clipboard instead of into the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopiedReason {
    /// Script Kit can't post keystrokes to other apps.
    AccessibilityMissing,
    /// ⌘V was sent but the app never read the text.
    NotRead,
}

/// What happened to a paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteResult {
    /// The app read the text.
    Pasted,
    /// ⌘V was sent; nothing could observe whether the app read it.
    Sent,
    /// Nothing to paste.
    Empty,
    /// The app didn't get the text. `on_clipboard` says whether the text was
    /// left on the clipboard for the user.
    CopiedInstead {
        reason: CopiedReason,
        on_clipboard: bool,
    },
}

impl PasteResult {
    /// The text is, as far as anyone can tell, in the app.
    pub fn landed(self) -> bool {
        matches!(self, Self::Pasted | Self::Sent | Self::Empty)
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Pasted => "pasted",
            Self::Sent => "sent",
            Self::Empty => "empty",
            Self::CopiedInstead {
                reason: CopiedReason::AccessibilityMissing,
                ..
            } => "copied_accessibility_missing",
            Self::CopiedInstead {
                reason: CopiedReason::NotRead,
                ..
            } => "copied_not_read",
        }
    }

    /// One sentence for the user about `what` ("The transcript"), or `None`
    /// when the text landed. It has no ": ", so a failure line shows all of
    /// it rather than only the words before the colon.
    pub fn user_message(self, what: &str) -> Option<String> {
        match self {
            Self::Pasted | Self::Sent | Self::Empty => None,
            Self::CopiedInstead {
                reason: CopiedReason::AccessibilityMissing,
                on_clipboard: true,
            } => Some(format!(
                "{what} is on the clipboard because Script Kit needs Accessibility to paste into other apps"
            )),
            Self::CopiedInstead {
                reason: CopiedReason::AccessibilityMissing,
                on_clipboard: false,
            } => Some(format!(
                "{what} wasn't pasted because Script Kit needs Accessibility to paste into other apps"
            )),
            Self::CopiedInstead {
                reason: CopiedReason::NotRead,
                on_clipboard: true,
            } => Some(format!(
                "{what} is on the clipboard because the app didn't take the paste"
            )),
            Self::CopiedInstead {
                reason: CopiedReason::NotRead,
                on_clipboard: false,
            } => Some(format!("{what} wasn't pasted because the app didn't take it")),
        }
    }
}

/// Turn the paste engine's report into a [`PasteResult`].
pub fn classify_paste(consumption: PasteConsumption, restore: &RestoreOutcome) -> PasteResult {
    match consumption {
        PasteConsumption::Requested { .. } => PasteResult::Pasted,
        PasteConsumption::Unobserved => PasteResult::Sent,
        PasteConsumption::NotRequested { .. } => PasteResult::CopiedInstead {
            reason: CopiedReason::NotRead,
            on_clipboard: matches!(restore, RestoreOutcome::KeptPastedText),
        },
    }
}

/// Tell the user, by system notification, that `what` didn't land. For paste
/// paths that run after Script Kit's window has hidden.
pub fn report_in_background(result: &Result<PasteResult>, what: &str) {
    match result {
        Ok(outcome) => {
            let Some(message) = outcome.user_message(what) else {
                return;
            };
            let (title, hint) = match outcome {
                PasteResult::CopiedInstead {
                    reason: CopiedReason::AccessibilityMissing,
                    on_clipboard,
                } => (
                    if *on_clipboard {
                        "Copied instead of pasted"
                    } else {
                        "Couldn't paste"
                    },
                    " Open Script Kit and run “Set Up Permissions”.",
                ),
                PasteResult::CopiedInstead {
                    on_clipboard: true, ..
                } => ("Copied instead of pasted", " Press ⌘V to paste it."),
                _ => ("Couldn't paste", ""),
            };
            crate::permissions::notify_user(title, &format!("{message}.{hint}"));
        }
        Err(error) => {
            warn!(category = "PASTE", error = %error, what, "Paste failed");
            crate::permissions::notify_user("Couldn't paste", &format!("{what} wasn't pasted."));
        }
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for text injection timing and behavior
#[derive(Debug, Clone)]
pub struct TextInjectorConfig {
    /// Delay in milliseconds between Delete key events (default: 2ms)
    pub key_delay_ms: u64,
    /// Delay in milliseconds between deleting and pasting in `inject_text` (default: 50ms)
    pub pre_paste_delay_ms: u64,
    /// Leave the text on the clipboard when the app never reads the paste
    /// (default: true). Snippets turn this off: their text lives in the
    /// snippet file, so the user's own clipboard matters more.
    pub keep_text_if_unread: bool,
}

impl Default for TextInjectorConfig {
    fn default() -> Self {
        Self {
            key_delay_ms: 2,
            pre_paste_delay_ms: 50,
            keep_text_if_unread: true,
        }
    }
}

// ============================================================================
// Text Injector
// ============================================================================

/// Text injector for deleting trigger text and inserting replacements
#[derive(Debug, Clone)]
pub struct TextInjector {
    config: TextInjectorConfig,
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInjector {
    /// Create a new TextInjector with default configuration
    pub fn new() -> Self {
        Self {
            config: TextInjectorConfig::default(),
        }
    }

    /// Create a new TextInjector with custom configuration
    pub fn with_config(config: TextInjectorConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &TextInjectorConfig {
        &self.config
    }

    /// Delete `count` characters before the caret of the focused app.
    #[instrument(skip(self), fields(count))]
    pub fn delete_chars(&self, count: usize) -> Result<()> {
        if count == 0 {
            debug!("No characters to delete");
            return Ok(());
        }
        self.delete_chars_in(synthetic_keys::focused_key_target(), count)
    }

    /// Delete `count` characters before the caret of `target`.
    #[instrument(skip(self), fields(count, route = target.route_name()))]
    pub fn delete_chars_in(&self, target: KeyTarget, count: usize) -> Result<()> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
        if count == 0 {
            debug!("No characters to delete");
            return Ok(());
        }
        synthetic_keys::post_deletes(
            target,
            count,
            Duration::from_millis(self.config.key_delay_ms),
        )?;
        info!(count, route = target.route_name(), "Deleted characters");
        Ok(())
    }

    /// Paste `text` into the focused app and restore the user's clipboard.
    #[instrument(skip(self, text), fields(text_len = text.len()))]
    pub fn paste_text(&self, text: &str) -> Result<PasteResult> {
        self.paste_into(None, text)
    }

    /// Paste `text` into `target` and restore the user's clipboard.
    #[instrument(skip(self, text), fields(text_len = text.len(), route = target.route_name()))]
    pub fn paste_text_in(&self, target: KeyTarget, text: &str) -> Result<PasteResult> {
        self.paste_into(Some(target), text)
    }

    /// Paste `text` into the focused app from a background path the user
    /// can't see, and tell them (system notification) when `what` didn't
    /// land, for example `"The AI Vault command"`.
    pub fn paste_text_reporting(&self, text: &str, what: &str) -> Result<PasteResult> {
        let result = self.paste_text(text);
        report_in_background(&result, what);
        result
    }

    fn paste_into(&self, target: Option<KeyTarget>, text: &str) -> Result<PasteResult> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
        if text.is_empty() {
            debug!("Empty text, nothing to paste");
            return Ok(PasteResult::Empty);
        }
        if !crate::permissions::can_post_keystrokes() {
            // ⌘V would be dropped without a trace; give the user the text.
            let on_clipboard =
                crate::platform::promised_paste::copy_text_to_clipboard(text).is_ok();
            warn!(
                category = "PASTE",
                event = "paste_copied_instead",
                reason = "accessibility_missing",
                on_clipboard,
                text_len = text.len(),
                "Script Kit can't post keystrokes; copied the text instead of pasting"
            );
            return Ok(PasteResult::CopiedInstead {
                reason: CopiedReason::AccessibilityMissing,
                on_clipboard,
            });
        }
        // Outcome details (target read the text, clipboard restored) are
        // logged by the paste engine as `synthetic_paste`.
        let report = crate::platform::promised_paste::paste_text_with_options(
            text,
            target,
            PasteOptions {
                keep_text_if_unread: self.config.keep_text_if_unread,
            },
        )?;
        Ok(classify_paste(report.consumption, &report.restore))
    }

    /// Inject text by deleting trigger characters and pasting replacement
    ///
    /// Both steps go to the app that is focused when this is called.
    #[allow(dead_code)]
    #[instrument(skip(self, replacement), fields(delete_count, replacement_len = replacement.len()))]
    pub fn inject_text(&self, delete_count: usize, replacement: &str) -> Result<PasteResult> {
        let target = synthetic_keys::focused_key_target();
        self.delete_chars_in(target, delete_count)?;
        if self.config.pre_paste_delay_ms > 0 {
            thread::sleep(Duration::from_millis(self.config.pre_paste_delay_ms));
        }
        let result = self.paste_text_in(target, replacement)?;
        info!(result = result.code(), "Text injection completed");
        Ok(result)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TextInjectorConfig::default();
        assert_eq!(config.key_delay_ms, 2);
        assert_eq!(config.pre_paste_delay_ms, 50);
    }

    #[test]
    fn test_custom_config() {
        let config = TextInjectorConfig {
            key_delay_ms: 5,
            pre_paste_delay_ms: 100,
            keep_text_if_unread: false,
        };
        let injector = TextInjector::with_config(config.clone());
        assert_eq!(injector.config().key_delay_ms, 5);
        assert_eq!(injector.config().pre_paste_delay_ms, 100);
    }

    #[test]
    fn unread_and_unpermitted_pastes_are_reported_not_silent() {
        let unread = classify_paste(
            PasteConsumption::NotRequested {
                waited: Duration::from_millis(1200),
            },
            &RestoreOutcome::KeptPastedText,
        );
        assert_eq!(
            unread,
            PasteResult::CopiedInstead {
                reason: CopiedReason::NotRead,
                on_clipboard: true
            }
        );
        assert!(!unread.landed());
        assert_eq!(
            unread.user_message("The transcript").as_deref(),
            Some("The transcript is on the clipboard because the app didn't take the paste")
        );
        let restored = classify_paste(
            PasteConsumption::NotRequested {
                waited: Duration::from_millis(1200),
            },
            &RestoreOutcome::Restored,
        );
        assert_eq!(
            restored.user_message("The snippet").as_deref(),
            Some("The snippet wasn't pasted because the app didn't take it")
        );
        let no_access = PasteResult::CopiedInstead {
            reason: CopiedReason::AccessibilityMissing,
            on_clipboard: true,
        };
        assert!(no_access
            .user_message("The transcript")
            .unwrap_or_default()
            .contains("needs Accessibility"));
        assert_eq!(no_access.code(), "copied_accessibility_missing");
    }

    #[test]
    fn read_and_unobservable_pastes_count_as_landed() {
        assert_eq!(
            classify_paste(
                PasteConsumption::Requested {
                    after: Duration::from_millis(30)
                },
                &RestoreOutcome::Restored
            ),
            PasteResult::Pasted
        );
        let sent = classify_paste(PasteConsumption::Unobserved, &RestoreOutcome::Restored);
        assert_eq!(sent, PasteResult::Sent);
        assert!(sent.landed());
        assert_eq!(sent.user_message("x"), None);
    }

    #[test]
    fn test_injector_new() {
        let injector = TextInjector::new();
        assert_eq!(injector.config().key_delay_ms, 2);
    }

    #[test]
    fn test_injector_default() {
        let injector = TextInjector::default();
        assert_eq!(injector.config().key_delay_ms, 2);
    }

    #[test]
    fn test_delete_chars_zero() {
        // Deleting zero chars should succeed without doing anything
        let injector = TextInjector::new();
        // This won't actually simulate keys in tests, but checks the early return
        let result = injector.delete_chars(0);
        assert!(result.is_ok());
    }
}

// ============================================================================
// System Tests (require `cargo test --features system-tests`)
// ============================================================================

#[cfg(all(test, feature = "system-tests"))]
mod system_tests {
    use super::*;

    #[test]
    #[ignore] // Requires accessibility permission and user interaction
    fn test_delete_chars_sends_backspaces() {
        // Instructions:
        // 1. Open TextEdit and type "hello"
        // 2. Run: cargo test --features system-tests test_delete_chars_sends_backspaces -- --ignored
        // 3. The last 2 characters should be deleted, leaving "hel"
        let injector = TextInjector::new();
        injector.delete_chars(2).expect("Should delete chars");
        println!("Deleted 2 characters");
    }

    #[test]
    #[ignore] // Requires accessibility permission and user interaction
    fn test_paste_text() {
        // Instructions:
        // 1. Open TextEdit with cursor positioned
        // 2. Run: cargo test --features system-tests test_paste_text -- --ignored
        // 3. "TEST PASTE" should be inserted
        let injector = TextInjector::new();
        injector.paste_text("TEST PASTE").expect("Should paste");
        println!("Pasted text");
    }

    #[test]
    #[ignore] // Requires accessibility permission and user interaction
    fn test_inject_text() {
        // Instructions:
        // 1. Open TextEdit and type "btw"
        // 2. Run: cargo test --features system-tests test_inject_text -- --ignored
        // 3. "btw" should be replaced with "by the way"
        let injector = TextInjector::new();
        injector
            .inject_text(3, "by the way")
            .expect("Should inject");
        println!("Injected text");
    }

    // REMOVED: test_empty_paste — calls paste_text which simulates real Cmd+V
}
