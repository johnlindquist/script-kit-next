use super::*;

fn find_action_title(actions: &[Action], id: &str) -> String {
    actions
        .iter()
        .find(|action| action.id == id)
        .map(|action| action.title.clone())
        .expect("action id should exist in script context actions")
}

fn find_action_description(actions: &[Action], id: &str) -> Option<String> {
    actions
        .iter()
        .find(|action| action.id == id)
        .and_then(|action| action.description.clone())
}

fn has_action(actions: &[Action], id: &str) -> bool {
    actions.iter().any(|action| action.id == id)
}

fn assert_all_actions_have_icons(context: &str, actions: &[Action]) {
    for action in actions {
        assert!(
            action.icon.is_some(),
            "context '{context}' action '{}' should include an icon",
            action.id
        );
    }
}

#[test]
fn test_get_script_context_actions_returns_empty_when_name_is_blank() {
    let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
    script.name = "   ".to_string();

    let actions = get_script_context_actions(&script);
    assert!(actions.is_empty());
}

#[test]
fn test_get_script_context_actions_returns_empty_when_action_verb_is_blank() {
    let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
    script.action_verb = "   ".to_string();

    let actions = get_script_context_actions(&script);
    assert!(actions.is_empty());
}

#[test]
fn test_get_script_context_actions_run_label_uses_title_case_verb() {
    let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
    script.action_verb = "switch to".to_string();

    let actions = get_script_context_actions(&script);

    assert_eq!(find_action_title(&actions, "run_script"), "Switch To");
}

#[test]
fn test_get_script_context_actions_preserves_builtin_action_text() {
    let script = ScriptInfo::with_action_verb(
        "Agent Chat",
        "builtin:builtin/ai-chat",
        false,
        "Open Agent Chat",
    );

    let actions = get_script_context_actions(&script);

    assert_eq!(find_action_title(&actions, "run_script"), "Open Agent Chat");
    assert_eq!(
        find_action_description(&actions, "run_script").as_deref(),
        Some("Open Agent Chat")
    );
}

#[test]
fn script_context_primary_action_consumes_the_selected_command_descriptor() {
    use sk_protocol::command_contract::{CommandDescriptor, CommandIdentity, CommandSource};

    let descriptor = CommandDescriptor::new(
        CommandIdentity::new(CommandSource::Script, "main:hello").unwrap(),
        "Hello",
        "Open Reviewed Result",
    )
    .unwrap();
    let script =
        ScriptInfo::new("Hello", "/tmp/hello.ts").with_command_descriptor(descriptor.clone());
    let actions = get_script_context_actions(&script);
    let primary = actions
        .iter()
        .find(|action| action.id == "run_script")
        .unwrap();

    assert_eq!(primary.title, descriptor.primary_action().unwrap().title);
    assert!(primary.is_enabled());
    assert_eq!(
        script
            .command_descriptor
            .as_ref()
            .unwrap()
            .identity
            .as_str(),
        "script/main:hello"
    );
}

#[test]
fn script_context_primary_action_preserves_the_canonical_disabled_reason() {
    use sk_protocol::command_contract::{
        CommandAvailability, CommandCapability, CommandDescriptor, CommandIdentity, CommandSource,
    };

    let mut descriptor = CommandDescriptor::new(
        CommandIdentity::new(CommandSource::Scriptlet, "main:camera").unwrap(),
        "Camera",
        "Inspect Camera",
    )
    .unwrap();
    descriptor.availability = CommandAvailability::MissingPermission {
        capability: CommandCapability::Microphone,
    };
    descriptor.actions[0].availability = descriptor.availability.clone();

    let script = ScriptInfo::scriptlet("Camera", "/tmp/camera.md", None, None)
        .with_command_descriptor(descriptor);
    let actions = get_script_context_actions(&script);
    let primary = actions
        .iter()
        .find(|action| action.id == "run_script")
        .unwrap();

    assert!(!primary.is_enabled());
    assert_eq!(
        primary.disabled_reason(),
        Some("Grant the required permission to continue.")
    );
}

