// ============================================
// NAME METADATA PARSING TESTS
// ============================================

#[test]
fn test_parse_metadata_line_name_basic() {
    // Basic case: "// Name: Test"
    let line = "// Name: Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_no_space_after_slashes() {
    // "//Name:Test" - no spaces
    let line = "//Name:Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_space_after_colon() {
    // "//Name: Test" - space after colon
    let line = "//Name: Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_space_before_key() {
    // "// Name:Test" - space before key
    let line = "// Name:Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_full_spacing() {
    // "// Name: Test" - standard spacing
    let line = "// Name: Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_multiple_spaces() {
    // "//  Name:Test" - multiple spaces after slashes
    let line = "//  Name:Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_multiple_spaces_and_colon_space() {
    // "//  Name: Test" - multiple spaces after slashes and space after colon
    let line = "//  Name: Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_with_tab() {
    // "//\tName:Test" - tab after slashes
    let line = "//\tName:Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_name_with_tab_and_space_after_colon() {
    // "//\tName: Test" - tab after slashes, space after colon
    let line = "//\tName: Test";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "name");
    assert_eq!(value, "Test");
}

#[test]
fn test_parse_metadata_line_case_insensitive_name() {
    // Case insensitivity: "// name: Test", "// NAME: Test"
    for line in ["// name: Test", "// NAME: Test", "// NaMe: Test"] {
        let result = parse_metadata_line(line);
        assert!(result.is_some(), "Failed for: {}", line);
        let (key, value) = result.unwrap();
        assert_eq!(key.to_lowercase(), "name");
        assert_eq!(value, "Test");
    }
}

#[test]
fn test_parse_metadata_line_description() {
    // Should also work for Description
    let line = "// Description: My script description";
    let result = parse_metadata_line(line);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key.to_lowercase(), "description");
    assert_eq!(value, "My script description");
}

#[test]
fn test_parse_metadata_line_not_a_comment() {
    // Non-comment lines should return None
    let line = "const name = 'test';";
    let result = parse_metadata_line(line);
    assert!(result.is_none());
}

#[test]
fn test_parse_metadata_line_no_colon() {
    // Comment without colon should return None
    let line = "// Just a comment";
    let result = parse_metadata_line(line);
    assert!(result.is_none());
}

#[test]
fn test_extract_script_metadata_name_and_description() {
    let content = r#"// Name: My Script Name
// Description: This is my script
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("My Script Name".to_string()));
    assert_eq!(metadata.description, Some("This is my script".to_string()));
}

#[test]
fn test_extract_script_metadata_with_alias() {
    let content = r#"// Name: Git Commit
// Description: Commit changes to git
// Alias: gc
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("Git Commit".to_string()));
    assert_eq!(
        metadata.description,
        Some("Commit changes to git".to_string())
    );
    assert_eq!(metadata.alias, Some("gc".to_string()));
}

#[test]
fn test_extract_script_metadata_alias_only() {
    let content = r#"// Alias: shortcut
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, None);
    assert_eq!(metadata.alias, Some("shortcut".to_string()));
}

#[test]
fn test_extract_script_metadata_name_only() {
    let content = r#"// Name: My Script
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, Some("My Script".to_string()));
    assert_eq!(metadata.description, None);
}

#[test]
fn test_extract_script_metadata_description_only() {
    let content = r#"// Description: A description
const x = 1;
"#;
    let metadata = extract_script_metadata(content);
    assert_eq!(metadata.name, None);
    assert_eq!(metadata.description, Some("A description".to_string()));
}

