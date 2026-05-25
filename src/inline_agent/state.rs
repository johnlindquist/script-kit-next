use super::types::{InlineAgentMutationReceipt, InlineAgentOutputAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineAgentMode {
    Compact,
    Expanded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineAgentRunState {
    Idle,
    Thinking {
        request_id: String,
        started_at_ms: u64,
    },
    Streaming {
        request_id: String,
        partial_output: String,
    },
    Completed {
        output: String,
    },
    Error {
        message: String,
        retryable: bool,
        latest_output: Option<String>,
    },
    Applying {
        action: InlineAgentOutputAction,
        latest_output: Option<String>,
    },
    Applied {
        action: InlineAgentOutputAction,
        output: String,
        receipt: InlineAgentMutationReceipt,
    },
}

impl InlineAgentRunState {
    pub fn latest_complete_output(&self) -> Option<&str> {
        match self {
            Self::Completed { output } => Some(output),
            Self::Applied { output, .. } => Some(output),
            Self::Applying {
                latest_output: Some(output),
                ..
            } => Some(output),
            Self::Error {
                latest_output: Some(output),
                ..
            } => Some(output),
            _ => None,
        }
    }

    pub fn latest_output_owned(&self) -> Option<String> {
        self.latest_complete_output().map(ToOwned::to_owned)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentState {
    pub mode: InlineAgentMode,
    pub run_state: InlineAgentRunState,
    pub latest_output: Option<String>,
}

impl Default for InlineAgentState {
    fn default() -> Self {
        Self {
            mode: InlineAgentMode::Compact,
            run_state: InlineAgentRunState::Idle,
            latest_output: None,
        }
    }
}

impl InlineAgentState {
    pub fn first_delta(&mut self, request_id: String, delta: &str) {
        self.run_state = InlineAgentRunState::Streaming {
            request_id,
            partial_output: delta.to_string(),
        };
    }

    pub fn finish(&mut self, output: String) {
        self.latest_output = Some(output.clone());
        self.run_state = InlineAgentRunState::Completed { output };
    }

    pub fn expand(&mut self) {
        self.mode = InlineAgentMode::Expanded;
    }

    pub fn collapse(&mut self) {
        self.mode = InlineAgentMode::Compact;
    }
}
