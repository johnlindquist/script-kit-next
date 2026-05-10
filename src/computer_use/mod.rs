//! Computer-use vocabulary for agent-facing desktop automation.
//!
//! This module is intentionally small: it maps computer-use language onto
//! Script Kit's existing state-first automation inspection protocol without
//! introducing a second targeting or screenshot model.

pub mod gpui_runtime_bridge;
pub mod native_window_capture;
pub mod runtime_bridge;
pub mod see;
pub mod types;
pub mod window_observation;
