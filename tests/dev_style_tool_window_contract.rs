use std::fs;

#[test]
fn dev_style_tool_window_is_env_gated_and_startup_scoped() {
    let window_source = fs::read_to_string("src/dev_style_tool/window.rs")
        .expect("read dev style tool window source");
    let startup_source =
        fs::read_to_string("src/main_entry/app_run_setup.rs").expect("read startup source");
    let design_sh = fs::read_to_string("design.sh").expect("read design.sh");

    assert!(window_source.contains("SCRIPT_KIT_STYLE_DEVTOOLS"));
    assert!(window_source.contains("cfg!(debug_assertions)"));
    assert!(window_source.contains("maybe_open_startup_sidecar"));
    assert!(window_source.contains("open_dev_style_tool_window"));
    assert!(design_sh.contains("export SCRIPT_KIT_STYLE_DEVTOOLS=1"));
    assert!(
        design_sh.contains("export SCRIPT_KIT_DEV_FORCE_RELAUNCH=1"),
        "design.sh must relaunch the dev session so the style-tool env reaches app startup"
    );
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
    assert!(render_source.contains("group_controls_by_section"));
    assert!(render_source.contains("render_control_section"));
    assert!(render_source.contains("StyleKnobSection::for_knob"));
    assert!(render_source.contains("style-subsection:{}:{}"));
    assert!(render_source.contains("save_current_settings"));
    assert!(render_source.contains("button:dev-style-tool-save"));
    assert!(render_source.contains("button:dev-style-tool-undo"));
    assert!(render_source.contains("button:dev-style-tool-redo"));
    assert!(render_source.contains("button:dev-style-tool-reset-all"));
    assert!(render_source.contains("button:dev-style-tool-copy-markdown"));
    assert!(render_source.contains("tabs:dev-style-tool-groups"));
    assert!(render_source.contains("tabs:dev-style-tool-primary"));
    assert!(render_source.contains("tabs:dev-style-tool-actions-groups"));
    assert!(render_source.contains("tabs:dev-style-tool-agent-chat-groups"));
    assert!(render_source.contains("tabs:dev-style-tool-confirm-modal-groups"));
    assert!(render_source.contains("summary:dev-style-tool-active-scope"));
    assert!(render_source.contains("render_active_scope_summary"));
    assert!(render_source.contains("active_actions_group"));
    assert!(render_source.contains("active_agent_chat_group"));
    assert!(render_source.contains("active_confirm_modal_group"));
    // Navigation is a Storybook-style sidebar (surface -> group tree), not
    // horizontal tab bars; the legacy `tabs:*` ids live on its containers.
    assert!(render_source.contains("fn render_sidebar"));
    assert!(render_source.contains("sidebar:dev-style-tool"));
    assert!(render_source.contains("render_content_header"));
    assert!(render_source.contains("Text / Copy"));
    assert!(render_source.contains("tab:dev-style-tool:text-copy"));
    assert!(render_source.contains("Actions Popup Styling"));
    assert!(render_source.contains("tab:dev-style-tool:actions-popup-styling"));
    assert!(render_source.contains("Confirm Modal Styling"));
    assert!(render_source.contains("tab:dev-style-tool:confirm-modal-styling"));
    assert!(render_source.contains("active_group"));
    assert!(render_source.contains("input:dev-style-tool-saved-markdown"));
    assert!(render_source.contains("save_current_settings_markdown_with_contents"));
    assert!(render_source.contains("undo_style_change"));
    assert!(render_source.contains("redo_style_change"));
    assert!(render_source.contains("reset_all_controls"));
    assert!(render_source.contains("sync_all_controls"));
    assert!(render_source.contains("body:dev-style-tool-scroll"));
    assert!(render_source.contains(".min_h_0()"));
    assert!(render_source.contains(".overflow_y_scroll()"));
    assert!(render_source.contains("export::save_current_settings_markdown"));
    assert!(render_source.contains("runtime_overrides::set_value"));
    assert!(render_source.contains("runtime_overrides::set_copy_value"));
    assert!(render_source.contains("runtime_overrides::set_actions_popup_value"));
    assert!(render_source.contains("runtime_overrides::set_confirm_modal_value"));
    assert!(render_source.contains("runtime_overrides::reset_value"));
    assert!(render_source.contains("main_app.update"));
    assert!(render_source.contains("update_theme"));
    assert!(!render_source.contains("SEARCH_HEIGHT_KNOB_ID"));
    assert!(!render_source.contains("build_layout_info"));
    assert!(!render_source.contains("build_component_bounds"));
    assert!(!render_source.contains("render_script_list"));
}

