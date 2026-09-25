//! Provider API keys Script Kit manages, and model descriptors.
//!
//! Keys are discovered from environment variables and the system keyring using
//! the `SCRIPT_KIT_*_API_KEY` pattern, then handed to the goose engine under the
//! variable names its providers read.
//!
//! # Key Detection Order
//!
//! API keys are detected in the following order (first found wins):
//! 1. Environment variable (`SCRIPT_KIT_*_API_KEY`)
//! 2. System keyring (macOS Keychain via `com.scriptkit.env` service)
//!
//! This allows users to either set environment variables in their shell profile
//! or use the built-in "Configure API Key" commands which store in the keyring.
//!
//! # Environment Variable Pattern
//!
//! API keys are detected with the `SCRIPT_KIT_` prefix:
//! - `SCRIPT_KIT_OPENAI_API_KEY` -> goose `openai`
//! - `SCRIPT_KIT_ANTHROPIC_API_KEY` -> goose `anthropic`
//! - `SCRIPT_KIT_VERCEL_API_KEY` -> goose Vercel AI Gateway
//!
//! This prefix ensures users explicitly configure keys for Script Kit,
//! rather than accidentally exposing keys from other applications.

use std::env;
use std::fmt;

use crate::secrets::get_secret;

/// Wrapper type for secret values that always redacts in logs.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SecretString(String);

impl SecretString {
    /// Create a new secret string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Expose the underlying secret value for API usage only.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Return true when the underlying secret is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and return the underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<SecretString> for String {
    fn from(value: SecretString) -> Self {
        value.0
    }
}

impl AsRef<str> for SecretString {
    fn as_ref(&self) -> &str {
        self.expose()
    }
}

/// Information about an AI model.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Unique identifier for the model (e.g., "gpt-4o", "claude-3-5-sonnet")
    pub id: String,
    /// Human-readable display name
    pub display_name: String,
    /// Provider this model belongs to
    pub provider: String,
    /// Whether this model supports streaming responses
    pub supports_streaming: bool,
    /// Context window size in tokens
    pub context_window: u32,
}

impl ModelInfo {
    /// Create a new model info.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        provider: impl Into<String>,
        supports_streaming: bool,
        context_window: u32,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            provider: provider.into(),
            supports_streaming,
            context_window,
        }
    }
}

/// Environment variable names for API keys.
pub mod env_vars {
    /// OpenAI API key environment variable
    pub const OPENAI_API_KEY: &str = "SCRIPT_KIT_OPENAI_API_KEY";
    /// Anthropic API key environment variable
    pub const ANTHROPIC_API_KEY: &str = "SCRIPT_KIT_ANTHROPIC_API_KEY";
    /// Google AI (Gemini) API key environment variable
    pub const GOOGLE_API_KEY: &str = "SCRIPT_KIT_GOOGLE_API_KEY";
    /// Groq API key environment variable
    pub const GROQ_API_KEY: &str = "SCRIPT_KIT_GROQ_API_KEY";
    /// Vercel API key environment variable
    pub const VERCEL_API_KEY: &str = "SCRIPT_KIT_VERCEL_API_KEY";
}

/// Read an environment variable, trimming whitespace and filtering empty values.
///
/// Returns `None` if the variable is not set, is empty after trimming, or contains only whitespace.
fn read_env_nonempty(name: &str) -> Option<String> {
    if let Err(error) =
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials)
    {
        tracing::warn!(%error, "Provider environment access refused");
        return None;
    }
    env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Read an API key from environment variable or system keyring.
///
/// Checks in order:
/// 1. Environment variable (for users who set keys in shell profile)
/// 2. System keyring (for users who use built-in "Configure API Key" commands)
///
/// Returns the first non-empty value found, or `None` if not configured.
fn read_key_env_or_keyring(name: &str) -> Option<String> {
    if let Err(error) =
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials)
    {
        tracing::warn!(%error, "Provider credential discovery refused");
        return None;
    }
    // First check environment variable
    if let Some(value) = read_env_nonempty(name) {
        tracing::info!(target: "ai::config", key_name = %name, source = "env", "Found API key");
        return Some(value);
    }

    // Fall back to keyring
    let keyring_result = get_secret(name);
    if keyring_result.is_some() {
        tracing::info!(target: "ai::config", key_name = %name, source = "keyring", "Found API key");
    }
    keyring_result
}

/// Detected API keys from environment.
#[derive(Default)]
pub struct DetectedKeys {
    pub(crate) openai: Option<SecretString>,
    pub(crate) anthropic: Option<SecretString>,
    pub(crate) google: Option<SecretString>,
    pub(crate) groq: Option<SecretString>,
    pub(crate) vercel: Option<SecretString>,
}

impl std::fmt::Debug for DetectedKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetectedKeys")
            .field("openai", &self.openai.is_some())
            .field("anthropic", &self.anthropic.is_some())
            .field("google", &self.google.is_some())
            .field("groq", &self.groq.is_some())
            .field("vercel", &self.vercel.is_some())
            .finish()
    }
}

