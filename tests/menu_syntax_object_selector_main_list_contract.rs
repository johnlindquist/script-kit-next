//! Source-level contracts for main-list ownership of menu-syntax object rows.

const OBJECT_SELECTOR_OWNER: &str =
    include_str!("../src/app_impl/menu_syntax_object_selector_popup_window.rs");
const FILTER_INPUT_CHANGE: &str = include_str!("../src/app_impl/filter_input_change.rs");
const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");
const RENDER_SCRIPT_LIST: &str = include_str!("../src/render_script_list/mod.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_TAB: &str = include_str!("../src/app_impl/startup_new_tab.rs");
const SIMULATE_KEY: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const FILTERING_CACHE: &str = include_str!("../src/app_impl/filtering_cache.rs");
const AUTOMATION_COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");

#[test]
fn object_selector_owner_no_longer_defines_detached_prompt_popup_window() {
    for stale in [
        "pub(crate) struct MenuSyntaxObjectSelectorPopupWindow",
        "pub(crate) fn sync_menu_syntax_object_selector_popup_window",
        "MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID",
        "AutomationWindowKind::PromptPopup",
        "register_attached_popup",
        "menuSyntaxObjectSelectorPopup",
        "menu-syntax-object-selector-popup",
        "open_window",
        "is_menu_syntax_object_selector_popup_window_open",
        "upsert_menu_syntax_object_selector_popup_snapshot",
    ] {
        assert!(
            !OBJECT_SELECTOR_OWNER.contains(stale),
            "object selector must not keep detached popup window code: {stale}"
        );
    }
}

#[test]
fn object_selector_filter_changes_update_state_without_window_sync() {
    assert!(
        OBJECT_SELECTOR_OWNER
            .contains("pub(crate) fn run_menu_syntax_object_selector_state_machine")
            && OBJECT_SELECTOR_OWNER.contains("self.menu_syntax_object_selector_state =")
            && OBJECT_SELECTOR_OWNER
                .contains("self.menu_syntax_trigger_popup_state = Default::default();")
            && OBJECT_SELECTOR_OWNER.contains("self.invalidate_grouped_cache();")
            && OBJECT_SELECTOR_OWNER.contains("self.reconcile_script_list_after_filter_change("),
        "object selector state machine should own state and rebuild the main list"
    );
    for source in [
        OBJECT_SELECTOR_OWNER,
        FILTER_INPUT_CHANGE,
        FILTER_INPUT_UPDATES,
    ] {
        assert!(
            !source.contains("sync_menu_syntax_object_selector_popup_window")
                && !source.contains("close_menu_syntax_object_selector_popup_window"),
            "object selector filter path must not sync or close a detached popup"
        );
    }
}

#[test]
fn script_list_keyboard_routes_object_selector_by_main_state() {
    assert!(
        OBJECT_SELECTOR_OWNER.contains("fn menu_syntax_object_selector_owns_main_keyboard")
            && OBJECT_SELECTOR_OWNER
                .contains("self.menu_syntax_object_selector_state.owns_main_list()"),
        "object selector keyboard ownership should be state-based"
    );
    for source in [STARTUP, STARTUP_NEW_TAB, SIMULATE_KEY, RENDER_SCRIPT_LIST] {
        assert!(
            source.contains("menu_syntax_object_selector_owns_main_keyboard()")
                && source.contains("apply_menu_syntax_object_selector_intent"),
            "ScriptList key path should route object selector intents by main-list state"
        );
        assert!(
            !source.contains("is_menu_syntax_object_selector_popup_window_open()"),
            "ScriptList key path must not depend on detached object selector popup state"
        );
    }
}

#[test]
fn script_list_elements_expose_object_selector_rows() {
    for needle in [
        "list:menu-syntax-object-selector",
        "menuSyntaxObjectSelector",
        "menu-syntax-object-selector-row",
        "protocol::generate_semantic_id(\"choice\", index, &row.id)",
        "self.menu_syntax_object_selector_state",
        "selected_row_id",
    ] {
        assert!(
            COLLECT_ELEMENTS.contains(needle),
            "ScriptList getElements must expose object selector rows: {needle}"
        );
    }
}

#[test]
fn prompt_popup_resolution_excludes_object_selector() {
    for stale in [
        "menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open",
        "batch_select_menu_syntax_object_selector_popup_row_by_value",
        "batch_select_menu_syntax_object_selector_popup_row_by_semantic_id",
        "menuSyntaxObjectSelectorPopup",
    ] {
        assert!(
            !PROMPT_HANDLER.contains(stale),
            "PromptPopup resolution/batch routing must not include object selector: {stale}"
        );
    }
}

#[test]
fn active_footer_does_not_treat_object_selector_as_popup() {
    let footer_block = PROMPT_HANDLER
        .split("pub(crate) fn active_footer_snapshot")
        .nth(1)
        .and_then(|tail| tail.split("let mut config").next())
        .expect("active footer block");

    assert!(
        !footer_block.contains("menu_syntax_object_selector_state")
            && !footer_block.contains("is_menu_syntax_object_selector_popup_window_open"),
        "main-owned object selector must not make active footer owner popup"
    );
}

#[test]
fn get_state_reports_object_selector_as_main_choices() {
    for needle in [
        "self.menu_syntax_object_selector_state",
        "self.menu_syntax_object_selector_state.owns_main_list()",
        "selected_row_id",
        "snapshot.rows.len()",
        "row.token.clone().unwrap_or_else(|| row.id.clone())",
    ] {
        assert!(
            PROMPT_HANDLER.contains(needle),
            "ScriptList getState/currentSelectedValue must report object selector choices: {needle}"
        );
    }
}

#[test]
fn render_and_cache_paths_back_main_owned_object_selector() {
    assert!(
        RENDER_SCRIPT_LIST.contains("render_menu_syntax_object_selector_main_list")
            && RENDER_SCRIPT_LIST.contains("menu_syntax_object_selector_state.owns_main_list()")
            && RENDER_SCRIPT_LIST.contains("accept_menu_syntax_object_selector_row"),
        "ScriptList render path should fill the main list with object selector rows"
    );
    assert!(
        FILTERING_CACHE.contains("self.menu_syntax_object_selector_state.owns_main_list()"),
        "filtering cache should keep normal launcher rows suppressed while object selector owns the main list"
    );
}

#[test]
fn object_selector_popup_automation_collector_is_removed() {
    for stale in [
        "upsert_menu_syntax_object_selector_popup_snapshot",
        "remove_menu_syntax_object_selector_popup_snapshot",
        "panel:menu-syntax-object-selector-popup",
        "list:menu-syntax-object-selector-popup",
    ] {
        assert!(
            !AUTOMATION_COLLECTOR.contains(stale),
            "object selector should not publish detached PromptPopup elements: {stale}"
        );
    }
}
