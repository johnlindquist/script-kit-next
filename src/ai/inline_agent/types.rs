use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineAgentSessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineAgentTurnId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineAgentEditSemantics {
    Replace,
    Append,
    Explain,
    Chat,
}

impl InlineAgentEditSemantics {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Replace => "replace",
            Self::Append => "append",
            Self::Explain => "explain",
            Self::Chat => "chat",
        }
    }
}

impl fmt::Display for InlineAgentEditSemantics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentProviderRequest {
    pub session_id: InlineAgentSessionId,
    pub turn_id: InlineAgentTurnId,
    pub instruction: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineAgentProviderEvent {
    AgentMessageDelta { text: String },
    AgentThoughtDelta { text: String },
    UsageUpdated,
    TurnFinished,
    Failed { message: String },
}
