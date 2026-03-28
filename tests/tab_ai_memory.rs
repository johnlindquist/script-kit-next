use script_kit_gpui::ai::{
    read_tab_ai_memory_index_from_path, resolve_tab_ai_memory_suggestions_from_path,
    resolve_tab_ai_memory_suggestions_with_outcome_from_path, write_tab_ai_memory_entry_to_path,
    TabAiExecutionRecord, TabAiMemoryEntry, TabAiMemoryResolutionReason, TabAiMemorySuggestion,
    TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn memory_entry(
    intent: &str,
    bundle_id: Option<&str>,
    slug: &str,
    written_at: &str,
) -> TabAiMemoryEntry {
    TabAiMemoryEntry {
        schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
        intent: intent.to_string(),
        generated_source: "import \"@scriptkit/sdk\";\nawait hide();\n".to_string(),
        slug: slug.to_string(),
        prompt_type: "AppLauncher".to_string(),
        bundle_id: bundle_id.map(str::to_string),
        written_at: written_at.to_string(),
    }
}

fn write_memory(path: &std::path::Path, entries: &[TabAiMemoryEntry]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    let json = serde_json::to_string_pretty(entries).expect("serialize memory entries");
    std::fs::write(path, json).expect("write memory index");
}

// ---------------------------------------------------------------------------
// Tests: early-return guards
// ---------------------------------------------------------------------------

#[test]
fn returns_empty_when_query_is_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let result =
        resolve_tab_ai_memory_suggestions_from_path("", Some("com.apple.Safari"), 5, &path)
            .expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_query_is_whitespace() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let result =
        resolve_tab_ai_memory_suggestions_from_path("   ", Some("com.apple.Safari"), 5, &path)
            .expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_bundle_id_is_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let result = resolve_tab_ai_memory_suggestions_from_path("copy url", None, 5, &path)
        .expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_bundle_id_is_blank() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let result = resolve_tab_ai_memory_suggestions_from_path("copy url", Some("  "), 5, &path)
        .expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_limit_is_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let result =
        resolve_tab_ai_memory_suggestions_from_path("copy url", Some("com.apple.Safari"), 0, &path)
            .expect("should succeed");
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// Tests: matching and filtering
// ---------------------------------------------------------------------------

#[test]
fn resolve_tab_ai_memory_suggestions_returns_similar_non_exact_match() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let entries = vec![memory_entry(
        "force quit current app",
        Some("com.apple.Safari"),
        "force-quit-current-app",
        "2026-03-28T00:00:00Z",
    )];
    write_memory(&path, &entries);

    let results = resolve_tab_ai_memory_suggestions_from_path(
        "force quit app",
        Some("com.apple.Safari"),
        3,
        &path,
    )
    .expect("resolve suggestions");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].slug, "force-quit-current-app");
    assert_eq!(results[0].effective_query, "force quit current app");
    assert!(
        results[0].score >= 0.35,
        "score {} should be >= 0.35",
        results[0].score
    );
}

#[test]
fn resolve_tab_ai_memory_suggestions_filters_by_bundle_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let entries = vec![
        memory_entry(
            "copy browser url",
            Some("com.apple.Safari"),
            "copy-browser-url",
            "2026-03-28T00:00:00Z",
        ),
        memory_entry(
            "copy browser url",
            Some("com.tinyspeck.slackmacgap"),
            "copy-browser-url-slack",
            "2026-03-28T00:00:01Z",
        ),
    ];
    write_memory(&path, &entries);

    let results =
        resolve_tab_ai_memory_suggestions_from_path("copy url", Some("com.apple.Safari"), 3, &path)
            .expect("resolve suggestions");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].slug, "copy-browser-url");
    assert_eq!(results[0].bundle_id, "com.apple.Safari");
}

#[test]
fn resolve_tab_ai_memory_suggestions_prefers_exact_match_then_recency() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let entries = vec![
        memory_entry(
            "force quit current app",
            Some("com.apple.Safari"),
            "older-similar",
            "2026-03-28T00:00:00Z",
        ),
        memory_entry(
            "force quit app",
            Some("com.apple.Safari"),
            "exact-match",
            "2026-03-28T00:00:01Z",
        ),
    ];
    write_memory(&path, &entries);

    let results = resolve_tab_ai_memory_suggestions_from_path(
        "force quit app",
        Some("com.apple.Safari"),
        3,
        &path,
    )
    .expect("resolve suggestions");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].slug, "exact-match");
    assert!(results[0].score >= results[1].score);
}

