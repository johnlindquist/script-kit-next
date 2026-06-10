//! Legacy AI execution boundary for the deprecated inline-agent overlay.
//!
//! Production focused-text entry uses the main-window Agent Chat Text profile
//! path in `app_impl::agent_handoff`, while this module remains for compatibility
//! with the old overlay and its tests.

pub mod actions;
pub(crate) mod agent_chat_adapter;
pub mod executor;
pub mod history;
pub mod mock;
pub mod privacy;
pub mod prompt;
pub mod session;
pub mod types;

pub use prompt::{build_inline_agent_prompt, InlineAgentPromptAudit};
pub use session::{
    InlineAgentPhase, InlineAgentRetryRequest, InlineAgentSession, InlineAgentSessionCommand,
    InlineAgentStreamState,
};
pub use types::{
    InlineAgentEditSemantics, InlineAgentProviderEvent, InlineAgentProviderRequest,
    InlineAgentSessionId, InlineAgentTurnId,
};
