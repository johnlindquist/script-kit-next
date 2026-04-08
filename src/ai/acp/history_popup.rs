use std::sync::{Mutex, OnceLock};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle, Focusable,
    FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};
use gpui_component::scroll::ScrollableElement;

use super::history::AcpHistoryEntry;
use super::view::AcpChatView;

const HISTORY_POPUP_MIN_WIDTH: f32 = crate::actions::constants::POPUP_WIDTH;
const HISTORY_POPUP_MAX_WIDTH: f32 = 420.0;
const HISTORY_POPUP_SIDE_MARGIN: f32 = 8.0;
const HISTORY_POPUP_TOP_INSET: f32 = 56.0;
const HISTORY_POPUP_BOTTOM_INSET: f32 = 12.0;
const HISTORY_POPUP_HEADER_HEIGHT: f32 = crate::actions::constants::HEADER_HEIGHT;
const HISTORY_POPUP_FOOTER_HEIGHT: f32 = crate::window_resize::mini_layout::HINT_STRIP_HEIGHT;
const HISTORY_POPUP_ROW_HEIGHT: f32 = crate::actions::constants::ACTION_ITEM_HEIGHT;
const HISTORY_POPUP_EMPTY_HEIGHT: f32 = 72.0;
const HISTORY_POPUP_VISIBLE_ROWS: usize = 6;
const HISTORY_POPUP_VERTICAL_PADDING: f32 = 4.0;

#[derive(Clone)]
pub(crate) struct AcpHistoryPopupSnapshot {
    pub(crate) title: SharedString,
    pub(crate) selected_index: usize,
    pub(crate) entries: Vec<AcpHistoryEntry>,
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

    let window_options = super::popup_window::popup_window_options(bounds, display_id);

    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| AcpHistoryPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
    })?;

    if let Err(error) =
        super::popup_window::configure_actions_style_popup_window(&handle, cx, parent_window_handle)
    {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to configure ACP history popup window"));
    }

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_history_popup_window_synced",
        entry_count = snapshot.entries.len(),
        selected_index = snapshot.selected_index,
        "Synced ACP history popup window"
    );

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
    fn attach_summary(&self, entry: &AcpHistoryEntry, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            let session_id = entry.session_id.clone();
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
                        error = %error,
                    );
                }
                cx.notify();
            });
        }
        close_history_popup_window(cx);
    }

    /// Alternate action (Cmd+Enter): load full transcript into ACP thread.
    fn load_transcript(&self, entry: &AcpHistoryEntry, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            let entry = entry.clone();
            view.update(cx, |view, cx| {
                view.select_history_from_popup(&entry, cx);
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
                                this.load_transcript(&entry, cx);
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
                                .child("No matching conversations")
                                .into_any_element()]
                        } else {
                            self.snapshot
                                .entries
                                .iter()
                                .enumerate()
                                .map(|(idx, entry)| {
                                    let is_selected = idx == self.snapshot.selected_index;
                                    let date = entry
                                        .timestamp
                                        .split('T')
                                        .next()
                                        .unwrap_or(&entry.timestamp)
                                        .to_string();
                                    let entry_clone = entry.clone();

                                    div()
                                        .id(SharedString::from(format!(
                                            "acp-history-popup-row-{idx}"
                                        )))
                                        .h(px(HISTORY_POPUP_ROW_HEIGHT))
                                        .w_full()
                                        .px(px(crate::actions::constants::ACTION_ROW_INSET))
                                        .py(px(2.0))
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
                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                            this.attach_summary(&entry_clone, cx);
                                        }))
                                        .child(
                                            div()
                                                .w_full()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .min_w(px(0.0))
                                                .overflow_hidden()
                                                .px(px(crate::actions::constants::ACTION_PADDING_X
                                                    - crate::actions::constants::ACTION_ROW_INSET))
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
                                                        .child(entry.title_display().to_string()),
                                                )
                                                .child(
                                                    div()
                                                        .flex_shrink_0()
                                                        .ml(px(8.0))
                                                        .text_xs()
                                                        .text_color(if is_selected {
                                                            secondary_text
                                                        } else {
                                                            dimmed_text
                                                        })
                                                        .whitespace_nowrap()
                                                        .child(format!(
                                                            "{} msgs · {}",
                                                            entry.message_count, date
                                                        )),
                                                ),
                                        )
                                        .into_any_element()
                                })
                                .collect::<Vec<_>>()
                        },
                    ))
                    .child(div().w_full().child(crate::components::HintStrip::new(vec![
                        "↑↓ Navigate".into(),
                        "↵ Attach Summary".into(),
                        "⌘↵ Load Transcript".into(),
                        "Esc Close".into(),
                    ]))),
            )
    }
}

fn set_popup_window_bounds(window: &mut Window, bounds: Bounds<Pixels>, cx: &mut App) {
    super::popup_window::set_popup_window_bounds(window, bounds, cx);
}

#[cfg(test)]
mod tests {
    use super::{
        popup_bounds, popup_height, AcpHistoryEntry, AcpHistoryPopupSnapshot,
        HISTORY_POPUP_MAX_WIDTH,
    };
    use gpui::SharedString;

    #[test]
    fn popup_height_accounts_for_rows_and_chrome() {
        let snapshot = AcpHistoryPopupSnapshot {
            title: SharedString::from("Recent Conversations"),
            selected_index: 0,
            entries: vec![
                AcpHistoryEntry {
                    timestamp: "2026-04-05T12:00:00Z".to_string(),
                    first_message: "First".to_string(),
                    message_count: 3,
                    session_id: "one".to_string(),
                    ..Default::default()
                },
                AcpHistoryEntry {
                    timestamp: "2026-04-04T12:00:00Z".to_string(),
                    first_message: "Second".to_string(),
                    message_count: 5,
                    session_id: "two".to_string(),
                    ..Default::default()
                },
            ],
        };

        assert!(popup_height(&snapshot) > 100.0);
    }

    #[test]
    fn popup_bounds_center_within_parent() {
        let snapshot = AcpHistoryPopupSnapshot {
            title: SharedString::from("Recent Conversations"),
            selected_index: 0,
            entries: vec![AcpHistoryEntry {
                timestamp: "2026-04-05T12:00:00Z".to_string(),
                first_message: "First".to_string(),
                message_count: 3,
                session_id: "one".to_string(),
                ..Default::default()
            }],
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
}
