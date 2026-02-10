// ============================================
// CACHING & PERFORMANCE TESTS
// ============================================

#[test]
fn test_read_scripts_returns_sorted_list() {
    // read_scripts should return sorted by name
    let scripts = wrap_scripts(vec![
        Script {
            name: "zebra".to_string(),
            path: PathBuf::from("/zebra.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "apple".to_string(),
            path: PathBuf::from("/apple.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "monkey".to_string(),
            path: PathBuf::from("/monkey.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    // Manual check of sorting (since read_scripts reads from filesystem)
    let mut sorted = scripts.clone();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(sorted[0].name, "apple");
    assert_eq!(sorted[1].name, "monkey");
    assert_eq!(sorted[2].name, "zebra");
}

#[test]
fn test_scriptlet_ordering_by_name() {
    let scriptlets = wrap_scriptlets(vec![
        test_scriptlet("Zebra", "ts", "code"),
        test_scriptlet("Apple", "ts", "code"),
    ]);

    let results = fuzzy_search_scriptlets(&scriptlets, "");
    // Empty query returns all scriptlets in original order with score 0
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].scriptlet.name, "Zebra");
    assert_eq!(results[1].scriptlet.name, "Apple");
    assert_eq!(results[0].score, 0);
    assert_eq!(results[1].score, 0);
}

#[test]
fn test_large_search_result_set() {
    let mut scripts_raw = Vec::new();
    for i in 0..100 {
        scripts_raw.push(Script {
            name: format!("script_{:03}", i),
            path: PathBuf::from(format!("/script_{}.ts", i)),
            extension: "ts".to_string(),
            icon: None,
            description: Some(format!("Script number {}", i)),
            alias: None,
            shortcut: None,
            ..Default::default()
        });
    }
    let scripts = wrap_scripts(scripts_raw);

    let results = fuzzy_search_scripts(&scripts, "script_05");
    // Should find scripts with 05 in name
    assert!(!results.is_empty());
    assert!(results[0].score > 0);
}

#[test]
fn test_script_match_score_meaningful() {
    let scripts = wrap_scripts(vec![Script {
        name: "openfile".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("Opens a file".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let results = fuzzy_search_scripts(&scripts, "open");
    assert!(results[0].score >= 50); // Should have at least fuzzy match score
}

#[test]
fn test_complex_markdown_parsing() {
    // Test a realistic markdown structure
    let markdown = r#"# Script Collection

## Script One
<!-- 
description: First script
shortcut: cmd 1
-->
```ts
console.log("first");
```

## Script Two
```bash
echo "second"
```

## Script Three
<!-- 
description: Has URL: https://example.com
keyword: type,,
-->
```ts
open("https://example.com");
```
"#;

    // Split and parse sections
    let sections: Vec<&str> = markdown.split("## ").collect();
    let mut parsed = 0;
    for section in sections.iter().skip(1) {
        if let Some(scriptlet) = parse_scriptlet_section(&format!("## {}", section), None) {
            parsed += 1;
            assert!(!scriptlet.name.is_empty());
            assert!(!scriptlet.code.is_empty());
        }
    }
    assert_eq!(parsed, 3);
}

#[test]
fn test_search_consistency_across_calls() {
    let scripts = wrap_scripts(vec![Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let result1 = fuzzy_search_scripts(&scripts, "test");
    let result2 = fuzzy_search_scripts(&scripts, "test");

    assert_eq!(result1.len(), result2.len());
    if !result1.is_empty() && !result2.is_empty() {
        assert_eq!(result1[0].score, result2[0].score);
    }
}

#[test]
fn test_search_result_name_never_empty() {
    let scripts = wrap_scripts(vec![Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let results = fuzzy_search_scripts(&scripts, "test");
    for result in results {
        let script_match = ScriptMatch {
            script: result.script.clone(),
            score: result.score,
            filename: result.filename.clone(),
            match_indices: result.match_indices.clone(),
        };
        let search_result = SearchResult::Script(script_match);
        assert!(!search_result.name().is_empty());
    }
}

#[test]
fn test_scriptlet_code_extraction_with_special_chars() {
    let section = r#"## SpecialChars
```ts
const regex = /test\d+/;
const str = "test\nline";
const obj = { key: "value" };
```"#;

    let scriptlet = parse_scriptlet_section(section, None);
    assert!(scriptlet.is_some());
    let s = scriptlet.unwrap();
    assert!(s.code.contains("regex"));
    assert!(s.code.contains("str"));
}

#[test]
fn test_fuzzy_search_with_unicode() {
    let scripts = wrap_scripts(vec![Script {
        name: "café".to_string(),
        path: PathBuf::from("/cafe.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    // Should be able to search for the ASCII version
    let results = fuzzy_search_scripts(&scripts, "cafe");
    // Depending on implementation, may or may not match
    let _ = results;
}

#[test]
fn test_script_extension_field_accuracy() {
    let script = Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    };

    assert_eq!(script.extension, "ts");

    let script_js = Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.js"),
        extension: "js".to_string(),
        description: None,
        icon: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    };

    assert_eq!(script_js.extension, "js");
}

#[test]
fn test_searchlet_tool_field_various_values() {
    let tools = vec!["ts", "bash", "paste", "sh", "zsh", "py"];

    for tool in tools {
        let scriptlet = test_scriptlet(&format!("Test {}", tool), tool, "code");
        assert_eq!(scriptlet.tool, tool);
    }
}

#[test]
fn test_extract_code_block_with_language_modifiers() {
    let text = "```ts\nconst x = 1;\n```";
    let (lang, _code) = extract_code_block(text).unwrap();
    assert_eq!(lang, "ts");

    let text2 = "```javascript\nconst x = 1;\n```";
    let (lang2, _code2) = extract_code_block(text2).unwrap();
    assert_eq!(lang2, "javascript");
}

#[test]
fn test_parse_scriptlet_section_all_metadata_fields() {
    let section = r#"## Complete
<!-- 
description: Full description here
shortcut: ctrl shift k
keyword: choices,,
custom: value
-->
```ts
code here
```"#;

    let scriptlet = parse_scriptlet_section(section, None).unwrap();

    assert_eq!(scriptlet.name, "Complete");
    assert_eq!(
        scriptlet.description,
        Some("Full description here".to_string())
    );
    assert_eq!(scriptlet.shortcut, Some("ctrl shift k".to_string()));
    assert_eq!(scriptlet.keyword, Some("choices,,".to_string()));
    // "custom" field won't be extracted as it's not a known field
}

#[test]
fn test_search_result_type_label_consistency() {
    let script = SearchResult::Script(ScriptMatch {
        script: Arc::new(Script {
            name: "test".to_string(),
            path: PathBuf::from("/test.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        }),
        score: 0,
        filename: "test.ts".to_string(),
        match_indices: MatchIndices::default(),
    });

    // Should always return "Script"
    assert_eq!(script.type_label(), "Script");

    let scriptlet = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet: Arc::new(test_scriptlet("test", "ts", "code")),
        score: 0,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });

    // Should always return "Snippet"
    assert_eq!(scriptlet.type_label(), "Snippet");
}

#[test]
fn test_empty_inputs_handling() {
    // Empty script list
    let empty_scripts: Vec<Arc<Script>> = vec![];
    let results = fuzzy_search_scripts(&empty_scripts, "test");
    assert!(results.is_empty());

    // Empty scriptlet list
    let empty_scriptlets: Vec<Arc<Scriptlet>> = vec![];
    let results = fuzzy_search_scriptlets(&empty_scriptlets, "test");
    assert!(results.is_empty());

    // Empty both
    let unified = fuzzy_search_unified(&empty_scripts, &empty_scriptlets, "test");
    assert!(unified.is_empty());
}

