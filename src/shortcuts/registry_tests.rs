//! Tests for ShortcutRegistry

use super::context::ShortcutContext;
use super::registry::{
    BindingSource, ConflictType, ShortcutBinding, ShortcutCategory, ShortcutRegistry,
};
use super::types::{Modifiers, Shortcut};

fn make_shortcut(key: &str, cmd: bool, shift: bool) -> Shortcut {
    Shortcut {
        key: key.to_string(),
        modifiers: Modifiers {
            cmd,
            shift,
            ..Default::default()
        },
    }
}

#[test]
fn register_and_get() {
    let mut registry = ShortcutRegistry::new();
    let binding = ShortcutBinding::builtin(
        "test.action",
        "Test Action",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    );
    registry.register(binding);

    let retrieved = registry.get("test.action").unwrap();
    assert_eq!(retrieved.name, "Test Action");
    assert_eq!(retrieved.default_shortcut.key, "k");
}

#[test]
fn registration_order_preserved() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "first",
        "First",
        make_shortcut("a", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "second",
        "Second",
        make_shortcut("b", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "third",
        "Third",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let bindings = registry.bindings();
    assert_eq!(bindings[0].id, "first");
    assert_eq!(bindings[1].id, "second");
    assert_eq!(bindings[2].id, "third");
}

#[test]
fn user_override_takes_precedence() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "test.action",
        "Test",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let shortcut = registry.get_shortcut("test.action").unwrap();
    assert_eq!(shortcut.key, "k");

    registry.set_override("test.action", Some(make_shortcut("j", true, true)));

    let shortcut = registry.get_shortcut("test.action").unwrap();
    assert_eq!(shortcut.key, "j");
    assert!(shortcut.modifiers.shift);
}

#[test]
fn disable_via_override() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "test.action",
        "Test",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    registry.set_override("test.action", None);

    assert!(registry.is_disabled("test.action"));
    assert!(registry.get_shortcut("test.action").is_none());
}

#[test]
fn clear_override_reverts_to_default() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "test.action",
        "Test",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    registry.set_override("test.action", Some(make_shortcut("j", true, true)));
    registry.clear_override("test.action");

    let shortcut = registry.get_shortcut("test.action").unwrap();
    assert_eq!(shortcut.key, "k");
}

#[test]
fn find_match_respects_context_order() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "editor.enter",
        "Editor Enter",
        make_shortcut("enter", false, false),
        ShortcutContext::Editor,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "global.enter",
        "Global Enter",
        make_shortcut("enter", false, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let keystroke = gpui::Keystroke {
        key: "enter".to_string(),
        key_char: None,
        modifiers: gpui::Modifiers::default(),
    };

    // Editor context first - should match editor binding
    let contexts = [ShortcutContext::Editor, ShortcutContext::Global];
    assert_eq!(
        registry.find_match(&keystroke, &contexts),
        Some("editor.enter")
    );

    // Global only - should match global binding
    let contexts = [ShortcutContext::Global];
    assert_eq!(
        registry.find_match(&keystroke, &contexts),
        Some("global.enter")
    );
}

#[test]
fn find_match_skips_disabled() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "first",
        "First",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "second",
        "Second",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    registry.set_override("first", None);

    let keystroke = gpui::Keystroke {
        key: "k".to_string(),
        key_char: None,
        modifiers: gpui::Modifiers {
            platform: true,
            ..Default::default()
        },
    };

    let contexts = [ShortcutContext::Global];
    assert_eq!(registry.find_match(&keystroke, &contexts), Some("second"));
}

