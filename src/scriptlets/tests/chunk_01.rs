use super::*;

// ========================================
// Type and Constant Tests
// ========================================

#[test]
fn test_valid_tools_contains_common_tools() {
    assert!(VALID_TOOLS.contains(&"bash"));
    assert!(VALID_TOOLS.contains(&"python"));
    assert!(VALID_TOOLS.contains(&"ts"));
    assert!(VALID_TOOLS.contains(&"js"));
    assert!(VALID_TOOLS.contains(&"kit"));
    assert!(VALID_TOOLS.contains(&"paste"));
    assert!(VALID_TOOLS.contains(&"template"));
}

#[test]
fn test_shell_tools_contains_shells() {
    assert!(SHELL_TOOLS.contains(&"bash"));
    assert!(SHELL_TOOLS.contains(&"zsh"));
    assert!(SHELL_TOOLS.contains(&"sh"));
    assert!(SHELL_TOOLS.contains(&"fish"));
    assert!(SHELL_TOOLS.contains(&"powershell"));
}

#[test]
fn test_scriptlet_new_basic() {
    let scriptlet = Scriptlet::new(
        "My Test Script".to_string(),
        "bash".to_string(),
        "echo hello".to_string(),
    );

    assert_eq!(scriptlet.name, "My Test Script");
    assert_eq!(scriptlet.command, "my-test-script");
    assert_eq!(scriptlet.tool, "bash");
    assert_eq!(scriptlet.scriptlet_content, "echo hello");
    assert!(scriptlet.inputs.is_empty());
}

#[test]
fn test_scriptlet_new_with_inputs() {
    let scriptlet = Scriptlet::new(
        "Test".to_string(),
        "ts".to_string(),
        "const name = '{{name}}'; const age = {{age}};".to_string(),
    );

    assert_eq!(scriptlet.inputs.len(), 2);
    assert!(scriptlet.inputs.contains(&"name".to_string()));
    assert!(scriptlet.inputs.contains(&"age".to_string()));
}

#[test]
fn test_scriptlet_is_shell() {
    let bash = Scriptlet::new("test".to_string(), "bash".to_string(), "echo".to_string());
    let ts = Scriptlet::new(
        "test".to_string(),
        "ts".to_string(),
        "console.log()".to_string(),
    );

    assert!(bash.is_shell());
    assert!(!ts.is_shell());
}

#[test]
fn test_scriptlet_is_valid_tool() {
    let valid = Scriptlet::new("test".to_string(), "bash".to_string(), "echo".to_string());
    let invalid = Scriptlet::new(
        "test".to_string(),
        "invalid_tool".to_string(),
        "echo".to_string(),
    );

    assert!(valid.is_valid_tool());
    assert!(!invalid.is_valid_tool());
}

// ========================================
// Slugify Tests
// ========================================

#[test]
fn test_slugify_basic() {
    assert_eq!(slugify("Hello World"), "hello-world");
    assert_eq!(slugify("My Script"), "my-script");
}

#[test]
fn test_slugify_special_chars() {
    assert_eq!(slugify("Hello, World!"), "hello-world");
    assert_eq!(slugify("test@123"), "test-123");
}

#[test]
fn test_slugify_multiple_spaces() {
    assert_eq!(slugify("Hello   World"), "hello-world");
    assert_eq!(slugify("  Leading Trailing  "), "leading-trailing");
}

// ========================================
// Extract Named Inputs Tests
// ========================================

#[test]
fn test_extract_named_inputs_basic() {
    let inputs = extract_named_inputs("Hello {{name}}!");
    assert_eq!(inputs, vec!["name"]);
}

#[test]
fn test_extract_named_inputs_multiple() {
    let inputs = extract_named_inputs("{{first}} and {{second}}");
    assert_eq!(inputs.len(), 2);
    assert!(inputs.contains(&"first".to_string()));
    assert!(inputs.contains(&"second".to_string()));
}

#[test]
fn test_extract_named_inputs_no_duplicates() {
    let inputs = extract_named_inputs("{{name}} is {{name}}");
    assert_eq!(inputs, vec!["name"]);
}

#[test]
fn test_extract_named_inputs_ignores_conditionals() {
    let inputs = extract_named_inputs("{{#if flag}}{{name}}{{/if}}");
    assert_eq!(inputs, vec!["name"]);
    assert!(!inputs.contains(&"#if flag".to_string()));
    assert!(!inputs.contains(&"/if".to_string()));
}

