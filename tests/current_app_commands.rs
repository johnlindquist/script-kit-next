//! Integration tests for the "Current App Commands" built-in.
//!
//! These tests verify registration, snapshot contract, and entry shaping
//! without requiring a live frontmost app or Accessibility permission.

use script_kit_gpui::builtins::{
    self, AiCommandType, BuiltInEntry, BuiltInFeature, BuiltInGroup, MenuBarActionInfo,
    UtilityCommandType,
};
use script_kit_gpui::config::BuiltInConfig;
use script_kit_gpui::menu_bar::current_app_commands::{
    build_current_app_command_recipe, build_current_app_intent_trace_receipt,
    build_generate_script_prompt_from_snapshot, normalize_trace_current_app_intent_request,
    normalize_turn_this_into_a_command_request, resolve_do_in_current_app_intent,
    suggest_current_app_command_name, DoInCurrentAppAction, FrontmostMenuSnapshot,
    CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION,
};
use script_kit_gpui::menu_bar::{KeyboardShortcut, MenuBarItem, ModifierFlags};

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

#[test]
fn do_in_current_app_builtin_is_registered() {
    let entries = builtins::get_builtin_entries(&BuiltInConfig::default());
    let entry = entries
        .iter()
        .find(|e| e.id == "builtin-do-in-current-app")
        .expect("builtin-do-in-current-app must be in the registry");

    assert_eq!(
        entry.feature,
        BuiltInFeature::UtilityCommand(UtilityCommandType::DoInCurrentApp)
    );

    // Must appear before builtin-current-app-commands
    let do_pos = entries.iter().position(|e| e.id == "builtin-do-in-current-app").unwrap();
    let cmd_pos = entries.iter().position(|e| e.id == "builtin-current-app-commands").unwrap();
    assert!(
        do_pos < cmd_pos,
        "builtin-do-in-current-app (pos {}) must appear before builtin-current-app-commands (pos {})",
        do_pos, cmd_pos
    );
}

#[test]
fn current_app_commands_builtin_is_registered() {
    let entries = builtins::get_builtin_entries(&BuiltInConfig::default());
    let found = entries
        .iter()
        .find(|e| e.id == "builtin-current-app-commands");
    assert!(
        found.is_some(),
        "builtin-current-app-commands must be in the registry"
    );
    let entry = found.unwrap();
    assert_eq!(
        entry.feature,
        BuiltInFeature::UtilityCommand(UtilityCommandType::CurrentAppCommands)
    );
}

#[test]
fn trace_current_app_intent_builtin_is_registered() {
    let entries = builtins::get_builtin_entries(&BuiltInConfig::default());
    let entry = entries
        .iter()
        .find(|e| e.id == "builtin-trace-current-app-intent")
        .expect("builtin-trace-current-app-intent must be in the registry");

    assert_eq!(
        entry.feature,
        BuiltInFeature::UtilityCommand(UtilityCommandType::TraceCurrentAppIntent)
    );
}

// ---------------------------------------------------------------------------
// leaf_name
// ---------------------------------------------------------------------------

#[test]
fn menu_bar_leaf_name_returns_last_path_segment() {
    let entry = BuiltInEntry::new_with_group(
        "menubar-com.apple.Safari-file-new-tab",
        "File → New Tab",
        "Safari  ⌘T",
        vec!["file".into(), "new".into(), "tab".into()],
        BuiltInFeature::MenuBarAction(MenuBarActionInfo {
            bundle_id: "com.apple.Safari".into(),
            menu_path: vec!["File".into(), "New Tab".into()],
            enabled: true,
            shortcut: Some("⌘T".into()),
        }),
        Some("📁".into()),
        BuiltInGroup::MenuBar,
    );

    assert_eq!(entry.leaf_name(), "New Tab");
}

// ---------------------------------------------------------------------------
// Frecency key derivation
// ---------------------------------------------------------------------------

#[test]
fn frecency_keys_for_builtins_are_derived_from_entry_id() {
    let entry = BuiltInEntry::new_with_group(
        "menubar-com.apple.Safari-file-new-tab",
        "File → New Tab",
        "Safari  ⌘T",
        vec!["file".into(), "new".into(), "tab".into()],
        BuiltInFeature::MenuBarAction(MenuBarActionInfo {
            bundle_id: "com.apple.Safari".into(),
            menu_path: vec!["File".into(), "New Tab".into()],
            enabled: true,
            shortcut: Some("⌘T".into()),
        }),
        Some("📁".into()),
        BuiltInGroup::MenuBar,
    );

    let key = format!("builtin:{}", entry.id);
    assert_eq!(key, "builtin:menubar-com.apple.Safari-file-new-tab");
    // Must differ from name-based key
    assert_ne!(key, format!("builtin:{}", entry.name));
}

