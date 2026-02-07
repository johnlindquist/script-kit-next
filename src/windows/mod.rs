#![allow(unused_imports)]
//! Window management module
//!
//! This module provides a unified window registry for managing multiple windows
//! (Main, Notes, AI) in a consistent way. It replaces the per-window statics
//! with a single registry, ensuring consistent lifecycle handling.

mod registry;
pub mod types;

pub use registry::{
    clear_window, close_window_with_bounds, get_valid_window, get_window, is_window_open,
    notify_all_windows, register_window, take_window, with_window, WindowRole,
};
pub use types::DisplayBounds;