#[test]
fn test_primary_action_plan_classifies_context_kind_matrix() {
    let app = ScriptInfo::app("App", "/Applications/App.app", None, None, None);
    let agent = ScriptInfo::agent("Agent", "/tmp/agent.md", None, None);
    let scriptlet = ScriptInfo::scriptlet("Scriptlet", "/tmp/scriptlets.md", None, None);
    let script = ScriptInfo::new("Script", "/tmp/script.ts");
    let skill = ScriptInfo::with_action_verb("Skill", "skill:scriptkit:demo", false, "open");
    let builtin = ScriptInfo::with_action_verb(
        "Agent Chat",
        "builtin:builtin/ai-chat",
        false,
        "Open Agent Chat",
    );
    let generic = ScriptInfo::with_action_verb("Thing", "virtual:thing", false, "open");
    let typed_builtin_path =
        ScriptInfo::with_action_verb("Typed", "builtin:builtin/new-script", true, "run");

    assert_eq!(script_context_kind(&app), ScriptContextKind::App);
    assert_eq!(script_context_kind(&agent), ScriptContextKind::Agent);
    assert_eq!(
        script_context_kind(&scriptlet),
        ScriptContextKind::Scriptlet
    );
    assert_eq!(script_context_kind(&script), ScriptContextKind::Script);
    assert_eq!(script_context_kind(&skill), ScriptContextKind::Skill);
    assert_eq!(script_context_kind(&builtin), ScriptContextKind::BuiltIn);
    assert_eq!(script_context_kind(&generic), ScriptContextKind::Generic);
    assert_eq!(
        script_context_kind(&typed_builtin_path),
        ScriptContextKind::Script
    );
}

#[test]
fn test_primary_action_copy_matrix() {
    let mut script = ScriptInfo::new("Script", "/tmp/script.ts");
    script.action_verb = "run".to_string();
    let mut app = ScriptInfo::app("App", "/Applications/App.app", None, None, None);
    app.action_verb = "launch".to_string();
    let mut scriptlet = ScriptInfo::scriptlet("Scriptlet", "/tmp/scriptlets.md", None, None);
    scriptlet.action_verb = "run".to_string();
    let mut agent = ScriptInfo::agent("Agent", "/tmp/agent.md", None, None);
    agent.action_verb = "open".to_string();
    let skill = ScriptInfo::with_action_verb("Skill", "skill:scriptkit:demo", false, "open");
    let generic = ScriptInfo::with_action_verb("Thing", "virtual:thing", false, "open");
    let builtin = ScriptInfo::with_action_verb(
        "Agent Chat",
        "builtin:builtin/ai-chat",
        false,
        "Open Agent Chat",
    );

    for (context, script, expected) in [
        (
            "script",
            script,
            PrimaryActionCopy {
                title: "Run".to_string(),
                description: "run this script".to_string(),
            },
        ),
        (
            "app",
            app,
            PrimaryActionCopy {
                title: "Launch".to_string(),
                description: "launch this application".to_string(),
            },
        ),
        (
            "scriptlet",
            scriptlet,
            PrimaryActionCopy {
                title: "Run".to_string(),
                description: "run this scriptlet".to_string(),
            },
        ),
        (
            "agent",
            agent,
            PrimaryActionCopy {
                title: "Open".to_string(),
                description: "open this agent".to_string(),
            },
        ),
        (
            "skill",
            skill,
            PrimaryActionCopy {
                title: "Open".to_string(),
                description: "open this skill".to_string(),
            },
        ),
        (
            "generic",
            generic,
            PrimaryActionCopy {
                title: "Open".to_string(),
                description: "open this item".to_string(),
            },
        ),
        (
            "builtin",
            builtin,
            PrimaryActionCopy {
                title: "Open Agent Chat".to_string(),
                description: "Open Agent Chat".to_string(),
            },
        ),
    ] {
        assert_eq!(primary_action_copy(&script), expected, "{context}");
    }
}

#[test]
fn test_favorite_action_copy_returns_add_copy_when_not_favorite() {
    let (title, description) = favorite_action_copy(false);

    assert_eq!(title, "Add to Favorites");
    assert_eq!(description, "Save this item to your favorites list");
}

#[test]
fn test_favorite_action_copy_returns_remove_copy_when_favorite() {
    let (title, description) = favorite_action_copy(true);

    assert_eq!(title, "Remove from Favorites");
    assert_eq!(description, "Remove this item from your favorites list");
}

