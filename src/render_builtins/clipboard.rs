impl ScriptListApp {
    /// Render clipboard history view
    /// P0 FIX: Data comes from self.cached_clipboard_entries, view passes only state
    fn render_clipboard_history(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        include!("clipboard_history_setup.rs");
        include!("clipboard_history_list.rs");
        include!("clipboard_history_layout.rs");
    }
}
