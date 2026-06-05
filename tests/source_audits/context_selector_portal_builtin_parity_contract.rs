fn source(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn clipboard_builtin_and_attachment_portal_share_opener() {
    let builtin = source("src/app_execute/builtin_execution.rs");
    let portal = source("src/app_impl/attachment_portal.rs");

    assert!(
        builtin.contains("fn open_clipboard_history_surface_with_filter(")
            || builtin.contains("pub(crate) fn open_clipboard_history_surface_with_filter("),
        "clipboard history should expose one shared surface opener"
    );
    assert!(
        builtin.contains("open_clipboard_history_surface_with_filter(String::new()"),
        "direct Clipboard History built-in should call the shared opener"
    );
    assert!(
        portal.contains("open_clipboard_history_surface_with_filter(portal_query.clone()"),
        "attachment portal Clipboard History should call the shared opener with the portal query"
    );

    let portal_clipboard_arm = portal
        .split("PortalKind::ClipboardHistory =>")
        .nth(1)
        .expect("clipboard portal arm should exist")
        .split("PortalKind::DictationHistory =>")
        .next()
        .expect("clipboard portal arm should be bounded");
    assert!(
        !portal_clipboard_arm.contains("open_builtin_filterable_view_with_filter("),
        "portal must not inline ClipboardHistoryView opening; use the shared opener"
    );
}

#[test]
fn context_picker_top_level_portals_are_full_surface_openers() {
    let picker = source("src/ai/window/context_picker/mod.rs");
    let inject_portals = picker
        .split("fn inject_portal_items(")
        .nth(1)
        .expect("inject_portal_items should exist")
        .split("fn portal_kind_detail_label(")
        .next()
        .expect("inject_portal_items should be bounded");

    assert!(
        inject_portals.contains("kind: ContextPickerItemKind::Portal(*kind)"),
        "top-level context picker portal rows should open full portal surfaces"
    );
    assert!(
        !inject_portals.contains("ContextPickerItemKind::PortalPrefix("),
        "top-level portal rows should not insert colon prefixes; colon mode owns inline search"
    );

    let inline_fallback = picker
        .split("fn inject_full_portal_fallback(")
        .nth(1)
        .expect("inject_full_portal_fallback should exist")
        .split("fn collect_inline_portal_items(")
        .next()
        .expect("inject_full_portal_fallback should be bounded");
    assert!(
        inline_fallback.contains("kind: ContextPickerItemKind::Portal(inline_query.kind)"),
        "colon inline searches should keep an explicit full-portal fallback"
    );
}

#[test]
fn context_picker_does_not_render_builtin_previews() {
    let picker_mod = source("src/ai/window/context_picker/mod.rs");
    let picker_render = source("src/ai/window/context_picker/render.rs");

    for forbidden in [
        "render_clipboard_history",
        "render_clipboard_preview_panel",
        "render_file_search",
        "clipboard-preview-content-area",
        "clipboard-preview-information",
        "file-search-preview",
    ] {
        assert!(
            !picker_mod.contains(forbidden),
            "context picker model must not copy built-in preview code: {forbidden}"
        );
        assert!(
            !picker_render.contains(forbidden),
            "context picker renderer must stay generic and preview-free: {forbidden}"
        );
    }
}

#[test]
fn file_search_builtin_and_attachment_portal_use_full_surface_path() {
    let builtin = source("src/app_execute/builtin_execution.rs");
    let portal = source("src/app_impl/attachment_portal.rs");
    let utility = source("src/app_execute/utility_views.rs");

    assert!(
        builtin.contains("SurfaceOpenBuiltinAction::FileSearch")
            && builtin.contains("open_file_search(String::new()"),
        "direct File Search built-in should call the shared file-search opener"
    );
    assert!(
        portal.contains("PortalKind::FileSearch =>")
            && portal.contains("open_file_search(portal_query"),
        "attachment portal File Search should call the same file-search opener"
    );
    assert!(
        utility.contains("self.open_file_search_view(query, FileSearchPresentation::Full, cx);"),
        "shared file-search opener should use the full split-preview presentation"
    );
}
