//! Menu-syntax trigger popup — GPUI window entity + singleton slot + sync
//! function.
//!
//! Binary-only half of the Oracle iter 015 popup pivot. The pure state
//! machine + row adapter lives in
//! `src/app_impl/menu_syntax_trigger_popup.rs` (re-exported as
//! `crate::menu_syntax_trigger_popup` in the lib crate for testing). This
//! file wires that pure core to GPUI: it owns the popup NSWindow entity,
//! the singleton slot, the sync/close helpers, and the row-click handler
//! that dispatches back to `ScriptListApp`.
//!
//! This module is not re-exported by the lib crate. `ScriptListApp` is
//! binary-only (defined via `include!` in `src/main.rs`) so any code that
//! holds a `WeakEntity<ScriptListApp>` can only live in the binary target.
//!
//! Mirrors `src/ai/acp/picker_popup.rs::AcpMentionPopupWindow` line-by-line
//! so the menu-syntax popup feels identical to the ACP `@` / `/` pickers
//! the user already knows: shared components, consistent behavior.

use std::io;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use gpui::{
    div, prelude::FluentBuilder, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::components::inline_dropdown::{
    inline_dropdown_visible_range_from_start, render_soft_compact_picker_row, InlineDropdown,
    InlineDropdownColors, InlineDropdownSynopsis, CONTEXT_PICKER_SYNOPSIS_HEIGHT,
    SOFT_COMPACT_PICKER_ROW_HEIGHT,
};
use crate::components::inline_popup_window::{
    configure_inline_popup_window, inline_popup_height_for_row_height,
    inline_popup_width_for_window, inline_popup_window_options, set_inline_popup_window_bounds,
};
use crate::components::inline_popup_window::{
    INLINE_POPUP_MAX_VISIBLE_ROWS, INLINE_POPUP_VERTICAL_PADDING,
};
use crate::menu_syntax::{TriggerPickerRow, TriggerPickerRowKind, TriggerPickerSnapshot};
use crate::menu_syntax_trigger_popup::{
    adapt_trigger_picker_row, trigger_popup_row_highlight_indices,
};
use crate::ScriptListApp;

struct AppCaptureHandlerScaffoldEffects<'a> {
    config: &'a crate::config::Config,
}

impl crate::menu_syntax::CaptureHandlerScaffoldEffects for AppCaptureHandlerScaffoldEffects<'_> {
    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn write_file(&self, path: &Path, contents: &str) -> io::Result<()> {
        std::fs::write(path, contents)
    }

    fn open_in_editor(&self, path: &Path) -> io::Result<()> {
        crate::script_creation::open_in_editor(path, self.config)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
            .or_else(|_| {
                let _child = std::process::Command::new("open").arg(path).spawn()?;
                Ok(())
            })
    }
}

const MENU_SYNTAX_TRIGGER_POPUP_MAX_PARENT_HEIGHT_RATIO: f32 = 0.90;

/// Snapshot handed to the GPUI window entity. Clone-cheap (rows are
/// `Vec<TriggerPickerRow>` which is what `build_trigger_picker_snapshot`
/// already produces).
#[derive(Clone)]
pub(crate) struct MenuSyntaxTriggerPopupSnapshot {
    pub(crate) snapshot: TriggerPickerSnapshot,
    pub(crate) selected_row_id: Option<String>,
    pub(crate) raw_filter_text: String,
    pub(crate) visible_start: usize,
    pub(crate) visible_row_limit: usize,
    pub(crate) width: f32,
}

impl MenuSyntaxTriggerPopupSnapshot {
    fn selected_index(&self) -> Option<usize> {
        self.selected_row_id
            .as_deref()
            .and_then(|id| self.snapshot.rows.iter().position(|row| row.id == id))
    }

    fn selectable_rows(&self) -> impl Iterator<Item = (usize, &TriggerPickerRow)> {
        self.snapshot.rows.iter().enumerate()
    }
}

