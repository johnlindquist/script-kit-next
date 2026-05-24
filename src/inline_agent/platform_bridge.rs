use anyhow::Result;

use super::types::{
    InlineAgentAnchor, InlineAgentMutationReceipt, InlineAgentSnapshot, InlineAgentTextMutation,
};

pub trait InlineAgentPlatformBridge {
    fn capture_focused_text_snapshot(&self) -> Result<InlineAgentSnapshot>;

    fn apply_text_mutation(
        &self,
        anchor: &InlineAgentAnchor,
        mutation: InlineAgentTextMutation,
    ) -> Result<InlineAgentMutationReceipt>;
}
