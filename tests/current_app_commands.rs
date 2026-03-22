//! Integration tests for the "Current App Commands" built-in.
//!
//! These tests verify registration, snapshot contract, and entry shaping
//! without requiring a live frontmost app or Accessibility permission.

use script_kit_gpui::builtins::{
    self, BuiltInEntry, BuiltInFeature, BuiltInGroup, MenuBarActionInfo, UtilityCommandType,
};
use script_kit_gpui::config::BuiltInConfig;
use script_kit_gpui::menu_bar::current_app_commands::FrontmostMenuSnapshot;
use script_kit_gpui::menu_bar::{KeyboardShortcut, MenuBarItem, ModifierFlags};

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

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
