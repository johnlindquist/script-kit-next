use super::types::InlineAgentEditSemantics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAgentTurn {
    pub instruction: String,
    pub semantics: InlineAgentEditSemantics,
    pub assistant_output: Option<String>,
}