/// Request payload for [`sync_menu_syntax_trigger_popup_window`]. Built by
/// the caller from the main window's bounds + the popup state machine.
#[derive(Clone)]
pub(crate) struct MenuSyntaxTriggerPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<ScriptListApp>,
    pub(crate) snapshot: MenuSyntaxTriggerPopupSnapshot,
    pub(crate) left: f32,
    pub(crate) top: f32,
}

#[derive(Clone, Copy)]
struct MenuSyntaxTriggerPopupSlot {
    handle: WindowHandle<MenuSyntaxTriggerPopupWindow>,
    parent_window_handle: AnyWindowHandle,
}

static MENU_SYNTAX_TRIGGER_POPUP_WINDOW: OnceLock<Mutex<Option<MenuSyntaxTriggerPopupSlot>>> =
    OnceLock::new();

const MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID: &str = "menu-syntax-trigger-popup";

fn is_trigger_popup_footer_row(row: &TriggerPickerRow) -> bool {
    row.kind == TriggerPickerRowKind::FooterAction
}

fn trigger_popup_footer_count(snapshot: &MenuSyntaxTriggerPopupSnapshot) -> usize {
    snapshot
        .snapshot
        .rows
        .iter()
        .filter(|row| is_trigger_popup_footer_row(row))
        .count()
}

fn trigger_popup_normal_row_capacity(snapshot: &MenuSyntaxTriggerPopupSnapshot) -> usize {
    snapshot
        .visible_row_limit
        .min(INLINE_POPUP_MAX_VISIBLE_ROWS)
        .saturating_sub(trigger_popup_footer_count(snapshot))
        .max(1)
}

fn trigger_popup_visible_row_count(snapshot: &MenuSyntaxTriggerPopupSnapshot) -> usize {
    let footer_count = trigger_popup_footer_count(snapshot);
    let normal_count = snapshot.snapshot.rows.len().saturating_sub(footer_count);
    normal_count.min(trigger_popup_normal_row_capacity(snapshot)) + footer_count
}

fn clear_menu_syntax_trigger_popup_slot() {
    if let Some(storage) = MENU_SYNTAX_TRIGGER_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

/// Close the popup NSWindow if it is open and clear the singleton slot.
/// Idempotent — safe to call when nothing is open.
pub(crate) fn close_menu_syntax_trigger_popup_window(cx: &mut App) {
    crate::windows::automation_surface_collector::remove_menu_syntax_prompt_popup_snapshot(
        MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
    );
    crate::windows::remove_automation_window(MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID);
    if let Some(storage) = MENU_SYNTAX_TRIGGER_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

/// Whether a menu-syntax trigger popup window is currently open.
/// Used by keyboard-dispatch sites (commit D2c) to decide whether
/// Arrow/Tab/Enter/Escape should route to the popup.
pub(crate) fn is_menu_syntax_trigger_popup_window_open() -> bool {
    if let Some(storage) = MENU_SYNTAX_TRIGGER_POPUP_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            return guard.is_some();
        }
    }
    false
}

fn selected_row_has_synopsis(snapshot: &MenuSyntaxTriggerPopupSnapshot) -> bool {
    trigger_popup_footer_count(snapshot) == 0
        && snapshot
            .selected_index()
            .and_then(|idx| snapshot.snapshot.rows.get(idx))
            .is_some_and(|row| row.detail.is_some() || row.example.is_some())
}

fn popup_height(snapshot: &MenuSyntaxTriggerPopupSnapshot) -> f32 {
    if snapshot.snapshot.rows.is_empty() {
        return inline_popup_height_for_row_height(0, SOFT_COMPACT_PICKER_ROW_HEIGHT);
    }

    let row_count = trigger_popup_visible_row_count(snapshot);
    let row_height = inline_popup_height_for_row_height(row_count, SOFT_COMPACT_PICKER_ROW_HEIGHT);

    row_height
        + if selected_row_has_synopsis(snapshot) {
            CONTEXT_PICKER_SYNOPSIS_HEIGHT
        } else {
            0.0
        }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MenuSyntaxTriggerPopupLayout {
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) visible_row_limit: usize,
}

