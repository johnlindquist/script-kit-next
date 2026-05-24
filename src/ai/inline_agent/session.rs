use super::actions::InlineAgentAction;
use super::types::InlineAgentEditSemantics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineAgentSessionCommand {
    Submit {
        instruction: String,
        semantics: InlineAgentEditSemantics,
    },
    CancelActiveTurn,
    RetryLastTurn,
    Expand,
    Collapse,
    ApplyLatest(InlineAgentAction),
    Dismiss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineAgentPhase {
    Capturing,
    Ready,
    Thinking,
    Streaming,
    Cancelling,
    Complete,
    Error,
}
