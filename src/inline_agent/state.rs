use super::types::InlineAgentOutputAction;

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
    },
    Applying {
        action: InlineAgentOutputAction,
    },
    Applied {
        action: InlineAgentOutputAction,
    },
}

impl InlineAgentRunState {
    pub fn latest_complete_output(&self) -> Option<&str> {
        match self {
            Self::Completed { output } => Some(output),
            _ => None,
        }
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
