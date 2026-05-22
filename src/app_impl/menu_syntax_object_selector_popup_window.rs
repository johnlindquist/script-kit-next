use std::sync::{Mutex, OnceLock};

use gpui::{
    div, prelude::FluentBuilder, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::components::inline_dropdown::{
    inline_dropdown_visible_range_from_start, InlineDropdown, InlineDropdownColors, MUTED_OP,
};
use crate::components::inline_popup_window::{
    configure_inline_popup_window, inline_popup_height_for_row_height,
    inline_popup_window_options, set_inline_popup_window_bounds,
    INLINE_POPUP_EDGE_GUTTER, INLINE_POPUP_MAX_VISIBLE_ROWS, INLINE_POPUP_VERTICAL_PADDING,
};
use crate::components::scrollbar::{Scrollbar, ScrollbarColors};
use crate::menu_syntax::{ObjectSelectorRow, ObjectSelectorSnapshot};
use crate::ScriptListApp;

const MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID: &str = "menu-syntax-object-selector-popup";
const MENU_SYNTAX_OBJECT_SELECTOR_MAX_PARENT_HEIGHT_RATIO: f32 = 0.90;

#[derive(Clone)]
pub(crate) struct MenuSyntaxObjectSelectorPopupSnapshot {
    pub(crate) snapshot: ObjectSelectorSnapshot,
    pub(crate) selected_row_id: Option<String>,
    pub(crate) visible_start: usize,
    pub(crate) visible_row_limit: usize,
    pub(crate) width: f32,
}

impl MenuSyntaxObjectSelectorPopupSnapshot {
    fn selected_index(&self) -> Option<usize> {
        self.selected_row_id
            .as_deref()
            .and_then(|id| self.snapshot.rows.iter().position(|row| row.id == id))
    }
}

#[derive(Clone)]
pub(crate) struct MenuSyntaxObjectSelectorPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_bounds: Option<Bounds<Pixels>>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<ScriptListApp>,
    pub(crate) snapshot: MenuSyntaxObjectSelectorPopupSnapshot,
}

#[derive(Clone, Copy)]
struct MenuSyntaxObjectSelectorPopupSlot {
    handle: WindowHandle<MenuSyntaxObjectSelectorPopupWindow>,
    parent_window_handle: AnyWindowHandle,
}

static MENU_SYNTAX_OBJECT_SELECTOR_POPUP_WINDOW: OnceLock<
    Mutex<Option<MenuSyntaxObjectSelectorPopupSlot>>,
> = OnceLock::new();

pub(crate) fn close_menu_syntax_object_selector_popup_window(cx: &mut App) {
    crate::windows::automation_surface_collector::remove_menu_syntax_object_selector_popup_snapshot(
        MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
    );
    crate::windows::remove_automation_window(MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID);
    if let Some(storage) = MENU_SYNTAX_OBJECT_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

pub(crate) fn is_menu_syntax_object_selector_popup_window_open() -> bool {
    if let Some(storage) = MENU_SYNTAX_OBJECT_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            return guard.is_some();
        }
    }
    false
}

