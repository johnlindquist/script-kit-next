//! Frontmost Application Tracker
//!
//! Tracks the "last real application" that was active before Script Kit.
//! This module provides a global, always-updated view of what app the user
//! was working in, which is useful for:
//!
//! - **Menu Bar Actions**: Get menu items from the app the user was in
//! - **Window Tiling**: Tile/move windows of the previous app
//! - **Context Actions**: Any action that should target "the app I was just using"
//!
//! ## Architecture
//!
//! A background observer watches for `NSWorkspaceDidActivateApplicationNotification`.
//! When an app activates:
//! - If it's NOT Script Kit → update the tracked "last real app"
//! - If it IS Script Kit → ignore (keep tracking the previous app)
//!
//! This means when Script Kit opens, we already know which app was active,
//! with no race conditions or timing issues.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::frontmost_app_tracker::{start_tracking, get_last_real_app, get_cached_menu_items};
//!
//! // Start tracking (call once at app startup)
//! start_tracking();
//!
//! // Get the last real app info
//! if let Some(app) = get_last_real_app() {
//!     println!("Last app: {} ({})", app.name, app.bundle_id);
//! }
//!
//! // Get cached menu items (pre-fetched in background)
//! let menu_items = get_cached_menu_items();
//! ```

include!("part_000.rs");
include!("part_001.rs");
