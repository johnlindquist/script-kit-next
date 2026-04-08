use std::sync::{Mutex, OnceLock};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle, Focusable,
    FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};
use gpui_component::scroll::ScrollableElement;

use super::history::{AcpHistoryEntry, AcpHistorySearchField, AcpHistorySearchHit};
use super::view::AcpChatView;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

const HISTORY_POPUP_MIN_WIDTH: f32 = crate::actions::constants::POPUP_WIDTH;
const HISTORY_POPUP_MAX_WIDTH: f32 = 420.0;
const HISTORY_POPUP_SIDE_MARGIN: f32 = 8.0;
const HISTORY_POPUP_TOP_INSET: f32 = 56.0;
const HISTORY_POPUP_BOTTOM_INSET: f32 = 12.0;
const HISTORY_POPUP_HEADER_HEIGHT: f32 = crate::actions::constants::HEADER_HEIGHT;
const HISTORY_POPUP_FOOTER_HEIGHT: f32 = crate::window_resize::mini_layout::HINT_STRIP_HEIGHT;
const HISTORY_POPUP_ROW_HEIGHT: f32 = 60.0;
const HISTORY_POPUP_EMPTY_HEIGHT: f32 = 72.0;
const HISTORY_POPUP_VISIBLE_ROWS: usize = 5;
const HISTORY_POPUP_VERTICAL_PADDING: f32 = 4.0;

#[cfg(target_os = "macos")]
const NS_WINDOW_ABOVE: i64 = 1;

/// A single popup row derived from a ranked search hit.
#[derive(Clone)]
pub(crate) struct AcpHistoryPopupEntry {
    pub(crate) hit: AcpHistorySearchHit,
    pub(crate) title: SharedString,
    pub(crate) preview: SharedString,
    pub(crate) meta: SharedString,
    pub(crate) match_label: SharedString,
}

impl AcpHistoryPopupEntry {
    pub(crate) fn from_hit(hit: AcpHistorySearchHit) -> Self {
        let entry = &hit.entry;
        let date = entry
            .timestamp
            .split('T')
            .next()
            .unwrap_or(&entry.timestamp)
            .to_string();
        let match_label = match hit.matched_field {
            AcpHistorySearchField::Title => "title",
            AcpHistorySearchField::Preview => "reply",
            AcpHistorySearchField::SearchText => "transcript",
            AcpHistorySearchField::Timestamp => "date",
        };
        Self {
            title: SharedString::from(entry.title_display().to_string()),
            preview: SharedString::from(entry.preview_display().to_string()),
            meta: SharedString::from(format!("{} msgs \u{00B7} {}", entry.message_count, date)),
            match_label: SharedString::from(match_label),
            hit,
        }
    }
}
#[derive(Clone)]
pub(crate) struct AcpHistoryPopupSnapshot {
    pub(crate) title: SharedString,
    pub(crate) selected_index: usize,
    pub(crate) entries: Vec<AcpHistoryPopupEntry>,
}

#[derive(Clone)]
pub(crate) struct AcpHistoryPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<AcpChatView>,
    pub(crate) snapshot: AcpHistoryPopupSnapshot,
}

#[derive(Clone, Copy)]
struct AcpHistoryPopupSlot {
    handle: WindowHandle<AcpHistoryPopupWindow>,
    parent_window_handle: AnyWindowHandle,
}

static ACP_HISTORY_POPUP_WINDOW: OnceLock<Mutex<Option<AcpHistoryPopupSlot>>> = OnceLock::new();

