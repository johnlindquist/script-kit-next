//! Source audits for the actions popup structured observability contract.
//!
//! Validates that the actions popup lifecycle (open, close, resize, route)
//! emits structured `tracing` events under the `ACTIONS_POPUP` target and
//! that the mini-mode position resolver branches on `MainWindowMode`.

use super::read_source as read;

#[test]
fn actions_popup_event_enum_defined_in_window() {
    let content = read("src/actions/window.rs");
    assert!(
        content.contains("pub(crate) enum ActionsPopupEvent"),
        "actions/window.rs must define ActionsPopupEvent enum"
    );
    for variant in [
        "OpenRequested",
        "OpenSucceeded",
        "OpenFailed",
        "RoutedKey",
        "Resized",
        "Closed",
    ] {
        assert!(
            content.contains(variant),
            "ActionsPopupEvent must include variant {variant}"
        );
    }
}

#[test]
fn emit_actions_popup_event_uses_structured_tracing_target() {
    let content = read("src/actions/window.rs");
    assert!(
        content.contains("target: \"ACTIONS_POPUP\""),
        "emit_actions_popup_event must emit under the ACTIONS_POPUP tracing target"
    );
}

#[test]
fn open_actions_window_emits_open_succeeded_receipt() {
    let content = read("src/actions/window.rs");
    let open_fn_start = content
        .find("pub fn open_actions_window(")
        .expect("open_actions_window function not found");
    let open_fn = &content[open_fn_start..];

    assert!(
        open_fn.contains("ActionsPopupEvent::OpenSucceeded"),
        "open_actions_window must emit OpenSucceeded receipt"
    );
    assert!(
        open_fn.contains("Some(position)"),
        "OpenSucceeded receipt must include the window position"
    );
}

#[test]
fn close_actions_window_emits_closed_receipt() {
    let content = read("src/actions/window.rs");
    let close_fn_start = content
        .find("pub fn close_actions_window(")
        .expect("close_actions_window function not found");
    let close_fn = &content[close_fn_start..];

    assert!(
        close_fn.contains("ActionsPopupEvent::Closed"),
        "close_actions_window must emit Closed receipt"
    );
}

#[test]
fn resize_actions_window_direct_emits_resized_receipt() {
    let content = read("src/actions/window.rs");
    let resize_fn_start = content
        .find("pub fn resize_actions_window_direct(")
        .expect("resize_actions_window_direct function not found");
    let resize_fn = &content[resize_fn_start..];

    assert!(
        resize_fn.contains("ActionsPopupEvent::Resized"),
        "resize_actions_window_direct must emit Resized receipt"
    );
}

#[test]
fn toggle_actions_emits_open_requested_receipt() {
    let content = read("src/app_impl/actions_toggle.rs");
    let toggle_fn_start = content
        .find("pub(crate) fn toggle_actions(")
        .expect("toggle_actions function not found");
    let toggle_fn = &content[toggle_fn_start..];

    assert!(
        toggle_fn.contains("ActionsPopupEvent::OpenRequested"),
        "toggle_actions must emit OpenRequested receipt"
    );
}

#[test]
fn close_actions_popup_delegates_close_receipt_to_window_layer() {
    let content = read("src/app_impl/actions_dialog.rs");
    let close_fn_start = content
        .find("pub(crate) fn close_actions_popup(")
        .expect("close_actions_popup function not found");
    let close_fn = &content[close_fn_start..];

    assert!(
        close_fn.contains("close_actions_window(cx);"),
        "close_actions_popup should delegate the Closed receipt to close_actions_window()"
    );
    assert!(
        !close_fn.contains("ActionsPopupEvent::Closed"),
        "close_actions_popup must not emit a duplicate Closed receipt"
    );
}

#[test]
fn main_list_actions_window_position_branches_on_window_mode() {
    let content = read("src/app_impl/actions_toggle.rs");
    let pos_fn_start = content
        .find("fn main_list_actions_window_position(")
        .expect("main_list_actions_window_position function not found");
    let pos_fn = &content[pos_fn_start..pos_fn_start + 400];

    assert!(
        pos_fn.contains("MainWindowMode::Mini"),
        "main_list_actions_window_position must branch on Mini mode"
    );
    assert!(
        pos_fn.contains("MainWindowMode::Full"),
        "main_list_actions_window_position must branch on Full mode"
    );
    assert!(
        pos_fn.contains("TopCenter"),
        "Mini mode should resolve to TopCenter position"
    );
    assert!(
        pos_fn.contains("BottomRight"),
        "Full mode should resolve to BottomRight position"
    );
}

#[test]
fn toggle_actions_uses_resolved_position() {
    let content = read("src/app_impl/actions_toggle.rs");
    let toggle_fn_start = content
        .find("pub(crate) fn toggle_actions(")
        .expect("toggle_actions function not found");
    let toggle_fn = &content[toggle_fn_start..];

    assert!(
        toggle_fn.contains("self.main_list_actions_window_position()"),
        "toggle_actions must call main_list_actions_window_position()"
    );
}

#[test]
fn spawn_open_emits_open_failed_on_error() {
    let content = read("src/app_impl/actions_toggle.rs");
    let spawn_fn_start = content
        .find("fn spawn_open_actions_window(")
        .expect("spawn_open_actions_window function not found");
    let spawn_fn = &content[spawn_fn_start..];

    assert!(
        spawn_fn.contains("ActionsPopupEvent::OpenFailed"),
        "spawn_open_actions_window must emit OpenFailed on error"
    );
}

