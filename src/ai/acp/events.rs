//! Typed ACP turn/event primitives.
//!
//! These replace the one-shot `StreamCallback` model with a structured
//! event stream that the `AcpThread` entity can consume for durable
//! per-thread history, pending tool state, and permission UX.

use std::path::PathBuf;

use agent_client_protocol::ContentBlock;

/// Channel types for ACP event streaming.
pub(crate) type AcpEventRx = async_channel::Receiver<AcpEvent>;
pub(crate) type AcpEventTx = async_channel::Sender<AcpEvent>;

/// A request to start a new ACP turn on a specific thread.
#[derive(Debug, Clone)]
pub(crate) struct AcpPromptTurnRequest {
    /// Script Kit UI thread identifier (maps to ACP session).
    pub ui_thread_id: String,
    /// Working directory for the ACP session.
    pub cwd: PathBuf,
    /// Content blocks to send as the prompt.
    pub blocks: Vec<ContentBlock>,
}

/// Typed events emitted by the ACP worker for a single turn.
///
/// Each variant maps to a specific `SessionUpdate` notification from
/// the ACP protocol, plus lifecycle events (`TurnFinished`, `Failed`).
#[derive(Debug, Clone)]
pub(crate) enum AcpEvent {
    /// A chunk of user message text echoed back by the agent.
    UserMessageDelta(String),
    /// A chunk of assistant message text.
    AgentMessageDelta(String),
    /// A chunk of agent internal reasoning / thought.
    AgentThoughtDelta(String),
    /// A new tool call has started.
    ToolCallStarted {
        tool_call_id: String,
        title: String,
        status: String,
    },
    /// An existing tool call has been updated.
    ToolCallUpdated {
        tool_call_id: String,
        title: Option<String>,
        status: Option<String>,
        body: Option<String>,
    },
    /// The agent's plan has been updated.
    PlanUpdated { entries: Vec<String> },
    /// Available commands have changed.
    AvailableCommandsUpdated { command_names: Vec<String> },
    /// The agent's mode has changed.
    ModeChanged { mode_id: String },
    /// The turn completed normally.
    TurnFinished { stop_reason: String },
    /// The turn failed with an error.
    Failed { error: String },
}

/// Commands sent from the GPUI thread to the ACP worker for event-driven turns.
pub(crate) enum AcpCommand {
    /// Start a new turn and stream events back through `event_tx`.
    StartTurn {
        request: AcpPromptTurnRequest,
        event_tx: AcpEventTx,
    },
    /// Legacy: stream prompt with a callback (used by AiProvider path).
    StreamPrompt {
        ui_session_id: String,
        cwd: PathBuf,
        messages: Vec<crate::ai::providers::ProviderMessage>,
        on_chunk: crate::ai::providers::StreamCallback,
        reply_tx: async_channel::Sender<anyhow::Result<()>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acp_event_variants_are_clone() {
        let event = AcpEvent::AgentMessageDelta("hello".to_string());
        let _cloned = event.clone();
    }

    #[test]
    fn acp_prompt_turn_request_is_debug() {
        let request = AcpPromptTurnRequest {
            ui_thread_id: "thread-1".to_string(),
            cwd: PathBuf::from("/tmp"),
            blocks: vec![],
        };
        let debug = format!("{:?}", request);
        assert!(debug.contains("thread-1"));
    }

    #[test]
    fn acp_command_start_turn_holds_channel() {
        let (tx, _rx) = async_channel::bounded::<AcpEvent>(1);
        let _cmd = AcpCommand::StartTurn {
            request: AcpPromptTurnRequest {
                ui_thread_id: "t1".to_string(),
                cwd: PathBuf::from("."),
                blocks: vec![],
            },
            event_tx: tx,
        };
        // Verify the command variant can be constructed and holds a valid channel
    }
}