fn clear_menu_syntax_object_selector_popup_slot() {
    if let Some(storage) = MENU_SYNTAX_OBJECT_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

const TRIGGER_POPUP_ROW_HEIGHT: f32 = 30.0;

fn popup_height(snapshot: &MenuSyntaxObjectSelectorPopupSnapshot) -> f32 {
    inline_popup_height_for_row_height(
        snapshot
            .snapshot
            .rows
            .len()
            .min(snapshot.visible_row_limit)
            .min(INLINE_POPUP_MAX_VISIBLE_ROWS),
        TRIGGER_POPUP_ROW_HEIGHT,
    )
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MenuSyntaxObjectSelectorPopupLayout {
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) visible_row_limit: usize,
}

fn popup_visible_row_limit(
    snapshot: &MenuSyntaxObjectSelectorPopupSnapshot,
    parent_height: f32,
) -> usize {
    let row_count = snapshot.snapshot.rows.len();
    if row_count == 0 {
        return 0;
    }
    let max_height = (parent_height * MENU_SYNTAX_OBJECT_SELECTOR_MAX_PARENT_HEIGHT_RATIO).max(1.0);
    let hard_limit = row_count.min(INLINE_POPUP_MAX_VISIBLE_ROWS);
    (1..=hard_limit)
        .rev()
        .find(|rows| {
            let mut candidate = snapshot.clone();
            candidate.visible_row_limit = *rows;
            popup_height(&candidate) <= max_height
        })
        .unwrap_or(1)
}

pub(crate) fn menu_syntax_object_selector_popup_layout_above(
    parent_bounds: Bounds<Pixels>,
    display_bounds: Option<Bounds<Pixels>>,
    snapshot: &MenuSyntaxObjectSelectorPopupSnapshot,
) -> MenuSyntaxObjectSelectorPopupLayout {
    let mut snapshot = snapshot.clone();
    let width = snapshot.width;

    let display_top = display_bounds
        .map(|db| db.origin.y.as_f32() + INLINE_POPUP_EDGE_GUTTER)
        .unwrap_or(0.0);
    let available_height = (parent_bounds.origin.y.as_f32() - display_top).max(1.0);

    let visible_row_limit =
        popup_visible_row_limit(&snapshot, available_height);
    snapshot.visible_row_limit = visible_row_limit;

    let height = popup_height(&snapshot);

    let preferred_left = parent_bounds.origin.x.as_f32();
    let left = display_bounds
        .map(|display_bounds| {
            let display_left = display_bounds.origin.x.as_f32();
            let display_right = display_left + display_bounds.size.width.as_f32();
            preferred_left.clamp(display_left, (display_right - width).max(display_left))
        })
        .unwrap_or(preferred_left);

    let top = parent_bounds.origin.y.as_f32() - height;

    MenuSyntaxObjectSelectorPopupLayout {
        bounds: Bounds {
            origin: gpui::point(gpui::px(left), gpui::px(top)),
            size: gpui::size(gpui::px(width), gpui::px(height)),
        },
        visible_row_limit,
    }
}

pub(crate) fn sync_menu_syntax_object_selector_popup_window(
    cx: &mut App,
    request: MenuSyntaxObjectSelectorPopupRequest,
) -> anyhow::Result<()> {
    let MenuSyntaxObjectSelectorPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_bounds,
        display_id,
        source_view,
        mut snapshot,
    } = request;

    snapshot.width = parent_bounds.size.width.as_f32();
    let layout = menu_syntax_object_selector_popup_layout_above(
        parent_bounds,
        display_bounds,
        &snapshot,
    );
    snapshot.visible_row_limit = layout.visible_row_limit;
    let bounds = layout.bounds;
    let storage = MENU_SYNTAX_OBJECT_SELECTOR_POPUP_WINDOW.get_or_init(|| Mutex::new(None));

    if let Ok(mut guard) = storage.lock() {
        if let Some(slot) = *guard {
            if slot.parent_window_handle == parent_window_handle {
                let update_result = slot.handle.update(cx, |popup, window, cx| {
                    popup.set_snapshot(snapshot.clone());
                    crate::windows::automation_surface_collector::upsert_menu_syntax_object_selector_popup_snapshot(
                        MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
                        &snapshot.snapshot,
                        snapshot.selected_row_id.as_deref(),
                    );
                    set_inline_popup_window_bounds(window, bounds, cx);
                    crate::windows::set_automation_bounds(
                        MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
                        Some(crate::protocol::AutomationWindowBounds {
                            x: f32::from(bounds.origin.x) as f64,
                            y: f32::from(bounds.origin.y) as f64,
                            width: f32::from(bounds.size.width) as f64,
                            height: f32::from(bounds.size.height) as f64,
                        }),
                    );
                    cx.notify();
                });
                if update_result.is_ok() {
                    return Ok(());
                }
            }
            let _ = slot.handle.update(cx, |_popup, window, _cx| {
                window.remove_window();
            });
            crate::windows::remove_automation_window(
                MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
            );
            crate::windows::automation_surface_collector::remove_menu_syntax_object_selector_popup_snapshot(
                MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
            );
            *guard = None;
        }
    }

    let window_options = inline_popup_window_options(bounds, display_id);
    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| {
            MenuSyntaxObjectSelectorPopupWindow::new(
                snapshot.clone(),
                source_view.clone(),
                parent_window_handle,
                cx,
            )
        })
    })?;

    if let Err(error) = configure_inline_popup_window(&handle, cx, parent_window_handle) {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to configure menu-syntax object selector popup window"));
    }

    let parent_automation_id =
        crate::windows::focused_automation_window_id().unwrap_or_else(|| "main".to_string());
    if let Err(error) = crate::windows::register_attached_popup(
        MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID.to_string(),
        crate::protocol::AutomationWindowKind::PromptPopup,
        Some("Menu Syntax Object Selector".to_string()),
        Some("menuSyntaxObjectSelectorPopup".to_string()),
        Some(crate::protocol::AutomationWindowBounds {
            x: f32::from(bounds.origin.x) as f64,
            y: f32::from(bounds.origin.y) as f64,
            width: f32::from(bounds.size.width) as f64,
            height: f32::from(bounds.size.height) as f64,
        }),
        Some(parent_automation_id.as_str()),
    ) {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to register menu-syntax object selector popup window"));
    }
    crate::windows::automation_surface_collector::upsert_menu_syntax_object_selector_popup_snapshot(
        MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
        &snapshot.snapshot,
        snapshot.selected_row_id.as_deref(),
    );

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(MenuSyntaxObjectSelectorPopupSlot {
            handle,
            parent_window_handle,
        });
    }
    Ok(())
}

