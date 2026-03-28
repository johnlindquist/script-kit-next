//! Integration tests for Tab AI post-execution contracts.
//!
//! Validates:
//! 1. Save-offer decision is deterministic and testable
//! 2. Memory write-back produces correct entries via the public API
//! 3. Temp-file cleanup works on both present and absent paths
//! 4. Execution records round-trip through JSON
//! 5. Public re-exports cover all post-execution types
//! 6. Append-only audit receipts write valid JSONL

use script_kit_gpui::ai::{
    append_tab_ai_execution_receipt_to_path, build_tab_ai_execution_receipt,
    cleanup_tab_ai_temp_script, read_tab_ai_memory_index_from_path, should_offer_save,
    write_tab_ai_memory_entry_to_path, TabAiExecutionReceipt, TabAiExecutionRecord,
    TabAiExecutionStatus, TabAiMemoryEntry, TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION,
    TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION, TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
};

/// Helper: build a representative execution record for testing.
fn sample_record() -> TabAiExecutionRecord {
    TabAiExecutionRecord::from_parts(
        "force quit Slack".to_string(),
        "import '@anthropic-ai/sdk';\nawait exec('kill Slack');\nconsole.log('done');".to_string(),
        "/tmp/scriptlet-tab-ai-test.ts".to_string(),
        "force-quit-slack".to_string(),
        "AppLauncher".to_string(),
        Some("com.tinyspeck.slackmacgap".to_string()),
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
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
    assert_eq!(parsed.model_id, "gpt-4.1");
    assert_eq!(parsed.provider_id, "vercel");
    assert_eq!(parsed.context_warning_count, 0);
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
    assert!(json.contains("modelId"));
    assert!(json.contains("providerId"));
    assert!(json.contains("contextWarningCount"));

    // No snake_case leakage
    assert!(!json.contains("schema_version"));
    assert!(!json.contains("generated_source"));
    assert!(!json.contains("temp_script_path"));
    assert!(!json.contains("prompt_type"));
    assert!(!json.contains("bundle_id"));
    assert!(!json.contains("executed_at"));
    assert!(!json.contains("model_id"));
    assert!(!json.contains("provider_id"));
    assert!(!json.contains("context_warning_count"));
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
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
        "2026-03-28T00:00:00Z".to_string(),
    );
    let json = serde_json::to_string(&record).expect("serialize");
    assert!(!json.contains("bundleId"));
}

// ── Save-Offer Decision ──

#[test]
fn save_offer_returns_true_for_three_plus_lines() {
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
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
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
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
        "2026-03-28T00:00:00Z".to_string(),
    );
    assert!(!should_offer_save(&record));
}

#[test]
fn save_offer_returns_false_for_fewer_than_three_lines() {
    let record = TabAiExecutionRecord::from_parts(
        "test".to_string(),
        "line1\nline2".to_string(),
        "/tmp/x.ts".to_string(),
        "test".to_string(),
        "ScriptList".to_string(),
        None,
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
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
    let entries = read_tab_ai_memory_index_from_path(&index_path).expect("read index");
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
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
        "2026-03-28T13:00:00Z".to_string(),
    );
    let _ = write_tab_ai_memory_entry_to_path(&record2, &index_path).expect("write 2");

    // Read back — should have exactly 1 entry (deduped)
    let entries = read_tab_ai_memory_index_from_path(&index_path).expect("read index");
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
        "gpt-4.1".to_string(),
        "vercel".to_string(),
        0,
        "2026-03-28T13:00:00Z".to_string(),
    );
    let _ = write_tab_ai_memory_entry_to_path(&record2, &index_path).expect("write 2");

    let entries = read_tab_ai_memory_index_from_path(&index_path).expect("read index");
    assert_eq!(entries.len(), 2, "different intents should both persist");
}

#[test]
fn memory_read_returns_empty_for_nonexistent_path() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("does-not-exist.json");
    let entries = read_tab_ai_memory_index_from_path(&path).expect("should succeed");
    assert!(entries.is_empty());
}

// ── Audit Receipts ──

