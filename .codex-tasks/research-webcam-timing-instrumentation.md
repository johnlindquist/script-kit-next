# Webcam Timing Instrumentation Research

## Files Involved
- `src/camera.rs`
  - `CameraCapture::start` opens the camera and sets format.
  - `CameraCapture::next_frame` captures, decodes to RGB, converts to BGRA.
- `src/app_execute.rs`
  - `App::open_webcam` spawns a background thread for capture and image creation.
  - Creates a channel `(RenderImage, width, height)` and polls from UI thread via timer.
- `src/prompts/webcam.rs`
  - `WebcamPrompt` stores and renders `RenderImage`.

## Current Pipeline Flow
1) **Capture**: `CameraCapture::next_frame` calls `camera.frame()`.
2) **Decode**: `raw.decode_image::<RgbFormat>()` to RGB bytes.
3) **RGB -> BGRA** conversion in `camera.rs` (BGRA vector)
4) **RenderImage**: in `app_execute.rs` background thread, `RgbaImage::from_raw` -> `RenderImage::new`.
5) **Channel**: `frame_tx` sends `(Arc<RenderImage>, width, height)`.
6) **UI**: UI thread receives latest frame and calls `WebcamPrompt::set_render_image` to render via `gpui::img`.

## Suspected Bottlenecks
- **Frame capture**: blocking call `camera.frame()` could stall thread.
- **Decode**: CPU cost of `decode_image`/RGB extraction.
- **RGB -> BGRA conversion**: loop over every pixel in `camera.rs`.
- **RenderImage creation**: allocating `RgbaImage` and `RenderImage` each frame.

## Proposed Timing Instrumentation
- Add `std::time::Instant` around each step and log via `eprintln!`:
  - In `camera.rs`:
    - capture start/finish
    - decode duration
    - conversion duration (RGB -> BGRA)
  - In `app_execute.rs`:
    - creation of `RgbaImage`/`RenderImage`
    - channel send/receive timing

Proposed output: `eprintln!("[webcam] capture {:?}"`), etc., to verify per-frame timing in logs.

## Verification
- `src/camera.rs` now imports `std::time::Instant` and logs timing via
  `eprintln!` in `[webcam] frame`, `decode`, and `bgra` messages formatted as
  `"[webcam] operation WxH: Nms correlation_id=webcam"`.
- `src/app_execute.rs` now logs timing for `RgbaImage` creation (`[webcam] rgba`),
  `RenderImage` creation (`[webcam] render`), and channel send (`[webcam] send`),
  all using `eprintln!` and `correlation_id=webcam`.
- `cargo check` passed with only warning: unused import `smallvec::smallvec` in
  `src/prompts/webcam.rs`.
