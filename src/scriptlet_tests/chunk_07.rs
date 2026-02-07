// ========================================
// Bundle Frontmatter Tests
// ========================================

#[test]
fn test_parse_bundle_frontmatter_basic() {
    let content = r#"---
name: Test Bundle
description: A test bundle
---

## Script
```bash
echo test
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_some());

    let fm = fm.unwrap();
    assert_eq!(fm.name, Some("Test Bundle".to_string()));
    assert_eq!(fm.description, Some("A test bundle".to_string()));
}

#[test]
fn test_parse_bundle_frontmatter_with_icon() {
    let content = r#"---
icon: Star
---

## Script
```bash
echo
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_some());
    assert_eq!(fm.unwrap().icon, Some("Star".to_string()));
}

#[test]
fn test_parse_bundle_frontmatter_no_frontmatter() {
    let content = r#"## Script Without Frontmatter

```bash
echo test
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_none());
}

#[test]
fn test_parse_bundle_frontmatter_unclosed() {
    // Frontmatter without closing ---
    let content = r#"---
name: Unclosed
author: Test

## Script
```bash
echo
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_none()); // Should fail to parse
}

// ========================================
// Icon Resolution Tests
// ========================================

#[test]
fn test_tool_type_to_icon_shells() {
    assert_eq!(tool_type_to_icon("bash"), "terminal");
    assert_eq!(tool_type_to_icon("zsh"), "terminal");
    assert_eq!(tool_type_to_icon("sh"), "terminal");
    assert_eq!(tool_type_to_icon("fish"), "terminal");
}

#[test]
fn test_tool_type_to_icon_languages() {
    assert_eq!(tool_type_to_icon("python"), "snake");
    assert_eq!(tool_type_to_icon("ruby"), "gem");
    assert_eq!(tool_type_to_icon("ts"), "file-code");
    assert_eq!(tool_type_to_icon("js"), "file-code");
}

#[test]
fn test_tool_type_to_icon_actions() {
    assert_eq!(tool_type_to_icon("open"), "external-link");
    assert_eq!(tool_type_to_icon("paste"), "clipboard");
    assert_eq!(tool_type_to_icon("type"), "keyboard");
    assert_eq!(tool_type_to_icon("edit"), "edit");
}

#[test]
fn test_tool_type_to_icon_unknown() {
    assert_eq!(tool_type_to_icon("unknown_tool"), "file");
}

#[test]
fn test_resolve_scriptlet_icon_metadata_priority() {
    let mut metadata = ScriptletMetadata::default();
    metadata
        .extra
        .insert("icon".to_string(), "custom-icon".to_string());

    let fm = BundleFrontmatter {
        icon: Some("bundle-icon".to_string()),
        ..Default::default()
    };

    // Metadata icon should take priority
    let icon = resolve_scriptlet_icon(&metadata, Some(&fm), "bash");
    assert_eq!(icon, "custom-icon");
}

#[test]
fn test_resolve_scriptlet_icon_frontmatter_fallback() {
    let metadata = ScriptletMetadata::default(); // No icon in metadata

    let fm = BundleFrontmatter {
        icon: Some("bundle-icon".to_string()),
        ..Default::default()
    };

    // Frontmatter should be used when no metadata icon
    let icon = resolve_scriptlet_icon(&metadata, Some(&fm), "bash");
    assert_eq!(icon, "bundle-icon");
}

#[test]
fn test_resolve_scriptlet_icon_tool_fallback() {
    let metadata = ScriptletMetadata::default();

    // No frontmatter icon either
    let icon = resolve_scriptlet_icon(&metadata, None, "python");
    assert_eq!(icon, "snake");
}