pub(crate) struct MenuSyntaxObjectSelectorPopupWindow {
    snapshot: MenuSyntaxObjectSelectorPopupSnapshot,
    source_view: WeakEntity<ScriptListApp>,
    focus_handle: FocusHandle,
    mouse_armed_row: Option<(usize, String)>,
}

impl MenuSyntaxObjectSelectorPopupWindow {
    fn new(
        snapshot: MenuSyntaxObjectSelectorPopupSnapshot,
        source_view: WeakEntity<ScriptListApp>,
        _parent_window_handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
            mouse_armed_row: None,
        }
    }

    fn set_snapshot(&mut self, mut snapshot: MenuSyntaxObjectSelectorPopupSnapshot) {
        snapshot.visible_start = self.visible_range().start;
        if let Some((armed_index, armed_id)) = self.mouse_armed_row.as_ref() {
            let still_same_row = snapshot
                .snapshot
                .rows
                .get(*armed_index)
                .is_some_and(|row| row.id.as_str() == armed_id.as_str());
            if !still_same_row {
                self.mouse_armed_row = None;
            }
        }
        self.snapshot = snapshot;
    }

    fn visible_range(&self) -> std::ops::Range<usize> {
        let row_count = self.snapshot.snapshot.rows.len();
        if row_count == 0 {
            return 0..0;
        }
        let selected_index = self
            .selected_index()
            .unwrap_or_else(|| self.snapshot.visible_start.min(row_count.saturating_sub(1)));
        inline_dropdown_visible_range_from_start(
            self.snapshot.visible_start,
            selected_index,
            row_count,
            self.snapshot
                .visible_row_limit
                .min(INLINE_POPUP_MAX_VISIBLE_ROWS),
        )
    }

    fn selected_index(&self) -> Option<usize> {
        self.snapshot.selected_index()
    }

    fn select_row(&mut self, idx: usize, cx: &mut Context<Self>) {
        let Some(row) = self.snapshot.snapshot.rows.get(idx) else {
            return;
        };
        self.snapshot.selected_row_id = Some(row.id.clone());
        if let Some(source) = self.source_view.upgrade() {
            source.update(cx, |app, _cx| {
                app.set_menu_syntax_object_selector_selection(row.id.clone());
            });
        }
        cx.notify();
    }

    fn accept_row(&mut self, idx: usize, cx: &mut Context<Self>) -> bool {
        let Some(row_id) = self
            .snapshot
            .snapshot
            .rows
            .get(idx)
            .map(|row| row.id.clone())
        else {
            return false;
        };
        let Some(source) = self.source_view.upgrade() else {
            return false;
        };
        source.update(cx, |app, cx| {
            app.accept_menu_syntax_object_selector_row(&row_id, None, cx)
        })
    }

    fn handle_row_click(
        &mut self,
        idx: usize,
        event: &gpui::ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self.snapshot.snapshot.rows.get(idx) else {
            return;
        };
        let row_id = row.id.clone();
        let was_mouse_armed = self
            .mouse_armed_row
            .as_ref()
            .is_some_and(|(armed_idx, armed_id)| *armed_idx == idx && *armed_id == row_id);
        let should_accept = should_submit_menu_syntax_object_selector_row_click(
            was_mouse_armed,
            event.click_count(),
        );
        if should_accept {
            self.mouse_armed_row = None;
            if self.accept_row(idx, cx) {
                return;
            }
            clear_menu_syntax_object_selector_popup_slot();
            crate::windows::automation_surface_collector::remove_menu_syntax_object_selector_popup_snapshot(
                MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
            );
            crate::windows::remove_automation_window(
                MENU_SYNTAX_OBJECT_SELECTOR_POPUP_AUTOMATION_ID,
            );
            window.remove_window();
        } else {
            self.mouse_armed_row = Some((idx, row_id));
            self.select_row(idx, cx);
        }
    }

    fn render_picker_row(
        &self,
        idx: usize,
        row: &ObjectSelectorRow,
        is_selected: bool,
        colors: InlineDropdownColors,
    ) -> gpui::Stateful<gpui::Div> {
        let label = SharedString::from(row.title.clone());
        let meta = row.token.clone().or_else(|| row.subtitle.clone()).map(SharedString::from);
        let label_hits = std::collections::HashSet::new();

        let mut left_side = div()
            .flex()
            .items_center();

        let label_spans = render_trigger_row_label(&label, &label_hits, if is_selected { colors.foreground } else { colors.foreground.opacity(MUTED_OP) }, colors.accent, is_selected);
        left_side = left_side.child(label_spans);

        let mut content = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(left_side);

        if let Some(meta_val) = meta.filter(|val| !val.is_empty()) {
            let meta_hits = std::collections::HashSet::new();
            content = content.child(
                div()
                    .px(gpui::px(6.0))
                    .py(gpui::px(2.0))
                    .rounded(gpui::px(4.0))
                    .bg(colors.foreground.opacity(0.06))
                    .child(render_trigger_row_meta_text(&meta_val, &meta_hits, if is_selected { colors.foreground.opacity(MUTED_OP) } else { colors.muted_foreground.opacity(0.45) }, colors.accent.opacity(0.45)))
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
            .id(SharedString::from(format!("menu-syntax-object-selector-popup-row-{idx}")))
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
        let selected_index = self.snapshot.selected_index();
        let visible_rows = self
            .snapshot
            .snapshot
            .rows
            .iter()
            .enumerate()
            .skip(visible.start)
            .take(visible.len())
            .collect::<Vec<_>>();

        let scrollbar = Scrollbar::new(
            self.snapshot.snapshot.rows.len(),
            visible.len(),
            visible.start,
            ScrollbarColors::from_theme(&theme),
        )
        .container_height(popup_height(&self.snapshot));

        let body = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .children(visible_rows.into_iter().map(|(idx, row)| {
                        let is_selected = selected_index == Some(idx);
                        let source_view = self.source_view.clone();
                        let enabled = row.enabled;
                        self.render_picker_row(idx, row, is_selected, colors)
                            .when(enabled, |row| row.cursor_pointer())
                            .when(!enabled, |row| row.opacity(0.55).cursor_default())
                            .on_click(cx.listener(move |this, event, window, cx| {
                                if source_view.upgrade().is_none() {
                                    close_menu_syntax_object_selector_popup_window(cx);
                                    return;
                                }
                                if !enabled {
                                    return;
                                }
                                this.handle_row_click(idx, event, window, cx);
                            }))
                            .into_any_element()
                    }))
            )
            .child(scrollbar)
            .into_any_element();

        InlineDropdown::new(
            SharedString::from("menu-syntax-object-selector-popup"),
            body,
            colors,
        )
        .vertical_padding(INLINE_POPUP_VERTICAL_PADDING / 2.0)
        .into_any_element()
    }
}

