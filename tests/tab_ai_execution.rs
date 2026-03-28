//! Integration tests for Tab AI post-execution contracts.
//!
//! Validates:
//! 1. Save-offer decision is deterministic and testable
//! 2. Memory write-back produces correct entries via the public API
//! 3. Temp-file cleanup works on both present and absent paths
//! 4. Execution records round-trip through JSON
//! 5. Public re-exports cover all post-execution types

use script_kit_gpui::ai::{
    cleanup_tab_ai_temp_script, read_tab_ai_memory_index_from_path, should_offer_save,
    write_tab_ai_memory_entry_to_path, TabAiExecutionRecord, TabAiMemoryEntry,
    TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION, TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
};

/// Helper: build a representative execution record for testing.
fn sample_record() -> TabAiExecutionRecord {
    TabAiExecutionRecord::from_parts(
        "force quit Slack".to_string(),
        "import '@anthropic-ai/sdk';\nawait exec('kill Slack');".to_string(),
        "/tmp/scriptlet-tab-ai-test.ts".to_string(),
        "force-quit-slack".to_string(),
        "AppLauncher".to_string(),
        Some("com.tinyspeck.slackmacgap".to_string()),
        "2026-03-28T12:00:00Z".to_string(),
    )
}

/// Helper: create a temp index path for isolated memory tests.
fn temp_index_path() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join(".tab-ai-memory.json");
    (dir, path)
}

// ── Execution Record ──

#[test]
fn execution_record_schema_version_matches_constant() {
    let record = sample_record();
    assert_eq!(
        record.schema_version,
        TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION
    );
}

#[test]
fn execution_record_json_roundtrip() {
    let record = sample_record();
    let json = serde_json::to_string_pretty(&record).expect("serialize");
    let parsed: TabAiExecutionRecord = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.schema_version, record.schema_version);
    assert_eq!(parsed.intent, "force quit Slack");
    assert_eq!(parsed.generated_source, record.generated_source);
    assert_eq!(parsed.temp_script_path, record.temp_script_path);
    assert_eq!(parsed.slug, "force-quit-slack");
    assert_eq!(parsed.prompt_type, "AppLauncher");
    assert_eq!(
        parsed.bundle_id.as_deref(),
        Some("com.tinyspeck.slackmacgap")
    );
    assert_eq!(parsed.executed_at, "2026-03-28T12:00:00Z");
}

#[test]
fn execution_record_camel_case_json_keys() {
    let record = sample_record();
    let json = serde_json::to_string(&record).expect("serialize");

    assert!(json.contains("schemaVersion"));
    assert!(json.contains("generatedSource"));
    assert!(json.contains("tempScriptPath"));
    assert!(json.contains("promptType"));
    assert!(json.contains("bundleId"));
    assert!(json.contains("executedAt"));

    // No snake_case leakage
    assert!(!json.contains("schema_version"));
    assert!(!json.contains("generated_source"));
    assert!(!json.contains("temp_script_path"));
    assert!(!json.contains("prompt_type"));
    assert!(!json.contains("bundle_id"));
    assert!(!json.contains("executed_at"));
}

#[test]
fn execution_record_omits_none_bundle_id() {
    let record = TabAiExecutionRecord::from_parts(
        "test".to_string(),
        "code".to_string(),
        "/tmp/x.ts".to_string(),
        "test".to_string(),
        "ScriptList".to_string(),
        None,
        "2026-03-28T00:00:00Z".to_string(),
    );
    let json = serde_json::to_string(&record).expect("serialize");
    assert!(!json.contains("bundleId"));
}

// ── Save-Offer Decision ──

#[test]
fn save_offer_returns_true_for_non_empty_source() {
    let record = sample_record();
    assert!(should_offer_save(&record));
}

#[test]
fn save_offer_returns_false_for_whitespace_only_source() {
    let record = TabAiExecutionRecord::from_parts(
        "test".to_string(),
        "   \n\t  ".to_string(),
        "/tmp/x.ts".to_string(),
        "test".to_string(),
        "ScriptList".to_string(),
        None,
        "2026-03-28T00:00:00Z".to_string(),
    );
    assert!(!should_offer_save(&record));
}

#[test]
fn save_offer_returns_false_for_empty_source() {
    let record = TabAiExecutionRecord::from_parts(
        "test".to_string(),
        "".to_string(),
        "/tmp/x.ts".to_string(),
        "test".to_string(),
        "ScriptList".to_string(),
        None,
        "2026-03-28T00:00:00Z".to_string(),
    );
    assert!(!should_offer_save(&record));
}

// ── Temp-File Cleanup ──

