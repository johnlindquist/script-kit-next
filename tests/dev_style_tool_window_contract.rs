use std::fs;

#[test]
fn dev_style_tool_window_is_env_gated_and_startup_scoped() {
    let window_source = fs::read_to_string("src/dev_style_tool/window.rs")
        .expect("read dev style tool window source");
    let startup_source =
        fs::read_to_string("src/main_entry/app_run_setup.rs").expect("read startup source");

    assert!(window_source.contains("SCRIPT_KIT_STYLE_DEVTOOLS"));
    assert!(window_source.contains("cfg!(debug_assertions)"));
    assert!(window_source.contains("maybe_open_startup_sidecar"));
    assert!(window_source.contains("open_dev_style_tool_window"));
    assert!(startup_source.contains(
        "crate::dev_style_tool::window::maybe_open_startup_sidecar(window, app_entity.clone(), cx)"
    ));
    assert!(
        !startup_source.contains("openDevStyleTool"),
        "iteration 2 must not add a broad command opener"
    );
}

#[test]
fn dev_style_tool_holds_safe_main_target_handles() {
    let render_source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style render source");

    assert!(render_source.contains("WindowHandle<Root>"));
    assert!(render_source.contains("Entity<ScriptListApp>"));
    assert!(!render_source.contains("&'static mut"));
    assert!(!render_source.contains("*mut"));
}

#[test]
fn dev_style_tool_render_is_catalog_driven_and_narrow() {
    let render_source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style render source");

    assert!(render_source.contains("STYLE_KNOBS"));
    assert!(render_source.contains("knob_by_id"));
    assert!(render_source.contains("render_groups"));
    assert!(render_source.contains("render_control"));
    assert!(render_source.contains("save_current_settings"));
    assert!(render_source.contains("button:dev-style-tool-save"));
    assert!(render_source.contains("export::save_current_settings_markdown"));
    assert!(render_source.contains("runtime_overrides::set_value"));
    assert!(render_source.contains("runtime_overrides::reset_value"));
    assert!(render_source.contains("main_app.update"));
    assert!(render_source.contains("update_theme"));
    assert!(!render_source.contains("SEARCH_HEIGHT_KNOB_ID"));
    assert!(!render_source.contains("build_layout_info"));
    assert!(!render_source.contains("build_component_bounds"));
    assert!(!render_source.contains("render_script_list"));
}

#[test]
fn dev_style_tool_registers_minimal_automation_target() {
    let window_source = fs::read_to_string("src/dev_style_tool/window.rs")
        .expect("read dev style tool window source");
    let kind_source = fs::read_to_string("src/protocol/types/automation_window.rs")
        .expect("read automation window type source");
    let registry_source = fs::read_to_string("src/windows/automation_registry.rs")
        .expect("read automation registry source");
    let collector_source = fs::read_to_string("src/windows/automation_surface_collector.rs")
        .expect("read automation surface collector source");

    assert!(kind_source.contains("DevStyleTool"));
    assert!(kind_source.contains("\"devStyleTool\""));
    assert!(registry_source.contains("AutomationWindowKind::DevStyleTool"));
    assert!(window_source.contains("upsert_automation_window"));
    assert!(window_source.contains("remove_automation_window(DEV_STYLE_TOOL_AUTOMATION_ID)"));
    assert!(collector_source.contains("collect_dev_style_tool_snapshot"));
    assert!(collector_source.contains("button:dev-style-tool-save"));
    assert!(collector_source.contains("saveCurrentStyleSettings"));
    assert!(collector_source.contains("crate::dev_style_tool::STYLE_KNOBS"));
    assert!(collector_source.contains("slider:dev-style-tool:{}"));
    assert!(collector_source.contains("input:dev-style-tool:{}"));
    assert!(collector_source.contains("button:dev-style-tool-reset:{}"));
}

#[test]
fn dev_style_tool_devtools_mutation_reuses_runtime_catalog() {
    let catalog_source =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let runtime_source = fs::read_to_string("src/dev_style_tool/runtime_overrides.rs")
        .expect("read dev style runtime overrides");
    let prompt_handler_source =
        fs::read_to_string("src/prompt_handler/mod.rs").expect("read prompt handler");

    assert!(catalog_source.contains("knob_id_from_str"));
    assert!(catalog_source.contains("LIST_ITEM_HEIGHT_KNOB_ID"));
    assert!(catalog_source.contains("ROW_INNER_PADDING_X_KNOB_ID"));
    assert!(catalog_source.contains("FOOTER_BUTTON_RADIUS_KNOB_ID"));
    assert!(catalog_source.contains("HEADER_INFO_HEIGHT_KNOB_ID"));
    assert!(runtime_source.contains("set_number_from_devtools"));
    assert!(runtime_source.contains("knob_id_from_str(control)"));
    assert!(runtime_source.contains("set_value(id, StyleValue::Number(parsed))"));
    assert!(prompt_handler_source.contains("AutomationBatchTargetKind::DevStyleTool"));
    assert!(prompt_handler_source
        .contains("supported_commands: &[\"setThemeControl\", \"saveCurrentStyleSettings\"]"));
    assert!(prompt_handler_source.contains("SaveCurrentStyleSettings"));
    assert!(prompt_handler_source.contains("\"saveCurrentStyleSettings\""));
    assert!(prompt_handler_source
        .contains("crate::dev_style_tool::export::save_current_settings_markdown"));
    assert!(prompt_handler_source.contains("set_number_from_devtools"));
    assert!(prompt_handler_source.contains("this.update_theme(cx);"));
}
