
// =========================================================================
// 29. Deeplink in script actions
// =========================================================================

#[test]
fn script_deeplink_description_contains_formatted_name() {
    let script = ScriptInfo::new("My Cool Script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let deeplink = find_action(&actions, "copy_deeplink").unwrap();
    assert!(
        deeplink
            .description
            .as_ref()
            .unwrap()
            .contains("my-cool-script"),
        "Deeplink description should contain formatted name"
    );
}

#[test]
fn builtin_also_has_deeplink() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_deeplink"));
}

// =========================================================================
// 30. Primary action verb propagation
// =========================================================================

#[test]
fn primary_action_uses_action_verb() {
    let script = ScriptInfo::with_action_verb("App Launcher", "builtin:launcher", false, "Launch");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(run.title.starts_with("Launch"));
    assert!(run.description.as_ref().unwrap().contains("Launch"),);
}

#[test]
fn default_action_verb_is_run() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(run.title.starts_with("Run"));
}
