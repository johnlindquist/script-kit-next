use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::{
    div, prelude::FluentBuilder, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::ai::window::context_picker::empty_state_hints;
use crate::ai::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, ContextPickerTrigger,
};
use crate::components::inline_dropdown::{
    inline_dropdown_visible_range_from_start, InlineDropdown, InlineDropdownColors,
    InlineDropdownEmptyState, MUTED_OP,
};
use crate::components::inline_picker::{
    InlinePickerHighlights, InlinePickerRow, InlinePickerRowKind,
};
use crate::components::inline_popup_window::{
    inline_popup_height_for_row_height, INLINE_POPUP_EDGE_GUTTER, INLINE_POPUP_MAX_VISIBLE_ROWS,
    INLINE_POPUP_VERTICAL_PADDING,
};
use gpui_component::scroll::Scrollbar;

use super::view::AcpChatView;

const ACP_MENTION_POPUP_AUTOMATION_ID: &str = "acp-mention-popup";
const ACP_MENTION_POPUP_MAX_PARENT_HEIGHT_RATIO: f32 = 0.90;

pub(crate) fn acp_context_picker_item_to_inline_picker_row(
    item: &ContextPickerItem,
) -> InlinePickerRow {
    let (kind, title, token, token_highlights, enabled) = match &item.kind {
        ContextPickerItemKind::SlashCommand(payload) => (
            InlinePickerRowKind::SlashCommand,
            SharedString::from(format!("/{}", payload.slash_name())),
            Some(SharedString::from(payload.owner_label())),
            item.meta_highlight_indices.clone(),
            true,
        ),
        ContextPickerItemKind::Inert => (
            InlinePickerRowKind::Context,
            item.label.clone(),
            Some(item.meta.clone()),
            item.meta_highlight_indices.clone(),
            false,
        ),
        _ => (
            InlinePickerRowKind::Context,
            item.label.clone(),
            Some(item.meta.clone()),
            item.meta_highlight_indices.clone(),
            true,
        ),
    };

    let title_highlights = if matches!(&kind, InlinePickerRowKind::SlashCommand) {
        item.label_highlight_indices
            .iter()
            .map(|ix| ix.saturating_add(1))
            .map(|ix| ix..ix.saturating_add(1))
            .collect()
    } else {
        item.label_highlight_indices
            .iter()
            .map(|ix| *ix..(*ix).saturating_add(1))
            .collect()
    };

    InlinePickerRow {
        id: item.id.clone(),
        kind,
        title,
        token,
        subtitle: None,
        detail: Some(item.description.clone()),
        example: None,
        leading: None,
        badges: Vec::new(),
        accessory: None,
        highlights: InlinePickerHighlights {
            title: title_highlights,
            token: token_highlights
                .into_iter()
                .map(|ix| ix..ix.saturating_add(1))
                .collect(),
            subtitle: Vec::new(),
            detail: Vec::new(),
        },
        enabled,
        disabled_reason: (!enabled).then(|| SharedString::from("Inert context picker row")),
    }
}

#[derive(Clone)]
pub(crate) struct AcpMentionPopupSnapshot {
    pub(crate) trigger: ContextPickerTrigger,
    pub(crate) selected_index: usize,
    pub(crate) visible_start: usize,
    pub(crate) items: Vec<ContextPickerItem>,
    pub(crate) width: f32,
}

#[derive(Clone)]
pub(crate) struct AcpMentionPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) display_bounds: Option<Bounds<Pixels>>,
    pub(crate) source_view: WeakEntity<AcpChatView>,
    pub(crate) snapshot: AcpMentionPopupSnapshot,
    pub(crate) left: f32,
    pub(crate) top: f32,
}

struct AcpMentionPopupSlot {
    handle: WindowHandle<AcpMentionPopupWindow>,
    parent_window_handle: AnyWindowHandle,
    _registration: super::popup_registry::AcpPopupRegistration,
}

static ACP_MENTION_POPUP_WINDOW: OnceLock<Mutex<Option<AcpMentionPopupSlot>>> = OnceLock::new();

