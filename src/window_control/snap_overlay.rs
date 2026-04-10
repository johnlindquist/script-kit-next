use std::sync::Mutex;

use anyhow::{Context as _, Result};
use gpui::{
    div, point, px, rgba, size, App, AppContext as _, Context, IntoElement, ParentElement, Pixels,
    Render, Styled, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};

use super::display::get_visible_display_bounds;
use super::snap_session::{SnapOverlayModel, SnapOverlayScene, SnapOverlayTarget};
use super::types::Bounds;

#[cfg(target_os = "macos")]
unsafe fn configure_snap_overlay_window_native(window: cocoa::base::id) {
    use objc::{class, msg_send, sel, sel_impl};

    if window.is_null() {
        tracing::warn!(
            target: "script_kit::snap_overlay",
            event = "snap_overlay_window_null",
            "Cannot configure null snap overlay window"
        );
        return;
    }

    // SAFETY: `window` is a valid NSWindow pointer obtained from GPUI on the main thread.
    let _: () = msg_send![window, setMovable: false];
    let _: () = msg_send![window, setMovableByWindowBackground: false];
    let _: () = msg_send![window, setOpaque: false];

    // SAFETY: `clearColor` is a standard NSColor class method returning a valid autoreleased color.
    let clear: cocoa::base::id = msg_send![class!(NSColor), clearColor];
    let _: () = msg_send![window, setBackgroundColor: clear];

    let _: () = msg_send![window, setHasShadow: false];
    let _: () = msg_send![window, setIgnoresMouseEvents: true];
    let _: () = msg_send![window, setRestorable: false];
    let _: () = msg_send![window, setAnimationBehavior: 2i64];

    // SAFETY: Empty NSString disables frame autosave for this transient overlay window.
    let empty_string: cocoa::base::id = msg_send![class!(NSString), string];
    let _: () = msg_send![window, setFrameAutosaveName: empty_string];
    let _: () = msg_send![window, orderFrontRegardless];
}

// ---------------------------------------------------------------------------
// Per-display overlay window registry
// ---------------------------------------------------------------------------

struct OverlayDisplayWindow {
    display_bounds: Bounds,
    handle: WindowHandle<SnapOverlayView>,
}

static SNAP_OVERLAY_WINDOWS: Mutex<Vec<OverlayDisplayWindow>> = Mutex::new(Vec::new());

fn pixels_to_i32(value: Pixels) -> i32 {
    let px_value: f64 = value.into();
    px_value.round() as i32
}

// ---------------------------------------------------------------------------
// Overlay view
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct SnapOverlayView {
    model: Option<SnapOverlayModel>,
}

impl SnapOverlayView {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn set_model(&mut self, model: Option<SnapOverlayModel>, cx: &mut Context<Self>) {
        let active_tile = model.as_ref().and_then(|m| {
            m.targets
                .iter()
                .find(|t| t.active)
                .map(|t| format!("{:?}", t.tile))
        });

        tracing::info!(
            target: "script_kit::snap_overlay",
            event = "snap_overlay_model_updated",
            has_model = model.is_some(),
            ?active_tile,
            "updated snap overlay model"
        );

        self.model = model;
        cx.notify();
    }
}

