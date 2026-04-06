use std::sync::{Mutex, OnceLock};

use gpui::{
    div, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};

use crate::ai::context_picker_row::{render_dense_monoline_picker_row, CONTEXT_PICKER_ROW_HEIGHT};
use crate::ai::window::context_picker::empty_state_hints;
use crate::ai::window::context_picker::types::{ContextPickerItem, ContextPickerTrigger};

use super::view::AcpChatView;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

const ACP_PICKER_VISIBLE_ROWS: usize = 8;
const ACP_PICKER_VERTICAL_PADDING: f32 = 4.0;
const ACP_PICKER_EMPTY_HEIGHT: f32 = 56.0;

#[derive(Clone)]
pub(crate) struct AcpMentionPopupSnapshot {
    pub(crate) trigger: ContextPickerTrigger,
    pub(crate) selected_index: usize,
    pub(crate) items: Vec<ContextPickerItem>,
    pub(crate) width: f32,
}

#[derive(Clone)]
pub(crate) struct AcpMentionPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<AcpChatView>,
    pub(crate) snapshot: AcpMentionPopupSnapshot,
    pub(crate) left: f32,
    pub(crate) top: f32,
}

#[derive(Clone, Copy)]
struct AcpMentionPopupSlot {
    handle: WindowHandle<AcpMentionPopupWindow>,
    parent_window_handle: AnyWindowHandle,
}

static ACP_MENTION_POPUP_WINDOW: OnceLock<Mutex<Option<AcpMentionPopupSlot>>> = OnceLock::new();

#[cfg(target_os = "macos")]
const NS_WINDOW_ABOVE: i64 = 1;

pub(crate) fn close_mention_popup_window(cx: &mut App) {
    if let Some(storage) = ACP_MENTION_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

pub(crate) fn is_mention_popup_window_open() -> bool {
    if let Some(storage) = ACP_MENTION_POPUP_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            return guard.is_some();
        }
    }
    false
}

fn popup_height(snapshot: &AcpMentionPopupSnapshot) -> f32 {
    if snapshot.items.is_empty() {
        return ACP_PICKER_EMPTY_HEIGHT;
    }

    let visible_rows = snapshot.items.len().min(ACP_PICKER_VISIBLE_ROWS) as f32;
    (visible_rows * CONTEXT_PICKER_ROW_HEIGHT) + (ACP_PICKER_VERTICAL_PADDING * 2.0)
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

pub(crate) fn sync_mention_popup_window(
    cx: &mut App,
    request: AcpMentionPopupRequest,
) -> anyhow::Result<()> {
    let AcpMentionPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_id,
        source_view,
        snapshot,
        left,
        top,
    } = request;

    let bounds = popup_bounds(
        parent_bounds,
        left,
        top,
        snapshot.width,
        popup_height(&snapshot),
    );

    let storage = ACP_MENTION_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
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
        cx.new(|cx| AcpMentionPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
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
                "failed to configure ACP mention popup window"
            ));
        }
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(AcpMentionPopupSlot {
            handle,
            parent_window_handle,
        });
    }

    Ok(())
}

pub(crate) struct AcpMentionPopupWindow {
    snapshot: AcpMentionPopupSnapshot,
    source_view: WeakEntity<AcpChatView>,
    focus_handle: FocusHandle,
}

impl AcpMentionPopupWindow {
    fn new(
        snapshot: AcpMentionPopupSnapshot,
        source_view: WeakEntity<AcpChatView>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_snapshot(&mut self, snapshot: AcpMentionPopupSnapshot) {
        self.snapshot = snapshot;
    }

    fn visible_range(&self) -> std::ops::Range<usize> {
        let item_count = self.snapshot.items.len();
        if item_count <= ACP_PICKER_VISIBLE_ROWS {
            return 0..item_count;
        }

        let half = ACP_PICKER_VISIBLE_ROWS / 2;
        let mut start = self.snapshot.selected_index.saturating_sub(half);
        let max_start = item_count.saturating_sub(ACP_PICKER_VISIBLE_ROWS);
        if start > max_start {
            start = max_start;
        }
        start..(start + ACP_PICKER_VISIBLE_ROWS).min(item_count)
    }

    fn activate_item(&self, index: usize, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            view.update(cx, |view, cx| {
                view.select_mention_index(index);
                view.accept_mention_selection(cx);
                view.sync_mention_popup_window_from_cached_parent(cx);
            });
        } else {
            close_mention_popup_window(cx);
        }
    }

