// ============================================
// TYPED METADATA & SCHEMA INTEGRATION TESTS
// ============================================

#[test]
fn test_script_struct_has_typed_fields() {
    // Test that Script struct includes typed_metadata and schema fields
    use crate::metadata_parser::TypedMetadata;
    use crate::schema_parser::{FieldDef, FieldType, Schema};
    use std::collections::HashMap;

    let typed_meta = TypedMetadata {
        name: Some("My Typed Script".to_string()),
        description: Some("A script with typed metadata".to_string()),
        alias: Some("mts".to_string()),
        icon: Some("Star".to_string()),
        ..Default::default()
    };

    let mut input_fields = HashMap::new();
    input_fields.insert(
        "title".to_string(),
        FieldDef {
            field_type: FieldType::String,
            required: true,
            description: Some("The title".to_string()),
            ..Default::default()
        },
    );

    let schema = Schema {
        input: input_fields,
        output: HashMap::new(),
    };

    let script = Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        typed_metadata: Some(typed_meta.clone()),
        schema: Some(schema.clone()),
        ..Default::default()
    };

    // Verify typed_metadata is accessible
    assert!(script.typed_metadata.is_some());
    let meta = script.typed_metadata.as_ref().unwrap();
    assert_eq!(meta.name, Some("My Typed Script".to_string()));
    assert_eq!(meta.alias, Some("mts".to_string()));
    assert_eq!(meta.icon, Some("Star".to_string()));

    // Verify schema is accessible
    assert!(script.schema.is_some());
    let sch = script.schema.as_ref().unwrap();
    assert_eq!(sch.input.len(), 1);
    assert!(sch.input.contains_key("title"));
}

#[test]
fn test_extract_typed_metadata_from_script() {
    // Test that extract_full_metadata correctly parses typed metadata
    let content = r#"
metadata = {
    name: "Create Note",
    description: "Creates a new note in the notes directory",
    author: "John Lindquist",
    alias: "note",
    icon: "File",
    shortcut: "cmd n"
}

const title = await arg("Enter title");
"#;

    let (script_meta, typed_meta, _schema) = extract_full_metadata(content);

    // Typed metadata should be parsed
    assert!(typed_meta.is_some());
    let meta = typed_meta.unwrap();
    assert_eq!(meta.name, Some("Create Note".to_string()));
    assert_eq!(
        meta.description,
        Some("Creates a new note in the notes directory".to_string())
    );
    assert_eq!(meta.alias, Some("note".to_string()));
    assert_eq!(meta.icon, Some("File".to_string()));
    assert_eq!(meta.shortcut, Some("cmd n".to_string()));

    // Script metadata should also be populated from typed
    assert_eq!(script_meta.name, Some("Create Note".to_string()));
    assert_eq!(script_meta.alias, Some("note".to_string()));
}

#[test]
fn test_extract_schema_from_script() {
    use crate::schema_parser::ItemsDef;

    // Test that extract_full_metadata correctly parses schema
    let content = r#"
schema = {
    input: {
        title: { type: "string", required: true, description: "Note title" },
        tags: { type: "array", items: "string" }
    },
    output: {
        path: { type: "string", description: "Path to created file" }
    }
}

const { title, tags } = await input();
"#;

    let (_script_meta, _typed_meta, schema) = extract_full_metadata(content);

    // Schema should be parsed
    assert!(schema.is_some());
    let sch = schema.unwrap();

    // Check input fields
    assert_eq!(sch.input.len(), 2);
    let title_field = sch.input.get("title").unwrap();
    assert!(title_field.required);
    assert_eq!(title_field.description, Some("Note title".to_string()));

    let tags_field = sch.input.get("tags").unwrap();
    assert_eq!(tags_field.items, Some(ItemsDef::Type("string".to_string())));

    // Check output fields
    assert_eq!(sch.output.len(), 1);
    assert!(sch.output.contains_key("path"));
}

