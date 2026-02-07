
// =========================================================================
// 28. Deeplink name — preserves unicode alphanumerics
// =========================================================================

#[test]
fn deeplink_name_preserves_numbers_in_script_name() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

#[test]
fn deeplink_name_handles_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

#[test]
fn deeplink_name_handles_all_whitespace() {
    assert_eq!(to_deeplink_name("   "), "");
}

// =========================================================================
// 29. Notes — minimum actions for every permutation
// =========================================================================

#[test]
fn notes_all_eight_permutations_have_at_least_two_actions() {
    for sel in [false, true] {
        for trash in [false, true] {
            for auto in [false, true] {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let count = get_notes_command_bar_actions(&info).len();
                assert!(
                    count >= 2,
                    "Notes permutation sel={}, trash={}, auto={} has only {} actions",
                    sel,
                    trash,
                    auto,
                    count
                );
            }
        }
    }
}

// =========================================================================
// 30. Action with_section chains correctly
// =========================================================================

#[test]
fn action_with_section_chains_preserve_other_fields() {
    let action = Action::new(
        "test",
        "Test Action",
        Some("A description".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Star)
    .with_section("My Section");

    assert_eq!(action.id, "test");
    assert_eq!(action.title, "Test Action");
    assert_eq!(action.description, Some("A description".into()));
    assert_eq!(action.shortcut, Some("⌘T".into()));
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section, Some("My Section".into()));
    assert!(!action.has_action);
    assert!(action.value.is_none());
}
