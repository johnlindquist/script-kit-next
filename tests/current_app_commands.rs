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
    build_generate_script_prompt_from_snapshot, resolve_do_in_current_app_intent,
    DoInCurrentAppAction, FrontmostMenuSnapshot,
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