#[test]
fn test_fallback_to_comment_metadata() {
    // Test that when no typed metadata exists, we fall back to comment-based metadata
    let content = r#"// Name: My Script
// Description: A script without typed metadata
// Icon: Terminal
// Alias: ms
// Shortcut: opt m

const x = await arg("Pick one");
"#;

    let (script_meta, typed_meta, schema) = extract_full_metadata(content);

    // No typed metadata in this script
    assert!(typed_meta.is_none());
    assert!(schema.is_none());

    // But script metadata should be extracted from comments
    assert_eq!(script_meta.name, Some("My Script".to_string()));
    assert_eq!(
        script_meta.description,
        Some("A script without typed metadata".to_string())
    );
    assert_eq!(script_meta.icon, Some("Terminal".to_string()));
    assert_eq!(script_meta.alias, Some("ms".to_string()));
    assert_eq!(script_meta.shortcut, Some("opt m".to_string()));
}

#[test]
fn test_both_typed_and_comment_prefers_typed() {
    // Test that when both typed metadata AND comment metadata exist,
    // the typed metadata takes precedence
    let content = r#"// Name: Comment Name
// Description: Comment Description
// Alias: cn

metadata = {
    name: "Typed Name",
    description: "Typed Description",
    alias: "tn"
}

const x = await arg("Pick");
"#;

    let (script_meta, typed_meta, _schema) = extract_full_metadata(content);

    // Typed metadata should be present
    assert!(typed_meta.is_some());
    let meta = typed_meta.unwrap();
    assert_eq!(meta.name, Some("Typed Name".to_string()));
    assert_eq!(meta.description, Some("Typed Description".to_string()));
    assert_eq!(meta.alias, Some("tn".to_string()));

    // Script metadata should use typed values (typed takes precedence)
    assert_eq!(script_meta.name, Some("Typed Name".to_string()));
    assert_eq!(
        script_meta.description,
        Some("Typed Description".to_string())
    );
    assert_eq!(script_meta.alias, Some("tn".to_string()));
}

#[test]
fn test_typed_metadata_partial_with_comment_fallback() {
    // Test that typed metadata can be partial and comment metadata fills gaps
    let content = r#"// Name: Comment Name
// Description: Full description
// Icon: Terminal
// Shortcut: opt x

metadata = {
    name: "Typed Name",
    alias: "tn"
}

const x = await arg("Pick");
"#;

    let (script_meta, typed_meta, _schema) = extract_full_metadata(content);

    // Typed metadata is present but partial
    assert!(typed_meta.is_some());
    let meta = typed_meta.unwrap();
    assert_eq!(meta.name, Some("Typed Name".to_string()));
    assert_eq!(meta.alias, Some("tn".to_string()));
    assert!(meta.description.is_none()); // Not in typed
    assert!(meta.icon.is_none()); // Not in typed
    assert!(meta.shortcut.is_none()); // Not in typed

    // Script metadata should use typed for what's available, comments for rest
    assert_eq!(script_meta.name, Some("Typed Name".to_string())); // From typed
    assert_eq!(script_meta.alias, Some("tn".to_string())); // From typed
    assert_eq!(
        script_meta.description,
        Some("Full description".to_string())
    ); // From comment
    assert_eq!(script_meta.icon, Some("Terminal".to_string())); // From comment
    assert_eq!(script_meta.shortcut, Some("opt x".to_string())); // From comment
}

#[test]
fn test_both_metadata_and_schema() {
    // Test extracting both metadata and schema from a single script
    let content = r#"
metadata = {
    name: "Full Featured Script",
    description: "Has both metadata and schema",
    alias: "ffs"
}

schema = {
    input: {
        query: { type: "string", required: true }
    },
    output: {
        result: { type: "string" }
    }
}

const { query } = await input();
"#;

    let (script_meta, typed_meta, schema) = extract_full_metadata(content);

    // Both should be present
    assert!(typed_meta.is_some());
    assert!(schema.is_some());

    // Verify metadata
    let meta = typed_meta.unwrap();
    assert_eq!(meta.name, Some("Full Featured Script".to_string()));
    assert_eq!(meta.alias, Some("ffs".to_string()));

    // Verify schema
    let sch = schema.unwrap();
    assert_eq!(sch.input.len(), 1);
    assert_eq!(sch.output.len(), 1);

    // Script metadata populated
    assert_eq!(script_meta.name, Some("Full Featured Script".to_string()));
}

