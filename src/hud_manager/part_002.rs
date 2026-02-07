/// Clean up expired HUD windows and show pending ones
fn cleanup_expired_huds(cx: &mut App) {
    let manager = get_hud_manager();
    let mut state = manager.lock();

    // Remove expired HUDs from tracking and release their slots
    let before_count = state.active_huds.len();
    let expired_ids: Vec<u64> = state
        .active_huds
        .iter()
        .filter(|hud| hud.is_expired())
        .map(|hud| hud.id)
        .collect();

    // Release slots for expired HUDs
    for id in &expired_ids {
        state.release_slot_by_id(*id);
    }

    state.active_huds.retain(|hud| !hud.is_expired());
    let removed = before_count - state.active_huds.len();

    if removed > 0 {
        logging::log("HUD", &format!("Cleaned up {} expired HUD(s)", removed));
    }

    // Show pending HUDs if we have free slots
    while state.first_free_slot().is_some() {
        if let Some(pending) = state.pending_queue.pop_front() {
            // Drop lock before showing HUD (show_notification will acquire it)
            drop(state);
            // Use show_notification to preserve action_label and action
            show_notification(pending, cx);
            // Re-acquire for next iteration
            state = manager.lock();
        } else {
            break;
        }
    }
}
/// Dismiss all active HUDs immediately
///
/// This closes all active HUD windows and clears the pending queue.
/// Must be called on the main thread (i.e., from within App context).
#[allow(dead_code)]
pub fn dismiss_all_huds(cx: &mut App) {
    let manager = get_hud_manager();

    // Collect window handles first, then close windows
    let windows_to_close: Vec<WindowHandle<HudView>> = {
        let mut state = manager.lock();
        let windows: Vec<_> = state.active_huds.drain(..).map(|hud| hud.window).collect();
        state.hud_slots = [None; MAX_SIMULTANEOUS_HUDS]; // Clear all slots
        state.pending_queue.clear();
        windows
    };

    let count = windows_to_close.len();

    // Close each window using GPUI's proper API
    for window_handle in windows_to_close {
        let _ = window_handle.update(cx, |_view, window, _cx| {
            window.remove_window();
        });
    }

    if count > 0 {
        logging::log("HUD", &format!("Dismissed {} active HUD(s)", count));
    }
}
