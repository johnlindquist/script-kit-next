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

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
