//! Actions Window Variations - Raycast-style action panel designs
//!
//! This story explores 20 variations of the Actions Window design,
//! heavily inspired by Raycast's action panel UI:
//! - Header with context title
//! - Action items with icons, labels, and keyboard shortcut keycaps
//! - Search input (typically at bottom)
//! - Footer with primary action and Actions shortcut
//!
//! Reference: Raycast's ⌘K action panel

use gpui::*;

use crate::storybook::{story_container, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
include!("split/part_04.rs");
include!("split/part_05.rs");
include!("split/part_06.rs");
