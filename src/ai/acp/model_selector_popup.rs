use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, svg, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle,
    Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::ai::context_picker_row::{render_dense_monoline_picker_row_with_accessory, GOLD};
use crate::components::inline_dropdown::{InlineDropdown, InlineDropdownColors};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{IconName, IconNamed};

use super::view::AcpChatView;

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupEntry {
    pub(crate) id: String,
    pub(crate) display: SharedString,
    pub(crate) is_active: bool,
}

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupSnapshot {
    pub(crate) selected_index: usize,
    pub(crate) entries: Vec<AcpModelSelectorPopupEntry>,
}

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<AcpChatView>,
    pub(crate) snapshot: AcpModelSelectorPopupSnapshot,
}

#[derive(Clone, Copy)]
struct AcpModelSelectorPopupSlot {
    handle: WindowHandle<AcpModelSelectorPopupWindow>,
    parent_window_handle: AnyWindowHandle,
}

static ACP_MODEL_SELECTOR_POPUP_WINDOW: OnceLock<Mutex<Option<AcpModelSelectorPopupSlot>>> =
    OnceLock::new();

pub(crate) fn close_model_selector_popup_window(cx: &mut App) {
    if let Some(storage) = ACP_MODEL_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

/// Check if the model selector popup window is currently open.
pub(crate) fn is_model_selector_popup_window_open() -> bool {
    if let Some(storage) = ACP_MODEL_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            return guard.is_some();
        }
    }
    false
}

/// Read the model selector popup snapshot if the popup window is open.
///
/// Used by the automation surface collector to extract semantic elements
/// from the live popup state without needing `&mut App`.
pub(crate) fn get_model_selector_popup_snapshot(
    cx: &gpui::App,
) -> Option<AcpModelSelectorPopupSnapshot> {
    let storage = ACP_MODEL_SELECTOR_POPUP_WINDOW.get()?;
    let guard = storage.lock().ok()?;
    let slot = (*guard)?;
    slot.handle
        .read_with(cx, |popup, _cx| popup.snapshot.clone())
        .ok()
}

/// Select a model by its ID for batch automation.
///
/// Returns `Some(model_id)` if found and selected, `None` otherwise.
pub(crate) fn batch_select_model_by_value(value: &str, cx: &mut App) -> Option<String> {
    let storage = ACP_MODEL_SELECTOR_POPUP_WINDOW.get()?;
    let guard = storage.lock().ok()?;
    let slot = (*guard)?;
    let snap = slot
        .handle
        .read_with(cx, |popup, _cx| popup.snapshot.clone())
        .ok()?;
    // Verify the model exists in the snapshot
    if !snap.entries.iter().any(|entry| entry.id == value) {
        return None;
    }
    let _ = slot.handle.update(cx, |popup, _window, cx| {
        popup.select_model(value, cx);
    });
    Some(value.to_string())
}

/// Select a model by its semantic ID (`choice:<idx>:<model_id>`).
///
/// Returns `Some(semantic_id)` if found and selected, `None` otherwise.
pub(crate) fn batch_select_model_by_semantic_id(semantic_id: &str, cx: &mut App) -> Option<String> {
    let parts: Vec<&str> = semantic_id.splitn(3, ':').collect();
    if parts.len() < 3 || parts[0] != "choice" {
        return None;
    }
    let model_id = parts[2];
    batch_select_model_by_value(model_id, cx)?;
    Some(semantic_id.to_string())
}

fn popup_height(snapshot: &AcpModelSelectorPopupSnapshot) -> f32 {
    super::popup_window::dense_picker_height(snapshot.entries.len())
}

fn popup_bounds(
    parent_bounds: Bounds<Pixels>,
    snapshot: &AcpModelSelectorPopupSnapshot,
) -> Bounds<Pixels> {
    let height = popup_height(snapshot);
    let width = super::popup_window::dense_picker_width_for_labels(
        parent_bounds.size.width.as_f32(),
        snapshot.entries.iter().map(|entry| entry.display.as_ref()),
        true,
    );
    let top =
        super::popup_window::footer_anchored_popup_top(parent_bounds.size.height.as_f32(), height);

    super::popup_window::popup_bounds(
        parent_bounds,
        super::popup_window::DENSE_PICKER_LEFT_MARGIN,
        top,
        width,
        height,
    )
}

pub(crate) fn sync_model_selector_popup_window(
    cx: &mut App,
    request: AcpModelSelectorPopupRequest,
) -> anyhow::Result<()> {
    let AcpModelSelectorPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_id,
        source_view,
        snapshot,
    } = request;

    if snapshot.entries.is_empty() {
        close_model_selector_popup_window(cx);
        return Ok(());
    }

    let bounds = popup_bounds(parent_bounds, &snapshot);
    let storage = ACP_MODEL_SELECTOR_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
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
        cx.new(|cx| AcpModelSelectorPopupWindow::new(snapshot.clone(), source_view.clone(), cx))
    })?;

    if let Err(error) =
        super::popup_window::configure_popup_window(&handle, cx, parent_window_handle)
    {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error.context("failed to configure ACP model selector popup window"));
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(AcpModelSelectorPopupSlot {
            handle,
            parent_window_handle,
        });
    }

    Ok(())
}

