use super::types::FocusedTextEditSemantics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedTextPromptAudit {
    pub session_id: String,
    pub app_bundle_id: Option<String>,
    pub semantics: FocusedTextEditSemantics,
    pub turn_count: usize,
    pub capture_char_count: usize,
    pub prompt_capture_char_count: usize,
    pub capture_truncated: bool,
    pub completion_status: String,
}
