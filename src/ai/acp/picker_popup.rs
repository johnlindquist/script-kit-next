use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::{
    div, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::ai::window::context_picker::empty_state_hints;
use crate::ai::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, ContextPickerTrigger,
};
use crate::components::inline_dropdown::{
    inline_dropdown_visible_range_from_start, render_soft_compact_picker_row, InlineDropdown,
    InlineDropdownColors, InlineDropdownEmptyState, InlineDropdownSynopsis,
    CONTEXT_PICKER_SYNOPSIS_HEIGHT, SOFT_COMPACT_PICKER_ROW_HEIGHT,
};

use super::view::AcpChatView;

const ACP_MENTION_POPUP_AUTOMATION_ID: &str = "acp-mention-popup";

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

fn clear_mention_popup_window_slot() {
    if let Some(storage) = ACP_MENTION_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

pub(crate) fn close_mention_popup_window(cx: &mut App) {
    crate::windows::remove_runtime_window_handle(ACP_MENTION_POPUP_AUTOMATION_ID);
    crate::windows::remove_automation_window(ACP_MENTION_POPUP_AUTOMATION_ID);
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

    super::popup_window::dense_picker_height_for_row_height(
        snapshot.items.len(),
        SOFT_COMPACT_PICKER_ROW_HEIGHT,
    ) + CONTEXT_PICKER_SYNOPSIS_HEIGHT
}

fn automation_bounds(bounds: Bounds<Pixels>) -> crate::protocol::AutomationWindowBounds {
    crate::protocol::AutomationWindowBounds {
        x: f32::from(bounds.origin.x) as f64,
        y: f32::from(bounds.origin.y) as f64,
        width: f32::from(bounds.size.width) as f64,
        height: f32::from(bounds.size.height) as f64,
    }
}

fn resolve_mention_popup_parent_automation_id(
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
) -> anyhow::Result<String> {
    for window in crate::windows::list_automation_windows() {
        if crate::windows::get_runtime_window_handle(&window.id)
            .is_some_and(|handle| handle == parent_window_handle)
        {
            return Ok(window.id);
        }
    }

    if crate::get_main_window_handle().is_some_and(|handle| handle == parent_window_handle) {
        let parent_id = "main".to_string();
        crate::windows::upsert_runtime_window_handle(&parent_id, parent_window_handle);
        let preserved_semantic_surface = crate::windows::list_automation_windows()
            .into_iter()
            .find(|window| window.id == parent_id)
            .and_then(|window| window.semantic_surface)
            .unwrap_or_else(|| "acpChat".to_string());
        crate::windows::upsert_automation_window(crate::protocol::AutomationWindowInfo {
            id: parent_id.clone(),
            kind: crate::protocol::AutomationWindowKind::Main,
            title: Some("Script Kit".to_string()),
            focused: true,
            visible: true,
            semantic_surface: Some(preserved_semantic_surface),
            bounds: Some(automation_bounds(parent_bounds)),
            parent_window_id: None,
            parent_kind: None,
        });
        return Ok(parent_id);
    }

    anyhow::bail!("Cannot register ACP mention popup: parent automation identity is required");
}

fn register_mention_popup_automation_window(
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    popup_bounds: Bounds<Pixels>,
) -> anyhow::Result<()> {
    let parent_id =
        resolve_mention_popup_parent_automation_id(parent_window_handle, parent_bounds)?;
    crate::windows::register_attached_popup(
        ACP_MENTION_POPUP_AUTOMATION_ID.to_string(),
        crate::protocol::AutomationWindowKind::PromptPopup,
        Some("ACP Mention Picker".to_string()),
        Some("promptPopup".to_string()),
        Some(automation_bounds(popup_bounds)),
        Some(parent_id.as_str()),
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

                crate::windows::remove_runtime_window_handle(ACP_MENTION_POPUP_AUTOMATION_ID);
                crate::windows::remove_automation_window(ACP_MENTION_POPUP_AUTOMATION_ID);
                *guard = None;
            } else {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
                crate::windows::remove_runtime_window_handle(ACP_MENTION_POPUP_AUTOMATION_ID);
                crate::windows::remove_automation_window(ACP_MENTION_POPUP_AUTOMATION_ID);
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

    let any_handle: AnyWindowHandle = handle.into();
    crate::windows::upsert_runtime_window_handle(ACP_MENTION_POPUP_AUTOMATION_ID, any_handle);
    if let Err(error) =
        register_mention_popup_automation_window(parent_window_handle, parent_bounds, bounds)
    {
        crate::windows::remove_runtime_window_handle(ACP_MENTION_POPUP_AUTOMATION_ID);
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error);
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
    mouse_armed_row: Option<(usize, String)>,
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

    fn visible_range(&self) -> std::ops::Range<usize> {
        inline_dropdown_visible_range_from_start(
            self.snapshot.visible_start,
            self.snapshot.selected_index,
            self.snapshot.items.len(),
            super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS,
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
                    super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS,
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
        if self.snapshot.trigger == ContextPickerTrigger::Slash {
            if let ContextPickerItemKind::SlashCommand(payload) = &item.kind {
                let label = SharedString::from(format!("/{}", payload.slash_name()));
                let shifted_label_hits = item
                    .label_highlight_indices
                    .iter()
                    .map(|ix| ix + 1)
                    .collect::<Vec<_>>();

                return render_soft_compact_picker_row(
                    SharedString::from(format!("acp-mention-popup-row-{idx}")),
                    label,
                    Some(SharedString::from(payload.owner_label())),
                    &shifted_label_hits,
                    &[],
                    is_selected,
                    colors,
                );
            }

            return render_soft_compact_picker_row(
                SharedString::from(format!("acp-mention-popup-row-{idx}")),
                item.label.clone(),
                None,
                &item.label_highlight_indices,
                &[],
                is_selected,
                colors,
            );
        }

        render_soft_compact_picker_row(
            SharedString::from(format!("acp-mention-popup-row-{idx}")),
            item.label.clone(),
            Some(item.meta.clone()),
            &item.label_highlight_indices,
            &item.meta_highlight_indices,
            is_selected,
            colors,
        )
    }

    fn render_picker(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let visible = self.visible_range();
        let selected_item = self
            .snapshot
            .items
            .get(self.snapshot.selected_index)
            .cloned();

        let body = div()
            .size_full()
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
            )
            .into_any_element();

        let synopsis = selected_item
            .filter(|item| !item.description.is_empty())
            .map(|item| InlineDropdownSynopsis {
                label: item.label.clone(),
                meta: item.meta.clone(),
                description: item.description.clone(),
            });

        tracing::info!(
            target: "script_kit::tab_ai",
            popup = "mention",
            trigger = ?self.snapshot.trigger,
            item_count = self.snapshot.items.len(),
            selected_index = self.snapshot.selected_index,
            "inline_dropdown_popup_synced"
        );

        InlineDropdown::new(SharedString::from("acp-mention-popup"), body, colors)
            .synopsis(synopsis)
            .vertical_padding(super::popup_window::DENSE_PICKER_VERTICAL_PADDING / 2.0)
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
            message: SharedString::from(if self.snapshot.trigger == ContextPickerTrigger::Slash {
                "No matching commands"
            } else {
                "No matching context"
            }),
            hints: chips,
        })
        .vertical_padding(super::popup_window::DENSE_PICKER_VERTICAL_PADDING)
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
    use super::{
        popup_bounds, popup_height, should_submit_acp_picker_row_click, AcpMentionPopupSnapshot,
    };
    use crate::ai::window::context_picker::types::{
        ContextPickerItem, ContextPickerItemKind, ContextPickerTrigger, SlashCommandPayload,
    };
    use crate::components::inline_dropdown::{
        CONTEXT_PICKER_SYNOPSIS_HEIGHT, SOFT_COMPACT_PICKER_ROW_HEIGHT,
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
    fn mention_popup_height_uses_soft_compact_rows_for_both_triggers() {
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

        let expected_height = super::super::popup_window::dense_picker_height_for_row_height(
            12,
            SOFT_COMPACT_PICKER_ROW_HEIGHT,
        ) + CONTEXT_PICKER_SYNOPSIS_HEIGHT;
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
    fn acp_picker_click_requires_second_single_click_after_mouse_focus() {
        assert!(!should_submit_acp_picker_row_click(false, 1));
        assert!(should_submit_acp_picker_row_click(true, 1));
    }

    #[test]
    fn acp_picker_click_still_submits_on_native_double_click() {
        assert!(should_submit_acp_picker_row_click(false, 2));
        assert!(should_submit_acp_picker_row_click(false, 3));
    }
}
