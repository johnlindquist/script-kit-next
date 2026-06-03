//! Source-level contracts for main-list ownership of menu-syntax trigger rows.

const TRIGGER_OWNER: &str = include_str!("../src/app_impl/menu_syntax_trigger_popup_window.rs");
const FILTER_INPUT_CHANGE: &str = include_str!("../src/app_impl/filter_input_change.rs");
const RENDER_SCRIPT_LIST: &str = include_str!("../src/render_script_list/mod.rs");
const SIMULATE_KEY: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

#[test]
fn trigger_owner_no_longer_defines_detached_prompt_popup_window() {
    for stale in [
        "pub(crate) struct MenuSyntaxTriggerPopupWindow",
        "pub(crate) fn sync_menu_syntax_trigger_popup_window",
        "sync_menu_syntax_trigger_popup_window_for_filter",
        "MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID",
        "AutomationWindowKind::PromptPopup",
        "register_attached_popup",
        "menuSyntaxTriggerPopup",
        "menu-syntax-trigger-popup",
        "open_window",
        "close_menu_syntax_trigger_popup_window",
        "is_menu_syntax_trigger_popup_window_open",
        "batch_select_menu_syntax_trigger_popup_row_by_value",
        "batch_select_menu_syntax_trigger_popup_row_by_semantic_id",
    ] {
        assert!(
            !TRIGGER_OWNER.contains(stale),
            "trigger picker must not keep detached popup window code: {stale}"
        );
    }
}

#[test]
fn filter_changes_update_trigger_state_without_window_sync() {
    assert!(
        FILTER_INPUT_CHANGE.contains("let mut trigger_state_changed = false;")
            && FILTER_INPUT_CHANGE.contains("trigger_state_changed = true;")
            && FILTER_INPUT_CHANGE.contains("} else if trigger_state_changed {")
            && FILTER_INPUT_CHANGE.contains("self.invalidate_grouped_cache();"),
        "filter changes should update trigger state and invalidate the main-list cache"
    );
    assert!(
        !FILTER_INPUT_CHANGE.contains("sync_menu_syntax_trigger_popup_window_for_filter"),
        "filter changes must not sync trigger rows into a detached popup"
    );
}

#[test]
fn script_list_keyboard_routes_trigger_picker_by_main_state() {
    assert!(
        TRIGGER_OWNER.contains("fn menu_syntax_trigger_picker_owns_main_keyboard")
            && TRIGGER_OWNER.contains("self.menu_syntax_trigger_popup_state.owns_main_list()"),
        "trigger picker keyboard ownership should be state-based"
    );
    for source in [RENDER_SCRIPT_LIST, SIMULATE_KEY] {
        assert!(
            source.contains("menu_syntax_trigger_picker_owns_main_keyboard()")
                && source.contains("apply_menu_syntax_trigger_popup_intent"),
            "ScriptList key path should route trigger picker intents by main-list state"
        );
    }
}

#[test]
fn prompt_popup_resolution_excludes_menu_syntax_trigger_picker() {
    assert!(
        !PROMPT_HANDLER
            .contains("menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open"),
        "PromptPopup target resolution must not include menu-syntax trigger picker"
    );
    assert!(
        !PROMPT_HANDLER.contains("batch_select_menu_syntax_trigger_popup_row_by_value")
            && !PROMPT_HANDLER
                .contains("batch_select_menu_syntax_trigger_popup_row_by_semantic_id"),
        "PromptPopup batch routing must not select menu-syntax trigger rows"
    );
}

#[test]
fn script_list_elements_expose_trigger_picker_rows() {
    for needle in [
        "list:menu-syntax-trigger-picker",
        "menuSyntaxTriggerPicker",
        "menu-syntax-trigger-row",
        "protocol::generate_semantic_id(\"choice\", index, &row.id)",
        "self.menu_syntax_trigger_popup_state.selected_row_id",
    ] {
        assert!(
            COLLECT_ELEMENTS.contains(needle),
            "ScriptList getElements must expose trigger picker rows: {needle}"
        );
    }
}

#[test]
fn active_footer_does_not_treat_trigger_picker_as_popup() {
    let footer_block = PROMPT_HANDLER
        .split("pub(crate) fn active_footer_snapshot")
        .nth(1)
        .and_then(|tail| tail.split("let mut config").next())
        .expect("active footer block");

    assert!(
        !footer_block.contains("menu_syntax_trigger_popup_state")
            && !footer_block.contains("is_menu_syntax_trigger_popup_window_open"),
        "main-owned trigger picker must not make active footer owner popup"
    );
    assert!(
        footer_block.contains("menu_syntax_object_selector_state"),
        "object selector popup ownership remains out of scope for this slice"
    );
}