fn popup_visible_row_limit(snapshot: &MenuSyntaxTriggerPopupSnapshot, parent_height: f32) -> usize {
    let row_count = snapshot.snapshot.rows.len();
    if row_count == 0 {
        return 0;
    }

    let max_height = (parent_height * MENU_SYNTAX_TRIGGER_POPUP_MAX_PARENT_HEIGHT_RATIO).max(1.0);
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

pub(crate) fn menu_syntax_trigger_popup_layout_left_drawer(
    parent_bounds: Bounds<Pixels>,
    snapshot: &MenuSyntaxTriggerPopupSnapshot,
) -> MenuSyntaxTriggerPopupLayout {
    let mut snapshot = snapshot.clone();
    let visible_row_limit =
        popup_visible_row_limit(&snapshot, f32::from(parent_bounds.size.height));
    snapshot.visible_row_limit = visible_row_limit;
    MenuSyntaxTriggerPopupLayout {
        bounds: Bounds {
            origin: gpui::point(
                parent_bounds.origin.x - gpui::px(snapshot.width),
                parent_bounds.origin.y,
            ),
            size: gpui::size(gpui::px(snapshot.width), gpui::px(popup_height(&snapshot))),
        },
        visible_row_limit,
    }
}

/// Open or update the menu-syntax trigger popup window.
pub(crate) fn sync_menu_syntax_trigger_popup_window(
    cx: &mut App,
    request: MenuSyntaxTriggerPopupRequest,
) -> anyhow::Result<()> {
    let MenuSyntaxTriggerPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_id,
        source_view,
        mut snapshot,
        left: _left,
        top: _top,
    } = request;

    let layout = menu_syntax_trigger_popup_layout_left_drawer(parent_bounds, &snapshot);
    snapshot.visible_row_limit = layout.visible_row_limit;
    let bounds = layout.bounds;

    let storage = MENU_SYNTAX_TRIGGER_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        if let Some(slot) = *guard {
            if slot.parent_window_handle == parent_window_handle {
                let update_result = slot.handle.update(cx, |popup, window, cx| {
                    popup.set_snapshot(snapshot.clone());
                    crate::windows::automation_surface_collector::upsert_menu_syntax_prompt_popup_snapshot(
                        MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
                        &snapshot.snapshot,
                        snapshot.selected_row_id.as_deref(),
                    );
                    set_inline_popup_window_bounds(window, bounds, cx);
                    crate::windows::set_automation_bounds(
                        MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
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

                crate::windows::remove_automation_window(MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID);
                crate::windows::automation_surface_collector::remove_menu_syntax_prompt_popup_snapshot(
                    MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
                );
                *guard = None;
            } else {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
                crate::windows::remove_automation_window(MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID);
                crate::windows::automation_surface_collector::remove_menu_syntax_prompt_popup_snapshot(
                    MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
                );
                *guard = None;
            }
        }
    }

    let window_options = inline_popup_window_options(bounds, display_id);

    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| MenuSyntaxTriggerPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
    })?;

    if let Err(error) = configure_inline_popup_window(&handle, cx, parent_window_handle) {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to configure menu-syntax trigger popup window"));
    }

    let parent_automation_id =
        crate::windows::focused_automation_window_id().unwrap_or_else(|| "main".to_string());
    if let Err(error) = crate::windows::register_attached_popup(
        MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID.to_string(),
        crate::protocol::AutomationWindowKind::PromptPopup,
        Some("Menu Syntax".to_string()),
        Some("menuSyntaxTriggerPopup".to_string()),
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
        return Err(error.context("failed to register menu-syntax trigger popup window"));
    }
    crate::windows::automation_surface_collector::upsert_menu_syntax_prompt_popup_snapshot(
        MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
        &snapshot.snapshot,
        snapshot.selected_row_id.as_deref(),
    );

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(MenuSyntaxTriggerPopupSlot {
            handle,
            parent_window_handle,
        });
    }

    Ok(())
}

