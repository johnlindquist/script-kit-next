use anyhow::Context as _;
use std::sync::{Mutex, OnceLock};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, svg, AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, FocusHandle,
    Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, Window, WindowHandle,
};

use crate::components::inline_dropdown::{
    render_dense_monoline_picker_row_with_accessory, InlineDropdown, InlineDropdownColors,
};
use gpui_component::{IconName, IconNamed};

use super::view::AcpChatView;

pub(crate) const AGENT_CHAT_PROFILE_SELECTOR_POPUP_AUTOMATION_ID: &str =
    "agent-chat-profile-selector-popup";

#[derive(Clone)]
pub(crate) struct AgentChatProfileSelectorPopupEntry {
    pub(crate) id: String,
    pub(crate) display: SharedString,
    pub(crate) is_active: bool,
}

#[derive(Clone)]
pub(crate) struct AgentChatProfileSelectorPopupSnapshot {
    pub(crate) selected_index: usize,
    pub(crate) entries: Vec<AgentChatProfileSelectorPopupEntry>,
}

#[derive(Clone)]
pub(crate) struct AgentChatProfileSelectorPopupRequest {
    pub(crate) parent_window_handle: AnyWindowHandle,
    pub(crate) parent_bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) source_view: WeakEntity<AcpChatView>,
    pub(crate) snapshot: AgentChatProfileSelectorPopupSnapshot,
}

struct AgentChatProfileSelectorPopupSlot {
    handle: WindowHandle<AgentChatProfileSelectorPopupWindow>,
    parent_window_handle: AnyWindowHandle,
    _registration: super::popup_registry::AcpPopupRegistration,
}

static AGENT_CHAT_PROFILE_SELECTOR_POPUP_WINDOW: OnceLock<
    Mutex<Option<AgentChatProfileSelectorPopupSlot>>,
> = OnceLock::new();

fn unregister_profile_selector_popup_automation_window() {
    super::popup_window::unregister_acp_prompt_popup_automation_window(
        AGENT_CHAT_PROFILE_SELECTOR_POPUP_AUTOMATION_ID,
    );
}

pub(crate) fn close_profile_selector_popup_window(cx: &mut App) {
    unregister_profile_selector_popup_automation_window();
    if let Some(storage) = AGENT_CHAT_PROFILE_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(slot) = guard.take() {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

fn register_profile_selector_popup_automation_window(
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    popup_bounds: Bounds<Pixels>,
) -> anyhow::Result<()> {
    super::popup_window::register_acp_prompt_popup_automation_window(
        AGENT_CHAT_PROFILE_SELECTOR_POPUP_AUTOMATION_ID,
        "Agent Chat Profile Selector",
        parent_window_handle,
        parent_bounds,
        popup_bounds,
    )
}

/// Check if the profile selector popup window is currently open.
pub(crate) fn is_profile_selector_popup_window_open() -> bool {
    if let Some(storage) = AGENT_CHAT_PROFILE_SELECTOR_POPUP_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            return guard.is_some();
        }
    }
    false
}

/// Read the profile selector popup snapshot if the popup window is open.
///
/// Used by the automation surface collector to extract semantic elements
/// from the live popup state without needing `&mut App`.
pub(crate) fn get_profile_selector_popup_snapshot(
    cx: &gpui::App,
) -> Option<AgentChatProfileSelectorPopupSnapshot> {
    let storage = AGENT_CHAT_PROFILE_SELECTOR_POPUP_WINDOW.get()?;
    let guard = storage.lock().ok()?;
    let slot = guard.as_ref()?;
    slot.handle
        .read_with(cx, |popup, _cx| popup.snapshot.clone())
        .ok()
}

/// Select a profile by its ID for batch automation.
///
/// Returns `Some(profile_id)` if found and selected, `None` otherwise.
pub(crate) fn batch_select_profile_by_value(value: &str, cx: &mut App) -> Option<String> {
    let handle = {
        let storage = AGENT_CHAT_PROFILE_SELECTOR_POPUP_WINDOW.get()?;
        let guard = storage.lock().ok()?;
        let slot = guard.as_ref()?;
        let snap = slot
            .handle
            .read_with(cx, |popup, _cx| popup.snapshot.clone())
            .ok()?;
        if !snap.entries.iter().any(|entry| entry.id == value) {
            return None;
        }
        slot.handle.clone()
    };
    let _ = handle.update(cx, |popup, _window, cx| {
        popup.select_profile(value, cx);
    });
    Some(value.to_string())
}

