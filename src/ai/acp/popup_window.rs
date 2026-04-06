use gpui::{
    px, AnyWindowHandle, App, AppContext, Bounds, DisplayId, Pixels, Window, WindowBounds,
    WindowHandle, WindowKind, WindowOptions,
};

use crate::ai::context_picker_row::CONTEXT_PICKER_ROW_HEIGHT;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

pub(crate) const DENSE_PICKER_MAX_VISIBLE_ROWS: usize = 8;
pub(crate) const DENSE_PICKER_VERTICAL_PADDING: f32 = 4.0;
pub(crate) const DENSE_PICKER_EMPTY_HEIGHT: f32 = 56.0;
pub(crate) const DENSE_PICKER_DEFAULT_WIDTH: f32 = 320.0;
pub(crate) const DENSE_PICKER_MIN_WIDTH: f32 = 200.0;
pub(crate) const DENSE_PICKER_EDGE_GUTTER: f32 = 12.0;
pub(crate) const DENSE_PICKER_LEFT_MARGIN: f32 = 8.0;

#[cfg(target_os = "macos")]
const NS_WINDOW_ABOVE: i64 = 1;

pub(crate) fn dense_picker_height(item_count: usize) -> f32 {
    if item_count == 0 {
        return DENSE_PICKER_EMPTY_HEIGHT;
    }

    let visible_rows = item_count.min(DENSE_PICKER_MAX_VISIBLE_ROWS) as f32;
    (visible_rows * CONTEXT_PICKER_ROW_HEIGHT) + (DENSE_PICKER_VERTICAL_PADDING * 2.0)
}

pub(crate) fn dense_picker_width_for_window(window_width: f32) -> f32 {
    let max_width =
        (window_width - (DENSE_PICKER_EDGE_GUTTER * 2.0)).min(DENSE_PICKER_DEFAULT_WIDTH);
    max_width.max(DENSE_PICKER_MIN_WIDTH)
}

pub(crate) fn footer_anchored_popup_top(parent_height: f32, popup_height: f32) -> f32 {
    let bottom_offset = crate::window_resize::mini_layout::HINT_STRIP_HEIGHT + 4.0;
    (parent_height - bottom_offset - popup_height).max(0.0)
}

pub(crate) fn popup_bounds(
    parent_bounds: Bounds<Pixels>,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> Bounds<Pixels> {
    Bounds {
        origin: gpui::point(
            parent_bounds.origin.x + px(left),
            parent_bounds.origin.y + px(top),
        ),
        size: gpui::size(px(width), px(height)),
    }
}

pub(crate) fn popup_window_options(
    bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
) -> WindowOptions {
    let theme = crate::theme::get_cached_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: false,
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    }
}

pub(crate) fn configure_popup_window<T: 'static>(
    handle: &WindowHandle<T>,
    cx: &mut App,
    parent_window_handle: AnyWindowHandle,
) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let is_dark_vibrancy = crate::theme::get_cached_theme().should_use_dark_vibrancy();
        handle
            .update(cx, move |_popup, window, cx| {
                window.defer(cx, move |window, cx| {
                    if let Some(ns_window) = popup_ns_window(window) {
                        // SAFETY: `ns_window` comes from the live GPUI popup window on the
                        // main thread and is nil-checked before configuration.
                        unsafe {
                            crate::platform::configure_actions_popup_window(
                                ns_window,
                                is_dark_vibrancy,
                            );
                        }
                        attach_popup_to_parent_window(cx, parent_window_handle, ns_window);
                    }
                });
            })
            .map_err(|_| anyhow::anyhow!("failed to configure ACP popup window"))?;
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (handle, cx, parent_window_handle);

    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn flipped_ns_window_y(bounds: Bounds<Pixels>, primary_height: f64) -> f64 {
    primary_height - f32::from(bounds.origin.y) as f64 - f32::from(bounds.size.height) as f64
}

