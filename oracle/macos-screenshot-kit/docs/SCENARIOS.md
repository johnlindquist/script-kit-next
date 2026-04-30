# Screenshot scenario checklist

This crate intentionally covers more than "take a PNG of the screen". The table below maps common product needs to API choices.

## Screenshot app scenarios

| Need | Use |
|---|---|
| Full desktop screenshot | `CaptureTarget::AllDisplays` |
| Per-monitor files | `ScreenshotClient::capture_each_display` |
| Main monitor only | `CaptureTarget::MainDisplay` |
| Specific monitor from the system screenshot tool | `CaptureTarget::DisplayOrdinal(n)` |
| Specific monitor from CoreGraphics | `CaptureTarget::Display(display_id)` |
| Drag-to-select rectangle | `CaptureTarget::InteractiveSelection` |
| Click-to-select window | `CaptureTarget::InteractiveWindow` |
| Shift-Command-5 style toolbar | `CaptureTarget::InteractiveToolbar` |
| Delayed screenshot for menus | `CaptureOptions::with_delay` |
| Include mouse pointer | `CaptureOptions::with_cursor(true)` |
| Omit window shadow | `CaptureOptions::without_window_shadow()` |
| Capture only a shadow | `WindowFrameMode::ShadowOnly` with CoreGraphics |
| Copy result to clipboard | `ScreenshotClient::capture_to_clipboard` |
| Touch Bar | `CaptureTarget::TouchBar` |
| PDF output | `ImageFormat::Pdf` |

## Window manager and launcher scenarios

| Need | Use |
|---|---|
| Enumerate displays | `ScreenshotClient::displays()` |
| Enumerate user windows | `ScreenshotClient::windows(WindowListOptions::default())` |
| Include menus/popovers/tooltips | `WindowListOptions::visible_all_layers()` |
| Get frontmost window screenshot | `CaptureTarget::FrontmostWindow` |
| Get hovered window screenshot | `CaptureTarget::WindowUnderCursor` |
| Hit-test a point | `ScreenshotClient::windows_at_point(point, options)` |
| Capture windows for an application PID | `CaptureTarget::Application(pid)` |
| Capture visible desktop excluding your overlay process | `CaptureTarget::VisibleWindows { exclude_pids: vec![std::process::id() as i32], .. }` |

## Automation / computer vision scenarios

| Need | Use |
|---|---|
| Pixel buffer for OCR or CV | `CapturedImage::to_rgba8()` |
| Stable target image for a known window | `CaptureTarget::Window(window_id)` |
| Crop around an accessibility element | `CaptureTarget::Region(rect)` from the AX frame |
| Capture an app while excluding own UI | `VisibleWindows` with PID/window exclusions |
| Capture protected/DRM content | Not reliably possible; macOS may blank it |
| Capture hidden/minimized windows | Not reliable; most captures require visible content |

## Backend rules of thumb

- Use `CaptureBackend::Auto` unless you have a reason not to.
- Use `CoreGraphics` for deterministic, programmatic, fast still captures.
- Use `SystemScreencapture` for cursor capture, interactive picks, Touch Bar, and clipboard parity with macOS.
- Use the optional `screen-capture-kit` re-export for real-time streams, IOSurface, Metal, HDR, app/window filters, audio, and newest macOS capture controls.
