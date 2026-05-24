use std::path::PathBuf;

use agent_client_protocol::ContentBlock;
use anyhow::Result;

use crate::ai::agent_chat::events::AgentChatEventRx;

#[derive(Debug, Clone)]
pub(crate) struct AgentChatTurnRequest {
    pub ui_thread_id: String,
    pub cwd: PathBuf,
    pub blocks: Vec<ContentBlock>,
    pub model_id: Option<String>,
}

pub(crate) trait AgentChatConnection: Send + Sync + 'static {
    fn start_turn(&self, request: AgentChatTurnRequest) -> Result<AgentChatEventRx>;
    fn cancel_turn(&self, ui_thread_id: String) -> Result<()>;
    fn prepare_session(&self, ui_thread_id: String, cwd: PathBuf) -> Result<AgentChatEventRx>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_chat_connection_trait_is_object_safe() {
        fn accepts_trait_object(_: Option<&dyn AgentChatConnection>) {}
        accepts_trait_object(None);
    }
}
