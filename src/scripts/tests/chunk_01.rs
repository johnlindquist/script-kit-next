use super::*;
use crate::config::SuggestedConfig;
use std::sync::Arc;

/// Helper to wrap Vec<Script> into Vec<Arc<Script>> for tests
fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
    scripts.into_iter().map(Arc::new).collect()
}

/// Helper to wrap Vec<Scriptlet> into Vec<Arc<Scriptlet>> for tests
fn wrap_scriptlets(scriptlets: Vec<Scriptlet>) -> Vec<Arc<Scriptlet>> {
    scriptlets.into_iter().map(Arc::new).collect()
}

/// Helper to create a test Scriptlet with minimal required fields
fn test_scriptlet(name: &str, tool: &str, code: &str) -> Scriptlet {
    Scriptlet {
        name: name.to_string(),
        description: None,
        code: code.to_string(),
        tool: tool.to_string(),
        shortcut: None,
        keyword: None,
        group: None,
        file_path: None,
        command: None,
        alias: None,
    }
}

/// Helper to create a test Scriptlet with description
fn test_scriptlet_with_desc(name: &str, tool: &str, code: &str, desc: &str) -> Scriptlet {
    Scriptlet {
        name: name.to_string(),
        description: Some(desc.to_string()),
        code: code.to_string(),
        tool: tool.to_string(),
        shortcut: None,
        keyword: None,
        group: None,
        file_path: None,
        command: None,
        alias: None,
    }
}

// ============================================
// LOAD_SCRIPTLETS INTEGRATION TESTS
// ============================================

#[test]
fn test_load_scriptlets_returns_vec() {
    // load_scriptlets should return a Vec even if directory doesn't exist
    let scriptlets = load_scriptlets();
    // Just verify it returns without panicking
    let _ = scriptlets.len();
}

#[test]
fn test_extract_kit_from_path_nested() {
    use std::path::Path;
    // kit_root is ~/.scriptkit, not home directory
    let kit_root = Path::new("/Users/test/.scriptkit");

    // Nested kit path: ~/.scriptkit/my-kit/scriptlets/file.md -> kit = "my-kit"
    let nested_path = Path::new("/Users/test/.scriptkit/my-kit/scriptlets/file.md");
    let kit = extract_kit_from_path(nested_path, kit_root);
    assert_eq!(kit, Some("my-kit".to_string()));
}

#[test]
fn test_extract_kit_from_path_main_kit() {
    use std::path::Path;
    // kit_root is ~/.scriptkit, not home directory
    let kit_root = Path::new("/Users/test/.scriptkit");

    // Main kit path: ~/.scriptkit/main/scriptlets/file.md -> kit = "main"
    let main_path = Path::new("/Users/test/.scriptkit/main/scriptlets/file.md");
    let kit = extract_kit_from_path(main_path, kit_root);
    assert_eq!(kit, Some("main".to_string()));
}

#[test]
fn test_build_scriptlet_file_path() {
    use std::path::Path;
    let md_path = Path::new("/Users/test/.scriptkit/main/scriptlets/my-scripts.md");
    let result = build_scriptlet_file_path(md_path, "my-slug");
    assert_eq!(
        result,
        "/Users/test/.scriptkit/main/scriptlets/my-scripts.md#my-slug"
    );
}

#[test]
fn test_read_scriptlets_from_file_nonexistent() {
    use std::path::Path;
    // Non-existent file should return empty vec
    let path = Path::new("/nonexistent/path/to/file.md");
    let scriptlets = read_scriptlets_from_file(path);
    assert!(scriptlets.is_empty());
}

#[test]
fn test_read_scriptlets_from_file_not_markdown() {
    use std::path::Path;
    // Non-markdown file should return empty vec
    let path = Path::new("/some/path/to/file.ts");
    let scriptlets = read_scriptlets_from_file(path);
    assert!(scriptlets.is_empty());
}

#[test]
fn test_scriptlet_new_fields() {
    // Verify the new Scriptlet struct fields work
    let scriptlet = Scriptlet {
        name: "Test".to_string(),
        description: Some("Desc".to_string()),
        code: "code".to_string(),
        tool: "ts".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("My Group".to_string()),
        file_path: Some("/path/to/file.md#test".to_string()),
        command: Some("test".to_string()),
        alias: None,
    };

    assert_eq!(scriptlet.group, Some("My Group".to_string()));
    assert_eq!(
        scriptlet.file_path,
        Some("/path/to/file.md#test".to_string())
    );
    assert_eq!(scriptlet.command, Some("test".to_string()));
}

