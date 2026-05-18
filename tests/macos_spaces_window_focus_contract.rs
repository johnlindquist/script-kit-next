use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("section start must exist");
    let tail = &source[start_idx..];
    let end_idx = tail
        .find(end)
        .map(|idx| start_idx + idx)
        .unwrap_or(source.len());
    &source[start_idx..end_idx]
}

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn main_panel_collection_behavior_strips_can_join_all_spaces() {
    let source = read("src/platform/app_window_management.rs");
    assert!(
        source.contains("main_panel_collection_behavior"),
        "main panel collection behavior must be centralized"
    );
    let helper = section_between(
        &source,
        "main_panel_collection_behavior",
        "pub fn configure_as_accessory_app",
    );
    let compact_helper = compact(helper);
    assert!(
        compact_helper.contains(&compact(
            "& !NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES"
        )),
        "main panel must strip CanJoinAllSpaces"
    );
    assert!(
        compact_helper.contains("NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE"),
        "main panel must still move to active Space when explicitly summoned"
    );
    let configure = section_between(
        &source,
        "pub fn configure_as_floating_panel()",
        "pub fn ensure_main_panel_configured",
    );
    assert!(
        configure.contains("main_panel_collection_behavior(current)"),
        "configure_as_floating_panel must use the centralized behavior helper"
    );
    assert!(
        !configure.contains("if has_can_join_all_spaces"),
        "main panel must not preserve a preexisting all-spaces bit"
    );
}

#[test]
fn main_panel_hides_on_active_space_change() {
    let visibility = read("src/platform/visibility_focus.rs");
    let app_management = read("src/platform/app_window_management.rs");
    assert!(
        visibility.contains("NSWorkspaceActiveSpaceDidChangeNotification"),
        "main panel must observe active Space changes"
    );
    assert!(
        visibility.contains("hide_main_window_for_active_space_change"),
        "Space-change observer must route through a named hide helper"
    );
    assert!(
        visibility.contains("crate::set_main_window_visible(false)"),
        "Space-change hide must update Rust visible state"
    );
    assert!(
        visibility.contains("orderOut:nil"),
        "Space-change hide must order out the native AppKit window"
    );
    assert!(
        app_management.contains("install_main_window_space_change_hide_observer()"),
        "observer install must be wired into main panel configuration"
    );
}

#[test]
fn panel_invariant_rejects_can_join_all_spaces_for_main() {
    let source = read("src/platform/panel_invariants.rs");
    assert!(
        source.contains("has_move_to_active && !has_can_join"),
        "main panel invariant must require MoveToActiveSpace and reject CanJoinAllSpaces"
    );
    assert!(
        source.contains("all_spaces_is_rejected"),
        "unit tests must explicitly reject all-spaces behavior"
    );
}

#[test]
fn notes_window_is_pinned_to_opening_space() {
    let source = read("src/notes/window/window_ops.rs");
    assert!(
        source.contains("notes_window_collection_behavior"),
        "Notes Space behavior must be centralized"
    );
    let helper = section_between(
        &source,
        "notes_window_collection_behavior",
        "fn ensure_theme_initialized",
    );
    let compact_helper = compact(helper);
    assert!(
        compact_helper.contains(&compact(
            "& !NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES"
        )),
        "Notes must strip CanJoinAllSpaces"
    );
    assert!(
        compact_helper.contains(&compact(
            "& !NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE"
        )),
        "Notes must strip MoveToActiveSpace"
    );
    let configure = section_between(
        &source,
        "fn configure_notes_as_floating_panel()",
        "#[cfg(not(target_os = \"macos\"))]",
    );
    assert!(
        configure.contains("notes_window_collection_behavior(current)"),
        "Notes native config must apply the pinned-Space helper"
    );
}
