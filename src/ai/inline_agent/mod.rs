//! AI execution boundary for the inline text-editing assistant.

pub mod actions;
pub mod executor;
pub mod history;
pub mod mock;
pub mod privacy;
pub mod prompt;
pub mod session;
pub mod types;

pub use prompt::{build_inline_agent_prompt, InlineAgentPromptAudit};
pub use session::{InlineAgentPhase, InlineAgentSessionCommand};
pub use types::{
    InlineAgentEditSemantics, InlineAgentProviderEvent, InlineAgentProviderRequest,
    InlineAgentSessionId, InlineAgentTurnId,
};
