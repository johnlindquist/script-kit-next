//! Header Raycast Style Variations - Separator & Spacing Explorations
//!
//! 20 variations of the Raycast-style header layout exploring:
//! - Separator styles (|, dots, spacing, none)
//! - Element spacing (tight, normal, loose)
//! - Logo placement and styling
//! - Button groupings
//!
//! All variations use the same theme colors and fonts.
//! The logo is rendered as dark inside a yellow rounded rectangle.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// Story showcasing 20 Raycast-style header variations

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
