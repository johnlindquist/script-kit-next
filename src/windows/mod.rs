#![allow(unused_imports)]
//! Window management module
//!
//! This module provides a unified window registry for managing multiple windows
//! (Main, Notes, AI) in a consistent way. It replaces the per-window statics
//! with a single registry, ensuring consistent lifecycle handling.
//!
//! The [`automation_registry`] sub-module adds a parallel registry of
//! [`AutomationWindowInfo`](crate::protocol::AutomationWindowInfo) entries
//! used by the stdin automation protocol to address windows by stable ID,
//! kind, title, or focus state.

pub mod automation_registry;
mod registry;
pub mod types;

pub use automation_registry::{
    focused_automation_window_id, list_automation_windows, remove_automation_window,
    resolve_automation_window, set_automation_focus, set_automation_visibility,
    upsert_automation_window,
};
pub use registry::{
    clear_window, close_window_with_bounds, get_valid_window, get_window, is_window_open,
    notify_all_windows, register_window, take_window, with_window, WindowRole,
};
pub use types::DisplayBounds;
