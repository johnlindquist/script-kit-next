# Clipboard + Webcam Improvements Audit

## Scope

Primary files reviewed:
- `src/clipboard_actions_tests.rs`
- `src/webcam_actions_consistency_tests.rs`
- `src/camera.rs`

Supporting implementation reviewed for integration/context:
- `src/app_actions.rs`
- `src/actions/builders.rs`
- `src/clipboard_history/*`
- `src/app_execute.rs`
- `src/app_impl.rs`
- `src/prompts/webcam.rs`
- `src/render_prompts/other.rs`
- `src/protocol/message.rs`

## Executive Summary

Clipboard history and webcam features are functional, but both areas rely heavily on source-audit tests and have important runtime coverage gaps.

Most important findings:
1. Clipboard data modeling only supports `Text` and `Image`, which limits format fidelity and causes feature mismatches.
2. Clipboard monitor prioritizes text and exits early, so mixed payload clipboard writes can drop image capture.
3. Webcam reliability paths (permission denial, camera busy/unavailable, teardown/reopen stability) are not validated by behavior tests.
4. Webcam photo save naming is second-granularity and can overwrite when capturing multiple images within one second.

## Findings

### 1) Clipboard content model is too coarse for format fidelity

Evidence:
- `ContentType` enum is only `Text | Image` (`src/clipboard_history/types.rs:9`).
- `ContentType::from_str` defaults unknown values to `Text` (`src/clipboard_history/types.rs:23`).
- Temp-file and save flows branch on only text/image (`src/clipboard_history/temp_file.rs:16`, `src/app_actions.rs:2039`).

Impact:
- No first-class handling for rich text/HTML/RTF, file lists, PDFs, URLs, or custom UTIs.
- Unknown/new formats silently degrade to `Text` on read, risking incorrect behavior and hidden data loss.
- Actions like "Save as File..." and "Open With..." cannot preserve original content semantics.

Recommendation:
- Extend `ContentType` to explicit variants (at minimum: `RichText`, `Html`, `FileList`, `Url`, `Binary`).
- Preserve original format metadata (MIME/UTI) in schema and model.
- Replace default-to-text decoding with explicit `Unknown(String)` variant.

---

### 2) Clipboard monitor drops image entries when text is also present

Evidence:
- Clipboard capture checks text first and `return`s before image logic (`src/clipboard_history/monitor.rs:204`, `src/clipboard_history/monitor.rs:252`).

Impact:
- Mixed payload writes (common in copy operations that include both text representations and image/file representations) can record only text.
- Behavior diverges from user expectations when copying rich objects/screenshots from some apps.

Recommendation:
- Capture in priority order based on available representations (or store multi-representation payload).
- At minimum, introduce policy: prefer image when image data exists and text is likely derivative metadata.
- Add telemetry when both text and image are present to verify real-world frequency before final policy.

---

### 3) Clipboard action surface and runtime constraints are partially misaligned

Evidence:
- Builder always exposes `clipboard_save_snippet` (`src/actions/builders.rs:1022`).
- Runtime rejects non-text entries for this action (`src/app_actions.rs:2084`).

Impact:
- Image entries can show an action that always fails with a HUD error.
- Extra error paths increase user friction and noise.

Recommendation:
- Only include `clipboard_save_snippet` for text entries in `get_clipboard_history_context_actions`.
- Add a regression test that validates action visibility by content type, not only handler existence.

---

### 4) Clipboard regression tests are mostly source-audit string checks

Evidence:
- `src/clipboard_actions_tests.rs` asserts string containment in source files (`content.contains(...)`).
- `src/clipboard_actions_menu_coverage_tests.rs`/`src/clipboard_actions_focus_routing_tests.rs` similarly check source structure.

Impact:
- Tests can pass even when runtime behavior is broken.
- Refactors can create false failures/false confidence.

Recommendation:
- Keep source-audit tests as guardrails, but add behavior tests around:
  - clipboard action dispatch outcomes (`paste`, `copy`, `share`, `save_file`, `save_snippet`),
  - content-type-specific action availability,
  - destructive action confirmation + cancel behavior,
  - mixed payload ingestion policy.

---

### 5) Webcam startup errors are surfaced, but reliability paths are under-specified

