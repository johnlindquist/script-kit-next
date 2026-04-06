use std::sync::{Mutex, OnceLock};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};

use super::view::AcpChatView;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

const MODEL_SELECTOR_WIDTH: f32 = 200.0;
const MODEL_SELECTOR_LEFT: f32 = 8.0;
const MODEL_SELECTOR_VERTICAL_PADDING: f32 = 8.0;
const MODEL_SELECTOR_ROW_HEIGHT: f32 = 30.0;

#[cfg(target_os = "macos")]
const NS_WINDOW_ABOVE: i64 = 1;

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupEntry {
    pub(crate) id: String,
    pub(crate) display: SharedString,
    pub(crate) is_selected: bool,
}

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupSnapshot {
    pub(crate) entries: Vec<AcpModelSelectorPopupEntry>,
}

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<AcpChatView>,
    pub(crate) snapshot: AcpModelSelectorPopupSnapshot,
}

#[derive(Clone, Copy)]
struct AcpModelSelectorPopupSlot {
    handle: WindowHandle<AcpModelSelectorPopupWindow>,
    parent_window_handle: AnyWindowHandle,
}

static ACP_MODEL_SELECTOR_POPUP_WINDOW: OnceLock<Mutex<Option<AcpModelSelectorPopupSlot>>> =
    OnceLock::new();

pub(crate) fn close_model_selector_popup_window(cx: &mut App) {
    if let Some(storage) = ACP_MODEL_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

fn popup_height(snapshot: &AcpModelSelectorPopupSnapshot) -> f32 {
    let rows = snapshot.entries.len() as f32;
    (rows * MODEL_SELECTOR_ROW_HEIGHT) + MODEL_SELECTOR_VERTICAL_PADDING
}

fn popup_bounds(
    parent_bounds: Bounds<Pixels>,
    snapshot: &AcpModelSelectorPopupSnapshot,
) -> Bounds<Pixels> {
    let height = popup_height(snapshot);
    let bottom_offset = crate::window_resize::mini_layout::HINT_STRIP_HEIGHT + 4.0;
    let parent_height = parent_bounds.size.height.as_f32();
    let top = (parent_height - bottom_offset - height).max(0.0);

    Bounds {
        origin: gpui::point(
            parent_bounds.origin.x + px(MODEL_SELECTOR_LEFT),
            parent_bounds.origin.y + px(top),
        ),
        size: gpui::size(px(MODEL_SELECTOR_WIDTH), px(height)),
    }
}

pub(crate) fn sync_model_selector_popup_window(
    cx: &mut App,
    request: AcpModelSelectorPopupRequest,
) -> anyhow::Result<()> {
    let AcpModelSelectorPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_id,
        source_view,
        snapshot,
    } = request;

    if snapshot.entries.is_empty() {
        close_model_selector_popup_window(cx);
        return Ok(());
    }

    let bounds = popup_bounds(parent_bounds, &snapshot);
    let storage = ACP_MODEL_SELECTOR_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        if let Some(slot) = *guard {
            if slot.parent_window_handle == parent_window_handle {
                let update_result = slot.handle.update(cx, |popup, window, cx| {
                    popup.set_snapshot(snapshot.clone());
                    set_popup_window_bounds(window, bounds, cx);
                    cx.notify();
                });

                if update_result.is_ok() {
                    return Ok(());
                }

                *guard = None;
            } else {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
                *guard = None;
            }
        }
    }

    let theme = crate::theme::get_cached_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };
    let is_dark_vibrancy = theme.should_use_dark_vibrancy();

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: false,
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| AcpModelSelectorPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
    })?;

    #[cfg(target_os = "macos")]
    {
        let configure_result = handle.update(cx, move |_popup, window, cx| {
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
        });

        if configure_result.is_err() {
            let _ = handle.update(cx, |_popup, window, _cx| {
                window.remove_window();
            });
            return Err(anyhow::anyhow!(
                "failed to configure ACP model selector popup window"
            ));
        }
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(AcpModelSelectorPopupSlot {
            handle,
            parent_window_handle,
        });
    }

    Ok(())
}

pub(crate) struct AcpModelSelectorPopupWindow {
    snapshot: AcpModelSelectorPopupSnapshot,
    source_view: WeakEntity<AcpChatView>,
    focus_handle: FocusHandle,
}

