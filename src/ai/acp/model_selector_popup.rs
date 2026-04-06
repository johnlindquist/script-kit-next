use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, svg, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle,
    Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::ai::context_picker_row::{render_dense_monoline_picker_row_with_accessory, GOLD};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{IconName, IconNamed};

use super::view::AcpChatView;

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupEntry {
    pub(crate) id: String,
    pub(crate) display: SharedString,
    pub(crate) is_selected: bool,
}

#[derive(Clone)]
pub(crate) struct AcpModelSelectorPopupSnapshot {
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
    let parent_height = parent_bounds.size.height.as_f32();
    let top = super::popup_window::footer_anchored_popup_top(parent_height, height);

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
        let fg: gpui::Hsla = gpui::rgb(theme.colors.text.primary).into();
        let muted_fg: gpui::Hsla = gpui::rgb(theme.colors.text.muted).into();

        super::popup_window::dense_picker_popup_surface(SharedString::from(
            "acp-model-selector-popup",
        ))
        .track_focus(&self.focus_handle)
        .w_full()
        .h_full()
        .py(px(super::popup_window::DENSE_PICKER_VERTICAL_PADDING / 2.0))
        .child(
            div()
                .w_full()
                .max_h(px(popup_height(&self.snapshot)))
                .overflow_y_scrollbar()
                .children(
                    self.snapshot
                        .entries
                        .iter()
                        .enumerate()
                        .map(|(idx, entry)| {
                            let model_id = entry.id.clone();
                            let accessory = entry.is_selected.then(|| {
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
                                entry.is_selected,
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
                ),
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
            entries: vec![
                AcpModelSelectorPopupEntry {
                    id: "a".into(),
                    display: SharedString::from("A"),
                    is_selected: false,
                },
                AcpModelSelectorPopupEntry {
                    id: "b".into(),
                    display: SharedString::from("B"),
                    is_selected: true,
                },
            ],
        };

        assert!(popup_height(&snapshot) > 40.0);
    }

    #[test]
    fn popup_bounds_anchor_above_hint_strip() {
        let snapshot = AcpModelSelectorPopupSnapshot {
            entries: vec![AcpModelSelectorPopupEntry {
                id: "a".into(),
                display: SharedString::from("A"),
                is_selected: false,
            }],
        };
        let parent = gpui::Bounds {
            origin: gpui::point(gpui::px(100.0), gpui::px(40.0)),
            size: gpui::size(gpui::px(480.0), gpui::px(440.0)),
        };

        let bounds = popup_bounds(parent, &snapshot);
        assert_eq!(f32::from(bounds.origin.x), 108.0);
        assert!(f32::from(bounds.origin.y) > 40.0);
        assert!(f32::from(bounds.size.width) >= super::super::popup_window::DENSE_PICKER_MIN_WIDTH);
    }
}
