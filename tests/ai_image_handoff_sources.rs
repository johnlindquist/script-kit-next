//! Source-level contract tests for the typed image-cache pipeline,
//! single-clone pending-image handoff, and stdin AI command observability.

use script_kit_gpui::test_utils::read_source;

fn slice_from<'a>(source: &'a str, needle: &str) -> &'a str {
    let idx = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find '{needle}'"));
    &source[idx..]
}

// ---------------------------------------------------------------------------
// Pending-image path: move + single clone
// ---------------------------------------------------------------------------

#[test]
fn pending_image_path_moves_original_string_and_clones_once_for_cache() {
    let source = read_source("src/ai/window/render_root.rs");
    let block = slice_from(&source, "AiCommand::SetInputWithImage {");

    assert!(
        block.contains("let cache_image = image_base64.clone();"),
        "pending image path should clone the base64 payload once for deferred cache work"
    );
    assert!(
        block.contains("self.pending_image = Some(image_base64);"),
        "pending image path should move the original String into pending_image"
    );
    assert!(
        block.contains("self.defer_cache_pending_image(cache_image, cx);"),
        "pending image path should pass the single clone into deferred caching"
    );
}

// ---------------------------------------------------------------------------
// Typed image-cache pipeline with provenance
// ---------------------------------------------------------------------------

#[test]
fn image_cache_pipeline_emits_structured_enqueue_prepare_and_insert_events() {
    let source = read_source("src/ai/window/images.rs");

    assert!(
        source.contains("enum ImageCacheSource"),
        "image cache pipeline should preserve source provenance"
    );
    assert!(
        source.contains("struct ImageCacheRequest"),
        "image cache pipeline should use a typed request instead of raw String payloads"
    );
    assert!(
        source.contains("event = \"ai_image_cache_enqueue\""),
        "image cache pipeline should log enqueue events"
    );
    assert!(
        source.contains("event = \"ai_image_cache_prepare\""),
        "image cache pipeline should log prepare events"
    );
    assert!(
        source.contains("event = \"ai_image_cache_insert\""),
        "image cache pipeline should log insert events"
    );
}

// ---------------------------------------------------------------------------
// Unified cache pipeline: both paths go through defer_cache_requests
// ---------------------------------------------------------------------------

#[test]
fn both_cache_paths_route_through_unified_defer_cache_requests() {
    let source = read_source("src/ai/window/images.rs");

    let pending = slice_from(&source, "pub(super) fn defer_cache_pending_image(");
    assert!(
        pending.contains("self.defer_cache_requests("),
        "pending-image path should route through defer_cache_requests"
    );

    let message = slice_from(&source, "pub(super) fn defer_cache_message_images(");
    assert!(
        message.contains("self.defer_cache_requests("),
        "message-image path should route through defer_cache_requests"
    );
}

// ---------------------------------------------------------------------------
// Duplicate cache entries are skipped by cache key
// ---------------------------------------------------------------------------

#[test]
fn defer_cache_requests_skips_duplicates_by_cache_key() {
    let source = read_source("src/ai/window/images.rs");
    let helper = slice_from(&source, "fn defer_cache_requests(");

    assert!(
        helper.contains("queued_keys"),
        "defer_cache_requests should track queued keys to skip duplicates in batch"
    );
    assert!(
        helper.contains("skipped_cached"),
        "defer_cache_requests should skip already-cached entries"
    );
    assert!(
        helper.contains("skipped_duplicate_in_batch"),
        "defer_cache_requests should skip duplicate entries within the same batch"
    );
}

// ---------------------------------------------------------------------------
// Stdin AI commands: structured logging and error propagation
// ---------------------------------------------------------------------------

#[test]
fn stdin_ai_commands_log_request_ids_and_do_not_swallow_failures() {
    let source = read_source("src/main_entry/runtime_stdin.rs");

    assert!(
        source.contains("event = \"stdin_ai_command_received\""),
        "stdin AI commands should log the received phase"
    );
    assert!(
        source.contains("event = \"stdin_ai_command_finished\""),
        "stdin AI commands should log the finished phase"
    );
    assert!(
        source.contains("request_id = ?request_id"),
        "stdin AI command logs should include requestId for machine correlation"
    );
    assert!(
        !source.contains("let _ = ai::set_ai_input("),
        "stdin AI input path must not swallow queue errors"
    );
}

// ---------------------------------------------------------------------------
// set_ai_search returns Result
// ---------------------------------------------------------------------------

#[test]
fn set_ai_search_returns_result() {
    let source = read_source("src/ai/window/window_api.rs");

    assert!(
        source.contains("pub fn set_ai_search(cx: &mut App, query: &str) -> Result<(), String>"),
        "set_ai_search should return Result<(), String> like set_ai_input"
    );
}

#[test]
fn enqueue_ai_window_command_rolls_back_stale_queue_entries_on_notify_failure() {
    let source = read_source("src/ai/window/window_api.rs");
    let helper = slice_from(&source, "fn enqueue_ai_window_command(");

    assert!(
        helper.contains("let queued_index = commands.len();"),
        "enqueue_ai_window_command should track the queued command slot"
    );
    assert!(
        helper.contains("commands.remove(queued_index);"),
        "enqueue_ai_window_command should remove the queued command when notify fails"
    );
}