impl AcpModelSelectorPopupWindow {
    fn new(
        snapshot: AcpModelSelectorPopupSnapshot,
        source_view: WeakEntity<AcpChatView>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_snapshot(&mut self, snapshot: AcpModelSelectorPopupSnapshot) {
        self.snapshot = snapshot;
    }

    fn select_model(&self, model_id: &str, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            let model_id = model_id.to_string();
            view.update(cx, |view, cx| {
                view.select_model_from_popup(&model_id, cx);
            });
        } else {
            close_model_selector_popup_window(cx);
        }
    }
}

impl Focusable for AcpModelSelectorPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpModelSelectorPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let accent = theme.colors.accent.selected;

        div()
            .track_focus(&self.focus_handle)
            .id("acp-model-selector-popup")
            .w(px(MODEL_SELECTOR_WIDTH))
            .rounded(px(8.0))
            .bg(gpui::rgba(chrome.dialog_surface_rgba))
            .border_1()
            .border_color(gpui::rgba(chrome.border_rgba))
            .py(px(4.0))
            .children(
                self.snapshot
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let model_id = entry.id.clone();
                        let display = entry.display.clone();
                        let is_selected = entry.is_selected;

                        div()
                            .id(SharedString::from(format!("acp-model-selector-{idx}")))
                            .w_full()
                            .px(px(10.0))
                            .py(px(5.0))
                            .cursor_pointer()
                            .rounded(px(4.0))
                            .mx(px(4.0))
                            .hover(|d| d.bg(gpui::rgba(chrome.hover_rgba)))
                            .when(is_selected, |d| d.bg(gpui::rgba(chrome.selection_rgba)))
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.select_model(&model_id, cx);
                            }))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .when(is_selected, |d| d.text_color(gpui::rgb(accent)))
                                            .child(display),
                                    )
                                    .when(is_selected, |d| {
                                        d.child(
                                            div()
                                                .text_xs()
                                                .text_color(gpui::rgb(accent))
                                                .child("\u{2713}"),
                                        )
                                    }),
                            )
                    }),
            )
    }
}

#[cfg(target_os = "macos")]
fn set_popup_window_bounds(window: &mut Window, bounds: Bounds<Pixels>, cx: &mut App) {
    if let Some(ns_window) = popup_ns_window(window) {
        // SAFETY: `ns_window` comes from a live GPUI popup window on the AppKit
        // main thread. Coordinates are converted from top-left origin into the
        // bottom-left origin NSWindow expects.
        unsafe {
            use cocoa::appkit::NSScreen;
            use cocoa::base::nil;

            let screens: cocoa::base::id = NSScreen::screens(nil);
            let primary_screen: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
            let primary_frame: cocoa::foundation::NSRect = msg_send![primary_screen, frame];
            let primary_height = primary_frame.size.height;
            let width = f32::from(bounds.size.width) as f64;
            let height = f32::from(bounds.size.height) as f64;
            let flipped_y = primary_height - f32::from(bounds.origin.y) as f64 - height;
            let target_frame = cocoa::foundation::NSRect::new(
                cocoa::foundation::NSPoint::new(f32::from(bounds.origin.x) as f64, flipped_y),
                cocoa::foundation::NSSize::new(width, height),
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
fn set_popup_window_bounds(window: &mut Window, bounds: Bounds<Pixels>, cx: &mut App) {
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
        popup_bounds, popup_height, AcpModelSelectorPopupEntry, AcpModelSelectorPopupSnapshot,
        MODEL_SELECTOR_WIDTH,
    };
    use gpui::SharedString;

    #[test]
    fn popup_height_accounts_for_model_rows() {
        let snapshot = AcpModelSelectorPopupSnapshot {
            entries: vec![
                AcpModelSelectorPopupEntry {
                    id: "a".into(),
                    display: SharedString::from("A"),
                    is_selected: false,
                },
                AcpModelSelectorPopupEntry {
                    id: "b".into(),
                    display: SharedString::from("B"),
                    is_selected: true,
                },
            ],
        };

        assert!(popup_height(&snapshot) > 40.0);
    }

    #[test]
    fn popup_bounds_anchor_above_hint_strip() {
        let snapshot = AcpModelSelectorPopupSnapshot {
            entries: vec![AcpModelSelectorPopupEntry {
                id: "a".into(),
                display: SharedString::from("A"),
                is_selected: false,
            }],
        };
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(40.0)),
            size: gpui::size(gpui::px(480.0), gpui::px(440.0)),
        };

        let bounds = popup_bounds(parent, &snapshot);
        assert_eq!(f32::from(bounds.origin.x), 108.0);
        assert!(f32::from(bounds.origin.y) > 40.0);
        assert_eq!(f32::from(bounds.size.width), MODEL_SELECTOR_WIDTH);
    }
}
