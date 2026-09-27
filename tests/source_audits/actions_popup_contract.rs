//! Source audits for the ⌘K actions rows: they keep the shared list-item
//! anatomy and expose shortcut parity to DevTools.

use super::read_source as read;

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
        dialog.contains("metrics.name_font_size = popup.row.title_font_size;")
            && dialog.contains("resolved_actions_dialog_row_metrics(")
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
