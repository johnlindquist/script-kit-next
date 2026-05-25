//! Detachable ACP Chat window.
//!
//! Creates a separate PopUp window for the ACP chat that persists
//! independently from the main Script Kit panel.

use std::sync::{Mutex, OnceLock};

use gpui::{
    px, AnyWindowHandle, App, AppContext as _, Entity, Focusable, WeakEntity, WindowBounds,
    WindowKind, WindowOptions,
};

use super::thread::AcpThread;
use super::view::AcpChatView;
use crate::ai::window::context_picker::types::PortalKind;

/// State for the detached ACP Chat window.
struct ChatWindowState {
    handle: AnyWindowHandle,
    /// The live AcpChatView entity inside the detached window, if opened with a thread.
    view_entity: Option<WeakEntity<AcpChatView>>,
    /// Automation window ID registered in the runtime handle registry.
    /// Stored so we can remove the exact handle on close.
    automation_id: Option<String>,
}

/// Global handle to the detached ACP Chat window.
static CHAT_WINDOW: OnceLock<Mutex<Option<ChatWindowState>>> = OnceLock::new();

/// Check if the detached ACP Chat window is open.
pub fn is_chat_window_open() -> bool {
    let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
    let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().is_some()
}

/// Check if the given window is the detached ACP chat window.
pub fn is_chat_window(window: &gpui::Window) -> bool {
    let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
    let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_ref()
        .map(|state| window.window_handle() == state.handle)
        .unwrap_or(false)
}

/// Clear the global chat window handle (called when the view closes itself).
pub fn clear_chat_window_handle() {
    let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut g) = slot.lock() {
        if let Some(state) = g.take() {
            if let Some(ref id) = state.automation_id {
                crate::windows::remove_runtime_window_handle(id);
            }
        }
    }
}

/// Build standard window options for the detached chat window.
///
/// If `inherit_bounds` is Some, the detached window uses those bounds
/// (offset slightly right so it doesn't overlap the main panel).
fn chat_window_options(inherit_bounds: Option<gpui::Bounds<gpui::Pixels>>) -> WindowOptions {
    let window_bounds = if let Some(bounds) = inherit_bounds {
        // Offset 20px right from the main window so both are visible
        WindowBounds::Windowed(gpui::Bounds {
            origin: gpui::Point {
                x: bounds.origin.x + px(20.0),
                y: bounds.origin.y + px(20.0),
            },
            size: bounds.size,
        })
    } else {
        crate::window_state::load_window_bounds(crate::window_state::WindowRole::AcpChat)
            .map(|persisted| persisted.to_gpui())
            .unwrap_or_else(|| {
                WindowBounds::Windowed(gpui::Bounds {
                    origin: gpui::Point {
                        x: px(100.0),
                        y: px(100.0),
                    },
                    size: gpui::Size {
                        width: px(480.0),
                        height: px(440.0),
                    },
                })
            })
    };

    WindowOptions {
        window_bounds: Some(window_bounds),
        titlebar: None,
        is_movable: true,
        window_background: gpui::WindowBackgroundAppearance::Blurred,
        focus: true,
        show: true,
        kind: WindowKind::PopUp,
        ..Default::default()
    }
}

/// Open (or focus) the detached ACP Chat window with a placeholder.
pub fn open_chat_window(cx: &mut App) -> anyhow::Result<()> {
    // If already open, just focus it
    let existing = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().ok().and_then(|g| g.as_ref().map(|s| s.handle))
    };

    if existing.is_some() {
        activate_chat_window(cx);
        return Ok(());
    }

    let handle = cx.open_window(chat_window_options(None), |window, cx| {
        window.on_window_should_close(cx, |window, _cx| {
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(
                crate::window_state::WindowRole::AcpChat,
                wb,
            );
            let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                if let Some(state) = g.take() {
                    if let Some(ref id) = state.automation_id {
                        crate::windows::remove_runtime_window_handle(id);
                    }
                }
            }
            true
        });
        cx.new(|_cx| ChatWindowPlaceholder)
    })?;

    let any_handle: AnyWindowHandle = handle.into();
    let automation_id = "acpDetached:placeholder".to_string();

    // Store the handle (placeholder has no AcpChatView entity)
    {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(ChatWindowState {
                handle: any_handle,
                view_entity: None,
                automation_id: Some(automation_id.clone()),
            });
        }
    }

    // Register the exact runtime handle so simulateGpuiEvent can target
    // this window by its automation ID without collapsing to WindowRole.
    crate::windows::upsert_runtime_window_handle(&automation_id, any_handle);

    tracing::info!("acp_chat_window_opened");
    Ok(())
}

