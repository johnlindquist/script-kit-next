#[test]
fn hotkey_startup_does_not_register_shortcuts_json() {
    let hotkeys = super::read_source("src/hotkeys/mod.rs");

    assert!(
        !hotkeys.contains("load_shortcut_overrides("),
        "start_hotkey_listener must not register shortcuts.json as an active hotkey source"
    );
    assert!(
        !hotkeys.contains("shortcuts.json"),
        "hotkey routing comments/logging should not describe shortcuts.json as active"
    );
}

#[test]
fn recorder_writes_config_shortcuts() {
    let recorder = super::read_source("src/app_impl/shortcut_recorder.rs");

    assert!(
        recorder.contains("update-config-shortcut.ts"),
        "shortcut recorder should write command shortcuts through config.ts tooling"
    );
    assert!(
        !recorder.contains("save_shortcut_override("),
        "shortcut recorder must not persist new shortcuts to shortcuts.json"
    );
}

#[test]
fn remove_action_writes_config_shortcuts() {
    let actions = super::read_source("src/app_actions/handle_action/shortcuts.rs");

    assert!(
        actions.contains("remove_config_command_shortcut("),
        "remove_shortcut should remove the shortcut field from config.ts"
    );
    assert!(
        !actions.contains("remove_shortcut_override("),
        "remove_shortcut must not mutate shortcuts.json"
    );
}

#[test]
fn dynamic_shortcut_unregister_absent_binding_is_noop() {
    let hotkeys = super::read_source("src/hotkeys/mod.rs");
    let unregister_pos = hotkeys
        .find("pub fn unregister_dynamic_shortcut")
        .expect("unregister_dynamic_shortcut not found");
    let block = &hotkeys[unregister_pos..hotkeys.len().min(unregister_pos + 2500)];

    assert!(
        block.contains("unregister treated as no-op") && block.contains("return Ok(())"),
        "dynamic shortcut removal should be a no-op when config has no live route"
    );
}

#[test]
fn preview_and_focused_info_read_config_shortcuts() {
    let preview = super::read_source("src/app_render/preview_panel.rs");
    let focused = super::read_source("src/app_render/focused_info.rs");

    assert!(
        preview.contains("get_command_shortcut(&command_id)")
            && focused.contains("get_command_shortcut(id)"),
        "shortcut display should read config.ts command shortcuts"
    );
    assert!(
        !preview.contains("get_cached_shortcut_overrides")
            && !focused.contains("get_cached_shortcut_overrides"),
        "shortcut display must not read shortcuts.json overrides"
    );
}
