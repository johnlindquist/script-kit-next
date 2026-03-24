// ============================================================
// Special Tool Tests (template, transform, edit, paste, type, submit, open)
// ============================================================

#[test]
fn test_run_scriptlet_template_basic() {
    // Template tool should return the content as stdout
    let scriptlet = Scriptlet::new(
        "Template Basic".to_string(),
        "template".to_string(),
        "Hello, World!".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok(), "Template should succeed: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "Hello, World!");
    assert!(result.stderr.is_empty());
}

#[test]
fn test_run_scriptlet_template_with_placeholders() {
    // Template with mustache placeholders should substitute values
    let scriptlet = Scriptlet::new(
        "Template Placeholders".to_string(),
        "template".to_string(),
        "Dear {{name}}, your order #{{order_id}} is ready.".to_string(),
    );

    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "Alice".to_string());
    inputs.insert("order_id".to_string(), "12345".to_string());

    let options = ScriptletExecOptions {
        inputs,
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert_eq!(result.stdout, "Dear Alice, your order #12345 is ready.");
}

#[test]
fn test_run_scriptlet_template_multiline() {
    // Template with multiple lines
    let template_content = "Line 1\nLine 2\nLine 3";
    let scriptlet = Scriptlet::new(
        "Template Multiline".to_string(),
        "template".to_string(),
        template_content.to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("Line 1"));
    assert!(result.stdout.contains("Line 2"));
    assert!(result.stdout.contains("Line 3"));
}

#[test]
fn test_run_scriptlet_template_empty() {
    // Empty template should return empty string
    let scriptlet = Scriptlet::new(
        "Template Empty".to_string(),
        "template".to_string(),
        "".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert_eq!(result.stdout, "");
}

// REMOVED: test_run_scriptlet_transform_basic — triggers real Cmd+C via selected text
// REMOVED: test_run_scriptlet_edit_returns_path — opens editor on actual system
// REMOVED: test_run_scriptlet_paste_basic — sends real paste keystrokes
// REMOVED: test_run_scriptlet_type_basic — sends real keystrokes to OS
// REMOVED: test_run_scriptlet_submit_basic — sends real paste+Enter keystrokes
// REMOVED: test_run_scriptlet_open_valid_url_format — opens browser URL

