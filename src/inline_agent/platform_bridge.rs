use anyhow::Result;

use super::types::{
    InlineAgentAnchor, InlineAgentMutationReceipt, InlineAgentSnapshot, InlineAgentTextMutation,
};
use crate::platform::accessibility::mutation::TextMutationAction;
use crate::platform::accessibility::{
    append_focused_text, capture_focused_text_field, copy_text_output, replace_focused_text,
    CaptureFocusedTextOptions, TextMutationOptions, TextMutationResult,
};

pub trait InlineAgentPlatformBridge {
    fn capture_focused_text_snapshot(&self) -> Result<InlineAgentSnapshot>;

    fn apply_text_mutation(
        &self,
        anchor: &InlineAgentAnchor,
        mutation: InlineAgentTextMutation,
    ) -> Result<InlineAgentMutationReceipt>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemInlineAgentPlatformBridge;

impl InlineAgentPlatformBridge for SystemInlineAgentPlatformBridge {
    fn capture_focused_text_snapshot(&self) -> Result<InlineAgentSnapshot> {
        Ok(capture_focused_text_field(CaptureFocusedTextOptions::default())?.into())
    }

    fn apply_text_mutation(
        &self,
        _anchor: &InlineAgentAnchor,
        mutation: InlineAgentTextMutation,
    ) -> Result<InlineAgentMutationReceipt> {
        let result = match mutation {
            InlineAgentTextMutation::Replace { session_id, text } => {
                replace_focused_text(session_id, &text, TextMutationOptions::default())?
            }
            InlineAgentTextMutation::Append { session_id, text } => {
                append_focused_text(session_id, &text, TextMutationOptions::default())?
            }
            InlineAgentTextMutation::Copy { text } => copy_text_output(&text)?,
        };

        Ok(mutation_result_to_receipt(result))
    }
}

fn mutation_result_to_receipt(result: TextMutationResult) -> InlineAgentMutationReceipt {
    InlineAgentMutationReceipt {
        action: match result.action {
            TextMutationAction::Replace => super::types::InlineAgentOutputAction::Replace,
            TextMutationAction::Append => super::types::InlineAgentOutputAction::Append,
            TextMutationAction::Copy => super::types::InlineAgentOutputAction::Copy,
        },
        success: result.changed_text || result.copied_to_clipboard,
        changed_text: result.changed_text,
        copied_to_clipboard: result.copied_to_clipboard,
        message: None,
    }
}
