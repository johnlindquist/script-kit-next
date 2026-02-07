//! EnvPrompt Story Variations
//!
//! Showcases the EnvPrompt component in various configurations:
//! - Basic environment variable input
//! - Masked/hidden value input (for secrets)
//! - With validation states
//! - With suggestions/hints

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;

// Story showcasing EnvPrompt variations

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
