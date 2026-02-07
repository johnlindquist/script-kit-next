
#[test]
fn format_shortcut_single_letter() {
    let result = ActionsDialog::format_shortcut_hint("a");
    assert_eq!(result, "A");
}

#[test]
fn format_shortcut_triple_modifier() {
    let result = ActionsDialog::format_shortcut_hint("cmd+shift+alt+x");
    assert!(result.contains('⌘'));
    assert!(result.contains('⇧'));
    assert!(result.contains('⌥'));
    assert!(result.contains('X'));
}

// =========================================================================
// 15. parse_shortcut_keycaps — additional patterns
// =========================================================================

#[test]
fn parse_shortcut_keycaps_mixed_modifiers_and_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(caps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn parse_shortcut_keycaps_enter_only() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn parse_shortcut_keycaps_modifier_plus_arrow() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↑");
    assert_eq!(caps, vec!["⌘", "↑"]);
}

#[test]
fn parse_shortcut_keycaps_multi_char_key_like_f12() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘F12");
    // parse_shortcut_keycaps treats each non-modifier character individually:
    // ⌘ is a modifier (single keycap), then F, 1, 2 are each separate keycaps
    assert_eq!(caps.len(), 4);
    assert_eq!(caps[0], "⌘");
    assert_eq!(caps[1], "F");
    assert_eq!(caps[2], "1");
    assert_eq!(caps[3], "2");
}

// =========================================================================
// 16. to_deeplink_name — more edge cases
// =========================================================================

#[test]
fn to_deeplink_name_japanese_chars() {
    // Japanese characters are alphanumeric per Rust's is_alphanumeric()
    let result = to_deeplink_name("スクリプト");
    assert!(!result.is_empty());
}

#[test]
fn to_deeplink_name_mixed_scripts() {
    let result = to_deeplink_name("Hello 世界");
    assert!(result.contains("hello"));
    assert!(result.contains("世界"));
}

#[test]
fn to_deeplink_name_very_long_name() {
    let long = "a".repeat(200);
    let result = to_deeplink_name(&long);
    assert_eq!(result.len(), 200);
}

#[test]
fn to_deeplink_name_single_hyphen() {
    assert_eq!(to_deeplink_name("-"), "");
}

// =========================================================================
// 17. CommandBarConfig — field interactions
// =========================================================================

#[test]
fn command_bar_all_presets_exist() {
    // Verify all 5 preset constructors work without panicking
    let _ = CommandBarConfig::default();
    let _ = CommandBarConfig::main_menu_style();
    let _ = CommandBarConfig::ai_style();
    let _ = CommandBarConfig::notes_style();
    let _ = CommandBarConfig::no_search();
}

#[test]
fn command_bar_no_search_still_has_close_behaviors() {
    let config = CommandBarConfig::no_search();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn command_bar_ai_style_uses_headers() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
}

#[test]
fn command_bar_main_menu_uses_separators() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
}

// =========================================================================
// 18. Grouped items — interleaved headers and items
// =========================================================================

#[test]
fn grouped_items_alternating_sections() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S2")),
        make_action("c", "C", Some("S1")),
        make_action("d", "D", Some("S2")),
    ];
    let result = build_grouped_items_static(&actions, &[0, 1, 2, 3], SectionStyle::Headers);
    // Each section change introduces a new header
    let header_count = result
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 4, "Each section change should have a header");
}

#[test]
fn grouped_items_filtered_subset_only_shows_subset_sections() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S2")),
        make_action("c", "C", Some("S3")),
    ];
    // Only show S1 and S3
    let result = build_grouped_items_static(&actions, &[0, 2], SectionStyle::Headers);
    let headers: Vec<String> = result
        .iter()
        .filter_map(|i| match i {
            GroupedActionItem::SectionHeader(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(headers, vec!["S1", "S3"]);
}

// =========================================================================
// 19. Coerce selection — wrap-around and boundary
// =========================================================================

#[test]
fn coerce_single_item_always_selects_it() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    assert_eq!(coerce_action_selection(&rows, 100), Some(0));
}

#[test]
fn coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
        GroupedActionItem::SectionHeader("C".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
    assert_eq!(coerce_action_selection(&rows, 2), None);
}

// =========================================================================
// 20. Window position variants exhaustive test
// =========================================================================

#[test]
fn window_position_all_variants_have_unique_debug_repr() {
    let variants = [
        WindowPosition::BottomRight,
        WindowPosition::TopRight,
        WindowPosition::TopCenter,
    ];
    let debug_strs: Vec<String> = variants.iter().map(|v| format!("{:?}", v)).collect();
    // Each variant should have a distinct debug representation
    for (i, s) in debug_strs.iter().enumerate() {
        for (j, other) in debug_strs.iter().enumerate() {
            if i != j {
                assert_ne!(
                    s, other,
                    "Variants {} and {} should have distinct Debug repr",
                    i, j
                );
            }
        }
    }
}

// =========================================================================
// 21. ProtocolAction — edge case field combos
// =========================================================================

#[test]
fn protocol_action_all_fields_populated() {
    let action = ProtocolAction {
        name: "Full Action".into(),
        description: Some("Full description".into()),
        shortcut: Some("cmd+f".into()),
        value: Some("full-value".into()),
        has_action: true,
        visible: Some(true),
        close: Some(true),
    };
    assert!(action.is_visible());
    assert!(action.should_close());
    assert_eq!(action.name, "Full Action");
    assert_eq!(action.value.as_deref(), Some("full-value"));
}

#[test]
fn protocol_action_minimal_fields() {
    let action = ProtocolAction::new("Minimal".into());
    assert!(action.is_visible());
    assert!(action.should_close());
    assert!(action.description.is_none());
    assert!(action.shortcut.is_none());
    assert!(action.value.is_none());
    assert!(!action.has_action);
}

// =========================================================================
// 22. Action property invariants across all contexts
// =========================================================================

#[test]
fn all_actions_have_non_empty_ids() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in get_script_context_actions(&script) {
        assert!(!action.id.is_empty(), "Action has empty ID");
    }
    for action in get_ai_command_bar_actions() {
        assert!(!action.id.is_empty(), "AI action has empty ID");
    }
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in get_notes_command_bar_actions(&info) {
        assert!(!action.id.is_empty(), "Notes action has empty ID");
    }
}