/// GPUI window entity backing the menu-syntax trigger popup.
pub(crate) struct MenuSyntaxTriggerPopupWindow {
    snapshot: MenuSyntaxTriggerPopupSnapshot,
    source_view: WeakEntity<ScriptListApp>,
    focus_handle: FocusHandle,
    mouse_armed_row: Option<(usize, String)>,
}

impl MenuSyntaxTriggerPopupWindow {
    fn new(
        snapshot: MenuSyntaxTriggerPopupSnapshot,
        source_view: WeakEntity<ScriptListApp>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
            mouse_armed_row: None,
        }
    }

    fn set_snapshot(&mut self, mut snapshot: MenuSyntaxTriggerPopupSnapshot) {
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
        let normal_count = self
            .snapshot
            .snapshot
            .rows
            .iter()
            .filter(|row| !is_trigger_popup_footer_row(row))
            .count();
        if normal_count == 0 {
            return 0..0;
        }
        let selected_index = self.selected_normal_index().unwrap_or_else(|| {
            self.snapshot
                .visible_start
                .min(normal_count.saturating_sub(1))
        });
        inline_dropdown_visible_range_from_start(
            self.snapshot.visible_start,
            selected_index,
            normal_count,
            trigger_popup_normal_row_capacity(&self.snapshot),
        )
    }

    fn selected_normal_index(&self) -> Option<usize> {
        let selected_id = self.snapshot.selected_row_id.as_deref()?;
        let mut normal_index = 0usize;
        for row in &self.snapshot.snapshot.rows {
            if is_trigger_popup_footer_row(row) {
                continue;
            }
            if row.id == selected_id {
                return Some(normal_index);
            }
            normal_index += 1;
        }
        None
    }

    fn accept_row(&self, row_index: usize, cx: &mut App) {
        let Some(row) = self.snapshot.snapshot.rows.get(row_index) else {
            return;
        };
        let row_id = row.id.clone();

        if let Some(view) = self.source_view.upgrade() {
            view.update(cx, |app, cx| {
                app.accept_menu_syntax_trigger_popup_row(&row_id, cx);
            });
        } else {
            close_menu_syntax_trigger_popup_window(cx);
        }
    }

    fn select_row(&mut self, row_index: usize, cx: &mut Context<Self>) {
        let Some(row) = self.snapshot.snapshot.rows.get(row_index) else {
            return;
        };
        let row_id = row.id.clone();
        self.snapshot.selected_row_id = Some(row_id.clone());

        if let Some(view) = self.source_view.upgrade() {
            view.update(cx, |app, _cx| {
                app.set_menu_syntax_trigger_popup_selection(row_id);
            });
        }
        cx.notify();
    }

    fn handle_row_click(
        &mut self,
        index: usize,
        event: &gpui::ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self.snapshot.snapshot.rows.get(index) else {
            return;
        };
        if !row.enabled {
            return;
        }
        let row_id = row.id.clone();
        let was_mouse_armed = self
            .mouse_armed_row
            .as_ref()
            .is_some_and(|(armed_index, armed_id)| *armed_index == index && armed_id == &row_id);
        let click_count = event.click_count();
        let should_accept =
            should_submit_menu_syntax_picker_row_click(was_mouse_armed, click_count);

        tracing::info!(
            target: "script_kit::menu_syntax_popup",
            event = "menu_syntax_trigger_popup_row_click",
            row_index = index,
            row_id = %row_id,
            click_count,
            was_mouse_armed,
            should_accept,
        );

        if should_accept {
            self.mouse_armed_row = None;
            self.accept_row(index, cx);
            clear_menu_syntax_trigger_popup_slot();
            crate::windows::automation_surface_collector::remove_menu_syntax_prompt_popup_snapshot(
                MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID,
            );
            crate::windows::remove_automation_window(MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID);
            window.remove_window();
        } else {
            self.mouse_armed_row = Some((index, row_id));
            self.select_row(index, cx);
        }
    }

    fn render_picker_row(
        &self,
        idx: usize,
        row: &TriggerPickerRow,
        is_selected: bool,
        colors: InlineDropdownColors,
    ) -> gpui::Stateful<gpui::Div> {
        let neutral = adapt_trigger_picker_row(row);
        let label = neutral.title.clone();
        let meta = neutral.token.clone().or_else(|| neutral.subtitle.clone());
        let highlights = trigger_popup_row_highlight_indices(row, &self.snapshot.raw_filter_text);
        render_soft_compact_picker_row(
            SharedString::from(format!("menu-syntax-trigger-popup-row-{idx}")),
            label,
            meta,
            &highlights.title,
            &highlights.meta,
            is_selected,
            colors,
        )
    }

    fn render_picker(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let visible = self.visible_range();
        let selected_index = self.snapshot.selected_index();
        let selected_row =
            selected_index.and_then(|idx| self.snapshot.snapshot.rows.get(idx).cloned());
        let normal_rows: Vec<_> = self
            .snapshot
            .selectable_rows()
            .filter(|(_, row)| !is_trigger_popup_footer_row(row))
            .collect();
        let footer_rows: Vec<_> = self
            .snapshot
            .selectable_rows()
            .filter(|(_, row)| is_trigger_popup_footer_row(row))
            .collect();
        let visible_rows: Vec<_> = normal_rows
            .iter()
            .skip(visible.start)
            .take(visible.len())
            .copied()
            .chain(footer_rows.iter().copied())
            .collect();

        let body = div()
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
                            close_menu_syntax_trigger_popup_window(cx);
                            return;
                        }
                        if !enabled {
                            return;
                        }
                        this.handle_row_click(idx, event, window, cx);
                    }))
                    .into_any_element()
            }))
            .into_any_element();

        let synopsis = (footer_rows.is_empty()).then_some(()).and_then(|_| {
            let row = selected_row.as_ref()?;
            let detail: String = row
                .detail
                .clone()
                .or_else(|| row.example.clone())
                .unwrap_or_default();
            if detail.is_empty() {
                return None;
            }
            let token: String = row.token.clone().unwrap_or_default();
            Some(InlineDropdownSynopsis {
                label: SharedString::from(row.title.clone()),
                meta: SharedString::from(token),
                description: SharedString::from(detail),
            })
        });

        tracing::info!(
            target: "script_kit::menu_syntax_popup",
            event = "menu_syntax_trigger_popup_render",
            row_count = self.snapshot.snapshot.rows.len(),
            ?selected_index,
        );

        InlineDropdown::new(
            SharedString::from("menu-syntax-trigger-popup"),
            body,
            colors,
        )
        .synopsis(synopsis)
        .vertical_padding(INLINE_POPUP_VERTICAL_PADDING / 2.0)
        .into_any_element()
    }
}

