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
        .split("ContextPortalKind::ClipboardHistory =>")
        .nth(1)
        .expect("clipboard portal arm should exist")
        .split("ContextPortalKind::DictationHistory =>")
        .next()
        .expect("clipboard portal arm should be bounded");
    assert!(
        !portal_clipboard_arm.contains("open_builtin_filterable_view_with_filter("),
        "portal must not inline ClipboardHistoryView opening; use the shared opener"
    );
}

#[test]
fn context_selector_top_level_portals_are_full_surface_openers() {
    let picker = source("src/ai/context_selector/mod.rs");
    let inject_portals = picker
        .split("fn inject_portal_items(")
        .nth(1)
        .expect("inject_portal_items should exist")
        .split("fn portal_kind_detail_label(")
        .next()
        .expect("inject_portal_items should be bounded");

    assert!(
        inject_portals.contains("kind: ContextSelectorRowKind::Portal(*kind)"),
        "top-level context selector portal rows should open full portal surfaces"
    );
    assert!(
        !inject_portals.contains("ContextSelectorRowKind::PortalPrefix("),
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
        inline_fallback.contains("kind: ContextSelectorRowKind::Portal(inline_query.kind)"),
        "colon inline searches should keep an explicit full-portal fallback"
    );
}

#[test]
fn context_selector_does_not_render_builtin_previews() {
    let selector_mod = source("src/ai/context_selector/mod.rs");
    let selector_types = source("src/ai/context_selector/types.rs");

    for forbidden in [
        "impl AiApp",
        concat!("render_context_", "picker"),
        "InlineDropdown::new",
        concat!("Context", "PickerState"),
    ] {
        assert!(
            !selector_mod.contains(forbidden),
            "context selector domain must stay UI-state-free: {forbidden}"
        );
        assert!(
            !selector_types.contains(forbidden),
            "context selector types must not reintroduce picker state: {forbidden}"
        );
    }

    for forbidden in [
        "render_clipboard_history",
        "render_clipboard_preview_panel",
        "render_file_search",
        "clipboard-preview-content-area",
        "clipboard-preview-information",
        "file-search-preview",
    ] {
        assert!(
            !selector_mod.contains(forbidden),
            "context selector model must not copy built-in preview code: {forbidden}"
        );
        assert!(
            !selector_types.contains(forbidden),
            "context selector renderer must stay generic and preview-free: {forbidden}"
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
        portal.contains("ContextPortalKind::FileSearch =>")
            && portal.contains("open_file_search(portal_query"),
        "attachment portal File Search should call the same file-search opener"
    );
    assert!(
        utility.contains("self.open_file_search_view(query, FileSearchPresentation::Full, cx);"),
        "shared file-search opener should use the full split-preview presentation"
    );
}

#[test]
fn main_menu_spine_file_flow_matches_agent_chat_portal_parity() {
    let catalog = source("src/spine/catalog_context.rs");
    let portal = source("src/app_impl/attachment_portal.rs");
    let updates = source("src/app_impl/filter_input_updates.rs");
    let plan = source("src/spine/prompt_plan.rs");
    let filtering = source("src/app_impl/filtering_cache.rs");

    // 1. A3 decision (2026-06-09): the top-level @ Files row completes
    //    inline to `@file:` colon mode — building the prompt must never
    //    replace the `@` input with the portal surface. The portal stays
    //    reachable only via the explicit fallback row inside colon mode,
    //    so the catalog must not mint portal actions itself.
    assert!(
        !catalog.contains("SpineListAction::OpenFileSearchPortal"),
        "main-menu @ Files row must complete inline to @file:, not open the portal; \
         the portal fallback row lives in filtering_cache colon mode"
    );
    assert!(
        portal.contains("fn open_spine_file_search_attachment_portal")
            && portal.contains("self.open_file_search(query, cx);"),
        "spine portal open path must reuse the shared full file-search opener"
    );

    // 2. Colon-mode inline `@file:` results keep an explicit full-portal
    //    fallback row, mirroring inject_full_portal_fallback in the picker.
    assert!(
        filtering.contains("spine:@:file-full-search"),
        "inline @file: subsearch must keep a full File Search fallback row"
    );

    // 3. Accepted files insert compact `@file:basename` tokens whose full
    //    path travels through the alias registry into the prompt plan.
    assert!(
        updates.contains("fn spine_file_mention_token")
            && updates.contains("register_spine_file_mention_alias"),
        "main-menu file accepts must insert compact tokens with full-path aliases"
    );
    assert!(
        plan.contains("fn build_spine_prompt_plan_with_aliases"),
        "spine prompt plan must resolve compact tokens through the alias registry"
    );

    // 4. Damaged alias tokens delete atomically, like Agent Chat's
    //    remove_inline_mention_at_cursor path.
    assert!(
        updates.contains("fn spine_mention_atomic_delete_fixup"),
        "main filter must remove damaged alias tokens atomically"
    );
}

#[test]
fn file_search_attachment_portal_accepts_with_basename_label() {
    let file_search = source("src/render_builtins/file_search.rs");
    let portal_accept = file_search
        .split("// Portal mode: attach file to Agent Chat chat and return.")
        .nth(1)
        .expect("file search portal accept branch should exist")
        .split("// Standard file search: open with the default app and close,")
        .next()
        .expect("file search portal accept branch should be bounded");

    assert!(
        portal_accept.contains("AiContextPart::FilePath")
            && portal_accept.contains("path: file.path.clone()"),
        "portal accept should attach the selected file path"
    );
    assert!(
        portal_accept.contains(".file_name()")
            && portal_accept.contains("unwrap_or_else(|| file.path.clone())"),
        "portal accept should display only filename.ext while preserving full path fallback"
    );
    assert!(
        portal_accept.contains("this.close_attachment_portal_with_part(part, cx);"),
        "portal accept should return through the shared attachment close path"
    );
}
