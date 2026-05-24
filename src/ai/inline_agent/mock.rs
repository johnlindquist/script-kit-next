use async_channel::Receiver;

use super::executor::InlineAgentExecutor;
use super::types::{
    InlineAgentProviderEvent, InlineAgentProviderRequest, InlineAgentSessionId, InlineAgentTurnId,
};

#[derive(Debug, Default)]
pub struct MockInlineAgentExecutor;

impl InlineAgentExecutor for MockInlineAgentExecutor {
    fn start_turn(
        &self,
        request: InlineAgentProviderRequest,
    ) -> anyhow::Result<Receiver<InlineAgentProviderEvent>> {
        let (tx, rx) = async_channel::bounded(4);
        let _ = tx.try_send(InlineAgentProviderEvent::AgentMessageDelta {
            text: request.instruction,
        });
        let _ = tx.try_send(InlineAgentProviderEvent::TurnFinished);
        Ok(rx)
    }

    fn cancel_turn(
        &self,
        _session_id: InlineAgentSessionId,
        _turn_id: InlineAgentTurnId,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
