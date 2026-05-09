//! Source-level contract test for the
//! `automation-semantic-surface-reflects-active-appview` user story.
//!
//! The story wants the automation protocol's `semanticSurface` field to
//! re-key as the main window's active subview transitions —
//! `triggerBuiltin file-search` must flip
//! `listAutomationWindows.windows[0].semanticSurface` from `"scriptList"`
//! to `"fileSearch"` WITHOUT closing and re-opening the window. Before
//! this pass the field was hardcoded at `sync_main_automation_window`
//! call sites (show/hide paths only), so subview transitions — which go
//! through `TriggerBuiltin → view.current_view = AppView::X` — left the
//! field stuck at whatever the last show/hide set it to (`"scriptList"`).
//!
//! Three structural invariants unlock the live behavior:
//!
//! 1. A registry-level API `update_automation_semantic_surface(id,
//!    surface)` must exist that mutates ONLY the `semantic_surface`
//!    field in place — callers in the subview-routing hot path don't
//!    know the window's bounds / title / focus, so forcing them through
//!    `upsert_automation_window` (full replacement) would either mint a
//!    stale entry or require every caller to recompute everything.
//!
//! 2. An `AppView → semanticSurface` mapping must exist near the enum
//!    definition. This is the single canonical map — if a new subview
//!    variant is added (e.g., a future `ThemeChooserSurface`), it gets a
//!    wire string in one place.
//!
//! 3. The three stdin `TriggerBuiltin` dispatchers
//!    (`runtime_stdin_match_core.rs`, `runtime_stdin.rs`,
//!    `app_run_setup.rs` — the dual/triple-embedded pattern noted in
//!    memory 6330/6331) must each invoke the re-key after the inner
//!    match. If only one of the three picks up the call, half the
//!    stdin entry points silently regress.
//!
//! A live verification (`triggerBuiltin file-search` → `getState` →
//! `windows[0].semanticSurface == "fileSearch"`) is the intended end-to-
//! end receipt; this contract test pins the structural scaffolding so a
//! future refactor cannot remove the primitives the live verification
//! depends on.

const REGISTRY_SOURCE: &str = include_str!("../src/windows/automation_registry.rs");
const WINDOWS_MOD_SOURCE: &str = include_str!("../src/windows/mod.rs");
const APP_VIEW_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const STDIN_CORE_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const STDIN_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../src/main_entry/app_run_setup.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start.find(end).unwrap_or(after_start.len());
    &after_start[..end_index]
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn registry_exposes_in_place_semantic_surface_update() {
    assert!(
        REGISTRY_SOURCE.contains(
            "pub fn update_automation_semantic_surface(id: &str, surface: Option<String>) -> bool {"
        ),
        "src/windows/automation_registry.rs must expose \
         `update_automation_semantic_surface(id, surface) -> bool` — \
         this is the single in-place mutator subview-routing callers \
         use to re-key `semanticSurface` without having to re-upsert \
         the whole `AutomationWindowInfo`. Forcing callers through \
         `upsert_automation_window` would either mint a stale entry or \
         require recomputing bounds/title/focus they don't know."
    );
    assert!(
        REGISTRY_SOURCE.contains("info.semantic_surface = surface;"),
        "`update_automation_semantic_surface` must actually assign the \
         new surface to `info.semantic_surface` — without the mutation \
         body the call becomes a no-op and live subview transitions \
         silently leak the old surface name to automation consumers"
    );
    assert!(
        REGISTRY_SOURCE.contains("\"automation_window_semantic_surface_changed\""),
        "`update_automation_semantic_surface` must emit the \
         `automation_window_semantic_surface_changed` tracing event on \
         actual change — this is the edge signal a telemetry consumer \
         correlates against to observe subview transitions in the log \
         stream"
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn registry_api_is_re_exported_from_windows_module() {
    assert!(
        WINDOWS_MOD_SOURCE.contains("update_automation_semantic_surface"),
        "src/windows/mod.rs must re-export \
         `update_automation_semantic_surface` — the stdin dispatchers \
         call it via `crate::windows::update_automation_semantic_surface` \
         so the symbol has to be reachable at that path. If the \
         re-export is dropped, the three dispatcher call sites stop \
         compiling."
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn app_view_defines_semantic_surface_mapping() {
    assert!(
        APP_VIEW_SOURCE
            .contains("fn semantic_surface_for_main_view(view: &AppView) -> Option<String> {"),
        "src/main_sections/app_view_state.rs must define \
         `semantic_surface_for_main_view` next to the `AppView` enum — \
         keeping the mapping co-located with the enum definition means \
         adding a new subview variant forces a matching wire name in \
         one file, not a scavenger hunt across three dispatchers"
    );

    let identity_body = source_between(
        APP_VIEW_SOURCE,
        "pub(crate) fn surface_kind(&self) -> SurfaceKind",
        "pub(crate) fn surface_contract(&self) -> LauncherSurfaceContract",
    );
    let registry_body = source_between(
        APP_VIEW_SOURCE,
        "impl SurfaceKind {",
        "/// Dismiss policy for the active top-level launcher view.",
    );
    for (variant, kind, surface) in [
        (
            "AppView::FileSearchView",
            "SurfaceKind::FileSearchFull",
            "fileSearch",
        ),
        (
            "AppView::ClipboardHistoryView",
            "SurfaceKind::ClipboardHistory",
            "clipboardHistory",
        ),
        (
            "AppView::AppLauncherView",
            "SurfaceKind::AppLauncher",
            "appLauncher",
        ),
        (
            "AppView::BrowserTabsView",
            "SurfaceKind::BrowserTabs",
            "browserTabs",
        ),
        (
            "AppView::EmojiPickerView",
            "SurfaceKind::EmojiPicker",
            "emojiPicker",
        ),
    ] {
        assert!(
            identity_body.contains(variant)
                && identity_body.contains(kind)
                && registry_body.contains(kind)
                && registry_body.contains(&format!("\"{surface}\"")),
            "`AppView::surface_kind` and `SurfaceKind::surface_contract` must map \
             `{variant}` through `{kind}` to `{surface}` — these are the five \
             subviews the story's acceptance criteria name explicitly. Dropping \
             any one of them leaves that subview silently reporting stale surface \
             metadata to automation consumers."
        );
    }
    assert!(
        APP_VIEW_SOURCE.contains(".surface_contract()")
            && APP_VIEW_SOURCE.contains(".automation_semantic_surface"),
        "`semantic_surface_for_main_view` must delegate to the exhaustive \
         surface contract registry instead of carrying its own fallback arm"
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn trigger_builtin_dispatchers_call_rekey_after_inner_match() {
    let snippet = "view\n                                    .rekey_main_automation_surface_after_trigger_builtin_dispatch();";
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        assert!(
            source.contains(snippet),
            "{name} must re-key the main window's semantic surface \
             after the TriggerBuiltin dispatch via the named \
             `rekey_main_automation_surface_after_trigger_builtin_dispatch` \
             helper. \
             The three dispatcher files share the same protocol arms \
             (see memory 6330/6331 — dual/triple-embedded stdin \
             pattern); losing the call in any one of them silently \
             drops re-key for that entry point."
        );
    }
}