#[test]
fn dev_style_tool_initializes_slider_max_before_min() {
    let render_source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style render source");

    let sites: Vec<usize> = render_source
        .match_indices("SliderState::new()")
        .map(|(idx, _)| idx)
        .collect();
    assert!(
        sites.len() >= 2,
        "dev style render must construct sliders via SliderState::new() (found {})",
        sites.len()
    );
    for site in sites {
        let window = &render_source[site..(site + 400).min(render_source.len())];
        let max_pos = window
            .find(".max(")
            .expect("each SliderState::new() must chain .max(...)");
        let min_pos = window
            .find(".min(")
            .expect("each SliderState::new() must chain .min(...)");
        assert!(
            max_pos < min_pos,
            "dev style sliders must set max before min because some controls have min values above the slider default max"
        );
    }
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
    assert!(window_source.contains("fn is_dev_style_tool_open"));
    assert!(collector_source.contains("collect_dev_style_tool_snapshot"));
    assert!(collector_source.contains("button:dev-style-tool-save"));
    assert!(collector_source.contains("button:dev-style-tool-undo"));
    assert!(collector_source.contains("button:dev-style-tool-redo"));
    assert!(collector_source.contains("button:dev-style-tool-reset-all"));
    assert!(collector_source.contains("button:dev-style-tool-copy-markdown"));
    assert!(collector_source.contains("input:dev-style-tool-saved-markdown"));
    assert!(collector_source.contains("tab:dev-style-tool:main-window-styling"));
    assert!(collector_source.contains("tab:dev-style-tool:text-copy"));
    assert!(collector_source.contains("tab:dev-style-tool:actions-popup-styling"));
    assert!(collector_source.contains("tab:dev-style-tool:agent-chat-styling"));
    assert!(collector_source.contains("tab:dev-style-tool:confirm-modal-styling"));
    assert!(collector_source.contains("summary:dev-style-tool-active-scope"));
    assert!(collector_source.contains("undoStyleChange"));
    assert!(collector_source.contains("redoStyleChange"));
    assert!(collector_source.contains("resetStyleControls"));
    assert!(collector_source.contains("saveCurrentStyleSettings"));
    assert!(collector_source.contains("copySavedStyleMarkdown"));
    assert!(collector_source.contains("crate::dev_style_tool::STYLE_KNOBS"));
    assert!(collector_source.contains("tab:dev-style-tool:{}"));
    assert!(collector_source.contains("style-section:{}"));
    assert!(collector_source.contains("style-subsection:{}:{}"));
    assert!(collector_source.contains("StyleKnobSection::for_knob"));
    assert!(collector_source.contains("slider:dev-style-tool:{}"));
    assert!(collector_source.contains("input:dev-style-tool:{}"));
    assert!(collector_source.contains("button:dev-style-tool-reset:{}"));
    assert!(collector_source.contains("crate::dev_style_tool::COPY_CONTROLS"));
    assert!(collector_source.contains("input:dev-style-tool-copy:{}"));
    assert!(collector_source.contains("button:dev-style-tool-copy-reset:{}"));
    assert!(collector_source.contains("crate::dev_style_tool::ACTIONS_POPUP_KNOBS"));
    assert!(collector_source.contains("tab:dev-style-tool-actions:{}"));
    assert!(collector_source.contains("actions-style-section:{}"));
    assert!(collector_source.contains("slider:dev-style-tool-actions:{}"));
    assert!(collector_source.contains("input:dev-style-tool-actions:{}"));
    assert!(collector_source.contains("button:dev-style-tool-actions-reset:{}"));
    assert!(collector_source.contains("crate::dev_style_tool::CONFIRM_MODAL_KNOBS"));
    assert!(collector_source.contains("tab:dev-style-tool-confirm-modal:{}"));
    assert!(collector_source.contains("confirm-modal-style-section:{}"));
    assert!(collector_source.contains("slider:dev-style-tool-confirm-modal:{}"));
    assert!(collector_source.contains("input:dev-style-tool-confirm-modal:{}"));
    assert!(collector_source.contains("button:dev-style-tool-confirm-modal-reset:{}"));
}

