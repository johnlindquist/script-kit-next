//! SelectPrompt component stories for the storybook
//!
//! Showcases variations of the SelectPrompt component:
//! - Single select vs multi-select
//! - With icons and descriptions
//! - With groupings
//! - Different item counts

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};

// Story showcasing the SelectPrompt component variations

include!("split/part_01.rs");
include!("split/part_02.rs");
