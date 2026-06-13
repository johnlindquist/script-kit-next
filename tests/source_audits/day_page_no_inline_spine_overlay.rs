//! Day/Today must not render its own inline Spine/Prompt Builder overlay.
//!
//! `@` context in Day round-trips through the main menu/shared context surface;
//! the Day editor itself is only an editor plus direct Cmd+Enter handoff.

use std::fs;

fn source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn function_body<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source
        .find(name)
        .unwrap_or_else(|| panic!("missing function marker: {name}"));
    let open = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing function body for: {name}"));
    let mut depth = 0usize;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body for: {name}");
}

#[test]
fn day_page_render_does_not_mount_inline_spine_overlay() {
    let view = source("src/main_sections/day_page_view.rs");
    let render = function_body(&view, "fn render(");

    for forbidden in [
        "render_day_page_spine_panel",
        "spine_panel",
        "day-page-spine-list",
    ] {
        assert!(
            !render.contains(forbidden),
            "Day render must not mount inline Spine overlay path: {forbidden}"
        );
    }
}

#[test]
fn day_page_keyboard_does_not_drive_spine_rows() {
    let view = source("src/main_sections/day_page_view.rs");
    let handle_key = function_body(&view, "pub(crate) fn handle_key_parts(");

    for forbidden in [
        "day_page_spine_model",
        "move_day_page_spine_selection",
        "accept_day_page_spine_selection",
        "reset_day_page_spine_navigation",
    ] {
        assert!(
            !handle_key.contains(forbidden),
            "Day key handling must not drive inline Spine rows: {forbidden}"
        );
    }
    assert!(
        handle_key.contains("submit_day_page_spine_prompt_from_current_line"),
        "Day may keep direct Cmd+Enter handoff without rendering a row overlay"
    );
}

#[test]
fn day_page_spine_adapter_exposes_no_overlay_rows() {
    let day_spine = source("src/main_sections/day_page_spine.rs");

    for forbidden in [
        "render_day_page_spine_panel",
        "day_page_spine_model",
        "day_page_spine_input",
        "build_day_page_spine_rows",
        "selected_day_page_spine_row",
        "move_day_page_spine_selection",
        "accept_day_page_spine_selection",
        "reset_day_page_spine_navigation",
        "day-page-spine-list",
        "day_page_spine_row",
    ] {
        assert!(
            !day_spine.contains(forbidden),
            "Day spine adapter must not expose inline overlay machinery: {forbidden}"
        );
    }
}

#[test]
fn shared_spine_list_cannot_build_prompt_builder_tail_overlay() {
    let list = source("src/spine/list.rs");
    let input_projection = source("src/spine/input_projection.rs");
    let notes_spine = source("src/components/notes_editor/spine.rs");

    for forbidden in [
        "Prompt Builder",
        "Ready to send",
        "Press Cmd+Enter to send",
        "SubmitPromptPlan",
        "projection_is_prompt_builder_tail",
        "build_prompt_builder_tail_section",
        "force_spine_tail_projection_after_trailing_space",
        "catalog_history",
        "RecentPrompt",
        "OpenConversation",
    ] {
        assert!(
            !list.contains(forbidden),
            "Shared Spine list must not be able to build deprecated prompt-builder tail overlay: {forbidden}"
        );
        assert!(
            !input_projection.contains(forbidden),
            "Spine projection must not preserve deprecated prompt-builder tail ownership: {forbidden}"
        );
        assert!(
            !notes_spine.contains(forbidden),
            "Notes editor Spine must not preserve deprecated prompt-builder tail ownership: {forbidden}"
        );
    }

    let owns_list = function_body(&input_projection, "fn projection_owns_prompt_builder_list(");
    assert!(
        !owns_list.contains("parse_has_prompt_builder_segments"),
        "Shared Spine ownership must not let free-text tails open a list"
    );
    assert!(
        owns_list.contains("SpineSegmentKind::FreeText"),
        "Shared Spine ownership should still reject FreeText projections explicitly"
    );
}

#[test]
fn day_page_context_round_trip_still_uses_main_menu() {
    let round_trip = source("src/main_sections/day_page_round_trip.rs");

    for required in [
        "reset_to_script_list",
        "set_filter_text_immediate(segment_text, window, cx)",
        "request_script_list_main_filter_focus",
    ] {
        assert!(
            round_trip.contains(required),
            "Day @ context must keep main-menu round trip path: {required}"
        );
    }
}
