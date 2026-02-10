// ========================================
// Shared Actions (.actions.md) Tests
// ========================================

#[test]
fn test_parse_actions_file_basic() {
    let content = r#"# URL Actions

### Copy URL
```bash
echo "{{url}}" | pbcopy
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "Copy URL");
    assert_eq!(actions[0].command, "copy-url");
    assert_eq!(actions[0].tool, "bash");
    assert!(actions[0].code.contains("pbcopy"));
}

#[test]
fn test_parse_actions_file_multiple_actions() {
    let content = r#"# URL Actions

### Copy URL
```bash
echo "{{url}}" | pbcopy
```

### Open URL
```open
{{url}}
```

# Text Actions

### Make Uppercase
```bash
echo "{{text}}" | tr '[:lower:]' '[:upper:]'
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].name, "Copy URL");
    assert_eq!(actions[1].name, "Open URL");
    assert_eq!(actions[2].name, "Make Uppercase");
}

#[test]
fn test_parse_actions_file_with_metadata() {
    let content = r#"### Copy URL
<!-- shortcut: cmd+c -->
<!-- description: Copy the URL to clipboard -->
```bash
echo "{{url}}" | pbcopy
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].shortcut, Some("cmd+c".to_string()));
    assert_eq!(
        actions[0].description,
        Some("Copy the URL to clipboard".to_string())
    );
}

#[test]
fn test_parse_actions_file_ignores_h2() {
    let content = r#"## This is an H2 (should be ignored)
```bash
echo "ignored"
```

### This is an H3 (should be parsed)
```bash
echo "action"
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "This is an H3 (should be parsed)");
}

