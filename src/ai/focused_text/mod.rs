//! Focused-text Agent Chat primitives shared by text-capture entry paths
//! and the main-window focused-text mini Agent Chat migration.

pub mod platform_bridge;
pub mod privacy;
pub mod prompt;
pub mod types;

pub use platform_bridge::{FocusedTextPlatformBridge, SystemFocusedTextPlatformBridge};
pub use privacy::FocusedTextPromptAudit;
pub use prompt::{
    build_focused_text_prompt, build_focused_text_prompt_with_angle, FocusedTextPromptAngle,
    FocusedTextPromptRequest, FocusedTextTurnSummary,
};
pub use types::{
    FocusedTextApplyAction, FocusedTextEditSemantics, FocusedTextMutation,
    FocusedTextMutationReceipt,
};
