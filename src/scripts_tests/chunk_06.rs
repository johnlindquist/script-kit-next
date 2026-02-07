// ============================================
// INTEGRATION TESTS: End-to-End Flows
// ============================================

#[test]
fn test_script_struct_creation_and_properties() {
    let script = Script {
        name: "myScript".to_string(),
        path: PathBuf::from("/home/user/.scriptkit/scripts/myScript.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("My custom script".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    };

    assert_eq!(script.name, "myScript");
    assert_eq!(script.extension, "ts");
    assert!(script.description.is_some());
    assert!(script.path.to_string_lossy().contains("myScript"));
}

#[test]
fn test_script_clone_independence() {
    let original = Script {
        name: "original".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("desc".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    };

    let cloned = original.clone();
    assert_eq!(original.name, cloned.name);
    assert_eq!(original.path, cloned.path);
}

#[test]
fn test_scriptlet_clone_independence() {
    let mut original = test_scriptlet("original", "ts", "code");
    original.description = Some("desc".to_string());
    original.shortcut = Some("cmd k".to_string());

    let cloned = original.clone();
    assert_eq!(original.name, cloned.name);
    assert_eq!(original.code, cloned.code);
}

#[test]
fn test_search_multiple_scriptlets() {
    let scriptlets = wrap_scriptlets(vec![
        test_scriptlet_with_desc("Copy", "ts", "copy()", "Copy to clipboard"),
        test_scriptlet_with_desc("Paste", "ts", "paste()", "Paste from clipboard"),
        test_scriptlet_with_desc(
            "Custom Paste",
            "ts",
            "pasteCustom()",
            "Custom paste with format",
        ),
    ]);

    let results = fuzzy_search_scriptlets(&scriptlets, "paste");
    assert_eq!(results.len(), 2); // "Paste" and "Custom Paste"
                                  // "Paste" should rank higher than "Custom Paste"
    assert_eq!(results[0].scriptlet.name, "Paste");
}

#[test]
fn test_unified_search_mixed_results() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "openFile".to_string(),
            path: PathBuf::from("/openFile.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "saveFile".to_string(),
            path: PathBuf::from("/saveFile.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let scriptlets = wrap_scriptlets(vec![test_scriptlet("Open URL", "ts", "open(url)")]);

    let results = fuzzy_search_unified(&scripts, &scriptlets, "open");
    assert_eq!(results.len(), 2); // "openFile" script and "Open URL" scriptlet
}

#[test]
fn test_search_result_name_accessor() {
    let script = SearchResult::Script(ScriptMatch {
        script: Arc::new(Script {
            name: "TestName".to_string(),
            path: PathBuf::from("/test.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        }),
        score: 50,
        filename: "test.ts".to_string(),
        match_indices: MatchIndices::default(),
    });

    assert_eq!(script.name(), "TestName");
}

#[test]
fn test_search_result_description_accessor() {
    let scriptlet = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet: Arc::new(test_scriptlet_with_desc(
            "Test",
            "ts",
            "code",
            "Test Description",
        )),
        score: 75,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });

    assert_eq!(scriptlet.description(), Some("Test Description"));
}

#[test]
fn test_parse_multiple_scriptlets_from_markdown() {
    let markdown = r#"## First Snippet
<!-- description: First desc -->
```ts
first()
```

## Second Snippet
<!-- description: Second desc -->
```bash
second
```

## Third Snippet
```ts
third()
```"#;

    // Simulate splitting and parsing
    let sections: Vec<&str> = markdown.split("## ").collect();
    let mut count = 0;
    for section in sections.iter().skip(1) {
        let full_section = format!("## {}", section);
        if let Some(scriptlet) = parse_scriptlet_section(&full_section, None) {
            count += 1;
            assert!(!scriptlet.name.is_empty());
        }
    }
    assert_eq!(count, 3);
}

#[test]
fn test_fuzzy_search_preserves_vector_order() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "alpha".to_string(),
            path: PathBuf::from("/alpha.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "beta".to_string(),
            path: PathBuf::from("/beta.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "gamma".to_string(),
            path: PathBuf::from("/gamma.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "");
    assert_eq!(results.len(), 3);
    // Empty query should return in name order
    assert_eq!(results[0].script.name, "alpha");
    assert_eq!(results[1].script.name, "beta");
    assert_eq!(results[2].script.name, "gamma");
}

#[test]
fn test_extract_html_metadata_whitespace_handling() {
    let text = "<!--\n  key1:   value1  \n  key2: value2\n-->";
    let metadata = extract_html_comment_metadata(text);
    // Values should be trimmed
    assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_parse_scriptlet_with_html_comment_no_fence() {
    // Test that parse_scriptlet_section requires code block even with metadata
    let section = "## NoCode\n\n<!-- description: Test -->\nJust text";
    let scriptlet = parse_scriptlet_section(section, None);
    assert!(scriptlet.is_none());
}

#[test]
fn test_fuzzy_match_special_characters() {
    assert!(is_fuzzy_match("test-file", "test"));
    assert!(is_fuzzy_match("test.file", "file"));
    assert!(is_fuzzy_match("test_name", "name"));
}

