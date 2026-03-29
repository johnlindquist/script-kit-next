//! ACP-backed `AiProvider` implementation.
//!
//! Wraps `AcpRuntime` so that any ACP-compatible agent (Claude Code, Gemini CLI,
//! Codex, OpenCode) can be used through the same `AiProvider` trait the rest of
//! the codebase already relies on.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};

use crate::ai::config::ModelInfo;
use crate::ai::providers::{AiProvider, ProviderMessage, StreamCallback};

use super::client::AcpRuntime;
use super::config::AcpAgentConfig;

/// An `AiProvider` that delegates to an ACP agent subprocess.
///
/// The runtime is lazily spawned on first use and kept alive for the
/// provider's lifetime. Each `stream_message` call reuses the same
/// runtime (and its session cache).
pub(crate) struct AcpProvider {
    agent: AcpAgentConfig,
    runtime: parking_lot::Mutex<Option<AcpRuntime>>,
}

impl AcpProvider {
    pub(crate) fn new(agent: AcpAgentConfig) -> Self {
        Self {
            agent,
            runtime: parking_lot::Mutex::new(None),
        }
    }

    /// Ensure the runtime is spawned and call `f` with a reference to it.
    fn with_runtime<T>(&self, f: impl FnOnce(&AcpRuntime) -> Result<T>) -> Result<T> {
        let mut guard = self.runtime.lock();
        if guard.is_none() {
            *guard = Some(AcpRuntime::spawn(self.agent.clone())?);
        }
        match guard.as_ref() {
            Some(rt) => f(rt),
            None => anyhow::bail!("ACP runtime failed to initialize"),
        }
    }

    fn current_cwd() -> Result<PathBuf> {
        std::env::current_dir().context("failed to determine current working directory")
    }
}

impl AiProvider for AcpProvider {
    fn provider_id(&self) -> &str {
        self.agent.provider_id()
    }

    fn display_name(&self) -> &str {
        self.agent.display_name()
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        self.agent.model_infos()
    }

    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
        let full_text = Arc::new(Mutex::new(String::new()));
        let full_text_clone = Arc::clone(&full_text);

        self.stream_message(
            messages,
            model_id,
            Box::new(move |chunk| {
                if let Ok(mut buf) = full_text_clone.lock() {
                    buf.push_str(&chunk);
                }
                true
            }),
            None,
        )?;

        Ok(full_text
            .lock()
            .map(|buf| buf.clone())
            .unwrap_or_default())
    }

    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        _model_id: &str,
        on_chunk: StreamCallback,
        session_id: Option<&str>,
    ) -> Result<()> {
        let ui_session_id = session_id
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        self.with_runtime(|runtime| {
            runtime.stream_prompt(
                ui_session_id,
                Self::current_cwd()?,
                messages.to_vec(),
                on_chunk,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_agent() -> AcpAgentConfig {
        AcpAgentConfig {
            id: "test-acp".into(),
            display_name: "Test ACP Agent".into(),
            command: "echo".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![super::super::config::AcpModelEntry {
                id: "test-model".into(),
                display_name: Some("Test Model".into()),
                context_window: Some(64_000),
            }],
        }
    }

    #[test]
    fn provider_metadata_matches_agent_config() {
        let provider = AcpProvider::new(test_agent());
        assert_eq!(provider.provider_id(), "test-acp");
        assert_eq!(provider.display_name(), "Test ACP Agent");

        let models = provider.available_models();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "test-model");
        assert_eq!(models[0].display_name, "Test Model");
        assert_eq!(models[0].context_window, 64_000);
        assert!(models[0].supports_streaming);
    }

    #[test]
    fn registry_can_register_acp_provider() {
        let mut registry = crate::ai::providers::ProviderRegistry::new();
        registry.register(Arc::new(AcpProvider::new(test_agent())));
        assert!(registry.get_provider("test-acp").is_some());
        assert!(registry.get_provider("nonexistent").is_none());
    }

    #[test]
    fn acp_provider_id_differs_from_legacy_claude_code() {
        let agent = AcpAgentConfig {
            id: "claude-code".into(),
            display_name: "Claude Code (ACP)".into(),
            command: "claude".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![],
        };
        let provider = AcpProvider::new(agent);
        // ACP Claude uses "claude-code", legacy uses "claude_code"
        assert_eq!(provider.provider_id(), "claude-code");
        assert_ne!(provider.provider_id(), "claude_code");
    }
}
