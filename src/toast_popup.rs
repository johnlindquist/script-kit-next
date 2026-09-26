//! Toasts for the Notes window, shown in their own small glass window.
//!
//! A toast drawn inside a window cannot blur the window's own content, so over
//! the Notes editor it reads as text on text. This panel is the same kind of
//! window as the confirm popup: a non-activating child attached to its parent,
//! with the popup backdrop, entering and leaving with the same glass animation.
//! It sits at the bottom of the parent and is never wider than it. One toast
//! shows at a time; a newer one replaces it.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    div, point, prelude::*, px, rgb, rgba, size, AnyWindowHandle, App, Bounds, Context, DisplayId,
    Pixels, Render, SharedString, Subscription, Window, WindowBackgroundAppearance, WindowBounds,
    WindowHandle, WindowKind, WindowOptions,
};

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::theme::get_cached_theme;

const TOAST_POPUP_MAX_WIDTH_PX: f32 = 360.0;
const TOAST_POPUP_HEIGHT_PX: f32 = 44.0;
/// Gap between the toast and its parent's side and bottom edges.
const TOAST_POPUP_PARENT_INSET_PX: f32 = 12.0;
const TOAST_POPUP_RADIUS_PX: f32 = crate::components::confirm_modal_shell::CONFIRM_MODAL_RADIUS;
const TOAST_POPUP_LOG_TARGET: &str = "TOAST";
const TOAST_POPUP_WINDOW_NAME: &str = "Toast popup";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToastPopupKind {
    Info,
    Success,
    Error,
}

impl ToastPopupKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

type ToastPopupCallback = Rc<dyn Fn(&mut Window, &mut App)>;

/// A button on the toast. Its callback runs against the parent window.
#[derive(Clone)]
pub(crate) struct ToastPopupAction {
    pub(crate) label: SharedString,
    pub(crate) shortcut: Option<SharedString>,
    pub(crate) on_click: ToastPopupCallback,
}

pub(crate) struct ToastPopupRequest {
    /// Identity: showing the same key again replaces it in place, and
    /// [`dismiss_toast_popup`] closes only the toast with this key.
    pub(crate) key: SharedString,
    pub(crate) message: SharedString,
    pub(crate) kind: ToastPopupKind,
    /// `None` keeps the toast until it is replaced, dismissed, or clicked.
    pub(crate) duration_ms: Option<u64>,
    pub(crate) action: Option<ToastPopupAction>,
}

struct CurrentToast {
    id: u64,
    key: SharedString,
    message: SharedString,
    kind: ToastPopupKind,
    action_label: Option<SharedString>,
    parent: AnyWindowHandle,
    window: Option<WindowHandle<ToastPopupView>>,
    _parent_closed: Option<Subscription>,
}

thread_local! {
    static CURRENT: RefCell<Option<CurrentToast>> = const { RefCell::new(None) };
    static NEXT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

/// Show `request` attached to `parent`, replacing any toast already showing.
/// Safe to call from render: the window opens on the next turn.
pub(crate) fn show_toast_popup(parent: &mut Window, request: ToastPopupRequest, cx: &mut App) {
    let id = NEXT_ID.get();
    NEXT_ID.set(id + 1);
    let parent_handle = parent.window_handle();
    let parent_bounds = parent.bounds();
    let display_id = parent.display(cx).map(|display| display.id());
    let parent_hidden = parent.is_owned_hidden();

    close_current(cx);
    CURRENT.set(Some(CurrentToast {
        id,
        key: request.key.clone(),
        message: request.message.clone(),
        kind: request.kind,
        action_label: request.action.as_ref().map(|action| action.label.clone()),
        parent: parent_handle,
        window: None,
        _parent_closed: None,
    }));

    let duration_ms = request.duration_ms;
    cx.defer(move |cx| {
        if !is_current(id) {
            return;
        }
        // A hidden (test-owned) parent keeps the toast as state only, so
        // automation still sees it without a visible window.
        if !parent_hidden {
            match open_toast_window(id, parent_handle, parent_bounds, display_id, request, cx) {
                Ok(window) => CURRENT.with_borrow_mut(|current| {
                    if let Some(current) = current.as_mut().filter(|current| current.id == id) {
                        current.window = Some(window);
                    }
                }),
                Err(error) => tracing::warn!(%error, "toast_popup_open_failed"),
            }
        }
        let parent_closed = cx.on_window_closed(move |cx, window_id| {
            if window_id == parent_handle.window_id() {
                close_if_current(id, cx);
            }
        });
        CURRENT.with_borrow_mut(|current| {
            if let Some(current) = current.as_mut().filter(|current| current.id == id) {
                current._parent_closed = Some(parent_closed);
            }
        });
        if let Some(duration_ms) = duration_ms {
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(duration_ms))
                    .await;
                cx.update(|cx| close_if_current(id, cx));
            })
            .detach();
        }
    });
}