pub(crate) fn close_history_popup_window(cx: &mut App) {
    if let Some(storage) = ACP_HISTORY_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

fn popup_height(snapshot: &AcpHistoryPopupSnapshot) -> f32 {
    let body_height = if snapshot.entries.is_empty() {
        HISTORY_POPUP_EMPTY_HEIGHT
    } else {
        let visible_rows = snapshot.entries.len().min(HISTORY_POPUP_VISIBLE_ROWS) as f32;
        visible_rows * HISTORY_POPUP_ROW_HEIGHT
    };

    HISTORY_POPUP_HEADER_HEIGHT
        + HISTORY_POPUP_FOOTER_HEIGHT
        + body_height
        + (HISTORY_POPUP_VERTICAL_PADDING * 2.0)
}

fn popup_bounds(
    parent_bounds: Bounds<Pixels>,
    snapshot: &AcpHistoryPopupSnapshot,
) -> Bounds<Pixels> {
    let parent_width = parent_bounds.size.width.as_f32();
    let parent_height = parent_bounds.size.height.as_f32();
    let width = (parent_width - (HISTORY_POPUP_SIDE_MARGIN * 2.0))
        .clamp(HISTORY_POPUP_MIN_WIDTH, HISTORY_POPUP_MAX_WIDTH);
    let height = popup_height(snapshot)
        .min((parent_height - HISTORY_POPUP_TOP_INSET - HISTORY_POPUP_BOTTOM_INSET).max(140.0));
    let left = ((parent_width - width) / 2.0).max(HISTORY_POPUP_SIDE_MARGIN);
    let max_top =
        (parent_height - height - HISTORY_POPUP_BOTTOM_INSET).max(HISTORY_POPUP_TOP_INSET);
    let centered_top = (parent_height - height) / 2.0;
    let top = centered_top.clamp(HISTORY_POPUP_TOP_INSET, max_top);

    Bounds {
        origin: gpui::point(
            parent_bounds.origin.x + px(left),
            parent_bounds.origin.y + px(top),
        ),
        size: gpui::size(px(width), px(height)),
    }
}

pub(crate) fn sync_history_popup_window(
    cx: &mut App,
    request: AcpHistoryPopupRequest,
) -> anyhow::Result<()> {
    let AcpHistoryPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_id,
        source_view,
        snapshot,
    } = request;

    let bounds = popup_bounds(parent_bounds, &snapshot);
    let storage = ACP_HISTORY_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
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
        cx.new(|cx| AcpHistoryPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
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
                "failed to configure ACP history popup window"
            ));
        }
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(AcpHistoryPopupSlot {
            handle,
            parent_window_handle,
        });
    }

    Ok(())
}

pub(crate) struct AcpHistoryPopupWindow {
    snapshot: AcpHistoryPopupSnapshot,
    source_view: WeakEntity<AcpChatView>,
    focus_handle: FocusHandle,
}

impl AcpHistoryPopupWindow {
    fn new(
        snapshot: AcpHistoryPopupSnapshot,
        source_view: WeakEntity<AcpChatView>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_snapshot(&mut self, snapshot: AcpHistoryPopupSnapshot) {
        self.snapshot = snapshot;
    }

    /// Default action (Enter / click): attach a summary as a context chip.
    fn attach_summary(&self, entry: &AcpHistoryPopupEntry, cx: &mut App) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_popup_action",
            action = "attach_summary",
            session_id = %entry.hit.entry.session_id,
            score = entry.hit.score,
            matched_field = ?entry.hit.matched_field,
        );
        if let Some(view) = self.source_view.upgrade() {
            let session_id = entry.hit.entry.session_id.clone();
            view.update(cx, |view, cx| {
                view.history_menu = None;
                view.sync_history_popup_window_from_cached_parent(cx);
                if let Err(error) = view.attach_history_session(
                    &session_id,
                    super::history_attachment::AcpHistoryAttachMode::Summary,
                    cx,
                ) {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "acp_history_popup_attach_failed",
                        session_id = %session_id,
                        mode = "summary",
                        error = %error,
                    );
                }
                cx.notify();
            });
        }
        close_history_popup_window(cx);
    }

    /// Shift+Enter: attach full transcript as a context chip.
    fn attach_transcript(&self, entry: &AcpHistoryPopupEntry, cx: &mut App) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_popup_action",
            action = "attach_transcript",
            session_id = %entry.hit.entry.session_id,
            score = entry.hit.score,
            matched_field = ?entry.hit.matched_field,
        );
        if let Some(view) = self.source_view.upgrade() {
            let session_id = entry.hit.entry.session_id.clone();
            view.update(cx, |view, cx| {
                view.history_menu = None;
                view.sync_history_popup_window_from_cached_parent(cx);
                if let Err(error) = view.attach_history_session(
                    &session_id,
                    super::history_attachment::AcpHistoryAttachMode::Transcript,
                    cx,
                ) {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "acp_history_popup_attach_failed",
                        session_id = %session_id,
                        mode = "transcript",
                        error = %error,
                    );
                }
                cx.notify();
            });
        }
        close_history_popup_window(cx);
    }

    /// Cmd+Enter: resume (load) the session into the ACP thread.
    fn resume_session(&self, entry: &AcpHistoryPopupEntry, cx: &mut App) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_popup_action",
            action = "resume",
            session_id = %entry.hit.entry.session_id,
            score = entry.hit.score,
            matched_field = ?entry.hit.matched_field,
        );
        if let Some(view) = self.source_view.upgrade() {
            let history_entry = entry.hit.entry.clone();
            view.update(cx, |view, cx| {
                view.select_history_from_popup(&history_entry, cx);
            });
        }
        close_history_popup_window(cx);
    }

    fn navigate(&mut self, delta: i32, cx: &mut Context<Self>) {
        if self.snapshot.entries.is_empty() {
            return;
        }
        let len = self.snapshot.entries.len();
        let idx = self.snapshot.selected_index;
        self.snapshot.selected_index = if delta < 0 {
            idx.saturating_sub((-delta) as usize)
        } else {
            (idx + delta as usize).min(len.saturating_sub(1))
        };
        cx.notify();
    }
}