#[test]
fn test_parse_actions_file_extracts_inputs() {
    let content = r#"### Send Email
```bash
echo "To: {{email}}, Subject: {{subject}}"
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].inputs.len(), 2);
    assert!(actions[0].inputs.contains(&"email".to_string()));
    assert!(actions[0].inputs.contains(&"subject".to_string()));
}

#[test]
fn test_parse_actions_file_empty() {
    let content = "# Just a header with no actions\n\nSome text here.";
    let actions = parse_actions_file(content);
    assert!(actions.is_empty());
}

#[test]
fn test_parse_actions_file_skips_invalid_tool() {
    let content = r#"### Invalid Tool Action
```invalidtool
some code
```
"#;

    let actions = parse_actions_file(content);
    assert!(actions.is_empty());
}

#[test]
fn test_get_actions_file_path() {
    use std::path::Path;

    let md_path = Path::new("/path/to/main.md");
    let actions_path = get_actions_file_path(md_path);
    assert_eq!(actions_path.to_string_lossy(), "/path/to/main.actions.md");

    let md_path2 = Path::new("/extensions/foo.bar.md");
    let actions_path2 = get_actions_file_path(md_path2);
    assert_eq!(
        actions_path2.to_string_lossy(),
        "/extensions/foo.bar.actions.md"
    );
}

#[cfg(unix)]
#[test]
fn test_get_actions_file_path_preserves_non_utf8_stem_bytes() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::path::PathBuf;

    let source = PathBuf::from(OsString::from_vec(vec![0x66, 0x6f, 0x80, b'.', b'm', b'd']));
    let actions_path = get_actions_file_path(&source);

    let file_name = actions_path
        .file_name()
        .expect("actions path should include file name");
    assert_eq!(
        file_name.as_bytes(),
        &[0x66, 0x6f, 0x80, b'.', b'a', b'c', b't', b'i', b'o', b'n', b's', b'.', b'm', b'd',]
    );
}

#[test]
fn test_merge_shared_actions_basic() {
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo test".to_string(),
    );

    let shared_actions = vec![
        ScriptletAction {
            name: "Copy".to_string(),
            command: "copy".to_string(),
            tool: "bash".to_string(),
            code: "pbcopy".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Open".to_string(),
            command: "open".to_string(),
            tool: "open".to_string(),
            code: "{{url}}".to_string(),
            inputs: vec!["url".to_string()],
            shortcut: None,
            description: None,
        },
    ];

    merge_shared_actions(&mut scriptlet, &shared_actions);
    assert_eq!(scriptlet.actions.len(), 2);
}

#[test]
fn test_merge_shared_actions_inline_takes_precedence() {
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo test".to_string(),
    );

    // Add an inline action
    scriptlet.actions.push(ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "inline copy code".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+c".to_string()),
        description: None,
    });

    // Shared action with same command
    let shared_actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "shared copy code".to_string(),
        inputs: vec![],
        shortcut: None,
        description: Some("Shared description".to_string()),
    }];

    merge_shared_actions(&mut scriptlet, &shared_actions);

    // Should still have only 1 action (inline takes precedence)
    assert_eq!(scriptlet.actions.len(), 1);
    // The code should be from the inline action
    assert_eq!(scriptlet.actions[0].code, "inline copy code");
    // Shortcut should be from inline action
    assert_eq!(scriptlet.actions[0].shortcut, Some("cmd+c".to_string()));
}

#[test]
fn test_merge_shared_actions_mixed() {
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo test".to_string(),
    );

    // Add an inline action
    scriptlet.actions.push(ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "inline".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    });

    // Shared actions: one conflicts, two are new
    let shared_actions = vec![
        ScriptletAction {
            name: "Copy".to_string(), // conflicts
            command: "copy".to_string(),
            tool: "bash".to_string(),
            code: "shared".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Open".to_string(), // new
            command: "open".to_string(),
            tool: "open".to_string(),
            code: "open".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Delete".to_string(), // new
            command: "delete".to_string(),
            tool: "bash".to_string(),
            code: "rm".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    merge_shared_actions(&mut scriptlet, &shared_actions);

    // Should have 3 actions: 1 inline + 2 new from shared
    assert_eq!(scriptlet.actions.len(), 3);

    // First is inline copy (unchanged)
    assert_eq!(scriptlet.actions[0].command, "copy");
    assert_eq!(scriptlet.actions[0].code, "inline");

    // Then the two new shared actions
    assert_eq!(scriptlet.actions[1].command, "open");
    assert_eq!(scriptlet.actions[2].command, "delete");
}

// ========================================
// Integration Tests (filesystem-based)
// ========================================

#[test]
fn test_shared_actions_loaded_from_companion_file() {
    use std::fs;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create main.md with a scriptlet
    let main_md_path = temp_path.join("main.md");
    let main_md_content = r#"## Open Google

```open
https://www.google.com
```
"#;
    fs::write(&main_md_path, main_md_content).unwrap();

    // Create main.actions.md with shared actions
    let actions_md_path = temp_path.join("main.actions.md");
    let actions_md_content = r#"### Copy URL
<!-- shortcut: cmd+c -->
```bash
echo "{{content}}" | pbcopy
```

### Open in Safari
```bash
open -a Safari "{{content}}"
```
"#;
    fs::write(&actions_md_path, actions_md_content).unwrap();

    // Parse the main.md file
    let content = fs::read_to_string(&main_md_path).unwrap();
    let scriptlets = parse_markdown_as_scriptlets(&content, Some(main_md_path.to_str().unwrap()));

    // Should have 1 scriptlet
    assert_eq!(scriptlets.len(), 1);

    // The scriptlet should have the shared actions merged
    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "Open Google");
    assert_eq!(scriptlet.actions.len(), 2);

    // Check the actions are correct
    assert_eq!(scriptlet.actions[0].name, "Copy URL");
    assert_eq!(scriptlet.actions[0].shortcut, Some("cmd+c".to_string()));
    assert!(scriptlet.actions[0].code.contains("pbcopy"));

    assert_eq!(scriptlet.actions[1].name, "Open in Safari");
    assert!(scriptlet.actions[1].code.contains("Safari"));
}

#[test]
fn test_shared_actions_not_loaded_for_actions_file() {
    use std::fs;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create main.actions.md (the actions file itself)
    let actions_md_path = temp_path.join("main.actions.md");
    let actions_md_content = r#"### Copy URL
```bash
echo "test" | pbcopy
```
"#;
    fs::write(&actions_md_path, actions_md_content).unwrap();

    // Parsing the .actions.md file should NOT try to load main.actions.actions.md
    // (it would be a recursive loop)
    let content = fs::read_to_string(&actions_md_path).unwrap();
    let scriptlets =
        parse_markdown_as_scriptlets(&content, Some(actions_md_path.to_str().unwrap()));

    // Should have 0 scriptlets (actions files only have H3, not H2)
    assert_eq!(scriptlets.len(), 0);
}

#[test]
fn test_inline_actions_take_precedence_over_shared() {
    use std::fs;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create main.md with a scriptlet that has an inline action
    let main_md_path = temp_path.join("main.md");
    let main_md_content = r#"## My Scriptlet

```bash
echo "hello"
```

### Copy
<!-- shortcut: cmd+shift+c -->
```bash
inline copy code
```
"#;
    fs::write(&main_md_path, main_md_content).unwrap();

    // Create main.actions.md with a shared action of the same name
    let actions_md_path = temp_path.join("main.actions.md");
    let actions_md_content = r#"### Copy
<!-- shortcut: cmd+c -->
```bash
shared copy code
```

### Delete
```bash
rm something
```
"#;
    fs::write(&actions_md_path, actions_md_content).unwrap();

    // Parse the main.md file
    let content = fs::read_to_string(&main_md_path).unwrap();
    let scriptlets = parse_markdown_as_scriptlets(&content, Some(main_md_path.to_str().unwrap()));

    assert_eq!(scriptlets.len(), 1);
    let scriptlet = &scriptlets[0];

    // Should have 2 actions: inline Copy + shared Delete
    // The shared Copy should be skipped because inline takes precedence
    assert_eq!(scriptlet.actions.len(), 2);

    // First action is inline Copy
    assert_eq!(scriptlet.actions[0].name, "Copy");
    assert_eq!(scriptlet.actions[0].code, "inline copy code");
    assert_eq!(
        scriptlet.actions[0].shortcut,
        Some("cmd+shift+c".to_string())
    ); // inline shortcut

    // Second action is shared Delete
    assert_eq!(scriptlet.actions[1].name, "Delete");
}
