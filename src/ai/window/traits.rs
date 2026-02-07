use super::*;

impl Focusable for AiApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Drop for AiApp {
    fn drop(&mut self) {
        // Clear the global window handle when AiApp is dropped
        // This ensures is_ai_window_open() returns false after the window closes
        // regardless of how it was closed (Cmd+W, traffic light, toggle, etc.)
        if let Some(window_handle) = AI_WINDOW.get() {
            if let Ok(mut guard) = window_handle.lock() {
                *guard = None;
                tracing::debug!("AiApp dropped - cleared global window handle");
            }
        }

        // Restore accessory app mode when AI window closes
        // This removes the app from Cmd+Tab and Dock (back to normal Script Kit behavior)
        // SAFETY: This runs on main thread (GPUI window lifecycle is main-thread only)
        crate::platform::set_accessory_app_mode();
    }
}
