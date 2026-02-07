//! UI Foundation - Shared UI patterns for consistent vibrancy and layout
//!
//! This module extracts common UI patterns from the main menu (render_script_list.rs)
//! into reusable helpers. The main menu is the "gold standard" for vibrancy support.
//!
//! NOTE: Many items are currently unused as this is a foundation module.
//! They will be used as other modules are refactored to use the shared patterns.
#![allow(dead_code)]
//!
//! # Key Vibrancy Pattern (from render_script_list.rs:699-707)
//!
//! ```ignore
//! // VIBRANCY: Remove background from content div - let gpui-component Root's
//! // semi-transparent background handle vibrancy effect. Content areas should NOT
//! // have their own backgrounds to allow blur to show through.
//! let _bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
//!
//! let mut main_div = div()
//!     .flex()
//!     .flex_col()
//!     // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
//!     .shadow(box_shadows)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui_foundation::{get_vibrancy_background, container_div, content_div};
//!
//! // In your render function:
//! let bg = get_vibrancy_background(&theme);
//! let container = container_div()
//!     .when_some(bg, |d, bg| d.bg(bg))
//!     .child(content_div().child(...));
//! ```

include!("part_000.rs");
include!("part_001.rs");
