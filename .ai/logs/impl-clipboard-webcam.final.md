Implemented scoped clipboard/webcam improvements in the claimed files.

Changed files:
- `src/camera.rs:22`  
Added typed startup error enum `WebcamStartError` with structured context (`attempted`, `failed`, `state`, `details`) and switched `start_capture` to return typed errors.
- `src/camera.rs:115`  
Added NSError parsing/classification (`PermissionDenied`, `DeviceBusy`, `NoDevice`, fallback `InputInitFailed`) plus helper conversion for Objective-C `NSError` fields.
- `src/camera.rs:262`  
Added centralized cleanup helper for failure paths to avoid partial-resource leaks.
- `src/camera.rs:582`  
Added unit tests for webcam error classification behavior.
- `src/clipboard_actions_tests.rs:57`  
Added regression test `clipboard_save_snippet_rejects_non_text_entries`.
- `src/webcam_actions_consistency_tests.rs:33`  
Expanded consistency coverage for webcam capture flow, open-flow error surfacing, and typed camera startup error contract.

Verification run:
- `cargo fmt` ✅
- `cargo test test_classify_input_error_returns_permission_denied_when_not_authorized -- --nocapture` ❌ blocked by unrelated workspace compile errors (`src/app_actions.rs:849` and peers, plus `src/ai/window.rs:8729`)
- `cargo check` ❌ blocked by same unrelated errors (outside scoped files)

How to test (after upstream compile blockers are resolved):
1. `cargo check`
2. `cargo test test_classify_input_error_returns_permission_denied_when_not_authorized`
3. `cargo test clipboard_save_snippet_rejects_non_text_entries`
4. `cargo test webcam_camera_module_uses_typed_startup_error_taxonomy`

Risks / known gaps:
- Webcam NSError classification uses domain/code + message heuristics; uncommon vendor/device messages may still fall back to generic `InputInitFailed`.
- `open_webcam` currently stringifies typed errors rather than branching UX by variant.
- Full test/cargo verification is currently blocked by unrelated in-progress agent changes in other files.

Commits made:
- None.