// ---------------------------------------------------------------------------
// menu_bar_items_to_entries: representative menu path & shortcut shape
// ---------------------------------------------------------------------------

fn sample_safari_items() -> Vec<MenuBarItem> {
    vec![
        // Apple menu (skipped by convention)
        MenuBarItem {
            title: "Apple".into(),
            enabled: true,
            shortcut: None,
            children: vec![],
            ax_element_path: vec![0],
        },
        // File menu with mixed children
        MenuBarItem {
            title: "File".into(),
            enabled: true,
            shortcut: None,
            children: vec![
                MenuBarItem {
                    title: "New Tab".into(),
                    enabled: true,
                    shortcut: Some(KeyboardShortcut::new("T".into(), ModifierFlags::COMMAND)),
                    children: vec![],
                    ax_element_path: vec![1, 0],
                },
                MenuBarItem::separator(vec![1, 1]),
                MenuBarItem {
                    title: "Close All".into(),
                    enabled: false,
                    shortcut: None,
                    children: vec![],
                    ax_element_path: vec![1, 2],
                },
            ],
            ax_element_path: vec![1],
        },
    ]
}

#[test]
fn menu_bar_items_to_entries_shape() {
    let entries =
        builtins::menu_bar_items_to_entries(&sample_safari_items(), "com.apple.Safari", "Safari");

    // Separator and disabled item are excluded
    assert_eq!(entries.len(), 1);

    let e = &entries[0];
    assert_eq!(e.name, "File → New Tab");
    assert_eq!(e.id, "menubar-com.apple.Safari-file-new-tab");
    assert!(e.description.contains("⌘T"));
    assert!(e.description.contains("Safari"));
    assert_eq!(e.group, BuiltInGroup::MenuBar);

    if let BuiltInFeature::MenuBarAction(ref info) = e.feature {
        assert_eq!(info.bundle_id, "com.apple.Safari");
        assert_eq!(info.menu_path, vec!["File", "New Tab"]);
        assert!(info.enabled);
    } else {
        panic!("expected MenuBarAction");
    }
}

// ---------------------------------------------------------------------------
// FrontmostMenuSnapshot contract
// ---------------------------------------------------------------------------

#[test]
fn snapshot_into_entries_delegates_correctly() {
    let snapshot = FrontmostMenuSnapshot {
        app_name: "Safari".into(),
        bundle_id: "com.apple.Safari".into(),
        items: sample_safari_items(),
    };

    let placeholder = snapshot.placeholder();
    assert!(
        placeholder.contains("Safari"),
        "placeholder should contain app name"
    );

    let entries = snapshot.into_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "File → New Tab");
}

#[test]
fn snapshot_empty_items_yields_empty_entries() {
    let snapshot = FrontmostMenuSnapshot {
        app_name: "TestApp".into(),
        bundle_id: "com.example.TestApp".into(),
        items: vec![],
    };

    assert!(snapshot.into_entries().is_empty());
}

// ---------------------------------------------------------------------------
// Keyword enrichment: app name + shortcut aliases
// ---------------------------------------------------------------------------

#[test]
fn menu_bar_items_to_entries_skips_disabled_items_and_separators() {
    let entries = script_kit_gpui::builtins::menu_bar_items_to_entries(
        &sample_safari_items(),
        "com.apple.Safari",
        "Safari",
    );

    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names, vec!["File → New Tab"]);
}

#[test]
fn menu_bar_entry_keywords_include_app_name_and_shortcut_aliases() {
    let entries = script_kit_gpui::builtins::menu_bar_items_to_entries(
        &sample_safari_items(),
        "com.apple.Safari",
        "Safari",
    );

    let entry = entries
        .iter()
        .find(|entry| entry.name == "File → New Tab")
        .expect("New Tab entry should exist");

    assert!(
        entry.keywords.contains(&"safari".to_string()),
        "keywords should contain app name; got: {:?}",
        entry.keywords
    );
    assert!(
        entry.keywords.contains(&"⌘t".to_string()),
        "keywords should contain ⌘t; got: {:?}",
        entry.keywords
    );
    assert!(
        entry.keywords.contains(&"cmd+t".to_string()),
        "keywords should contain cmd+t; got: {:?}",
        entry.keywords
    );
    assert!(
        entry.keywords.contains(&"cmd t".to_string()),
        "keywords should contain 'cmd t'; got: {:?}",
        entry.keywords
    );
    assert!(
        entry.keywords.contains(&"cmdt".to_string()),
        "keywords should contain cmdt; got: {:?}",
        entry.keywords
    );
}

