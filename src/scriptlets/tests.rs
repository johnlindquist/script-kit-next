include!("tests/chunk_01.rs");
include!("tests/chunk_02.rs");
include!("tests/chunk_03.rs");
include!("tests/chunk_04.rs");
include!("tests/chunk_05.rs");
include!("tests/chunk_06.rs");
include!("tests/chunk_07.rs");
include!("tests/chunk_08.rs");
include!("tests/chunk_09.rs");

// ============================================================================
// normalize_scriptlet_tool tests
// ============================================================================

#[test]
fn test_normalize_tool_colon_maps_to_kit() {
    assert_eq!(super::normalize_scriptlet_tool("tool:quick-note"), "kit");
    assert_eq!(super::normalize_scriptlet_tool("tool:date"), "kit");
    assert_eq!(super::normalize_scriptlet_tool("Tool:MyTool"), "kit");
}

#[test]
fn test_normalize_template_colon_maps_to_template() {
    assert_eq!(super::normalize_scriptlet_tool("template:date"), "template");
    assert_eq!(
        super::normalize_scriptlet_tool("Template:Greeting"),
        "template"
    );
}

#[test]
fn test_normalize_bare_tool_maps_to_kit() {
    assert_eq!(super::normalize_scriptlet_tool("tool"), "kit");
}

#[test]
fn test_normalize_bare_template_unchanged() {
    assert_eq!(super::normalize_scriptlet_tool("template"), "template");
}

#[test]
fn test_normalize_empty_defaults_to_ts() {
    assert_eq!(super::normalize_scriptlet_tool(""), "ts");
    assert_eq!(super::normalize_scriptlet_tool("  "), "ts");
}

#[test]
fn test_normalize_plain_tools_unchanged() {
    assert_eq!(super::normalize_scriptlet_tool("bash"), "bash");
    assert_eq!(super::normalize_scriptlet_tool("ts"), "ts");
    assert_eq!(super::normalize_scriptlet_tool("kit"), "kit");
    assert_eq!(super::normalize_scriptlet_tool("paste"), "paste");
    assert_eq!(super::normalize_scriptlet_tool("Python"), "python");
}

#[test]
fn test_is_shell_with_normalized_tool() {
    let scriptlet = super::Scriptlet::new("test".into(), "tool:my-script".into(), "echo hi".into());
    assert!(
        !scriptlet.is_shell(),
        "tool:name should normalize to kit, not shell"
    );

    let shell_scriptlet = super::Scriptlet::new("test".into(), "bash".into(), "echo hi".into());
    assert!(shell_scriptlet.is_shell());
}

#[test]
fn test_is_valid_tool_with_normalized_tool() {
    let scriptlet = super::Scriptlet::new("test".into(), "tool:quick-note".into(), "code".into());
    assert!(
        scriptlet.is_valid_tool(),
        "tool:name should normalize to kit which is valid"
    );

    let template = super::Scriptlet::new("test".into(), "template:date".into(), "{{date}}".into());
    assert!(
        template.is_valid_tool(),
        "template:name should normalize to template which is valid"
    );
}
