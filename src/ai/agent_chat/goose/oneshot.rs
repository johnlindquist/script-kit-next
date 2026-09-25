//! Single-prompt completions through goose, for callers that need an answer
//! rather than a chat session (script generation, titles, SDK chat, brain).
//!
//! Each call starts a short-lived goose process with no tools, sends one
//! prompt, and streams the assistant text back.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::ai::agent_chat::content::{ContentBlock, ImageContent, TextContent};
use crate::ai::agent_chat::events::AgentChatEvent;
use crate::ai::agent_chat::runtime::{AgentChatConnection, AgentChatTurnRequest};
use crate::ai::agent_chat::ui::capabilities::AgentChatToolPolicy;

use super::launch_spec::{ensure_goose_root, goose_process_env, goose_provider_id, goose_root};
use super::protocol::{split_model_selection, GooseLaunchSpec};
use super::recipe::protect_instructions;

/// One completion request.
#[derive(Debug, Clone, Default)]
pub(crate) struct OneshotRequest {
    pub system: Option<String>,
    pub prompt: String,
    pub images: Vec<ImageContent>,
    /// `provider/model`; `None` uses the Agent Chat default.
    pub model: Option<String>,
    pub timeout: Option<Duration>,
    /// A person is waiting on this answer, so a missing browser sign-in may
    /// be started. Background work leaves this `false`.
    pub interactive_sign_in: bool,
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// Run one prompt and return the full assistant text.
pub(crate) fn complete(request: OneshotRequest) -> Result<String> {
    let mut text = String::new();
    stream(request, |delta| {
        text.push_str(delta);
        true
    })?;
    Ok(text)
}

/// Run one prompt, calling `on_delta` for each piece of assistant text.
/// Returning `false` from `on_delta` stops the turn early.
pub(crate) fn stream(
    request: OneshotRequest,
    mut on_delta: impl FnMut(&str) -> bool,
) -> Result<()> {
    // Refuse before touching the filesystem or resolving a binary when the
    // runtime policy forbids provider effects (owned evaluation, fixtures).
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Provider)?;
    let goose_binary = super::binary::default_goose_binary()
        .ok_or_else(|| anyhow!("The goose AI engine is not installed"))?;
    let kit_path = crate::setup::get_kit_path();
    let root = goose_root(&kit_path);
    ensure_goose_root(&root)?;
    let cwd = kit_path.join("agent-chat").join("oneshot");
    std::fs::create_dir_all(&cwd)?;

    let (provider, model) = resolve_model(request.model.as_deref());
    let mut settings = serde_json::Map::new();
    settings.insert("goose_provider".into(), provider.clone().into());
    settings.insert("goose_model".into(), model.clone().into());
    let instructions = request
        .system
        .as_deref()
        .map(str::trim)
        .filter(|system| !system.is_empty())
        .unwrap_or("Answer the user's request directly.");
    let recipe = serde_json::json!({
        "version": "1.0.0",
        "title": "Script Kit",
        "description": "Script Kit one-shot completion",
        "instructions": protect_instructions(instructions),
        "extensions": [],
        "settings": settings,
    });

    let mut spec = GooseLaunchSpec::new(goose_binary, cwd.clone());
    spec.env = goose_process_env(&root);
    spec.recipe = recipe;
    spec.session_title = "Script Kit completion".to_string();
    spec.hidden_session = true;
    let runtime = super::runtime::GooseAcpRuntime::spawn(spec)?;

    let mut blocks = vec![ContentBlock::Text(TextContent::new(request.prompt))];
    blocks.extend(request.images.into_iter().map(ContentBlock::Image));
    let events = runtime
        .start_turn(AgentChatTurnRequest {
            ui_thread_id: "goose-oneshot".to_string(),
            cwd: PathBuf::from(&cwd),
            blocks,
            model_id: None,
            tool_policy: AgentChatToolPolicy::WebSearchOnly,
            allow_interactive_sign_in: request.interactive_sign_in,
        })
        .map_err(|error| anyhow!(error.failure.primary_message().to_string()))?;

    let deadline = std::time::Instant::now() + request.timeout.unwrap_or(DEFAULT_TIMEOUT);
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            let _ = runtime.cancel_turn("goose-oneshot".to_string());
            return Err(anyhow!("The AI response timed out"));
        }
        let event = match events.recv_blocking() {
            Ok(event) => event,
            Err(_) => return Err(anyhow!("The AI connection stopped before answering")),
        };
        match event {
            AgentChatEvent::AgentMessageDelta(delta) => {
                if !on_delta(&delta) {
                    let _ = runtime.cancel_turn("goose-oneshot".to_string());
                    return Ok(());
                }
            }
            // No chat surface hosts this request: the person waiting on the
            // answer already asked for AI, so sign in without a second prompt.
            AgentChatEvent::SignInRequired { consent, .. } => {
                let _ = consent.try_send(true);
            }
            AgentChatEvent::TurnCompleted { .. } => return Ok(()),
            AgentChatEvent::TurnFailed { failure } => {
                return Err(OneshotFailure(failure).into());
            }
            _ => {}
        }
    }
}

/// A classified goose failure, kept intact so callers can show its safe copy
/// and keep the diagnostic record (rules/AI_RELIABILITY.md Rule 2).
#[derive(Debug)]
pub(crate) struct OneshotFailure(pub crate::ai::reliability::AppFailureRecord);

impl std::fmt::Display for OneshotFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.primary_message())
    }
}

impl std::error::Error for OneshotFailure {}

/// Split a `provider/model` selection, falling back to the Agent Chat
/// defaults for whichever half is missing.
pub(crate) fn resolve_model(selection: Option<&str>) -> (String, String) {
    use crate::ai::agent_chat::profiles::{DEFAULT_MODEL, DEFAULT_PROVIDER};
    let selection = selection
        .map(str::to_string)
        .or_else(|| crate::config::load_user_preferences().ai.selected_model_id)
        .filter(|value| !value.trim().is_empty());
    match selection.as_deref().and_then(split_model_selection) {
        Some((provider, model)) => (goose_provider_id(provider), model.to_string()),
        None => (DEFAULT_PROVIDER.to_string(), DEFAULT_MODEL.to_string()),
    }
}