/// Open the detached ACP Chat window with an existing AcpThread entity.
/// This is used when "Detach to Window" transfers a live conversation.
///
/// If `inherit_bounds` is provided, the detached window opens at those bounds
/// (offset +20px x/y) instead of using persisted ACP chat bounds.
pub fn open_chat_window_with_thread(
    thread: Entity<AcpThread>,
    inherit_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    cx: &mut App,
) -> anyhow::Result<()> {
    // Close existing if any
    if is_chat_window_open() {
        close_chat_window(cx);
    }

    let has_inherited_bounds = inherit_bounds.is_some();

    // Read the thread's UI ID before the closure moves ownership of `thread`.
    let ui_thread_id = thread.read(cx).ui_thread_id().to_string();

    let view_entity_slot: std::sync::Arc<Mutex<Option<WeakEntity<AcpChatView>>>> =
        std::sync::Arc::new(Mutex::new(None));
    let view_entity_slot_inner = view_entity_slot.clone();
    let view_entity_slot_on_close = view_entity_slot.clone();

    let handle = cx.open_window(chat_window_options(inherit_bounds), |window, cx| {
        // Save bounds and clear handle when window closes
        window.on_window_should_close(cx, move |window, cx| {
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(
                crate::window_state::WindowRole::AcpChat,
                wb,
            );
            if let Ok(slot) = view_entity_slot_on_close.lock() {
                if let Some(entity) = slot.as_ref().and_then(|weak| weak.upgrade()) {
                    entity.update(cx, |view, cx| {
                        view.prepare_for_host_hide(cx);
                    });
                }
            }
            // Clear the global handle and remove the runtime automation handle
            let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                if let Some(state) = g.take() {
                    if let Some(ref id) = state.automation_id {
                        crate::windows::remove_runtime_window_handle(id);
                        crate::windows::remove_automation_window(id);
                    }
                }
            }
            true // allow close
        });

        let view = cx.new(|cx| AcpChatView::new(thread, cx));
        let entity_weak = view.downgrade();
        view.update(cx, |view, _cx| {
            view.set_allowed_portal_kinds(vec![PortalKind::AcpHistory]);
            view.set_on_toggle_actions(move |_window, cx| {
                toggle_detached_actions(cx);
            });
            view.set_on_close_requested(move |_window, cx| {
                close_chat_window(cx);
            });
            view.set_on_open_history_command(move |_window, cx| {
                let _ = open_picker_in_detached_chat_window(cx, |view, window, cx| {
                    let parent_handle = window.window_handle();
                    let parent_bounds = window.bounds();
                    let display_id = window.display(cx).map(|display| display.id());
                    view.open_history_popup_from_host(parent_handle, parent_bounds, display_id, cx);
                });
            });
            let paste_entity_weak = entity_weak.clone();
            view.set_on_paste_response_requested(move |_window, cx| {
                dispatch_detached_action(&paste_entity_weak, "acp_paste_to_frontmost", cx);
            });
            view.set_on_open_portal(move |kind, cx| match kind {
                PortalKind::AcpHistory => {
                    let opened = open_history_portal_in_detached_chat_window(cx);
                    if !opened {
                        let _ = cancel_portal_session_in_detached_chat_window(kind, cx);
                    }
                    tracing::info!(
                        target: "script_kit::acp",
                        event = "detached_acp_portal_requested",
                        kind = "AcpHistory",
                        opened,
                    );
                }
                other => {
                    tracing::info!(
                        target: "script_kit::acp",
                        event = "detached_acp_portal_requested",
                        kind = ?other,
                        opened = false,
                        reason = "unsupported_in_detached_host",
                    );
                }
            });
        });
        // Capture weak reference to the view entity for action dispatch.
        if let Ok(mut slot) = view_entity_slot_inner.lock() {
            *slot = Some(view.downgrade());
        }
        cx.new(|cx| gpui_component::Root::new(view, window, cx))
    })?;

    // Extract the captured weak entity.
    let view_weak = view_entity_slot.lock().ok().and_then(|mut g| g.take());

    let any_handle: AnyWindowHandle = handle.into();
    let automation_id = format!("acpDetached:{ui_thread_id}");

    {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(ChatWindowState {
                handle: any_handle,
                view_entity: view_weak,
                automation_id: Some(automation_id.clone()),
            });
        }
    }

    // Register the exact runtime handle so simulateGpuiEvent can target
    // this window by its automation ID without collapsing to WindowRole.
    crate::windows::upsert_runtime_window_handle(&automation_id, any_handle);

    // Also register the window in the automation metadata registry so that
    // `listAutomationWindows` and target-resolution by kind=acpDetached can
    // find it. Runtime handle registry and metadata registry are separate —
    // a missing metadata entry makes the window invisible to discovery even
    // when the runtime handle exists.
    let detached_bounds = inherit_bounds.map(|b| crate::protocol::AutomationWindowBounds {
        x: f32::from(b.origin.x) as f64,
        y: f32::from(b.origin.y) as f64,
        width: f32::from(b.size.width) as f64,
        height: f32::from(b.size.height) as f64,
    });
    crate::windows::upsert_automation_window(crate::protocol::AutomationWindowInfo {
        id: automation_id.clone(),
        kind: crate::protocol::AutomationWindowKind::AcpDetached,
        title: Some("Script Kit".to_string()),
        focused: true,
        visible: true,
        semantic_surface: Some("acpChat".to_string()),
        bounds: detached_bounds,
        parent_window_id: None,
        parent_kind: None,
        pid: Some(std::process::id()),
    });

    // Activate the detached window so it gets keyboard focus immediately.
    activate_chat_window(cx);

    // Configure vibrancy to match main window appearance
    configure_acp_chat_vibrancy(cx);

    tracing::info!(
        event = "acp_chat_window_opened_with_thread",
        has_inherited_bounds,
        activated = true,
    );
    Ok(())
}

