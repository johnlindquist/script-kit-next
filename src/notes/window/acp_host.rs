//! Notes-hosted ACP surface helpers.
//!
//! Manages the embedded ACP chat lifecycle inside the Notes window:
//! open/reuse, switch back to Notes editor, toggle actions dialog,
//! and dispatch Notes-specific ACP actions.

use super::*;

impl NotesApp {
    /// Switch the Notes window to show an embedded ACP chat surface.
    ///
    /// If an ACP view is already cached, reuses it (sets input if provided).
    /// Otherwise spawns a fresh hosted view via the host-neutral bootstrap.
    pub(super) fn open_or_focus_embedded_acp(
        &mut self,
        initial_input: Option<String>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let reused = self.embedded_acp_chat.is_some();

        tracing::info!(
            event = "notes_acp_surface_open_requested",
            reused,
            has_input = initial_input.as_ref().is_some_and(|s| !s.trim().is_empty()),
        );

        if let Some(entity) = self.embedded_acp_chat.as_ref().cloned() {
            // Reuse existing view — just update input if provided.
            if let Some(input) = initial_input.filter(|s| !s.trim().is_empty()) {
                entity.update(cx, |chat, cx| {
                    if !chat.is_setup_mode() {
                        chat.set_input(input, cx);
                    }
                });
            }
        } else {
            // Spawn a fresh view.
            let requirements = crate::ai::acp::preflight::AcpLaunchRequirements::default();
            let view = crate::ai::acp::hosted::spawn_hosted_view(initial_input, requirements, cx)?;

            // Wire host callbacks before caching.
            self.wire_acp_host_callbacks(&view, cx);
            self.embedded_acp_chat = Some(view);
        }

        self.surface_mode = NotesSurfaceMode::Acp;
        self.pending_focus_surface = Some(focus::NotesFocusSurface::AcpChat);

        tracing::info!(event = "notes_acp_surface_opened", reused, mode = "acp",);

        cx.notify();
        Ok(())
    }

    /// Switch from embedded ACP back to the Notes editor surface.
    ///
    /// Calls `prepare_for_host_hide()` on the cached ACP view so popups
    /// and mention sessions are properly closed, then returns focus to
    /// the editor.
    pub(super) fn switch_to_notes_surface(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Hide ACP popups before switching.
        if let Some(ref entity) = self.embedded_acp_chat {
            entity.update(cx, |chat, cx| {
                chat.prepare_for_host_hide(cx);
            });
        }

        self.surface_mode = NotesSurfaceMode::Notes;
        self.request_focus_surface(focus::NotesFocusSurface::Editor, window, cx);

        tracing::info!(event = "notes_acp_surface_closed_to_notes");
        cx.notify();
    }

