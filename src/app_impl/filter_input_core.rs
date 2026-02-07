use super::*;

impl ScriptListApp {
    pub(crate) fn current_view_uses_shared_filter_input(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::ThemeChooserView { .. }
                | AppView::FileSearchView { .. }
        )
    }

    pub(crate) fn sync_builtin_query_state(
        query: &mut String,
        selected_index: &mut usize,
        new_text: &str,
    ) -> bool {
        if query == new_text {
            return false;
        }

        *query = new_text.to_string();
        *selected_index = 0;
        true
    }

    pub(crate) fn clear_builtin_query_state(query: &mut String, selected_index: &mut usize) {
        query.clear();
        *selected_index = 0;
    }

}
