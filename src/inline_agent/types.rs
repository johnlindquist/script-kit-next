use crate::platform::accessibility::{
    ActiveAppIdentity, FocusedFieldGeometry, FocusedTextCapabilities, FocusedTextSessionId,
    TextMetrics,
};

pub const INLINE_AGENT_INPUT_PLACEHOLDER: &str = "Edit, refine, ask...";

#[derive(Debug, Clone, PartialEq)]
pub struct InlineAgentSnapshot {
    pub session_id: FocusedTextSessionId,
    pub app: ActiveAppIdentity,
    pub text: String,
    pub metrics: TextMetrics,
    pub capabilities: FocusedTextCapabilities,
    pub anchor: InlineAgentAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineAgentAnchor {
    pub geometry: FocusedFieldGeometry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineAgentOutputAction {
    Replace,
    Append,
    Copy,
    Chat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineAgentTextMutation {
    Replace { text: String },
    Append { text: String },
    Copy { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentMutationReceipt {
    pub action: InlineAgentOutputAction,
    pub success: bool,
    pub message: Option<String>,
}
