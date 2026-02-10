// ========================================
// Per-Scriptlet Validation Tests
// ========================================

#[test]
fn test_validation_file_with_middle_malformed_loads_two() {
    // File with 3 scriptlets, middle one is malformed (no code block)
    let markdown = r#"## First Script

```bash
echo "first"
```

## Second Script (Malformed)

This scriptlet has no code block at all!
Just text here.

## Third Script

```bash
echo "third"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/file.md"));

    // Should load 2 scriptlets (first and third)
    assert_eq!(result.scriptlets.len(), 2);
    assert_eq!(result.scriptlets[0].name, "First Script");
    assert_eq!(result.scriptlets[1].name, "Third Script");

    // Should have 1 error for the malformed middle scriptlet
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0]
        .scriptlet_name
        .as_ref()
        .unwrap()
        .contains("Second Script"));
    assert!(result.errors[0]
        .error_message
        .contains("No code block found"));
}

#[test]
fn test_validation_all_valid_scriptlets_loads_all() {
    let markdown = r#"## Script A

```bash
echo "A"
```

## Script B

```python
print("B")
```

## Script C

```ts
console.log("C");
```
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/valid.md"));

    // All 3 should load
    assert_eq!(result.scriptlets.len(), 3);
    assert_eq!(result.scriptlets[0].name, "Script A");
    assert_eq!(result.scriptlets[1].name, "Script B");
    assert_eq!(result.scriptlets[2].name, "Script C");

    // No errors
    assert!(result.errors.is_empty());
}

#[test]
fn test_validation_all_invalid_scriptlets_loads_none() {
    let markdown = r#"## Bad One

No code block here.

## Bad Two

Also no code block.

## Bad Three

Still no code!
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/invalid.md"));

    // No scriptlets should load
    assert!(result.scriptlets.is_empty());

    // Should have 3 errors
    assert_eq!(result.errors.len(), 3);
}

#[test]
fn test_validation_error_includes_line_number() {
    let markdown = r#"## Good Script

```bash
echo "good"
```

## Bad Script On Line 8

No code block here.
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/file.md"));

    assert_eq!(result.errors.len(), 1);
    let error = &result.errors[0];

    // Line number should be present and greater than 1 (since bad script is not first)
    assert!(error.line_number.is_some());
    assert!(error.line_number.unwrap() > 1);
}

#[test]
fn test_validation_error_includes_file_path() {
    let markdown = r#"## Bad Script

No code block.
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/path/to/my/scripts.md"));

    assert_eq!(result.errors.len(), 1);
    let error = &result.errors[0];

    assert_eq!(error.file_path.to_string_lossy(), "/path/to/my/scripts.md");
}

#[test]
fn test_validation_error_includes_reason() {
    let markdown = r#"## Broken Script

Just text, no code fence.
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test.md"));

    assert_eq!(result.errors.len(), 1);
    let error = &result.errors[0];

    // Error message should explain the problem
    assert!(!error.error_message.is_empty());
    assert!(error.error_message.contains("code block"));
}

#[test]
fn test_validation_empty_h2_name_generates_error() {
    let markdown = r#"## 

```bash
echo "orphan code"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test.md"));

    // Empty name should generate an error
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].error_message.contains("Empty"));
}

#[test]
fn test_validation_parses_frontmatter() {
    let markdown = r#"---
name: My Bundle
icon: Star
author: Test Author
---

## Script One

```bash
echo "one"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, None);

    // Frontmatter should be parsed
    assert!(result.frontmatter.is_some());
    let fm = result.frontmatter.unwrap();
    assert_eq!(fm.name, Some("My Bundle".to_string()));
    assert_eq!(fm.icon, Some("Star".to_string()));
    assert_eq!(fm.author, Some("Test Author".to_string()));

    // Script should still load
    assert_eq!(result.scriptlets.len(), 1);
}

#[test]
fn test_validation_backward_compatibility_with_existing_parser() {
    // Same input should produce same scriptlets from both parsers
    let markdown = r#"# My Group

## Script A

```bash
echo "A"
```

## Script B

<!-- shortcut: cmd b -->

```ts
console.log("B");
```
"#;

    let old_result = parse_markdown_as_scriptlets(markdown, Some("/test.md"));
    let new_result = parse_scriptlets_with_validation(markdown, Some("/test.md"));

    // Same number of scriptlets
    assert_eq!(old_result.len(), new_result.scriptlets.len());

    // Same names
    assert_eq!(old_result[0].name, new_result.scriptlets[0].name);
    assert_eq!(old_result[1].name, new_result.scriptlets[1].name);

    // Same groups
    assert_eq!(old_result[0].group, new_result.scriptlets[0].group);
    assert_eq!(old_result[1].group, new_result.scriptlets[1].group);

    // Same metadata
    assert_eq!(
        old_result[1].metadata.shortcut,
        new_result.scriptlets[1].metadata.shortcut
    );
}

#[test]
fn test_validation_error_display() {
    let error = ScriptletValidationError::new(
        "/path/to/file.md",
        Some("My Script".to_string()),
        Some(42),
        "Something went wrong",
    );

    let display = format!("{}", error);

    // Should contain file path
    assert!(display.contains("/path/to/file.md"));
    // Should contain line number
    assert!(display.contains(":42"));
    // Should contain script name
    assert!(display.contains("[My Script]"));
    // Should contain error message
    assert!(display.contains("Something went wrong"));
}

#[test]
fn test_validation_error_display_without_optional_fields() {
    let error = ScriptletValidationError::new("/file.md", None, None, "Error message");

    let display = format!("{}", error);

    // Should still work without optional fields
    assert!(display.contains("/file.md"));
    assert!(display.contains("Error message"));
    // Should NOT contain line number prefix or script name brackets
    assert!(!display.contains("["));
    assert!(!display.contains("]"));
}

#[test]
fn test_scriptlet_parse_result_default() {
    let result = ScriptletParseResult::default();

    assert!(result.scriptlets.is_empty());
    assert!(result.errors.is_empty());
    assert!(result.frontmatter.is_none());
}

#[test]
fn test_validation_mixed_valid_invalid_preserves_order() {
    let markdown = r#"## First (Valid)

```bash
echo "1"
```

## Second (Invalid)

No code.

## Third (Valid)

```bash
echo "3"
```

## Fourth (Invalid)

Also no code.

## Fifth (Valid)

```bash
echo "5"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, None);

    // Valid scriptlets should preserve order
    assert_eq!(result.scriptlets.len(), 3);
    assert_eq!(result.scriptlets[0].name, "First (Valid)");
    assert_eq!(result.scriptlets[1].name, "Third (Valid)");
    assert_eq!(result.scriptlets[2].name, "Fifth (Valid)");

    // Errors should also be in order
    assert_eq!(result.errors.len(), 2);
    assert!(result.errors[0]
        .scriptlet_name
        .as_ref()
        .unwrap()
        .contains("Second"));
    assert!(result.errors[1]
        .scriptlet_name
        .as_ref()
        .unwrap()
        .contains("Fourth"));
}

