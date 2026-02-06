# Nokhwa decode_image Research Notes (AVFoundation)

## Files Investigated (nokhwa source)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/buffer.rs` (buffer decode)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/pixel_format.rs` (FormatDecoder and RgbFormat conversion)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/types.rs` (FrameFormat definitions and conversion helpers)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-0.10.10/src/camera.rs` (Camera API including frame_raw)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-0.10.10/src/backends/capture/avfoundation.rs` (AVFoundation frame/frame_raw)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-bindings-macos-0.2.3/src/lib.rs` (AVFoundation FourCC mapping)
- `../src/camera.rs` (Script Kit wrapper over Nokhwa)
- `../src/app_execute.rs` (Script Kit webcam prompt rendering)

## decode_image Behavior
- `Buffer::decode_image` allocates output via `FormatDecoder::write_output` and wraps it in `image::ImageBuffer` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/buffer.rs` lines 73-88).
- `FormatDecoder::write_output` dispatches based on source `FrameFormat` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/pixel_format.rs` lines 58-126).
  - `RgbFormat` handles MJPEG via `mjpeg_to_rgb`, YUYV via `yuyv422_to_rgb`, NV12 via `nv12_to_rgb`, GRAY by component expansion, RAWRGB by direct copy, RAWBGR by channel swap, and RAWRGB by copying.
- `mjpeg_to_rgb` (and `buf_mjpeg_to_rgb`) decode MJPEG using `mozjpeg::Decompress` and `read_scanlines` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/types.rs` lines 1456-1506) — CPU decode, no SIMD.
- `yuyv422_to_rgb` and `buf_yuyv422_to_rgb` (in `types.rs` lines 1603-1693) use iterator loops over `chunks_exact(4)` and call `yuyv444_to_rgb` per pair; no SIMD.
- `yuyv444_to_rgb` performs CPU arithmetic conversion formulas, no SIMD (`types.rs` lines 1699-1710).

## AVFoundation Pixel Formats
- AVFoundation fourcc mapping via `raw_fcc_to_frameformat` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-bindings-macos-0.2.3/src/lib.rs` lines 365-377) maps to `FrameFormat`:
  - `kCMVideoCodecType_422YpCbCr8` / `kCMPixelFormat_422YpCbCr8_yuvs` → `YUYV`
  - `kCMVideoCodecType_JPEG` / `kCMVideoCodecType_JPEG_OpenDML` → `MJPEG`
  - `kCMPixelFormat_8IndexedGray_WhiteIsZero` → `GRAY`
  - `kCVPixelFormatType_420YpCbCr*` → `YUYV`
  - `kCMPixelFormat_24RGB` → `RAWRGB`

## Resolution Negotiation: AbsoluteHighestFrameRate
- `RequestedFormatType::AbsoluteHighestFrameRate` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-core/src/types.rs` lines 101-113) picks
  - highest frame rate among all formats,
  - then among those picks highest resolution (sort by `CameraFormat::resolution`).

## Alternative: frame_raw (bypass decode)
- `Camera::frame_raw` exposes raw bytes without decoding (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-0.10.10/src/camera.rs` lines 380-386).
- AVFoundation backend `frame_raw` returns `Cow<[u8]>` of raw bytes before decoding (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nokhwa-0.10.10/src/backends/capture/avfoundation.rs` lines 282-296).
- Use custom decode path to avoid MJPEG/YUYV/other decode_image CPU work and control SIMD/zero-copy paths manually.

## Script Kit Webcam Integration (Current)
- `src/camera.rs` defines `CameraCapture::start` and `CameraCapture::next_frame` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`):
  - `CameraCapture::start` requests `YUYV` first, then `MJPEG`, then `AbsoluteHighestFrameRate` with `RequestedFormat::new::<RgbFormat>` (`src/camera.rs` lines 23-57).
  - `next_frame` captures `raw.decode_image::<RgbFormat>()`, converts to RGB vector, then manually builds BGRA by pushing B,G,R,A (`src/camera.rs` lines 60-107).
- `src/app_execute.rs` consumes `CameraCapture::next_frame` on a background thread, creates `gpui::RenderImage` by wrapping `image::RgbaImage` using BGRA buffer (`src/app_execute.rs` lines 1851-1917).

## Performance Bottlenecks
- `Buffer::decode_image` allocates `Vec<u8>` and creates `ImageBuffer`, so each frame includes:
  - JPEG decoding via `mozjpeg` (CPU, single-thread), per-frame allocation and expansion (`mjpeg_to_rgb` / `buf_mjpeg_to_rgb`), no SIMD.
  - YUYV decoding via `buf_yuyv422_to_rgb` loops over `chunks_exact(4)` and `yuyv444_to_rgb` arithmetic, no SIMD (`types.rs` lines 1603-1693).
  - Script Kit's `bgra` conversion uses a scalar loop over every pixel and pushes 4 bytes per pixel (`src/camera.rs` lines 85-95).
- `image::RgbaImage::from_raw` + `gpui::RenderImage::new` each frame requires CPU memory writes and copy; this occurs in `src/app_execute.rs`.

## Optimization Recommendations
- Use `Camera::frame_raw` (from `nokhwa`), then detect raw format and decode manually:
  - If AVFoundation can provide `kCMPixelFormat_24RGB` or native BGRA (`kCVPixelFormatType_32BGRA`), request that format and skip `decode_image` and RGB→BGRA conversion.
  - If not, implement custom SIMD conversion (e.g., `yuyv422_to_rgb` and BGR↔RGB swap) via `frame_raw` and optimized paths.
- Prefer `Buffer::decode_image_to_buffer` if CPU decoding unavoidable; avoid allocating each frame (`decode_image` -> `Vec<u8>`).
- Use GPU-based conversion: use `frame_raw` + Metal/GPUI conversion to BGRA to move heavy color conversion off CPU if raw frames can be uploaded as textures.

## Performance Notes
- `decode_image` allocates a new `Vec<u8>` (`mjpeg_to_rgb` and `yuyv422_to_rgb`), and then creates `ImageBuffer` — this means per-frame allocations and CPU-bound decoding (especially MJPEG via mozjpeg and YUYV via iterative conversion).
- Using `buffer.decode_image_to_buffer` or custom `frame_raw` avoids these allocations and allows custom SIMD or zero-copy paths.
