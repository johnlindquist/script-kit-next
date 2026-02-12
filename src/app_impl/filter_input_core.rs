use super::*;

impl ScriptListApp {
    pub(crate) fn current_view_uses_shared_filter_input(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ScriptList
                | AppView::ClipboardHistoryView { .. }
                | AppView::EmojiPickerView { .. }
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

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_current_view_uses_shared_filter_input_includes_script_list_and_builtin_views() {
        let source = fs::read_to_string("src/app_impl/filter_input_core.rs")
            .expect("Failed to read src/app_impl/filter_input_core.rs");
        let required_views = [
            "AppView::ScriptList",
            "AppView::ClipboardHistoryView",
            "AppView::EmojiPickerView",
            "AppView::AppLauncherView",
            "AppView::WindowSwitcherView",
            "AppView::DesignGalleryView",
            "AppView::ThemeChooserView",
            "AppView::FileSearchView",
        ];

        for view in required_views {
            assert!(
                source.contains(view),
                "current_view_uses_shared_filter_input must include {}",
                view
            );
        }
    }
}
