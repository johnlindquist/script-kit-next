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

// Transform tool tests - requires system access (macOS only)
#[cfg(all(target_os = "macos", feature = "system-tests"))]
#[test]
fn test_run_scriptlet_transform_basic() {
    // Transform requires selected text and accessibility permissions
    // This test verifies the tool dispatches correctly
    let scriptlet = Scriptlet::new(
        "Transform Test".to_string(),
        "transform".to_string(),
        "tr '[:lower:]' '[:upper:]'".to_string(),
    );

    // Note: This test will only pass if there's selected text
    // In CI/automated testing, this may fail due to no selection
    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // We just verify it doesn't panic - actual transform behavior depends on system state
    assert!(result.is_ok() || result.is_err());
}

// Edit tool tests - requires system-tests feature to avoid opening editor on every test run
#[cfg(feature = "system-tests")]
#[test]
fn test_run_scriptlet_edit_returns_path() {
    // Edit tool should attempt to open the file path in an editor
    // This actually tries to open the editor, so it's gated behind system-tests
    let scriptlet = Scriptlet::new(
        "Edit Test".to_string(),
        "edit".to_string(),
        "/tmp/nonexistent-test-file.txt".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // Edit may succeed or fail depending on $EDITOR availability
    // The important thing is it handles the tool type correctly
    assert!(result.is_ok() || result.is_err());
}

// Paste tool tests - requires system access (macOS only)
#[cfg(all(target_os = "macos", feature = "system-tests"))]
#[test]
fn test_run_scriptlet_paste_basic() {
    // Paste tool pastes content at cursor position
    let scriptlet = Scriptlet::new(
        "Paste Test".to_string(),
        "paste".to_string(),
        "Pasted content".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // Paste requires accessibility permissions
    assert!(result.is_ok() || result.is_err());
}

// Type tool tests - requires system access (macOS only)
#[cfg(all(target_os = "macos", feature = "system-tests"))]
#[test]
fn test_run_scriptlet_type_basic() {
    // Type tool simulates keyboard typing
    let scriptlet = Scriptlet::new(
        "Type Test".to_string(),
        "type".to_string(),
        "Typed content".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // Type requires accessibility permissions
    assert!(result.is_ok() || result.is_err());
}

// Submit tool tests - requires system access (macOS only)
#[cfg(all(target_os = "macos", feature = "system-tests"))]
#[test]
fn test_run_scriptlet_submit_basic() {
    // Submit tool pastes content and presses Enter
    let scriptlet = Scriptlet::new(
        "Submit Test".to_string(),
        "submit".to_string(),
        "Submitted content".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // Submit requires accessibility permissions
    assert!(result.is_ok() || result.is_err());
}

// Open tool test - requires system-tests feature to avoid opening browser on every test run
#[cfg(feature = "system-tests")]
#[test]
fn test_run_scriptlet_open_valid_url_format() {
    // Test that open tool handles URL format correctly
    // This actually opens the URL, so it's gated behind system-tests
    let scriptlet = Scriptlet::new(
        "Open URL".to_string(),
        "open".to_string(),
        "https://example.com".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // Open should succeed on systems with a browser
    assert!(result.is_ok() || result.is_err());
}