fn clear_mention_popup_window_slot() {
    if let Some(storage) = ACP_MENTION_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

fn unregister_mention_popup_automation_window() {
    super::popup_window::unregister_acp_prompt_popup_automation_window(
        ACP_MENTION_POPUP_AUTOMATION_ID,
    );
}

pub(crate) fn close_mention_popup_window(cx: &mut App) {
    unregister_mention_popup_automation_window();
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
    let slot = guard.as_ref()?;
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
    let slot = guard.as_ref()?;
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

const TRIGGER_POPUP_ROW_HEIGHT: f32 = 30.0;

fn popup_height(snapshot: &AcpMentionPopupSnapshot) -> f32 {
    let row_count = snapshot.items.len().min(INLINE_POPUP_MAX_VISIBLE_ROWS);
    inline_popup_height_for_row_height(row_count, TRIGGER_POPUP_ROW_HEIGHT)
}

fn popup_visible_row_limit(item_count: usize, parent_height: f32) -> usize {
    if item_count == 0 {
        return 0;
    }

    let max_height = (parent_height * ACP_MENTION_POPUP_MAX_PARENT_HEIGHT_RATIO).max(1.0);
    let hard_limit = item_count.min(INLINE_POPUP_MAX_VISIBLE_ROWS);

    (1..=hard_limit)
        .rev()
        .find(|rows| {
            let height = inline_popup_height_for_row_height(*rows, TRIGGER_POPUP_ROW_HEIGHT);
            height <= max_height
        })
        .unwrap_or(1)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AcpMentionPopupLayout {
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) visible_row_limit: usize,
}

fn register_mention_popup_automation_window(
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    popup_bounds: Bounds<Pixels>,
) -> anyhow::Result<()> {
    super::popup_window::register_acp_prompt_popup_automation_window(
        ACP_MENTION_POPUP_AUTOMATION_ID,
        "ACP Mention Picker",
        parent_window_handle,
        parent_bounds,
        popup_bounds,
    )
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

pub(crate) fn popup_layout_above(
    parent_bounds: Bounds<Pixels>,
    display_bounds: Option<Bounds<Pixels>>,
    snapshot: &AcpMentionPopupSnapshot,
) -> AcpMentionPopupLayout {
    let width = snapshot.width;

    let display_top = display_bounds
        .map(|db| db.origin.y.as_f32() + INLINE_POPUP_EDGE_GUTTER)
        .unwrap_or(0.0);
    let available_height = (parent_bounds.origin.y.as_f32() - display_top).max(1.0);

    let visible_row_limit = popup_visible_row_limit(snapshot.items.len(), available_height);
    let row_count = snapshot.items.len().min(visible_row_limit);
    let height = inline_popup_height_for_row_height(row_count, TRIGGER_POPUP_ROW_HEIGHT);

    let preferred_left = parent_bounds.origin.x.as_f32();
    let left = display_bounds
        .map(|display_bounds| {
            let display_left = display_bounds.origin.x.as_f32();
            let display_right = display_left + display_bounds.size.width.as_f32();
            preferred_left.clamp(display_left, (display_right - width).max(display_left))
        })
        .unwrap_or(preferred_left);

    let top = parent_bounds.origin.y.as_f32() - height;

    AcpMentionPopupLayout {
        bounds: Bounds {
            origin: gpui::point(gpui::px(left), gpui::px(top)),
            size: gpui::size(gpui::px(width), gpui::px(height)),
        },
        visible_row_limit,
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
        display_bounds,
        source_view,
        mut snapshot,
        left: _left,
        top: _top,
    } = request;

    let parent_width = parent_bounds.size.width.as_f32();
    snapshot.width = parent_width;

    let layout = popup_layout_above(parent_bounds, display_bounds, &snapshot);
    let bounds = layout.bounds;

    let storage = ACP_MENTION_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        if let Some(slot) = guard.as_ref() {
            if slot.parent_window_handle == parent_window_handle {
                let update_result = slot.handle.update(cx, |popup, window, cx| {
                    popup.set_snapshot(snapshot.clone());
                    popup.set_visible_row_limit(layout.visible_row_limit);
                    super::popup_window::set_popup_window_bounds(window, bounds, cx);
                    cx.notify();
                });

                if update_result.is_ok() {
                    if let Err(error) = register_mention_popup_automation_window(
                        parent_window_handle,
                        parent_bounds,
                        bounds,
                    ) {
                        tracing::warn!(
                            target: "script_kit::automation",
                            event = "acp_mention_popup_registry_failed",
                            error = %error,
                            "Failed to refresh ACP mention popup automation registry entry"
                        );
                    }
                    return Ok(());
                }

                unregister_mention_popup_automation_window();
                *guard = None;
            } else {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
                unregister_mention_popup_automation_window();
                *guard = None;
            }
        }
    }

    let window_options = super::popup_window::popup_window_options(bounds, display_id);

    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| {
            AcpMentionPopupWindow::new(
                snapshot.clone(),
                layout.visible_row_limit,
                source_view.clone(),
                cx,
            )
        })
    })?;

    if let Err(error) =
        super::popup_window::configure_popup_window(&handle, cx, parent_window_handle)
    {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to configure ACP mention popup window"));
    }

    let any_handle: AnyWindowHandle = handle.into();
    let registration = super::popup_registry::AcpPopupRegistration::register(
        ACP_MENTION_POPUP_AUTOMATION_ID,
        any_handle,
    );
    if let Err(error) =
        register_mention_popup_automation_window(parent_window_handle, parent_bounds, bounds)
    {
        unregister_mention_popup_automation_window();
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error);
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(AcpMentionPopupSlot {
            handle,
            parent_window_handle,
            _registration: registration,
        });
    }

    Ok(())
}

