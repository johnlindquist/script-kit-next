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
//! - The arm is present in all three triple-embedded dispatchers
//!   (`runtime_stdin_match_core.rs`, `runtime_stdin.rs`,
//!   `app_run_setup.rs`) with identical aliasing (`"design-gallery" |
//!   "designgallery" | "design gallery"`).
//! - Each arm flips `view.current_view` to `AppView::DesignGalleryView`
//!   and ends with `update_window_size_deferred` so the panel resizes to
//!   the design-gallery layout.
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
fn triggerbuiltin_dispatchers_route_design_gallery_to_appview() {
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
    let design_start = prepare
        .find("FilterableView::DesignGallery => FilterableRoutePlan")
        .expect("shared dispatcher must prepare DesignGallery route");
    let design_arm = &prepare[design_start
        ..prepare[design_start..]
            .find("FilterableView::ClipboardHistory")
            .map(|ix| design_start + ix)
            .expect("ClipboardHistory arm follows DesignGallery arm")];
    assert!(
        design_arm.contains("next_view: AppView::DesignGalleryView {"),
        "DesignGallery route plan must target AppView::DesignGalleryView"
    );
    assert!(
        design_arm.contains("filter: String::new(),"),
        "DesignGallery route plan must initialize an empty filter"
    );
    assert!(
        design_arm.contains("selected_index: 0,"),
        "DesignGallery route plan must select the first gallery tile"
    );
    assert!(
        design_arm.contains("resize: true,"),
        "DesignGallery route plan must request deferred resize"
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
fn triggerbuiltin_dispatchers_include_design_gallery_aliases() {
    // The dispatcher intentionally accepts three aliases so operator
    // shorthand and UI bindings can both reach the subview. Dropping any
    // alias would silently break a subset of callers without any other
    // test failing, so this pins the alias set.
    let alias_arm = body_of(TRIGGER_REGISTRY_SOURCE, "pub const fn legacy_aliases(");
    let design_start = alias_arm
        .find("TriggerBuiltin::DesignGallery =>")
        .expect("DesignGallery legacy aliases must be registered");
    let design_line = alias_arm[design_start..]
        .lines()
        .next()
        .expect("DesignGallery alias line");
    assert!(
        design_line.contains("\"design-gallery\""),
        "DesignGallery aliases must include canonical kebab-case `design-gallery`"
    );
    assert!(
        design_line.contains("\"designgallery\""),
        "DesignGallery aliases must include single-word `designgallery`"
    );
    assert!(
        design_line.contains("\"design gallery\""),
        "DesignGallery aliases must include natural-language `design gallery`"
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