/// Return a strong reference to the detached ACP chat view entity, if the
/// detached window is open and was opened with a live thread.
///
/// This is used by the automation substrate to read ACP state and test-probe
/// data from the detached window without routing through the main window.
pub fn get_detached_acp_view_entity() -> Option<Entity<AcpChatView>> {
    let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
    let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_ref()
        .and_then(|state| state.view_entity.as_ref())
        .and_then(|weak| weak.upgrade())
}

fn open_picker_in_detached_chat_window(
    cx: &mut App,
    open_picker: impl FnOnce(&mut AcpChatView, &mut gpui::Window, &mut gpui::Context<AcpChatView>),
) -> bool {
    let (handle, entity) = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = match slot.lock() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        };
        let Some(state) = guard.as_ref() else {
            return false;
        };
        let Some(entity) = state.view_entity.as_ref().and_then(|weak| weak.upgrade()) else {
            return false;
        };
        (state.handle, entity)
    };

    handle
        .update(cx, |_root, window, cx| {
            entity.update(cx, |view, cx| {
                open_picker(view, window, cx);
            });
        })
        .is_ok()
}

fn open_history_portal_in_detached_chat_window(cx: &mut App) -> bool {
    let (handle, entity) = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = match slot.lock() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        };
        let Some(state) = guard.as_ref() else {
            return false;
        };
        let Some(entity) = state.view_entity.as_ref().and_then(|weak| weak.upgrade()) else {
            return false;
        };
        (state.handle, entity)
    };

    handle
        .update(cx, |_root, window, cx| {
            window.activate_window();
            let parent_handle = window.window_handle();
            let parent_bounds = window.bounds();
            let display_id = window.display(cx).map(|display| display.id());
            entity.update(cx, |view, cx| {
                let query = view.take_pending_history_portal_query().unwrap_or_default();
                tracing::info!(
                    target: "script_kit::acp",
                    event = "detached_acp_history_portal_query_seeded_from_contract",
                    query = %query,
                );
                let hits = crate::ai::acp::history::search_history(&query, 12);
                view.open_history_portal_with_entries(query, hits, cx);
                view.open_history_popup_from_host(parent_handle, parent_bounds, display_id, cx);
            });
            let focus_handle = entity.read(cx).focus_handle(cx);
            window.focus(&focus_handle, cx);
        })
        .is_ok()
}

fn cancel_portal_session_in_detached_chat_window(kind: PortalKind, cx: &mut App) -> bool {
    let entity = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = match slot.lock() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        };
        let Some(state) = guard.as_ref() else {
            return false;
        };
        let Some(entity) = state.view_entity.as_ref().and_then(|weak| weak.upgrade()) else {
            return false;
        };
        entity
    };

    entity.update(cx, |view, cx| view.cancel_pending_portal_session(kind, cx))
}

pub fn open_detached_slash_picker(cx: &mut App) -> bool {
    open_picker_in_detached_chat_window(cx, |view, window, cx| {
        view.open_slash_picker_in_window(window, cx);
    })
}

pub fn open_detached_mention_picker(cx: &mut App) -> bool {
    open_picker_in_detached_chat_window(cx, |view, window, cx| {
        view.open_mention_picker_in_window(window, cx);
    })
}

pub fn open_detached_profile_picker(cx: &mut App) -> bool {
    open_picker_in_detached_chat_window(cx, |view, window, cx| {
        view.open_profile_picker_in_window(window, cx);
    })
}