#[derive(Clone)]
struct PopupScrollbarHandle {
    total_items: usize,
    visible_items: usize,
    scroll_offset: usize,
}

impl gpui_component::scroll::ScrollbarHandle for PopupScrollbarHandle {
    fn offset(&self) -> gpui::Point<gpui::Pixels> {
        gpui::point(
            gpui::px(0.0),
            gpui::px(-(self.scroll_offset as f32 * TRIGGER_POPUP_ROW_HEIGHT)),
        )
    }

    fn set_offset(&self, _offset: gpui::Point<gpui::Pixels>) {}

    fn content_size(&self) -> gpui::Size<gpui::Pixels> {
        gpui::size(
            gpui::px(0.0),
            gpui::px(self.total_items as f32 * TRIGGER_POPUP_ROW_HEIGHT),
        )
    }
}

pub(crate) struct AcpMentionPopupWindow {
    snapshot: AcpMentionPopupSnapshot,
    visible_row_limit: usize,
    source_view: WeakEntity<AcpChatView>,
    focus_handle: FocusHandle,
    mouse_armed_row: Option<(usize, String)>,
}

impl AcpMentionPopupWindow {
    fn new(
        snapshot: AcpMentionPopupSnapshot,
        visible_row_limit: usize,
        source_view: WeakEntity<AcpChatView>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            visible_row_limit,
            source_view,
            focus_handle: cx.focus_handle(),
            mouse_armed_row: None,
        }
    }

    fn set_snapshot(&mut self, snapshot: AcpMentionPopupSnapshot) {
        if let Some((armed_index, armed_id)) = self.mouse_armed_row.as_ref() {
            let still_same_row = snapshot
                .items
                .get(*armed_index)
                .is_some_and(|item| item.id.as_ref() == armed_id.as_str());
            if !still_same_row {
                self.mouse_armed_row = None;
            }
        }
        self.snapshot = snapshot;
    }

    fn set_visible_row_limit(&mut self, visible_row_limit: usize) {
        self.visible_row_limit = visible_row_limit;
    }

    fn visible_range(&self) -> std::ops::Range<usize> {
        inline_dropdown_visible_range_from_start(
            self.snapshot.visible_start,
            self.snapshot.selected_index,
            self.snapshot.items.len(),
            self.visible_row_limit
                .clamp(1, super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS),
        )
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

    fn select_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(view) = self.source_view.upgrade() {
            view.update(cx, |view, cx| {
                view.select_mention_index(index);
                cx.notify();
            });
            if !self.snapshot.items.is_empty() {
                self.snapshot.selected_index =
                    index.min(self.snapshot.items.len().saturating_sub(1));
                let visible = inline_dropdown_visible_range_from_start(
                    self.snapshot.visible_start,
                    self.snapshot.selected_index,
                    self.snapshot.items.len(),
                    self.visible_row_limit
                        .clamp(1, super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS),
                );
                self.snapshot.visible_start = visible.start;
            }
            cx.notify();
        } else {
            close_mention_popup_window(cx);
        }
    }

    fn handle_row_click(
        &mut self,
        index: usize,
        event: &gpui::ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(item) = self.snapshot.items.get(index) else {
            return;
        };
        let item_id = item.id.to_string();
        let is_actionable = !matches!(item.kind, ContextPickerItemKind::Inert);
        let was_mouse_armed = self
            .mouse_armed_row
            .as_ref()
            .is_some_and(|(armed_index, armed_id)| *armed_index == index && armed_id == &item_id);
        let click_count = event.click_count();
        let should_accept =
            is_actionable && should_submit_acp_picker_row_click(was_mouse_armed, click_count);

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_picker_row_click",
            row_index = index,
            item_id = %item_id,
            click_count,
            was_mouse_armed,
            is_actionable,
            should_accept,
        );

        if should_accept {
            self.mouse_armed_row = None;
            self.activate_item(index, cx);
            clear_mention_popup_window_slot();
            unregister_mention_popup_automation_window();
            window.remove_window();
        } else {
            self.mouse_armed_row = Some((index, item_id));
            self.select_item(index, cx);
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

    fn render_picker_row(
        &self,
        idx: usize,
        item: &ContextPickerItem,
        is_selected: bool,
        colors: InlineDropdownColors,
    ) -> gpui::Stateful<gpui::Div> {
        let row = acp_context_picker_item_to_inline_picker_row(item);
        let label = row.title.clone();
        let meta = row.token.clone().or_else(|| row.subtitle.clone());
        let label_hits: std::collections::HashSet<usize> =
            row.highlights.title.iter().map(|r| r.start).collect();

        let mut left_side = div().flex().items_center();

        let label_spans = render_trigger_row_label(
            &label,
            &label_hits,
            if is_selected {
                colors.foreground
            } else {
                colors.foreground.opacity(MUTED_OP)
            },
            colors.accent,
            is_selected,
        );
        left_side = left_side.child(label_spans);

        let mut content = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(left_side);

        if let Some(meta_val) = meta.filter(|val| !val.is_empty()) {
            let meta_hits: std::collections::HashSet<usize> =
                row.highlights.token.iter().map(|r| r.start).collect();
            content = content.child(
                div()
                    .px(gpui::px(6.0))
                    .py(gpui::px(2.0))
                    .rounded(gpui::px(4.0))
                    .bg(colors.foreground.opacity(0.06))
                    .child(render_trigger_row_meta_text(
                        &meta_val,
                        &meta_hits,
                        if is_selected {
                            colors.foreground.opacity(MUTED_OP)
                        } else {
                            colors.muted_foreground.opacity(0.45)
                        },
                        colors.accent.opacity(0.45),
                    )),
            );
        }

        let selected_row_bg = colors.foreground.opacity(0.18);
        let hover_row_bg = colors.foreground.opacity(0.06);

        let inner_row = div()
            .w_full()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .px(gpui::px(8.0))
            .rounded(gpui::px(6.0))
            .bg(if is_selected {
                selected_row_bg
            } else {
                gpui::transparent_black()
            })
            .when(!is_selected, |row| {
                row.hover(move |style| style.bg(hover_row_bg))
            })
            .child(content);

        div()
            .id(SharedString::from(format!("acp-mention-popup-row-{idx}")))
            .h(gpui::px(30.0))
            .w_full()
            .px(gpui::px(4.0))
            .py(gpui::px(2.0))
            .flex()
            .flex_col()
            .justify_center()
            .border_l(gpui::px(2.0))
            .border_color(if is_selected {
                colors.accent
            } else {
                gpui::transparent_black()
            })
            .child(inner_row)
    }

    fn render_picker(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let visible = self.visible_range();

        let scrollbar_handle = PopupScrollbarHandle {
            total_items: self.snapshot.items.len(),
            visible_items: visible.len(),
            scroll_offset: visible.start,
        };

        let scrollbar = Scrollbar::vertical(&scrollbar_handle).id("acp-mention-popup-scrollbar");

        let body = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .on_scroll_wheel(cx.listener(
                move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                    let delta_y = event.delta.pixel_delta(gpui::px(1.0)).y.as_f32();
                    if delta_y.abs() > 1.0 {
                        let normal_count = this.snapshot.items.len();
                        let capacity = this
                            .visible_row_limit
                            .clamp(1, super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS);
                        if normal_count > capacity {
                            let max_start = normal_count.saturating_sub(capacity);
                            let mut current_start = this.snapshot.visible_start;
                            if delta_y < 0.0 {
                                current_start = (current_start + 1).min(max_start);
                            } else {
                                current_start = current_start.saturating_sub(1);
                            }
                            if current_start != this.snapshot.visible_start {
                                this.snapshot.visible_start = current_start;
                                cx.notify();
                            }
                        }
                    }
                },
            ))
            .child(
                div().size_full().flex().flex_col().children(
                    self.snapshot
                        .items
                        .iter()
                        .enumerate()
                        .skip(visible.start)
                        .take(visible.len())
                        .map(|(idx, item)| {
                            let is_selected = idx == self.snapshot.selected_index;
                            let source_view = self.source_view.clone();
                            self.render_picker_row(idx, item, is_selected, colors)
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, event, window, cx| {
                                    if source_view.upgrade().is_none() {
                                        close_mention_popup_window(cx);
                                        return;
                                    }
                                    this.handle_row_click(idx, event, window, cx);
                                }))
                                .into_any_element()
                        }),
                ),
            )
            .child(scrollbar)
            .into_any_element();

        tracing::info!(
            target: "script_kit::tab_ai",
            popup = "mention",
            trigger = ?self.snapshot.trigger,
            item_count = self.snapshot.items.len(),
            selected_index = self.snapshot.selected_index,
            "inline_dropdown_popup_synced"
        );

        InlineDropdown::new(SharedString::from("acp-mention-popup"), body, colors)
            .vertical_padding(INLINE_POPUP_VERTICAL_PADDING / 2.0)
            .into_any_element()
    }

    fn render_empty_state(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        use crate::components::inline_dropdown::{GHOST, HINT};
        use crate::list_item::FONT_MONO;

        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let fg = colors.foreground;
        let muted_fg = colors.muted_foreground;

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

        InlineDropdown::new(
            SharedString::from("acp-mention-popup-empty-state"),
            div().into_any_element(),
            colors,
        )
        .empty_state(InlineDropdownEmptyState {
            message: SharedString::from(match self.snapshot.trigger {
                ContextPickerTrigger::Slash => "No matching commands",
                ContextPickerTrigger::Profile => "No matching profiles",
                ContextPickerTrigger::Mention => "No matching context",
            }),
            hints: chips,
        })
        .vertical_padding(INLINE_POPUP_VERTICAL_PADDING)
        .into_any_element()
    }
}

