use std::sync::{Mutex, OnceLock};

use gpui::{
    div, point, prelude::*, px, rgba, size, AnyWindowHandle, App, Bounds, Pixels, Render, Window,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};

use crate::theme::get_cached_theme;

/// NSWindowOrderingMode::NSWindowAbove — place child above parent.
const NS_WINDOW_ABOVE: i64 = 1;

static FOOTER_WINDOW: OnceLock<Mutex<Option<WindowHandle<MainFooterPopupWindow>>>> =
    OnceLock::new();
static FOOTER_BOUNDS: OnceLock<Mutex<Option<Bounds<Pixels>>>> = OnceLock::new();

fn footer_popup_bounds(parent_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    let height = px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT);
    let x = parent_bounds.origin.x;
    let y = parent_bounds.origin.y + parent_bounds.size.height - height;

    Bounds {
        origin: point(x, y),
        size: size(parent_bounds.size.width, height),
    }
}

fn main_window_bounds() -> Option<Bounds<Pixels>> {
    let (x, y, width, height) = crate::platform::get_main_window_bounds()?;
    Some(Bounds {
        origin: point(px(x as f32), px(y as f32)),
        size: size(px(width as f32), px(height as f32)),
    })
}

pub(crate) fn sync_main_footer_popup(should_show: bool, cx: &mut App) {
    let desired_bounds = should_show
        .then(main_window_bounds)
        .flatten()
        .map(footer_popup_bounds);
    let current_bounds = FOOTER_BOUNDS
        .get()
        .and_then(|storage| storage.lock().ok())
        .and_then(|guard| *guard);
    let window_open = FOOTER_WINDOW
        .get()
        .and_then(|storage| storage.lock().ok())
        .is_some_and(|guard| guard.is_some());

    match desired_bounds {
        Some(bounds) if window_open && current_bounds == Some(bounds) => {}
        Some(bounds) => {
            close_main_footer_popup(cx);
            open_main_footer_popup(bounds, cx);
        }
        None => close_main_footer_popup(cx),
    }
}

pub(crate) fn notify_main_footer_popup(cx: &mut App) {
    let Some(storage) = FOOTER_WINDOW.get() else {
        return;
    };
    let Ok(guard) = storage.lock() else {
        return;
    };
    let Some(handle) = *guard else {
        return;
    };
    let _ = handle.update(cx, |_view, _window, cx| {
        cx.notify();
    });
}

pub(crate) fn close_main_footer_popup(cx: &mut App) {
    if let Some(storage) = FOOTER_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.update(cx, |_view, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }

    if let Some(storage) = FOOTER_BOUNDS.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

fn open_main_footer_popup(bounds: Bounds<Pixels>, cx: &mut App) {
    let Some(parent_window_handle) = crate::get_main_window_handle() else {
        return;
    };

    let theme = get_cached_theme();
    let is_dark_vibrancy = theme.should_use_dark_vibrancy();
    let window_background = if theme.is_vibrancy_enabled() {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    };

    let handle = match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            window_background,
            focus: false,
            show: true,
            kind: WindowKind::PopUp,
            ..Default::default()
        },
        |_window, cx| cx.new(|_cx| MainFooterPopupWindow),
    ) {
        Ok(handle) => handle,
        Err(error) => {
            tracing::warn!(
                target: "script_kit::footer_popup",
                event = "footer_popup_open_failed",
                error = ?error,
                "Failed to open main footer popup"
            );
            return;
        }
    };

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, move |_view, window, cx| {
            window.defer(cx, move |window, cx| {
                if let Some(ns_window) = footer_popup_ns_window(window) {
                    // SAFETY: `ns_window` comes from the live GPUI popup window on
                    // the AppKit main thread when the deferred callback runs.
                    unsafe {
                        crate::platform::configure_footer_popup_window(ns_window, is_dark_vibrancy);
                    }
                    attach_footer_popup_to_parent_window(cx, parent_window_handle, ns_window);
                }
            });
        });
    }

    let storage = FOOTER_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        *guard = Some(handle);
    }

    let bounds_storage = FOOTER_BOUNDS.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = bounds_storage.lock() {
        *guard = Some(bounds);
    }
}

#[cfg(target_os = "macos")]
fn footer_popup_ns_window(window: &mut Window) -> Option<cocoa::base::id> {
    if let Ok(window_handle) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = window_handle.as_raw() {
            use cocoa::base::nil;
            use objc::{msg_send, sel, sel_impl};

            let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
            // SAFETY: `ns_view` comes from the live GPUI popup window on the AppKit
            // main thread. `-[NSView window]` returns the owning NSWindow or nil.
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
fn attach_footer_popup_to_parent_window(
    cx: &mut App,
    parent_window_handle: AnyWindowHandle,
    child_ns_window: cocoa::base::id,
) {
    let _ = cx.update_window(parent_window_handle, move |_, parent_window, _cx| {
        let Some(parent_ns_window) = footer_popup_ns_window(parent_window) else {
            return;
        };

        // SAFETY: both NSWindow pointers come from live GPUI windows on the main
        // thread. We guard against nil/equal pointers before attaching.
        unsafe {
            use cocoa::base::nil;
            use objc::{msg_send, sel, sel_impl};

            if parent_ns_window == nil
                || child_ns_window == nil
                || parent_ns_window == child_ns_window
            {
                return;
            }

            let _: () =
                msg_send![parent_ns_window, addChildWindow:child_ns_window ordered:NS_WINDOW_ABOVE];
            let _: () = msg_send![child_ns_window, orderFrontRegardless];
        }
    });
}

struct MainFooterPopupWindow;

impl Render for MainFooterPopupWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = get_cached_theme();
        let hint_text_hex = theme.colors.text.primary;
        let hint_opacity_byte =
            (crate::window_resize::mini_layout::HINT_TEXT_OPACITY * 255.0).round() as u32;
        let hint_text_rgba = (hint_text_hex << 8) | hint_opacity_byte;
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);

        div()
            .w_full()
            .h(px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT))
            .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
            .py(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_Y))
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .border_t(px(crate::window_resize::mini_layout::DIVIDER_HEIGHT))
            .border_color(rgba(chrome.divider_rgba))
            .child(crate::components::render_hint_icons(
                &["↵ Run", "⌘K Actions", "Tab AI"],
                hint_text_rgba,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::footer_popup_bounds;
    use gpui::{point, px, size, Bounds};

    #[test]
    fn footer_popup_bounds_bottom_aligns_to_parent() {
        let parent_bounds = Bounds {
            origin: point(px(120.), px(80.)),
            size: size(px(480.), px(440.)),
        };

        let bounds = footer_popup_bounds(parent_bounds);

        assert_eq!(bounds.origin.x, px(120.));
        assert_eq!(bounds.origin.y, px(490.));
        assert_eq!(bounds.size.width, px(480.));
        assert_eq!(
            bounds.size.height,
            px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT)
        );
    }
}