#[test]
fn all_actions_have_non_empty_titles() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in get_script_context_actions(&script) {
        assert!(
            !action.title.is_empty(),
            "Action '{}' has empty title",
            action.id
        );
    }
    for action in get_ai_command_bar_actions() {
        assert!(
            !action.title.is_empty(),
            "AI action '{}' has empty title",
            action.id
        );
    }
}

#[test]
fn title_lower_always_matches_title_lowercased() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in get_script_context_actions(&script) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for '{}'",
            action.id
        );
    }
}

#[test]
fn description_lower_matches_description_lowercased() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in get_script_context_actions(&script) {
        match (&action.description, &action.description_lower) {
            (Some(desc), Some(lower)) => {
                assert_eq!(
                    *lower,
                    desc.to_lowercase(),
                    "description_lower mismatch for '{}'",
                    action.id
                );
            }
            (None, None) => {} // Both none is fine
            _ => panic!(
                "description/description_lower mismatch for '{}': {:?} vs {:?}",
                action.id, action.description, action.description_lower
            ),
        }
    }
}

// =========================================================================
// 23. Scriptlet with multiple custom actions — ordering and fields
// =========================================================================

#[test]
fn scriptlet_five_custom_actions_all_have_has_action() {
    let script = ScriptInfo::scriptlet("Multi", "/path/multi.md", None, None);
    let mut scriptlet = Scriptlet::new("Multi".into(), "bash".into(), "echo main".into());
    for i in 0..5 {
        scriptlet.actions.push(ScriptletAction {
            name: format!("Action {}", i),
            command: format!("action-{}", i),
            tool: "bash".into(),
            code: format!("echo {}", i),
            inputs: vec![],
            shortcut: if i % 2 == 0 {
                Some(format!("cmd+{}", i))
            } else {
                None
            },
            description: Some(format!("Does thing {}", i)),
        });
    }
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom: Vec<&Action> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .collect();
    assert_eq!(custom.len(), 5);
    for c in &custom {
        assert!(
            c.has_action,
            "Custom action '{}' should have has_action=true",
            c.id
        );
        assert!(
            c.value.is_some(),
            "Custom action '{}' should have value",
            c.id
        );
    }
    // Verify ordering: run_script first, then custom actions in order
    assert_eq!(actions[0].id, "run_script");
    for i in 0..5 {
        assert_eq!(actions[i + 1].id, format!("scriptlet_action:action-{}", i));
    }
}

// =========================================================================
// 24. count_section_headers matches grouped items header count
// =========================================================================

#[test]
fn section_header_count_matches_for_notes_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let from_count = count_section_headers(&actions, &filtered);
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let from_grouped = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(
        from_count, from_grouped,
        "count_section_headers should match actual headers"
    );
}

#[test]
fn section_header_count_matches_for_ai_actions() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let from_count = count_section_headers(&actions, &filtered);
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let from_grouped = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(from_count, from_grouped);
}

// =========================================================================
// 25. Enum variant completeness
// =========================================================================

#[test]
fn search_position_all_variants_distinct() {
    let variants = [
        SearchPosition::Bottom,
        SearchPosition::Top,
        SearchPosition::Hidden,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn section_style_all_variants_distinct() {
    let variants = [
        SectionStyle::None,
        SectionStyle::Separators,
        SectionStyle::Headers,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn anchor_position_all_variants_distinct() {
    let variants = [AnchorPosition::Bottom, AnchorPosition::Top];
    assert_ne!(variants[0], variants[1]);
}