impl Render for SnapOverlayView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let Some(ref model) = self.model else {
            return div().size_full();
        };

        let display = model.display_bounds;
        let targets: Vec<SnapOverlayTarget> = model.targets.clone();

        div()
            .absolute()
            .top(px(0.))
            .left(px(0.))
            .w(px(display.width as f32))
            .h(px(display.height as f32))
            .children(targets.into_iter().map(move |target| {
                let rel_x = (target.bounds.x - display.x) as f32;
                let rel_y = (target.bounds.y - display.y) as f32;

                let border = if target.active {
                    0xFFFFFFAA
                } else {
                    0xFFFFFF33
                };
                let fill = if target.active {
                    0xFFFFFF14
                } else {
                    0x00000000
                };

                div()
                    .absolute()
                    .left(px(rel_x))
                    .top(px(rel_y))
                    .w(px(target.bounds.width as f32))
                    .h(px(target.bounds.height as f32))
                    .rounded(px(10.))
                    .border_1()
                    .border_dashed()
                    .border_color(rgba(border))
                    .bg(rgba(fill))
            }))
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Ensure one overlay window exists per connected display.
///
/// Idempotent — returns immediately if windows are already open.
pub fn ensure_snap_overlay_windows(cx: &mut App) -> Result<()> {
    let mut guard = SNAP_OVERLAY_WINDOWS
        .lock()
        .map_err(|e| anyhow::anyhow!("snap overlay lock poisoned: {e}"))?;

    if !guard.is_empty() {
        return Ok(());
    }

    for display in cx.displays() {
        let full = display.bounds();
        let full_x = pixels_to_i32(full.origin.x);
        let full_y = pixels_to_i32(full.origin.y);
        let full_w = pixels_to_i32(full.size.width);
        let full_h = pixels_to_i32(full.size.height);

        let visible = get_visible_display_bounds(full_x + (full_w / 2), full_y + (full_h / 2));

        let local_x = visible.x - full_x;
        let local_y = visible.y - full_y;

        let overlay_bounds = gpui::Bounds {
            origin: point(px(local_x as f32), px(local_y as f32)),
            size: size(px(visible.width as f32), px(visible.height as f32)),
        };

        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(overlay_bounds)),
                    display_id: Some(display.id()),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    focus: false,
                    show: false,
                    kind: WindowKind::PopUp,
                    is_movable: false,
                    ..Default::default()
                },
                |_window, cx| cx.new(|_| SnapOverlayView::new()),
            )
            .context("failed to open snap overlay window")?;

        // Configure the native window as click-through, transparent, and non-restorable.
        #[cfg(target_os = "macos")]
        {
            let _ = handle.update(cx, |_view, window, _cx| {
                if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
                    if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
                        use objc::{msg_send, sel, sel_impl};
                        let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
                        // SAFETY: ns_view is a valid NSView from a just-created GPUI window.
                        // We obtain the parent NSWindow via -[NSView window] on the main thread.
                        unsafe {
                            let ns_window: cocoa::base::id = msg_send![ns_view, window];
                            configure_snap_overlay_window_native(ns_window);
                        }
                    }
                }
            });
        }

        guard.push(OverlayDisplayWindow {
            display_bounds: visible,
            handle,
        });
    }

    tracing::info!(
        target: "script_kit::snap_overlay",
        event = "snap_overlay_windows_opened",
        window_count = guard.len(),
        "opened snap overlay windows"
    );

    Ok(())
}

/// Show the snap overlay scene, distributing models to matching display windows.
///
/// Each overlay window receives the model whose `display_bounds` matches its
/// own, or `None` if no model in the scene covers that display.
pub fn show_snap_overlay(scene: SnapOverlayScene, cx: &mut App) -> Result<()> {
    ensure_snap_overlay_windows(cx)?;

    let guard = SNAP_OVERLAY_WINDOWS
        .lock()
        .map_err(|e| anyhow::anyhow!("snap overlay lock poisoned: {e}"))?;

    for overlay in guard.iter() {
        let model_for_window = scene
            .displays
            .iter()
            .find(|m| m.display_bounds == overlay.display_bounds)
            .cloned();

        let _ = overlay.handle.update(cx, |view, _window, cx| {
            view.set_model(model_for_window, cx);
        });
    }

    tracing::info!(
        target: "script_kit::snap_overlay",
        event = "snap_overlay_scene_updated",
        display_count = scene.displays.len(),
        ?scene.mode,
        "updated snap overlay scene"
    );

    Ok(())
}

/// Hide the snap overlay on all displays.
pub fn hide_snap_overlay(cx: &mut App) -> Result<()> {
    let guard = SNAP_OVERLAY_WINDOWS
        .lock()
        .map_err(|e| anyhow::anyhow!("snap overlay lock poisoned: {e}"))?;

    for overlay in guard.iter() {
        let _ = overlay.handle.update(cx, |view, _window, cx| {
            view.set_model(None, cx);
        });
    }

    tracing::info!(
        target: "script_kit::snap_overlay",
        event = "snap_overlay_hidden",
        "hid snap overlay"
    );

    Ok(())
}