#[test]
fn actions_popup_search_uses_requested_typography_and_synced_cursor_height() {
    let theme = read("src/designs/core/actions_popup_theme.rs");
    let dialog = read("src/actions/dialog.rs");
    let text_input = read("src/components/text_input/render.rs");

    assert!(
        theme.contains("height: 40.0"),
        "actions popup search height must default to 40px"
    );
    assert!(
        theme.contains("font_size: 14.0"),
        "actions popup search font must match compact action-row typography"
    );
    assert!(
        theme.contains("cursor_height: 14.0"),
        "actions popup cursor height must default to the 14pt font height"
    );
    assert!(
        theme.contains("def.search.cursor_height = def.search.font_size;"),
        "actions popup cursor height must stay synced to search font size after overrides"
    );
    assert!(dialog.contains("render_compact_search_text("));
    assert!(
        dialog.contains("font_size: popup_theme.search.font_size")
            && dialog.contains("cursor_height: popup_theme.search.cursor_height"),
        "actions popup search text and cursor must use their shared popup tokens"
    );
    assert!(
        text_input.contains(".line_height(px(config.font_size))"),
        "shared compact search text must keep line height synced to its font size"
    );
}

#[test]
fn actions_popup_search_placeholder_and_typed_text_share_origin() {
    let dialog = read("src/actions/dialog.rs");
    let text_input = read("src/components/text_input/render.rs");

    assert!(
        dialog.contains("let build_search_content = |search_display: SharedString|")
            && dialog.contains("render_compact_search_text("),
        "top and bottom actions search fields must share one content builder"
    );
    assert!(
        text_input.contains("fn render_compact_search_cursor(")
            && text_input.contains(".relative()")
            && text_input.contains(".w(px(0.0))"),
        "shared compact search cursor must be zero-width so it cannot shift placeholder text"
    );
    assert!(
        !text_input.contains(".mr(px(2.))") && !text_input.contains(".ml(px(2.))"),
        "actions search cursor margins must not offset placeholder or typed text"
    );
}

#[test]
fn actions_popup_rows_use_shared_list_item_with_popup_font_override() {
    let theme = read("src/designs/core/actions_popup_theme.rs");
    let dialog = read("src/actions/dialog.rs");
    let list_item = read("src/list_item/mod.rs");

    assert!(
        theme.contains("title_font_size: 14.0"),
        "actions popup row title font must default to 14pt"
    );
    assert!(
        list_item.contains("metrics_override: Option<ListItemMetricsOverride>")
            && list_item
                .contains("pub fn metrics_override(mut self, metrics: ListItemMetricsOverride)"),
        "shared ListItem must expose an explicit metrics override hook"
    );
    assert!(
        dialog.contains("actions_row_metrics.name_font_size =\n                                            popup_theme.row.title_font_size;")
            && dialog.contains(".metrics_override(actions_row_metrics)"),
        "actions popup rows must keep shared ListItem anatomy while applying popup font tokens"
    );
}

#[test]
fn actions_popup_state_exposes_shortcut_parity() {
    let dialog = read("src/actions/dialog.rs");
    let prompt_handler = read("src/prompt_handler/mod.rs");
    let keyboard_cli = read("scripts/devtools/keyboard.ts");

    assert!(
        dialog.contains("action_shortcut_parity_report(&self.actions, &self.filtered_actions)"),
        "ActionsDialog automation state must compute shortcut parity from visible action metadata"
    );
    assert!(
        dialog.contains("\"canonicalShortcut\": canonical_shortcut"),
        "visible action summaries must expose canonical shortcuts for DevTools receipts"
    );
    assert!(
        dialog.contains("\"shortcutParity\"")
            && dialog.contains("\"unroutableDisplayedShortcuts\"")
            && dialog.contains("\"visibleShortcutBindings\""),
        "ActionsDialog automation state must expose shortcut parity fields"
    );
    assert!(
        prompt_handler.contains("\"shortcutParity\": shortcut_parity"),
        "main getState actionsDialog summary must expose shortcut parity for DevTools receipts"
    );
    assert!(
        keyboard_cli.contains("actionsDialogShortcutParity"),
        "keyboard DevTools receipts must surface actions dialog shortcut parity directly"
    );
}

/// Extract the brace-balanced body of the function starting at `signature`.
fn actions_popup_fn_body(source: &str, signature: &str) -> String {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} not found"));
    let rest = &source[start..];
    let open = rest.find('{').expect("function body open brace");
    let mut depth = 0usize;
    for (index, ch) in rest[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return rest[open..open + index + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body for {signature}");
}

/// WHY: the actions popup must read as an attached child of its parent window.
/// `focus: true` makes GPUI call `makeKeyAndOrderFront` on the popup NSPanel,
/// which steals key-window status from the parent and visibly drops the
/// parent's active shadow (regressed once in `a1cbd2cf9`). Keys reach the
/// popup via parent routing while the parent stays key, and locally after a
/// click promotes it (`setBecomesKeyOnlyIfNeeded:`). If this assertion fires
/// on an intentional focus-model rework, re-verify the parent shadow stays
/// active while the popup is open before changing it.
#[test]
fn open_actions_window_does_not_take_key_window_focus() {
    let content = read("src/actions/window.rs");
    let body = actions_popup_fn_body(content.as_str(), "pub fn open_actions_window(");
    assert!(
        !body.contains("focus: true"),
        "open_actions_window must not open the popup with focus: true — \
         that steals key status from the parent window and drops its shadow"
    );
    assert!(
        body.contains("focus: false"),
        "open_actions_window must explicitly open the popup with focus: false"
    );
}
