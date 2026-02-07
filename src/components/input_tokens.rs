//! Shared text/style tokens for input-oriented components.
//!
//! Keeping these in one place prevents drift between `PromptInput` and
//! `ScriptKitInput` defaults/factory presets.

pub const INPUT_PLACEHOLDER_DEFAULT: &str = "Type to search...";
pub const INPUT_PLACEHOLDER_CHAT: &str = "Ask anything...";
pub const INPUT_PLACEHOLDER_SEARCH: &str = "Search...";
pub const INPUT_PLACEHOLDER_MAIN_MENU: &str = "Script Kit";
pub const INPUT_PLACEHOLDER_ARG: &str = "Enter value...";

pub const INPUT_FONT_SIZE_DEFAULT: f32 = 16.0;
pub const INPUT_FONT_SIZE_CHAT: f32 = 14.0;
pub const INPUT_FONT_SIZE_SEARCH: f32 = 16.0;
pub const INPUT_FONT_SIZE_MAIN_MENU: f32 = 18.0;
pub const INPUT_FONT_SIZE_ARG: f32 = 16.0;