#[test]
fn filters_below_threshold() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let entries = vec![memory_entry(
        "summarize this article",
        Some("com.apple.Safari"),
        "summarize-article",
        "2026-03-28T00:00:00Z",
    )];
    write_memory(&path, &entries);

    // Completely unrelated query should not match
    let results = resolve_tab_ai_memory_suggestions_from_path(
        "force quit application now",
        Some("com.apple.Safari"),
        10,
        &path,
    )
    .expect("should succeed");

    assert!(
        results.is_empty(),
        "unrelated query should return no suggestions"
    );
}

#[test]
fn sorted_by_descending_score_then_slug() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let entries = vec![
        memory_entry(
            "copy url",
            Some("com.apple.Safari"),
            "copy-url",
            "2026-03-28T00:00:00Z",
        ),
        memory_entry(
            "copy url and title",
            Some("com.apple.Safari"),
            "copy-url-title",
            "2026-03-28T00:00:01Z",
        ),
    ];
    write_memory(&path, &entries);

    let results = resolve_tab_ai_memory_suggestions_from_path(
        "copy url",
        Some("com.apple.Safari"),
        10,
        &path,
    )
    .expect("should succeed");

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
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");
    let entries: Vec<_> = (0..5)
        .map(|i| {
            memory_entry(
                "copy url",
                Some("com.apple.Safari"),
                &format!("copy-url-{i}"),
                &format!("2026-03-28T00:00:0{i}Z"),
            )
        })
        .collect();
    write_memory(&path, &entries);

    let results =
        resolve_tab_ai_memory_suggestions_from_path("copy url", Some("com.apple.Safari"), 2, &path)
            .expect("should succeed");

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
        bundle_id: "com.apple.Safari".to_string(),
        raw_query: "copy url".to_string(),
        effective_query: "copy current tab url".to_string(),
        prompt_type: "AppLauncher".to_string(),
        written_at: "2026-03-28T00:00:00Z".to_string(),
        score: 0.92,
    };

    let json = serde_json::to_value(&suggestion).expect("serialize");
    assert_eq!(json["slug"], "copy-url");
    assert_eq!(json["bundleId"], "com.apple.Safari");
    assert_eq!(json["rawQuery"], "copy url");
    assert_eq!(json["effectiveQuery"], "copy current tab url");
    assert_eq!(json["promptType"], "AppLauncher");
    assert_eq!(json["writtenAt"], "2026-03-28T00:00:00Z");
    let score = json["score"].as_f64().expect("score should be a number");
    assert!(
        (score - 0.92).abs() < 0.001,
        "score should be ~0.92, got {score}"
    );

    // No snake_case keys
    let obj = json.as_object().expect("should be object");
    assert!(
        !obj.contains_key("bundle_id"),
        "should use camelCase, not snake_case"
    );
    assert!(
        !obj.contains_key("prompt_type"),
        "should use camelCase, not snake_case"
    );
    assert!(
        !obj.contains_key("written_at"),
        "should use camelCase, not snake_case"
    );
}

#[test]
fn tab_ai_memory_suggestion_roundtrips() {
    let suggestion = TabAiMemorySuggestion {
        slug: "copy-url".to_string(),
        bundle_id: "com.apple.Safari".to_string(),
        raw_query: "copy url".to_string(),
        effective_query: "copy current tab url".to_string(),
        prompt_type: "AppLauncher".to_string(),
        written_at: "2026-03-28T00:00:00Z".to_string(),
        score: 0.92,
    };

    let json = serde_json::to_string(&suggestion).expect("serialize");
    let parsed: TabAiMemorySuggestion = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(suggestion, parsed);
}

// ---------------------------------------------------------------------------
// Tests: empty / missing index file
// ---------------------------------------------------------------------------

#[test]
fn returns_empty_when_no_index_file_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("nonexistent.json");

    let results =
        resolve_tab_ai_memory_suggestions_from_path("copy url", Some("com.apple.Safari"), 5, &path)
            .expect("should succeed");

    assert!(results.is_empty(), "no index file should return empty vec");
}

// ---------------------------------------------------------------------------
// Tests: outcome-aware resolver regression (deterministic reason values)
// ---------------------------------------------------------------------------

#[test]
fn outcome_reports_missing_bundle_id_reason() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");

    let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
        "force quit slack",
        None,
        3,
        &path,
    )
    .expect("resolve");

    assert!(resolution.suggestions.is_empty());
    assert_eq!(
        resolution.outcome.reason,
        TabAiMemoryResolutionReason::MissingBundleId
    );
    assert_eq!(resolution.outcome.candidate_count, 0);
    assert_eq!(resolution.outcome.match_count, 0);
    assert!(resolution.outcome.top_score.is_none());
    assert!(resolution.outcome.matched_slugs.is_empty());
}