impl DetectedKeys {
    /// Scan environment variables and system keyring for API keys.
    ///
    /// Looks for the `SCRIPT_KIT_*_API_KEY` pattern in:
    /// 1. Environment variables (for users who set keys in shell profile)
    /// 2. System keyring (for users who use built-in "Configure API Key" commands)
    ///
    /// This allows the SDK's `await env("SCRIPT_KIT_VERCEL_API_KEY")` to store keys
    /// in the keyring, and have Agent Chat automatically pick them up.
    pub fn from_environment() -> Self {
        Self {
            openai: read_key_env_or_keyring(env_vars::OPENAI_API_KEY).map(SecretString::from),
            anthropic: read_key_env_or_keyring(env_vars::ANTHROPIC_API_KEY).map(SecretString::from),
            google: read_key_env_or_keyring(env_vars::GOOGLE_API_KEY).map(SecretString::from),
            groq: read_key_env_or_keyring(env_vars::GROQ_API_KEY).map(SecretString::from),
            vercel: read_key_env_or_keyring(env_vars::VERCEL_API_KEY).map(SecretString::from),
        }
    }

    /// Check if any API keys were detected.
    pub fn has_any(&self) -> bool {
        self.openai.is_some()
            || self.anthropic.is_some()
            || self.google.is_some()
            || self.groq.is_some()
            || self.vercel.is_some()
    }

    /// Get a summary of which providers are available (for logging).
    ///
    /// Returns a list of provider names that have API keys configured.
    /// Does NOT include the actual keys.
    pub fn available_providers(&self) -> Vec<&'static str> {
        let mut providers = Vec::new();
        if self.openai.is_some() {
            providers.push("OpenAI");
        }
        if self.anthropic.is_some() {
            providers.push("Anthropic");
        }
        if self.google.is_some() {
            providers.push("Google");
        }
        if self.groq.is_some() {
            providers.push("Groq");
        }
        if self.vercel.is_some() {
            providers.push("Vercel");
        }
        providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_keys_empty() {
        // Clear any existing env vars for this test
        let keys = DetectedKeys::default();
        assert!(!keys.has_any());
        assert!(keys.available_providers().is_empty());
    }

    #[test]
    fn test_detected_keys_with_provider() {
        // Manually construct to avoid env dependency in test
        let keys = DetectedKeys {
            openai: Some("sk-test".into()),
            anthropic: None,
            google: None,
            groq: None,
            vercel: None,
        };
        assert!(keys.has_any());
        assert_eq!(keys.available_providers(), vec!["OpenAI"]);
    }

    #[test]
    fn test_detected_keys_debug_shows_only_presence() {
        let keys = DetectedKeys {
            openai: Some("sk-secret-openai-key".into()),
            anthropic: Some("sk-ant-secret-key".into()),
            google: None,
            groq: None,
            vercel: Some("vk-vercel-key".into()),
        };
        let debug_output = format!("{:?}", keys);

        // Actual key values should NOT appear
        assert!(!debug_output.contains("sk-secret-openai-key"));
        assert!(!debug_output.contains("sk-ant-secret-key"));
        assert!(!debug_output.contains("vk-vercel-key"));
        // Should show boolean presence indicators
        assert!(debug_output.contains("openai: true"));
        assert!(debug_output.contains("anthropic: true"));
        assert!(debug_output.contains("google: false"));
        assert!(debug_output.contains("vercel: true"));
    }

    #[test]
    fn test_read_env_nonempty_trims_whitespace() {
        // Set up test env var with whitespace
        std::env::set_var("TEST_READ_ENV_WHITESPACE", "  test-value  ");
        let result = read_env_nonempty("TEST_READ_ENV_WHITESPACE");
        assert_eq!(result, Some("test-value".to_string()));
        std::env::remove_var("TEST_READ_ENV_WHITESPACE");
    }

    #[test]
    fn test_read_env_nonempty_filters_empty() {
        // Set up test env var that's empty
        std::env::set_var("TEST_READ_ENV_EMPTY", "");
        let result = read_env_nonempty("TEST_READ_ENV_EMPTY");
        assert_eq!(result, None);
        std::env::remove_var("TEST_READ_ENV_EMPTY");
    }

    #[test]
    fn test_read_env_nonempty_filters_whitespace_only() {
        // Set up test env var with only whitespace
        std::env::set_var("TEST_READ_ENV_WS_ONLY", "   ");
        let result = read_env_nonempty("TEST_READ_ENV_WS_ONLY");
        assert_eq!(result, None);
        std::env::remove_var("TEST_READ_ENV_WS_ONLY");
    }

    #[test]
    fn test_detected_keys_with_vercel() {
        let keys = DetectedKeys {
            openai: None,
            anthropic: None,
            google: None,
            groq: None,
            vercel: Some("vk-test".into()),
        };
        assert!(keys.has_any());
        assert_eq!(keys.available_providers(), vec!["Vercel"]);
    }

    #[test]
    fn test_secret_string_debug_masks_value() {
        let secret = SecretString::new("sk-test-super-secret");
        assert_eq!(format!("{:?}", secret), "***");
    }

    #[test]
    fn test_secret_string_display_masks_value() {
        let secret = SecretString::new("sk-test-super-secret");
        assert_eq!(format!("{}", secret), "***");
    }

    #[test]
    fn test_secret_string_expose_returns_inner_value() {
        let secret = SecretString::new("sk-test-super-secret");
        assert_eq!(secret.expose(), "sk-test-super-secret");
    }
}
