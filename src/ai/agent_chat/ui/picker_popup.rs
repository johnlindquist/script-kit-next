use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::{
    div, prelude::FluentBuilder, px, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::ai::context_selector::context_selector_empty_state_hints;
use crate::ai::context_selector::types::{
    ContextSelectorRow, ContextSelectorRowKind, ContextSelectorTrigger,
};
use crate::components::inline_dropdown::{
    inline_dropdown_visible_range_from_start, InlineDropdown, InlineDropdownColors,
    InlineDropdownEmptyState,
};
use crate::components::inline_picker::{
    InlinePickerHighlights, InlinePickerRow, InlinePickerRowKind,
};
use crate::components::inline_popup_window::{
    inline_popup_height_for_row_height, INLINE_POPUP_EDGE_GUTTER, INLINE_POPUP_MAX_VISIBLE_ROWS,
    INLINE_POPUP_VERTICAL_PADDING,
};
use gpui_component::scroll::Scrollbar;

use super::view::AgentChatView;

const AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID: &str = "agent_chat-mention-popup";
const AGENT_CHAT_MENTION_POPUP_MAX_PARENT_HEIGHT_RATIO: f32 = 0.90;

pub(crate) fn agent_chat_context_selector_row_to_inline_picker_row(
    item: &ContextSelectorRow,
) -> InlinePickerRow {
    let (kind, title, token, token_highlights, enabled) = match &item.kind {
        ContextSelectorRowKind::SlashCommand(payload) => (
            InlinePickerRowKind::SlashCommand,
            SharedString::from(format!("/{}", payload.slash_name())),
            Some(SharedString::from(payload.owner_label())),
            item.meta_highlight_indices.clone(),
            true,
        ),
        ContextSelectorRowKind::Inert => (
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
        disabled_reason: (!enabled).then(|| SharedString::from("Inert context selector row")),
    }
}

#[derive(Clone)]
pub(crate) struct AgentChatMentionPopupSnapshot {
    pub(crate) trigger: ContextSelectorTrigger,
    pub(crate) selected_index: usize,
    pub(crate) visible_start: usize,
    pub(crate) items: Vec<ContextSelectorRow>,
    pub(crate) width: f32,
}

#[derive(Clone)]
pub(crate) struct AgentChatMentionPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) display_bounds: Option<Bounds<Pixels>>,
    pub(crate) source_view: WeakEntity<AgentChatView>,
    pub(crate) snapshot: AgentChatMentionPopupSnapshot,
    pub(crate) left: f32,
    pub(crate) top: f32,
}

struct AgentChatMentionPopupSlot {
    handle: WindowHandle<AgentChatMentionPopupWindow>,
    parent_window_handle: AnyWindowHandle,
    _registration: super::popup_registry::AgentChatPopupRegistration,
}

static AGENT_CHAT_MENTION_POPUP_WINDOW: OnceLock<Mutex<Option<AgentChatMentionPopupSlot>>> =
    OnceLock::new();

