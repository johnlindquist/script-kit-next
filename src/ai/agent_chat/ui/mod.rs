//! Canonical Agent Chat UI boundary.
//!
//! This is the feature-level import surface for the Agent Chat chat UI. The
//! implementation still lives under `crate::ai::acp` while the ACP-named
//! source, automation, and serialization contracts remain frozen for
//! compatibility (action IDs, route IDs, `getAcpState`, serialized surface
//! IDs, telemetry labels). New code should import `AgentChat*` types from here
//! instead of reaching into `crate::ai::acp` directly, so the eventual physical
//! move of the implementation is an internal change behind a stable boundary.

// Forward-looking boundary: the full alias set is established now so later
// slices can migrate outer consumers to `agent_chat::ui` import paths without
// re-touching this file. Not every alias has a consumer yet.
#[allow(unused_imports)]
pub(crate) use crate::ai::acp::{
    open_or_focus_chat_with_input, AcpChatSession as AgentChatSession,
    AcpChatView as AgentChatView, AcpEvent as AgentChatEvent, AcpEventRx as AgentChatEventRx,
    AcpHistoryResumeRequest as AgentChatHistoryResumeRequest,
    AcpInlineSetupState as AgentChatInlineSetupState, AcpLaunchBlocker as AgentChatLaunchBlocker,
    AcpLaunchRequirements as AgentChatLaunchRequirements,
    AcpLaunchResolution as AgentChatLaunchResolution,
    AcpPermissionBroker as AgentChatPermissionBroker, AcpRetryRequest as AgentChatRetryRequest,
    AcpSetupAction as AgentChatSetupAction, AcpThread as AgentChatThread,
    AcpThreadInit as AgentChatThreadInit, AcpThreadMessage as AgentChatThreadMessage,
    AcpThreadStatus as AgentChatThreadStatus, AcpToolCallState as AgentChatToolCallState,
};
