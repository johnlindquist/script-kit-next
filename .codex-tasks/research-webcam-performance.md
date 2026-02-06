# Webcam Performance Research Notes

## GPUI APIs and components found

- **RenderImage / gpui::img (GPUI rendering):**
  - `src/prompts/webcam.rs` (`WebcamPrompt`): stores `Option<Arc<RenderImage>>` and renders it via `gpui::img` in `render()` (lines 41-176).
  - `src/app_execute.rs` (`open_webcam`): background thread builds `gpui::RenderImage::new(smallvec::smallvec![image::Frame::new(rgba_img)])` and sends `Arc<RenderImage>` to UI (`open_webcam` lines 1851-1888).
  - `src/list_item.rs` (PNG decoding) provides `decode_png_to_render_image` / `decode_png_to_render_image_with_bgra_conversion` creating `RenderImage` by converting PNGs to BGRA for Metal (`src/list_item.rs` lines ~1290-1350).

- **Surface types / theme mapping (vibrancy):**
  - `src/theme/semantic.rs` defines `Surface` enum and `SurfaceStyle` for UI surface-specific styling (`src/theme/semantic.rs` lines 94-133).
  - `src/theme/gpui_integration.rs` maps theme colors to `gpui_component::ThemeColor` and applies `main_bg` opacity in `map_scriptkit_to_gpui_theme` (`src/theme/gpui_integration.rs` lines 23-106).

## objc/Objective-C patterns (from app_launcher.rs)

- `src/app_launcher.rs` uses Cocoa + Objective-C runtime in `extract_app_icon` (lines 1035-1099):
  - Imports: `cocoa::base::id`, `cocoa::foundation::NSString`, `objc::{class, msg_send, sel, sel_impl}`.
  - Example: `let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];`
  - Uses `NSBitmapImageRep` and `NSPNGFileType` conversion via `msg_send!` to produce raw PNG bytes (`extract_app_icon` lines 1046-1089).

These patterns suggest AVFoundation calls can use the same `msg_send!`/`class!`/`id` approach for native view/layer embedding.

## AVFoundation / AVCaptureVideoPreviewLayer feasibility

- No existing AVFoundation or `AVCapture` integration in the current codebase.
- To use `AVCaptureVideoPreviewLayer`:
  - Use `window_manager::get_main_window()` (from `src/window_manager.rs`) to obtain `NSWindow` pointer.
  - Build or reuse `NSView`/`CALayer` from macOS APIs via `msg_send!` and attach `AVCaptureVideoPreviewLayer` as a sublayer (`NSView` layer) before GPUI renders. No existing helper wrappers.
  - Requires bridging the AVFoundation capture pipeline and ensuring it runs on the main thread (window view mutations).

## CALayer / NSView embedding in GPUI Metal rendering

- `src/platform.rs` shows how GPUI already manipulates `NSVisualEffectView` and layer tree via `msg_send!`, e.g. `configure_visual_effect_views_recursive` and `swizzle_gpui_blurred_view` (`platform.rs` lines 961-1166 and 1359-1470).
- The app manipulates `NSView` and `CALayer` pointers manually, suggesting potential to add a custom `NSView` subview for AVFoundation output.

## Current webcam pipeline (pixel-copy, not zero-copy)

- `src/camera.rs` captures via `nokhwa` (`CameraCapture::start`) and converts each frame to BGRA (`src/camera.rs` lines 9-108), copying pixels to `Vec<u8>` and swapping RGB→BGRA.
- `src/app_execute.rs` creates `image::RgbaImage` and `RenderImage` each frame before pushing through a channel (`open_webcam` lines 1851-1900) — this is CPU copy per frame.

## Comparison to zero-copy GPU approaches

- **Current**: frame decode + conversion CPU (RGB→BGRA), `RgbaImage` allocation each frame, then `RenderImage::new` copies pixels to GPUI/Metal.
- **Zero-copy**: possible via AVFoundation `CVPixelBuffer` / Metal texture sharing, avoiding the `Vec<u8>` allocations and BGRA conversion; no such API currently used.

## Most promising solution outline

1. Build AVFoundation capture (`AVCaptureSession`) in a dedicated background thread and configure `AVCaptureVideoPreviewLayer` attached to a custom `NSView` layered into GPUI's `NSWindow` (`window_manager::get_main_window()`).
2. On macOS, use Objective-C calls (`msg_send!`) to get `contentView` and add a subview with the preview layer. Ensure layer updates run on the main thread.
3. For GPUI rendering fallback, use `RenderImage` as a fallback if CPU conversion remains. But to avoid CPU copying, route the `CVPixelBuffer` or `IOSurface` directly to GPU rendering (requires custom GPUI `RenderImage` creation or Metal texture extension, not currently present).

## Summary and Recommendation

- **Bottlenecks in current nokhwa approach:** CPU RGB→BGRA conversion per frame (`src/camera.rs`), per-frame `Vec<u8>` allocation and `image::RgbaImage` creation (`src/camera.rs`/`src/app_execute.rs`), and GPUI `RenderImage::new` copying pixels into GPU textures each frame.
- **Recommended solution:** Use `AVCaptureVideoPreviewLayer` attached to a native `NSView`/`CALayer` in `NSWindow` via Objective-C runtime calls (`msg_send!`), yielding zero-copy GPU rendering and avoiding CPU color conversion and allocation.
- **Example objc call pattern (mirrors `extract_app_icon` in `src/app_launcher.rs`):**

  ```rust
  use cocoa::base::id;
  use objc::{class, msg_send, sel, sel_impl};

  let window: id = window_id;
  let content_view: id = msg_send![window, contentView];
  let preview_view: id = msg_send![class!(NSView), alloc];
  let preview_layer: id = msg_send![class!(AVCaptureVideoPreviewLayer), layerWithSession: session];
  let _: () = msg_send![preview_view, setWantsLayer: YES];
  let _: () = msg_send![preview_view, setLayer: preview_layer];
  let _: () = msg_send![content_view, addSubview: preview_view];
  ```
- **Why better than CVPixelBuffer/Metal texture approach:** AVCaptureVideoPreviewLayer renders directly by OS-provided Core Animation, requiring no custom GPUI `RenderImage` extensions or `CVPixelBuffer`/Metal texture plumbing, so lower friction and less engineering risk.
- **Pattern alignment:** This embedding approach matches the Objective-C runtime style used in `extract_app_icon` (`src/app_launcher.rs` lines 1035-1099), so the same API surface (class!, msg_send!, id) can be reused.