pub fn submit_reused_entry_intent_in_detached_chat(
    intent: String,
    cx: &mut App,
) -> anyhow::Result<bool> {
    let state = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        guard.as_ref().map(|state| {
            (
                state.handle,
                state.view_entity.as_ref().and_then(|weak| weak.upgrade()),
            )
        })
    };

    let Some((handle, Some(view_entity))) = state else {
        return Ok(false);
    };

    let input_len = intent.len();
    let reused = handle
        .update(cx, |_root, window, cx| {
            window.activate_window();
            let reused = view_entity.update(cx, |chat, cx| {
                if chat.is_setup_mode() {
                    return false;
                }

                chat.submit_reused_entry_intent(intent.clone(), cx);
                true
            });
            let focus_handle = view_entity.read(cx).focus_handle(cx);
            window.focus(&focus_handle, cx);
            Ok::<bool, anyhow::Error>(reused)
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))??;

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "tab_ai_reused_detached_window_with_entry_intent",
        reused,
        input_len,
    );
    Ok(reused)
}

pub fn submit_reused_entry_intent_with_host_context_in_detached_chat(
    intent: String,
    parts: Vec<crate::ai::message_parts::AiContextPart>,
    source: &'static str,
    cx: &mut App,
) -> anyhow::Result<bool> {
    let state = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        guard.as_ref().map(|state| {
            (
                state.handle,
                state.view_entity.as_ref().and_then(|weak| weak.upgrade()),
            )
        })
    };

    let Some((handle, Some(view_entity))) = state else {
        return Ok(false);
    };

    let input_len = intent.len();
    let part_count = parts.len();
    let reused = handle
        .update(cx, |_root, window, cx| {
            window.activate_window();
            let reused = view_entity.update(cx, |chat, cx| {
                if chat.is_setup_mode() {
                    return Ok(false);
                }

                chat.submit_reused_entry_intent_with_host_context(
                    intent.clone(),
                    parts.clone(),
                    source,
                    cx,
                )?;
                Ok::<bool, String>(true)
            });
            let focus_handle = view_entity.read(cx).focus_handle(cx);
            window.focus(&focus_handle, cx);
            reused.map_err(anyhow::Error::msg)
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))??;

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "tab_ai_reused_detached_window_with_host_context_entry_intent",
        reused,
        source,
        input_len,
        part_count,
    );
    Ok(reused)
}

/// Close the detached ACP Chat window.
#[allow(dead_code)]
pub fn close_chat_window(cx: &mut App) {
    let existing = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().ok().and_then(|mut g| g.take())
    };

    if let Some(state) = existing {
        if let Some(entity) = state.view_entity.as_ref().and_then(|weak| weak.upgrade()) {
            entity.update(cx, |view, cx| {
                view.prepare_for_host_hide(cx);
            });
        }

        // Remove both the runtime handle AND the automation registry
        // entry before `window.remove_window()`. The on_close callback
        // registered in `chat_window_options` would normally do both,
        // but this helper has already taken CHAT_WINDOW.slot above so
        // the callback's `g.take()` returns `None` and the
        // registry-cleanup branch never runs. Doing it here keeps
        // `listAutomationWindows` in sync with the window's actual
        // lifetime regardless of whether close is initiated from the
        // window's titlebar (on_close fires first) or from an outside
        // TriggerAction / automation path (this helper fires first).
        if let Some(ref id) = state.automation_id {
            crate::windows::remove_runtime_window_handle(id);
            crate::windows::remove_automation_window(id);
        }

        let _ = state.handle.update(cx, |_root, window, _cx| {
            // Save window bounds before closing
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(
                crate::window_state::WindowRole::AcpChat,
                wb,
            );
            window.remove_window();
        });
    }
}

// Detached ACP action allowlist now lives in the ACP route builder layer:
// `AcpActionsDialogHost::Detached` in src/actions/builders/script_context.rs.

/// Dispatch an action to the detached ACP chat window from outside it.
///
/// Used by the automation substrate's `ExternalCommand::TriggerAction`
/// dispatcher when `host="acpDetached"`: routes the supplied action id
/// through the same `dispatch_detached_action` that the detached
/// window's own actions popup uses, so closing / reattaching / model
/// switching from automation is end-to-end identical to clicking the
/// action in the detached popup.
///
/// Returns `false` if no detached window exists or its view entity has
/// been deallocated, so callers can distinguish a dropped dispatch
/// from a successful one.
// doc-anchor-removed: [[removed-docs metadata]]
pub fn dispatch_action_to_detached(action_id: &str, cx: &mut App) -> bool {
    let view_weak = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = match slot.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        guard.as_ref().and_then(|state| state.view_entity.clone())
    };
    let Some(view_weak) = view_weak else {
        tracing::warn!(
            event = "detached_action_dispatch_no_window",
            action = %action_id,
        );
        return false;
    };
    dispatch_detached_action_checked(&view_weak, action_id, cx)
}

