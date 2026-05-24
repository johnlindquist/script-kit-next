#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineAgentAction {
    Replace,
    Append,
    Copy,
}