#[test]
fn menu_bar_entry_query_matching_supports_multi_term_queries() {
    let entries = script_kit_gpui::builtins::menu_bar_items_to_entries(
        &sample_safari_items(),
        "com.apple.Safari",
        "Safari",
    );

    let entry = entries
        .iter()
        .find(|entry| entry.name == "File → New Tab")
        .expect("New Tab entry should exist");

    assert!(script_kit_gpui::builtins::menu_bar_entry_matches_query(
        entry, "new tab"
    ));
    assert!(script_kit_gpui::builtins::menu_bar_entry_matches_query(
        entry, "safari"
    ));
    assert!(script_kit_gpui::builtins::menu_bar_entry_matches_query(
        entry, "⌘t"
    ));
    assert!(script_kit_gpui::builtins::menu_bar_entry_matches_query(
        entry, "cmd+t"
    ));
    assert!(script_kit_gpui::builtins::menu_bar_entry_matches_query(
        entry,
        "safari cmd+t"
    ));
    assert!(!script_kit_gpui::builtins::menu_bar_entry_matches_query(
        entry,
        "close all"
    ));
}

// ---------------------------------------------------------------------------
// Generate Script from Current App: registration
// ---------------------------------------------------------------------------

#[test]
fn generate_script_from_current_app_builtin_is_registered() {
    let entries = builtins::get_builtin_entries(&BuiltInConfig::default());
    let entry = entries
        .iter()
        .find(|e| e.id == "builtin-generate-script-from-current-app")
        .expect("builtin-generate-script-from-current-app must be in the registry");

    assert_eq!(
        entry.feature,
        BuiltInFeature::AiCommand(AiCommandType::GenerateScriptFromCurrentApp)
    );
}

// ---------------------------------------------------------------------------
// Generate Script from Current App: prompt shaping (integration)
// ---------------------------------------------------------------------------

fn safari_snapshot_with_menus() -> FrontmostMenuSnapshot {
    FrontmostMenuSnapshot {
        app_name: "Safari".into(),
        bundle_id: "com.apple.Safari".into(),
        items: vec![
            // Apple menu (skipped)
            MenuBarItem {
                title: "Apple".into(),
                enabled: true,
                shortcut: None,
                children: vec![],
                ax_element_path: vec![0],
            },
            MenuBarItem {
                title: "File".into(),
                enabled: true,
                shortcut: None,
                children: vec![
                    MenuBarItem {
                        title: "New Tab".into(),
                        enabled: true,
                        shortcut: Some(KeyboardShortcut::new("T".into(), ModifierFlags::COMMAND)),
                        children: vec![],
                        ax_element_path: vec![1, 0],
                    },
                    MenuBarItem {
                        title: "Close Window".into(),
                        enabled: true,
                        shortcut: Some(KeyboardShortcut::new("W".into(), ModifierFlags::COMMAND)),
                        children: vec![],
                        ax_element_path: vec![1, 1],
                    },
                ],
                ax_element_path: vec![1],
            },
        ],
    }
}

#[test]
fn prompt_shaping_includes_user_request_selected_text_and_browser_url() {
    let (prompt, receipt) = build_generate_script_prompt_from_snapshot(
        safari_snapshot_with_menus(),
        Some("close duplicate tabs"),
        Some("some selected text"),
        Some("https://example.com/page"),
    );

    assert_eq!(receipt.app_name, "Safari");
    assert_eq!(receipt.bundle_id, "com.apple.Safari");
    assert!(receipt.included_user_request);
    assert!(receipt.included_selected_text);
    assert!(receipt.included_browser_url);

    assert!(prompt.contains("User Request:\nclose duplicate tabs"));
    assert!(prompt.contains("Frontmost App: Safari"));
    assert!(prompt.contains("Bundle ID: com.apple.Safari"));
    assert!(prompt.contains("Selected Text:\n```text\nsome selected text\n```"));
    assert!(prompt.contains("Focused Browser URL:\nhttps://example.com/page"));
}