#[test]
fn dev_style_tool_sections_are_catalog_driven_by_control_criteria() {
    let catalog_source =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let render_source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style render source");

    assert!(catalog_source.contains("pub struct StyleKnobSection"));
    assert!(catalog_source.contains("pub fn for_knob(knob: &StyleKnob) -> Self"));

    for expected in [
        "List geometry",
        "List sections",
        "Main hint panel",
        "Main hint rows",
        "Main hint fragments",
        "Main hint form",
        "Inline calculator",
        "Footer action slots",
        "Footer glyph alignment",
        "Footer keycaps",
        "Header pills and keys",
    ] {
        assert!(
            catalog_source.contains(expected),
            "missing section label {expected}"
        );
    }

    assert!(render_source.contains("group_controls_by_section"));
    assert!(render_source.contains("render_control_section"));
    assert!(render_source.contains("style-subsection:{}:{}"));
    assert!(
        render_source.contains("render_control(control, chrome, cx)"),
        "sectioning must preserve existing control rendering and semantic ids"
    );
}

#[test]
fn dev_style_tool_devtools_mutation_reuses_runtime_catalog() {
    let catalog_source =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let runtime_source = fs::read_to_string("src/dev_style_tool/runtime_overrides.rs")
        .expect("read dev style runtime overrides");
    let prompt_handler_source =
        fs::read_to_string("src/prompt_handler/mod.rs").expect("read prompt handler");
    let protocol_source =
        fs::read_to_string("src/protocol/types/batch_wait.rs").expect("read batch protocol");

    assert!(catalog_source.contains("knob_id_from_str"));
    assert!(catalog_source.contains("LIST_ITEM_HEIGHT_KNOB_ID"));
    assert!(catalog_source.contains("ROW_INNER_PADDING_X_KNOB_ID"));
    assert!(catalog_source.contains("FOOTER_BUTTON_RADIUS_KNOB_ID"));
    assert!(catalog_source.contains("HEADER_INFO_HEIGHT_KNOB_ID"));
    assert!(runtime_source.contains("set_number_from_devtools"));
    assert!(runtime_source.contains("knob_id_from_str(control)"));
    assert!(runtime_source.contains("set_value(id, StyleValue::Number(parsed))"));
    assert!(runtime_source.contains("set_confirm_modal_number_from_devtools"));
    assert!(prompt_handler_source.contains("confirmModal."));
    assert!(prompt_handler_source.contains("set_confirm_modal_number_from_devtools"));
    assert!(runtime_source.contains("undo_stack"));
    assert!(runtime_source.contains("redo_stack"));
    assert!(runtime_source.contains("pub fn undo_last"));
    assert!(runtime_source.contains("pub fn redo_last"));
    assert!(runtime_source.contains("pub fn reset_all"));
    assert!(protocol_source.contains("UndoStyleChange"));
    assert!(protocol_source.contains("RedoStyleChange"));
    assert!(protocol_source.contains("ResetStyleControls"));
    assert!(prompt_handler_source.contains("AutomationBatchTargetKind::DevStyleTool"));
    assert!(prompt_handler_source.contains("\"undoStyleChange\""));
    assert!(prompt_handler_source.contains("\"redoStyleChange\""));
    assert!(prompt_handler_source.contains("\"resetStyleControls\""));
    assert!(prompt_handler_source.contains("SaveCurrentStyleSettings"));
    assert!(prompt_handler_source.contains("UndoStyleChange"));
    assert!(prompt_handler_source.contains("RedoStyleChange"));
    assert!(prompt_handler_source.contains("ResetStyleControls"));
    assert!(prompt_handler_source.contains("\"saveCurrentStyleSettings\""));
    assert!(prompt_handler_source
        .contains("crate::dev_style_tool::export::save_current_settings_markdown"));
    assert!(prompt_handler_source.contains("set_number_from_devtools"));
    assert!(prompt_handler_source.contains("this.update_theme(cx);"));
}