/// Close the toast with `key`, if it is the one showing.
pub(crate) fn dismiss_toast_popup(key: &str, cx: &mut App) {
    let matches = CURRENT.with_borrow(|current| {
        current
            .as_ref()
            .is_some_and(|current| current.key.as_ref() == key)
    });
    if matches {
        close_current(cx);
    }
}

/// The toast showing over `parent`, as an automation element on the parent.
pub(crate) fn semantic_element(parent: AnyWindowHandle) -> Option<crate::protocol::ElementInfo> {
    CURRENT.with_borrow(|current| {
        let current = current
            .as_ref()
            .filter(|current| current.parent == parent)?;
        let mut element = crate::protocol::ElementInfo::button(0, &current.message);
        element.semantic_id = format!("toast:{}", current.key);
        element.element_type = crate::protocol::ElementType::Panel;
        element.index = None;
        element.value = current.action_label.as_ref().map(|label| label.to_string());
        element.kind = Some("toastPopup".into());
        element.status_kind = Some(current.kind.as_str().into());
        element.source = Some("toast_popup".into());
        Some(element)
    })
}

fn is_current(id: u64) -> bool {
    CURRENT.with_borrow(|current| current.as_ref().is_some_and(|current| current.id == id))
}

fn close_if_current(id: u64, cx: &mut App) {
    if is_current(id) {
        close_current(cx);
    }
}

fn close_current(cx: &mut App) {
    let Some(current) = CURRENT.take() else {
        return;
    };
    let Some(window) = current.window else {
        return;
    };
    let _ = window.update(cx, |_, window, cx| {
        if window.is_owned_hidden() {
            window.remove_window();
        } else {
            crate::platform::dematerialize_then_remove_gpui_window(
                window,
                cx,
                TOAST_POPUP_LOG_TARGET,
                TOAST_POPUP_WINDOW_NAME,
            );
        }
    });
}

/// Bottom-center of the parent, inset from its edges, kept on screen.
fn toast_popup_bounds(parent: Bounds<Pixels>, display: Option<Bounds<Pixels>>) -> Bounds<Pixels> {
    let inset = px(TOAST_POPUP_PARENT_INSET_PX);
    let width = (parent.size.width - inset * 2.0)
        .min(px(TOAST_POPUP_MAX_WIDTH_PX))
        .max(px(0.0));
    let height = px(TOAST_POPUP_HEIGHT_PX);
    let mut x = parent.center().x - width / 2.0;
    let mut y = parent.bottom() - inset - height;
    if let Some(display) = display {
        x = x
            .max(display.left() + inset)
            .min(display.right() - inset - width);
        y = y
            .max(display.top() + inset)
            .min(display.bottom() - inset - height);
    }
    Bounds {
        origin: point(x, y),
        size: size(width, height),
    }
}

fn open_toast_window(
    id: u64,
    parent: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    request: ToastPopupRequest,
    cx: &mut App,
) -> anyhow::Result<WindowHandle<ToastPopupView>> {
    let theme = get_cached_theme();
    let is_dark_vibrancy = theme.should_use_dark_vibrancy();
    let window_background = if theme.is_vibrancy_enabled() {
        crate::platform::vibrancy_window_background()
    } else {
        WindowBackgroundAppearance::Opaque
    };
    let display_bounds = display_id
        .and_then(|id| cx.find_display(id))
        .map(|display| display.visible_bounds());
    let bounds = toast_popup_bounds(parent_bounds, display_bounds);
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            window_background,
            focus: false,
            show: true,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            display_id,
            ..Default::default()
        },
        move |_, cx| cx.new(|_| ToastPopupView::new(id, parent, request)),
    )?;

    #[cfg(target_os = "macos")]
    handle.update(cx, move |_, window, cx| {
        window.defer(cx, move |window, cx| {
            if !is_current(id)
                || crate::runtime_policy::check(
                    crate::runtime_policy::ExternalEffect::NativeVisibility,
                )
                .is_err()
            {
                window.remove_window();
                return;
            }
            let child = crate::components::inline_popup_window::inline_popup_ns_window(window);
            let parent = cx
                .update_window(parent, |_, parent, _| {
                    crate::components::inline_popup_window::inline_popup_ns_window(parent)
                })
                .ok()
                .flatten();
            let (Some(child), Some(parent)) = (child, parent) else {
                window.remove_window();
                return;
            };
            if child == parent {
                window.remove_window();
                return;
            }
            crate::platform::configure_child_attached_overlay_window_glass(
                window,
                TOAST_POPUP_LOG_TARGET,
                TOAST_POPUP_WINDOW_NAME,
            );
            // SAFETY: pointers come from exact live GPUI handles on the foreground thread.
            unsafe {
                use objc::{msg_send, sel, sel_impl};
                crate::platform::configure_toast_popup_window(child, is_dark_vibrancy);
                let _: () = msg_send![parent, addChildWindow: child ordered: 1_i64];
                let _: () = msg_send![child, orderFrontRegardless];
            }
        });
    })?;

    Ok(handle)
}

