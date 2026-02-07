impl ScriptListApp {
    /// Render file search view with 50/50 split (list + preview)
    pub(crate) fn render_file_search(
        &mut self,
        query: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        include!("file_search_setup_key.rs");
        include!("file_search_list.rs");
        include!("file_search_preview.rs");
        include!("file_search_layout.rs");
    }
}