#[inline]
fn should_submit_menu_syntax_picker_row_click(was_mouse_armed: bool, click_count: usize) -> bool {
    let _ = (was_mouse_armed, click_count);
    true
}

impl Focusable for MenuSyntaxTriggerPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MenuSyntaxTriggerPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .child(self.render_picker(cx))
    }
}

impl ScriptListApp {
    /// Build the popup request from the current ScriptListApp state and
    /// sync the GPUI window. Callers invoke this after
    /// `plan_trigger_popup_transition` returns `Open` or `Update`.
    pub(crate) fn sync_menu_syntax_trigger_popup_window(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sync_menu_syntax_trigger_popup_window_for_filter(self.filter_text.clone(), window, cx);
    }

    pub(crate) fn sync_menu_syntax_trigger_popup_window_for_filter(
        &mut self,
        raw_filter_text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(snapshot) = self
            .menu_syntax_trigger_popup_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            close_menu_syntax_trigger_popup_window(cx);
            return;
        };

        let parent_bounds = window.bounds();
        let parent_window_handle = window.window_handle();
        let display_id = window.display(cx).map(|display| display.id());
        let width = inline_popup_width_for_window(parent_bounds.size.width.as_f32());
        let popup_snapshot = MenuSyntaxTriggerPopupSnapshot {
            snapshot,
            selected_row_id: self.menu_syntax_trigger_popup_state.selected_row_id.clone(),
            raw_filter_text,
            visible_start: self.menu_syntax_trigger_popup_state.visible_start,
            visible_row_limit: INLINE_POPUP_MAX_VISIBLE_ROWS,
            width,
        };

