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

// @lat: [[lat.md/automation#Automation#Window metadata]]
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
        let Some(arm_start) =
            source.find("\"design-gallery\" | \"designgallery\" | \"design gallery\" =>")
        else {
            panic!(
                "{name} is missing the `design-gallery` triggerBuiltin arm. \
                 All three stdin dispatchers must route \
                 `triggerBuiltin design-gallery` (and its `designgallery` / \
                 `design gallery` aliases) to `AppView::DesignGalleryView`; \
                 otherwise automation cannot reach the designGallery subview \
                 and Pass #19's semantic-surface re-key emits stale tags."
            );
        };
        // Scope: from this arm's start through the next 20 lines. The
        // design-gallery arm is 5–6 lines (no loader), so 20 comfortably
        // covers the body without reaching into unrelated arms regardless
        // of which builtin comes next in each dispatcher.
        let arm_body: String = source[arm_start..]
            .lines()
            .take(20)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            arm_body.contains("view.current_view = AppView::DesignGalleryView {"),
            "{name} `design-gallery` arm must set \
             `view.current_view = AppView::DesignGalleryView {{ ... }}`. \
             Any other view variant breaks the subview routing + \
             semantic-surface re-key for this entry point."
        );
        assert!(
            arm_body.contains("filter: String::new(),"),
            "{name} `design-gallery` arm must initialize the view with \
             `filter: String::new()` so the gallery opens at a clean \
             filter — mirrors every other subview's arm and keeps the \
             renderer's filter-field expectations satisfied."
        );
        assert!(
            arm_body.contains("selected_index: 0,"),
            "{name} `design-gallery` arm must initialize the view with \
             `selected_index: 0` so the first gallery tile is selected — \
             dropping the field makes `getState.selectedIndex` unstable \
             for automation."
        );
        assert!(
            arm_body.contains("view.update_window_size_deferred(window, ctx);"),
            "{name} `design-gallery` arm must end with \
             `view.update_window_size_deferred(window, ctx);` — skipping it \
             leaves the panel at the scriptList height (440px) instead of \
             the design-gallery height (500px), so the rendered grid clips."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn triggerbuiltin_dispatchers_include_design_gallery_aliases() {
    // The dispatcher intentionally accepts three aliases so operator
    // shorthand and UI bindings can both reach the subview. Dropping any
    // alias would silently break a subset of callers without any other
    // test failing, so this pins the alias set.
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let arm_line = source
            .lines()
            .find(|line| line.contains("\"design-gallery\"") && line.contains("=>"))
            .unwrap_or_else(|| panic!("{name} missing design-gallery arm line"));
        assert!(
            arm_line.contains("\"design-gallery\""),
            "{name} design-gallery arm must accept the canonical \
             kebab-case alias `\"design-gallery\"` — this is the form \
             documented in the triggerBuiltin protocol and used by the \
             main-menu entry."
        );
        assert!(
            arm_line.contains("\"designgallery\""),
            "{name} design-gallery arm must accept the single-word \
             alias `\"designgallery\"` — convenience form for operator \
             shorthand."
        );
        assert!(
            arm_line.contains("\"design gallery\""),
            "{name} design-gallery arm must accept the space-separated \
             alias `\"design gallery\"` — natural-language form useful \
             when the name flows through user-visible input."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
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
