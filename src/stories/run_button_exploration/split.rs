//! Run Button Exploration - 50+ Variations
//!
//! The challenge: The "Run" button changes text based on context:
//! - "Run" for scripts
//! - "Submit" for forms
//! - "Select" for choices
//! - "Open Chrome" for app launchers
//! - etc.
//!
//! This creates layout instability and visual clutter.
//! We want the header to feel simple, not busy.
//!
//! This story explores every possible approach to solve this.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;
use crate::utils;

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
include!("split/part_04.rs");
include!("split/part_05.rs");
include!("split/part_06.rs");
include!("split/part_07.rs");
include!("split/part_08.rs");
include!("split/part_09.rs");
include!("split/part_10.rs");
