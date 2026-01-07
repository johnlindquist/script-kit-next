//! Tests for ShortcutRegistry

use super::context::ShortcutContext;
use super::registry::*;
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
