#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentTelemetrySnapshot {
    pub session_id: String,
    pub app_bundle_id: Option<String>,
    pub turn_count: usize,
    pub capture_char_count: usize,
    pub completion_status: String,
}