#[test]
fn prompt_shaping_includes_menu_shortcut_formatting() {
    let (prompt, receipt) =
        build_generate_script_prompt_from_snapshot(safari_snapshot_with_menus(), None, None, None);

    // Receipt tracks correct menu item count (Apple menu items are skipped)
    assert_eq!(receipt.included_menu_items, 2);
    assert!(!receipt.included_user_request);
    assert!(!receipt.included_selected_text);
    assert!(!receipt.included_browser_url);

    // Shortcuts should be formatted in parentheses
    assert!(
        prompt.contains("(⌘T)"),
        "Prompt should contain ⌘T shortcut, got:\n{}",
        prompt
    );
    assert!(
        prompt.contains("(⌘W)"),
        "Prompt should contain ⌘W shortcut, got:\n{}",
        prompt
    );
}

#[test]
fn prompt_shaping_truncates_to_20_menu_items() {
    let children: Vec<MenuBarItem> = (0..30)
        .map(|idx| MenuBarItem {
            title: format!("Action {}", idx),
            enabled: true,
            shortcut: None,
            children: vec![],
            ax_element_path: vec![1, idx],
        })
        .collect();

    let snapshot = FrontmostMenuSnapshot {
        app_name: "BigApp".into(),
        bundle_id: "com.example.BigApp".into(),
        items: vec![
            MenuBarItem {
                title: "Apple".into(),
                enabled: true,
                shortcut: None,
                children: vec![],
                ax_element_path: vec![0],
            },
            MenuBarItem {
                title: "Edit".into(),
                enabled: true,
                shortcut: None,
                children,
                ax_element_path: vec![1],
            },
        ],
    };

    let (prompt, receipt) = build_generate_script_prompt_from_snapshot(snapshot, None, None, None);

    assert_eq!(receipt.total_menu_items, 30);
    assert_eq!(receipt.included_menu_items, 20);
    assert!(prompt.contains("showing 20 of 30"));
}

// ---------------------------------------------------------------------------
// No-match intent → GenerateScript bridge
// ---------------------------------------------------------------------------

#[test]
fn do_in_current_app_no_match_bridges_cleanly_into_current_app_script_prompt() {
    let snap = safari_snapshot_with_menus();
    let entries = snap.clone().into_entries();

    // Step 1: Router returns GenerateScript for a query that matches no menu entry
    let (action, receipt) =
        resolve_do_in_current_app_intent(&entries, Some("close duplicate tabs"));

    assert_eq!(action, DoInCurrentAppAction::GenerateScript);
    assert_eq!(receipt.filtered_entries, 0);
    assert_eq!(receipt.exact_matches, 0);
    assert_eq!(receipt.action, "generate_script");

    // Step 2: The same snapshot feeds the prompt builder with full context
    let (prompt, prompt_receipt) = build_generate_script_prompt_from_snapshot(
        safari_snapshot_with_menus(),
        Some("close duplicate tabs"),
        Some("pricing"),
        Some("https://example.com/pricing"),
    );

    // Prompt includes user request, app metadata, and optional context
    assert!(
        prompt.contains("User Request:\nclose duplicate tabs"),
        "Prompt must include user request"
    );
    assert!(
        prompt.contains("Frontmost App: Safari"),
        "Prompt must include frontmost app name"
    );
    assert!(
        prompt.contains("Bundle ID: com.apple.Safari"),
        "Prompt must include bundle ID"
    );
    assert!(
        prompt.contains("Focused Browser URL:\nhttps://example.com/pricing"),
        "Prompt must include browser URL when provided"
    );

    // Receipt flags confirm context inclusion
    assert!(prompt_receipt.included_user_request);
    assert!(prompt_receipt.included_selected_text);
    assert!(prompt_receipt.included_browser_url);
}

// ---------------------------------------------------------------------------
// Trace Current App Intent: label normalization
// ---------------------------------------------------------------------------

#[test]
fn normalize_trace_current_app_intent_request_strips_builtin_label_prefix() {
    assert_eq!(
        normalize_trace_current_app_intent_request(Some("Trace Current App Intent")),
        None
    );
    assert_eq!(
        normalize_trace_current_app_intent_request(Some(
            "Trace Current App Intent close duplicate tabs"
        )),
        Some("close duplicate tabs".to_string())
    );
    assert_eq!(
        normalize_trace_current_app_intent_request(Some("trace current app intent: cmd+t")),
        Some("cmd+t".to_string())
    );
    assert_eq!(
        normalize_trace_current_app_intent_request(Some("   ")),
        None
    );
    assert_eq!(normalize_trace_current_app_intent_request(None), None);
    // Unrelated input is returned as-is
    assert_eq!(
        normalize_trace_current_app_intent_request(Some("close duplicate tabs")),
        Some("close duplicate tabs".to_string())
    );
}

