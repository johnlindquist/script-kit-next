//! Source-level contract for selection-owned builtin wheel scrolling.
//!
//! Wheel handlers own selection movement and schedule `scroll_to_item`; render-time
//! reanchor owns settled scrollbar/native-scroll sync. Doing both in the wheel
//! handler can read stale scroll state before GPUI applies the deferred scroll.

fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn scrollbar_metrics_prefer_pending_deferred_scroll_for_thumb_position() {
    let source = include_str!("../src/components/scrollbar.rs");
    let compacted = compact(source);
    assert!(
        compacted.contains("letraw_offset=deferred_scroll_offset.unwrap_or(live_scroll_offset);"),
        "preferred_scroll_offset must prefer pending deferred scroll_to_item before live offset"
    );
    assert!(
        !compacted.contains("ifhas_measurement{live_scroll_offset}else{deferred_scroll_offset"),
        "preferred_scroll_offset must not ignore deferred scroll_to_item after measurement"
    );
}

#[test]
fn current_app_commands_render_does_not_reanchor_selection_from_scroll() {
    let source = include_str!("../src/render_builtins/current_app_commands.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source should exist");
    assert!(
        !production_source.contains("builtin_reanchor_selection_from_scroll("),
        "CurrentAppCommandsView must match main-menu scrolling: render does not reanchor selected_index from scrollbar metrics"
    );
}

#[test]
fn uniform_list_wheel_handlers_do_not_immediately_reanchor_deferred_scroll() {
    let sources = [
        (
            "current_app_commands",
            include_str!("../src/render_builtins/current_app_commands.rs"),
        ),
        (
            "browser_tabs",
            include_str!("../src/render_builtins/browser_tabs.rs"),
        ),
        (
            "window_switcher",
            include_str!("../src/render_builtins/window_switcher.rs"),
        ),
        (
            "process_manager",
            include_str!("../src/render_builtins/process_manager.rs"),
        ),
        (
            "clipboard",
            include_str!("../src/render_builtins/clipboard.rs"),
        ),
        (
            "app_launcher",
            include_str!("../src/render_builtins/app_launcher.rs"),
        ),
        (
            "kit_store",
            include_str!("../src/render_builtins/kit_store.rs"),
        ),
    ];

    let forbidden = [
        "scroll_to_item(new_selected,ScrollStrategy::Nearest);ifletSome(reanchored)=Self::builtin_reanchor_selection_from_scroll(",
        "scroll_to_item(new_selected);ifletSome(reanchored)=Self::builtin_reanchor_selection_from_scroll(",
    ];

    for (label, source) in sources {
        let compacted = compact(source);
        for pattern in forbidden {
            assert!(
                !compacted.contains(pattern),
                "{label} wheel handler must not immediately reanchor after scheduling deferred scroll_to_item"
            );
        }
        assert!(
            compacted.contains(
                "scroll_to_item(new_selected,ScrollStrategy::Nearest);this.note_builtin_selection_owned_wheel_scroll(new_selected);"
            ),
            "{label} wheel handler must suppress render-time reanchor after selection-owned wheel scroll"
        );
    }
}