/// Activate (bring to front) the detached chat window.
pub(crate) fn activate_chat_window(cx: &mut App) {
    let state = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().ok().and_then(|g| {
            g.as_ref().map(|s| {
                (
                    s.handle,
                    s.view_entity.as_ref().and_then(|weak| weak.upgrade()),
                )
            })
        })
    };
    if let Some((handle, view_entity)) = state {
        let _ = handle.update(cx, |_root, window, cx| {
            window.activate_window();
            if let Some(ref entity) = view_entity {
                let focus_handle = entity.read(cx).focus_handle(cx);
                window.focus(&focus_handle, cx);
            }
        });
        tracing::info!(event = "acp_chat_window_activated");
    }
}

/// Toggle the actions popup from the detached ACP chat window.
///
/// Creates a dialog with the subset of ACP chat actions that work in the
/// detached context, positioned relative to the detached chat window.
/// After selection, the detached chat re-gains focus.
pub fn toggle_detached_actions(cx: &mut App) {
    use crate::actions::{self, ActionsDialog, WindowPosition};

    let actions_window_open_before = actions::is_actions_window_open();

    // If actions are already open, close them and re-focus the chat (toggle behavior)
    if actions_window_open_before {
        actions::close_actions_window(cx);
        activate_chat_window(cx);
        tracing::info!(
            target: "script_kit::keyboard",
            event = "detached_actions_toggle_result",
            route_owner = "detached_chat",
            restored_chat_focus = true,
            actions_window_open_before,
            actions_window_open_after = actions::is_actions_window_open(),
        );
        return;
    }

    let (handle, view_weak) = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        let guard = match slot.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match guard.as_ref() {
            Some(state) => (state.handle, state.view_entity.clone()),
            None => return,
        }
    };

    if view_weak.is_none() {
        tracing::warn!(target: "script_kit::keyboard", event = "detached_actions_no_view_entity");
        return;
    }

    // Get window bounds and display from the detached chat window
    let window_info = handle.update(cx, |_root, window, cx| {
        (
            window.window_handle(),
            window.bounds(),
            window.display(cx).map(|d| d.id()),
        )
    });

    let Ok((parent_window_handle, bounds, display_id)) = window_info else {
        return;
    };

    let theme_arc = std::sync::Arc::new(crate::theme::get_cached_theme());

    // Channel for the on_select callback to send the selected action_id
    // to the async dispatch task that has App context.
    let (action_tx, action_rx) = async_channel::bounded::<String>(1);

    let callback: std::sync::Arc<dyn Fn(String) + Send + Sync> =
        std::sync::Arc::new(move |action_id: String| {
            tracing::info!(
                event = "detached_action_selected_from_popup",
                action = %action_id,
            );
            let _ = action_tx.try_send(action_id);
        });

    // Build ACP action context from the view entity (mirrors actions_toggle.rs pattern)
    #[allow(clippy::type_complexity)]
    let acp_context: Option<(
        Option<String>,
        Vec<crate::ai::acp::AcpAgentCatalogEntry>,
        Option<String>,
        Vec<crate::ai::acp::config::AcpModelEntry>,
    )> = view_weak.as_ref().and_then(|weak| {
        weak.upgrade().map(|entity| {
            let view = entity.read(cx);
            match &view.session {
                crate::ai::acp::AcpChatSession::Setup(state) => (
                    state
                        .selected_agent
                        .as_ref()
                        .map(|agent| agent.id.to_string()),
                    crate::ai::acp::refresh_acp_agent_catalog_entries_with_snapshot(
                        &state.catalog_entries,
                    ),
                    None,
                    Vec::new(),
                ),
                crate::ai::acp::AcpChatSession::Live(thread) => {
                    let thread = thread.read(cx);
                    (
                        thread.selected_agent_id().map(str::to_string),
                        crate::ai::acp::refresh_acp_agent_catalog_entries_with_snapshot(
                            thread.available_agents(),
                        ),
                        thread.selected_model_id().map(str::to_string),
                        thread.available_models().to_vec(),
                    )
                }
            }
        })
    });

    let (selected_agent_id, catalog_entries, selected_model_id, available_models) =
        acp_context.unwrap_or_else(|| (None, Vec::new(), None, Vec::new()));

    let dialog = cx.new(|cx| {
        let focus_handle = cx.focus_handle();
        let mut dialog = ActionsDialog::with_acp_chat_for_host(
            focus_handle,
            callback,
            crate::actions::AcpActionsDialogContext {
                catalog_entries: &catalog_entries,
                selected_agent_id: selected_agent_id.as_deref(),
                available_models: &available_models,
                selected_model_id: selected_model_id.as_deref(),
            },
            theme_arc,
            crate::actions::AcpActionsDialogHost::Detached,
        );
        dialog.set_skip_track_focus(true);
        dialog
    });

    let activation_dialog = dialog.clone();
    dialog.update(cx, |dialog, _cx| {
        dialog.set_on_activation(std::sync::Arc::new(move |activation, _window, cx| match activation {
            crate::actions::ActionsDialogActivation::DrillDownPushed { .. } => {
                let (route_id, search_placeholder, route_depth, escape_hint) = {
                    let dialog_ref = activation_dialog.read(cx);
                    (
                        dialog_ref.current_route_id().map(str::to_string),
                        dialog_ref.current_search_placeholder().map(str::to_string),
                        dialog_ref.route_depth(),
                        dialog_ref.route_hint_label(),
                    )
                };
                tracing::info!(
                    target: "script_kit::actions",
                    host = "detached_acp_actions_window",
                    route_id = ?route_id,
                    route_depth,
                    escape_hint,
                    search_placeholder = ?search_placeholder,
                    "actions_dialog_route_visible"
                );
                crate::actions::resize_actions_window(cx, &activation_dialog);
            }
            crate::actions::ActionsDialogActivation::Executed { should_close, .. } => {
                if should_close {
                    let on_close = activation_dialog.read(cx).on_close.clone();
                    if let Some(on_close) = on_close {
                        on_close(cx);
                    }
                    crate::actions::close_actions_window(cx);
                }
            }
            crate::actions::ActionsDialogActivation::NoSelection => {}
        }));
        dialog.set_on_close(std::sync::Arc::new(|cx| {
            activate_chat_window(cx);
            tracing::info!(target: "script_kit::keyboard", event = "detached_actions_closed_restore_chat_focus");
        }));
    });

    let parent_automation_id = crate::windows::focused_automation_window_id();
    let actions_handle = match actions::open_actions_window(
        cx,
        parent_window_handle,
        bounds,
        display_id,
        dialog,
        WindowPosition::TopRight,
        parent_automation_id.as_deref(),
    ) {
        Ok(handle) => handle,
        Err(e) => {
            tracing::warn!(target: "script_kit::keyboard", %e, "detached_actions_open_failed");
            return;
        }
    };

    let _ = actions_handle.update(cx, |_root, window, _cx| {
        window.activate_window();
    });

    tracing::info!(
        target: "script_kit::keyboard",
        event = "detached_actions_toggle_result",
        route_owner = "detached_chat",
        restored_chat_focus = false,
        actions_window_open_before,
        actions_window_open_after = crate::actions::is_actions_window_open(),
        has_view_entity = view_weak.is_some(),
    );

    // Spawn a one-shot task that receives the selected action_id from the
    // channel, dispatches it to the AcpChatView entity, and re-focuses the chat.
    if let Some(entity_weak) = view_weak {
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            if let Ok(action_id) = action_rx.recv().await {
                if action_id == "__cancel__" {
                    return;
                }
                tracing::info!(
                    event = "detached_action_dispatch",
                    action = %action_id,
                );
                cx.update(|cx| {
                    let handled = dispatch_detached_action_checked(&entity_weak, &action_id, cx);
                    // Re-focus the detached chat after action dispatch
                    // (unless the action closed the window)
                    if handled && action_id != "acp_close" {
                        activate_chat_window(cx);
                    }
                    tracing::info!(
                        event = "detached_action_dispatch_completed",
                        action = %action_id,
                        handled,
                    );
                });
            }
        })
        .detach();
    }
}