#[inline]
fn should_submit_acp_picker_row_click(was_mouse_armed: bool, click_count: usize) -> bool {
    was_mouse_armed || click_count >= 2
}

impl Focusable for AcpMentionPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AcpMentionPopupWindow {
    fn owner_is_live(&self) -> bool {
        // Rendering this popup can happen while the owning AcpChatView is still
        // inside the update that opened/refreshed it. The owner is responsible
        // for syncing or closing the popup when the mention session changes, so
        // render must only verify that the weak owner still upgrades.
        self.source_view.upgrade().is_some()
    }
}

impl Render for AcpMentionPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.owner_is_live() {
            // Owner ACP view dropped; defer the close so we don't mutate window
            // state mid-render.
            cx.defer(|cx| {
                close_mention_popup_window(cx);
            });
            return div().size_full().into_any_element();
        }
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .child(if self.snapshot.items.is_empty() {
                self.render_empty_state(cx)
            } else {
                self.render_picker(cx)
            })
            .into_any_element()
    }
}

fn render_trigger_row_label(
    text: &str,
    hits: &std::collections::HashSet<usize>,
    base: gpui::Hsla,
    accent: gpui::Hsla,
    is_selected: bool,
) -> gpui::AnyElement {
    let font_weight = if is_selected {
        gpui::FontWeight::MEDIUM
    } else {
        gpui::FontWeight::NORMAL
    };

    if hits.is_empty() {
        return div()
            .text_sm()
            .font_weight(font_weight)
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()))
            .into_any_element();
    }

    let mut spans: Vec<gpui::AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(
                div()
                    .text_sm()
                    .font_weight(font_weight)
                    .text_color(if current_highlighted { accent } else { base })
                    .child(SharedString::from(std::mem::take(&mut current)))
                    .into_any_element(),
            );
        }
        current_highlighted = is_hit;
        current.push(ch);
    }
    if !current.is_empty() {
        spans.push(
            div()
                .text_sm()
                .font_weight(font_weight)
                .text_color(if current_highlighted { accent } else { base })
                .child(SharedString::from(current))
                .into_any_element(),
        );
    }

    div()
        .flex()
        .items_center()
        .text_ellipsis()
        .children(spans)
        .into_any_element()
}

