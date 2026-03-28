use script_kit_gpui::ai::{
    resolve_tab_ai_memory_suggestions, CurrentAppAutomationMemoryIndexEntry,
    TabAiMemorySuggestion,
};
use std::sync::{Mutex, OnceLock};

fn home_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_home_dir<T>(dir: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let _guard = home_env_lock().lock().expect("lock HOME test guard");
    let previous = std::env::var_os("HOME");

    // SAFETY: tests serialize HOME mutation through `home_env_lock`, and the
    // previous value is restored before releasing the lock.
    unsafe {
        std::env::set_var("HOME", dir);
    }

    let result = f();

    // SAFETY: guarded by the same process-wide mutex as the corresponding set.
    unsafe {
        match previous {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_test_recipe(
    bundle_id: &str,
    effective_query: &str,
) -> script_kit_gpui::menu_bar::current_app_commands::CurrentAppCommandRecipe {
    script_kit_gpui::menu_bar::current_app_commands::CurrentAppCommandRecipe {
        schema_version: 1,
        recipe_type: "currentAppCommand".to_string(),
        raw_query: effective_query.to_string(),
        effective_query: effective_query.to_string(),
        suggested_script_name: "test-script".to_string(),
        trace: script_kit_gpui::menu_bar::current_app_commands::CurrentAppIntentTraceReceipt {
            schema_version: 1,
            source: "test".to_string(),
            app_name: "TestApp".to_string(),
            bundle_id: bundle_id.to_string(),
            raw_query: effective_query.to_string(),
            effective_query: effective_query.to_string(),
            normalized_query: effective_query.to_lowercase(),
            top_level_menu_count: 0,
            leaf_entry_count: 0,
            filtered_entries: 0,
            exact_matches: 0,
            action: "generate_script".to_string(),
            selected_entry: None,
            candidates: vec![],
            prompt_receipt: None,
            prompt_preview: None,
        },
        prompt_receipt:
            script_kit_gpui::menu_bar::current_app_commands::CurrentAppScriptPromptReceipt {
                app_name: "TestApp".to_string(),
                bundle_id: bundle_id.to_string(),
                total_menu_items: 0,
                included_menu_items: 0,
                included_user_request: true,
                included_selected_text: false,
                included_browser_url: false,
            },
        prompt: "test prompt".to_string(),
    }
}

fn make_memory_entry(
    bundle_id: &str,
    effective_query: &str,
    slug: &str,
    raw_query: &str,
    provider_id: &str,
    model_id: &str,
) -> CurrentAppAutomationMemoryIndexEntry {
    let recipe = make_test_recipe(bundle_id, effective_query);
    let lookup_key = format!(
        "{}::{}",
        bundle_id.to_lowercase().replace('.', " "),
        effective_query.to_lowercase()
    );
    CurrentAppAutomationMemoryIndexEntry {
        schema_version: 1,
        slug: slug.to_string(),
        script_path: format!("/tmp/test-scripts/{slug}.ts"),
        receipt_path: format!("/tmp/test-scripts/{slug}.scriptkit.json"),
        bundle_id: bundle_id.to_string(),
        app_name: "TestApp".to_string(),
        effective_query: effective_query.to_string(),
        raw_query: raw_query.to_string(),
        prompt: format!("test prompt for {effective_query}"),
        provider_id: provider_id.to_string(),
        model_id: model_id.to_string(),
        lookup_key,
        auto_replay_eligible: true,
        written_at_unix_ms: 1000,
        recipe,
    }
}

fn write_index(dir: &std::path::Path, entries: &[CurrentAppAutomationMemoryIndexEntry]) {
    let scripts_dir = dir.join(".scriptkit").join("scripts");
    std::fs::create_dir_all(&scripts_dir).expect("create scripts dir");
    let index_path = scripts_dir.join(".current-app-automation-memory.json");
    let json = serde_json::to_string_pretty(entries).expect("serialize index");
    std::fs::write(index_path, json).expect("write index");
}

// ---------------------------------------------------------------------------
// Tests: early-return guards
// ---------------------------------------------------------------------------

#[test]
fn returns_empty_when_query_is_empty() {
    let result =
        resolve_tab_ai_memory_suggestions("", Some("com.apple.Safari"), 5).expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_query_is_whitespace() {
    let result = resolve_tab_ai_memory_suggestions("   ", Some("com.apple.Safari"), 5)
        .expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_bundle_id_is_none() {
    let result =
        resolve_tab_ai_memory_suggestions("copy url", None, 5).expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_bundle_id_is_blank() {
    let result =
        resolve_tab_ai_memory_suggestions("copy url", Some("  "), 5).expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_limit_is_zero() {
    let result = resolve_tab_ai_memory_suggestions("copy url", Some("com.apple.Safari"), 0)
        .expect("should succeed");
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// Tests: matching and filtering
// ---------------------------------------------------------------------------

#[test]
fn matches_entries_for_correct_bundle_id() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let safari_entry = make_memory_entry(
        "com.apple.Safari",
        "copy current tab url",
        "copy-url",
        "copy url",
        "openai",
        "gpt-5",
    );
    let vscode_entry = make_memory_entry(
        "com.microsoft.VSCode",
        "copy current tab url",
        "copy-url-vscode",
        "copy url",
        "openai",
        "gpt-5",
    );
    write_index(dir.path(), &[safari_entry, vscode_entry]);

    let results = with_home_dir(dir.path(), || {
        resolve_tab_ai_memory_suggestions("copy current tab url", Some("com.apple.Safari"), 10)
            .expect("should succeed")
    });

    assert!(!results.is_empty(), "should find Safari matches");
    for suggestion in &results {
        assert_eq!(suggestion.bundle_id, "com.apple.Safari");
    }
}

#[test]
fn filters_below_threshold() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let entry = make_memory_entry(
        "com.apple.Safari",
        "summarize this article",
        "summarize-article",
        "summarize this article",
        "openai",
        "gpt-5",
    );
    write_index(dir.path(), &[entry]);

    // Completely unrelated query should not match
    let results = with_home_dir(dir.path(), || {
        resolve_tab_ai_memory_suggestions(
            "force quit application now",
            Some("com.apple.Safari"),
            10,
        )
        .expect("should succeed")
    });

    assert!(
        results.is_empty(),
        "unrelated query should return no suggestions"
    );
}

#[test]
fn sorted_by_descending_score_then_slug() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Two entries for Safari, one exact match and one partial
    let exact_entry = make_memory_entry(
        "com.apple.Safari",
        "copy url",
        "copy-url",
        "copy url",
        "openai",
        "gpt-5",
    );
    let partial_entry = make_memory_entry(
        "com.apple.Safari",
        "copy url and title",
        "copy-url-title",
        "copy url and title",
        "openai",
        "gpt-5",
    );
    write_index(dir.path(), &[exact_entry, partial_entry]);

    let results = with_home_dir(dir.path(), || {
        resolve_tab_ai_memory_suggestions("copy url", Some("com.apple.Safari"), 10)
            .expect("should succeed")
    });

    if results.len() >= 2 {
        assert!(
            results[0].score >= results[1].score,
            "first result score ({}) should be >= second ({})",
            results[0].score,
            results[1].score
        );
    }
}

#[test]
fn truncated_to_limit() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Create 5 entries that all exact-match "copy url"
    let entries: Vec<_> = (0..5)
        .map(|i| {
            make_memory_entry(
                "com.apple.Safari",
                "copy url",
                &format!("copy-url-{i}"),
                "copy url",
                "openai",
                "gpt-5",
            )
        })
        .collect();
    write_index(dir.path(), &entries);

    let results = with_home_dir(dir.path(), || {
        resolve_tab_ai_memory_suggestions("copy url", Some("com.apple.Safari"), 2)
            .expect("should succeed")
    });

    assert!(
        results.len() <= 2,
        "should be truncated to limit 2, got {}",
        results.len()
    );
}

// ---------------------------------------------------------------------------
// Tests: serde contract
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_memory_suggestion_serializes_camel_case() {
    let suggestion = TabAiMemorySuggestion {
        slug: "copy-url".to_string(),
        app_name: "Safari".to_string(),
        bundle_id: "com.apple.Safari".to_string(),
        raw_query: "copy url".to_string(),
        effective_query: "copy current tab url".to_string(),
        provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        score: 0.92,
    };

    let json = serde_json::to_value(&suggestion).expect("serialize");
    assert_eq!(json["slug"], "copy-url");
    assert_eq!(json["appName"], "Safari");
    assert_eq!(json["bundleId"], "com.apple.Safari");
    assert_eq!(json["rawQuery"], "copy url");
    assert_eq!(json["effectiveQuery"], "copy current tab url");
    assert_eq!(json["providerId"], "openai");
    assert_eq!(json["modelId"], "gpt-5");
    let score = json["score"].as_f64().expect("score should be a number");
    assert!(
        (score - 0.92).abs() < 0.001,
        "score should be ~0.92, got {score}"
    );

    // No snake_case keys
    let obj = json.as_object().expect("should be object");
    assert!(
        !obj.contains_key("app_name"),
        "should use camelCase, not snake_case"
    );
    assert!(
        !obj.contains_key("bundle_id"),
        "should use camelCase, not snake_case"
    );
}

#[test]
fn tab_ai_memory_suggestion_roundtrips() {
    let suggestion = TabAiMemorySuggestion {
        slug: "copy-url".to_string(),
        app_name: "Safari".to_string(),
        bundle_id: "com.apple.Safari".to_string(),
        raw_query: "copy url".to_string(),
        effective_query: "copy current tab url".to_string(),
        provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        score: 0.92,
    };

    let json = serde_json::to_string(&suggestion).expect("serialize");
    let parsed: TabAiMemorySuggestion = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(suggestion, parsed);
}

// ---------------------------------------------------------------------------
// Tests: empty index file
// ---------------------------------------------------------------------------

#[test]
fn returns_empty_when_no_index_file_exists() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let results = with_home_dir(dir.path(), || {
        resolve_tab_ai_memory_suggestions("copy url", Some("com.apple.Safari"), 5)
            .expect("should succeed")
    });

    assert!(results.is_empty(), "no index file should return empty vec");
}
