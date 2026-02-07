//! DropPrompt Story Variations
//!
//! Showcases the DropPrompt component in various states:
//! - Empty state (waiting for drop)
//! - Drag hover state (files being dragged over)
//! - Files dropped (showing file list)
//! - Different file types
//! - Multiple files

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;

// Story showcasing DropPrompt variations

include!("split/part_01.rs");
include!("split/part_02.rs");
