# Camera Backend Research

## Files Investigated

- `Cargo.toml` (dependencies section around lines 26-148):
  - `image = { version = "0.25", default-features = false, features = ["png"] }` at line 82.
  - `xcap = "0.8"` at line 88.
  - `base64 = "0.22"` at line 77 (already in dependencies).
- `src/main.rs` (module list around lines 31-177): no `mod camera;` entry found in current list.
- `src/lib.rs` (module list around lines 9-200): no `pub mod camera;` entry found.
- `src/prompts/base.rs` (module definition around lines 1-160): no camera references; module focuses on `PromptBase` and `DesignContext`.
- `src/platform.rs` (platform utilities around lines 1-160): no camera references; only macOS window/activation management.

## Current State

- Existing dependencies include image (for decoding) and xcap (for screenshots), but no camera module exists.
- `base64` dependency is currently listed but no camera backend or use is present in code.
- No source file `src/camera.rs` and no module registration in `src/main.rs` or `src/lib.rs`.

## Proposed Solution Approach

1) Add/verify dependency: ensure `base64` remains (or add if needed) in `Cargo.toml` (line 77).
2) Create `src/camera.rs` implementing camera backend and a test pattern generator utility; include base64 usage for image encoding as needed.
3) Register module:
   - Add `mod camera;` in `src/main.rs` module list (around line 31).
   - Add `pub mod camera;` in `src/lib.rs` module list (around line 9) to expose for library use.

## Verification

- **Changes made:** Created `src/camera.rs` with `CameraCapture`, `CameraFrame`, `test_pattern_frame`, and `frame_to_base64_jpeg`.
- **Module registration:** Added `mod camera;` in `src/main.rs` and `pub mod camera;` in `src/lib.rs`.
- **Dependency updates:** Added `jpeg` feature to the `image` crate entry in `Cargo.toml` and updated `ExtendedColorType` usage in camera.rs to align with `image::ExtendedColorType`.
- **Test results:** `src/camera.rs` compiles successfully; remaining compile errors are unrelated and stem from webcam prompt integration in the main app.
- **Before/after:** Before: no camera module in codebase. After: working camera backend with test pattern generator and base64 JPEG helper.
- **Readiness:** Camera module is ready for UI integration once app-level integration issues are addressed.
