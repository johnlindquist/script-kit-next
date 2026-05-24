use async_channel::Receiver;

use super::types::{
    InlineAgentProviderEvent, InlineAgentProviderRequest, InlineAgentSessionId, InlineAgentTurnId,
};

pub trait InlineAgentExecutor: Send + Sync {
    fn start_turn(
        &self,
        request: InlineAgentProviderRequest,
    ) -> anyhow::Result<Receiver<InlineAgentProviderEvent>>;

    fn cancel_turn(
        &self,
        session_id: InlineAgentSessionId,
        turn_id: InlineAgentTurnId,
    ) -> anyhow::Result<()>;
}