#[test]
fn cleanup_removes_existing_temp_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("tab-ai-cleanup-test.ts");
    std::fs::write(&path, "console.log('test')").expect("write");

    assert!(path.exists(), "precondition: file exists");
    let result = cleanup_tab_ai_temp_script(path.to_str().expect("utf8"));
    assert!(result, "cleanup should succeed");
    assert!(!path.exists(), "file should be removed");
}

#[test]
fn cleanup_returns_true_for_already_absent_file() {
    let result = cleanup_tab_ai_temp_script("/tmp/nonexistent-tab-ai-integration-test-99999.ts");
    assert!(result, "absent file should return true (idempotent)");
}

// ── Memory Write-Back ──

#[test]
fn memory_write_back_produces_correct_entry() {
    let (_dir, index_path) = temp_index_path();

    let record = sample_record();
    let entry =
        write_tab_ai_memory_entry_to_path(&record, &index_path).expect("write should succeed");

    assert_eq!(entry.schema_version, TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION);
    assert_eq!(entry.intent, "force quit Slack");
    assert_eq!(entry.slug, "force-quit-slack");
    assert_eq!(entry.prompt_type, "AppLauncher");
    assert_eq!(
        entry.bundle_id.as_deref(),
        Some("com.tinyspeck.slackmacgap")
    );

    // Verify the file was actually written
    assert!(index_path.exists(), "index file should exist on disk");

    // Read back and verify
    let entries =
        read_tab_ai_memory_index_from_path(&index_path).expect("read index");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0], entry);
}

#[test]
fn memory_write_back_deduplicates_by_intent_and_bundle_id() {
    let (_dir, index_path) = temp_index_path();

    // Write first entry
    let record1 = sample_record();
    let _ = write_tab_ai_memory_entry_to_path(&record1, &index_path).expect("write 1");

    // Write second entry with same intent + bundle_id but different source
    let record2 = TabAiExecutionRecord::from_parts(
        "force quit Slack".to_string(),
        "process.kill('Slack')".to_string(),
        "/tmp/scriptlet-updated.ts".to_string(),
        "force-quit-slack-v2".to_string(),
        "AppLauncher".to_string(),
        Some("com.tinyspeck.slackmacgap".to_string()),
        "2026-03-28T13:00:00Z".to_string(),
    );
    let _ = write_tab_ai_memory_entry_to_path(&record2, &index_path).expect("write 2");

    // Read back — should have exactly 1 entry (deduped)
    let entries =
        read_tab_ai_memory_index_from_path(&index_path).expect("read index");
    assert_eq!(entries.len(), 1, "duplicate should be removed");
    assert_eq!(entries[0].slug, "force-quit-slack-v2", "latest entry wins");
    assert_eq!(
        entries[0].generated_source, "process.kill('Slack')",
        "latest source preserved"
    );
}

#[test]
fn memory_write_back_keeps_different_intents_separate() {
    let (_dir, index_path) = temp_index_path();

    let record1 = sample_record();
    let _ = write_tab_ai_memory_entry_to_path(&record1, &index_path).expect("write 1");

    let record2 = TabAiExecutionRecord::from_parts(
        "open finder".to_string(),
        "await exec('open /Applications/Finder.app')".to_string(),
        "/tmp/scriptlet-finder.ts".to_string(),
        "open-finder".to_string(),
        "ScriptList".to_string(),
        None,
        "2026-03-28T13:00:00Z".to_string(),
    );
    let _ = write_tab_ai_memory_entry_to_path(&record2, &index_path).expect("write 2");

    let entries =
        read_tab_ai_memory_index_from_path(&index_path).expect("read index");
    assert_eq!(entries.len(), 2, "different intents should both persist");
}

#[test]
fn memory_read_returns_empty_for_nonexistent_path() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("does-not-exist.json");
    let entries = read_tab_ai_memory_index_from_path(&path).expect("should succeed");
    assert!(entries.is_empty());
}

// ── Public Export Coverage ──

#[test]
fn public_exports_cover_all_post_execution_types() {
    // This test exists solely to verify that the re-export path in
    // src/ai/mod.rs covers all post-execution types. If a re-export breaks,
    // this test fails at compile time.
    let _record = TabAiExecutionRecord::from_parts(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        None,
        String::new(),
    );
    let _entry = TabAiMemoryEntry {
        schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
        intent: String::new(),
        generated_source: String::new(),
        slug: String::new(),
        prompt_type: String::new(),
        bundle_id: None,
        written_at: String::new(),
    };
    assert_eq!(TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION, 1);
    assert_eq!(TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION, 1);
}
