use super::types::InlineAgentEditSemantics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentPromptAudit {
    pub session_id: String,
    pub app_bundle_id: Option<String>,
    pub semantics: InlineAgentEditSemantics,
    pub turn_count: usize,
    pub capture_char_count: usize,
    pub completion_status: String,
}
