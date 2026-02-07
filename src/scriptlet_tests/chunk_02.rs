// ========================================
// Alias Metadata Tests
// ========================================

#[test]
fn test_parse_metadata_alias_basic() {
    let metadata = parse_html_comment_metadata("<!-- alias: goog -->");
    assert_eq!(metadata.alias, Some("goog".to_string()));
}

#[test]
fn test_parse_metadata_alias_with_punctuation() {
    let metadata = parse_html_comment_metadata("<!-- alias: g! -->");
    assert_eq!(metadata.alias, Some("g!".to_string()));
}

#[test]
fn test_parse_metadata_alias_with_numbers() {
    let metadata = parse_html_comment_metadata("<!-- alias: cmd123 -->");
    assert_eq!(metadata.alias, Some("cmd123".to_string()));
}

#[test]
fn test_parse_metadata_alias_with_other_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nalias: search\nshortcut: cmd s\ndescription: Search the web\n-->",
    );
    assert_eq!(metadata.alias, Some("search".to_string()));
    assert_eq!(metadata.shortcut, Some("cmd s".to_string()));
    assert_eq!(metadata.description, Some("Search the web".to_string()));
}

#[test]
fn test_parse_metadata_alias_empty_value() {
    // Empty alias value should not be stored
    let metadata = parse_html_comment_metadata("<!-- alias: -->");
    assert_eq!(metadata.alias, None);
}

#[test]
fn test_parse_markdown_scriptlet_with_alias() {
    let markdown = r#"## Google Search

<!-- alias: goog -->

```bash
open "https://www.google.com/search?q=$1"
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Google Search");
    assert_eq!(scriptlets[0].metadata.alias, Some("goog".to_string()));
    assert_eq!(scriptlets[0].tool, "bash");
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_alias() {
    let markdown = r#"# Launchers

## Google Search

<!-- alias: goog -->

```open
https://google.com
```

## GitHub

<!-- alias: gh -->

```open
https://github.com
```

## No Alias

```open
https://example.com
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 3);

    assert_eq!(scriptlets[0].metadata.alias, Some("goog".to_string()));
    assert_eq!(scriptlets[1].metadata.alias, Some("gh".to_string()));
    assert_eq!(scriptlets[2].metadata.alias, None);
}

#[test]
fn test_alias_metadata_serialization() {
    let metadata = ScriptletMetadata {
        alias: Some("test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"alias\":\"test\""));

    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.alias, Some("test".to_string()));
}

#[test]
fn test_alias_metadata_deserialization_missing() {
    // When alias is not present in JSON, it should be None
    let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
    let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.alias, None);
}

#[test]
fn test_alias_and_keyword_together() {
    // Both alias and expand can coexist on the same scriptlet
    let metadata = parse_html_comment_metadata("<!--\nalias: goog\nkeyword: :google\n-->");
    assert_eq!(metadata.alias, Some("goog".to_string()));
    assert_eq!(metadata.keyword, Some(":google".to_string()));
}

// ========================================
// Code Block Extraction Tests
// ========================================

#[test]
fn test_extract_code_block_basic_backticks() {
    let result = extract_code_block_nested("```ts\nconst x = 1;\n```");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "ts");
    assert_eq!(code, "const x = 1;");
}

#[test]
fn test_extract_code_block_basic_tildes() {
    let result = extract_code_block_nested("~~~bash\necho hello\n~~~");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "bash");
    assert_eq!(code, "echo hello");
}

#[test]
fn test_extract_code_block_nested_backticks_in_tildes() {
    let content = "~~~md\nHere's code:\n```ts\nconst x = 1;\n```\nDone!\n~~~";
    let result = extract_code_block_nested(content);
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "md");
    assert!(code.contains("```ts"));
    assert!(code.contains("const x = 1;"));
}

#[test]
fn test_extract_code_block_no_language() {
    let result = extract_code_block_nested("```\ncode here\n```");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "");
    assert_eq!(code, "code here");
}

#[test]
fn test_extract_code_block_none_without_fence() {
    let result = extract_code_block_nested("No code fence here");
    assert!(result.is_none());
}

#[test]
fn test_extract_code_block_multiline() {
    let result = extract_code_block_nested("```python\ndef foo():\n    return 42\n```");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "python");
    assert!(code.contains("def foo():"));
    assert!(code.contains("return 42"));
}

// ========================================
// Markdown Parsing Tests
// ========================================

#[test]
fn test_parse_markdown_basic_scriptlet() {
    let markdown = r#"## Test Script

```ts
console.log("hello");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Test Script");
    assert_eq!(scriptlets[0].tool, "ts");
    assert!(scriptlets[0].scriptlet_content.contains("console.log"));
}

#[test]
fn test_parse_markdown_with_group() {
    let markdown = r#"# My Group

## Script One

```bash
echo one
```

## Script Two

```bash
echo two
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 2);
    assert_eq!(scriptlets[0].group, "My Group");
    assert_eq!(scriptlets[1].group, "My Group");
}

#[test]
fn test_parse_markdown_with_metadata() {
    let markdown = r#"## Shortcut Script

<!-- shortcut: cmd k -->

```ts
console.log("triggered");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].metadata.shortcut, Some("cmd k".to_string()));
}

#[test]
fn test_parse_markdown_with_global_prepend() {
    let markdown = r#"# Shell Scripts

```bash
#!/bin/bash
set -e
```

## Script A

```bash
echo "A"
```

## Script B

```bash
echo "B"
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 2);

    // Both should have the prepended content
    assert!(scriptlets[0].scriptlet_content.contains("#!/bin/bash"));
    assert!(scriptlets[0].scriptlet_content.contains("set -e"));
    assert!(scriptlets[0].scriptlet_content.contains("echo \"A\""));

    assert!(scriptlets[1].scriptlet_content.contains("#!/bin/bash"));
    assert!(scriptlets[1].scriptlet_content.contains("echo \"B\""));
}

#[test]
fn test_parse_markdown_default_tool() {
    let markdown = r#"## No Language

```
just code
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    // Empty tool should default to "ts"
    assert_eq!(scriptlets[0].tool, "ts");
}

#[test]
fn test_parse_markdown_extracts_inputs() {
    let markdown = r#"## Template

```ts
console.log("Hello {{name}}! You are {{age}} years old.");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert!(scriptlets[0].inputs.contains(&"name".to_string()));
    assert!(scriptlets[0].inputs.contains(&"age".to_string()));
}

#[test]
fn test_parse_markdown_source_path() {
    let markdown = "## Test\n\n```bash\necho\n```";
    let scriptlets = parse_markdown_as_scriptlets(markdown, Some("/path/to/file.md"));
    assert_eq!(
        scriptlets[0].source_path,
        Some("/path/to/file.md".to_string())
    );
}

#[test]
fn test_parse_markdown_empty() {
    let scriptlets = parse_markdown_as_scriptlets("", None);
    assert!(scriptlets.is_empty());
}

#[test]
fn test_parse_markdown_no_code_block() {
    let markdown = "## Title\n\nJust text, no code.";
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert!(scriptlets.is_empty());
}

