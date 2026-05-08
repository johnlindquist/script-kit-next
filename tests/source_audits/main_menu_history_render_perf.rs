use super::read_source as read;

fn section_between<'a>(content: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = content
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let tail = &content[start_ix..];
    let end_ix = tail
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &tail[..end_ix]
}

#[test]
fn filter_replacement_sync_does_not_measure_all_rows() {
    let content = read("src/app_navigation/impl_scroll.rs");
    let body = section_between(
        &content,
        "pub fn sync_list_state_for_filter_replacement",
        "pub fn validate_selection_bounds",
    );

    assert!(
        !body.contains(".measure_all()"),
        "history filter replacement must not measure every row"
    );
    assert!(
        body.contains("main_list_row_generation"),
        "history filter replacement must bump row generation for fresh row identity"
    );
    assert!(
        body.contains("effective_average_item_height_for_scroll"),
        "history filter replacement should use the launcher row-height estimate"
    );
    assert!(
        !body.contains("scroll_to_reveal_item(self.selected_index)"),
        "history filter replacement should leave final reveal to filter reconciliation"
    );
}

#[test]
fn script_list_rows_include_generation_in_element_ids() {
    let content = read("src/render_script_list/mod.rs");
    let list_body = section_between(
        &content,
        "list(self.main_list_state.clone()",
        ".with_sizing_behavior",
    );

    assert!(
        content.contains("row_generation = self.main_list_row_generation"),
        "ScriptList render must capture row generation before building rows"
    );
    assert!(
        list_body.contains("script-item-gen-{row_generation}")
            && list_body.contains("section-header-gen-{row_generation}"),
        "row element ids must include generation so same-count history recalls repaint visible rows"
    );
}

#[test]
fn ignored_history_render_benchmark_is_registered() {
    let perf_mod = read("src/perf/mod.rs");
    let bench = read("src/perf/main_menu_history_render_bench.rs");

    assert!(
        perf_mod.contains("mod main_menu_history_render_bench"),
        "perf module must register the main-menu history render benchmark"
    );
    assert!(
        bench.contains("main_menu_history_render_prep_benchmark")
            && bench.contains("#[ignore = \"performance benchmark"),
        "benchmark must be available as an ignored CLI test"
    );
    assert!(
        bench.contains("list_state_measure_all_count")
            && bench.contains("report.total_p95_ms <= 8.0"),
        "benchmark must assert timing and measure_all regression counters"
    );
}