/// Select a profile by its semantic ID (`choice:<idx>:<profile_id>`).
///
/// Returns `Some(semantic_id)` if found and selected, `None` otherwise.
pub(crate) fn batch_select_profile_by_semantic_id(
    semantic_id: &str,
    cx: &mut App,
) -> Option<String> {
    let parts: Vec<&str> = semantic_id.splitn(3, ':').collect();
    if parts.len() < 3 || parts[0] != "choice" {
        return None;
    }
    let profile_id = parts[2];
    batch_select_profile_by_value(profile_id, cx)?;
    Some(semantic_id.to_string())
}

fn popup_height(snapshot: &AgentChatProfileSelectorPopupSnapshot) -> f32 {
    super::popup_window::dense_picker_height(snapshot.entries.len())
}

fn popup_bounds(
    parent_bounds: Bounds<Pixels>,
    snapshot: &AgentChatProfileSelectorPopupSnapshot,
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

pub(crate) fn sync_profile_selector_popup_window(
    cx: &mut App,
    request: AgentChatProfileSelectorPopupRequest,
) -> anyhow::Result<()> {
    let AgentChatProfileSelectorPopupRequest {
        parent_window_handle,
        parent_bounds,
        display_id,
        source_view,
        snapshot,
    } = request;

    if snapshot.entries.is_empty() {
        close_profile_selector_popup_window(cx);
        return Ok(());
    }

    let bounds = popup_bounds(parent_bounds, &snapshot);
    let storage = AGENT_CHAT_PROFILE_SELECTOR_POPUP_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        if let Some(slot) = guard.as_ref() {
            if slot.parent_window_handle == parent_window_handle {
                let update_result = slot.handle.update(cx, |popup, window, cx| {
                    popup.set_snapshot(snapshot.clone());
                    super::popup_window::set_popup_window_bounds(window, bounds, cx);
                    cx.notify();
                });

                if update_result.is_ok() {
                    if let Err(error) = register_profile_selector_popup_automation_window(
                        parent_window_handle,
                        parent_bounds,
                        bounds,
                    ) {
                        tracing::warn!(
                            target: "script_kit::automation",
                            event = "acp_profile_selector_popup_registry_failed",
                            error = %error,
                            "Failed to refresh ACP profile selector popup automation registry entry"
                        );
                    }
                    return Ok(());
                }

                unregister_profile_selector_popup_automation_window();
                *guard = None;
            } else {
                let _ = slot.handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
                unregister_profile_selector_popup_automation_window();
                *guard = None;
            }
        }
    }

    let window_options = super::popup_window::popup_window_options(bounds, display_id);

    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| {
            AgentChatProfileSelectorPopupWindow::new(snapshot.clone(), source_view.clone(), cx)
        })
    })?;

    if let Err(error) =
        super::popup_window::configure_popup_window(&handle, cx, parent_window_handle)
    {
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        unregister_profile_selector_popup_automation_window();
        return Err(error.context("failed to configure ACP profile selector popup window"));
    }

    let any_handle: AnyWindowHandle = handle.into();
    let registration = super::popup_registry::AcpPopupRegistration::register(
        AGENT_CHAT_PROFILE_SELECTOR_POPUP_AUTOMATION_ID,
        any_handle,
    );
    if let Err(error) = register_profile_selector_popup_automation_window(
        parent_window_handle,
        parent_bounds,
        bounds,
    ) {
        unregister_profile_selector_popup_automation_window();
        let _ = handle.update(cx, |_popup, window, _cx| {
            window.remove_window();
        });
        return Err(error);
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(AgentChatProfileSelectorPopupSlot {
            handle,
            parent_window_handle,
            _registration: registration,
        });
    }

    Ok(())
}

pub(crate) struct AgentChatProfileSelectorPopupWindow {
    snapshot: AgentChatProfileSelectorPopupSnapshot,
    source_view: WeakEntity<AcpChatView>,
    focus_handle: FocusHandle,
}