// ---------------------------------------------------------------------------
// Trace Current App Intent: trace receipt builder
// ---------------------------------------------------------------------------

#[test]
fn trace_receipt_reports_execute_entry_for_shortcut_keyword() {
    let receipt = build_current_app_intent_trace_receipt(
        safari_snapshot_with_menus(),
        Some("Trace Current App Intent cmd+t"),
    );

    assert_eq!(receipt.action, "execute_entry");
    assert_eq!(receipt.filtered_entries, 1);
    assert_eq!(receipt.exact_matches, 1);
    assert_eq!(
        receipt
            .selected_entry
            .as_ref()
            .map(|item| item.leaf_name.as_str()),
        Some("New Tab")
    );
    assert!(receipt.prompt_preview.is_none());
    assert!(receipt.prompt_receipt.is_none());
    assert_eq!(receipt.schema_version, 1);
    assert_eq!(receipt.app_name, "Safari");
    assert_eq!(receipt.bundle_id, "com.apple.Safari");
}

#[test]
fn trace_receipt_reports_generate_script_and_includes_prompt_preview() {
    let receipt = build_current_app_intent_trace_receipt(
        safari_snapshot_with_menus(),
        Some("Trace Current App Intent close duplicate tabs"),
    );

    assert_eq!(receipt.action, "generate_script");
    assert_eq!(receipt.filtered_entries, 0);
    assert_eq!(receipt.exact_matches, 0);
    assert!(receipt.selected_entry.is_none());
    assert!(receipt.candidates.is_empty());
    assert!(receipt
        .prompt_preview
        .as_ref()
        .expect("prompt_preview should be present")
        .contains("User Request:\nclose duplicate tabs"));
    assert!(receipt
        .prompt_receipt
        .as_ref()
        .expect("prompt_receipt should be present")
        .included_user_request);
}

#[test]
fn trace_receipt_serializes_to_valid_json() {
    let receipt = build_current_app_intent_trace_receipt(
        safari_snapshot_with_menus(),
        Some("Trace Current App Intent cmd+t"),
    );

    let json = serde_json::to_string_pretty(&receipt).expect("receipt must serialize");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("JSON must be valid");

    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["action"], "execute_entry");
    assert_eq!(parsed["app_name"], "Safari");
}

// ---------------------------------------------------------------------------
// Turn This Into a Command — registration & contract
// ---------------------------------------------------------------------------

#[test]
fn turn_this_into_a_command_builtin_is_registered() {
    let entries = builtins::get_builtin_entries(&BuiltInConfig::default());
    let entry = entries
        .iter()
        .find(|e| e.id == "builtin-turn-this-into-a-command")
        .expect("builtin-turn-this-into-a-command must be in the registry");

    assert_eq!(
        entry.feature,
        BuiltInFeature::UtilityCommand(UtilityCommandType::TurnThisIntoCommand)
    );

    let do_pos = entries
        .iter()
        .position(|e| e.id == "builtin-do-in-current-app")
        .unwrap();
    let turn_pos = entries
        .iter()
        .position(|e| e.id == "builtin-turn-this-into-a-command")
        .unwrap();
    let cmd_pos = entries
        .iter()
        .position(|e| e.id == "builtin-current-app-commands")
        .unwrap();

    assert!(
        do_pos < turn_pos,
        "Turn This Into a Command should follow Do in Current App"
    );
    assert!(
        turn_pos < cmd_pos,
        "Turn This Into a Command should appear before Current App Commands"
    );
}

#[test]
fn normalize_turn_this_into_a_command_request_strips_builtin_label_prefix() {
    assert_eq!(
        normalize_turn_this_into_a_command_request(Some("Turn This Into a Command")),
        None
    );
    assert_eq!(
        normalize_turn_this_into_a_command_request(Some(
            "Turn This Into a Command close duplicate tabs"
        )),
        Some("close duplicate tabs".to_string())
    );
    assert_eq!(
        normalize_turn_this_into_a_command_request(Some("turn this into a command: cmd+t")),
        Some("cmd+t".to_string())
    );
    assert_eq!(
        normalize_turn_this_into_a_command_request(Some(
            "Turn This Into a Command \u{2014} archive inbox zero"
        )),
        Some("archive inbox zero".to_string())
    );
    assert_eq!(
        normalize_turn_this_into_a_command_request(Some("   ")),
        None
    );
    assert_eq!(normalize_turn_this_into_a_command_request(None), None);
}

