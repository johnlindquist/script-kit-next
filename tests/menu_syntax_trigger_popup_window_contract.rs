//! Source-level contracts for the detached menu-syntax trigger popup window.

const POPUP_SOURCE: &str = include_str!("../src/app_impl/menu_syntax_trigger_popup_window.rs");
const STATE_SOURCE: &str = include_str!("../src/app_impl/menu_syntax_trigger_popup.rs");

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
        "INLINE_POPUP_MAX_VISIBLE_ROWS\n        .saturating_sub(trigger_popup_footer_count(snapshot))\n        .max(1)",
        ".filter(|(_, row)| !is_trigger_popup_footer_row(row))",
        ".filter(|(_, row)| is_trigger_popup_footer_row(row))",
        ".chain(footer_rows.iter().copied())",
        "let synopsis = (footer_rows.is_empty()).then_some(()).and_then(|_|",
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
