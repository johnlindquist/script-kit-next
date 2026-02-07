//! HUD Manager - System-level overlay notifications
//!
//! Creates independent floating windows for HUD messages, similar to Raycast's showHUD().
//! HUDs are:
//! - Separate windows (not part of main app window)
//! - Floating above all other windows
//! - Positioned at bottom-center of the screen containing the mouse
//! - Auto-dismiss after configurable duration
//! - Queued if multiple arrive in sequence

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
