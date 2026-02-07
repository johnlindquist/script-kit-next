// ============================================================
// Scriptlet Execution Tests
// ============================================================

use super::{build_final_content, run_scriptlet, tool_extension, ScriptletExecOptions};
use crate::scriptlets::Scriptlet;

#[test]
fn test_tool_extension() {
    assert_eq!(tool_extension("ruby"), "rb");
    assert_eq!(tool_extension("python"), "py");
    assert_eq!(tool_extension("perl"), "pl");
    assert_eq!(tool_extension("php"), "php");
    assert_eq!(tool_extension("bash"), "sh");
    assert_eq!(tool_extension("sh"), "sh");
    assert_eq!(tool_extension("zsh"), "zsh");
    assert_eq!(tool_extension("fish"), "fish");
    assert_eq!(tool_extension("node"), "js");
    assert_eq!(tool_extension("js"), "js");
    assert_eq!(tool_extension("ts"), "ts");
    assert_eq!(tool_extension("kit"), "ts");
    assert_eq!(tool_extension("bun"), "ts");
    assert_eq!(tool_extension("deno"), "ts");
    assert_eq!(tool_extension("applescript"), "applescript");
    assert_eq!(tool_extension("powershell"), "ps1");
    assert_eq!(tool_extension("pwsh"), "ps1");
    assert_eq!(tool_extension("cmd"), "bat");
    assert_eq!(tool_extension("unknown"), "sh");
}

#[test]
fn test_build_final_content_no_modifications() {
    let content = "echo hello";
    let result = build_final_content(content, &None, &None);
    assert_eq!(result, "echo hello");
}

#[test]
fn test_build_final_content_with_prepend() {
    let content = "echo hello";
    let prepend = Some("#!/bin/bash".to_string());
    let result = build_final_content(content, &prepend, &None);
    assert_eq!(result, "#!/bin/bash\necho hello");
}

#[test]
fn test_build_final_content_with_append() {
    let content = "echo hello";
    let append = Some("echo done".to_string());
    let result = build_final_content(content, &None, &append);
    assert_eq!(result, "echo hello\necho done");
}

#[test]
fn test_build_final_content_with_both() {
    let content = "echo hello";
    let prepend = Some("#!/bin/bash\nset -e".to_string());
    let append = Some("echo done".to_string());
    let result = build_final_content(content, &prepend, &append);
    assert_eq!(result, "#!/bin/bash\nset -e\necho hello\necho done");
}

#[test]
fn test_build_final_content_handles_trailing_newlines() {
    let content = "echo hello";
    let prepend = Some("#!/bin/bash\n".to_string());
    let result = build_final_content(content, &prepend, &None);
    assert_eq!(result, "#!/bin/bash\necho hello");
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_run_scriptlet_bash_echo() {
    let scriptlet = Scriptlet::new(
        "Echo Test".to_string(),
        "bash".to_string(),
        "echo hello".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(result.success, "Script should succeed");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("hello"),
        "Expected 'hello' in stdout: {}",
        result.stdout
    );
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_run_scriptlet_bash_with_variable_substitution() {
    let scriptlet = Scriptlet::new(
        "Variable Test".to_string(),
        "bash".to_string(),
        "echo Hello {{name}}".to_string(),
    );

    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "World".to_string());

    let options = ScriptletExecOptions {
        inputs,
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    assert!(
        result.stdout.contains("Hello World"),
        "Expected 'Hello World' in stdout: {}",
        result.stdout
    );
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_run_scriptlet_bash_with_positional_args() {
    let scriptlet = Scriptlet::new(
        "Positional Test".to_string(),
        "bash".to_string(),
        "echo $1 and $2".to_string(),
    );

    let options = ScriptletExecOptions {
        positional_args: vec!["first".to_string(), "second".to_string()],
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    assert!(
        result.stdout.contains("first and second"),
        "Expected 'first and second' in stdout: {}",
        result.stdout
    );
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_run_scriptlet_with_prepend_append() {
    let scriptlet = Scriptlet::new(
        "Prepend Append Test".to_string(),
        "bash".to_string(),
        "echo middle".to_string(),
    );

    let options = ScriptletExecOptions {
        prepend: Some("echo start".to_string()),
        append: Some("echo end".to_string()),
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    let stdout = result.stdout;
    assert!(
        stdout.contains("start"),
        "Should contain 'start': {}",
        stdout
    );
    assert!(
        stdout.contains("middle"),
        "Should contain 'middle': {}",
        stdout
    );
    assert!(stdout.contains("end"), "Should contain 'end': {}", stdout);
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_run_scriptlet_with_cwd() {
    let scriptlet = Scriptlet::new(
        "CWD Test".to_string(),
        "bash".to_string(),
        "pwd".to_string(),
    );

    let options = ScriptletExecOptions {
        cwd: Some(PathBuf::from("/tmp")),
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    // /tmp might be symlinked to /private/tmp on macOS
    assert!(
        result.stdout.contains("/tmp") || result.stdout.contains("/private/tmp"),
        "Expected '/tmp' in stdout: {}",
        result.stdout
    );
}

#[test]
fn test_run_scriptlet_template_returns_content() {
    let scriptlet = Scriptlet::new(
        "Template Test".to_string(),
        "template".to_string(),
        "Hello {{name}}!".to_string(),
    );

    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "World".to_string());

    let options = ScriptletExecOptions {
        inputs,
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "Hello World!");
}

#[test]
fn test_run_scriptlet_with_conditionals() {
    let scriptlet = Scriptlet::new(
        "Conditional Test".to_string(),
        "template".to_string(),
        "{{#if formal}}Dear Sir{{else}}Hey there{{/if}}".to_string(),
    );

    let mut flags = HashMap::new();
    flags.insert("formal".to_string(), true);

    let options = ScriptletExecOptions {
        flags,
        ..Default::default()
    };

    let result = run_scriptlet(&scriptlet, options);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(
        result.stdout.contains("Dear Sir"),
        "Expected 'Dear Sir' in output: {}",
        result.stdout
    );
}

// This test actually opens Finder to /tmp, so it's a system test
#[cfg(all(unix, feature = "system-tests"))]
#[test]
fn test_run_scriptlet_open() {
    // Just test that open doesn't error on a valid path
    // We can't really verify it opens, but we can test the function runs
    let scriptlet = Scriptlet::new(
        "Open Test".to_string(),
        "open".to_string(),
        "/tmp".to_string(),
    );

    let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
    // This should succeed on macOS/Linux with /tmp
    assert!(result.is_ok(), "Expected success, got: {:?}", result);
}

#[test]
fn test_scriptlet_exec_options_default() {
    let options = ScriptletExecOptions::default();
    assert!(options.cwd.is_none());
    assert!(options.prepend.is_none());
    assert!(options.append.is_none());
    assert!(options.inputs.is_empty());
    assert!(options.positional_args.is_empty());
    assert!(options.flags.is_empty());
}

