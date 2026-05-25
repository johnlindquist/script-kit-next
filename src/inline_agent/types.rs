use crate::platform::accessibility::{
    ActiveAppIdentity, FocusedFieldGeometry, FocusedTextCapabilities, FocusedTextSessionId,
    FocusedTextSnapshot, TextMetrics,
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

impl From<FocusedTextSnapshot> for InlineAgentSnapshot {
    fn from(snapshot: FocusedTextSnapshot) -> Self {
        Self {
            session_id: snapshot.session_id,
            app: snapshot.app,
            text: snapshot.text,
            metrics: snapshot.metrics,
            capabilities: snapshot.capabilities,
            anchor: InlineAgentAnchor {
                geometry: snapshot.geometry,
            },
        }
    }
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
    Replace {
        session_id: FocusedTextSessionId,
        text: String,
    },
    Append {
        session_id: FocusedTextSessionId,
        text: String,
    },
    Copy {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentMutationReceipt {
    pub action: InlineAgentOutputAction,
    pub success: bool,
    pub changed_text: bool,
    pub copied_to_clipboard: bool,
    pub message: Option<String>,
}