#[test]
fn bindings_by_category() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "nav.up",
        "Move Up",
        make_shortcut("up", false, false),
        ShortcutContext::Global,
        ShortcutCategory::Navigation,
    ));
    registry.register(ShortcutBinding::builtin(
        "action.submit",
        "Submit",
        make_shortcut("enter", false, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let nav_bindings = registry.bindings_by_category(ShortcutCategory::Navigation);
    assert_eq!(nav_bindings.len(), 1);
    assert_eq!(nav_bindings[0].id, "nav.up");
}

#[test]
fn replace_existing_binding() {
    let mut registry = ShortcutRegistry::new();
    registry.register(ShortcutBinding::builtin(
        "test.action",
        "Original",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "test.action",
        "Replaced",
        make_shortcut("j", true, true),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    assert_eq!(registry.bindings().len(), 1);
    let binding = registry.get("test.action").unwrap();
    assert_eq!(binding.name, "Replaced");
    assert_eq!(binding.default_shortcut.key, "j");
}

// ========================
// Priority Model Tests
// ========================

#[test]
fn builtin_wins_over_script_same_shortcut() {
    let mut registry = ShortcutRegistry::new();

    // Register script first (lower priority)
    registry.register(ShortcutBinding::script(
        "script.action",
        "Script Action",
        make_shortcut("k", true, false),
    ));

    // Register builtin second (higher priority)
    registry.register(ShortcutBinding::builtin(
        "builtin.action",
        "Builtin Action",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let keystroke = gpui::Keystroke {
        key: "k".to_string(),
        key_char: None,
        modifiers: gpui::Modifiers {
            platform: true,
            ..Default::default()
        },
    };

    let contexts = [ShortcutContext::Global];
    // Builtin should win even though script was registered first
    assert_eq!(
        registry.find_match(&keystroke, &contexts),
        Some("builtin.action")
    );
}

#[test]
fn user_override_wins_over_builtin() {
    let mut registry = ShortcutRegistry::new();

    // Register builtin
    registry.register(ShortcutBinding::builtin(
        "builtin.action",
        "Builtin Action",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    // Register script with different default shortcut
    registry.register(ShortcutBinding::script(
        "script.action",
        "Script Action",
        make_shortcut("j", true, false),
    ));

    // User overrides script to use same shortcut as builtin
    registry.set_override("script.action", Some(make_shortcut("k", true, false)));

    let keystroke = gpui::Keystroke {
        key: "k".to_string(),
        key_char: None,
        modifiers: gpui::Modifiers {
            platform: true,
            ..Default::default()
        },
    };

    let contexts = [ShortcutContext::Global];
    // User override on script should win over builtin
    assert_eq!(
        registry.find_match(&keystroke, &contexts),
        Some("script.action")
    );
}

#[test]
fn check_builtin_conflict_detects_collision() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.copy",
        "Copy",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Edit,
    ));

    // Check if a script trying to use cmd+c would conflict
    let script_shortcut = make_shortcut("c", true, false);
    let conflict = registry.check_builtin_conflict(&script_shortcut, ShortcutContext::Global);

    assert_eq!(conflict, Some("builtin.copy"));
}

#[test]
fn check_builtin_conflict_no_collision_different_shortcut() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.copy",
        "Copy",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Edit,
    ));

    // Script using different shortcut should not conflict
    let script_shortcut = make_shortcut("k", true, false);
    let conflict = registry.check_builtin_conflict(&script_shortcut, ShortcutContext::Global);

    assert!(conflict.is_none());
}

#[test]
fn check_builtin_conflict_respects_user_override() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.copy",
        "Copy",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Edit,
    ));

    // User overrides builtin to different shortcut
    registry.set_override("builtin.copy", Some(make_shortcut("x", true, false)));

    // Now cmd+c should not conflict (builtin was moved)
    let script_shortcut = make_shortcut("c", true, false);
    let conflict = registry.check_builtin_conflict(&script_shortcut, ShortcutContext::Global);

    assert!(conflict.is_none());

    // But cmd+x should now conflict (builtin's new shortcut)
    let script_shortcut_x = make_shortcut("x", true, false);
    let conflict_x = registry.check_builtin_conflict(&script_shortcut_x, ShortcutContext::Global);

    assert_eq!(conflict_x, Some("builtin.copy"));
}

#[test]
fn check_builtin_conflict_ignores_disabled() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.copy",
        "Copy",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Edit,
    ));

    // User disables the builtin
    registry.set_override("builtin.copy", None);

    // Now cmd+c should not conflict (builtin is disabled)
    let script_shortcut = make_shortcut("c", true, false);
    let conflict = registry.check_builtin_conflict(&script_shortcut, ShortcutContext::Global);

    assert!(conflict.is_none());
}

#[test]
fn binding_source_priority_values() {
    // Verify priority order: lower value = higher priority
    assert!(BindingSource::Builtin.priority() < BindingSource::Script.priority());
}

// ========================
// Conflict Detection Tests
// ========================

