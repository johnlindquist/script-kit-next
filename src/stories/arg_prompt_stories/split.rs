//! ArgPrompt Component Stories
//!
//! Showcases ArgPrompt variations including:
//! - Basic text input
//! - With placeholder text
//! - With validation states
//! - With hints
//! - Focused/unfocused states

use gpui::*;

use crate::storybook::{
    code_block, story_container, story_divider, story_section, Story, StoryVariant,
};
use crate::theme::Theme;

// Story showcasing ArgPrompt component variations

include!("split/part_01.rs");
include!("split/part_02.rs");
