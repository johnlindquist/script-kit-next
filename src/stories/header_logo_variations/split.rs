//! Header Logo Variations
//!
//! 20 variations exploring logo size and placement in the header.
//! Uses the "No Separators" style with golden ratio logo as baseline.
//!
//! Variations explore:
//! - Logo sizes (container and SVG dimensions)
//! - Logo placement (left, right, after title)
//! - Spacing between logo and other elements
//! - Corner radius options

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// Story showcasing 20 header logo variations

include!("split/part_01.rs");
include!("split/part_02.rs");
