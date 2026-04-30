//! Source-level contract for Oracle-Session `shortcuts-hud-grid-dismiss-logic`.
//!
//! Background: `is_dismissable_view()` was a negative `matches!` list far
//! from the `AppView` enum. Adding a new non-dismissable view meant
//! remembering to edit `shortcuts_hud_grid.rs`, and old semantic-surface
//! fallback mapping silently treated new variants as `scriptList`. The fix is
//! to own behavior on `AppView` via `AppView::surface_contract()` with an
//! exhaustive match (no wildcard) and no `Default` impl on `DismissPolicy`, so
//! rustc rejects any `AppView` addition that forgets a policy.
//!
//! These source-level tests pin the structural guarantees the pure unit
//! tests cannot pin: the absence of a wildcard arm and the absence of a
//! `Default` escape hatch. Without both, the compile-time guarantee
//! silently regresses.

const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
const SHORTCUTS_HUD_GRID: &str = include_str!("../src/app_impl/shortcuts_hud_grid.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start.find(end).unwrap_or(after_start.len());
    &after_start[..end_index]
}

#[test]
fn dismiss_policy_has_no_default_escape_hatches() {
    let policy_decl = source_between(
        APP_VIEW_STATE,
        "pub(crate) struct DismissPolicy",
        "impl DismissPolicy",
    );
    assert!(
        !policy_decl.contains("Default"),
        "DismissPolicy must not derive or implement Default; new AppView \
         variants must declare policy explicitly"
    );

    let default_impl = ["impl ", "Default for ", "DismissPolicy"].concat();
    let default_call = ["DismissPolicy", "::", "default"].concat();
    assert!(
        !APP_VIEW_STATE.contains(&default_impl),
        "DismissPolicy must not implement Default"
    );
    assert!(
        !APP_VIEW_STATE.contains(&default_call),
        "Do not call DismissPolicy::default(); declare the policy explicitly"
    );
}

#[test]
fn app_view_dismiss_policy_has_no_wildcard_arm() {
    let dismiss_policy_body = source_between(
        APP_VIEW_STATE,
        "pub(crate) fn dismiss_policy(&self) -> DismissPolicy",
        "/// Map an [`AppView`] variant to the automation",
    );
    assert!(
        !dismiss_policy_body.contains("_ =>"),
        "AppView::dismiss_policy must not use a wildcard arm; rustc \
         exhaustiveness is the contract"
    );
    assert!(
        !dismiss_policy_body.contains("_ if "),
        "AppView::dismiss_policy must not use guard-wildcard arms"
    );
}

#[test]
fn app_view_surface_contract_has_no_wildcard_arm() {
    let surface_contract_body = source_between(
        APP_VIEW_STATE,
        "pub(crate) fn surface_contract(&self) -> LauncherSurfaceContract",
        "/// Dismiss policy for the active top-level launcher view.",
    );
    assert!(
        !surface_contract_body.contains("_ =>"),
        "AppView::surface_contract must not use a wildcard arm; rustc \
         exhaustiveness is the contract"
    );
    assert!(
        !surface_contract_body.contains("_ if "),
        "AppView::surface_contract must not use guard-wildcard arms"
    );
}

#[test]
fn semantic_surface_delegates_to_surface_contract() {
    let semantic_surface_body = source_between(
        APP_VIEW_STATE,
        "fn semantic_surface_for_main_view(view: &AppView) -> Option<String>",
        "/// Wrapper to hold a script session",
    );
    assert!(
        semantic_surface_body.contains(".surface_contract()")
            && semantic_surface_body.contains(".automation_semantic_surface"),
        "semantic_surface_for_main_view must read the automation tag from \
         AppView::surface_contract"
    );
    assert!(
        !semantic_surface_body.contains("_ =>"),
        "semantic_surface_for_main_view must not keep its old wildcard fallback"
    );
}

#[test]
fn is_dismissable_view_delegates_to_policy() {
    // The HUD/grid file must not carry a per-variant negative match. The
    // policy lives with AppView; this helper is a delegate only.
    assert!(
        SHORTCUTS_HUD_GRID.contains(".dismiss_policy()")
            && SHORTCUTS_HUD_GRID.contains("DismissTrigger::WindowBlur"),
        "is_dismissable_view must delegate to AppView::dismiss_policy() \
         and check DismissTrigger::WindowBlur"
    );
    assert!(
        !SHORTCUTS_HUD_GRID
            .contains("AppView::TermPrompt { .. }\n                | AppView::EditorPrompt"),
        "the legacy negative matches!() list in is_dismissable_view must \
         not reappear; keep policy on AppView::dismiss_policy"
    );
}

#[test]
fn dismiss_policy_surface_exports_are_wired() {
    // Keep the public shape stable: a silent rename that drops the trigger
    // enum, the policy struct, or the inherent method is caught here.
    assert!(
        APP_VIEW_STATE.contains("pub(crate) enum DismissTrigger")
            && APP_VIEW_STATE.contains("pub(crate) enum DismissEffect")
            && APP_VIEW_STATE.contains("pub(crate) struct DismissPolicy")
            && APP_VIEW_STATE.contains("pub(crate) struct LauncherSurfaceContract")
            && APP_VIEW_STATE
                .contains("pub(crate) fn surface_contract(&self) -> LauncherSurfaceContract")
            && APP_VIEW_STATE.contains("pub(crate) fn dismiss_policy(&self) -> DismissPolicy"),
        "app_view_state.rs must expose DismissTrigger, DismissEffect, \
         DismissPolicy, LauncherSurfaceContract, AppView::surface_contract, \
         and AppView::dismiss_policy"
    );
}
