//! Source-level regression tests for the deferred image-cache behavior.
//!
//! These tests verify that image cache preparation runs off the UI thread,
//! that both pending-image and chat-history paths route through the unified
//! `defer_cache_requests` pipeline, and that provenance is preserved.

use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn slice_from<'a>(source: &'a str, needle: &str) -> &'a str {
    let idx = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find '{needle}'"));
    &source[idx..]
}

#[test]
fn render_root_moves_original_string_and_clones_once_for_cache() {
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

#[test]
fn chat_load_paths_pass_only_image_payloads_into_deferred_cache() {
    let source = read_source("src/ai/window/chat.rs");

    assert!(
        source.contains("Self::collect_message_image_payloads(&saved_messages)")
            && source.contains("Self::collect_message_image_payloads(&self.current_messages)")
            && source.contains("Self::collect_message_image_payloads(&messages)"),
        "chat load paths should extract image payloads before deferred cache preparation"
    );
}

#[test]
fn defer_cache_pending_image_routes_through_unified_pipeline() {
    let source = read_source("src/ai/window/images.rs");
    let helper = slice_from(&source, "pub(super) fn defer_cache_pending_image(");

    assert!(
        helper.contains("self.defer_cache_requests("),
        "pending-image helper should route through the unified defer_cache_requests pipeline"
    );
    assert!(
        helper.contains("ImageCacheSource::PendingInput"),
        "pending-image helper should tag requests with PendingInput source"
    );
}

#[test]
fn defer_cache_message_images_routes_through_unified_pipeline() {
    let source = read_source("src/ai/window/images.rs");
    let helper = slice_from(&source, "pub(super) fn defer_cache_message_images(");

    assert!(
        helper.contains("self.defer_cache_requests(requests, cx);"),
        "message-image helper should route through the unified defer_cache_requests pipeline"
    );
}

#[test]
fn defer_cache_requests_uses_bounded_channel_and_background_executor() {
    let source = read_source("src/ai/window/images.rs");
    let helper = slice_from(&source, "fn defer_cache_requests(");

    assert!(
        helper.contains("async_channel::bounded"),
        "defer_cache_requests should use a bounded channel for production queueing"
    );
    assert!(
        helper.contains("background_executor()"),
        "defer_cache_requests should prepare images on the background executor"
    );
    assert!(
        helper.contains("prepare_image_cache_work"),
        "defer_cache_requests should call prepare_image_cache_work for each request"
    );
}
