pub(crate) const AGENT_CHAT_LABEL: &str = "Agent Chat";
pub(crate) const AGENT_CHAT_CMD_ENTER_HINT: &str = "⌘↵ Agent Chat";
pub(crate) const AGENT_CHAT_OPEN_ACTION: &str = "Open Agent Chat";
pub(crate) const AGENT_CHAT_CHANGE_AGENT: &str = "Change Agent";
pub(crate) const AGENT_CHAT_CHANGE_MODEL: &str = "Change Model";

pub(crate) fn acp_entry_hint(_origin: &str) -> &'static str {
    AGENT_CHAT_CMD_ENTER_HINT
}
