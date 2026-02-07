//! PathPrompt component stories for the storybook
//!
//! Showcases different states and variations of the file/folder picker:
//! - File browser view
//! - Folder selection
//! - With breadcrumbs
//! - With file icons
//! - Filtered view
//! - Search state

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;

// Story showcasing the PathPrompt component variations

include!("split/part_01.rs");
include!("split/part_02.rs");