#[inline]
fn should_submit_menu_syntax_object_selector_row_click(
    was_mouse_armed: bool,
    click_count: usize,
) -> bool {
    let _ = (was_mouse_armed, click_count);
    true
}

impl Focusable for MenuSyntaxObjectSelectorPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MenuSyntaxObjectSelectorPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .child(self.render_picker(cx))
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

impl ScriptListApp {
    pub(crate) fn sync_menu_syntax_object_selector_popup_window(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(snapshot) = self
            .menu_syntax_object_selector_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            close_menu_syntax_object_selector_popup_window(cx);
            return;
        };

        let parent_bounds = window.bounds();
        let parent_window_handle = window.window_handle();
        let display = window.display(cx);
        let display_id = display.as_ref().map(|display| display.id());
        let display_bounds = display.as_ref().map(|display| display.visible_bounds());
        let width = parent_bounds.size.width.as_f32();
        let popup_snapshot = MenuSyntaxObjectSelectorPopupSnapshot {
            snapshot,
            selected_row_id: self
                .menu_syntax_object_selector_state
                .selected_row_id
                .clone(),
            visible_start: self.menu_syntax_object_selector_state.visible_start,
            visible_row_limit: INLINE_POPUP_MAX_VISIBLE_ROWS,
            width,
        };

