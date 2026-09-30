//! The main window's height: one height for the window's mode, kept while it shows. The list,
//! the views under the input, and whatever arrives later all fill that height; typing,
//! filtering, and results landing never resize the window.

use gpui::{AppContext as _, Context};

use crate::{AppView, MainWindowMode, ScriptListApp};

impl ScriptListApp {
    /// The main window's one height in its current mode.
    pub(crate) fn one_window_fixed_height(&self) -> f32 {
        match self.main_window_mode {
            MainWindowMode::Mini => crate::window_resize::main_window_full_height(),
            MainWindowMode::Full => crate::window_resize::current_standard_height(),
        }
    }

    /// Puts the window at its fixed height when what it draws in isn't. Runs on every render of
    /// the main list, so a mode switch or a return from a prompt that left another height is
    /// corrected before the next frame; one request is made per wrong height.
    fn one_window_keep_fixed_height(&mut self, window: &gpui::Window, cx: &mut Context<Self>) {
        if !matches!(self.current_view, AppView::ScriptList) {
            // A prompt sizes the window its own way; coming back must check again.
            self.one_window_host.height = None;
            return;
        }
        let height = self.one_window_fixed_height();
        let shown = f32::from(window.viewport_size().height);
        if (shown - height).abs() < 1.0 || self.one_window_host.height == Some(height) {
            self.one_window_host.height = Some(height);
            return;
        }
        tracing::info!(
            target: "script_kit::one_window",
            event = "main_window_height",
            height_px = height,
            shown_px = shown,
            mode = ?self.main_window_mode,
        );
        self.one_window_host.height = Some(height);
        let handle = window.window_handle();
        cx.defer(move |cx| {
            let _ = cx.update_window(handle, |_, window, cx| {
                if crate::runtime_policy::is_owned_evaluation() {
                    // The owned window is hidden and has no screen edge to keep.
                    let width = window.viewport_size().width;
                    window.resize(gpui::size(width, gpui::px(height)));
                } else {
                    // The native path keeps the top edge, so the input never moves.
                    crate::window_ops::queue_resize_with_width(
                        height,
                        Some(crate::window_resize::MAIN_WINDOW_WIDTH),
                        window,
                        cx,
                    );
                }
            });
        });
    }

    /// The window the main list draws in, where its height is kept.
    pub(crate) fn one_window_rendered_in(&mut self, window: &gpui::Window, cx: &mut Context<Self>) {
        let handle = window.window_handle();
        if self.one_window_host.window != Some(handle) {
            self.one_window_host.window = Some(handle);
            self.one_window_host.height = None;
        }
        self.one_window_keep_fixed_height(window, cx);
    }
}