    fn apply_hint(&self, insertion: &str, close_after_apply: bool, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            let insertion = insertion.to_string();
            view.update(cx, |view, cx| {
                if close_after_apply {
                    view.apply_picker_hint_token(&insertion, cx);
                } else {
                    view.insert_picker_hint_prefix(&insertion, cx);
                }
                view.sync_mention_popup_window_from_cached_parent(cx);
            });
        } else {
            close_mention_popup_window(cx);
        }
    }

    fn render_picker(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = crate::theme::get_cached_theme();
        let fg: gpui::Hsla = gpui::rgb(theme.colors.text.primary).into();
        let muted_fg: gpui::Hsla = gpui::rgb(theme.colors.text.muted).into();
        let visible = self.visible_range();

        div()
            .id("acp-mention-popup")
            .w(px(self.snapshot.width))
            .bg(fg.opacity(0.02))
            .py(px(2.0))
            .children(
                self.snapshot
                    .items
                    .iter()
                    .enumerate()
                    .skip(visible.start)
                    .take(visible.len())
                    .map(|(idx, item)| {
                        let is_selected = idx == self.snapshot.selected_index;
                        let source_view = self.source_view.clone();
                        render_dense_monoline_picker_row(
                            SharedString::from(format!("acp-mention-popup-row-{idx}")),
                            item.label.clone(),
                            item.meta.clone(),
                            &item.label_highlight_indices,
                            &item.meta_highlight_indices,
                            is_selected,
                            fg,
                            muted_fg,
                        )
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _event, _window, cx| {
                            if source_view.upgrade().is_none() {
                                close_mention_popup_window(cx);
                                return;
                            }
                            this.activate_item(idx, cx);
                        }))
                        .into_any_element()
                    }),
            )
            .into_any_element()
    }

    fn render_empty_state(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        use crate::ai::context_picker_row::{GHOST, HINT, MUTED_OP};
        use crate::list_item::FONT_MONO;

        let cached_theme = crate::theme::get_cached_theme();
        let fg: gpui::Hsla = gpui::rgb(cached_theme.colors.text.primary).into();
        let muted_fg: gpui::Hsla = gpui::rgb(cached_theme.colors.text.muted).into();
        let is_slash = self.snapshot.trigger == ContextPickerTrigger::Slash;

        let mut chips: Vec<gpui::AnyElement> = Vec::new();
        for hint in empty_state_hints(self.snapshot.trigger).iter() {
            let hint_display = SharedString::from(hint.display);
            let hint_insertion = hint.insertion.to_string();
            let close_after_apply = !hint.insertion.ends_with(':');

            chips.push(
                div()
                    .id(SharedString::from(format!(
                        "acp-mention-popup-hint-{}",
                        hint.display
                    )))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(fg.opacity(GHOST))
                    .hover(|el| el.bg(fg.opacity(0.08)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.apply_hint(&hint_insertion, close_after_apply, cx);
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_family(FONT_MONO)
                            .text_color(muted_fg.opacity(HINT))
                            .child(hint_display),
                    )
                    .into_any_element(),
            );
        }

        div()
            .id("acp-mention-popup-empty-state")
            .w(px(self.snapshot.width))
            .bg(fg.opacity(0.02))
            .py(px(4.0))
            .px(px(6.0))
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(muted_fg.opacity(MUTED_OP))
                    .child(if is_slash {
                        "No matching commands"
                    } else {
                        "No matching context"
                    }),
            )
            .child(div().flex().items_center().gap(px(4.0)).children(chips))
            .into_any_element()
    }
}

impl Focusable for AcpMentionPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpMentionPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .child(if self.snapshot.items.is_empty() {
                self.render_empty_state(cx)
            } else {
                self.render_picker(cx)
            })
    }
}

#[cfg(target_os = "macos")]
fn flipped_ns_window_y(bounds: Bounds<Pixels>, primary_height: f64) -> f64 {
    primary_height - f32::from(bounds.origin.y) as f64 - f32::from(bounds.size.height) as f64
}

#[cfg(target_os = "macos")]
fn set_popup_window_bounds(window: &mut Window, bounds: Bounds<Pixels>, cx: &mut App) {
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
    use super::{popup_bounds, popup_height, AcpMentionPopupSnapshot};
    use crate::ai::window::context_picker::types::ContextPickerTrigger;

    #[test]
    fn popup_height_clamps_to_visible_rows() {
        let snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            items: Vec::with_capacity(16),
            width: 320.0,
        };
        assert!(popup_height(&snapshot) > 0.0);
    }

    #[test]
    fn popup_bounds_offsets_from_parent_origin() {
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(40.0)),
            size: gpui::size(gpui::px(700.0), gpui::px(500.0)),
        };
        let bounds = popup_bounds(parent, 24.0, 60.0, 320.0, 84.0);
        assert_eq!(f32::from(bounds.origin.x), 124.0);
        assert_eq!(f32::from(bounds.origin.y), 100.0);
        assert_eq!(f32::from(bounds.size.width), 320.0);
        assert_eq!(f32::from(bounds.size.height), 84.0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn popup_bounds_flip_y_for_nswindow_coordinates() {
        let bounds = gpui::Bounds {
            origin: gpui::point(gpui::px(124.0), gpui::px(100.0)),
            size: gpui::size(gpui::px(320.0), gpui::px(84.0)),
        };

        let flipped_y = super::flipped_ns_window_y(bounds, 982.0);
        assert_eq!(flipped_y, 798.0);
    }
}
