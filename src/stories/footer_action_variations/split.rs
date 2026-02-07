//! Footer Action Variations - Script Kit branded footer layouts
//!
//! Based on Raycast's design pattern:
//! - Header: Clean input area + Ask AI (minimal)
//! - Footer: Logo left, contextual action + Actions right
//!
//! This story explores 10 variations of the footer design.

use gpui::*;

use crate::storybook::{story_container, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;
use crate::utils;

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
include!("split/part_04.rs");
