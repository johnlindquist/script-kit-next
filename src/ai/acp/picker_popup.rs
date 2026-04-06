use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::{
    div, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::ai::context_picker_row::{
    render_compact_synopsis_strip, render_dense_monoline_picker_row, CONTEXT_PICKER_SYNOPSIS_HEIGHT,
};
use crate::ai::window::context_picker::empty_state_hints;
use crate::ai::window::context_picker::types::{ContextPickerItem, ContextPickerTrigger};

use super::view::AcpChatView;

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

/// Read the mention popup snapshot if the popup window is open.
///
/// Used by the automation surface collector to extract semantic elements
/// from the live popup state without needing `&mut App`.
pub(crate) fn get_mention_popup_snapshot(cx: &gpui::App) -> Option<AcpMentionPopupSnapshot> {
    let storage = ACP_MENTION_POPUP_WINDOW.get()?;
    let guard = storage.lock().ok()?;
    let slot = (*guard)?;
    slot.handle
        .read_with(cx, |popup, _cx| popup.snapshot.clone())
        .ok()
}

/// Select a mention popup item by its ID (value) for batch automation.
///
/// Returns `Some(item_id)` if the item was found and activated, `None` otherwise.
pub(crate) fn batch_select_mention_item_by_value(value: &str, cx: &mut App) -> Option<String> {
    let storage = ACP_MENTION_POPUP_WINDOW.get()?;
    let guard = storage.lock().ok()?;
    let slot = (*guard)?;
    let snap = slot
        .handle
        .read_with(cx, |popup, _cx| popup.snapshot.clone())
        .ok()?;
    let idx = snap
        .items
        .iter()
        .position(|item| item.id.as_ref() == value)?;
    let _ = slot.handle.update(cx, |popup, _window, cx| {
        popup.activate_item(idx, cx);
    });
    Some(value.to_string())
}

/// Select a mention popup item by its semantic ID (`choice:<idx>:<id>`).
///
/// Returns `Some(semantic_id)` if found and activated, `None` otherwise.
pub(crate) fn batch_select_mention_item_by_semantic_id(
    semantic_id: &str,
    cx: &mut App,
) -> Option<String> {
    let parts: Vec<&str> = semantic_id.splitn(3, ':').collect();
    if parts.len() < 3 || parts[0] != "choice" {
        return None;
    }
    let item_id = parts[2];
    batch_select_mention_item_by_value(item_id, cx)?;
    Some(semantic_id.to_string())
}

fn popup_height(snapshot: &AcpMentionPopupSnapshot) -> f32 {
    if snapshot.items.is_empty() {
        return super::popup_window::dense_picker_height(0);
    }

    super::popup_window::dense_picker_height(snapshot.items.len()) + CONTEXT_PICKER_SYNOPSIS_HEIGHT
}

pub(crate) fn popup_bounds(
    parent_bounds: Bounds<Pixels>,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> Bounds<Pixels> {
    super::popup_window::popup_bounds(parent_bounds, left, top, width, height)
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
                    super::popup_window::set_popup_window_bounds(window, bounds, cx);
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
        cx.new(|cx| AcpMentionPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
    })?;

    if let Err(error) =
        super::popup_window::configure_popup_window(&handle, cx, parent_window_handle)
    {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to configure ACP mention popup window"));
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
        if item_count <= super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS {
            return 0..item_count;
        }

        let half = super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS / 2;
        let mut start = self.snapshot.selected_index.saturating_sub(half);
        let max_start =
            item_count.saturating_sub(super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS);
        if start > max_start {
            start = max_start;
        }
        start..(start + super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS).min(item_count)
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
        let selected_item = self
            .snapshot
            .items
            .get(self.snapshot.selected_index)
            .cloned();

        let mut popup = super::popup_window::dense_picker_popup_surface(SharedString::from(
            "acp-mention-popup",
        ))
        .size_full()
        .py(px(super::popup_window::DENSE_PICKER_VERTICAL_PADDING / 2.0))
        .flex()
        .flex_col()
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
        );

        if let Some(item) = selected_item.filter(|item| !item.description.is_empty()) {
            popup = popup.child(div().h(px(1.0)).bg(fg.opacity(0.06))).child(
                render_compact_synopsis_strip(
                    item.label.clone(),
                    item.meta.clone(),
                    item.description.clone(),
                    fg,
                    muted_fg,
                ),
            );
        }

        popup.into_any_element()
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

        super::popup_window::dense_picker_popup_surface(SharedString::from(
            "acp-mention-popup-empty-state",
        ))
        .size_full()
        .py(px(super::popup_window::DENSE_PICKER_VERTICAL_PADDING))
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
            .size_full()
            .track_focus(&self.focus_handle)
            .child(if self.snapshot.items.is_empty() {
                self.render_empty_state(cx)
            } else {
                self.render_picker(cx)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{popup_bounds, popup_height, AcpMentionPopupSnapshot};
    use crate::ai::window::context_picker::types::ContextPickerTrigger;
    use gpui::SharedString;

    #[test]
    fn popup_height_clamps_to_visible_rows() {
        let snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            items: (0..16)
                .map(|ix| crate::ai::window::context_picker::types::ContextPickerItem {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(format!("Description {ix}")),
                    meta: SharedString::from(""),
                    kind: crate::ai::window::context_picker::types::ContextPickerItemKind::SlashCommand(
                        format!("cmd-{ix}"),
                    ),
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                })
                .collect(),
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

        let flipped_y = crate::ai::acp::popup_window::flipped_ns_window_y(bounds, 982.0);
        assert_eq!(flipped_y, 798.0);
    }
}