#[test]
fn test_extract_named_inputs_empty() {
    let inputs = extract_named_inputs("No placeholders here");
    assert!(inputs.is_empty());
}

// ========================================
// Metadata Parsing Tests
// ========================================

#[test]
fn test_parse_metadata_basic() {
    let metadata = parse_html_comment_metadata("<!-- shortcut: cmd k -->");
    assert_eq!(metadata.shortcut, Some("cmd k".to_string()));
}

#[test]
fn test_parse_metadata_multiple_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nshortcut: cmd k\ndescription: My script\ntrigger: test\n-->",
    );
    assert_eq!(metadata.shortcut, Some("cmd k".to_string()));
    assert_eq!(metadata.description, Some("My script".to_string()));
    assert_eq!(metadata.trigger, Some("test".to_string()));
}

#[test]
fn test_parse_metadata_background_bool() {
    let metadata = parse_html_comment_metadata("<!-- background: true -->");
    assert_eq!(metadata.background, Some(true));

    let metadata = parse_html_comment_metadata("<!-- background: false -->");
    assert_eq!(metadata.background, Some(false));
}

#[test]
fn test_parse_metadata_extra_fields() {
    let metadata = parse_html_comment_metadata("<!-- custom_field: value -->");
    assert_eq!(
        metadata.extra.get("custom_field"),
        Some(&"value".to_string())
    );
}

#[test]
fn test_parse_metadata_empty() {
    let metadata = parse_html_comment_metadata("No comments here");
    assert!(metadata.shortcut.is_none());
    assert!(metadata.description.is_none());
}

#[test]
fn test_parse_metadata_colons_in_value() {
    let metadata =
        parse_html_comment_metadata("<!-- description: Visit https://example.com for info -->");
    assert_eq!(
        metadata.description,
        Some("Visit https://example.com for info".to_string())
    );
}

// ========================================
// Expand Metadata Tests
// ========================================

#[test]
fn test_parse_metadata_keyword_basic() {
    let metadata = parse_html_comment_metadata("<!-- keyword: :sig -->");
    assert_eq!(metadata.keyword, Some(":sig".to_string()));
}

#[test]
fn test_parse_metadata_keyword_with_punctuation() {
    let metadata = parse_html_comment_metadata("<!-- keyword: !email -->");
    assert_eq!(metadata.keyword, Some("!email".to_string()));
}

#[test]
fn test_parse_metadata_keyword_with_double_suffix() {
    // Common pattern: keyword followed by double char like "ddate,,"
    let metadata = parse_html_comment_metadata("<!-- keyword: ddate,, -->");
    assert_eq!(metadata.keyword, Some("ddate,,".to_string()));
}

#[test]
fn test_parse_metadata_keyword_with_other_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nkeyword: :addr\nshortcut: cmd e\ndescription: Insert address\n-->",
    );
    assert_eq!(metadata.keyword, Some(":addr".to_string()));
    assert_eq!(metadata.shortcut, Some("cmd e".to_string()));
    assert_eq!(metadata.description, Some("Insert address".to_string()));
}

#[test]
fn test_parse_metadata_keyword_empty_value() {
    // Empty expand value should not be stored
    let metadata = parse_html_comment_metadata("<!-- keyword: -->");
    assert_eq!(metadata.keyword, None);
}

#[test]
fn test_parse_markdown_scriptlet_with_keyword() {
    let markdown = r#"## Email Signature

<!-- keyword: :sig -->

```type
Best regards,
John Doe
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Email Signature");
    assert_eq!(scriptlets[0].metadata.keyword, Some(":sig".to_string()));
    assert_eq!(scriptlets[0].tool, "type");
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_keyword() {
    let markdown = r#"# Snippets

## Date Insert

<!-- keyword: :date -->

```type
{{date}}
```

## Email Template

<!-- keyword: !email -->

```type
Hello {{name}},
```

## No Expand

```type
Plain text
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 3);

    assert_eq!(scriptlets[0].metadata.keyword, Some(":date".to_string()));
    assert_eq!(scriptlets[1].metadata.keyword, Some("!email".to_string()));
    assert_eq!(scriptlets[2].metadata.keyword, None);
}

#[test]
fn test_keyword_metadata_serialization() {
    let metadata = ScriptletMetadata {
        keyword: Some(":test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"keyword\":\":test\""));

    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.keyword, Some(":test".to_string()));
}

#[test]
fn test_keyword_metadata_deserialization_missing() {
    // When expand is not present in JSON, it should be None
    let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
    let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.keyword, None);
}