fn clear_mention_popup_window_slot() {
    if let Some(storage) = AGENT_CHAT_MENTION_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

fn unregister_mention_popup_automation_window() {
    super::popup_window::unregister_agent_chat_prompt_popup_automation_window(
        AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID,
    );
}

pub(crate) fn close_mention_popup_window(cx: &mut App) {
    unregister_mention_popup_automation_window();
    if let Some(storage) = AGENT_CHAT_MENTION_POPUP_WINDOW.get() {
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
    if let Some(storage) = AGENT_CHAT_MENTION_POPUP_WINDOW.get() {
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
pub(crate) fn get_mention_popup_snapshot(cx: &gpui::App) -> Option<AgentChatMentionPopupSnapshot> {
    let storage = AGENT_CHAT_MENTION_POPUP_WINDOW.get()?;
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
    let (handle, idx) = {
        let storage = AGENT_CHAT_MENTION_POPUP_WINDOW.get()?;
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
        (slot.handle, idx)
    };
    let _ = handle.update(cx, |popup, _window, cx| {
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

fn agent_chat_mention_popup_main_menu_theme() -> crate::designs::MainMenuThemeVariant {
    crate::designs::current_main_menu_theme()
}

fn agent_chat_mention_popup_row_height_for_theme(
    theme: crate::designs::MainMenuThemeVariant,
) -> f32 {
    crate::list_item::effective_list_item_height_for_theme(theme)
}

fn popup_height(snapshot: &AgentChatMentionPopupSnapshot) -> f32 {
    let row_count = snapshot.items.len().min(INLINE_POPUP_MAX_VISIBLE_ROWS);
    inline_popup_height_for_row_height(
        row_count,
        agent_chat_mention_popup_row_height_for_theme(agent_chat_mention_popup_main_menu_theme()),
    )
}

fn popup_visible_row_limit(item_count: usize, parent_height: f32) -> usize {
    if item_count == 0 {
        return 0;
    }

    let max_height = (parent_height * AGENT_CHAT_MENTION_POPUP_MAX_PARENT_HEIGHT_RATIO).max(1.0);
    let hard_limit = item_count.min(INLINE_POPUP_MAX_VISIBLE_ROWS);

    (1..=hard_limit)
        .rev()
        .find(|rows| {
            let height = inline_popup_height_for_row_height(
                *rows,
                agent_chat_mention_popup_row_height_for_theme(
                    agent_chat_mention_popup_main_menu_theme(),
                ),
            );
            height <= max_height
        })
        .unwrap_or(1)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AgentChatMentionPopupLayout {
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) visible_row_limit: usize,
}

fn register_mention_popup_automation_window(
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    popup_bounds: Bounds<Pixels>,
) -> anyhow::Result<()> {
    super::popup_window::register_agent_chat_prompt_popup_automation_window(
        AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID,
        "Agent Chat Mention Picker",
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
    snapshot: &AgentChatMentionPopupSnapshot,
) -> AgentChatMentionPopupLayout {
    let width = snapshot.width;

    let display_top = display_bounds
        .map(|db| db.origin.y.as_f32() + INLINE_POPUP_EDGE_GUTTER)
        .unwrap_or(0.0);
    let available_height = (parent_bounds.origin.y.as_f32() - display_top).max(1.0);

    let visible_row_limit = popup_visible_row_limit(snapshot.items.len(), available_height);
    let row_count = snapshot.items.len().min(visible_row_limit);
    let height = inline_popup_height_for_row_height(
        row_count,
        agent_chat_mention_popup_row_height_for_theme(agent_chat_mention_popup_main_menu_theme()),
    );

    let preferred_left = parent_bounds.origin.x.as_f32();
    let left = display_bounds
        .map(|display_bounds| {
            let display_left = display_bounds.origin.x.as_f32();
            let display_right = display_left + display_bounds.size.width.as_f32();
            preferred_left.clamp(display_left, (display_right - width).max(display_left))
        })
        .unwrap_or(preferred_left);

    let top = parent_bounds.origin.y.as_f32() - height;

    AgentChatMentionPopupLayout {
        bounds: Bounds {
            origin: gpui::point(gpui::px(left), gpui::px(top)),
            size: gpui::size(gpui::px(width), gpui::px(height)),
        },
        visible_row_limit,
    }
}

pub(crate) fn sync_mention_popup_window(
    cx: &mut App,
    request: AgentChatMentionPopupRequest,
) -> anyhow::Result<()> {
    let AgentChatMentionPopupRequest {
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

    let storage = AGENT_CHAT_MENTION_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
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
                            event = "agent_chat_mention_popup_registry_failed",
                            error = %error,
                            "Failed to refresh Agent Chat mention popup automation registry entry"
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
            AgentChatMentionPopupWindow::new(
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
        return Err(error.context("failed to configure Agent Chat mention popup window"));
    }

    let any_handle: AnyWindowHandle = handle.into();
    let registration = super::popup_registry::AgentChatPopupRegistration::register(
        AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID,
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
        *guard = Some(AgentChatMentionPopupSlot {
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
    row_height: f32,
}

impl gpui_component::scroll::ScrollbarHandle for PopupScrollbarHandle {
    fn offset(&self) -> gpui::Point<gpui::Pixels> {
        gpui::point(
            gpui::px(0.0),
            gpui::px(-(self.scroll_offset as f32 * self.row_height)),
        )
    }

    fn set_offset(&self, _offset: gpui::Point<gpui::Pixels>) {}

    fn content_size(&self) -> gpui::Size<gpui::Pixels> {
        gpui::size(
            gpui::px(0.0),
            gpui::px(self.total_items as f32 * self.row_height),
        )
    }
}

struct AgentChatMentionListRowSpec {
    title: String,
    description: Option<String>,
    source_hint: Option<String>,
    icon_kind: Option<crate::list_item::IconKind>,
    type_accessory: crate::list_item::TypeAccessory,
    title_highlights: Vec<usize>,
    description_highlights: Vec<usize>,
    enabled: bool,
}

fn non_empty_string(value: &SharedString) -> Option<String> {
    let value = value.as_ref();
    (!value.is_empty()).then(|| value.to_string())
}

fn agent_chat_context_selector_row_to_list_row_spec(
    item: &ContextSelectorRow,
) -> AgentChatMentionListRowSpec {
    let mut title = item.label.to_string();
    let mut description = non_empty_string(&item.description);
    let mut source_hint = non_empty_string(&item.meta);
    let mut icon_kind = None;
    let mut type_accessory = crate::list_item::TypeAccessory {
        label: "Context",
        icon_name: "at-sign",
    };
    let mut title_highlights = item.label_highlight_indices.clone();
    let description_highlights = Vec::new();
    let enabled = !matches!(item.kind, ContextSelectorRowKind::Inert);

    match &item.kind {
        ContextSelectorRowKind::SlashCommand(payload) => {
            title = format!("/{}", payload.slash_name());
            source_hint = Some(payload.owner_label().to_string());
            title_highlights = item
                .label_highlight_indices
                .iter()
                .map(|ix| ix.saturating_add(1))
                .collect();
            type_accessory = crate::list_item::TypeAccessory {
                label: "Command",
                icon_name: "command",
            };
        }
        ContextSelectorRowKind::AgentChatProfile { icon_name, .. } => {
            let icon_path = crate::components::footer_chrome::footer_icon_path_or_profile(
                icon_name
                    .as_deref()
                    .unwrap_or(crate::components::footer_chrome::FOOTER_PROFILE_ICON_TOKEN),
            );
            icon_kind = Some(crate::list_item::IconKind::Svg(icon_path));
            type_accessory = crate::list_item::TypeAccessory {
                label: "Profile",
                icon_name: "bot",
            };
        }
        ContextSelectorRowKind::File(_) => {
            type_accessory = crate::list_item::TypeAccessory {
                label: "File",
                icon_name: "file",
            };
        }
        ContextSelectorRowKind::Folder(_) => {
            type_accessory = crate::list_item::TypeAccessory {
                label: "Folder",
                icon_name: "folder",
            };
        }
        ContextSelectorRowKind::Portal(_) | ContextSelectorRowKind::PortalPrefix(_) => {
            type_accessory = crate::list_item::TypeAccessory {
                label: "Portal",
                icon_name: "panel-top-open",
            };
        }
        ContextSelectorRowKind::PortalResult(_) => {
            type_accessory = crate::list_item::TypeAccessory {
                label: "Result",
                icon_name: "search",
            };
        }
        ContextSelectorRowKind::BuiltIn(_) => {}
        ContextSelectorRowKind::Inert => {
            if description.is_none() {
                description = source_hint.clone();
            }
        }
    }

    AgentChatMentionListRowSpec {
        title,
        description,
        source_hint,
        icon_kind,
        type_accessory,
        title_highlights,
        description_highlights,
        enabled,
    }
}

pub(crate) struct AgentChatMentionPopupWindow {
    snapshot: AgentChatMentionPopupSnapshot,
    visible_row_limit: usize,
    source_view: WeakEntity<AgentChatView>,
    focus_handle: FocusHandle,
    mouse_armed_row: Option<(usize, String)>,
}

impl AgentChatMentionPopupWindow {
    fn new(
        snapshot: AgentChatMentionPopupSnapshot,
        visible_row_limit: usize,
        source_view: WeakEntity<AgentChatView>,
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

    fn set_snapshot(&mut self, snapshot: AgentChatMentionPopupSnapshot) {
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
        let is_actionable = !matches!(item.kind, ContextSelectorRowKind::Inert);
        let was_mouse_armed = self
            .mouse_armed_row
            .as_ref()
            .is_some_and(|(armed_index, armed_id)| *armed_index == index && armed_id == &item_id);
        let click_count = event.click_count();
        let should_accept = is_actionable
            && should_submit_agent_chat_picker_row_click(was_mouse_armed, click_count);

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_picker_row_click",
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
        item: &ContextSelectorRow,
        is_selected: bool,
        theme: &crate::theme::Theme,
        main_menu_theme: crate::designs::MainMenuThemeVariant,
    ) -> gpui::Stateful<gpui::Div> {
        let spec = agent_chat_context_selector_row_to_list_row_spec(item);
        let colors = crate::list_item::ListItemColors::from_theme(theme);
        let row_height = agent_chat_mention_popup_row_height_for_theme(main_menu_theme);
        let row = crate::list_item::ListItem::new(spec.title, colors)
            .index(idx)
            .selected(is_selected)
            .hovered(false)
            .main_menu_theme(main_menu_theme)
            .semantic_id(format!("choice:{idx}:{}", item.id))
            .description_opt(spec.description)
            .source_hint_opt(spec.source_hint)
            .icon_kind_opt(spec.icon_kind)
            .type_accessory(spec.type_accessory)
            .highlight_indices(spec.title_highlights)
            .description_highlight_indices(spec.description_highlights);

        div()
            .id(SharedString::from(format!(
                "agent_chat-mention-popup-row-{idx}"
            )))
            .h(gpui::px(row_height))
            .w_full()
            .when(!spec.enabled, |d| d.opacity(0.55))
            .child(row)
    }

    fn render_picker(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let main_menu_theme = agent_chat_mention_popup_main_menu_theme();
        let row_height = agent_chat_mention_popup_row_height_for_theme(main_menu_theme);
        let visible = self.visible_range();

        let scrollbar_handle = PopupScrollbarHandle {
            total_items: self.snapshot.items.len(),
            visible_items: visible.len(),
            scroll_offset: visible.start,
            row_height,
        };

        let scrollbar =
            Scrollbar::vertical(&scrollbar_handle).id("agent_chat-mention-popup-scrollbar");

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
                            self.render_picker_row(idx, item, is_selected, &theme, main_menu_theme)
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

        InlineDropdown::new(SharedString::from("agent_chat-mention-popup"), body, colors)
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
        for hint in context_selector_empty_state_hints(self.snapshot.trigger).iter() {
            let hint_display = SharedString::from(hint.display);
            let hint_insertion = hint.insertion.to_string();
            let close_after_apply = !hint.insertion.ends_with(':');

            chips.push(
                div()
                    .id(SharedString::from(format!(
                        "agent_chat-mention-popup-hint-{}",
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
            SharedString::from("agent_chat-mention-popup-empty-state"),
            div().into_any_element(),
            colors,
        )
        .empty_state(InlineDropdownEmptyState {
            message: SharedString::from(match self.snapshot.trigger {
                ContextSelectorTrigger::Slash => "No matching commands",
                ContextSelectorTrigger::Profile => "No matching profiles",
                ContextSelectorTrigger::Mention => "No matching context",
            }),
            hints: chips,
        })
        .vertical_padding(INLINE_POPUP_VERTICAL_PADDING)
        .into_any_element()
    }
}

#[inline]
fn should_submit_agent_chat_picker_row_click(was_mouse_armed: bool, click_count: usize) -> bool {
    was_mouse_armed || click_count >= 2
}

impl Focusable for AgentChatMentionPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AgentChatMentionPopupWindow {
    fn owner_is_live(&self) -> bool {
        // Rendering this popup can happen while the owning AgentChatView is still
        // inside the update that opened/refreshed it. The owner is responsible
        // for syncing or closing the popup when the mention session changes, so
        // render must only verify that the weak owner still upgrades.
        self.source_view.upgrade().is_some()
    }
}

impl Render for AgentChatMentionPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.owner_is_live() {
            // Owner Agent Chat view dropped; defer the close so we don't mutate window
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

#[cfg(test)]
mod tests {
    use super::{
        agent_chat_mention_popup_main_menu_theme, agent_chat_mention_popup_row_height_for_theme,
        inline_popup_height_for_row_height, popup_bounds, popup_height, popup_layout_above,
        popup_visible_row_limit, should_submit_agent_chat_picker_row_click,
        AgentChatMentionPopupSnapshot,
    };
    use crate::ai::context_selector::types::{
        ContextSelectorRow, ContextSelectorRowKind, ContextSelectorTrigger, SlashCommandPayload,
    };
    use gpui::SharedString;

    #[test]
    fn popup_height_clamps_to_visible_rows() {
        let snapshot = AgentChatMentionPopupSnapshot {
            trigger: ContextSelectorTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items:
                (0..16)
                    .map(
                        |ix| {
                            crate::ai::context_selector::types::ContextSelectorRow {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(format!("Description {ix}")),
                    meta: SharedString::from(""),
                    kind: crate::ai::context_selector::types::ContextSelectorRowKind::SlashCommand(
                        crate::ai::context_selector::types::SlashCommandPayload::Default {
                            name: format!("cmd-{ix}"),
                        },
                    ),
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                }
                        },
                    )
                    .collect(),
            width: 320.0,
        };
        assert!(popup_height(&snapshot) > 0.0);
    }

    #[test]
    fn mention_popup_height_uses_row_height_for_both_triggers() {
        let items: Vec<ContextSelectorRow> = (0..12)
            .map(|ix| ContextSelectorRow {
                id: SharedString::from(format!("slash-cmd:default:cmd-{ix}")),
                label: SharedString::from(format!("cmd-{ix}")),
                description: SharedString::from(format!("Description {ix}")),
                meta: SharedString::from(format!("/cmd-{ix}")),
                kind: ContextSelectorRowKind::SlashCommand(SlashCommandPayload::Default {
                    name: format!("cmd-{ix}"),
                }),
                score: 0,
                label_highlight_indices: Vec::new(),
                meta_highlight_indices: Vec::new(),
            })
            .collect();

        let slash_snapshot = AgentChatMentionPopupSnapshot {
            trigger: ContextSelectorTrigger::Slash,
            selected_index: 0,
            visible_start: 0,
            items: items.clone(),
            width: 320.0,
        };
        let mention_snapshot = AgentChatMentionPopupSnapshot {
            trigger: ContextSelectorTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items,
            width: 320.0,
        };

        let expected_height = inline_popup_height_for_row_height(
            12,
            agent_chat_mention_popup_row_height_for_theme(
                agent_chat_mention_popup_main_menu_theme(),
            ),
        );
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
        let snapshot = AgentChatMentionPopupSnapshot {
            trigger: ContextSelectorTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..3)
                .map(|ix| ContextSelectorRow {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(""),
                    meta: SharedString::from("@item"),
                    kind: ContextSelectorRowKind::Inert,
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                })
                .collect(),
            width: 700.0,
        };
        let layout = popup_layout_above(parent, None, &snapshot);

        assert_eq!(f32::from(layout.bounds.size.width), 700.0);
        let expected_height = inline_popup_height_for_row_height(
            3,
            agent_chat_mention_popup_row_height_for_theme(
                agent_chat_mention_popup_main_menu_theme(),
            ),
        );
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
        let snapshot = AgentChatMentionPopupSnapshot {
            trigger: ContextSelectorTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..20)
                .map(|ix| ContextSelectorRow {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(format!("Description {ix}")),
                    meta: SharedString::from("@item"),
                    kind: ContextSelectorRowKind::Inert,
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
        // Anchor far enough down that the available-height cap never bites:
        // this test pins the shrink-to-content behavior (3 items => 3 rows,
        // not INLINE_POPUP_MAX_VISIBLE_ROWS) independent of themed row height.
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(500.0), gpui::px(500.0)),
            size: gpui::size(gpui::px(700.0), gpui::px(600.0)),
        };
        let snapshot = AgentChatMentionPopupSnapshot {
            trigger: ContextSelectorTrigger::Mention,
            selected_index: 0,
            visible_start: 0,
            items: (0..3)
                .map(|ix| ContextSelectorRow {
                    id: SharedString::from(format!("item-{ix}")),
                    label: SharedString::from(format!("Item {ix}")),
                    description: SharedString::from(""),
                    meta: SharedString::from("@item"),
                    kind: ContextSelectorRowKind::Inert,
                    score: 0,
                    label_highlight_indices: Vec::new(),
                    meta_highlight_indices: Vec::new(),
                })
                .collect(),
            width: 700.0,
        };

        let layout = popup_layout_above(parent, None, &snapshot);
        let expected_height = inline_popup_height_for_row_height(
            3,
            agent_chat_mention_popup_row_height_for_theme(
                agent_chat_mention_popup_main_menu_theme(),
            ),
        );
        assert_eq!(f32::from(layout.bounds.size.height), expected_height);
        assert_eq!(popup_visible_row_limit(snapshot.items.len(), 600.0), 3);
    }

    #[test]
    fn mention_popup_rows_use_main_list_item_chrome() {
        let source = include_str!("picker_popup.rs");
        let row_body = source
            .split("fn render_picker_row(")
            .nth(1)
            .and_then(|tail| tail.split("fn render_picker(").next())
            .expect("render_picker_row should exist before render_picker");

        for required in [
            "crate::list_item::ListItem::new",
            "crate::list_item::ListItemColors::from_theme",
            ".selected(is_selected)",
            ".main_menu_theme(",
            ".semantic_id(format!(\"choice:{idx}:{}\", item.id))",
        ] {
            assert!(
                row_body.contains(required),
                "missing shared ListItem row contract: {required}"
            );
        }

        for forbidden in [
            "render_trigger_row_label",
            "render_trigger_row_meta_text",
            ".border_l(gpui::px(2.0))",
            "selected_row_bg",
            "hover_row_bg",
        ] {
            assert!(
                !row_body.contains(forbidden),
                "must not reintroduce bespoke popup row chrome: {forbidden}"
            );
        }
    }

    #[test]
    fn agent_chat_picker_click_requires_second_single_click_after_mouse_focus() {
        assert!(!should_submit_agent_chat_picker_row_click(false, 1));
        assert!(should_submit_agent_chat_picker_row_click(true, 1));
    }

    #[test]
    fn agent_chat_picker_click_still_submits_on_native_double_click() {
        assert!(should_submit_agent_chat_picker_row_click(false, 2));
        assert!(should_submit_agent_chat_picker_row_click(false, 3));
    }

    #[test]
    fn popup_render_liveness_does_not_read_owner_view() {
        let source = include_str!("picker_popup.rs");
        assert!(
            source.contains("fn owner_is_live(&self) -> bool")
                && source.contains("self.source_view.upgrade().is_some()"),
            "popup render liveness must not read AgentChatView; opening the popup can render while the owner is still updating"
        );
        assert!(
            !source.contains(&["view.read(cx)", ".has_active_mention_session()"].concat()),
            "the popup render path must not read the owner AgentChatView"
        );
    }
}
