//! Menu Bar Reader module using macOS Accessibility APIs
//!
//! This module provides menu bar scanning functionality including:
//! - Reading the menu bar of the frontmost application
//! - Parsing menu item titles, keyboard shortcuts, and hierarchy
//! - Detecting menu separators
//! - Caching scanned menus for performance
//!
//! ## Architecture
//!
//! Uses macOS Accessibility APIs (AXUIElement) to read menu bar structure.
//! The hierarchy is: AXApplication -> AXMenuBar -> AXMenuBarItem -> AXMenu -> AXMenuItem
//!
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility
//!
//! ## Usage
//!
//! ```ignore
//! use script_kit_gpui::menu_bar::{get_frontmost_menu_bar, MenuBarItem};
//!
//! let items = get_frontmost_menu_bar()?;
//! for item in items {
//!     println!("{}: {:?}", item.title, item.shortcut);
//! }
//! ```

// Note: #[cfg(target_os = "macos")] is applied at the lib.rs level
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");