        let request = MenuSyntaxObjectSelectorPopupRequest {
            parent_window_handle,
            parent_bounds,
            display_bounds,
            display_id,
            source_view: cx.entity().downgrade(),
            snapshot: popup_snapshot,
        };

        if let Err(error) = sync_menu_syntax_object_selector_popup_window(cx, request) {
            tracing::warn!(
                target: "script_kit::menu_syntax_object_selector",
                error = %error,
                "menu_syntax_object_selector_popup_window_sync_failed",
            );
        }
    }

    pub(crate) fn set_menu_syntax_object_selector_selection(&mut self, row_id: String) {
        self.menu_syntax_object_selector_state.selected_row_id = Some(row_id);
    }

    pub(crate) fn accept_menu_syntax_object_selector_row(
        &mut self,
        row_id: &str,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self
            .menu_syntax_object_selector_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            return false;
        };
        let Some(selected_index) = snapshot.rows.iter().position(|row| row.id == row_id) else {
            return false;
        };
        let raw_filter_text = self.filter_text.clone();
        let outcome = crate::menu_syntax::apply_object_selector_intent(
            crate::menu_syntax::InlinePickerKeyIntent::Accept,
            &snapshot,
            Some(selected_index),
            &raw_filter_text,
        );
        self.dispatch_menu_syntax_object_selector_outcome(outcome, window, cx);
        true
    }

    fn dispatch_menu_syntax_object_selector_outcome(
        &mut self,
        outcome: crate::menu_syntax::ObjectSelectorIntentOutcome,
        _window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        match outcome {
            crate::menu_syntax::ObjectSelectorIntentOutcome::Ignored
            | crate::menu_syntax::ObjectSelectorIntentOutcome::SelectionChanged { .. } => {}
            crate::menu_syntax::ObjectSelectorIntentOutcome::ReplaceInput { text } => {
                self.filter_text = text.clone();
                self.pending_filter_sync = true;
                self.computed_filter_text = text.clone();
                self.set_menu_syntax_mode_from_filter(&text);
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_replace",
                    cx,
                );
                self.menu_syntax_object_selector_state = Default::default();
                close_menu_syntax_object_selector_popup_window(cx);
                cx.notify();
            }
            crate::menu_syntax::ObjectSelectorIntentOutcome::Close => {
                self.menu_syntax_object_selector_state = Default::default();
                close_menu_syntax_object_selector_popup_window(cx);
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_close",
                    cx,
                );
                cx.notify();
            }
        }
    }

    pub(crate) fn run_menu_syntax_object_selector_state_machine(
        &mut self,
        raw_filter: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu_syntax_form_input_active && self.menu_syntax_capture_form_owns_input() {
            self.menu_syntax_object_selector_state = Default::default();
            close_menu_syntax_object_selector_popup_window(cx);
            return;
        }
        let capture_targets =
            crate::menu_syntax::registered_capture_targets_from_scripts(&self.scripts);
        let ctx = crate::menu_syntax::ObjectSelectorContext {
            candidates: self.menu_syntax_object_candidates_for_filter(raw_filter),
        };
        let transition = crate::menu_syntax::plan_object_selector_transition(
            &self.menu_syntax_object_selector_state,
            raw_filter,
            &capture_targets,
            &ctx,
        );
        match transition {
            crate::menu_syntax::ObjectSelectorTransition::NoChange => {}
            crate::menu_syntax::ObjectSelectorTransition::Close => {
                self.menu_syntax_object_selector_state = Default::default();
                close_menu_syntax_object_selector_popup_window(cx);
            }
            crate::menu_syntax::ObjectSelectorTransition::Open {
                snapshot,
                selected_row_id,
            } => {
                self.menu_syntax_object_selector_state =
                    crate::menu_syntax::MenuSyntaxObjectSelectorState {
                        snapshot: Some(snapshot),
                        selected_row_id,
                        visible_start: 0,
                    };
                crate::menu_syntax_trigger_popup_window::close_menu_syntax_trigger_popup_window(cx);
                self.menu_syntax_trigger_popup_state = Default::default();
                self.sync_menu_syntax_object_selector_popup_window(window, cx);
            }
            crate::menu_syntax::ObjectSelectorTransition::Update {
                snapshot,
                selected_row_id,
            } => {
                let selected_index = selected_row_id
                    .as_deref()
                    .and_then(|id| snapshot.rows.iter().position(|row| row.id == id))
                    .unwrap_or(0);
                let visible_start = crate::menu_syntax::object_selector_visible_start_for_selection(
                    self.menu_syntax_object_selector_state.visible_start,
                    selected_index,
                    snapshot.rows.len(),
                );
                self.menu_syntax_object_selector_state =
                    crate::menu_syntax::MenuSyntaxObjectSelectorState {
                        snapshot: Some(snapshot),
                        selected_row_id,
                        visible_start,
                    };
                crate::menu_syntax_trigger_popup_window::close_menu_syntax_trigger_popup_window(cx);
                self.menu_syntax_trigger_popup_state = Default::default();
                self.sync_menu_syntax_object_selector_popup_window(window, cx);
            }
        }
    }

    pub(crate) fn menu_syntax_object_candidates_for_filter(
        &self,
        raw_filter: &str,
    ) -> Vec<crate::menu_syntax::ObjectSelectorCandidate> {
        let capture_targets =
            crate::menu_syntax::registered_capture_targets_from_scripts(&self.scripts);
        let Some(selector) = crate::menu_syntax::capture::active_object_selector_for_input(
            raw_filter,
            &capture_targets,
        ) else {
            return Vec::new();
        };
        let query = selector.query.trim();
        match selector.kind {
            crate::menu_syntax::CaptureObjectKind::Note => {
                crate::notes::search_root_notes_meta_direct(
                    query,
                    crate::notes::RootNotesSectionOptions {
                        enabled: true,
                        max_results: 10,
                        min_query_chars: 0,
                        search_content: true,
                    },
                )
                .into_iter()
                .map(|hit| crate::menu_syntax::ObjectSelectorCandidate {
                    kind: crate::menu_syntax::CaptureObjectKind::Note,
                    id: hit.id.to_string(),
                    label: if hit.title.trim().is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        hit.title
                    },
                    subtitle: format!(
                        "Updated {} · {} chars",
                        crate::formatting::format_relative_time_short_dt(hit.updated_at),
                        hit.char_count
                    ),
                })
                .collect()
            }
            kind => crate::menu_syntax::search_root_object_candidates_direct(kind, query, 10),
        }
    }

    pub(crate) fn apply_menu_syntax_object_selector_intent(
        &mut self,
        intent: crate::menu_syntax::InlinePickerKeyIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self
            .menu_syntax_object_selector_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            return false;
        };
        let selected_index = self
            .menu_syntax_object_selector_state
            .selected_row_id
            .as_deref()
            .and_then(|id| snapshot.rows.iter().position(|row| row.id == id));
        let raw_filter_text = self.filter_text.clone();
        let outcome = crate::menu_syntax::apply_object_selector_intent(
            intent,
            &snapshot,
            selected_index,
            &raw_filter_text,
        );
        match outcome {
            crate::menu_syntax::ObjectSelectorIntentOutcome::SelectionChanged { new_index } => {
                let next_row_id = snapshot.rows.get(new_index).map(|row| row.id.clone());
                self.menu_syntax_object_selector_state.visible_start =
                    crate::menu_syntax::object_selector_visible_start_for_selection(
                        self.menu_syntax_object_selector_state.visible_start,
                        new_index,
                        snapshot.rows.len(),
                    );
                self.menu_syntax_object_selector_state.selected_row_id = next_row_id;
                self.sync_menu_syntax_object_selector_popup_window(window, cx);
                cx.notify();
                true
            }
            crate::menu_syntax::ObjectSelectorIntentOutcome::Ignored => false,
            other => {
                self.dispatch_menu_syntax_object_selector_outcome(other, Some(window), cx);
                true
            }
        }
    }
}