#[test]
fn audit_receipt_appends_valid_jsonl() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join(".tab-ai-executions.jsonl");

    let record = sample_record();
    let receipt = build_tab_ai_execution_receipt(
        &record,
        TabAiExecutionStatus::Dispatched,
        false,
        false,
        None,
    );
    append_tab_ai_execution_receipt_to_path(&receipt, &path).expect("append");

    let content = std::fs::read_to_string(&path).expect("read");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1, "exactly one line per receipt");

    let parsed: TabAiExecutionReceipt =
        serde_json::from_str(lines[0]).expect("valid JSON on line 1");
    assert_eq!(parsed.status, TabAiExecutionStatus::Dispatched);
    assert_eq!(
        parsed.schema_version,
        TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION
    );
    assert_eq!(parsed.model_id, "gpt-4.1");
    assert_eq!(parsed.provider_id, "vercel");
}

#[test]
fn audit_receipt_append_only_preserves_prior_lines() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join(".tab-ai-executions.jsonl");

    let record = sample_record();

    let r1 = build_tab_ai_execution_receipt(
        &record,
        TabAiExecutionStatus::Dispatched,
        false,
        false,
        None,
    );
    append_tab_ai_execution_receipt_to_path(&r1, &path).expect("append 1");

    let r2 =
        build_tab_ai_execution_receipt(&record, TabAiExecutionStatus::Succeeded, true, true, None);
    append_tab_ai_execution_receipt_to_path(&r2, &path).expect("append 2");

    let content = std::fs::read_to_string(&path).expect("read");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "two receipts = two lines");

    let p1: TabAiExecutionReceipt = serde_json::from_str(lines[0]).expect("parse line 1");
    let p2: TabAiExecutionReceipt = serde_json::from_str(lines[1]).expect("parse line 2");
    assert_eq!(p1.status, TabAiExecutionStatus::Dispatched);
    assert_eq!(p2.status, TabAiExecutionStatus::Succeeded);
    assert!(p2.memory_write_eligible);
    assert!(p2.save_offer_eligible);
}

// ── Source Contract Scans ──

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn prompt_handler_completes_tab_ai_on_real_script_lifecycle_events() {
    let source = std::fs::read_to_string("src/prompt_handler/mod.rs").expect("read prompt_handler");
    let normalized = normalize_ws(&source);
    assert!(
        normalized.contains("self.complete_tab_ai_execution(true, None, cx);"),
        "ScriptExit must finalize successful Tab AI runs"
    );
    assert!(
        normalized.contains("self.complete_tab_ai_execution(false,"),
        "ScriptError must finalize failed Tab AI runs"
    );
    assert!(
        normalized.contains("if keep_tab_ai_save_offer_open {")
            && normalized
                .contains("Tab AI save offer active after script exit - preserving main window"),
        "ScriptExit must keep the window alive while the Tab AI save offer is open"
    );
}

#[test]
fn tab_ai_success_opens_real_save_offer_overlay() {
    let source = std::fs::read_to_string("src/app_impl/tab_ai_mode.rs").expect("read tab_ai_mode");
    let normalized = normalize_ws(&source);
    assert!(
        normalized.contains("self.open_tab_ai_save_offer(record, cx);"),
        "successful Tab AI runs should surface a real save offer UI"
    );
    assert!(
        normalized.contains("pub(crate) fn render_tab_ai_save_offer_overlay")
            && normalized.contains(".id(\"tab-ai-save-offer\")"),
        "save offer overlay must be renderable and keyboard-driven"
    );
}

#[test]
fn render_impl_layers_tab_ai_save_offer_overlay_above_main_content() {
    let source =
        std::fs::read_to_string("src/main_sections/render_impl.rs").expect("read render_impl");
    let normalized = normalize_ws(&source);
    assert!(
        normalized.contains(
            "let tab_ai_save_offer_overlay = self.render_tab_ai_save_offer_overlay(window, cx);"
        ),
        "render_impl must build the save-offer overlay"
    );
    assert!(
        normalized.contains("tab_ai_save_offer_overlay"),
        "render_impl must layer the save-offer overlay above main content"
    );
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
        String::new(),
        0,
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
    assert_eq!(TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION, 2);
    assert_eq!(TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION, 1);
    assert_eq!(TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION, 1);
}

// --- Prompt handler completion hooks (source audit) ---

const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn prompt_handler_completes_tab_ai_on_script_error() {
    assert!(
        PROMPT_HANDLER_SOURCE.contains("complete_tab_ai_execution(false,"),
        "ScriptError handler must complete failed Tab AI runs"
    );
}

#[test]
fn prompt_handler_completes_tab_ai_on_script_exit() {
    assert!(
        PROMPT_HANDLER_SOURCE.contains("complete_tab_ai_execution(true, None, cx)"),
        "ScriptExit handler must complete successful Tab AI runs"
    );
}