fn render_trigger_row_meta_text(
    text: &str,
    hits: &std::collections::HashSet<usize>,
    base: gpui::Hsla,
    accent: gpui::Hsla,
) -> gpui::AnyElement {
    if hits.is_empty() {
        return div()
            .text_size(gpui::px(10.5))
            .line_height(gpui::px(14.0))
            .font_family(crate::list_item::FONT_MONO)
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()))
            .into_any_element();
    }

    let mut spans: Vec<gpui::AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(
                div()
                    .text_size(gpui::px(10.5))
                    .line_height(gpui::px(14.0))
                    .font_family(crate::list_item::FONT_MONO)
                    .text_color(if current_highlighted { accent } else { base })
                    .child(SharedString::from(std::mem::take(&mut current)))
                    .into_any_element(),
            );
        }
        current_highlighted = is_hit;
        current.push(ch);
    }
    if !current.is_empty() {
        spans.push(
            div()
                .text_size(gpui::px(10.5))
                .line_height(gpui::px(14.0))
                .font_family(crate::list_item::FONT_MONO)
                .text_color(if current_highlighted { accent } else { base })
                .child(SharedString::from(current))
                .into_any_element(),
        );
    }

    div()
        .flex()
        .items_center()
        .text_ellipsis()
        .children(spans)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        inline_popup_height_for_row_height, popup_bounds, popup_height, popup_layout_above,
        popup_visible_row_limit, should_submit_acp_picker_row_click, AcpMentionPopupSnapshot,
        TRIGGER_POPUP_ROW_HEIGHT,
    };
    use crate::ai::window::context_picker::types::{
        ContextPickerItem, ContextPickerItemKind, ContextPickerTrigger, SlashCommandPayload,
    };
    use gpui::SharedString;

    #[test]
    fn popup_height_clamps_to_visible_rows() {
        let snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..16)
                .map(|ix| crate::ai::window::context_picker::types::ContextPickerItem {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(format!("Description {ix}")),
                    meta: SharedString::from(""),
                    kind: crate::ai::window::context_picker::types::ContextPickerItemKind::SlashCommand(
                        crate::ai::window::context_picker::types::SlashCommandPayload::Default {
                            name: format!("cmd-{ix}"),
                        },
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
    fn mention_popup_height_uses_row_height_for_both_triggers() {
        let items: Vec<ContextPickerItem> = (0..12)
            .map(|ix| ContextPickerItem {
                id: SharedString::from(format!("slash-cmd:default:cmd-{ix}")),
                label: SharedString::from(format!("cmd-{ix}")),
                description: SharedString::from(format!("Description {ix}")),
                meta: SharedString::from(format!("/cmd-{ix}")),
                kind: ContextPickerItemKind::SlashCommand(SlashCommandPayload::Default {
                    name: format!("cmd-{ix}"),
                }),
                score: 0,
                label_highlight_indices: Vec::new(),
                meta_highlight_indices: Vec::new(),
            })
            .collect();

        let slash_snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Slash,
            selected_index: 0,
            visible_start: 0,
            items: items.clone(),
            width: 320.0,
        };
        let mention_snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items,
            width: 320.0,
        };

        let expected_height = inline_popup_height_for_row_height(12, TRIGGER_POPUP_ROW_HEIGHT);
        assert_eq!(popup_height(&slash_snapshot), expected_height);
        assert_eq!(popup_height(&mention_snapshot), expected_height);
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

    #[test]
    fn popup_layout_above_bounds() {
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(240.0)),
            size: gpui::size(gpui::px(700.0), gpui::px(500.0)),
        };
        let snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..3)
                .map(|ix| ContextPickerItem {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(""),
                    meta: SharedString::from("@item"),
                    kind: ContextPickerItemKind::Inert,
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                })
                .collect(),
            width: 700.0,
        };
        let layout = popup_layout_above(parent, None, &snapshot);

        assert_eq!(f32::from(layout.bounds.size.width), 700.0);
        let expected_height = inline_popup_height_for_row_height(3, TRIGGER_POPUP_ROW_HEIGHT);
        assert_eq!(f32::from(layout.bounds.size.height), expected_height);
        assert_eq!(f32::from(layout.bounds.origin.x), 100.0);
        assert_eq!(f32::from(layout.bounds.origin.y), 240.0 - expected_height);
    }

    #[test]
    fn popup_layout_above_caps_height_to_parent_ratio() {
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(500.0), gpui::px(120.0)),
            size: gpui::size(gpui::px(700.0), gpui::px(400.0)),
        };
        let snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..20)
                .map(|ix| ContextPickerItem {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(format!("Description {ix}")),
                    meta: SharedString::from("@item"),
                    kind: ContextPickerItemKind::Inert,
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                })
                .collect(),
            width: 700.0,
        };

        let layout = popup_layout_above(parent, None, &snapshot);
        assert_eq!(f32::from(layout.bounds.origin.x), 500.0);
        assert!(f32::from(layout.bounds.size.height) <= 360.0);
        assert!(
            layout.visible_row_limit
                <= crate::components::inline_popup_window::INLINE_POPUP_MAX_VISIBLE_ROWS
        );
    }

    #[test]
    fn popup_layout_above_shrinks_to_short_item_list() {
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(500.0), gpui::px(120.0)),
            size: gpui::size(gpui::px(700.0), gpui::px(600.0)),
        };
        let snapshot = AcpMentionPopupSnapshot {
            trigger: ContextPickerTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..3)
                .map(|ix| ContextPickerItem {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(""),
                    meta: SharedString::from("@item"),
                    kind: ContextPickerItemKind::Inert,
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                })
                .collect(),
            width: 700.0,
        };

        let layout = popup_layout_above(parent, None, &snapshot);
        let expected_height = inline_popup_height_for_row_height(3, TRIGGER_POPUP_ROW_HEIGHT);
        assert_eq!(f32::from(layout.bounds.size.height), expected_height);
        assert_eq!(popup_visible_row_limit(snapshot.items.len(), 600.0), 3);
    }

    #[test]
    fn acp_picker_click_requires_second_single_click_after_mouse_focus() {
        assert!(!should_submit_acp_picker_row_click(false, 1));
        assert!(should_submit_acp_picker_row_click(true, 1));
    }

    #[test]
    fn acp_picker_click_still_submits_on_native_double_click() {
        assert!(should_submit_acp_picker_row_click(false, 2));
        assert!(should_submit_acp_picker_row_click(false, 3));
    }

    #[test]
    fn popup_render_liveness_does_not_read_owner_view() {
        let source = include_str!("picker_popup.rs");
        assert!(
            source.contains("fn owner_is_live(&self) -> bool")
                && source.contains("self.source_view.upgrade().is_some()"),
            "popup render liveness must not read AcpChatView; opening the popup can render while the owner is still updating"
        );
        assert!(
            !source.contains(&["view.read(cx)", ".has_active_mention_session()"].concat()),
            "the popup render path must not read the owner AcpChatView"
        );
    }
}