/// Performance benchmark for get_grouped_results
/// This test verifies that repeated calls with the same filter don't regress performance.
/// It creates realistic data (100 scripts, 50 scriptlets, 20 builtins, 30 apps)
/// and measures the time for 100 repeated calls.
#[test]
fn bench_get_grouped_results_repeated_calls() {
    use std::time::Instant;

    // Create realistic test data
    let scripts: Vec<Arc<Script>> = (0..100)
        .map(|i| {
            Arc::new(Script {
                name: format!("script-{:03}", i),
                path: PathBuf::from(format!("/test/scripts/script-{:03}.ts", i)),
                extension: "ts".to_string(),
                description: Some(format!("Description for script {}", i)),
                ..Default::default()
            })
        })
        .collect();

    let scriptlets: Vec<Arc<Scriptlet>> = (0..50)
        .map(|i| {
            Arc::new(Scriptlet {
                name: format!("snippet-{:02}", i),
                file_path: Some(format!("/test/scriptlets/snippet-{:02}.md", i)),
                tool: "ts".to_string(),
                code: format!("console.log('snippet {}')", i),
                description: Some(format!("Snippet {} description", i)),
                shortcut: None,
                keyword: None,
                group: None,
                command: None,
                alias: None,
            })
        })
        .collect();

    let builtins: Vec<crate::builtins::BuiltInEntry> = (0..20)
        .map(|i| crate::builtins::BuiltInEntry {
            id: format!("builtin-{:02}", i),
            name: format!("builtin-{:02}", i),
            description: format!("Built-in {} description", i),
            keywords: vec![format!("keyword{}", i)],
            feature: crate::builtins::BuiltInFeature::ClipboardHistory,
            icon: None,
            group: crate::builtins::BuiltInGroup::Core,
        })
        .collect();

    let apps: Vec<crate::app_launcher::AppInfo> = (0..30)
        .map(|i| crate::app_launcher::AppInfo {
            name: format!("App {:02}", i),
            path: PathBuf::from(format!("/Applications/App{:02}.app", i)),
            bundle_id: Some(format!("com.test.app{:02}", i)),
            icon: None,
        })
        .collect();

    let frecency_store = crate::frecency::FrecencyStore::new();

    // Warm up
    let _ = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Benchmark: 100 calls with empty filter (grouped mode)
    let start = Instant::now();
    for _ in 0..100 {
        let _ = get_grouped_results(
            &scripts,
            &scriptlets,
            &builtins,
            &apps,
            &frecency_store,
            "",
            &SuggestedConfig::default(),
            &[],
            None,
        );
    }
    let empty_filter_duration = start.elapsed();

    // Benchmark: 100 calls with filter (search mode)
    let start = Instant::now();
    for _ in 0..100 {
        let _ = get_grouped_results(
            &scripts,
            &scriptlets,
            &builtins,
            &apps,
            &frecency_store,
            "scr",
            &SuggestedConfig::default(),
            &[],
            None,
        );
    }
    let search_filter_duration = start.elapsed();

    // Log results (visible with cargo test -- --nocapture)
    println!("\n=== get_grouped_results Performance Benchmark ===");
    println!(
        "Data: {} scripts, {} scriptlets, {} builtins, {} apps",
        scripts.len(),
        scriptlets.len(),
        builtins.len(),
        apps.len()
    );
    println!(
        "Empty filter (100 calls): {:?} ({:.2}ms per call)",
        empty_filter_duration,
        empty_filter_duration.as_secs_f64() * 10.0
    );
    println!(
        "Search filter 'scr' (100 calls): {:?} ({:.2}ms per call)",
        search_filter_duration,
        search_filter_duration.as_secs_f64() * 10.0
    );
    println!("===============================================\n");

    // Performance assertions - each call should be under 5ms
    // (with caching, repeated calls should be nearly instant)
    let max_per_call_ms = 5.0;
    assert!(
        empty_filter_duration.as_secs_f64() * 10.0 < max_per_call_ms,
        "Empty filter calls too slow: {:.2}ms per call (max: {}ms)",
        empty_filter_duration.as_secs_f64() * 10.0,
        max_per_call_ms
    );
    assert!(
        search_filter_duration.as_secs_f64() * 10.0 < max_per_call_ms,
        "Search filter calls too slow: {:.2}ms per call (max: {}ms)",
        search_filter_duration.as_secs_f64() * 10.0,
        max_per_call_ms
    );
}