/// Checked wrapper around `dispatch_detached_action` that logs when the
/// view entity has already been deallocated and avoids a silent no-op.
fn dispatch_detached_action_checked(
    entity_weak: &WeakEntity<AcpChatView>,
    action_id: &str,
    cx: &mut App,
) -> bool {
    if entity_weak.upgrade().is_none() {
        tracing::warn!(
            event = "detached_action_dispatch_dropped_no_view",
            action = %action_id,
        );
        return false;
    }
    dispatch_detached_action(entity_weak, action_id, cx);
    true
}

/// Dispatch an action to the detached AcpChatView entity.
///
/// Handles the subset of ACP chat actions that make sense in the detached
/// window context (copy, scroll, expand/collapse, close, reattach, etc.).
fn dispatch_detached_action(entity_weak: &WeakEntity<AcpChatView>, action_id: &str, cx: &mut App) {
    tracing::info!(
        event = "acp_actions_menu_selected",
        host = "detached",
        action_id,
        "Selected ACP Actions Menu item"
    );

    if let Some(model_id) = crate::actions::acp_switch_model_id_from_action(action_id) {
        if let Some(entity) = entity_weak.upgrade() {
            entity.update(cx, |chat, cx| {
                if let Some(thread) = chat.thread() {
                    thread.update(cx, |thread, cx| {
                        thread.select_model(model_id, cx);
                    });
                }
            });
            tracing::info!(
                event = "detached_action_switch_model",
                model_id = %model_id,
            );
        }
        return;
    }

    match action_id {
        "acp_copy_last_response" => {
            if let Some(entity) = entity_weak.upgrade() {
                let maybe_last = {
                    let view = entity.read(cx);
                    view.thread().and_then(|thread| {
                        thread
                            .read(cx)
                            .messages
                            .iter()
                            .rev()
                            .find(|m| {
                                matches!(m.role, super::thread::AcpThreadMessageRole::Assistant)
                            })
                            .map(|m| m.body.to_string())
                    })
                };
                if let Some(last_assistant) = maybe_last {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(last_assistant));
                    tracing::info!(event = "detached_action_copy_last_response");
                }
            }
        }
        "acp_paste_to_frontmost" => {
            if let Some(entity) = entity_weak.upgrade() {
                let maybe_last = {
                    let view = entity.read(cx);
                    view.thread().and_then(|thread| {
                        thread
                            .read(cx)
                            .messages
                            .iter()
                            .rev()
                            .find(|m| {
                                matches!(m.role, super::thread::AcpThreadMessageRole::Assistant)
                                    && !m.body.trim().is_empty()
                            })
                            .map(|m| m.body.to_string())
                    })
                };
                if let Some(last_assistant) = maybe_last {
                    close_chat_window(cx);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        let injector = crate::text_injector::TextInjector::new();
                        if let Err(error) = injector.paste_text(&last_assistant) {
                            tracing::warn!(
                                event = "detached_action_paste_last_response_failed",
                                %error
                            );
                        }
                    });
                    tracing::info!(event = "detached_action_paste_last_response");
                }
            }
        }
        "acp_export_markdown" => {
            if let Some(entity) = entity_weak.upgrade() {
                let maybe_markdown = {
                    let view = entity.read(cx);
                    view.thread().and_then(|thread| {
                        super::export::build_acp_conversation_markdown_from_thread(thread.read(cx))
                    })
                };
                if let Some(md) = maybe_markdown {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(md));
                    tracing::info!(event = "detached_action_export_markdown");
                }
            }
        }
        "acp_save_as_note" => {
            if let Some(entity) = entity_weak.upgrade() {
                let maybe_markdown = {
                    let view = entity.read(cx);
                    view.thread().and_then(|thread| {
                        super::export::build_acp_conversation_markdown_from_thread(thread.read(cx))
                    })
                };
                if let Some(markdown) = maybe_markdown {
                    match crate::notes::save_note_with_content(cx, markdown) {
                        Ok(()) => {
                            close_chat_window(cx);
                            tracing::info!(
                                event = "detached_action_save_as_note",
                                handoff = "notes_window"
                            );
                        }
                        Err(error) => {
                            tracing::warn!(
                                event = "detached_action_save_as_note_failed",
                                error = %error
                            );
                        }
                    }
                } else {
                    tracing::warn!(event = "detached_action_save_as_note_blocked");
                }
            }
        }
        "acp_retry_last" => {
            if let Some(entity) = entity_weak.upgrade() {
                let last_user_msg = {
                    let view = entity.read(cx);
                    view.thread().and_then(|thread| {
                        thread
                            .read(cx)
                            .messages
                            .iter()
                            .rev()
                            .find(|m| matches!(m.role, super::thread::AcpThreadMessageRole::User))
                            .map(|m| m.body.to_string())
                    })
                };
                if let Some(text) = last_user_msg {
                    entity.update(cx, |chat, cx| {
                        chat.live_thread().update(cx, |thread, cx| {
                            thread.set_input(text, cx);
                            let _ = thread.submit_input(cx);
                        });
                    });
                    tracing::info!(event = "detached_action_retry_last");
                }
            }
        }
        "acp_new_conversation" => {
            if let Some(entity) = entity_weak.upgrade() {
                entity.update(cx, |chat, cx| {
                    chat.live_thread().update(cx, |thread, cx| {
                        thread.clear_messages(cx);
                    });
                    if let Some(transcript) = &chat.transcript {
                        transcript.update(cx, |t, cx| t.clear_collapsed_ids(cx));
                    }
                    cx.notify();
                });
                tracing::info!(event = "detached_action_new_conversation");
            }
        }
        "acp_show_history" => {
            // Removed: clipboard-export behavior. History browsing now uses
            // the dedicated AcpHistory builtin in the main panel. This action
            // is filtered out of DETACHED_SUPPORTED_ACTIONS, so this arm only
            // fires if dispatched programmatically.
            tracing::info!(event = "detached_action_show_history_noop");
        }
        "acp_clear_history" => {
            let kit = crate::setup::get_kit_path();
            let _ = std::fs::remove_file(kit.join("acp-history.jsonl"));
            let _ = std::fs::remove_dir_all(kit.join("acp-conversations"));
            tracing::info!(event = "detached_action_clear_history");
        }
        "acp_scroll_to_top" => {
            if let Some(entity) = entity_weak.upgrade() {
                entity.update(cx, |chat, cx| {
                    if let Some(transcript) = &chat.transcript {
                        transcript.read(cx).scroll_to(gpui::ListOffset {
                            item_ix: 0,
                            offset_in_item: px(0.),
                        });
                    }
                    cx.notify();
                });
            }
        }
        "acp_scroll_to_bottom" => {
            if let Some(entity) = entity_weak.upgrade() {
                entity.update(cx, |chat, cx| {
                    if let Some(transcript) = &chat.transcript {
                        transcript.read(cx).scroll_to_end();
                    }
                    cx.notify();
                });
            }
        }
        "acp_expand_all" => {
            if let Some(entity) = entity_weak.upgrade() {
                entity.update(cx, |chat, cx| {
                    let _ids: Vec<u64> = chat
                        .live_thread()
                        .read(cx)
                        .messages
                        .iter()
                        .filter(|m| {
                            matches!(
                                m.role,
                                super::thread::AcpThreadMessageRole::Thought
                                    | super::thread::AcpThreadMessageRole::Tool
                            )
                        })
                        .map(|m| m.id)
                        .collect();
                    // TODO: Re-implement collapse-all via transcript entity.
                    // The collapsed_ids field has moved to AcpTranscript.
                    cx.notify();
                });
            }
        }
        "acp_collapse_all" => {
            if let Some(entity) = entity_weak.upgrade() {
                entity.update(cx, |chat, cx| {
                    if let Some(transcript) = &chat.transcript {
                        transcript.update(cx, |t, cx| t.clear_collapsed_ids(cx));
                    }
                    cx.notify();
                });
            }
        }
        "acp_reattach_panel" => {
            close_chat_window(cx);
            tracing::info!(event = "detached_action_reattach_panel");
        }
        "acp_close" => {
            close_chat_window(cx);
            tracing::info!(event = "detached_action_close");
        }
        other => {
            tracing::warn!(
                event = "detached_action_unhandled",
                action = %other,
            );
        }
    }
}

