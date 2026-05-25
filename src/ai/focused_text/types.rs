use std::fmt;

use crate::platform::accessibility::FocusedTextSessionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedTextEditSemantics {
    Replace,
    Append,
    Explain,
    Chat,
}

impl FocusedTextEditSemantics {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Replace => "replace",
            Self::Append => "append",
            Self::Explain => "explain",
            Self::Chat => "chat",
        }
    }
}

impl fmt::Display for FocusedTextEditSemantics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedTextApplyAction {
    Replace,
    Append,
    Copy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedTextMutation {
    Replace {
        session_id: FocusedTextSessionId,
        text: String,
    },
    Append {
        session_id: FocusedTextSessionId,
        text: String,
    },
    Copy {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedTextMutationReceipt {
    pub action: FocusedTextApplyAction,
    pub success: bool,
    pub changed_text: bool,
    pub copied_to_clipboard: bool,
    pub message: Option<String>,
}
