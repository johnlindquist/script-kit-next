use script_kit_gpui::clipboard_history::ocr::run_vision_ocr;
use script_kit_gpui::clipboard_history::{enqueue_ocr, stop_ocr_worker};

#[test]
fn test_enqueue_ocr_returns_error_when_worker_is_not_started() {
    // Ensure this test does not depend on global state from other tests.
    let _ = stop_ocr_worker();

    let error = enqueue_ocr("entry-id".to_string(), "blob:abc123".to_string())
        .expect_err("expected enqueue_ocr to fail without a running worker")
        .to_string();

    assert!(
        error.contains("worker_not_started") || error.contains("worker_disconnected"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_run_vision_ocr_returns_error_when_blob_file_is_missing() {
    let error = run_vision_ocr("/tmp/script-kit-ocr-missing-file.png")
        .expect_err("expected OCR to fail for a missing file")
        .to_string();

    assert!(
        error.contains("missing_blob_file") || error.contains("unsupported_platform_or_feature"),
        "unexpected error: {error}"
    );
}
