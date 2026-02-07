//! Global keyboard monitoring using macOS CGEventTap API
//!
//! This module provides system-wide keyboard event capture, regardless of which
//! application has focus. This is essential for text expansion/snippet features
//! that need to detect trigger sequences typed in any application.
//!
//! # Requirements
//! - macOS only (uses Core Graphics CGEventTap)
//! - Requires Accessibility permissions to be enabled in System Preferences
//!
//! # Example
//! ```no_run
//! use script_kit_gpui::keyboard_monitor::{KeyboardMonitor, KeyEvent};
//!
//! let mut monitor = KeyboardMonitor::new(|event: KeyEvent| {
//!     println!("Key pressed: {:?}", event.character);
//! });
//!
//! monitor.start().expect("Failed to start keyboard monitor");
//! // ... monitor runs in background thread ...
//! monitor.stop();
//! ```

include!("part_000.rs");
include!("part_001.rs");
