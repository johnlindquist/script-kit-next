#[test]
fn main_window_preflight_module_exports_receipt_types() {
    let source = std::fs::read_to_string("src/main_window_preflight/types.rs")
        .expect("should read types.rs");
    assert!(
        source.contains("MainWindowPreflightReceipt"),
        "types.rs should define MainWindowPreflightReceipt"
    );
    assert!(
        source.contains("MainWindowPreflightAction"),
        "types.rs should define MainWindowPreflightAction"
    );
    assert!(
        source.contains("serde(rename_all = \"camelCase\")"),
        "types should use camelCase serde rename"
    );
}

#[test]
fn main_window_preflight_logs_structured_receipt_event() {
    let source = std::fs::read_to_string("src/main_window_preflight/build.rs")
        .expect("should read build.rs");
    assert!(
        source.contains("event = \"main_window_preflight_receipt\""),
        "build.rs should log structured receipt events"
    );
}

#[test]
fn main_window_preflight_render_uses_theme_colors() {
    let source = std::fs::read_to_string("src/main_window_preflight/render.rs")
        .expect("should read render.rs");
    assert!(
        source.contains("AppChromeColors::from_theme"),
        "render.rs should use AppChromeColors from theme"
    );
    assert!(
        !source.contains("rgb(0x"),
        "render.rs should not hardcode hex colors"
    );
}

#[test]
fn script_list_renders_preflight_and_preview_in_same_right_pane() {
    let source = std::fs::read_to_string("src/render_script_list/mod.rs")
        .expect("should read render_script_list");
    assert!(
        source.contains("render_main_window_preflight_receipt"),
        "right pane should render preflight receipt"
    );
    assert!(
        source.contains("render_preview_panel"),
        "right pane should still render preview panel"
    );
    assert!(
        source.contains(".w_1_2()"),
        "right pane should use 50% width"
    );
}

#[test]
fn app_state_has_cached_preflight_fields() {
    let source = std::fs::read_to_string("src/main_sections/app_state.rs")
        .expect("should read app_state.rs");
    assert!(
        source.contains("cached_main_window_preflight"),
        "ScriptListApp should have cached_main_window_preflight field"
    );
    assert!(
        source.contains("main_window_preflight_cache_key"),
        "ScriptListApp should have main_window_preflight_cache_key field"
    );
}

#[test]
fn invalidation_helper_exists() {
    let source = std::fs::read_to_string("src/app_impl/ui_window.rs")
        .expect("should read ui_window.rs");
    assert!(
        source.contains("fn invalidate_main_window_preflight"),
        "ui_window.rs should have invalidation helper"
    );
    assert!(
        source.contains("fn rebuild_main_window_preflight_if_needed"),
        "ui_window.rs should have rebuild helper"
    );
}

#[test]
fn selection_change_triggers_preflight_rebuild() {
    let source = std::fs::read_to_string("src/app_navigation/impl_movement.rs")
        .expect("should read impl_movement.rs");
    assert!(
        source.contains("rebuild_main_window_preflight_if_needed"),
        "set_selected_index should rebuild preflight receipt"
    );
}

#[test]
fn filter_change_triggers_preflight_rebuild() {
    let source = std::fs::read_to_string("src/app_impl/filter_input_updates.rs")
        .expect("should read filter_input_updates.rs");
    let count = source.matches("rebuild_main_window_preflight_if_needed").count();
    assert!(
        count >= 2,
        "filter_input_updates.rs should rebuild preflight in both coalesce and immediate paths (found {})",
        count
    );
}
