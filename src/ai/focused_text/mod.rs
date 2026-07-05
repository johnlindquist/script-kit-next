//! Focused-text Agent Chat primitives shared by text-capture entry paths
//! and the main-window focused-text mini Agent Chat migration.

pub mod platform_bridge;
pub mod privacy;
pub mod prompt;
pub mod types;

pub use platform_bridge::{FocusedTextPlatformBridge, SystemFocusedTextPlatformBridge};

/// Instruction used by the instant rewrite flow (rewrite hotkey / "Rewrite
/// selection" chip) when firing variations without the user typing anything.
/// Kept short because it doubles as the visible user prompt in the mini UI.
pub const DEFAULT_REWRITE_INSTRUCTION: &str = "Improve this text: fix grammar, spelling, \
punctuation, and clarity. Keep the original meaning, tone, language, and formatting.";
pub use privacy::FocusedTextPromptAudit;
pub use prompt::{
    build_focused_text_prompt, build_focused_text_prompt_with_angle, FocusedTextPromptAngle,
    FocusedTextPromptRequest, FocusedTextTurnSummary,
};
pub use types::{
    FocusedTextApplyAction, FocusedTextEditSemantics, FocusedTextMutation,
    FocusedTextMutationReceipt,
};
