// ============================================
// SHORTCUT METADATA PARSING TESTS
// ============================================

#[test]
fn test_extract_script_metadata_with_shortcut() {
    let content = r#"// Name: Quick Action
// Description: Run a quick action
// Shortcut: opt i
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("Quick Action".to_string()));
    assert_eq!(metadata.description, Some("Run a quick action".to_string()));
    assert_eq!(metadata.shortcut, Some("opt i".to_string()));
}

#[test]
fn test_extract_script_metadata_shortcut_with_modifiers() {
    let content = r#"// Shortcut: cmd shift k
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.shortcut, Some("cmd shift k".to_string()));
}

#[test]
fn test_extract_script_metadata_shortcut_ctrl_alt() {
    let content = r#"// Shortcut: ctrl alt t
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.shortcut, Some("ctrl alt t".to_string()));
}

#[test]
fn test_extract_script_metadata_shortcut_only() {
    let content = r#"// Shortcut: opt space
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, None);
    assert_eq!(metadata.alias, None);
    assert_eq!(metadata.shortcut, Some("opt space".to_string()));
}

#[test]
fn test_extract_script_metadata_shortcut_with_alias() {
    let content = r#"// Name: Git Status
// Alias: gs
// Shortcut: cmd g
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("Git Status".to_string()));
    assert_eq!(metadata.alias, Some("gs".to_string()));
    assert_eq!(metadata.shortcut, Some("cmd g".to_string()));
}

#[test]
fn test_extract_script_metadata_shortcut_case_insensitive() {
    // Shortcut key should be case-insensitive (SHORTCUT, Shortcut, shortcut)
    for variant in [
        "// Shortcut: opt x",
        "// shortcut: opt x",
        "// SHORTCUT: opt x",
    ] {
        let content = format!("{}\nconst x = 1;", variant);
        let metadata = extract_script_metadata(&content);
        assert_eq!(
            metadata.shortcut,
            Some("opt x".to_string()),
            "Failed for variant: {}",
            variant
        );
    }
}

#[test]
fn test_extract_script_metadata_shortcut_lenient_whitespace() {
    // Test lenient whitespace handling like other metadata fields
    let variants = [
        "//Shortcut:opt j",
        "//Shortcut: opt j",
        "// Shortcut:opt j",
        "// Shortcut: opt j",
        "//  Shortcut: opt j",
    ];

    for variant in variants {
        let content = format!("{}\nconst x = 1;", variant);
        let metadata = extract_script_metadata(&content);
        assert_eq!(
            metadata.shortcut,
            Some("opt j".to_string()),
            "Failed for variant: {}",
            variant
        );
    }
}

#[test]
fn test_extract_script_metadata_shortcut_empty_ignored() {
    // Empty shortcut value should be ignored
    let content = r#"// Shortcut:
// Name: Has a name
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.shortcut, None);
    assert_eq!(metadata.name, Some("Has a name".to_string()));
}

#[test]
fn test_extract_script_metadata_first_shortcut_wins() {
    // If multiple Shortcut: lines exist, the first one wins
    let content = r#"// Shortcut: first shortcut
// Shortcut: second shortcut
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.shortcut, Some("first shortcut".to_string()));
}

#[test]
fn test_extract_script_metadata_all_fields() {
    // Test all metadata fields together
    let content = r#"// Name: Complete Script
// Description: A complete script with all metadata
// Icon: Terminal
// Alias: cs
// Shortcut: cmd shift c
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("Complete Script".to_string()));
    assert_eq!(
        metadata.description,
        Some("A complete script with all metadata".to_string())
    );
    assert_eq!(metadata.icon, Some("Terminal".to_string()));
    assert_eq!(metadata.alias, Some("cs".to_string()));
    assert_eq!(metadata.shortcut, Some("cmd shift c".to_string()));
}

#[test]
fn test_extract_script_metadata_no_metadata() {
    let content = r#"const x = 1;
console.log(x);
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, None);
    assert_eq!(metadata.description, None);
}

#[test]
fn test_extract_script_metadata_lenient_whitespace() {
    // Test all the lenient whitespace variants for Name
    let variants = [
        "//Name:Test",
        "//Name: Test",
        "// Name:Test",
        "// Name: Test",
        "//  Name:Test",
        "//  Name: Test",
        "//\tName:Test",
        "//\tName: Test",
    ];

    for content in variants {
        let full_content = format!("{}\nconst x = 1;", content);
        let metadata = extract_script_metadata(&full_content);
        assert_eq!(
            metadata.name,
            Some("Test".to_string()),
            "Failed for variant: {}",
            content
        );
    }
}

#[test]
fn test_extract_script_metadata_first_name_wins() {
    // If multiple Name: lines exist, the first one wins
    let content = r#"// Name: First Name
// Name: Second Name
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("First Name".to_string()));
}

#[test]
fn test_extract_script_metadata_empty_value_ignored() {
    // Empty value should be ignored
    let content = r#"// Name:
// Description: Has a description
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, None);
    assert_eq!(metadata.description, Some("Has a description".to_string()));
}

#[test]
fn test_parse_metadata_line_value_with_colons() {
    // Value can contain colons (e.g., URLs)
    let line = "// Description: Visit https://example.com for more info";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "description");
    assert_eq!(value, "Visit https://example.com for more info");
}

#[test]
fn test_parse_metadata_line_value_with_leading_trailing_spaces() {
    // Value should be trimmed
    let line = "// Name:   Padded Value   ";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (_, value) = result.unwrap();
    assert_eq!(value, "Padded Value");
}

#[test]
fn test_extract_script_metadata_only_first_20_lines() {
    // Metadata after line 20 should be ignored
    let mut content = String::new();
    for i in 1..=25 {
        if i == 22 {
            content.push_str("// Name: Too Late\n");
        } else {
            content.push_str(&format!("// Comment line {}\n", i));
        }
    }
    let metadata = extract_script_metadata(&content);
    assert_eq!(metadata.name, None);
}

#[test]
fn test_extract_script_metadata_within_first_20_lines() {
    // Metadata within first 20 lines should be captured
    let mut content = String::new();
    for i in 1..=25 {
        if i == 15 {
            content.push_str("// Name: Just In Time\n");
        } else {
            content.push_str(&format!("// Comment line {}\n", i));
        }
    }
    let metadata = extract_script_metadata(&content);
    assert_eq!(metadata.name, Some("Just In Time".to_string()));
}

