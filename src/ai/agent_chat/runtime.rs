use std::path::PathBuf;
use std::sync::Arc;

use crate::ai::agent_chat::content::ContentBlock;
use anyhow::Result;

use crate::ai::agent_chat::events::AgentChatEventRx;

#[derive(Debug, Clone)]
pub(crate) struct AgentChatTurnRequest {
    pub ui_thread_id: String,
    pub cwd: PathBuf,
    pub blocks: Vec<ContentBlock>,
    pub model_id: Option<String>,
}

pub(crate) struct IsolatedTurnHandle {
    pub rx: AgentChatEventRx,
    pub cancel: Option<Arc<std::sync::atomic::AtomicBool>>,
}

impl IsolatedTurnHandle {
    pub(crate) fn signal_cancel(&self) {
        if let Some(flag) = &self.cancel {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

pub(crate) trait AgentChatConnection: Send + Sync + 'static {
    fn start_turn(&self, request: AgentChatTurnRequest) -> Result<AgentChatEventRx>;
    fn start_isolated_turn(&self, request: AgentChatTurnRequest) -> Result<IsolatedTurnHandle> {
        let rx = self.start_turn(request)?;
        Ok(IsolatedTurnHandle { rx, cancel: None })
    }
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
