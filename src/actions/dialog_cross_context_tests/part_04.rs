
#[test]
fn deeplink_name_with_numbers() {
    assert_eq!(to_deeplink_name("script123"), "script123");
}

#[test]
fn deeplink_name_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

#[test]
fn deeplink_name_only_special_chars() {
    assert_eq!(to_deeplink_name("@#$%^&"), "");
}

#[test]
fn deeplink_name_preserves_hyphens_as_single() {
    assert_eq!(to_deeplink_name("a---b"), "a-b");
}

// ============================================================================
// count_section_headers edge cases
// ============================================================================

#[test]
fn count_section_headers_with_none_sections_among_some() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Sec1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext), // no section
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Sec1"),
        Action::new("d", "D", None, ActionCategory::ScriptContext).with_section("Sec2"),
    ];
    let filtered = vec![0, 1, 2, 3];
    let count = count_section_headers(&actions, &filtered);
    // idx 0: Sec1 → header (first section)
    // idx 1: None → no header (section is None, only Some sections count)
    // idx 2: Sec1 → header (prev was None, section changed from None to Some(Sec1))
    // idx 3: Sec2 → header (section changed from Sec1 to Sec2)
    assert_eq!(count, 3);
}

#[test]
fn count_section_headers_all_same_section() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Same"),
    ];
    let filtered = vec![0, 1, 2];
    let count = count_section_headers(&actions, &filtered);
    assert_eq!(count, 1); // Only one header for the single section
}

#[test]
fn count_section_headers_filtered_subset_skips_some() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("S3"),
    ];
    // Only show actions 0 and 2 (skip S2)
    let filtered = vec![0, 2];
    let count = count_section_headers(&actions, &filtered);
    assert_eq!(count, 2); // S1 and S3
}

// ============================================================================
// File context with different FileType variants
// ============================================================================

#[test]
fn file_context_application_treated_as_file() {
    let info = FileInfo {
        path: "/Applications/Safari.app".into(),
        name: "Safari.app".into(),
        file_type: FileType::Application,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert_eq!(actions[0].id, "open_file");
    assert_eq!(actions[0].title, "Open \"Safari.app\"");
}

#[test]
fn file_context_image_treated_as_file() {
    let info = FileInfo {
        path: "/photos/sunset.jpg".into(),
        name: "sunset.jpg".into(),
        file_type: FileType::Image,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert_eq!(actions[0].id, "open_file");
    #[cfg(target_os = "macos")]
    {
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"quick_look"));
    }
}

// ============================================================================
// Global actions
// ============================================================================

#[test]
fn global_actions_is_always_empty() {
    let actions = get_global_actions();
    assert!(actions.is_empty());
}

// ============================================================================
// Enum default values
// ============================================================================

#[test]
fn search_position_default_is_bottom() {
    assert!(matches!(SearchPosition::default(), SearchPosition::Bottom));
}

#[test]
fn section_style_default_is_separators() {
    assert!(matches!(SectionStyle::default(), SectionStyle::Separators));
}

#[test]
fn anchor_position_default_is_bottom() {
    assert!(matches!(AnchorPosition::default(), AnchorPosition::Bottom));
}

#[test]
fn actions_dialog_config_default_values() {
    let config = ActionsDialogConfig::default();
    assert!(matches!(config.search_position, SearchPosition::Bottom));
    assert!(matches!(config.section_style, SectionStyle::Separators));
    assert!(matches!(config.anchor, AnchorPosition::Bottom));
    assert!(!config.show_icons);
    assert!(!config.show_footer);
}
