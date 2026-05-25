//! Typed ACP turn/event primitives.
//!
//! These replace the one-shot `StreamCallback` model with a structured
//! event stream that the `AcpThread` entity can consume for durable
//! per-thread history, pending tool state, and permission UX.

/// Channel types for ACP event streaming.
pub(crate) type AcpEventTx = async_channel::Sender<AcpEvent>;
pub(crate) type AcpEventRx = async_channel::Receiver<AcpEvent>;

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
    /// Session usage metrics updated.
    UsageUpdated {
        used_tokens: u64,
        context_size: u64,
        cost_usd: Option<f64>,
    },
    /// The agent advertised its available models (and optional current model)
    /// on `session/new`. Emitted once per session so the UI can replace the
    /// hardcoded fallback model list with the agent's live list.
    ModelsAvailable {
        current_model_id: Option<String>,
        models: Vec<super::config::AcpModelEntry>,
    },
    /// The turn completed normally.
    TurnFinished { stop_reason: String },
    /// Agent requires setup (authentication, install, etc.).
    SetupRequired {
        reason: String,
        auth_methods: Vec<String>,
    },
    /// The turn failed with an error.
    Failed { error: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acp_event_variants_are_clone() {
        let event = AcpEvent::AgentMessageDelta("hello".to_string());
        let _cloned = event.clone();
    }
}