    /// Wire ACP host callbacks (toggle-actions, close) to Notes-owned handlers.
    fn wire_acp_host_callbacks(
        &self,
        view: &Entity<crate::ai::acp::view::AcpChatView>,
        cx: &mut Context<Self>,
    ) {
        let notes_entity = cx.entity().downgrade();

        // Toggle actions: open the Notes-hosted ACP actions dialog.
        let toggle_entity = notes_entity.clone();
        view.update(cx, |chat, _cx| {
            chat.set_on_toggle_actions(move |window, cx| {
                if let Some(entity) = toggle_entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.toggle_acp_actions(window, cx);
                    });
                }
            });
        });

        // Close: return to Notes editor rather than closing the window.
        let close_entity = notes_entity;
        view.update(cx, |chat, _cx| {
            chat.set_on_close_requested(move |window, cx| {
                if let Some(entity) = close_entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.switch_to_notes_surface(window, cx);
                    });
                }
            });
        });
    }

    /// Toggle the ACP actions dialog for the Notes-hosted ACP surface.
    ///
    /// Opens a filtered actions popup positioned relative to the Notes window.
    /// On close, re-focuses the ACP chat inside Notes.
    pub(super) fn toggle_acp_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::actions::{self, ActionsDialog, WindowPosition};

        let actions_open_before = actions::is_actions_window_open();

        if actions_open_before {
            actions::close_actions_window(cx);
            tracing::info!(event = "notes_acp_actions_closed");
            cx.notify();
            return;
        }

        let Some(ref acp_view) = self.embedded_acp_chat else {
            return;
        };

        // Read ACP context from the cached view.
        let (selected_agent_id, catalog_entries, selected_model_id, available_models) = {
            let view = acp_view.read(cx);
            match &view.session {
                crate::ai::acp::AcpChatSession::Setup(state) => (
                    state
                        .selected_agent
                        .as_ref()
                        .map(|agent| agent.id.to_string()),
                    state.catalog_entries.clone(),
                    None,
                    Vec::new(),
                ),
                crate::ai::acp::AcpChatSession::Live(thread) => {
                    let thread = thread.read(cx);
                    (
                        thread.selected_agent_id().map(str::to_string),
                        thread.available_agents().to_vec(),
                        thread.selected_model_id().map(str::to_string),
                        thread.available_models().to_vec(),
                    )
                }
            }
        };

        let theme_arc = std::sync::Arc::new(crate::theme::get_cached_theme());
        let (action_tx, action_rx) = async_channel::bounded::<String>(1);

        let callback: std::sync::Arc<dyn Fn(String) + Send + Sync> =
            std::sync::Arc::new(move |action_id: String| {
                tracing::info!(
                    event = "notes_acp_action_selected_from_popup",
                    action = %action_id,
                );
                let _ = action_tx.try_send(action_id);
            });

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
                crate::actions::AcpActionsDialogHost::Notes,
            );
            dialog.set_skip_track_focus(true);
            dialog
        });

        let activation_dialog = dialog.clone();
        let notes_entity = cx.entity().downgrade();
        dialog.update(cx, |dialog, _cx| {
            dialog.set_on_activation(std::sync::Arc::new(move |activation, _window, cx| {
                match activation {
                    crate::actions::ActionsDialogActivation::DrillDownPushed { .. } => {
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
                }
            }));
            dialog.set_on_close(std::sync::Arc::new(move |cx| {
                if let Some(entity) = notes_entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        if app.surface_mode == NotesSurfaceMode::Acp {
                            app.pending_focus_surface = Some(focus::NotesFocusSurface::AcpChat);
                            cx.notify();
                        }
                    });
                }
                tracing::info!(event = "notes_acp_actions_closed_restore_focus");
            }));
        });

        let parent_window_handle = window.window_handle();
        let bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());
        let parent_automation_id = crate::windows::focused_automation_window_id();

        match actions::open_actions_window(
            cx,
            parent_window_handle,
            bounds,
            display_id,
            dialog,
            WindowPosition::TopRight,
            parent_automation_id.as_deref(),
        ) {
            Ok(handle) => {
                let _ = handle.update(cx, |_root, window, _cx| {
                    window.activate_window();
                });
            }
            Err(e) => {
                tracing::warn!(event = "notes_acp_actions_open_failed", error = %e);
                return;
            }
        }

        tracing::info!(event = "notes_acp_actions_opened");

        // Spawn a one-shot task to dispatch the selected action.
        // `spawn_in(window, ...)` gives `AsyncWindowContext` so `.update()`
        // runs inside the Notes window and yields (&mut Window, &mut App).
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(action_id) = action_rx.recv().await {
                if action_id == "__cancel__" {
                    return;
                }
                let _ = cx.update(|window, app_cx| {
                    dispatch_notes_acp_action(this.upgrade(), &action_id, window, app_cx);
                });
            }
        })
        .detach();
    }

    /// Accessor for the current surface mode.
    pub(crate) fn surface_mode(&self) -> NotesSurfaceMode {
        self.surface_mode
    }
}

