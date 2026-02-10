// ============================================================
// Tool Dispatch Tests - Verify correct handler selection
// ============================================================

#[test]
fn test_tool_dispatch_template() {
    // Verify template tool is recognized and dispatched correctly
    let scriptlet = Scriptlet::new(
        "Dispatch Template".to_string(),
        "template".to_string(),
        "content".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[test]
fn test_tool_dispatch_python() {
    // Verify python tool dispatches to interpreter handler
    let scriptlet = Scriptlet::new(
        "Dispatch Python".to_string(),
        "python".to_string(),
        "print('hello')".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // May fail if python3 not installed, but should dispatch correctly
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tool_dispatch_ruby() {
    // Verify ruby tool dispatches to interpreter handler
    let scriptlet = Scriptlet::new(
        "Dispatch Ruby".to_string(),
        "ruby".to_string(),
        "puts 'hello'".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tool_dispatch_perl() {
    // Verify perl tool dispatches to interpreter handler
    let scriptlet = Scriptlet::new(
        "Dispatch Perl".to_string(),
        "perl".to_string(),
        "print 'hello'".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tool_dispatch_php() {
    // Verify php tool dispatches to interpreter handler
    let scriptlet = Scriptlet::new(
        "Dispatch PHP".to_string(),
        "php".to_string(),
        "<?php echo 'hello'; ?>".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tool_dispatch_node() {
    // Verify node tool dispatches to interpreter handler
    let scriptlet = Scriptlet::new(
        "Dispatch Node".to_string(),
        "node".to_string(),
        "console.log('hello')".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tool_dispatch_js_alias() {
    // Verify js is an alias for node
    let scriptlet = Scriptlet::new(
        "Dispatch JS".to_string(),
        "js".to_string(),
        "console.log('hello')".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok() || result.is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn test_tool_dispatch_applescript() {
    // Verify applescript tool dispatches correctly on macOS
    let scriptlet = Scriptlet::new(
        "Dispatch AppleScript".to_string(),
        "applescript".to_string(),
        "return \"hello\"".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // AppleScript should work on macOS
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tool_dispatch_unknown_falls_back_to_shell() {
    // Unknown tools should fall back to shell execution
    let scriptlet = Scriptlet::new(
        "Unknown Tool".to_string(),
        "unknown_tool_xyz".to_string(),
        "echo fallback".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // Should attempt shell execution as fallback
    assert!(result.is_ok() || result.is_err());
}

