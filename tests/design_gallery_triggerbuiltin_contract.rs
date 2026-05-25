//! Source-level contract test for the `tool-design-gallery-triggerbuiltin`
//! user story (Run 2 Pass #23).
//!
//! Pass #22 closed the `windowSwitcher` dispatcher gap. `designGallery`
//! already had a `triggerBuiltin design-gallery` arm wired in all three
//! stdin dispatchers (confirmed live: Pass #23 receipts p23d1..p23d6 show
//! `scriptList → designGallery → scriptList` with `choiceCount=85` on
//! the intermediate `getState`), but the subview had NO dedicated
//! verification story, so the arm's presence was a latent coincidence —
//! the next mechanical refactor of the dispatcher match blocks could
//! silently drop it without any test firing.
//!
//! This contract pins the structural invariants that keep the arm
//! reachable:
//!
//! - The stdin dispatchers delegate through the shared triggerBuiltin
//!   resolver/dispatcher.
//! - The internal route planner still knows how to open
//!   `AppView::DesignGalleryView` for non-launcher design tooling.
//! - `AppView::DesignGalleryView` maps to the wire string
//!   `"designGallery"` inside `semantic_surface_for_main_view`, so
//!   Pass #19's semantic-surface re-key emits the right tag for this
//!   subview.
//!
//! Unlike `windowSwitcher`, `designGallery` has no list loader — the
//! view populates its own 85-item gallery from bundled assets — so the
//! contract intentionally omits any `list_windows()` / cache-field
//! expectation.

const STDIN_CORE_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const STDIN_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../src/main_entry/app_run_setup.rs");
const TRIGGER_REGISTRY_SOURCE: &str = include_str!("../src/builtins/trigger_registry.rs");
const TRIGGER_DISPATCH_SOURCE: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");

fn body_of<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let open_rel = source[start..]
        .find('{')
        .unwrap_or_else(|| panic!("missing function body open: {signature}"));
    let open = start + open_rel;
    let mut depth = 0usize;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("missing function body close: {signature}");
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn triggerbuiltin_dispatchers_do_not_route_pruned_design_gallery_alias() {
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        assert!(
            source.contains("ref cmd @ ExternalCommand::TriggerBuiltin { .. }")
                && source.contains("view.dispatch_trigger_builtin(cmd, window, ctx)"),
            "{name} must delegate triggerBuiltin payloads through the shared resolver/dispatcher"
        );
    }

    let prepare = body_of(TRIGGER_DISPATCH_SOURCE, "fn prepare_filterable_route(");
    assert!(
        !prepare.contains("FilterableView::DesignGallery => FilterableRoutePlan"),
        "DesignGallery should not remain reachable through triggerBuiltin after pruning"
    );
    let apply = body_of(TRIGGER_DISPATCH_SOURCE, "fn apply_filterable_route_plan(");
    assert!(
        apply.contains("self.current_view = plan.next_view;")
            && apply.contains("self.update_window_size_deferred(window, cx);"),
        "shared filterable route apply step must own current_view assignment and deferred resize"
    );
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn triggerbuiltin_dispatchers_prune_design_gallery_aliases() {
    let alias_arm = body_of(TRIGGER_REGISTRY_SOURCE, "pub const fn legacy_aliases(");
    assert!(
        !alias_arm.contains("TriggerBuiltin::DesignGallery =>"),
        "DesignGallery should no longer register triggerBuiltin aliases"
    );
    assert!(
        !alias_arm.contains("\"design-gallery\""),
        "DesignGallery aliases should not include canonical kebab-case `design-gallery`"
    );
    assert!(
        !alias_arm.contains("\"designgallery\"") && !alias_arm.contains("\"design gallery\""),
        "DesignGallery aliases should not include shorthand or natural-language forms"
    );
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn design_gallery_appview_variant_maps_to_semantic_surface() {
    // Pass #19's re-key path looks up `semantic_surface_for_main_view`
    // on every view transition. If `DesignGalleryView` is ever removed
    // from that map, `triggerBuiltin design-gallery` would still flip
    // the view but leave `semanticSurface` at the prior subview's tag,
    // silently breaking the automation introspection channel for this
    // entry point. This test pins the map entry.
    const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
    assert!(
        APP_VIEW_STATE.contains("AppView::DesignGalleryView { .. }")
            && APP_VIEW_STATE.contains("\"designGallery\""),
        "`src/main_sections/app_view_state.rs` must map \
         `AppView::DesignGalleryView` to the wire string \
         `\"designGallery\"` inside `AppView::surface_contract`. \
         Live verification (Pass #23 receipt p23d3) depends on this \
         tag — dropping the arm would regress the semantic-surface \
         re-key for the design-gallery entry point."
    );
}