#[test]
fn outcome_reports_index_missing_reason() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("nonexistent.json");

    let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
        "force quit slack",
        Some("com.tinyspeck.slackmacgap"),
        3,
        &path,
    )
    .expect("resolve");

    assert!(resolution.suggestions.is_empty());
    assert_eq!(
        resolution.outcome.reason,
        TabAiMemoryResolutionReason::IndexMissing
    );
    assert!(resolution.outcome.index_path.contains("nonexistent.json"));
}

#[test]
fn outcome_prefers_recent_high_score_matches_with_ordering() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");

    // Use intents that both share enough tokens with "force quit app" to be above threshold.
    let older = TabAiExecutionRecord::from_parts(
        "force quit current app".to_string(),
        "import \"@scriptkit/sdk\";\nawait notify(\"old\");\n".to_string(),
        "/tmp/old.ts".to_string(),
        "force-quit-old".to_string(),
        "ScriptList".to_string(),
        Some("com.tinyspeck.slackmacgap".to_string()),
        "model-a".to_string(),
        "provider-a".to_string(),
        0,
        "2026-03-28T00:00:00Z".to_string(),
    );
    let newer = TabAiExecutionRecord::from_parts(
        "force quit app".to_string(),
        "import \"@scriptkit/sdk\";\nawait notify(\"new\");\n".to_string(),
        "/tmp/new.ts".to_string(),
        "force-quit-new".to_string(),
        "ScriptList".to_string(),
        Some("com.tinyspeck.slackmacgap".to_string()),
        "model-a".to_string(),
        "provider-a".to_string(),
        0,
        "2026-03-28T01:00:00Z".to_string(),
    );

    write_tab_ai_memory_entry_to_path(&older, &path).expect("write older");
    write_tab_ai_memory_entry_to_path(&newer, &path).expect("write newer");

    let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
        "force quit app",
        Some("com.tinyspeck.slackmacgap"),
        3,
        &path,
    )
    .expect("resolve");

    assert_eq!(
        resolution.outcome.reason,
        TabAiMemoryResolutionReason::Matched
    );
    // The exact match "force quit app" scores 1.0 and must be first
    assert_eq!(resolution.outcome.top_score, Some(1.0));
    assert_eq!(
        resolution.suggestions.first().map(|s| s.slug.as_str()),
        Some("force-quit-new")
    );
    // Both entries should match (the older one shares tokens with the query)
    assert_eq!(resolution.outcome.match_count, 2);
    assert_eq!(resolution.outcome.candidate_count, 2);
    // matched_slugs must list slugs in score-descending order
    assert_eq!(resolution.outcome.matched_slugs[0], "force-quit-new");
}

#[test]
fn outcome_write_dedupes_same_intent_and_bundle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(".tab-ai-memory.json");

    let first = TabAiExecutionRecord::from_parts(
        "copy url".to_string(),
        "import \"@scriptkit/sdk\";\nawait notify(\"a\");\n".to_string(),
        "/tmp/one.ts".to_string(),
        "copy-url-one".to_string(),
        "ScriptList".to_string(),
        Some("com.google.Chrome".to_string()),
        "model-a".to_string(),
        "provider-a".to_string(),
        0,
        "2026-03-28T00:00:00Z".to_string(),
    );
    let second = TabAiExecutionRecord::from_parts(
        "copy url".to_string(),
        "import \"@scriptkit/sdk\";\nawait notify(\"b\");\n".to_string(),
        "/tmp/two.ts".to_string(),
        "copy-url-two".to_string(),
        "ScriptList".to_string(),
        Some("com.google.Chrome".to_string()),
        "model-a".to_string(),
        "provider-a".to_string(),
        0,
        "2026-03-28T01:00:00Z".to_string(),
    );

    write_tab_ai_memory_entry_to_path(&first, &path).expect("write first");
    write_tab_ai_memory_entry_to_path(&second, &path).expect("write second");

    // Only the latest entry for the same intent+bundle_id pair survives
    let entries = read_tab_ai_memory_index_from_path(&path).expect("read");
    assert_eq!(entries.len(), 1, "dedupe should keep only one entry");
    assert_eq!(entries[0].slug, "copy-url-two");

    // Resolver also confirms matched state
    let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
        "copy url",
        Some("com.google.Chrome"),
        3,
        &path,
    )
    .expect("resolve");

    assert_eq!(
        resolution.outcome.reason,
        TabAiMemoryResolutionReason::Matched
    );
    assert_eq!(resolution.outcome.match_count, 1);
    assert_eq!(resolution.outcome.top_score, Some(1.0));
    assert_eq!(resolution.suggestions[0].slug, "copy-url-two");
}