#[test]
fn find_conflicts_detects_hard_conflict() {
    let mut registry = ShortcutRegistry::new();

    // Two builtins with same shortcut + same context = hard conflict
    registry.register(ShortcutBinding::builtin(
        "builtin.a",
        "Action A",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "builtin.b",
        "Action B",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let conflicts = registry.find_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::Hard);
    assert_eq!(conflicts[0].shortcut, "cmd+k");
}

#[test]
fn find_conflicts_detects_shadowed_conflict() {
    let mut registry = ShortcutRegistry::new();

    // Builtin shadows script with same shortcut
    registry.register(ShortcutBinding::builtin(
        "builtin.action",
        "Builtin Action",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::script(
        "script.action",
        "Script Action",
        make_shortcut("k", true, false),
    ));

    let conflicts = registry.find_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::Shadowed);
    assert_eq!(conflicts[0].winner_id, "builtin.action");
    assert_eq!(conflicts[0].loser_id, "script.action");
}

#[test]
fn find_conflicts_detects_os_reserved() {
    let mut registry = ShortcutRegistry::new();

    // Cmd+Tab is OS reserved on macOS
    registry.register(ShortcutBinding::builtin(
        "app.switcher",
        "App Switcher",
        make_shortcut("tab", true, false),
        ShortcutContext::Global,
        ShortcutCategory::System,
    ));

    let conflicts = registry.find_conflicts();

    // Should have unreachable conflict on macOS
    #[cfg(target_os = "macos")]
    {
        assert!(conflicts
            .iter()
            .any(|c| c.conflict_type == ConflictType::Unreachable));
    }
}

#[test]
fn find_conflicts_ignores_disabled() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.a",
        "Action A",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::builtin(
        "builtin.b",
        "Action B",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    // Disable one binding
    registry.set_override("builtin.a", None);

    let conflicts = registry.find_conflicts();
    // No conflict when one is disabled
    assert!(conflicts
        .iter()
        .all(|c| c.winner_id != "builtin.a" && c.loser_id != "builtin.a"));
}

#[test]
fn conflicts_for_returns_specific_binding_conflicts() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.a",
        "Action A",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));
    registry.register(ShortcutBinding::script(
        "script.a",
        "Script A",
        make_shortcut("k", true, false),
    ));
    registry.register(ShortcutBinding::builtin(
        "builtin.b",
        "Action B",
        make_shortcut("j", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let conflicts = registry.conflicts_for("script.a");
    assert_eq!(conflicts.len(), 1);
    assert!(conflicts[0].loser_id == "script.a" || conflicts[0].winner_id == "script.a");
}

#[test]
fn would_conflict_detects_existing_shortcut() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.copy",
        "Copy",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Edit,
    ));

    let new_shortcut = make_shortcut("c", true, false);
    let conflicts = registry.would_conflict(
        &new_shortcut,
        ShortcutContext::Global,
        BindingSource::Script,
    );

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].existing_id, "builtin.copy");
}

#[test]
fn would_conflict_no_conflict_different_shortcut() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.copy",
        "Copy",
        make_shortcut("c", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Edit,
    ));

    let new_shortcut = make_shortcut("k", true, false);
    let conflicts = registry.would_conflict(
        &new_shortcut,
        ShortcutContext::Global,
        BindingSource::Script,
    );

    assert!(conflicts.is_empty());
}

#[test]
fn would_conflict_detects_os_reserved() {
    let registry = ShortcutRegistry::new();

    let reserved_shortcut = make_shortcut("tab", true, false); // Cmd+Tab
    let conflicts = registry.would_conflict(
        &reserved_shortcut,
        ShortcutContext::Global,
        BindingSource::Builtin,
    );

    #[cfg(target_os = "macos")]
    {
        assert!(conflicts
            .iter()
            .any(|c| c.conflict_type == ConflictType::Unreachable));
    }
}

#[test]
fn would_conflict_same_priority_is_hard() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.action",
        "Action",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let new_shortcut = make_shortcut("k", true, false);
    let conflicts = registry.would_conflict(
        &new_shortcut,
        ShortcutContext::Global,
        BindingSource::Builtin, // Same priority as existing
    );

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::Hard);
}

#[test]
fn would_conflict_different_priority_is_shadowed() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutBinding::builtin(
        "builtin.action",
        "Action",
        make_shortcut("k", true, false),
        ShortcutContext::Global,
        ShortcutCategory::Actions,
    ));

    let new_shortcut = make_shortcut("k", true, false);
    let conflicts = registry.would_conflict(
        &new_shortcut,
        ShortcutContext::Global,
        BindingSource::Script, // Lower priority than existing builtin
    );

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::Shadowed);
}
