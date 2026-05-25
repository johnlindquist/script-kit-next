//! Focused-text Agent Chat primitives shared by the legacy Inline Agent path
//! and the main-window focused-text mini Agent Chat migration.

pub mod platform_bridge;
pub mod privacy;
pub mod prompt;
pub mod types;

pub use platform_bridge::{FocusedTextPlatformBridge, SystemFocusedTextPlatformBridge};
pub use privacy::FocusedTextPromptAudit;
pub use prompt::{build_focused_text_prompt, FocusedTextPromptRequest, FocusedTextTurnSummary};
pub use types::{
    FocusedTextApplyAction, FocusedTextEditSemantics, FocusedTextMutation,
    FocusedTextMutationReceipt,
};