Evidence:
- `start_capture` returns generic errors like `No camera found` / input failures (`src/camera.rs:103`, `src/camera.rs:117`, `src/camera.rs:123`, `src/camera.rs:180`).
- No explicit authorization preflight/request handling for camera access found in webcam path (`src/camera.rs`, `src/app_execute.rs:1857`).

Impact:
- Permission-denied vs no-device vs in-use cases are not distinguished for user guidance.
- Harder to auto-recover/retry with the right UX action.

Recommendation:
- Introduce typed webcam startup errors (`PermissionDenied`, `NoDevice`, `DeviceBusy`, `OutputInitFailed`, etc.).
- Add permission preflight/status check before `start_capture` and user-specific remediation messaging.

---

### 6) Webcam save path can collide within one second

Evidence:
- Capture filename uses `%Y%m%d-%H%M%S` only (`src/app_impl.rs:4410`).
- File write uses `std::fs::write` to that path (`src/app_impl.rs:4415`).

Impact:
- Multiple captures within the same second can overwrite prior images.

Recommendation:
- Use millisecond timestamp, monotonic counter, or UUID suffix (`webcam-photo-<ts>-<uuid>.png`).
- Add a test ensuring sequential rapid captures produce unique paths.

---

### 7) Webcam tests are consistency/source audits, not runtime reliability tests

Evidence:
- `src/webcam_actions_consistency_tests.rs` only inspects source sections and string patterns.
- No tests found for `src/camera.rs` lifecycle, frame callback behavior, or teardown safety.

Impact:
- Critical paths (drop safety, callback/disconnect behavior, save/encode errors) may regress undetected.

Recommendation:
- Add test layers:
  - Unit tests for filename/path generation and action routing.
  - Integration tests for `open_webcam` error path and prompt error rendering.
  - macOS system tests (`--features system-tests`) for start/stop/reopen and permission-denied messaging.

## Integration Gaps

1. Protocol limitations:
- Webcam protocol currently only accepts `{ "type": "webcam", "id": "..." }` (`src/protocol/message.rs:430`).
- No device selection, resolution hints, mirror setting, or capture destination options.

2. Clipboard ↔ AI integration fidelity:
- Attach-to-AI supports text and PNG image only (`src/app_actions.rs:597`, `src/app_actions.rs:605`).
- Rich text and file-list semantics cannot be preserved with current model.

3. Observability consistency:
- Clipboard monitor includes some `correlation_id` logging (oversize trim path) (`src/clipboard_history/monitor.rs:75`).
- Webcam capture/open paths log plain strings without structured per-action correlation in reviewed paths (`src/app_execute.rs:1858`, `src/app_impl.rs:4372`).

## Prioritized Improvement Plan

### P0 (correctness / user trust)
1. Fix webcam filename collisions.
2. Hide incompatible clipboard actions per content type.
3. Add behavior tests for destructive clipboard confirmations and webcam capture success/failure paths.

### P1 (robustness / reliability)
1. Add typed webcam error taxonomy + permission-aware messaging.
2. Add mixed clipboard payload policy with explicit tests.
3. Add runtime tests for webcam open -> capture -> close -> reopen cycle.

### P2 (feature completeness)
1. Expand clipboard content model beyond `Text|Image`.
2. Extend webcam protocol with optional options (`deviceId`, `resolution`, `mirror`, save mode).
3. Support richer clipboard-to-AI attachments (rich text/file references with metadata).

## Suggested Test Additions

Clipboard:
- `test_clipboard_builder_hides_save_snippet_for_images`
- `test_clipboard_monitor_prefers_image_when_text_and_image_available` (or chosen policy)
- `test_clipboard_content_type_unknown_is_not_silently_text`

Webcam:
- `test_webcam_capture_generates_unique_filename_on_rapid_calls`
- `test_webcam_open_shows_permission_denied_message_when_access_blocked`
- `test_webcam_start_stop_reopen_does_not_leak_or_stall`

## Notes on Current Strengths

- Camera capture teardown path in `CaptureHandle::drop` is careful about stop + queue drain + sender cleanup (`src/camera.rs:40`).
- Webcam actions use shared `ActionsDialog::with_config` path and stable IDs (`src/app_impl.rs:3835`, `src/app_impl.rs:4443`).
- Clipboard destructive actions are confirmation-gated (`src/app_actions.rs:1902`, `src/app_actions.rs:1940`).
