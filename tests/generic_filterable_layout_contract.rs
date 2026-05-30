const LAYOUT_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");
const ELEMENTS_SOURCE: &str = include_str!("../src/app_layout/collect_elements.rs");
const TRIGGER_REGISTRY_SOURCE: &str = include_str!("../src/builtins/trigger_registry.rs");
const ROUTES_SOURCE: &str = include_str!("../src/app_impl/routes.rs");
const TRIGGER_DISPATCH_SOURCE: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");

#[test]
fn generic_filterable_layout_uses_dedicated_full_width_nodes_before_generic_split_shell() {
    let branch = LAYOUT_SOURCE
        .find("AppView::FavoritesBrowseView")
        .expect("build_layout_info must mention FavoritesBrowseView");
    let generic_script_list = LAYOUT_SOURCE
        .find("LayoutComponentInfo::new(\"ScriptList\"")
        .expect("build_layout_info must retain the generic ScriptList branch");

    assert!(
        branch < generic_script_list,
        "GenericFilterableList variants must be measured before the generic ScriptList/PreviewPanel branch"
    );

    for needle in [
        "GenericFilterableSurface",
        "GenericFilterableHeader",
        "GenericFilterableSearch",
        "GenericFilterableCount",
        "GenericFilterableDivider",
        "GenericFilterableList",
        "GenericFilterableRow",
        "GenericFilterableEmptyState",
        "GenericFilterableFooter",
        "full-width list surface",
        "no preview panel",
    ] {
        assert!(
            LAYOUT_SOURCE.contains(needle),
            "GenericFilterable layout receipt is missing `{needle}`"
        );
    }
}

#[test]
fn generic_filterable_elements_do_not_fall_back_to_current_view_panel() {
    for needle in [
        "AppView::SearchAiPresetsView",
        "AppView::FavoritesBrowseView",
        "collect_generic_filterable_rows",
        "\"ai-presets-filter\"",
        "\"favorites-filter\"",
        "\"ai-presets\"",
        "\"favorites\"",
        "\"ai-presets-empty\"",
        "\"favorites-empty\"",
    ] {
        assert!(
            ELEMENTS_SOURCE.contains(needle),
            "GenericFilterable element collection is missing `{needle}`"
        );
    }
}

#[test]
fn generic_filterable_variants_have_deterministic_trigger_builtin_routes() {
    for needle in [
        "TriggerBuiltin::Favorites",
        "TriggerBuiltin::SearchAiPresets",
        "\"builtin/favorites\"",
        "\"builtin/search-ai-presets\"",
    ] {
        assert!(
            TRIGGER_REGISTRY_SOURCE.contains(needle),
            "triggerBuiltin registry is missing `{needle}`"
        );
    }

    for needle in [
        "FilterableView::Favorites",
        "FilterableView::SearchAiPresets",
    ] {
        assert!(
            ROUTES_SOURCE.contains(needle),
            "triggerBuiltin route planner is missing `{needle}`"
        );
    }

    for needle in [
        "AppView::FavoritesBrowseView",
        "AppView::SearchAiPresetsView",
    ] {
        assert!(
            TRIGGER_DISPATCH_SOURCE.contains(needle),
            "triggerBuiltin dispatcher is missing `{needle}`"
        );
    }
}