/// Dispatch an ACP action from the Notes-hosted actions dialog popup.
///
/// Called from the async spawn inside `toggle_acp_actions`.  Receives
/// `&mut Window` and `&mut App` from `AsyncWindowContext::update`, so it
/// can interact with the ACP view entity and the Notes host state.
fn dispatch_notes_acp_action(
    entity: Option<Entity<NotesApp>>,
    action_id: &str,
    window: &mut Window,
    cx: &mut gpui::App,
) {
    let Some(entity) = entity else { return };

    tracing::info!(
        event = "notes_acp_action_dispatched",
        action_id = %action_id,
    );

    // For `acp_close`, route to the Notes host to switch surfaces.
    if action_id == "acp_close" {
        entity.update(cx, |app: &mut NotesApp, cx| {
            // Need window for focus management — use update_in if available,
            // otherwise defer to next frame.
            app.surface_mode = NotesSurfaceMode::Notes;
            if let Some(ref acp_entity) = app.embedded_acp_chat {
                acp_entity.update(cx, |chat, cx| {
                    chat.prepare_for_host_hide(cx);
                });
            }
            app.pending_focus_surface = Some(focus::NotesFocusSurface::Editor);
            tracing::info!(event = "notes_acp_surface_closed_to_notes");
            cx.notify();
        });
        return;
    }

    // Read the embedded ACP entity from NotesApp.
    let acp_entity = entity.read(cx).embedded_acp_chat.clone();
    let Some(acp_entity) = acp_entity else {
        tracing::warn!(event = "notes_acp_action_no_view", action_id = %action_id);
        return;
    };

    // Handle model switch.
    if let Some(model_id) = crate::actions::acp_switch_model_id_from_action(action_id) {
        acp_entity.update(cx, |chat, cx| {
            if let Some(thread) = chat.thread() {
                thread.update(cx, |thread, cx| {
                    thread.select_model(model_id, cx);
                });
            }
        });
        return;
    }

    match action_id {
        "acp_copy_last_response" => {
            let maybe_last = {
                let view = acp_entity.read(cx);
                view.thread().and_then(|thread| {
                    thread
                        .read(cx)
                        .messages
                        .iter()
                        .rev()
                        .find(|m| {
                            matches!(
                                m.role,
                                crate::ai::acp::thread::AcpThreadMessageRole::Assistant
                            )
                        })
                        .map(|m| m.body.to_string())
                })
            };
            if let Some(last_assistant) = maybe_last {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(last_assistant));
            }
        }
        "acp_export_markdown" => {
            let maybe_markdown = {
                let view = acp_entity.read(cx);
                view.thread().and_then(|thread| {
                    let messages = thread.read(cx).messages.clone();
                    crate::ai::acp::export::build_acp_conversation_markdown(&messages)
                })
            };
            if let Some(md) = maybe_markdown {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(md));
            }
        }
        "acp_retry_last" => {
            let last_user_msg = {
                let view = acp_entity.read(cx);
                view.thread().and_then(|thread| {
                    thread
                        .read(cx)
                        .messages
                        .iter()
                        .rev()
                        .find(|m| {
                            matches!(m.role, crate::ai::acp::thread::AcpThreadMessageRole::User)
                        })
                        .map(|m| m.body.to_string())
                })
            };
            if let Some(text) = last_user_msg {
                acp_entity.update(cx, |chat, cx| {
                    chat.live_thread().update(cx, |thread, cx| {
                        thread.set_input(text, cx);
                        let _ = thread.submit_input(cx);
                    });
                });
            }
        }
        "acp_new_conversation" => {
            acp_entity.update(cx, |chat, cx| {
                chat.live_thread().update(cx, |thread, cx| {
                    thread.clear_messages(cx);
                });
                chat.collapsed_ids.clear();
                cx.notify();
            });
        }
        "acp_clear_history" => {
            let kit = crate::setup::get_kit_path();
            let _ = std::fs::remove_file(kit.join("acp-history.jsonl"));
            let _ = std::fs::remove_dir_all(kit.join("acp-conversations"));
        }
        "acp_scroll_to_top" => {
            acp_entity.update(cx, |chat, cx| {
                chat.list_state.scroll_to(gpui::ListOffset {
                    item_ix: 0,
                    offset_in_item: px(0.),
                });
                cx.notify();
            });
        }
        "acp_scroll_to_bottom" => {
            acp_entity.update(cx, |chat, cx| {
                chat.list_state.scroll_to_end();
                cx.notify();
            });
        }
        "acp_expand_all" => {
            acp_entity.update(cx, |chat, cx| {
                let ids: Vec<u64> = chat
                    .live_thread()
                    .read(cx)
                    .messages
                    .iter()
                    .filter(|m| {
                        matches!(
                            m.role,
                            crate::ai::acp::thread::AcpThreadMessageRole::Thought
                                | crate::ai::acp::thread::AcpThreadMessageRole::Tool
                        )
                    })
                    .map(|m| m.id)
                    .collect();
                for id in ids {
                    chat.collapsed_ids.insert(id);
                }
                cx.notify();
            });
        }
        "acp_collapse_all" => {
            acp_entity.update(cx, |chat, cx| {
                chat.collapsed_ids.clear();
                cx.notify();
            });
        }
        other => {
            tracing::warn!(
                event = "notes_acp_action_unhandled",
                action = %other,
            );
        }
    }

    // Suppress unused-window warning — window is available but most ACP
    // actions dispatch through entity.update() which uses App context.
    let _ = window;
}

/// Close the Notes-hosted embedded ACP (from outside the NotesApp entity).
///
/// Switches the Notes window back to the editor surface.  No-op if
/// the Notes window is not open or is not in ACP mode.
pub fn close_notes_embedded_acp(cx: &mut gpui::App) -> anyhow::Result<()> {
    let (entity, _handle) = match super::get_notes_app_entity_and_handle() {
        Some(pair) => pair,
        None => return Ok(()),
    };

    entity.update(cx, |app: &mut NotesApp, cx| {
        if app.surface_mode == NotesSurfaceMode::Acp {
            if let Some(ref acp_entity) = app.embedded_acp_chat {
                acp_entity.update(cx, |chat, cx| {
                    chat.prepare_for_host_hide(cx);
                });
            }
            app.surface_mode = NotesSurfaceMode::Notes;
            app.pending_focus_surface = Some(focus::NotesFocusSurface::Editor);
            tracing::info!(event = "notes_acp_surface_closed_to_notes");
            cx.notify();
        }
    });

    Ok(())
}