/// Window title used internally for NSWindow matching (not displayed — titlebar is None).
const ACP_CHAT_WINDOW_TITLE: &str = "Script Kit Agent Chat";

/// Configure vibrancy on the ACP chat window to match the main window appearance.
///
/// Sets the NSWindow title (invisible since titlebar is None), then finds the
/// window by title and applies the same vibrancy material as Notes/AI windows.
#[cfg(target_os = "macos")]
fn configure_acp_chat_vibrancy(cx: &mut App) {
    use objc::{msg_send, sel, sel_impl};

    let handle = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().ok().and_then(|g| g.as_ref().map(|s| s.handle))
    };

    let Some(handle) = handle else { return };

    // First, set the window title via the GPUI handle so we can find it by title.
    let _ = handle.update(cx, |_root, window, _cx| {
        // SAFETY: GPUI Window exposes the title setter.
        window.set_window_title(&gpui::SharedString::from(ACP_CHAT_WINDOW_TITLE));
    });

    // Now find it by title, same pattern as Notes/AI windows.
    // SAFETY: We're on the main thread (GPUI guarantees this for App callbacks).
    // All NSWindow pointers are nil-checked. Title string pointer is nil-checked.
    unsafe {
        use cocoa::appkit::NSApp;
        use cocoa::base::nil;
        use std::ffi::CStr;

        let app = NSApp();
        let windows: cocoa::base::id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
            let title: cocoa::base::id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == ACP_CHAT_WINDOW_TITLE {
                        let theme = crate::theme::get_cached_theme();
                        let is_dark = theme.should_use_dark_vibrancy();
                        crate::platform::configure_secondary_window_vibrancy(
                            window,
                            "Agent Chat",
                            is_dark,
                        );
                        tracing::info!("acp_chat_vibrancy_configured");
                        return;
                    }
                }
            }
        }

        tracing::warn!("acp_chat_window_not_found_for_vibrancy");
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_acp_chat_vibrancy(_cx: &mut App) {}

/// Minimal placeholder view for the detached chat window.
struct ChatWindowPlaceholder;

impl gpui::Render for ChatWindowPlaceholder {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::{div, prelude::*, rgb};
        let theme = crate::theme::get_cached_theme();

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(div().text_base().opacity(0.7).child("Agent Chat Window"))
            .child(
                div()
                    .pt(px(8.0))
                    .text_sm()
                    .opacity(0.45)
                    .child("Detached chat \u{2014} full implementation coming soon"),
            )
            .child(
                div()
                    .pt(px(4.0))
                    .text_xs()
                    .opacity(0.35)
                    .text_color(rgb(theme.colors.accent.selected))
                    .child("\u{2318}W to close"),
            )
    }
}
