//! Header Design Variations - Layout Explorations
//!
//! 20 different header layout variations exploring different arrangements
//! of: input, Ask AI hint, buttons, logo, separators, and spacing.
//!
//! All variations use the same theme colors and fonts - only layout differs.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// Story showcasing 20 header layout variations

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