#[test]
fn test_get_script_context_actions_includes_toggle_favorite_for_script_items() {
    let script = ScriptInfo::new("Valid", "/tmp/script-context-favorites-test.ts");

    let actions = get_script_context_actions(&script);

    assert!(has_action(&actions, "toggle_favorite"));
}

#[test]
fn test_get_script_context_actions_skips_toggle_favorite_for_builtin_items() {
    let script = ScriptInfo::builtin("Clipboard History");

    let actions = get_script_context_actions(&script);

    assert!(!has_action(&actions, "toggle_favorite"));
}

#[test]
fn test_get_script_context_actions_labels_use_consistent_verb_style() {
    let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
    script.shortcut = Some("cmd-shift-k".to_string());
    script.alias = Some("v".to_string());
    script.is_suggested = true;

    let actions = get_script_context_actions(&script);

    assert_eq!(find_action_title(&actions, "run_script"), "Run");
    assert_eq!(
        find_action_title(&actions, "update_shortcut"),
        "Edit Keyboard Shortcut"
    );
    assert_eq!(
        find_action_title(&actions, "remove_shortcut"),
        "Delete Keyboard Shortcut"
    );
    assert_eq!(find_action_title(&actions, "update_alias"), "Edit Alias");
    assert_eq!(find_action_title(&actions, "remove_alias"), "Delete Alias");
    assert_eq!(find_action_title(&actions, "view_logs"), "Show Logs");
    assert_eq!(
        find_action_title(&actions, "reveal_in_finder"),
        "Open in Finder"
    );
    assert_eq!(find_action_title(&actions, "copy_deeplink"), "Share");
    assert_eq!(
        find_action_title(&actions, "reset_ranking"),
        "Delete Ranking Entry"
    );

    for action in &actions {
        assert!(
            !action.title.ends_with("..."),
            "label should not end with ellipsis: {}",
            action.title
        );
        assert!(
            action.title.chars().count() < 30,
            "label should stay concise: {}",
            action.title
        );
    }
}

#[test]
fn test_get_script_context_actions_assigns_icons_for_all_contexts() {
    let script = ScriptInfo::new("Script", "/tmp/script-context-icon-test.ts");
    let builtin = ScriptInfo::builtin("Clipboard History");
    let scriptlet =
        ScriptInfo::scriptlet("Scriptlet", "/tmp/script-context-icon-test.md", None, None);
    let agent = ScriptInfo::agent(
        "Agent",
        "/tmp/script-context-icon-test.agent.md",
        None,
        None,
    );

    let script_actions = get_script_context_actions(&script);
    assert!(
        !script_actions.is_empty(),
        "script actions should not be empty"
    );
    assert_all_actions_have_icons("script", &script_actions);

    let builtin_actions = get_script_context_actions(&builtin);
    assert!(
        !builtin_actions.is_empty(),
        "builtin actions should not be empty"
    );
    assert_all_actions_have_icons("builtin", &builtin_actions);

    let scriptlet_actions = get_script_context_actions(&scriptlet);
    assert!(
        !scriptlet_actions.is_empty(),
        "scriptlet actions should not be empty"
    );
    assert_all_actions_have_icons("scriptlet", &scriptlet_actions);

    let agent_actions = get_script_context_actions(&agent);
    assert!(
        !agent_actions.is_empty(),
        "agent actions should not be empty"
    );
    assert_all_actions_have_icons("agent", &agent_actions);
}

#[test]
fn test_script_context_actions_include_toggle_info_with_cmd_i() {
    let script = ScriptInfo::new("TestScript", "/tmp/info-test.ts");
    let actions = get_script_context_actions(&script);

    let info_action = actions
        .iter()
        .find(|a| a.id == "toggle_info")
        .expect("script context actions must include toggle_info");

    assert_eq!(info_action.title, "Show Info");
    assert_eq!(
        info_action.shortcut.as_deref(),
        Some("⌘I"),
        "toggle_info action must have ⌘I shortcut for discoverability"
    );
    assert_eq!(
        info_action.section.as_deref(),
        Some("Actions"),
        "toggle_info must appear in the Actions section"
    );
    assert!(
        info_action.icon.is_some(),
        "toggle_info must have an icon for visual consistency"
    );
}

