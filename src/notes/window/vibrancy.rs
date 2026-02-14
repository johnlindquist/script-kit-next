use super::*;

impl NotesApp {
    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================
    // These use the same approach as the main window (render_script_list.rs)
    // to ensure vibrancy works correctly by using rgba() with hex colors
    // directly from the Script Kit theme.
    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get background color with vibrancy opacity applied
    ///
    /// When vibrancy is enabled, backgrounds need to be semi-transparent
    /// to show the blur effect behind them. This helper returns the
    /// theme background color with the appropriate opacity from config.
    pub(super) fn get_vibrancy_background(_cx: &Context<Self>) -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get modal overlay background (theme-aware)
    ///
    /// For dark mode: black overlay (darkens content behind)
    /// For light mode: white overlay (keeps content readable on light backgrounds)
    /// 50% opacity (0x80) for good contrast without being too heavy
    pub(super) fn get_modal_overlay_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        crate::theme::modal_overlay_bg(&sk_theme, 0x80)
    }

    pub(super) fn set_mouse_cursor_hidden_state(
        mouse_cursor_hidden: &mut bool,
        hidden: bool,
    ) -> bool {
        if *mouse_cursor_hidden == hidden {
            return false;
        }
        *mouse_cursor_hidden = hidden;
        true
    }

    pub(super) fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if Self::set_mouse_cursor_hidden_state(&mut self.mouse_cursor_hidden, true) {
            crate::platform::hide_cursor_until_mouse_moves();
            cx.notify();
        }
    }

    pub(super) fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if Self::set_mouse_cursor_hidden_state(&mut self.mouse_cursor_hidden, false) {
            cx.notify();
        }
    }
}