        let request = MenuSyntaxTriggerPopupRequest {
            parent_window_handle,
            parent_bounds,
            display_id,
            source_view: cx.entity().downgrade(),
            snapshot: popup_snapshot,
            left: 0.0,
            top: 0.0,
        };

        if let Err(error) = sync_menu_syntax_trigger_popup_window(cx, request) {
            tracing::warn!(
                target: "script_kit::menu_syntax_popup",
                error = %error,
                "menu_syntax_trigger_popup_window_sync_failed",
            );
        }
    }

    /// Update the cached selected row id from a mouse-driven popup
    /// selection change. The popup renders from this state on the next
    /// sync.
    pub(crate) fn set_menu_syntax_trigger_popup_selection(&mut self, row_id: String) {
        self.menu_syntax_trigger_popup_state.selected_row_id = Some(row_id);
    }

    /// Apply the Accept outcome for a clicked popup row. Mouse-click path
    /// only — keyboard goes through
    /// [`apply_menu_syntax_trigger_popup_intent`], which has access to
    /// `&mut Window` and can therefore re-sync the popup after a
    /// `keep_open` apply. Mouse clicks always close the popup (the row
    /// action produces Accept, not Apply).
    pub(crate) fn accept_menu_syntax_trigger_popup_row(
        &mut self,
        row_id: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(snapshot) = self
            .menu_syntax_trigger_popup_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            return;
        };
        let Some(selected_index) = snapshot.rows.iter().position(|row| row.id == row_id) else {
            return;
        };
        let raw_filter_text = self.filter_text.clone();
        let outcome = crate::menu_syntax::apply_intent(
            crate::menu_syntax::InlinePickerKeyIntent::Accept,
            &snapshot,
            Some(selected_index),
            &raw_filter_text,
        );

        self.dispatch_menu_syntax_trigger_popup_outcome(outcome, None, cx);
    }

    fn dispatch_menu_syntax_trigger_popup_outcome(
        &mut self,
        outcome: crate::menu_syntax::TriggerPickerIntentOutcome,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        use crate::menu_syntax::TriggerPickerIntentOutcome;
        match outcome {
            TriggerPickerIntentOutcome::Ignored
            | TriggerPickerIntentOutcome::SelectionChanged { .. } => {}
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                // Stage the replacement — render() will reconcile the GPUI
                // InputState on the next frame (needs `&mut Window`). The
                // input history, fallback state, and grouped cache all key
                // off `computed_filter_text`, so updating it directly keeps
                // the main list in sync for the current frame.
                self.filter_text = text.clone();
                self.pending_filter_sync = true;
                self.computed_filter_text = text.clone();
                self.set_menu_syntax_mode_from_filter(&text);
                self.invalidate_grouped_cache();

                if keep_open {
                    // Re-run the popup state machine against the new filter
                    // so the popup shows a snapshot matching the replaced
                    // text (e.g. Tab on `;` -> replace filter with `;todo `
                    // -> popup should now show todo's capture-handler rows,
                    // not the bare target list).
                    if let Some(window) = window {
                        self.run_menu_syntax_trigger_popup_state_machine(&text, window, cx);
                    }
                } else {
                    self.menu_syntax_trigger_popup_state = Default::default();
                    close_menu_syntax_trigger_popup_window(cx);
                    // Mark this exact filter text as "user just accepted,
                    // do not re-open the popup". Without this, pressing
                    // Enter on `;` selects `;todo`, sets the filter to
                    // `;todo ` which parses to
                    // `Incomplete(MissingCaptureBody)`, and the next
                    // `handle_filter_input_change` re-runs
                    // `plan_trigger_popup_transition` -> `Open` with the
                    // handler snapshot - the popup flickers back open
                    // immediately after the user dismissed it. The
                    // suppression is cleared as soon as the filter text
                    // changes (user types a body character or deletes).
                    self.menu_syntax_trigger_popup_suppressed_filter = Some(text.clone());
                }
                cx.notify();
            }
            TriggerPickerIntentOutcome::Close => {
                self.menu_syntax_trigger_popup_state = Default::default();
                close_menu_syntax_trigger_popup_window(cx);
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_trigger_popup_close",
                    cx,
                );
                cx.notify();
            }
            TriggerPickerIntentOutcome::OpenCaptures { .. }
            | TriggerPickerIntentOutcome::OpenHelp => {
                // Deferred — these routes wire through in follow-up work.
                // For now, treat as a close so the popup dismisses instead
                // of lingering with a stale snapshot.
                self.menu_syntax_trigger_popup_state = Default::default();
                close_menu_syntax_trigger_popup_window(cx);
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_trigger_popup_close_deferred",
                    cx,
                );
                cx.notify();
            }
            TriggerPickerIntentOutcome::CreateHandler { target } => {
                if let Some(slug) = target {
                    let effects = AppCaptureHandlerScaffoldEffects {
                        config: &self.config,
                    };
                    let scripts_dir = crate::script_creation::scripts_dir();
                    match crate::menu_syntax::create_capture_handler_scaffold(
                        &effects,
                        &scripts_dir,
                        &slug,
                        true,
                    ) {
                        Ok(created) => {
                            self.filter_text.clear();
                            self.pending_filter_sync = true;
                            self.computed_filter_text.clear();
                            self.set_menu_syntax_mode_from_filter("");
                            self.invalidate_grouped_cache();
                            self.show_hud(
                                format!("Created {}", created.filename),
                                Some(crate::HUD_SHORT_MS),
                                cx,
                            );
                        }
                        Err(error) => {
                            tracing::warn!(
                                target: "script_kit::menu_syntax",
                                event = "create_capture_handler_failed",
                                slug = %slug,
                                error = %error,
                            );
                            self.show_error_toast(format!("Create handler failed: {error}"), cx);
                        }
                    }
                }
                self.menu_syntax_trigger_popup_state = Default::default();
                close_menu_syntax_trigger_popup_window(cx);
                cx.notify();
            }
            TriggerPickerIntentOutcome::AiScaffoldHandler {
                slug,
                nearest_targets,
            } => {
                let nearest = if nearest_targets.is_empty() {
                    "none".to_string()
                } else {
                    nearest_targets.join(", ")
                };
                let prompt = format!(
                    "You are scaffolding a new Script Kit capture handler.\n\nSlug: {slug}\nTyped by user: ;{slug}\nNearest existing targets: {nearest}\n\nGenerate a TypeScript handler that registers `capture.v1` with target \"{slug}\" and a sensible `label`. Output ONLY the TypeScript code."
                );
                self.menu_syntax_trigger_popup_state = Default::default();
                close_menu_syntax_trigger_popup_window(cx);
                self.open_tab_ai_acp_with_entry_intent_preserving_return(Some(prompt), cx);
                cx.notify();
            }
        }
    }

    /// Re-run the popup state machine against a (possibly new) filter text
    /// and dispatch the resulting transition to the GPUI window. Extracted
    /// here so both `apply_menu_syntax_trigger_popup_intent` (keyboard
    /// Tab-apply path) and `handle_filter_input_change` can share the
    /// state-machine invocation.
    pub(crate) fn run_menu_syntax_trigger_popup_state_machine(
        &mut self,
        raw_filter: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let picker_ctx = crate::menu_syntax::TriggerPickerContext {
            recent_queries: self.input_history.recent_entries(8),
            scripts: self.scripts.clone(),
            scriptlets: self.scriptlets.clone(),
        };
        let transition = crate::menu_syntax_trigger_popup::plan_trigger_popup_transition(
            &self.menu_syntax_trigger_popup_state,
            raw_filter,
            &picker_ctx,
        );
        use crate::menu_syntax_trigger_popup::TriggerPopupTransition;
        match transition {
            TriggerPopupTransition::NoChange => {}
            TriggerPopupTransition::Close => {
                self.menu_syntax_trigger_popup_state = Default::default();
                close_menu_syntax_trigger_popup_window(cx);
            }
            TriggerPopupTransition::Open {
                snapshot,
                selected_row_id,
            } => {
                self.menu_syntax_trigger_popup_state =
                    crate::menu_syntax_trigger_popup::MenuSyntaxTriggerPopupState {
                        snapshot: Some(snapshot),
                        selected_row_id,
                        visible_start: 0,
                    };
                self.sync_menu_syntax_trigger_popup_window_for_filter(
                    raw_filter.to_string(),
                    window,
                    cx,
                );
            }
            TriggerPopupTransition::Update {
                snapshot,
                selected_row_id,
            } => {
                let selected_index = selected_row_id
                    .as_deref()
                    .and_then(|id| snapshot.rows.iter().position(|row| row.id == id))
                    .unwrap_or(0);
                let visible_start =
                    crate::menu_syntax_trigger_popup::trigger_popup_visible_start_for_selection(
                        self.menu_syntax_trigger_popup_state.visible_start,
                        selected_index,
                        snapshot.rows.len(),
                    );
                self.menu_syntax_trigger_popup_state =
                    crate::menu_syntax_trigger_popup::MenuSyntaxTriggerPopupState {
                        snapshot: Some(snapshot),
                        selected_row_id,
                        visible_start,
                    };
                self.sync_menu_syntax_trigger_popup_window_for_filter(
                    raw_filter.to_string(),
                    window,
                    cx,
                );
            }
        }
    }

    /// Keyboard entry point for the menu-syntax trigger popup. Keyboard
    /// interceptors in `startup.rs` (arrow keys), `startup_new_tab.rs`
    /// (Tab / Enter), and `render_script_list/mod.rs` (Escape) call this
    /// when the popup is open. Returns `true` when the intent was consumed
    /// and the caller should NOT route the keystroke anywhere else.
    pub(crate) fn apply_menu_syntax_trigger_popup_intent(
        &mut self,
        intent: crate::menu_syntax::InlinePickerKeyIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self
            .menu_syntax_trigger_popup_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            return false;
        };

        let selected_index = self
            .menu_syntax_trigger_popup_state
            .selected_row_id
            .as_deref()
            .and_then(|id| snapshot.rows.iter().position(|row| row.id == id));

        let raw_filter_text = self.filter_text.clone();
        let outcome =
            crate::menu_syntax::apply_intent(intent, &snapshot, selected_index, &raw_filter_text);

        match outcome {
            crate::menu_syntax::TriggerPickerIntentOutcome::SelectionChanged { new_index } => {
                let next_row_id = snapshot.rows.get(new_index).map(|row| row.id.clone());
                self.menu_syntax_trigger_popup_state.visible_start =
                    crate::menu_syntax_trigger_popup::trigger_popup_visible_start_for_selection(
                        self.menu_syntax_trigger_popup_state.visible_start,
                        new_index,
                        snapshot.rows.len(),
                    );
                self.menu_syntax_trigger_popup_state.selected_row_id = next_row_id;
                // Re-sync so the popup re-renders with the new selection.
                self.sync_menu_syntax_trigger_popup_window(window, cx);
                cx.notify();
                true
            }
            crate::menu_syntax::TriggerPickerIntentOutcome::Ignored => false,
            other => {
                self.dispatch_menu_syntax_trigger_popup_outcome(other, Some(window), cx);
                true
            }
        }
    }
}
