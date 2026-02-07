//! Enhanced Window Control Module
//!
//! This module builds on the base `window_control` module to provide:
//! - **Coordinate foundation**: Standardized WindowBounds in AX coordinates (top-left origin)
//! - **Capability detection**: Settability checks via AXUIElementIsAttributeSettable
//! - **Enhanced window info**: EnhancedWindowInfo with capabilities (can_move/can_resize)
//! - **Multi-monitor support**: DisplayInfo with proper overlap-based display detection
//! - **Spaces backend**: Optional SpaceManager trait (defaults to Unsupported)
//!
//! ## Coordinate System
//!
//! Internally, all bounds use **AX coordinates** (top-left origin, Y grows downward).
//! This matches what `kAXPositionAttribute` get/set operations expect.
//!
//! AppKit's `NSScreen.visibleFrame` uses bottom-left origin, so we convert it
//! to AX coordinates when building `DisplayInfo`.
//!
//! ## API Visibility Contract
//!
//! These internals are crate-only and should not leak as part of the public API.
//! ```compile_fail
//! use script_kit_gpui::window_control_enhanced::WindowBounds;
//! use script_kit_gpui::window_control_enhanced::detect_window_capabilities;
//! ```

#![allow(dead_code)]

mod bounds;
mod capabilities;
mod coords;
mod display;
mod spaces;
