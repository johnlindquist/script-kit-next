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
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility

use crate::platform::synthetic_keys::{self, KeyTarget};
use anyhow::Result;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, instrument};

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
}

impl Default for TextInjectorConfig {
    fn default() -> Self {
        Self {
            key_delay_ms: 2,
            pre_paste_delay_ms: 50,
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
    pub fn paste_text(&self, text: &str) -> Result<()> {
        self.paste_into(None, text)
    }

    /// Paste `text` into `target` and restore the user's clipboard.
    #[instrument(skip(self, text), fields(text_len = text.len(), route = target.route_name()))]
    pub fn paste_text_in(&self, target: KeyTarget, text: &str) -> Result<()> {
        self.paste_into(Some(target), text)
    }

    fn paste_into(&self, target: Option<KeyTarget>, text: &str) -> Result<()> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
        if text.is_empty() {
            debug!("Empty text, nothing to paste");
            return Ok(());
        }
        // Outcome details (target read the text, clipboard restored) are
        // logged by the paste engine as `synthetic_paste`.
        crate::platform::promised_paste::paste_text_preserving_clipboard(text, target)?;
        Ok(())
    }

    /// Inject text by deleting trigger characters and pasting replacement
    ///
    /// Both steps go to the app that is focused when this is called.
    #[allow(dead_code)]
    #[instrument(skip(self, replacement), fields(delete_count, replacement_len = replacement.len()))]
    pub fn inject_text(&self, delete_count: usize, replacement: &str) -> Result<()> {
        let target = synthetic_keys::focused_key_target();
        self.delete_chars_in(target, delete_count)?;
        if self.config.pre_paste_delay_ms > 0 {
            thread::sleep(Duration::from_millis(self.config.pre_paste_delay_ms));
        }
        self.paste_text_in(target, replacement)?;
        info!("Text injection completed");
        Ok(())
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
        };
        let injector = TextInjector::with_config(config.clone());
        assert_eq!(injector.config().key_delay_ms, 5);
        assert_eq!(injector.config().pre_paste_delay_ms, 100);
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
