// ========================================
// Variable Substitution Tests
// ========================================

#[test]
fn test_format_scriptlet_named_inputs() {
    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "Alice".to_string());
    inputs.insert("greeting".to_string(), "Hello".to_string());

    let result = format_scriptlet("{{greeting}}, {{name}}!", &inputs, &[], false);

    assert_eq!(result, "Hello, Alice!");
}

#[test]
fn test_format_scriptlet_positional_unix() {
    let result = format_scriptlet(
        "echo $1 and $2",
        &HashMap::new(),
        &["first".to_string(), "second".to_string()],
        false,
    );

    assert_eq!(result, "echo first and second");
}

#[test]
fn test_format_scriptlet_positional_windows() {
    let result = format_scriptlet(
        "echo %1 and %2",
        &HashMap::new(),
        &["first".to_string(), "second".to_string()],
        true,
    );

    assert_eq!(result, "echo first and second");
}

#[test]
fn test_format_scriptlet_all_args_unix() {
    let result = format_scriptlet(
        "echo $@",
        &HashMap::new(),
        &["one".to_string(), "two".to_string(), "three".to_string()],
        false,
    );

    assert_eq!(result, r#"echo "one" "two" "three""#);
}

#[test]
fn test_format_scriptlet_all_args_windows() {
    let result = format_scriptlet(
        "echo %*",
        &HashMap::new(),
        &["one".to_string(), "two".to_string()],
        true,
    );

    assert_eq!(result, r#"echo "one" "two""#);
}

#[test]
fn test_format_scriptlet_combined() {
    let mut inputs = HashMap::new();
    inputs.insert("prefix".to_string(), "Result:".to_string());

    let result = format_scriptlet(
        "{{prefix}} $1 and $2",
        &inputs,
        &["A".to_string(), "B".to_string()],
        false,
    );

    assert_eq!(result, "Result: A and B");
}

#[test]
fn test_format_scriptlet_escape_quotes() {
    let result = format_scriptlet(
        "echo $@",
        &HashMap::new(),
        &["has\"quote".to_string()],
        false,
    );

    assert_eq!(result, r#"echo "has\"quote""#);
}

// ========================================
// Conditional Processing Tests
// ========================================

#[test]
fn test_process_conditionals_if_true() {
    let mut flags = HashMap::new();
    flags.insert("show".to_string(), true);

    let result = process_conditionals("{{#if show}}visible{{/if}}", &flags);
    assert_eq!(result, "visible");
}

#[test]
fn test_process_conditionals_if_false() {
    let mut flags = HashMap::new();
    flags.insert("show".to_string(), false);

    let result = process_conditionals("{{#if show}}visible{{/if}}", &flags);
    assert_eq!(result, "");
}

#[test]
fn test_process_conditionals_if_missing_flag() {
    let flags = HashMap::new();

    let result = process_conditionals("{{#if undefined}}visible{{/if}}", &flags);
    assert_eq!(result, "");
}

#[test]
fn test_process_conditionals_if_else_true() {
    let mut flags = HashMap::new();
    flags.insert("flag".to_string(), true);

    let result = process_conditionals("{{#if flag}}yes{{else}}no{{/if}}", &flags);
    assert_eq!(result, "yes");
}

#[test]
fn test_process_conditionals_if_else_false() {
    let mut flags = HashMap::new();
    flags.insert("flag".to_string(), false);

    let result = process_conditionals("{{#if flag}}yes{{else}}no{{/if}}", &flags);
    assert_eq!(result, "no");
}

#[test]
fn test_process_conditionals_else_if() {
    let mut flags = HashMap::new();
    flags.insert("a".to_string(), false);
    flags.insert("b".to_string(), true);

    let result = process_conditionals("{{#if a}}A{{else if b}}B{{else}}C{{/if}}", &flags);
    assert_eq!(result, "B");
}

#[test]
fn test_process_conditionals_nested() {
    let mut flags = HashMap::new();
    flags.insert("outer".to_string(), true);
    flags.insert("inner".to_string(), true);

    let result = process_conditionals("{{#if outer}}[{{#if inner}}nested{{/if}}]{{/if}}", &flags);
    assert_eq!(result, "[nested]");
}

#[test]
fn test_process_conditionals_preserves_other_content() {
    let mut flags = HashMap::new();
    flags.insert("show".to_string(), true);

    let result = process_conditionals("Before {{#if show}}middle{{/if}} after", &flags);
    assert_eq!(result, "Before middle after");
}

#[test]
fn test_process_conditionals_with_variables() {
    let mut flags = HashMap::new();
    flags.insert("useTitle".to_string(), true);

    let result = process_conditionals("{{#if useTitle}}Hello {{name}}{{/if}}", &flags);
    assert_eq!(result, "Hello {{name}}");
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_full_scriptlet_workflow() {
    let markdown = r#"# Tools

## Greeter

<!-- 
description: Greets a person
shortcut: cmd g
-->

```ts
const name = "{{name}}";
{{#if formal}}console.log(`Dear ${name}`);{{else}}console.log(`Hey ${name}!`);{{/if}}
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, Some("/test.md"));
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "Greeter");
    assert_eq!(scriptlet.group, "Tools");
    assert_eq!(
        scriptlet.metadata.description,
        Some("Greets a person".to_string())
    );
    assert_eq!(scriptlet.metadata.shortcut, Some("cmd g".to_string()));
    assert!(scriptlet.inputs.contains(&"name".to_string()));

    // Test variable substitution
    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "Alice".to_string());

    let mut flags = HashMap::new();
    flags.insert("formal".to_string(), true);

    let content = process_conditionals(&scriptlet.scriptlet_content, &flags);
    let result = format_scriptlet(&content, &inputs, &[], false);

    assert!(result.contains("Alice"));
    assert!(result.contains("Dear"));
    assert!(!result.contains("Hey"));
}

#[test]
fn test_complex_markdown_parsing() {
    let markdown = r#"# Productivity

## Open URL

<!-- shortcut: cmd u -->

```open
https://example.com
```

## Type Date

<!-- keyword: ddate,, -->

```type
{{#if iso}}{{date}}{{else}}{{formattedDate}}{{/if}}
```

# Development

```bash
# Common setup
export PATH="$HOME/bin:$PATH"
```

## Run Tests

```bash
npm test $@
```

## Build

```bash
npm run build $1
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);

    // Should have 4 scriptlets: Open URL, Type Date, Run Tests, Build
    assert_eq!(scriptlets.len(), 4);

    // First two belong to "Productivity" group
    assert_eq!(scriptlets[0].group, "Productivity");
    assert_eq!(scriptlets[0].name, "Open URL");
    assert_eq!(scriptlets[0].tool, "open");

    assert_eq!(scriptlets[1].group, "Productivity");
    assert_eq!(scriptlets[1].name, "Type Date");
    assert_eq!(scriptlets[1].metadata.keyword, Some("ddate,,".to_string()));

    // Last two belong to "Development" group and have the common setup prepended
    assert_eq!(scriptlets[2].group, "Development");
    assert_eq!(scriptlets[2].name, "Run Tests");
    assert!(scriptlets[2].scriptlet_content.contains("export PATH"));
    assert!(scriptlets[2].scriptlet_content.contains("npm test"));

    assert_eq!(scriptlets[3].group, "Development");
    assert_eq!(scriptlets[3].name, "Build");
    assert!(scriptlets[3].scriptlet_content.contains("export PATH"));
}

#[test]
fn test_scriptlet_metadata_serialization() {
    let metadata = ScriptletMetadata {
        shortcut: Some("cmd k".to_string()),
        description: Some("Test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.shortcut, deserialized.shortcut);
    assert_eq!(metadata.description, deserialized.description);
}

#[test]
fn test_scriptlet_serialization() {
    let scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo hello".to_string(),
    );

    let json = serde_json::to_string(&scriptlet).unwrap();
    let deserialized: Scriptlet = serde_json::from_str(&json).unwrap();

    assert_eq!(scriptlet.name, deserialized.name);
    assert_eq!(scriptlet.tool, deserialized.tool);
    assert_eq!(scriptlet.scriptlet_content, deserialized.scriptlet_content);
}