#[test]
fn suggest_current_app_command_name_is_stable_and_human_readable() {
    assert_eq!(
        suggest_current_app_command_name("Safari", "close duplicate tabs"),
        "Safari Close Duplicate Tabs"
    );
    assert_eq!(
        suggest_current_app_command_name("Safari", "cmd+t"),
        "Safari Cmd T"
    );
    assert_eq!(
        suggest_current_app_command_name("Safari", ""),
        "Safari Command"
    );
}

#[test]
fn turn_this_into_a_command_recipe_keeps_trace_prompt_in_sync_for_generate_script() {
    let recipe = build_current_app_command_recipe(
        safari_snapshot_with_menus(),
        Some("Turn This Into a Command close duplicate tabs"),
        Some("pricing"),
        Some("browser-tab-url"),
    );

    assert_eq!(
        recipe.schema_version,
        CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION
    );
    assert_eq!(recipe.recipe_type, "currentAppCommand");
    assert_eq!(
        recipe.raw_query,
        "Turn This Into a Command close duplicate tabs"
    );
    assert_eq!(recipe.effective_query, "close duplicate tabs");
    assert_eq!(recipe.suggested_script_name, "Safari Close Duplicate Tabs");

    assert_eq!(recipe.trace.action, "generate_script");
    assert_eq!(
        recipe.trace.raw_query,
        "Turn This Into a Command close duplicate tabs"
    );
    assert_eq!(recipe.trace.effective_query, "close duplicate tabs");
    assert_eq!(
        recipe.trace.prompt_preview.as_deref(),
        Some(recipe.prompt.as_str())
    );
    assert_eq!(
        recipe.trace.prompt_receipt,
        Some(recipe.prompt_receipt.clone())
    );

    assert!(recipe
        .prompt
        .contains("User Request:\nclose duplicate tabs"));
    assert!(recipe
        .prompt
        .contains("Selected Text:\n```text\npricing\n```"));
    assert!(recipe
        .prompt
        .contains("Focused Browser URL:\nbrowser-tab-url"));
    assert!(recipe.prompt_receipt.included_user_request);
    assert!(recipe.prompt_receipt.included_selected_text);
    assert!(recipe.prompt_receipt.included_browser_url);
}

#[test]
fn turn_this_into_a_command_recipe_preserves_exact_match_trace_without_overwriting_it() {
    let recipe = build_current_app_command_recipe(
        safari_snapshot_with_menus(),
        Some("Turn This Into a Command new tab"),
        Some("selection"),
        Some("browser-tab-url"),
    );

    assert_eq!(recipe.effective_query, "new tab");
    assert_eq!(recipe.trace.action, "execute_entry");
    assert_eq!(
        recipe
            .trace
            .selected_entry
            .as_ref()
            .map(|item| item.leaf_name.as_str()),
        Some("New Tab")
    );
    assert!(recipe.trace.prompt_preview.is_none());
    assert!(recipe.trace.prompt_receipt.is_none());

    assert!(recipe.prompt.contains("User Request:\nnew tab"));
    assert!(recipe
        .prompt
        .contains("Selected Text:\n```text\nselection\n```"));
    assert!(recipe
        .prompt
        .contains("Focused Browser URL:\nbrowser-tab-url"));
}

#[test]
fn turn_this_into_a_command_recipe_serializes_to_valid_json() {
    let recipe = build_current_app_command_recipe(
        safari_snapshot_with_menus(),
        Some("Turn This Into a Command close duplicate tabs"),
        Some("pricing"),
        Some("browser-tab-url"),
    );

    let json = serde_json::to_string_pretty(&recipe).expect("recipe must serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON must be valid");

    assert_eq!(
        parsed["schemaVersion"],
        CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION
    );
    assert_eq!(parsed["recipeType"], "currentAppCommand");
    assert_eq!(parsed["effectiveQuery"], "close duplicate tabs");
    assert_eq!(
        parsed["suggestedScriptName"],
        "Safari Close Duplicate Tabs"
    );
    assert_eq!(parsed["trace"]["action"], "generate_script");
}
