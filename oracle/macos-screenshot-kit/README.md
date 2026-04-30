# macos-screenshot-kit

A standalone Rust library for macOS still screenshots. It is designed for the same messy real-world cases handled by screenshot apps, visual agents, launchers, QA automation, window managers, and support tools.

The crate exposes one high-level synchronous API:

```rust
use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(CaptureTarget::AllDisplays, CaptureOptions::png())?;
    image.save("desktop.png")?;
    Ok(())
}
```

## What it covers

| Scenario | API |
|---|---|
| Entire virtual desktop | `CaptureTarget::AllDisplays` |
| Main display | `CaptureTarget::MainDisplay` |
| Display by system screenshot ordinal | `CaptureTarget::DisplayOrdinal(1)` |
| Specific display by `CGDirectDisplayID` | `CaptureTarget::Display(id)` |
| Each monitor as a separate image | `ScreenshotClient::capture_each_display(...)` |
| Global rectangle, including across monitors | `CaptureTarget::Region(Rect)` |
| Display-relative rectangle | `CaptureTarget::DisplayRegion { display_id, rect }` |
| Specific window by `CGWindowID` | `CaptureTarget::Window(id)` |
| Multiple windows composited | `CaptureTarget::Windows(vec![...])` |
| Frontmost/topmost window | `CaptureTarget::FrontmostWindow` |
| Window under mouse | `CaptureTarget::WindowUnderCursor` |
| Window at a point | `CaptureTarget::WindowAtPoint(Point)` |
| All visible windows for a PID | `CaptureTarget::Application(pid)` |
| Visible windows excluding overlay/app | `CaptureTarget::VisibleWindows { exclude_window_ids, exclude_pids, include_all_layers }` |
| Interactive picker | `CaptureTarget::Interactive` |
| Interactive region picker | `CaptureTarget::InteractiveSelection` |
| Interactive window picker | `CaptureTarget::InteractiveWindow` |
| Screenshot toolbar UI | `CaptureTarget::InteractiveToolbar` |
| Touch Bar screenshot | `CaptureTarget::TouchBar` |
| Capture with cursor | `CaptureOptions::with_cursor(true)`; uses the system backend |
| Delay/timer screenshots | `CaptureOptions::with_delay(Duration)` |
| Window with or without shadow | `WindowFrameMode` / `CaptureOptions::without_window_shadow()` |
| Copy to clipboard | `ScreenshotClient::capture_to_clipboard(...)` |
| PNG/JPEG/TIFF/HEIC/PDF/BMP bytes | `CapturedImage::to_bytes(ImageFormat)` |
| Save to file | `CapturedImage::save(...)` / `save_as(...)` |
| Raw pixels | `CapturedImage::to_rgba8()` |
| Window enumeration | `ScreenshotClient::windows(WindowListOptions)` |
| Display enumeration | `ScreenshotClient::displays()` |
| Permission check/request | `permission_status()`, `request_permission()`, `open_screen_recording_settings()` |

## Backends

### CoreGraphics backend

Fast native still screenshots via CoreGraphics and ImageIO. This backend supports displays, windows, regions, multi-window composites, app-by-PID composites, ImageIO encoding, and RGBA pixels.

It is the best default for programmatic screenshot tools that already know what they want to capture.

### System `screencapture` backend

The crate can call `/usr/sbin/screencapture` without a shell. This is deliberate: macOS's system tool handles interactive selection/window capture and cursor inclusion with behavior users already understand.

Use it explicitly:

```rust
use macos_screenshot_kit::{CaptureBackend, CaptureOptions};

let options = CaptureOptions::png().with_backend(CaptureBackend::SystemScreencapture);
```

Or let `CaptureBackend::Auto` choose it when the target is interactive or `include_cursor` is true.

### ScreenCaptureKit feature

For real-time streams, IOSurface/Metal, HDR, app/window filtering beyond still screenshots, or the newest macOS capture controls, enable the optional re-export:

```toml
[dependencies]
macos-screenshot-kit = { path = "../macos-screenshot-kit", features = ["screen-capture-kit"] }
```

Then use:

```rust
use macos_screenshot_kit::screen_capture_kit::prelude::*;
```

The high-level `ScreenshotClient` remains synchronous and still-image focused. ScreenCaptureKit is a broader framework and is best consumed directly when you need streams, sample buffers, IOSurface, audio, or HDR.

## Permission notes

macOS requires Screen Recording permission for screen capture. During development, grant permission to the executable that launches the process, often Terminal, your IDE, or your app bundle.

```rust
let client = macos_screenshot_kit::ScreenshotClient::new();
if !client.permission_status().is_granted() {
    let _ = client.open_screen_recording_settings();
    let _ = client.request_permission();
}
```

After changing permission, macOS often requires you to restart the app.

## Examples

```bash
cargo run --example capture_all
cargo run --example list_windows
cargo run --example capture_window
cargo run --example capture_region
cargo run --example interactive_selection
cargo run --example capture_with_cursor
cargo run --example interactive_toolbar
cargo run --example touch_bar
cargo run --example raw_pixels
cargo run --example copy_frontmost_to_clipboard
cargo run --example exclude_current_process
```

See `docs/SCENARIOS.md` for the broader product-use-case checklist.

## Design choices

- **No shell execution.** The system backend uses `std::process::Command` with fixed executable paths and argument arrays.
- **No mandatory third-party runtime.** The default crate has no required dependencies.
- **Native image encoding.** ImageIO writes PNG, JPEG, TIFF, HEIC, PDF, and BMP, so output matches macOS platform behavior.
- **Practical fallback model.** CoreGraphics covers deterministic captures; the system backend covers user-interactive and cursor cases; ScreenCaptureKit is available as an optional low-level path for modern streaming/HDR scenarios.

## Known limits

- CoreGraphics still screenshots do not include the cursor. Use `CaptureBackend::Auto` with `include_cursor = true`, which routes through `/usr/sbin/screencapture`.
- The system backend captures one window ID at a time. Use CoreGraphics for multi-window composites.
- Minimized or hidden windows may be unavailable or stale depending on the app and macOS version.
- Some protected content may render blank due to macOS privacy or DRM rules.
- Display and region coordinates use global macOS display coordinates. Retina output may have more pixels than point-sized bounds.
