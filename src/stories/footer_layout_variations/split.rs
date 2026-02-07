//! Footer Layout Variations - Raycast-inspired designs
//!
//! Raycast's layout:
//! - Header: Input field + "Ask AI" button (minimal, clean)
//! - List: Results label + items (icon | name | subtitle | type)
//! - Footer: Logo left, contextual action + "↵" + divider + "Actions ⌘K" right
//!
//! This moves the Run/Action buttons OUT of the header into a footer,
//! keeping the header clean and focused on input.

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

include!("split/part_01.rs");
include!("split/part_02.rs");
include!("split/part_03.rs");
include!("split/part_04.rs");
