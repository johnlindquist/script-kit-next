use super::actions::InlineAgentAction;
use super::executor::InlineAgentExecutor;
use super::history::InlineAgentTurn;
use super::prompt::{build_inline_agent_prompt, InlineAgentPromptAudit, InlineAgentPromptRequest};
use super::types::{
    InlineAgentEditSemantics, InlineAgentProviderEvent, InlineAgentProviderRequest,
    InlineAgentSessionId, InlineAgentTurnId,
};
use crate::platform::accessibility::FocusedTextSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingInlineAgentTurn {
    turn_id: InlineAgentTurnId,
    instruction: String,
    semantics: InlineAgentEditSemantics,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentStreamState {
    pub phase: InlineAgentPhase,
    pub visible_output: String,
    pub thought_log: String,
    pub latest_complete_output: Option<String>,
    pub error: Option<String>,
}

impl Default for InlineAgentStreamState {
    fn default() -> Self {
        Self {
            phase: InlineAgentPhase::Ready,
            visible_output: String::new(),
            thought_log: String::new(),
            latest_complete_output: None,
            error: None,
        }
    }
}

impl InlineAgentStreamState {
    pub fn start_turn(&mut self) {
        self.phase = InlineAgentPhase::Thinking;
        self.visible_output.clear();
        self.thought_log.clear();
        self.error = None;
    }

    pub fn apply_provider_event(&mut self, event: InlineAgentProviderEvent) {
        match event {
            InlineAgentProviderEvent::AgentMessageDelta { text } => {
                if !text.is_empty() {
                    self.phase = InlineAgentPhase::Streaming;
                    self.visible_output.push_str(&text);
                }
            }
            InlineAgentProviderEvent::AgentThoughtDelta { text } => {
                self.thought_log.push_str(&text);
                if self.phase != InlineAgentPhase::Streaming {
                    self.phase = InlineAgentPhase::Thinking;
                }
            }
            InlineAgentProviderEvent::UsageUpdated => {}
            InlineAgentProviderEvent::TurnFinished => {
                self.phase = InlineAgentPhase::Complete;
                self.latest_complete_output = Some(self.visible_output.clone());
            }
            InlineAgentProviderEvent::Failed { message } => {
                self.phase = InlineAgentPhase::Error;
                self.error = Some(message);
            }
        }
    }

    pub fn cancel(&mut self) {
        self.phase = InlineAgentPhase::Cancelling;
        self.visible_output.clear();
        self.error = None;
    }
}

#[derive(Debug, Clone)]
pub struct InlineAgentSession {
    pub snapshot: FocusedTextSnapshot,
    pub stream: InlineAgentStreamState,
    pub history: Vec<InlineAgentTurn>,
    active_turn: Option<PendingInlineAgentTurn>,
    next_turn_index: usize,
}

impl InlineAgentSession {
    pub fn new(snapshot: FocusedTextSnapshot) -> Self {
        Self {
            snapshot,
            stream: InlineAgentStreamState::default(),
            history: Vec::new(),
            active_turn: None,
            next_turn_index: 1,
        }
    }

    pub fn begin_turn(
        &mut self,
        instruction: impl Into<String>,
        semantics: InlineAgentEditSemantics,
        executor: &dyn InlineAgentExecutor,
    ) -> anyhow::Result<(
        async_channel::Receiver<InlineAgentProviderEvent>,
        InlineAgentPromptAudit,
    )> {
        let instruction = instruction.into();
        let turn_id = InlineAgentTurnId(format!(
            "{}-turn-{}",
            self.snapshot.session_id, self.next_turn_index
        ));
        self.next_turn_index += 1;

        let (prompt, audit) = build_inline_agent_prompt(InlineAgentPromptRequest {
            snapshot: &self.snapshot,
            instruction: &instruction,
            semantics,
            previous_turns: &self.history,
        });

        let receiver = executor.start_turn(InlineAgentProviderRequest {
            session_id: InlineAgentSessionId(self.snapshot.session_id.to_string()),
            turn_id: turn_id.clone(),
            instruction: instruction.clone(),
            prompt,
        })?;

        self.active_turn = Some(PendingInlineAgentTurn {
            turn_id,
            instruction,
            semantics,
        });
        self.stream.start_turn();

        Ok((receiver, audit))
    }

    pub fn apply_provider_event(&mut self, event: InlineAgentProviderEvent) {
        let finished = matches!(event, InlineAgentProviderEvent::TurnFinished);
        let failed = matches!(event, InlineAgentProviderEvent::Failed { .. });

        self.stream.apply_provider_event(event);

        if finished {
            if let Some(active_turn) = self.active_turn.take() {
                self.history.push(InlineAgentTurn {
                    instruction: active_turn.instruction,
                    semantics: active_turn.semantics,
                    assistant_output: self.stream.latest_complete_output.clone(),
                });
            }
        } else if failed {
            self.active_turn = None;
        }
    }

    pub fn drain_provider_events(
        &mut self,
        receiver: async_channel::Receiver<InlineAgentProviderEvent>,
    ) {
        while let Ok(event) = receiver.recv_blocking() {
            let terminal = matches!(
                event,
                InlineAgentProviderEvent::TurnFinished | InlineAgentProviderEvent::Failed { .. }
            );
            self.apply_provider_event(event);
            if terminal {
                break;
            }
        }
    }

    pub fn cancel_active_turn(&mut self, executor: &dyn InlineAgentExecutor) -> anyhow::Result<()> {
        if let Some(active_turn) = self.active_turn.take() {
            executor.cancel_turn(
                InlineAgentSessionId(self.snapshot.session_id.to_string()),
                active_turn.turn_id,
            )?;
        }
        self.stream.cancel();
        Ok(())
    }

    pub fn latest_complete_output(&self) -> Option<&str> {
        self.stream.latest_complete_output.as_deref()
    }
}
