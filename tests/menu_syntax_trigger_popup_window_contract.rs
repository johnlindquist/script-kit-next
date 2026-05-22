//! Source-level contracts for the detached menu-syntax trigger popup window.

const POPUP_SOURCE: &str = include_str!("../src/app_impl/menu_syntax_trigger_popup_window.rs");
const STATE_SOURCE: &str = include_str!("../src/app_impl/menu_syntax_trigger_popup.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature should exist");
    let tail = &source[start..];
    let open = tail.find('{').expect("function should have body");
    let mut depth = 0usize;
    for (offset, ch) in tail[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &tail[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    tail
}

#[test]
fn default_selection_predicate_excludes_footer_actions() {
    let body = function_body(
        STATE_SOURCE,
        "pub(crate) fn trigger_popup_row_is_default_selectable",
    );
    assert!(
        body.contains("row.enabled && row.kind != TriggerPickerRowKind::FooterAction"),
        "default selection must skip footer actions while keeping them clickable"
    );

    let preserve = function_body(STATE_SOURCE, "fn preserve_or_pick_first_enabled");
    assert!(
        preserve
            .matches("trigger_popup_row_is_default_selectable")
            .count()
            >= 2,
        "preserve/fallback selection must use the footer-aware predicate"
    );
}

#[test]
fn selected_index_is_optional_not_row_zero_fallback() {
    let selected = function_body(POPUP_SOURCE, "fn selected_index(&self) -> Option<usize>");
    assert!(
        !selected.contains("unwrap_or(0)"),
        "missing selection must not visually select row zero"
    );

    let render = function_body(POPUP_SOURCE, "fn render_picker(&self");
    assert!(
        render.contains("let is_selected = selected_index == Some(idx);"),
        "row highlight should compare against a real optional selected index"
    );
}

#[test]
fn footer_rows_are_pinned_below_paged_normal_rows() {
    for needle in [
        "fn trigger_popup_normal_row_capacity",
        "visible_row_limit\n        .min(INLINE_POPUP_MAX_VISIBLE_ROWS)",
        ".saturating_sub(trigger_popup_footer_count(snapshot))\n        .max(1)",
        ".filter(|(_, row)| !is_trigger_popup_footer_row(row))",
        ".filter(|(_, row)| is_trigger_popup_footer_row(row))",
        ".chain(footer_rows.iter().copied())",
    ] {
        assert!(
            POPUP_SOURCE.contains(needle),
            "popup should page normal rows, append footer rows once, and avoid a second footer-like synopsis strip: {needle}"
        );
    }
}

#[test]
fn stale_popup_slot_discards_clear_automation_registration() {
    let sync = function_body(
        POPUP_SOURCE,
        "pub(crate) fn sync_menu_syntax_trigger_popup_window",
    );
    let discard_count = sync.matches("*guard = None;").count();
    let automation_clear_count = sync
        .matches(
            "crate::windows::remove_automation_window(MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID);",
        )
        .count();
    assert!(
        automation_clear_count >= discard_count,
        "every sync discard path should clear the attached automation window"
    );
}

#[test]
fn main_input_trigger_popup_uses_above_layout() {
    let sync = function_body(
        POPUP_SOURCE,
        "pub(crate) fn sync_menu_syntax_trigger_popup_window",
    );
    assert!(
        POPUP_SOURCE.contains("pub(crate) fn menu_syntax_trigger_popup_layout_above")
            && POPUP_SOURCE.contains("parent_bounds.origin.x.as_f32()")
            && POPUP_SOURCE.contains("parent_bounds.origin.y.as_f32()")
            && POPUP_SOURCE.contains("display_bounds")
            && POPUP_SOURCE.contains("INLINE_POPUP_EDGE_GUTTER"),
        "main-input trigger popups should sit above the main menu while staying inside display bounds"
    );
    assert!(
        sync.contains(
            "menu_syntax_trigger_popup_layout_above(parent_bounds, display_bounds, &snapshot);"
        ) && sync.contains("snapshot.visible_row_limit = layout.visible_row_limit;")
            && sync.contains("let bounds = layout.bounds;")
            && !sync.contains("inline_popup_bounds("),
        "sync should derive bounds from the display-aware above layout"
    );

    let request_builder = function_body(
        POPUP_SOURCE,
        "pub(crate) fn sync_menu_syntax_trigger_popup_window_for_filter",
    );
    assert!(
        request_builder.contains("visible_row_limit: INLINE_POPUP_MAX_VISIBLE_ROWS")
            && request_builder.contains("let display_bounds = display.as_ref().map(|display| display.visible_bounds());")
            && request_builder.contains("display_bounds,")
            && !POPUP_SOURCE.contains("pub(crate) left: f32")
            && !POPUP_SOURCE.contains("pub(crate) top: f32"),
        "request construction should seed the row cap, pass display bounds, and remove legacy below-input offsets"
    );
}

#[test]
fn mouse_accept_preserves_keep_open_rows_through_parent_window() {
    let popup_struct = function_body(
        POPUP_SOURCE,
        "pub(crate) struct MenuSyntaxTriggerPopupWindow",
    );
    assert!(
        popup_struct.contains("parent_window_handle: AnyWindowHandle"),
        "popup entity must retain the parent main-window handle for click keep-open resync"
    );

    let accept_row = function_body(POPUP_SOURCE, "fn accept_row(&self");
    assert!(
        accept_row.contains("-> bool")
            && accept_row.contains("cx.update_window(self.parent_window_handle")
            && accept_row.contains("Some(parent_window)")
            && accept_row.contains("app.accept_menu_syntax_trigger_popup_row(&row_id, None, cx)")
            && accept_row.contains("keep_open"),
        "mouse accept should dispatch through the parent window and report keep_open"
    );

    let click = function_body(POPUP_SOURCE, "fn handle_row_click");
    assert!(
        click.contains("if self.accept_row(index, cx) {\n                return;\n            }")
            && click.contains("window.remove_window();"),
        "keep-open click rows must not fall through to popup teardown"
    );

    let app_accept = function_body(
        POPUP_SOURCE,
        "pub(crate) fn accept_menu_syntax_trigger_popup_row",
    );
    assert!(
        app_accept.contains("window: Option<&mut Window>")
            && app_accept.contains("TriggerPickerIntentOutcome::ReplaceInput")
            && app_accept.contains("keep_open: true")
            && app_accept
                .contains("self.dispatch_menu_syntax_trigger_popup_outcome(outcome, window, cx);"),
        "app accept should pass the parent window into the existing keep-open dispatcher"
    );
}

#[test]
fn prompt_popup_batch_targets_can_act_on_menu_syntax_drawer() {
    assert!(
        POPUP_SOURCE.contains(
            "pub(crate) fn batch_select_menu_syntax_trigger_popup_row_by_value"
        ) && POPUP_SOURCE.contains(
            "pub(crate) fn batch_select_menu_syntax_trigger_popup_row_by_semantic_id"
        ) && POPUP_SOURCE.contains("self.accept_menu_syntax_trigger_popup_row(&row_id, None, cx);"),
        "menu-syntax drawer should expose entity batch selection helpers that reuse the real accept path"
    );

    let target_resolution =
        function_body(PROMPT_HANDLER_SOURCE, "fn resolve_automation_read_target");
    assert!(
        target_resolution
            .contains("menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open"),
        "PromptPopup target resolution must treat menu-syntax trigger popup as an open popup"
    );

    assert!(
        PROMPT_HANDLER_SOURCE
            .contains("_this.batch_select_menu_syntax_trigger_popup_row_by_value(&value, cx)")
            && PROMPT_HANDLER_SOURCE.contains(
                "_this.batch_select_menu_syntax_trigger_popup_row_by_semantic_id(&semantic_id, cx)"
            ),
        "PromptPopup batch selectByValue/selectBySemanticId should route to menu-syntax drawer rows"
    );
}
