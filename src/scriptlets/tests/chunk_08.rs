// ========================================
// H3 Scriptlet Action Tests
// ========================================

#[test]
fn test_scriptlet_action_struct() {
    let action = ScriptletAction {
        name: "Copy to Clipboard".to_string(),
        command: "copy-to-clipboard".to_string(),
        tool: "bash".to_string(),
        code: "echo '{{text}}' | pbcopy".to_string(),
        inputs: vec!["text".to_string()],
        shortcut: Some("cmd+c".to_string()),
        description: Some("Copy text to clipboard".to_string()),
    };

    assert_eq!(action.name, "Copy to Clipboard");
    assert_eq!(action.action_id(), "scriptlet_action:copy-to-clipboard");
}

#[test]
fn test_parse_h3_action_basic() {
    let content = r#"
```bash
echo "action code"
```
"#;

    let action = parse_h3_action("My Action", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert_eq!(action.name, "My Action");
    assert_eq!(action.command, "my-action");
    assert_eq!(action.tool, "bash");
    assert_eq!(action.code, "echo \"action code\"");
}

#[test]
fn test_parse_h3_action_with_metadata() {
    let content = r#"
<!-- shortcut: cmd+c -->
<!-- description: Copies to clipboard -->

```bash
echo "copy"
```
"#;

    let action = parse_h3_action("Copy Action", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert_eq!(action.shortcut, Some("cmd+c".to_string()));
    assert_eq!(action.description, Some("Copies to clipboard".to_string()));
}

#[test]
fn test_parse_h3_action_extracts_inputs() {
    let content = r#"
```bash
echo "Hello {{name}}, your age is {{age}}"
```
"#;

    let action = parse_h3_action("Greeting", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert!(action.inputs.contains(&"name".to_string()));
    assert!(action.inputs.contains(&"age".to_string()));
}

#[test]
fn test_parse_h3_action_no_code_block() {
    let content = "Just text, no code fence.";
    let action = parse_h3_action("Bad Action", content);
    assert!(action.is_none());
}

#[test]
fn test_parse_h3_action_open_tool() {
    let content = r#"
```open
https://github.com/{{repo}}
```
"#;

    let action = parse_h3_action("Open GitHub", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert_eq!(action.tool, "open");
    assert!(action.code.contains("https://github.com"));
}

#[test]
fn test_parse_h3_action_invalid_tool() {
    let content = r#"
```invalidtool
some code
```
"#;

    // Invalid tool should return None
    let action = parse_h3_action("Bad Tool Action", content);
    assert!(action.is_none());
}

#[test]
fn test_extract_h3_actions_basic() {
    let section = r#"## My Scriptlet

```bash
echo "main code"
```

### Copy to Clipboard

```bash
echo "copy"
```

### Open Browser

```open
https://example.com
```
"#;

    let actions = extract_h3_actions(section);
    assert_eq!(actions.len(), 2);

    assert_eq!(actions[0].name, "Copy to Clipboard");
    assert_eq!(actions[0].tool, "bash");

    assert_eq!(actions[1].name, "Open Browser");
    assert_eq!(actions[1].tool, "open");
}

#[test]
fn test_extract_h3_actions_with_metadata() {
    let section = r#"## Scriptlet

```bash
main code
```

### Action One
<!-- shortcut: cmd+1 -->
```bash
action one code
```

### Action Two
<!-- shortcut: cmd+2 -->
<!-- description: Second action -->
```bash
action two code
```
"#;

    let actions = extract_h3_actions(section);
    assert_eq!(actions.len(), 2);

    assert_eq!(actions[0].shortcut, Some("cmd+1".to_string()));
    assert_eq!(actions[1].shortcut, Some("cmd+2".to_string()));
    assert_eq!(actions[1].description, Some("Second action".to_string()));
}

#[test]
fn test_extract_h3_actions_none_before_main_code() {
    // H3s before the main code block should not be captured
    let section = r#"## Scriptlet

### This Should Be Ignored
```bash
ignored
```

```bash
main code - this is the FIRST valid tool codefence
```

### This Should Be Captured
```bash
captured
```
"#;

    let actions = extract_h3_actions(section);
    // Only the H3 AFTER the main code should be captured
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "This Should Be Captured");
}

#[test]
fn test_extract_h3_actions_empty_section() {
    let section = r#"## Scriptlet

```bash
main code
```
"#;

    let actions = extract_h3_actions(section);
    assert!(actions.is_empty());
}

#[test]
fn test_extract_h3_actions_h3_without_code() {
    let section = r#"## Scriptlet

```bash
main
```

### Bad H3 Without Code

Just text, no code fence.

### Good H3

```bash
good code
```
"#;

    let actions = extract_h3_actions(section);
    // Only the good H3 should be captured
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "Good H3");
}

#[test]
fn test_parse_markdown_includes_h3_actions() {
    let markdown = r#"## GitHub Tools

<!-- shortcut: cmd+g -->

```open
https://github.com/{{repo}}
```

### Copy SSH URL
<!-- shortcut: cmd+shift+c -->
```bash
echo "git@github.com:{{repo}}.git" | pbcopy
```

### View README
```open
https://github.com/{{repo}}/blob/main/README.md
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "GitHub Tools");
    assert_eq!(scriptlet.actions.len(), 2);

    // First action
    assert_eq!(scriptlet.actions[0].name, "Copy SSH URL");
    assert_eq!(scriptlet.actions[0].tool, "bash");
    assert_eq!(
        scriptlet.actions[0].shortcut,
        Some("cmd+shift+c".to_string())
    );

    // Second action
    assert_eq!(scriptlet.actions[1].name, "View README");
    assert_eq!(scriptlet.actions[1].tool, "open");
    assert!(scriptlet.actions[1].shortcut.is_none());
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_actions() {
    let markdown = r#"# Tools

## URL Opener

```open
https://{{url}}
```

### Open in Safari
```bash
open -a Safari "https://{{url}}"
```

## File Manager

```bash
ls -la {{path}}
```

### Open in Finder
```bash
open {{path}}
```

### Copy Path
```bash
echo "{{path}}" | pbcopy
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 2);

    // First scriptlet
    assert_eq!(scriptlets[0].name, "URL Opener");
    assert_eq!(scriptlets[0].actions.len(), 1);
    assert_eq!(scriptlets[0].actions[0].name, "Open in Safari");

    // Second scriptlet
    assert_eq!(scriptlets[1].name, "File Manager");
    assert_eq!(scriptlets[1].actions.len(), 2);
    assert_eq!(scriptlets[1].actions[0].name, "Open in Finder");
    assert_eq!(scriptlets[1].actions[1].name, "Copy Path");
}

#[test]
fn test_scriptlet_action_serialization() {
    let action = ScriptletAction {
        name: "Test Action".to_string(),
        command: "test-action".to_string(),
        tool: "bash".to_string(),
        code: "echo test".to_string(),
        inputs: vec!["var".to_string()],
        shortcut: Some("cmd+t".to_string()),
        description: Some("A test action".to_string()),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: ScriptletAction = serde_json::from_str(&json).unwrap();

    assert_eq!(action.name, deserialized.name);
    assert_eq!(action.tool, deserialized.tool);
    assert_eq!(action.shortcut, deserialized.shortcut);
}

#[test]
fn test_scriptlet_with_actions_serialization() {
    let markdown = r#"## Test Scriptlet

```bash
echo main
```

### Action One
```bash
echo one
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    let scriptlet = &scriptlets[0];

    let json = serde_json::to_string(scriptlet).unwrap();
    let deserialized: Scriptlet = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.actions.len(), 1);
    assert_eq!(deserialized.actions[0].name, "Action One");
}

#[test]
fn test_validation_parser_includes_actions() {
    let markdown = r#"## Test

```bash
main
```

### Sub Action
```bash
sub
```
"#;

    let result = parse_scriptlets_with_validation(markdown, None);
    assert_eq!(result.scriptlets.len(), 1);
    assert_eq!(result.scriptlets[0].actions.len(), 1);
}

#[test]
fn test_scriptlet_new_has_empty_actions() {
    let scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());

    assert!(scriptlet.actions.is_empty());
}