pub(crate) struct AcpModelSelectorPopupWindow {
    snapshot: AcpModelSelectorPopupSnapshot,
    source_view: WeakEntity<AcpChatView>,
    focus_handle: FocusHandle,
}

impl AcpModelSelectorPopupWindow {
    fn new(
        snapshot: AcpModelSelectorPopupSnapshot,
        source_view: WeakEntity<AcpChatView>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_snapshot(&mut self, snapshot: AcpModelSelectorPopupSnapshot) {
        self.snapshot = snapshot;
    }

    fn select_model(&self, model_id: &str, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            let model_id = model_id.to_string();
            view.update(cx, |view, cx| {
                view.select_model_from_popup(&model_id, cx);
            });
        } else {
            close_model_selector_popup_window(cx);
        }
    }
}

impl Focusable for AcpModelSelectorPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpModelSelectorPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::from_theme(&theme);
        let fg: gpui::Hsla = gpui::rgb(theme.colors.text.primary).into();
        let muted_fg: gpui::Hsla = gpui::rgb(theme.colors.text.muted).into();
        let popup_height = popup_height(&self.snapshot);

        let body = div()
            .w_full()
            .max_h(px(popup_height))
            .overflow_y_scrollbar()
            .children(
                self.snapshot
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let model_id = entry.id.clone();
                        let accessory = entry.is_active.then(|| {
                            svg()
                                .path(IconName::Check.path())
                                .size(px(12.0))
                                .text_color(GOLD)
                                .into_any_element()
                        });
                        render_dense_monoline_picker_row_with_accessory(
                            SharedString::from(format!("acp-model-selector-{idx}")),
                            entry.display.clone(),
                            SharedString::default(),
                            &[],
                            &[],
                            idx == self.snapshot.selected_index,
                            fg,
                            muted_fg,
                            accessory,
                        )
                        .cursor_pointer()
                        .on_click(cx.listener(
                            move |this, _event, _window, cx| {
                                this.select_model(&model_id, cx);
                            },
                        ))
                    }),
            )
            .into_any_element();

        tracing::info!(
            target: "script_kit::tab_ai",
            popup = "model_selector",
            entry_count = self.snapshot.entries.len(),
            selected_index = self.snapshot.selected_index,
            "inline_dropdown_model_selector_rendered"
        );

        div().size_full().track_focus(&self.focus_handle).child(
            InlineDropdown::new(SharedString::from("acp-model-selector-popup"), body, colors)
                .vertical_padding(super::popup_window::DENSE_PICKER_VERTICAL_PADDING / 2.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        popup_bounds, popup_height, AcpModelSelectorPopupEntry, AcpModelSelectorPopupSnapshot,
    };
    use gpui::SharedString;

    #[test]
    fn popup_height_accounts_for_model_rows() {
        let snapshot = AcpModelSelectorPopupSnapshot {
            selected_index: 1,
            entries: vec![
                AcpModelSelectorPopupEntry {
                    id: "a".into(),
                    display: SharedString::from("A"),
                    is_active: false,
                },
                AcpModelSelectorPopupEntry {
                    id: "b".into(),
                    display: SharedString::from("B"),
                    is_active: true,
                },
            ],
        };

        assert!(popup_height(&snapshot) > 40.0);
    }

    #[test]
    fn popup_height_still_accounts_for_model_rows_after_inline_dropdown_adoption() {
        let snapshot = AcpModelSelectorPopupSnapshot {
            selected_index: 1,
            entries: vec![
                AcpModelSelectorPopupEntry {
                    id: "a".into(),
                    display: SharedString::from("A"),
                    is_active: false,
                },
                AcpModelSelectorPopupEntry {
                    id: "b".into(),
                    display: SharedString::from("B"),
                    is_active: true,
                },
            ],
        };
        assert!(popup_height(&snapshot) > 40.0);
    }

    #[test]
    fn popup_bounds_anchor_above_hint_strip() {
        let snapshot = AcpModelSelectorPopupSnapshot {
            selected_index: 0,
            entries: vec![AcpModelSelectorPopupEntry {
                id: "a".into(),
                display: SharedString::from("A"),
                is_active: false,
            }],
        };
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(40.0)),
            size: gpui::size(gpui::px(480.0), gpui::px(440.0)),
        };

        let bounds = popup_bounds(parent, &snapshot);
        let expected_height = popup_height(&snapshot);
        let expected_width = super::super::popup_window::dense_picker_width_for_labels(
            parent.size.width.as_f32(),
            snapshot.entries.iter().map(|entry| entry.display.as_ref()),
            true,
        );
        let expected_top = super::super::popup_window::footer_anchored_popup_top(
            parent.size.height.as_f32(),
            expected_height,
        );

        assert_eq!(
            bounds,
            gpui::Bounds {
                origin: gpui::point(
                    parent.origin.x
                        + gpui::px(super::super::popup_window::DENSE_PICKER_LEFT_MARGIN),
                    parent.origin.y + gpui::px(expected_top),
                ),
                size: gpui::size(gpui::px(expected_width), gpui::px(expected_height)),
            }
        );
    }
}
