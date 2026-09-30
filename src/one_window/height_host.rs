//! The main window's height: how tall the list wants it, and where the reducer's height
//! decisions put it.

use gpui::{AppContext as _, Context};
use sk_one_window::{HeightDecision, HeightMode};

use crate::{AppView, MainWindowMode, ScriptListApp};

impl ScriptListApp {
    /// How tall the launcher may get in its current mode.
    fn one_window_height_ceiling(&self) -> f32 {
        match self.main_window_mode {
            MainWindowMode::Mini => crate::window_resize::main_window_full_height(),
            MainWindowMode::Full => crate::window_resize::current_standard_height(),
        }
    }

    /// The line above the input, the input, the list as drawn, and the floating footer's zone.
    pub(super) fn one_window_natural_height(&self) -> f32 {
        if self.one_window_work_view_open() {
            return self.one_window_height_ceiling();
        }
        if let Some(px) = self.one_window_view_height() {
            return px.min(self.one_window_height_ceiling());
        }
        if self.menu_syntax_card_owns_list() {
            return self.one_window_height_ceiling();
        }
        let (grouped, _) = self.get_grouped_results_cached();
        let heights = crate::scrolling::list_geometry::GroupedListRowHeights::for_theme(
            self.current_main_menu_theme,
        );
        let coming = if self.one_window_owns_list() {
            self.one_window.list().rows_coming()
        } else {
            0
        };
        let list = crate::scrolling::list_geometry::content_height(grouped.len(), |ix| {
            heights.row_height(&grouped[ix], ix)
        }) + coming as f32 * heights.item;
        let chrome = sk_one_window::contract::app::SLOT_LINE_PX
            + f32::from(crate::main_list_header_overlay_height(
                self.current_main_menu_theme.def(),
            ))
            + crate::footer_popup::main_window_footer_reservation_height();
        (chrome + list + self.one_window_view_extra_height()).min(self.one_window_height_ceiling())
    }

    /// Whether a command composer or filter guide fills the list area in place of rows; it has no
    /// rows to measure, so it gets the whole window. A typed capture is a row.
    fn menu_syntax_card_owns_list(&self) -> bool {
        let text = self.filter_text.as_str();
        if self.menu_syntax_trigger_picker_state.owns_main_list()
            || self.menu_syntax_object_selector_state.owns_main_list()
            || self.menu_syntax_mode.capture_composer_owns_input_for(text)
        {
            return false;
        }
        (self.menu_syntax_mode.command_owns_input_for(text)
            && !self.spine_projection_owns_main_list())
            || crate::menu_syntax::active_filter_head_owns_main_list(text)
    }

    pub(super) fn one_window_apply_height(
        &mut self,
        decision: HeightDecision,
        cx: &mut Context<Self>,
    ) {
        if decision.mode == HeightMode::Hold && self.one_window_host.height == Some(decision.px) {
            return;
        }
        self.one_window_set_height(decision.px, cx);
    }

    fn one_window_set_height(&mut self, height: f32, cx: &mut Context<Self>) {
        self.one_window_host.height = Some(height);
        crate::window_resize::set_main_window_height(height);
        let Some(handle) = self.one_window_host.window else {
            return;
        };
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }
        let height = height.min(self.one_window_height_ceiling());
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

    /// The window the main list draws in, where its height is applied. A height the reducer
    /// placed before the window first drew is applied now, or the first key would snap to it.
    pub(crate) fn one_window_rendered_in(&mut self, window: &gpui::Window, cx: &mut Context<Self>) {
        let handle = window.window_handle();
        if self.one_window_host.window == Some(handle) {
            return;
        }
        self.one_window_host.window = Some(handle);
        if let Some(height) = self.one_window_host.height {
            self.one_window_set_height(height, cx);
        }
    }
}