pub(crate) struct ToastPopupView {
    id: u64,
    parent: AnyWindowHandle,
    message: SharedString,
    kind: ToastPopupKind,
    action: Option<ToastPopupAction>,
}

impl ToastPopupView {
    fn new(id: u64, parent: AnyWindowHandle, request: ToastPopupRequest) -> Self {
        Self {
            id,
            parent,
            message: request.message,
            kind: request.kind,
            action: request.action,
        }
    }
}

impl Render for ToastPopupView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let dot_color = match self.kind {
            ToastPopupKind::Info => chrome.accent_hex,
            ToastPopupKind::Success => theme.colors.ui.success,
            ToastPopupKind::Error => theme.colors.ui.error,
        };
        let id = self.id;

        let mut row = div()
            .id("toast-popup-row")
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .px(px(14.0))
            .cursor_pointer()
            .on_click(move |_, _, cx| close_if_current(id, cx))
            .child(div().size(px(6.0)).rounded_full().bg(rgb(dot_color)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(rgb(chrome.text_primary_hex))
                    .child(self.message.clone()),
            );

        if let Some(action) = self.action.clone() {
            let parent = self.parent;
            let on_click = action.on_click.clone();
            row = row.child(
                Button::new(
                    "toast-popup-action",
                    action.label,
                    ButtonColors::from_theme(&theme),
                )
                .variant(ButtonVariant::Ghost)
                .shortcut_opt(action.shortcut.map(|shortcut| shortcut.to_string()))
                .on_click(Box::new(move |_, _, cx| {
                    cx.stop_propagation();
                    let on_click = on_click.clone();
                    cx.defer(move |cx| {
                        let _ = parent.update(cx, |_, window, cx| on_click(window, cx));
                    });
                })),
            );
        }

        div()
            .size_full()
            .bg(gpui::transparent_black())
            .overflow_hidden()
            .child(
                div()
                    .id("toast-popup-shell")
                    .size_full()
                    .border_1()
                    .border_color(rgba(chrome.border_rgba))
                    .rounded(px(TOAST_POPUP_RADIUS_PX))
                    .bg(rgba(chrome.popup_surface_rgba))
                    .overflow_hidden()
                    .child(row),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::toast_popup_bounds;
    use gpui::{point, px, size, Bounds};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Bounds<gpui::Pixels> {
        Bounds {
            origin: point(px(x), px(y)),
            size: size(px(w), px(h)),
        }
    }

    #[test]
    fn toast_sits_at_the_bottom_center_of_a_wide_parent() {
        let toast = toast_popup_bounds(rect(100., 100., 800., 600.), None);
        assert_eq!(toast.size.width, px(360.));
        assert!((toast.center().x - px(500.)).abs() < px(0.5));
        assert_eq!(toast.bottom(), px(700. - 12.));
    }

    #[test]
    fn toast_is_never_wider_than_a_narrow_parent() {
        let toast = toast_popup_bounds(rect(0., 0., 300., 500.), None);
        assert_eq!(toast.size.width, px(300. - 24.));
        assert_eq!(toast.left(), px(12.));
    }

    #[test]
    fn toast_stays_on_screen_for_a_parent_hanging_off_the_bottom() {
        let display = rect(0., 25., 1728., 1050.);
        let toast = toast_popup_bounds(rect(100., 900., 600., 500.), Some(display));
        assert_eq!(toast.bottom(), px(25. + 1050. - 12.));
    }
}
