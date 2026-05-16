//! Source-audit tests for SDK image clipboard behavior.
//!
//! Image clipboard support crosses the TypeScript SDK and Rust executor, so
//! these tests pin the protocol contract without requiring a live pasteboard in
//! CI.

use super::read_source as read;

const SDK_PATH: &str = "scripts/kit-sdk.ts";
const EXECUTOR_PATH: &str = "src/execute_script/mod.rs";

#[test]
fn sdk_image_clipboard_contract_stays_png_buffer_based() {
    let sdk = read(SDK_PATH);

    assert!(
        sdk.contains("readImage(): Promise<Buffer>"),
        "readImage must keep the existing Buffer API"
    );
    assert!(
        sdk.contains("Read the current clipboard image as PNG-encoded bytes"),
        "readImage must document that the Buffer contains PNG bytes"
    );
    assert!(
        sdk.contains("Decode PNG or JPEG bytes"),
        "writeImage must document accepted encoded image inputs"
    );
    assert!(
        sdk.contains("clipboardErrorFromSubmitValue"),
        "image clipboard methods must parse executor error submit values"
    );
}

#[test]
fn sdk_image_clipboard_rejects_distinct_error_submit_values() {
    let sdk = read(SDK_PATH);
    let read_image_body = sdk
        .split("async readImage(): Promise<Buffer>")
        .nth(1)
        .and_then(|rest| {
            rest.split("async writeImage(buffer: Buffer): Promise<void>")
                .next()
        })
        .expect("readImage implementation must exist");
    let write_image_body = sdk
        .split("async writeImage(buffer: Buffer): Promise<void>")
        .nth(1)
        .and_then(|rest| rest.split("};").next())
        .expect("writeImage implementation must exist");

    assert!(
        read_image_body.contains("ERR_CLIPBOARD_NO_RESPONSE"),
        "readImage auto-submit fallback must reject instead of resolving an empty Buffer"
    );
    assert!(
        read_image_body.contains("ERR_CLIPBOARD_IMAGE_NOT_AVAILABLE"),
        "readImage must distinguish no supported image from successful empty data"
    );
    assert!(
        read_image_body.contains("reject(error)"),
        "readImage must reject executor image errors"
    );
    assert!(
        write_image_body.contains("ERR_CLIPBOARD_NO_RESPONSE"),
        "writeImage auto-submit fallback must reject missing runtime responses"
    );
    assert!(
        write_image_body.contains("reject(error)"),
        "writeImage must reject executor image errors"
    );
}

#[test]
fn executor_image_read_encodes_png_not_raw_rgba() {
    let executor = read(EXECUTOR_PATH);
    let read_helper = executor
        .split("fn read_clipboard_image_as_png_base64()")
        .nth(1)
        .and_then(|rest| rest.split("fn write_clipboard_image_from_base64").next())
        .expect("image read helper must exist");

    for required in [
        "get_image()",
        "image::RgbaImage::from_raw",
        "image::ImageFormat::Png",
        "STANDARD.encode(png_bytes)",
        "ERR_CLIPBOARD_IMAGE_NOT_AVAILABLE",
        "ERR_CLIPBOARD_IMAGE_ENCODE_FAILED",
    ] {
        assert!(
            read_helper.contains(required),
            "image read helper must preserve PNG contract: {required}"
        );
    }
}

#[test]
fn executor_image_write_sets_real_image_not_text() {
    let executor = read(EXECUTOR_PATH);
    let write_helper = executor
        .split("fn write_clipboard_image_from_base64")
        .nth(1)
        .and_then(|rest| rest.split("/// Get information about all displays").next())
        .expect("image write helper must exist");
    let write_match = executor
        .split("protocol::ClipboardAction::Write =>")
        .nth(1)
        .expect("clipboard write action branch must exist");
    let write_branch = write_match
        .split("Some(protocol::ClipboardFormat::Image) =>")
        .nth(1)
        .and_then(|rest| {
            rest.split("Some(protocol::ClipboardFormat::Text) | None")
                .next()
        })
        .expect("clipboard image write branch must exist");

    for required in [
        "image::load_from_memory",
        "arboard::ImageData",
        "set_image",
        "ERR_CLIPBOARD_IMAGE_DECODE_FAILED",
        "ERR_CLIPBOARD_IMAGE_WRITE_FAILED",
        "ERR_CLIPBOARD_IMAGE_MISSING_CONTENT",
    ] {
        assert!(
            write_helper.contains(required),
            "image write helper must decode and write real image data: {required}"
        );
    }
    assert!(
        write_branch.contains("write_clipboard_image_from_base64"),
        "image write branch must use the image helper"
    );
    assert!(
        !write_branch.contains("set_text"),
        "image write branch must not write base64 as text"
    );
}
