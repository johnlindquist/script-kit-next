// ============================================================
// Tool Constants Verification Tests
// ============================================================

#[test]
fn test_valid_tools_includes_all_special_tools() {
    use crate::scriptlets::VALID_TOOLS;

    assert!(
        VALID_TOOLS.contains(&"template"),
        "VALID_TOOLS should contain 'template'"
    );
    assert!(
        VALID_TOOLS.contains(&"transform"),
        "VALID_TOOLS should contain 'transform'"
    );
    assert!(
        VALID_TOOLS.contains(&"open"),
        "VALID_TOOLS should contain 'open'"
    );
    assert!(
        VALID_TOOLS.contains(&"edit"),
        "VALID_TOOLS should contain 'edit'"
    );
    assert!(
        VALID_TOOLS.contains(&"paste"),
        "VALID_TOOLS should contain 'paste'"
    );
    assert!(
        VALID_TOOLS.contains(&"type"),
        "VALID_TOOLS should contain 'type'"
    );
    assert!(
        VALID_TOOLS.contains(&"submit"),
        "VALID_TOOLS should contain 'submit'"
    );
}

#[test]
fn test_valid_tools_includes_all_interpreter_tools() {
    use crate::scriptlets::VALID_TOOLS;

    assert!(
        VALID_TOOLS.contains(&"python"),
        "VALID_TOOLS should contain 'python'"
    );
    assert!(
        VALID_TOOLS.contains(&"ruby"),
        "VALID_TOOLS should contain 'ruby'"
    );
    assert!(
        VALID_TOOLS.contains(&"perl"),
        "VALID_TOOLS should contain 'perl'"
    );
    assert!(
        VALID_TOOLS.contains(&"php"),
        "VALID_TOOLS should contain 'php'"
    );
    assert!(
        VALID_TOOLS.contains(&"node"),
        "VALID_TOOLS should contain 'node'"
    );
    assert!(
        VALID_TOOLS.contains(&"applescript"),
        "VALID_TOOLS should contain 'applescript'"
    );
}

#[test]
fn test_valid_tools_includes_all_typescript_tools() {
    use crate::scriptlets::VALID_TOOLS;

    assert!(
        VALID_TOOLS.contains(&"kit"),
        "VALID_TOOLS should contain 'kit'"
    );
    assert!(
        VALID_TOOLS.contains(&"ts"),
        "VALID_TOOLS should contain 'ts'"
    );
    assert!(
        VALID_TOOLS.contains(&"js"),
        "VALID_TOOLS should contain 'js'"
    );
    assert!(
        VALID_TOOLS.contains(&"bun"),
        "VALID_TOOLS should contain 'bun'"
    );
    assert!(
        VALID_TOOLS.contains(&"deno"),
        "VALID_TOOLS should contain 'deno'"
    );
}

