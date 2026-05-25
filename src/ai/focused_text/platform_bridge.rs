use anyhow::Result;

use super::types::{FocusedTextApplyAction, FocusedTextMutation, FocusedTextMutationReceipt};
use crate::platform::accessibility::mutation::TextMutationAction;
use crate::platform::accessibility::{
    append_focused_text, copy_text_output, replace_focused_text, TextMutationOptions,
    TextMutationResult,
};

pub trait FocusedTextPlatformBridge {
    fn apply_text_mutation(
        &self,
        mutation: FocusedTextMutation,
    ) -> Result<FocusedTextMutationReceipt>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemFocusedTextPlatformBridge;

impl FocusedTextPlatformBridge for SystemFocusedTextPlatformBridge {
    fn apply_text_mutation(
        &self,
        mutation: FocusedTextMutation,
    ) -> Result<FocusedTextMutationReceipt> {
        let result = match mutation {
            FocusedTextMutation::Replace { session_id, text } => {
                replace_focused_text(session_id, &text, TextMutationOptions::default())?
            }
            FocusedTextMutation::Append { session_id, text } => {
                append_focused_text(session_id, &text, TextMutationOptions::default())?
            }
            FocusedTextMutation::Copy { text } => copy_text_output(&text)?,
        };

        Ok(mutation_result_to_receipt(result))
    }
}

fn mutation_result_to_receipt(result: TextMutationResult) -> FocusedTextMutationReceipt {
    FocusedTextMutationReceipt {
        action: match result.action {
            TextMutationAction::Replace => FocusedTextApplyAction::Replace,
            TextMutationAction::Append => FocusedTextApplyAction::Append,
            TextMutationAction::Copy => FocusedTextApplyAction::Copy,
        },
        success: result.changed_text || result.copied_to_clipboard,
        changed_text: result.changed_text,
        copied_to_clipboard: result.copied_to_clipboard,
        message: None,
    }
}
