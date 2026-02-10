// ========================================
// Codefence Metadata Integration Tests
// ========================================

#[test]
fn test_scriptlet_with_codefence_metadata() {
    // Test that scriptlets can be parsed from markdown with codefence metadata blocks
    let markdown = r#"## Quick Todo

```metadata
{ "name": "Quick Todo", "description": "Add a todo item", "shortcut": "cmd t" }
```

```ts
const item = await arg("Todo item");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "Quick Todo");
    assert_eq!(scriptlet.tool, "ts");

    // Typed metadata should be populated
    assert!(scriptlet.typed_metadata.is_some());
    let typed = scriptlet.typed_metadata.as_ref().unwrap();
    assert_eq!(typed.name, Some("Quick Todo".to_string()));
    assert_eq!(typed.description, Some("Add a todo item".to_string()));
    assert_eq!(typed.shortcut, Some("cmd t".to_string()));
}

#[test]
fn test_scriptlet_with_codefence_schema() {
    // Test that scriptlets can parse schema blocks
    let markdown = r#"## Input Script

```schema
{
    "input": {
        "title": { "type": "string", "required": true }
    },
    "output": {
        "result": { "type": "string" }
    }
}
```

```ts
const { title } = await input();
output({ result: title.toUpperCase() });
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert!(scriptlet.schema.is_some());

    let schema = scriptlet.schema.as_ref().unwrap();
    assert_eq!(schema.input.len(), 1);
    assert!(schema.input.contains_key("title"));
    assert_eq!(schema.output.len(), 1);
    assert!(schema.output.contains_key("result"));
}

#[test]
fn test_scriptlet_falls_back_to_html_comments() {
    // When no codefence metadata exists, should fall back to HTML comments
    let markdown = r#"## Legacy Script

<!-- shortcut: cmd l -->
<!-- description: A legacy script using HTML comments -->

```bash
echo "Hello from legacy"
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    // HTML comment metadata should still work
    assert_eq!(scriptlet.metadata.shortcut, Some("cmd l".to_string()));
    assert_eq!(
        scriptlet.metadata.description,
        Some("A legacy script using HTML comments".to_string())
    );

    // Typed metadata should be None since no codefence metadata block
    assert!(scriptlet.typed_metadata.is_none());
    assert!(scriptlet.schema.is_none());
}

#[test]
fn test_scriptlet_struct_has_typed_fields() {
    // Verify the Scriptlet struct has the new fields
    let scriptlet = Scriptlet::new(
        "Test".to_string(),
        "ts".to_string(),
        "console.log('test')".to_string(),
    );

    // New fields should exist and default to None
    assert!(scriptlet.typed_metadata.is_none());
    assert!(scriptlet.schema.is_none());
}

#[test]
fn test_mixed_codefence_and_html_prefers_codefence() {
    // When both codefence metadata and HTML comments exist,
    // codefence should take precedence for typed_metadata
    let markdown = r#"## Mixed Script

<!-- shortcut: cmd x -->
<!-- description: HTML description -->

```metadata
{ "name": "Codefence Name", "description": "Codefence description", "shortcut": "cmd y" }
```

```ts
console.log("mixed");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];

    // Codefence metadata should populate typed_metadata
    assert!(scriptlet.typed_metadata.is_some());
    let typed = scriptlet.typed_metadata.as_ref().unwrap();
    assert_eq!(typed.name, Some("Codefence Name".to_string()));
    assert_eq!(typed.description, Some("Codefence description".to_string()));
    assert_eq!(typed.shortcut, Some("cmd y".to_string()));

    // HTML comments should still populate legacy metadata struct
    // (for backward compatibility)
    assert_eq!(scriptlet.metadata.shortcut, Some("cmd x".to_string()));
    assert_eq!(
        scriptlet.metadata.description,
        Some("HTML description".to_string())
    );
}

#[test]
fn test_codefence_metadata_and_schema_together() {
    // Test scriptlet with both metadata and schema codefence blocks
    let markdown = r#"## Full Featured

```metadata
{ "name": "Full Featured", "description": "Has both metadata and schema" }
```

```schema
{
    "input": {
        "name": { "type": "string", "required": true }
    }
}
```

```ts
const { name } = await input();
console.log(name);
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];

    // Both should be populated
    assert!(scriptlet.typed_metadata.is_some());
    assert!(scriptlet.schema.is_some());

    let typed = scriptlet.typed_metadata.as_ref().unwrap();
    assert_eq!(typed.name, Some("Full Featured".to_string()));

    let schema = scriptlet.schema.as_ref().unwrap();
    assert!(schema.input.contains_key("name"));
}

