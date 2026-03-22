//! Source-level regression tests for the deferred image-cache behavior.
//!
//! These tests verify that image cache preparation runs off the UI thread
//! and that chat load paths extract image payloads instead of cloning full
//! `Message` structs.

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
fn render_root_sets_pending_image_and_defers_cache_work() {
    let source = read_source("src/ai/window/render_root.rs");
    let block = slice_from(&source, "AiCommand::SetInputWithImage {");

    assert!(
        block.contains("self.pending_image = Some(image_base64.clone());"),
        "SetInputWithImage should keep the image immediately available for the next submit"
    );
    assert!(
        block.contains("self.defer_cache_pending_image(image_base64.clone(), cx);"),
        "SetInputWithImage should defer image cache preparation"
    );
    assert!(
        !block.contains("self.cache_image_from_base64(&image_base64);"),
        "SetInputWithImage must not decode thumbnails inline on the command-application path"
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
fn defer_cache_pending_image_prepares_bytes_off_thread() {
    let source = read_source("src/ai/window/images.rs");
    let helper = slice_from(&source, "pub(super) fn defer_cache_pending_image(");

    assert!(
        helper.contains("background_executor()")
            && helper.contains("prepare_image_cache_work")
            && helper.contains(".detach();"),
        "pending-image helper should prepare image bytes off the UI thread"
    );
    assert!(
        !helper.contains("decode_png_to_render_image_with_bgra_conversion"),
        "pending-image helper must not perform PNG render-image decoding on the UI thread"
    );
}

#[test]
fn defer_cache_message_images_batches_background_preparation() {
    let source = read_source("src/ai/window/images.rs");
    let helper = slice_from(&source, "pub(super) fn defer_cache_message_images(");

    assert!(
        helper.contains("background_executor()")
            && helper.contains("prepare_image_cache_work")
            && helper.contains("collect::<Vec<_>>()"),
        "message-image helper should batch background preparation before a single UI update"
    );
    assert!(
        !helper.contains("decode_png_to_render_image_with_bgra_conversion"),
        "message-image helper must not perform PNG render-image decoding on the UI thread"
    );
}