impl Focusable for AcpHistoryPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpHistoryPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let title_color = gpui::rgb(theme.colors.text.primary);
        let dimmed_text = gpui::rgb(theme.colors.text.dimmed);
        let secondary_text = gpui::rgb(theme.colors.text.secondary);
        let hover_bg = gpui::rgba(chrome.hover_rgba);
        let selected_bg = gpui::rgba(chrome.selection_rgba);
        let selected_bar = gpui::rgba((theme.colors.accent.selected << 8) | 0xFF);
        let container_bg = gpui::rgba(chrome.popup_surface_rgba);
        let container_border = gpui::rgba(chrome.border_rgba);

        div()
            .track_focus(&self.focus_handle)
            .id("acp-history-popup")
            .on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                    let key = event.keystroke.key.as_str();
                    let has_cmd = event.keystroke.modifiers.platform;
                    let has_shift = event.keystroke.modifiers.shift;

                    if crate::ui_foundation::is_key_up(key) {
                        this.navigate(-1, cx);
                        cx.stop_propagation();
                    } else if crate::ui_foundation::is_key_down(key) {
                        this.navigate(1, cx);
                        cx.stop_propagation();
                    } else if crate::ui_foundation::is_key_enter(key) {
                        if let Some(entry) = this
                            .snapshot
                            .entries
                            .get(this.snapshot.selected_index)
                            .cloned()
                        {
                            if has_cmd {
                                this.resume_session(&entry, cx);
                            } else if has_shift {
                                this.attach_transcript(&entry, cx);
                            } else {
                                this.attach_summary(&entry, cx);
                            }
                        }
                        cx.stop_propagation();
                    } else if crate::ui_foundation::is_key_escape(key) {
                        close_history_popup_window(cx);
                        cx.stop_propagation();
                    } else {
                        cx.propagate();
                    }
                }),
            )
            .w_full()
            .h_full()
            .px(px(HISTORY_POPUP_SIDE_MARGIN))
            .py(px(HISTORY_POPUP_VERTICAL_PADDING))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .bg(container_bg)
                    .border_1()
                    .border_color(container_border)
                    .overflow_hidden()
                    .child(
                        div()
                            .w_full()
                            .h(px(HISTORY_POPUP_HEADER_HEIGHT))
                            .px(px(crate::actions::constants::ACTION_PADDING_X))
                            .pt(px(crate::actions::constants::ACTION_PADDING_TOP))
                            .pb(px(4.0))
                            .flex()
                            .flex_col()
                            .justify_center()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(dimmed_text)
                                    .child(self.snapshot.title.clone()),
                            ),
                    )
                    .child(div().w_full().flex_1().overflow_y_scrollbar().children(
                        if self.snapshot.entries.is_empty() {
                            vec![div()
                                .w_full()
                                .h(px(HISTORY_POPUP_EMPTY_HEIGHT))
                                .px(px(crate::actions::constants::ACTION_PADDING_X))
                                .flex()
                                .items_center()
                                .text_sm()
                                .text_color(dimmed_text)
                                .child(
                                    "No matches yet \u{2014} try words from the prompt, reply, or date",
                                )
                                .into_any_element()]
                        } else {
                            self.snapshot
                                .entries
                                .iter()
                                .enumerate()
                                .map(|(idx, entry)| {
                                    let is_selected = idx == self.snapshot.selected_index;
                                    let row_entry = entry.clone();

                                    div()
                                        .id(SharedString::from(format!(
                                            "acp-history-popup-row-{idx}"
                                        )))
                                        .h(px(HISTORY_POPUP_ROW_HEIGHT))
                                        .w_full()
                                        .px(px(crate::actions::constants::ACTION_ROW_INSET))
                                        .py(px(4.0))
                                        .flex()
                                        .flex_col()
                                        .justify_center()
                                        .border_l(px(crate::actions::constants::ACCENT_BAR_WIDTH))
                                        .border_color(if is_selected {
                                            selected_bar
                                        } else {
                                            gpui::rgba(0x0000_0000)
                                        })
                                        .cursor_pointer()
                                        .when(is_selected, |d| d.bg(selected_bg))
                                        .when(!is_selected, |d| d.hover(|d| d.bg(hover_bg)))
                                        .on_click(
                                            cx.listener(move |this, _event, _window, cx| {
                                                this.attach_summary(&row_entry, cx);
                                            }),
                                        )
                                        .child(
                                            div()
                                                .w_full()
                                                .flex()
                                                .flex_col()
                                                .gap(px(2.0))
                                                .px(px(
                                                    crate::actions::constants::ACTION_PADDING_X
                                                        - crate::actions::constants::ACTION_ROW_INSET,
                                                ))
                                                // ── Row 1: title + match badge + meta ──
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap(px(8.0))
                                                        .child(
                                                            div()
                                                                .flex_1()
                                                                .min_w(px(0.0))
                                                                .text_sm()
                                                                .font_weight(if is_selected {
                                                                    FontWeight::MEDIUM
                                                                } else {
                                                                    FontWeight::NORMAL
                                                                })
                                                                .text_color(title_color)
                                                                .overflow_hidden()
                                                                .text_ellipsis()
                                                                .whitespace_nowrap()
                                                                .child(entry.title.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex_shrink_0()
                                                                .px(px(6.0))
                                                                .py(px(2.0))
                                                                .rounded(px(999.0))
                                                                .bg(if is_selected {
                                                                    gpui::rgba(
                                                                        (theme.colors.accent.selected
                                                                            << 8)
                                                                            | 0x18,
                                                                    )
                                                                } else {
                                                                    gpui::rgba(
                                                                        (theme.colors.ui.border << 8)
                                                                            | 0x10,
                                                                    )
                                                                })
                                                                .border_1()
                                                                .border_color(if is_selected {
                                                                    gpui::rgba(
                                                                        (theme.colors.accent.selected
                                                                            << 8)
                                                                            | 0x30,
                                                                    )
                                                                } else {
                                                                    gpui::rgba(
                                                                        (theme.colors.ui.border << 8)
                                                                            | 0x20,
                                                                    )
                                                                })
                                                                .text_xs()
                                                                .text_color(if is_selected {
                                                                    gpui::rgb(
                                                                        theme.colors.accent.selected,
                                                                    )
                                                                } else {
                                                                    dimmed_text
                                                                })
                                                                .child(entry.match_label.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex_shrink_0()
                                                                .text_xs()
                                                                .text_color(if is_selected {
                                                                    secondary_text
                                                                } else {
                                                                    dimmed_text
                                                                })
                                                                .whitespace_nowrap()
                                                                .child(entry.meta.clone()),
                                                        ),
                                                )
                                                // ── Row 2: preview ──
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .text_xs()
                                                        .text_color(if is_selected {
                                                            secondary_text
                                                        } else {
                                                            dimmed_text
                                                        })
                                                        .overflow_hidden()
                                                        .text_ellipsis()
                                                        .whitespace_nowrap()
                                                        .child(entry.preview.clone()),
                                                ),
                                        )
                                        .into_any_element()
                                })
                                .collect::<Vec<_>>()
                        },
                    ))
                    .child(div().w_full().child(crate::components::HintStrip::new(vec![
                        "\u{2191}\u{2193} Navigate".into(),
                        "\u{21B5} Attach Summary".into(),
                        "\u{21E7}\u{21B5} Attach Transcript".into(),
                        "\u{2318}\u{21B5} Resume".into(),
                        "Esc Close".into(),
                    ]))),
            )
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
    use super::{
        popup_bounds, popup_height, AcpHistoryPopupEntry, AcpHistoryPopupSnapshot,
        HISTORY_POPUP_MAX_WIDTH,
    };
    use crate::ai::acp::history::{AcpHistoryEntry, AcpHistorySearchField, AcpHistorySearchHit};
    use gpui::SharedString;

    fn make_entry(
        session_id: &str,
        first_message: &str,
        message_count: usize,
    ) -> AcpHistoryPopupEntry {
        AcpHistoryPopupEntry::from_hit(AcpHistorySearchHit {
            entry: AcpHistoryEntry {
                timestamp: "2026-04-05T12:00:00Z".to_string(),
                first_message: first_message.to_string(),
                message_count,
                session_id: session_id.to_string(),
                ..Default::default()
            },
            score: 0,
            matched_field: AcpHistorySearchField::Title,
        })
    }

    #[test]
    fn popup_height_accounts_for_rows_and_chrome() {
        let snapshot = AcpHistoryPopupSnapshot {
            title: SharedString::from("Recent Conversations"),
            selected_index: 0,
            entries: vec![
                make_entry("one", "First", 3),
                make_entry("two", "Second", 5),
            ],
        };

        assert!(popup_height(&snapshot) > 100.0);
    }

    #[test]
    fn popup_bounds_center_within_parent() {
        let snapshot = AcpHistoryPopupSnapshot {
            title: SharedString::from("Recent Conversations"),
            selected_index: 0,
            entries: vec![make_entry("one", "First", 3)],
        };
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(40.0)),
            size: gpui::size(gpui::px(480.0), gpui::px(440.0)),
        };

        let bounds = popup_bounds(parent, &snapshot);
        assert_eq!(f32::from(bounds.size.width), HISTORY_POPUP_MAX_WIDTH);
        assert!(f32::from(bounds.origin.x) > 100.0);
        assert!(f32::from(bounds.origin.y) >= 96.0);
    }

    #[test]
    fn from_hit_preserves_match_metadata() {
        let hit = AcpHistorySearchHit {
            entry: AcpHistoryEntry {
                timestamp: "2026-04-08T08:10:00Z".to_string(),
                first_message: "continue".to_string(),
                title: "Continue the deployment cleanup".to_string(),
                preview: "I found the stale kubernetes secret".to_string(),
                search_text: "kubernetes secret deployment".to_string(),
                message_count: 14,
                session_id: "test-session".to_string(),
            },
            score: 42,
            matched_field: AcpHistorySearchField::SearchText,
        };

        let entry = AcpHistoryPopupEntry::from_hit(hit);
        assert_eq!(entry.title.as_ref(), "Continue the deployment cleanup");
        assert_eq!(
            entry.preview.as_ref(),
            "I found the stale kubernetes secret"
        );
        assert_eq!(entry.match_label.as_ref(), "transcript");
        assert!(entry.meta.as_ref().contains("14 msgs"));
        assert_eq!(entry.hit.score, 42);
    }
}
