//! Day/Today must not render its own inline Spine overlay.
//!
//! `@` context in Day round-trips through the main menu/shared context surface;
//! the Day editor itself must not expose a Day-local prompt-builder or Agent
//! handoff surface.

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

    assert!(
        !render.contains("spine"),
        "Day render must stay editor-only; Spine is direct handoff logic, not a mounted Day overlay"
    );
    assert!(
        render.contains("editor_input") && render.contains("day_switcher_panel"),
        "Day render should only compose the shared editor plus the day switcher overlay"
    );
}

#[test]
fn day_page_keyboard_does_not_drive_spine_rows() {
    let view = source("src/main_sections/day_page_view.rs");
    let types = source("src/main_sections/day_page_types.rs");
    let handle_key = function_body(&view, "pub(crate) fn handle_key_parts(");

    for forbidden in [
        "SpineList",
        "selected_index",
        "hovered_index",
        "accept_",
        "move_",
    ] {
        assert!(
            !handle_key.contains(forbidden),
            "Day key handling must not drive inline Spine row/navigation state: {forbidden}"
        );
    }
    assert!(
        !handle_key.contains("submit_day_page_spine_prompt_from_current_line"),
        "Day Cmd+Enter must not open Agent Chat/prompt-builder surfaces"
    );
    let deprecated_submit_anchor = concat!("cwd_", "submit_", "anchor");
    assert!(
        !types.contains(deprecated_submit_anchor),
        "Day spine state must not keep submit anchors for deprecated prompt-builder handoff"
    );
}

#[test]
fn day_page_spine_adapter_exposes_no_overlay_rows() {
    let day_spine = source("src/main_sections/day_page_spine.rs");

    for forbidden in [
        "impl Render",
        "IntoElement",
        "GroupedListItem",
        "SpineListRow",
        "selected_index",
        "hovered_index",
        "MouseButton",
        "on_mouse_down",
    ] {
        assert!(
            !day_spine.contains(forbidden),
            "Day spine helper must not expose inline overlay/list machinery: {forbidden}"
        );
    }
}

#[test]
fn free_text_spine_projection_does_not_own_a_list() {
    let list = source("src/spine/list.rs");
    let input_projection = source("src/spine/input_projection.rs");
    let notes_spine = source("src/components/notes_editor/spine.rs");

    assert!(
        list.contains("SpineSegmentKind::FreeText => Vec::new()"),
        "Shared Spine list must not build rows for free-text tails"
    );

    let owns_list = function_body(&input_projection, "fn projection_owns_prompt_builder_list(");
    assert!(
        owns_list.contains("SpineSegmentKind::FreeText") && owns_list.contains("return false"),
        "Shared Spine ownership must reject free-text projections explicitly"
    );

    let notes_owns_list = function_body(&notes_spine, "fn spine_projection_owns_editor_list(");
    assert!(
        notes_owns_list.contains("SpineSegmentKind::ContextMention")
            && notes_owns_list.contains("return false"),
        "Notes editor Spine keeps context mentions on the shared main-menu path"
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

#[test]
fn day_page_header_context_chips_are_inert() {
    let view = source("src/main_sections/day_page_view.rs");
    let render = function_body(&view, "fn render(");

    assert!(
        render.contains("render_inert_main_view_context_zone"),
        "Day header context labels should display state without clickable CWD/Agent chip exits"
    );
    assert!(
        !render.contains("render_clickable_main_view_context_zone"),
        "Day must not mount clickable header chips that can leave Day for prompt-builder surfaces"
    );
}

#[test]
fn day_page_actions_do_not_offer_agent_handoff() {
    let actions = source("src/main_sections/day_page_actions.rs");
    let agent_handoff = source("src/app_impl/agent_handoff/mod.rs");
    let dialog = source("src/actions/dialog.rs");
    let actions_toggle = source("src/app_impl/actions_toggle.rs");

    for forbidden in [
        "day_page:handoff_line",
        "Send Line to Agent Chat",
        "handoff_current_line_to_agent_chat",
        "day_page_handoff_plain_line",
    ] {
        assert!(
            !actions.contains(forbidden),
            "Day actions must not expose Agent handoff path: {forbidden}"
        );
    }

    for forbidden in [
        concat!("submit_", "day_page_", "spine_prompt_plan_", "with_aliases"),
        concat!("submit_", "day_page_", "markdown_line_", "with_context"),
        concat!("day_page_", "markdown_reference_", "handoff"),
        concat!("day_page_", "line_", "handoff"),
    ] {
        assert!(
            !agent_handoff.contains(forbidden),
            "Agent handoff must not retain deprecated Day prompt-builder handoff path: {forbidden}"
        );
    }

    assert!(
        dialog.contains("suppress_prompt_handoff_actions")
            && dialog.contains("!action.id.starts_with(\"prompt-action/\")")
            && dialog.contains("!action.id.starts_with(\"prompt-target/\")"),
        "ActionsDialog must be able to suppress generic prompt export/target handoff rows"
    );
    assert!(
        actions_toggle.contains("if on_day_page")
            && actions_toggle.contains("dialog.set_suppress_prompt_handoff_actions(true)"),
        "Day-hosted Actions dialog must suppress generic prompt handoff rows"
    );
}

#[test]
fn day_page_footer_cannot_open_generic_agent_chat_popup() {
    let view = source("src/main_sections/day_page_view.rs");
    let footer = function_body(&view, "pub(crate) fn day_page_footer_buttons(");

    assert!(
        footer.contains("FooterAction::Run") && footer.contains("FooterAction::Actions"),
        "Day footer should keep Save/Actions affordances"
    );
    assert!(
        !footer.contains("FooterAction::Ai"),
        "Day footer must not expose the generic Agent footer button; stale clicks opened the deleted inline assistant panel"
    );

    let ui_window = source("src/app_impl/ui_window.rs");
    let dispatcher = function_body(&ui_window, "fn dispatch_main_window_footer_action(");
    let ai_arm = dispatcher
        .split("crate::footer_popup::FooterAction::Ai =>")
        .nth(1)
        .and_then(|tail| {
            tail.split("crate::footer_popup::FooterAction::Stop =>")
                .next()
        })
        .expect("FooterAction::Ai arm should exist before Stop arm");
    assert!(
        ai_arm.contains("AppView::DayPage { .. }")
            && ai_arm.contains("main_window_footer_ai_ignored_day_page"),
        "stale Day footer AI events must be ignored before the generic Agent Chat open path"
    );
}
