/// Render and filter-performance diagnostics owned by the main script list surface.
#[derive(Debug)]
pub struct MainMenuRenderDiagnosticsState {
    /// Last filter value that produced render diagnostics.
    pub last_render_log_filter: String,
    /// Last selection index that produced render diagnostics.
    pub last_render_log_selection: usize,
    /// Last item count that produced render diagnostics.
    pub last_render_log_item_count: usize,
    /// True when the current render changed enough to log preview diagnostics.
    pub log_this_render: bool,
    /// Start time for the current input-to-grouped-results performance sample.
    pub filter_perf_start: Option<std::time::Instant>,
    /// Cache fields for highlighting
    pub last_input_highlight_text: String,
    pub last_input_highlight_ranges: Vec<(std::ops::Range<usize>, gpui::Hsla, String)>,
}

impl Default for MainMenuRenderDiagnosticsState {
    fn default() -> Self {
        Self {
            last_render_log_filter: String::new(),
            last_render_log_selection: usize::MAX,
            last_render_log_item_count: usize::MAX,
            log_this_render: true,
            filter_perf_start: None,
            last_input_highlight_text: String::new(),
            last_input_highlight_ranges: Vec::new(),
        }
    }
}

#[cfg(test)]
mod main_menu_render_diagnostics_tests {
    use super::MainMenuRenderDiagnosticsState;

    #[test]
    fn a_fresh_launcher_logs_its_first_render_and_times_no_filter() {
        let diagnostics = MainMenuRenderDiagnosticsState::default();

        assert!(diagnostics.log_this_render);
        assert!(diagnostics.filter_perf_start.is_none());
        assert!(diagnostics.last_render_log_filter.is_empty());
        assert!(diagnostics.last_input_highlight_text.is_empty());
        assert!(diagnostics.last_input_highlight_ranges.is_empty());
    }

    #[test]
    fn no_real_render_matches_the_fresh_launcher_log_key() {
        let diagnostics = MainMenuRenderDiagnosticsState::default();

        assert_eq!(diagnostics.last_render_log_selection, usize::MAX);
        assert_eq!(diagnostics.last_render_log_item_count, usize::MAX);
    }
}
