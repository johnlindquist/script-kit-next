const ACTIONS_DIALOG: &str = include_str!("../src/actions/dialog.rs");
const LIST_ITEM: &str = include_str!("../src/list_item/mod.rs");

fn slice_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source.find(start).expect("missing start marker");
    let rest = &source[start_ix..];
    let end_ix = rest.find(end).expect("missing end marker");
    &rest[..end_ix]
}

#[test]
fn actions_dialog_action_rows_use_shared_list_item_chrome() {
    let item_branch = slice_between(
        ACTIONS_DIALOG,
        "GroupedActionItem::Item(filter_idx) =>",
        "let action_row = div()",
    );

    for required in [
        "crate::list_item::ListItem::new(",
        "crate::list_item::ListItemColors::from_theme",
        ".selected(is_selected)",
        ".hovered(this.hovered_row == Some(ix))",
        ".main_menu_theme(main_menu_theme)",
        ".semantic_id(format!(\"choice:{ix}:{}\", action.id))",
        ".description_opt(",
        ".shortcut_opt(shortcut)",
        "crate::list_item::RowShortcutVisibilityPolicy::AllRows",
    ] {
        assert!(
            item_branch.contains(required),
            "missing shared action row contract: {required}"
        );
    }

    for forbidden in [
        "selected_row_bg",
        "hover_row_bg",
        "destructive_selected_bg",
        "destructive_hover_bg",
        "border_l(px(ACCENT_BAR_WIDTH))",
        "let inner_row = div()",
        "let content = div()",
    ] {
        assert!(
            !item_branch.contains(forbidden),
            "Actions Dialog kept bespoke row chrome: {forbidden}"
        );
    }
}

#[test]
fn actions_dialog_preserves_existing_row_interactions_around_shared_item() {
    let row_wrapper = slice_between(
        ACTIONS_DIALOG,
        "let action_row = div()",
        "action_row.child(list_item).into_any_element()",
    );

    for required in [
        "this.handle_row_click(ix, event, cx)",
        "this.on_activation.clone()",
        ".on_mouse_move({",
        "this.hovered_row = Some(ix)",
        ".on_hover({",
    ] {
        assert!(
            row_wrapper.contains(required),
            "missing Actions row interaction contract: {required}"
        );
    }
}

#[test]
fn list_item_defaults_shortcuts_to_selected_only_and_supports_policy_override() {
    for required in [
        "shortcut_visibility_policy: RowShortcutVisibilityPolicy::SelectedOnly",
        "pub(crate) fn shortcut_visibility_policy(",
        "should_show_row_shortcut(shortcut_visibility_policy, self.selected, hover_visible)",
    ] {
        assert!(
            LIST_ITEM.contains(required),
            "missing ListItem shortcut policy contract: {required}"
        );
    }

    assert!(
        !LIST_ITEM
            .contains("should_show_search_shortcut(is_filtering, self.selected, hover_visible)"),
        "ListItem render should not hard-code search shortcut visibility"
    );
}