impl AgentChatProfileSelectorPopupWindow {
    fn new(
        snapshot: AgentChatProfileSelectorPopupSnapshot,
        source_view: WeakEntity<AcpChatView>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            source_view,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_snapshot(&mut self, snapshot: AgentChatProfileSelectorPopupSnapshot) {
        self.snapshot = snapshot;
    }

    fn visible_range(&self) -> std::ops::Range<usize> {
        crate::components::inline_dropdown::inline_dropdown_visible_range(
            self.snapshot.selected_index,
            self.snapshot.entries.len(),
            super::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS,
        )
    }

    fn select_profile(&self, profile_id: &str, cx: &mut App) {
        if let Some(view) = self.source_view.upgrade() {
            let profile_id = profile_id.to_string();
            view.update(cx, |view, cx| {
                view.select_profile_from_popup(&profile_id, cx);
            });
        } else {
            close_profile_selector_popup_window(cx);
        }
    }
}

impl Focusable for AgentChatProfileSelectorPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AgentChatProfileSelectorPopupWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::from_theme(&theme);
        let fg = colors.foreground;
        let muted_fg = colors.muted_foreground;
        let accent = colors.accent;
        let visible = self.visible_range();

        let body = div()
            .size_full()
            .flex()
            .flex_col()
            .children(
                self.snapshot
                    .entries
                    .iter()
                    .enumerate()
                    .skip(visible.start)
                    .take(visible.len())
                    .map(|(idx, entry)| {
                        let profile_id = entry.id.clone();
                        let accessory = entry.is_active.then(|| {
                            svg()
                                .path(IconName::Check.path())
                                .size(px(12.0))
                                .text_color(accent)
                                .into_any_element()
                        });
                        render_dense_monoline_picker_row_with_accessory(
                            SharedString::from(format!("acp-profile-selector-{idx}")),
                            entry.display.clone(),
                            SharedString::default(),
                            &[],
                            &[],
                            idx == self.snapshot.selected_index,
                            fg,
                            muted_fg,
                            accent,
                            accessory,
                        )
                        .cursor_pointer()
                        .on_click(cx.listener(
                            move |this, _event, _window, cx| {
                                this.select_profile(&profile_id, cx);
                            },
                        ))
                    }),
            )
            .into_any_element();

        tracing::info!(
            target: "script_kit::tab_ai",
            popup = "profile_selector",
            entry_count = self.snapshot.entries.len(),
            selected_index = self.snapshot.selected_index,
            visible_start = visible.start,
            visible_end = visible.end,
            "inline_dropdown_profile_selector_rendered"
        );

        div().size_full().track_focus(&self.focus_handle).child(
            InlineDropdown::new(
                SharedString::from("acp-profile-selector-popup"),
                body,
                colors,
            )
            .vertical_padding(super::popup_window::DENSE_PICKER_VERTICAL_PADDING / 2.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        popup_bounds, popup_height, AgentChatProfileSelectorPopupEntry,
        AgentChatProfileSelectorPopupSnapshot,
    };
    use gpui::SharedString;

    #[test]
    fn popup_height_accounts_for_profile_rows() {
        let snapshot = AgentChatProfileSelectorPopupSnapshot {
            selected_index: 1,
            entries: vec![
                AgentChatProfileSelectorPopupEntry {
                    id: "a".into(),
                    display: SharedString::from("A"),
                    is_active: false,
                },
                AgentChatProfileSelectorPopupEntry {
                    id: "b".into(),
                    display: SharedString::from("B"),
                    is_active: true,
                },
            ],
        };

        assert!(popup_height(&snapshot) > 40.0);
    }

    #[test]
    fn popup_height_still_accounts_for_profile_rows_after_inline_dropdown_adoption() {
        let snapshot = AgentChatProfileSelectorPopupSnapshot {
            selected_index: 1,
            entries: vec![
                AgentChatProfileSelectorPopupEntry {
                    id: "a".into(),
                    display: SharedString::from("A"),
                    is_active: false,
                },
                AgentChatProfileSelectorPopupEntry {
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
        let snapshot = AgentChatProfileSelectorPopupSnapshot {
            selected_index: 0,
            entries: vec![AgentChatProfileSelectorPopupEntry {
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