#[test]
fn test_get_script_context_actions_includes_app_actions_when_is_app() {
    let script = ScriptInfo::app(
        "Google Chrome",
        "/Applications/Google Chrome.app",
        Some("com.google.Chrome".to_string()),
        None,
        None,
    );
    let actions = get_script_context_actions(&script);

    // App-specific actions
    assert!(has_action(&actions, "reveal_in_finder"));
    assert!(has_action(&actions, "show_info_in_finder"));
    assert!(has_action(&actions, "show_package_contents"));
    assert!(has_action(&actions, "copy_name"));
    assert!(has_action(&actions, "copy_path"));
    assert!(has_action(&actions, "copy_bundle_id"));
    assert!(has_action(&actions, "quit_app"));
    assert!(has_action(&actions, "force_quit_app"));
    assert!(has_action(&actions, "restart_app"));

    // Common actions still present
    assert!(has_action(&actions, "run_script"));
    assert!(has_action(&actions, "toggle_info"));
    assert!(has_action(&actions, "copy_deeplink"));
}

#[test]
fn builtin_rows_include_copy_command_id_but_not_script_or_app_actions() {
    let script = ScriptInfo::with_all(
        "Clipboard History",
        "builtin:clipboard-history",
        false,
        "Open Clipboard History",
        None,
        None,
    );
    let actions = get_script_context_actions(&script);

    assert!(has_action(&actions, "copy_command_id"));
    assert!(has_action(&actions, "run_script"));
    assert!(has_action(&actions, "toggle_info"));
    assert!(!has_action(&actions, "edit_script"));
    assert!(!has_action(&actions, "reveal_in_finder"));
    assert!(!has_action(&actions, "quit_app"));
}

#[test]
fn test_get_script_context_actions_omits_copy_bundle_id_when_none() {
    let script = ScriptInfo::app("MyApp", "/Applications/MyApp.app", None, None, None);
    let actions = get_script_context_actions(&script);

    assert!(!has_action(&actions, "copy_bundle_id"));
    // Other app actions still present
    assert!(has_action(&actions, "reveal_in_finder"));
    assert!(has_action(&actions, "quit_app"));
}

#[test]
fn test_get_script_context_actions_app_does_not_include_script_only_actions() {
    let script = ScriptInfo::app(
        "Safari",
        "/Applications/Safari.app",
        Some("com.apple.Safari".to_string()),
        None,
        None,
    );
    let actions = get_script_context_actions(&script);

    assert!(!has_action(&actions, "edit_script"));
    assert!(!has_action(&actions, "view_logs"));
    assert!(!has_action(&actions, "copy_content"));
    assert!(!has_action(&actions, "delete_script"));
    assert!(!has_action(&actions, "edit_scriptlet"));
}

#[test]
fn test_get_script_context_actions_includes_favorites_for_apps() {
    let script = ScriptInfo::app(
        "Safari",
        "/Applications/Safari.app",
        Some("com.apple.Safari".to_string()),
        None,
        None,
    );
    let actions = get_script_context_actions(&script);

    assert!(has_action(&actions, "toggle_favorite"));
}

#[test]
fn test_get_script_context_actions_app_actions_all_have_icons() {
    let script = ScriptInfo::app(
        "Chrome",
        "/Applications/Google Chrome.app",
        Some("com.google.Chrome".to_string()),
        None,
        None,
    );
    let actions = get_script_context_actions(&script);
    assert_all_actions_have_icons("app", &actions);
}

#[test]
fn test_toggle_info_appears_for_all_script_types() {
    let script = ScriptInfo::new("Script", "/tmp/all-types-info.ts");
    let builtin = ScriptInfo::builtin("Clipboard History");
    let scriptlet = ScriptInfo::scriptlet("Scriptlet", "/tmp/all-types-info.md", None, None);
    let agent = ScriptInfo::agent("Agent", "/tmp/all-types-info.agent.md", None, None);

    for (label, actions) in [
        ("script", get_script_context_actions(&script)),
        ("builtin", get_script_context_actions(&builtin)),
        ("scriptlet", get_script_context_actions(&scriptlet)),
        ("agent", get_script_context_actions(&agent)),
    ] {
        assert!(
            actions.iter().any(|a| a.id == "toggle_info"),
            "toggle_info must be present in {label} context actions"
        );
    }
}
