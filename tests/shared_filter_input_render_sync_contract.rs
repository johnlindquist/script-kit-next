const FILTER_INPUT_CORE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const RENDER_IMPL: &str = include_str!("../src/main_sections/render_impl.rs");

fn shared_filter_views_from_core() -> Vec<&'static str> {
    let start = FILTER_INPUT_CORE
        .find("pub(crate) fn current_view_uses_shared_filter_input")
        .expect("shared filter classifier should exist");
    let body = &FILTER_INPUT_CORE[start..];
    let end = body
        .find("pub(crate) fn source_filter_mode_blocks_input_history_recall")
        .expect("shared filter classifier should have a following function");
    body[..end]
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("AppView::") {
                Some(trimmed.trim_start_matches("| ").trim_end_matches(','))
            } else {
                None
            }
        })
        .collect()
}

fn render_sync_section() -> &'static str {
    let start = RENDER_IMPL
        .find("// Sync filter input if needed (views that use shared input)")
        .expect("render should sync shared filter input");
    let after = &RENDER_IMPL[start..];
    let end = after
        .find("self.sync_filter_input_if_needed(window, cx);")
        .expect("render sync guard should call sync_filter_input_if_needed");
    &after[..end]
}

#[test]
fn render_sync_guard_covers_shared_filter_input_views() {
    let render_sync = render_sync_section();
    for view in shared_filter_views_from_core() {
        assert!(
            render_sync.contains(view),
            "render sync guard must include {view}; otherwise pending shared filter text can fail to hydrate when entering that surface"
        );
    }
}
