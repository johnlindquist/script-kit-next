use std::fs;

#[test]
fn root_windows_source_refreshes_in_app_layer_not_grouping() {
    let filtering = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("filtering cache source should be readable");
    let grouping =
        fs::read_to_string("src/scripts/grouping.rs").expect("grouping source should be readable");

    let refresh_method = filtering
        .split("fn maybe_start_root_windows_refresh_for_query(")
        .nth(1)
        .and_then(|section| {
            section
                .split("fn root_passive_frame_for_current_query(")
                .next()
        })
        .expect("root windows async refresh source should be present");
    for needle in [
        "background_executor()",
        ".spawn(async move { crate::window_control::list_windows() })",
        "crate::window_control::list_windows()",
        "begin_root_windows_refresh",
        "install_root_windows",
        "fail_root_windows_refresh",
    ] {
        assert!(
            refresh_method.contains(needle),
            "explicit windows source filters should refresh in the app layer with `{needle}`"
        );
    }

    let grouped_cache = filtering
        .split("fn get_grouped_results_cached(")
        .nth(1)
        .expect("grouped cache source should be present");
    assert!(
        !grouped_cache.contains("list_windows("),
        "hot grouping cache must not synchronously call the AX window provider"
    );

    let append_windows = grouping
        .split("fn append_root_windows_section(")
        .nth(1)
        .and_then(|section| {
            section
                .split("fn merge_root_global_file_results_with_recent(")
                .next()
        })
        .expect("append_root_windows_section source should be present");
    assert!(
        !append_windows.contains("list_windows("),
        "grouping must stay provider-free; the app layer owns window refresh"
    );
}

#[test]
fn root_windows_source_enriches_metadata_without_render_time_icon_decode() {
    let types =
        fs::read_to_string("src/window_control/types.rs").expect("window types should be readable");
    let query =
        fs::read_to_string("src/window_control/query.rs").expect("window query should be readable");
    let render =
        fs::read_to_string("src/designs/core/render.rs").expect("core renderer should be readable");
    let preflight = fs::read_to_string("src/main_window_preflight/types.rs")
        .expect("preflight types should be readable");

    for needle in [
        "bundle_id: Option<String>",
        "app_path: Option<PathBuf>",
        "is_frontmost_app: bool",
        "is_focused: bool",
        "is_main: bool",
        "is_minimized: bool",
        "is_on_current_space: bool",
        "Other Space",
        "descriptor: String",
        "selection_key",
    ] {
        assert!(
            types.contains(needle),
            "WindowInfo should carry richer root window metadata `{needle}`"
        );
    }

    for needle in [
        "bundleIdentifier",
        "bundleURL",
        "frontmostApplication",
        "AXFocusedWindow",
        "AXMainWindow",
        "AXMinimized",
        "CGWindowListCopyWindowInfo",
        "K_CG_WINDOW_LIST_OPTION_ALL",
        "K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS",
        "kCGWindowIsOnscreen",
        "append_core_graphics_windows",
        "WindowInfoInit",
    ] {
        assert!(
            query.contains(needle),
            "native window query should collect `{needle}` metadata"
        );
    }

    for forbidden in ["get_or_extract_icon", "decode_png", "NSImage"] {
        assert!(
            !query.contains(forbidden),
            "native window query must not depend on app icon extraction `{forbidden}`"
        );
    }

    let window_render_branch = render
        .split("SearchResult::Window(wm) =>")
        .nth(1)
        .and_then(|section| section.split("SearchResult::File").next())
        .expect("window render branch should be present");
    assert!(
        window_render_branch.contains("wm.app_icon"),
        "window rows should render the pre-decoded app icon when available"
    );
    for forbidden in ["get_or_extract_icon", "decode_png", "NSImage"] {
        assert!(
            !window_render_branch.contains(forbidden),
            "window row rendering must not decode icons at render time `{forbidden}`"
        );
    }

    for needle in [
        "leading_icon_present",
        "leading_icon_kind",
        "leading_icon_bundle_id",
    ] {
        assert!(
            preflight.contains(needle),
            "preflight receipts should expose `{needle}` for runtime icon proof"
        );
    }
}

#[test]
fn root_windows_source_renders_truthful_status_rows() {
    let grouping =
        fs::read_to_string("src/scripts/grouping.rs").expect("grouping source should be readable");
    let append_windows = grouping
        .split("fn append_root_windows_section(")
        .nth(1)
        .and_then(|section| {
            section
                .split("fn merge_root_global_file_results_with_recent(")
                .next()
        })
        .expect("append_root_windows_section source should be present");

    for needle in [
        "RootWindowsProviderStatus::PermissionRequired",
        "Accessibility permission required to list windows",
        "RootWindowsProviderStatus::ProviderError",
        "Window provider failed:",
        "No windows found",
        "No window matches",
        "SourceChipStatusKind::ProviderUnavailable",
        "RootWindowsProviderStatus::Refreshing",
        "Loading windows...",
        "Refreshing windows...",
        "SourceChipStatusKind::Loading",
    ] {
        assert!(
            append_windows.contains(needle),
            "windows grouping should expose truthful status row for `{needle}`"
        );
    }
}

#[test]
fn grouped_status_rows_are_protocol_visible_and_non_selectable() {
    let collector = fs::read_to_string("src/app_layout/collect_elements.rs")
        .expect("collect elements source should be readable");
    let status_branch = collector
        .split("crate::list_item::GroupedListItem::Status(status)")
        .nth(1)
        .expect("grouped status rows should be collected");

    for needle in [
        "role: Some(\"status\".to_string())",
        "selectable: Some(false)",
        "source: Some(status.source.receipt_label().to_string())",
        "source_name: Some(status.source_name.clone())",
        "status_kind: Some(status.status_kind.as_str().to_string())",
    ] {
        assert!(
            status_branch.contains(needle),
            "status rows should expose `{needle}` through getElements"
        );
    }
}
