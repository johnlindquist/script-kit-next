//! Window Control module using macOS Accessibility APIs
//!
//! This module provides window management functionality including:
//! - Listing all visible windows with their properties
//! - Moving, resizing, minimizing, maximizing, and closing windows
//! - Tiling windows to predefined positions (halves, quadrants, fullscreen)
//!
//! ## Architecture
//!
//! Uses macOS Accessibility APIs (AXUIElement) to control windows across applications.
//! The accessibility framework allows querying and modifying window properties for any
//! application, provided the user has granted accessibility permissions.
//!
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility
//!

#![allow(non_upper_case_globals)]
#![allow(dead_code)]

mod actions;
mod ax;
mod cache;
mod cf;
mod display;
mod ffi;
mod query;
mod tiling;
mod types;

use ffi::AXUIElementRef;

pub use actions::{
    close_window, focus_window, maximize_window, minimize_window, move_to_next_display,
    move_to_previous_display, move_window, resize_window, tile_window,
};
pub use query::{get_frontmost_window_of_previous_app, has_accessibility_permission, list_windows};
pub use types::*;
