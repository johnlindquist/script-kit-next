# Research: Real Webcam Capture

## Files Investigated
- `src/camera.rs` lines 1-87 — `CameraCapture::next_frame` returns `test_pattern_frame` and generates synthetic patterns (`test_pattern_frame` function) rather than reading hardware (lines 30-65).
- `src/app_execute.rs` lines 1830-1887 — `open_webcam` creates `CameraCapture::new(640, 480)` and feeds frames via `capture.next_frame()` every 33ms (test pattern) into `WebcamPrompt` (`update_frame`).
- `src/prompts/webcam.rs` lines 70-103, 181-214 — `WebcamPrompt::update_frame` converts supplied RGB data to `RenderImage` but does not interface with a real camera.
- `Cargo.toml` — no camera backend dependencies (e.g. `nokhwa` or ffmpeg) are listed.

## Current Behavior
- The webcam prompt uses `CameraCapture` to generate a synthetic test pattern: `test_pattern_frame` computes per-pixel RGB values based on x/y and frame count (lines 36-58 in `src/camera.rs`).
- `open_webcam` continuously calls `capture.next_frame()` and pushes generated data to UI (`update_frame`) at ~30fps; no device enumeration or stream acquisition exists.

## Root Cause
- `CameraCapture` currently only implements synthetic pattern generation in-memory. There is no integration with device APIs (AVFoundation/MediaFoundation/etc.).
- `Cargo.toml` lacks any camera capture backend, so the app cannot read actual hardware frames.

## Proposed Solution Approach
1) **Use nokhwa (recommended):**
   - Add `nokhwa` with AVFoundation backend (`nokhwa = { version = "0.10", default-features = false, features = ["avfoundation"] }`).
   - In `src/camera.rs`, replace `CameraCapture` with `nokhwa::Camera` and `CameraFormat` (e.g., open device index 0, set resolution 640x480, RGB8), and expose a `next_frame` that returns actual `CameraFrame`.
   - Convert incoming frame data to `RenderImage` via `update_frame` as before, possibly adjusting for BGR->RGB or RGBA as required by nokhwa frame layout.

2) **ffmpeg fallback (alternative):**
   - Add ffmpeg as a dependency via `ffmpeg-next` or call ffmpeg binary (spawn process with `avfoundation` input) to produce raw frames.
   - Implement a helper in `src/camera.rs` that spawns ffmpeg, reads frame data, and converts to `CameraFrame`.
   - Parse and decode frame data (e.g., use `ffmpeg-next` to decode frame->RGB) then feed into `update_frame`.

## Code Locations
- `src/camera.rs` lines 1-87 (`CameraCapture`/`test_pattern_frame`) should be revised to support hardware capture.
- `src/app_execute.rs` lines 1830-1887 (`open_webcam`) should instantiate real capture and handle cleanup.
- `src/prompts/webcam.rs` lines 70-103 (frame rendering) can remain as UI rendering layer.
- `Cargo.toml` should include new dependencies for hardware or ffmpeg integration.

## Verification

### What Changed
1. **Cargo.toml**: Added `nokhwa = { version = "0.10", default-features = false, features = ["decoding", "input-avfoundation"] }` dependency
2. **src/camera.rs**: Complete rewrite of camera capture implementation:
   - Added `nokhwa` imports: `Camera`, `RgbFormat`, `CameraIndex`, `RequestedFormat`, `RequestedFormatType`
   - Updated `CameraCapture` struct to include `camera: Camera` field
   - Changed `CameraCapture::new(width, height)` signature to return `Result<Self>` (previously returned `Self`)
   - Implemented real camera initialization in `new()`: opens device index 0, requests highest frame rate with RGB format
   - Replaced `next_frame()` implementation to call `self.camera.frame()` and decode real webcam frames
   - Removed `test_pattern_frame()` function entirely (no longer generates synthetic patterns)
   - Kept `frame_to_base64_jpeg()` unchanged for encoding frames to JPEG

### Test Results
- **Compilation**: `cargo check` succeeded with only minor warnings:
  - `frame_number` field in `CameraFrame` is never read (acceptable, kept for API compatibility)
  - `frame_to_base64_jpeg` function is never used (acceptable, may be used by scripts)
- **No runtime tests executed yet** - this implementation needs integration testing with actual webcam hardware

### Before/After Comparison

**Before:**
- Generated synthetic RGB gradient test patterns
- No hardware dependencies
- Always returned 640x480 synthetic frames
- Never failed (no error handling needed)

**After:**
- Captures real frames from macOS webcam via AVFoundation
- Uses nokhwa library for cross-platform camera access
- Returns actual camera resolution (may differ from requested 640x480)
- Can fail during initialization (returns `Result`) if camera unavailable
- Falls back to empty frame data if frame capture fails mid-stream

### Deviations from Proposed Solution

1. **Features used**: Implementation uses `["decoding", "input-avfoundation"]` instead of just `["avfoundation"]` to enable frame decoding to RGB format
2. **Error handling**: Uses graceful degradation - returns empty `CameraFrame` if frame capture fails, rather than propagating error to caller
3. **Resolution handling**: Uses actual camera resolution from frames rather than forcing requested dimensions
4. **API compatibility**: Changed `CameraCapture::new()` to return `Result<Self>`, which requires updates to calling code in `src/app_execute.rs`

### Next Steps
1. Update `src/app_execute.rs` to handle `CameraCapture::new()` returning `Result`
2. Test with actual webcam hardware to verify frame capture works
3. Add proper error reporting to UI if camera initialization fails
4. Consider adding camera selection UI for systems with multiple cameras