#[cfg(target_os = "macos")]
pub(crate) fn set_popup_window_bounds(window: &mut Window, bounds: Bounds<Pixels>, cx: &mut App) {
    if let Some(ns_window) = popup_ns_window(window) {
        // SAFETY: `ns_window` comes from a live GPUI popup window on the AppKit
        // main thread. Coordinates are converted from GPUI's screen-relative
        // top-left origin into the bottom-left origin NSWindow expects.
        unsafe {
            use cocoa::appkit::NSScreen;
            use cocoa::base::nil;

            let screens: cocoa::base::id = NSScreen::screens(nil);
            let primary_screen: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
            let primary_frame: cocoa::foundation::NSRect = msg_send![primary_screen, frame];
            let primary_height = primary_frame.size.height;
            let target_frame = cocoa::foundation::NSRect::new(
                cocoa::foundation::NSPoint::new(
                    f32::from(bounds.origin.x) as f64,
                    flipped_ns_window_y(bounds, primary_height),
                ),
                cocoa::foundation::NSSize::new(
                    f32::from(bounds.size.width) as f64,
                    f32::from(bounds.size.height) as f64,
                ),
            );
            let _: () = msg_send![
                ns_window,
                setFrame: target_frame
                display: true
                animate: false
            ];
        }
    }

    window.resize(bounds.size);
    window.bounds_changed(cx);
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn set_popup_window_bounds(window: &mut Window, bounds: Bounds<Pixels>, cx: &mut App) {
    let _ = cx;
    window.resize(bounds.size);
}

#[cfg(target_os = "macos")]
fn popup_ns_window(window: &mut Window) -> Option<cocoa::base::id> {
    if let Ok(window_handle) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = window_handle.as_raw() {
            use cocoa::base::nil;

            let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
            // SAFETY: `ns_view` comes from the live GPUI window on the main thread.
            unsafe {
                let ns_window: cocoa::base::id = msg_send![ns_view, window];
                if ns_window != nil {
                    return Some(ns_window);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn attach_popup_to_parent_window(
    cx: &mut App,
    parent_window_handle: AnyWindowHandle,
    child_ns_window: cocoa::base::id,
) {
    let _ = cx.update_window(parent_window_handle, move |_, parent_window, _cx| {
        let Some(parent_ns_window) = popup_ns_window(parent_window) else {
            return;
        };

        // SAFETY: both NSWindow pointers come from live GPUI windows on the main
        // thread, and nil/equality are guarded before AppKit receives them.
        unsafe {
            use cocoa::base::nil;

            if parent_ns_window == nil
                || child_ns_window == nil
                || parent_ns_window == child_ns_window
            {
                return;
            }

            let _: () = msg_send![
                parent_ns_window,
                addChildWindow: child_ns_window
                ordered: NS_WINDOW_ABOVE
            ];
            let _: () = msg_send![child_ns_window, orderFrontRegardless];
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        dense_picker_height, dense_picker_width_for_window, footer_anchored_popup_top,
        popup_bounds, DENSE_PICKER_DEFAULT_WIDTH, DENSE_PICKER_MIN_WIDTH,
    };

    #[test]
    fn dense_picker_height_uses_shared_row_contract() {
        assert!(dense_picker_height(0) > 0.0);
        assert!(dense_picker_height(12) >= dense_picker_height(8));
        assert_eq!(dense_picker_height(12), dense_picker_height(8));
    }

    #[test]
    fn dense_picker_width_matches_window_constraints() {
        assert_eq!(
            dense_picker_width_for_window(900.0),
            DENSE_PICKER_DEFAULT_WIDTH
        );
        assert_eq!(dense_picker_width_for_window(180.0), DENSE_PICKER_MIN_WIDTH);
    }

    #[test]
    fn footer_anchor_keeps_popup_above_hint_strip() {
        assert!(footer_anchored_popup_top(400.0, 80.0) >= 0.0);
    }

    #[test]
    fn popup_bounds_offset_from_parent_origin() {
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(40.0)),
            size: gpui::size(gpui::px(600.0), gpui::px(400.0)),
        };

        let bounds = popup_bounds(parent, 8.0, 16.0, 200.0, 80.0);
        assert_eq!(f32::from(bounds.origin.x), 108.0);
        assert_eq!(f32::from(bounds.origin.y), 56.0);
        assert_eq!(f32::from(bounds.size.width), 200.0);
        assert_eq!(f32::from(bounds.size.height), 80.0);
    }
}
