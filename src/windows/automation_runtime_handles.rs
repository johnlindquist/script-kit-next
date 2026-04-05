//! Exact runtime window handle registry for automation dispatch.
//!
//! Maps automation window IDs (e.g. `"acpDetached:thread-2"`) to live GPUI
//! [`AnyWindowHandle`] values, so that [`dispatch_gpui_event`] can target a
//! specific window without collapsing back to a shared [`WindowRole`].
//!
//! The automation *metadata* registry (`automation_registry.rs`) stores
//! [`AutomationWindowInfo`] for discovery and targeting.  This module stores
//! the *runtime handle* that GPUI needs to actually deliver events.
//!
//! # Lifecycle
//!
//! - **Upsert** when a window is created and its automation ID is known.
//! - **Remove** when the window closes.
//! - **Validate** with [`get_valid_runtime_window_handle`] before dispatch;
//!   stale handles are evicted automatically.

use gpui::{AnyWindowHandle, App};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;

static RUNTIME_WINDOW_HANDLES: LazyLock<Mutex<HashMap<String, AnyWindowHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Register or update the GPUI window handle for an automation window ID.
pub fn upsert_runtime_window_handle(id: impl Into<String>, handle: AnyWindowHandle) {
    let id = id.into();
    RUNTIME_WINDOW_HANDLES.lock().insert(id.clone(), handle);
    tracing::info!(
        target: "script_kit::automation",
        window_id = %id,
        "automation.runtime_handle_upserted"
    );
}

/// Remove the runtime handle for an automation window ID.
///
/// Returns `true` if a handle was present and removed.
pub fn remove_runtime_window_handle(id: &str) -> bool {
    let removed = RUNTIME_WINDOW_HANDLES.lock().remove(id).is_some();
    if removed {
        tracing::info!(
            target: "script_kit::automation",
            window_id = %id,
            "automation.runtime_handle_removed"
        );
    }
    removed
}

/// Get the runtime handle for an automation window ID without validation.
pub fn get_runtime_window_handle(id: &str) -> Option<AnyWindowHandle> {
    RUNTIME_WINDOW_HANDLES.lock().get(id).copied()
}

/// Get the runtime handle for an automation window ID, validating that it
/// still refers to a live GPUI window.  Stale handles are evicted.
pub fn get_valid_runtime_window_handle(id: &str, cx: &mut App) -> Option<AnyWindowHandle> {
    let handle = get_runtime_window_handle(id)?;
    match handle.update(cx, |_, _, _| {}) {
        Ok(_) => Some(handle),
        Err(_) => {
            remove_runtime_window_handle(id);
            tracing::warn!(
                target: "script_kit::automation",
                window_id = %id,
                "automation.runtime_handle_stale"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_and_remove_round_trips() {
        let id = "test_rt_handle_upsert_remove";
        // Nothing registered yet
        assert!(get_runtime_window_handle(id).is_none());

        // Remove on missing ID returns false
        assert!(!remove_runtime_window_handle(id));

        // We can't fabricate a real AnyWindowHandle without GPUI context,
        // so this test only verifies the map operations via remove.
    }

    #[test]
    fn remove_returns_false_when_absent() {
        assert!(!remove_runtime_window_handle("nonexistent_rt_handle_test"));
    }
}
