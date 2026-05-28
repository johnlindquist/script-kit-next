//! Notes-hosted ACP surface helpers.
//!
//! Manages the embedded ACP chat lifecycle inside the Notes window:
//! open/reuse, switch back to Notes editor, toggle actions dialog,
//! and dispatch Notes-specific ACP actions.

use super::*;

impl NotesApp {
    pub(super) fn ensure_embedded_acp_view(
        &mut self,
        initial_input: Option<String>,
        cx: &mut Context<Self>,
    ) -> Result<gpui::Entity<crate::ai::agent_chat::ui::AgentChatView>, String> {
        if let Some(entity) = self.embedded_acp_chat.as_ref().cloned() {
            tracing::info!(
                event = "notes_embedded_acp_view_ensured",
                created = false,
                has_input = initial_input.as_ref().is_some_and(|s| !s.trim().is_empty()),
            );
            return Ok(entity);
        }

        let requirements = crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default();
        let view =
            crate::ai::acp::hosted::spawn_hosted_view(initial_input.clone(), requirements, cx)?;
        self.wire_acp_host_callbacks(&view, cx);
        self.embedded_acp_chat = Some(view.clone());
        self.notes_acp_generation = self.notes_acp_generation.wrapping_add(1);

        tracing::info!(
            event = "notes_embedded_acp_view_ensured",
            created = true,
            has_input = initial_input.as_ref().is_some_and(|s| !s.trim().is_empty()),
        );

        Ok(view)
    }

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

        let _view = self.ensure_embedded_acp_view(initial_input, cx)?;

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
        self.close_embedded_acp_via_host("switch_to_notes_surface", Some(window), cx);
    }

    fn close_embedded_acp_via_host(
        &mut self,
        reason: &'static str,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        if let Some(ref entity) = self.embedded_acp_chat {
            entity.update(cx, |chat, cx| {
                chat.prepare_for_host_hide(cx);
            });
        }

        if crate::actions::is_actions_window_open() {
            crate::actions::close_actions_window(cx);
        }

        self.surface_mode = NotesSurfaceMode::Notes;
        if let Some(window) = window {
            self.request_focus_surface(focus::NotesFocusSurface::Editor, window, cx);
        } else {
            self.pending_focus_surface = Some(focus::NotesFocusSurface::Editor);
            cx.notify();
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_embedded_acp_closed_via_host",
            reason,
        );
    }

    pub(super) fn prepare_embedded_acp_for_window_close(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        if let Some(ref entity) = self.embedded_acp_chat {
            entity.update(cx, |chat, cx| {
                chat.prepare_for_host_hide(cx);
            });
        }

        if crate::actions::is_actions_window_open() {
            crate::actions::close_actions_window(cx);
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_embedded_acp_prepared_for_window_close",
            reason,
        );
    }

    pub(super) fn clear_notes_hosted_acp_context_for_note(
        &mut self,
        note_id: Option<NoteId>,
        cx: &mut Context<Self>,
    ) {
        let Some(note_id) = note_id else {
            return;
        };
        if let Some(ref entity) = self.embedded_acp_chat {
            entity.update(cx, |chat, cx| {
                chat.clear_hosted_context_parts_from_host("notes_note_switch_detach", cx);
            });
        }
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_hosted_acp_context_cleared_for_note",
            note_id = %note_id,
        );
    }

    fn close_notes_acp_actions_via_host(
        &mut self,
        reason: &'static str,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        // When this runs from the dialog's on_close callback, the actions window
        // is already in its close path. Only actively close it from call sites
        // that still own a live parent window handle.
        if window.is_some() && crate::actions::is_actions_window_open() {
            crate::actions::close_actions_window(cx);
        }

        if let Some(window) = window {
            self.request_focus_surface(focus::NotesFocusSurface::AcpChat, window, cx);
        } else if self.surface_mode == NotesSurfaceMode::Acp {
            self.pending_focus_surface = Some(focus::NotesFocusSurface::AcpChat);
            cx.notify();
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_acp_actions_closed_via_host",
            reason,
        );
    }

    /// Open the ACP history popup anchored to the Notes window.
    ///
    /// Returns `true` if the popup was opened, `false` if no embedded ACP
    /// view exists.
    pub(super) fn open_embedded_acp_history_popup(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(entity) = self.embedded_acp_chat.as_ref().cloned() else {
            tracing::info!(event = "notes_acp_history_popup_requested", opened = false);
            return false;
        };

        let parent_handle = window.window_handle();
        let parent_bounds = window.bounds();
        let display_id = window.display(cx).map(|display| display.id());

        entity.update(cx, |view, cx| {
            view.open_history_popup_from_host(parent_handle, parent_bounds, display_id, cx);
        });

        tracing::info!(event = "notes_acp_history_popup_requested", opened = true);
        true
    }

    /// Handle a portal open request from the embedded ACP context picker.
    ///
    /// Static helper so it can be called from the `set_on_open_portal` closure
    /// without holding an immutable borrow on `NotesApp` while also needing
    /// `&mut App`.
    ///
    /// `AcpHistory` is supported in Notes because ACP already stages the query.
    /// `FileSearch` and `ClipboardHistory` are filtered from the picker and
    /// guarded at the accept path by `allowed_portal_kinds`, so they should
    /// not reach here in normal operation. The branches remain as
    /// defense-in-depth logging.
    fn handle_acp_portal_static(
        chat: Option<Entity<crate::ai::agent_chat::ui::AgentChatView>>,
        kind: crate::ai::window::context_picker::types::PortalKind,
        cx: &mut gpui::App,
    ) {
        use crate::ai::window::context_picker::types::PortalKind;

        let Some(chat) = chat else {
            tracing::info!(
                event = "notes_acp_portal_requested",
                kind = ?kind,
                opened = false,
                reason = "no_embedded_acp_view",
            );
            return;
        };

        match kind {
            PortalKind::AcpHistory => {
                let query = chat.update(cx, |view, _cx| {
                    view.take_pending_history_portal_query().unwrap_or_default()
                });
                tracing::info!(
                    target: "script_kit::acp",
                    event = "notes_acp_history_portal_query_seeded_from_contract",
                    query = %query,
                );

                let hits = crate::ai::acp::history::search_history(&query, 12);

                tracing::info!(
                    event = "notes_acp_portal_requested",
                    kind = "AcpHistory",
                    opened = true,
                    query = %query,
                    hit_count = hits.len(),
                );

                let opened = chat.update(cx, |view, cx| {
                    view.open_history_portal_with_entries(query, hits, cx)
                });
                if !opened {
                    chat.update(cx, |view, cx| {
                        let _ = view.cancel_pending_portal_session(PortalKind::AcpHistory, cx);
                    });
                }
            }
            PortalKind::FileSearch
            | PortalKind::BrowserHistory
            | PortalKind::DictationHistory
            | PortalKind::ScriptSearch
            | PortalKind::ScriptletSearch
            | PortalKind::SkillSearch
            | PortalKind::NotesBrowse => {
                tracing::info!(
                    event = "notes_acp_portal_requested",
                    kind = ?kind,
                    opened = false,
                    reason = "unsupported_in_notes_host",
                );
                chat.update(cx, |view, cx| {
                    let _ = view.cancel_pending_portal_session(kind, cx);
                });
            }
            PortalKind::ClipboardHistory => {
                tracing::info!(
                    event = "notes_acp_portal_requested",
                    kind = "ClipboardHistory",
                    opened = false,
                    reason = "unsupported_in_notes_host",
                );
                chat.update(cx, |view, cx| {
                    let _ = view.cancel_pending_portal_session(PortalKind::ClipboardHistory, cx);
                });
            }
        }
    }

    /// Wire ACP host callbacks (toggle-actions, close, history, portals)
    /// to Notes-owned handlers.
    fn wire_acp_host_callbacks(
        &self,
        view: &Entity<crate::ai::agent_chat::ui::AgentChatView>,
        cx: &mut Context<Self>,
    ) {
        let notes_entity = cx.entity().downgrade();

        // Restrict portal kinds to those Notes can actually host.
        // Notes owns @history locally; @file and @clipboard require
        // main-panel view switching that Notes cannot provide.
        view.update(cx, |chat, _cx| {
            chat.set_footer_host(crate::ai::acp::view::AcpFooterHost::External);
            chat.set_allowed_portal_kinds(vec![
                crate::ai::window::context_picker::types::PortalKind::AcpHistory,
            ]);
        });

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
        let close_entity = notes_entity.clone();
        view.update(cx, |chat, _cx| {
            chat.set_on_close_requested(move |window, cx| {
                if let Some(entity) = close_entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.close_embedded_acp_via_host("acp_close_requested", Some(window), cx);
                    });
                }
            });
        });

        // History command (Cmd+P): open Notes-anchored history popup.
        let history_entity = notes_entity.clone();
        view.update(cx, |chat, _cx| {
            chat.set_on_open_history_command(move |window, cx| {
                if let Some(entity) = history_entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        let _ = app.open_embedded_acp_history_popup(window, cx);
                    });
                }
            });
        });

        // Portal requests: only @history reaches here because the view's
        // allowed_portal_kinds filters @file and @clipboard from the picker.
        // The handler still logs rejected kinds as defense-in-depth.
        let portal_view = view.downgrade();
        view.update(cx, |chat, _cx| {
            chat.set_on_open_portal(move |kind, cx| {
                if let Some(chat) = portal_view.upgrade() {
                    Self::handle_acp_portal_static(Some(chat), kind, cx);
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
            self.close_notes_acp_actions_via_host("toggle_existing_window", Some(window), cx);
            return;
        }

        let Some(ref acp_view) = self.embedded_acp_chat else {
            return;
        };
        let actions_target = acp_view.downgrade();
        let actions_generation = self.notes_acp_generation;

        if let Some(thread) = acp_view.read(cx).thread() {
            thread.update(cx, |thread, cx| thread.refresh_models(cx));
        }

        // Read ACP model context from the cached view.
        let (selected_model_id, available_models) = {
            let view = acp_view.read(cx);
            match &view.session {
                crate::ai::agent_chat::ui::AgentChatSession::Setup(_) => (None, Vec::new()),
                crate::ai::agent_chat::ui::AgentChatSession::Live(thread) => {
                    let thread = thread.read(cx);
                    (
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
                    available_models: &available_models,
                    selected_model_id: selected_model_id.as_deref(),
                    focused_text: false,
                    focused_text_expanded: false,
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
            let close_entity = notes_entity.clone();
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
                if let Some(entity) = close_entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.close_notes_acp_actions_via_host("dialog_on_close", None, cx);
                    });
                }
            }));
        });

        let parent_window_handle = window.window_handle();
        let bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());
        let parent_automation_id = Some("notes".to_string());

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
                let _ = cx.update(|window, app_cx| {
                    if action_id == "__cancel__" {
                        // ActionsWindow request_close already ran the dialog on_close
                        // callback, so the host focus restore has happened.
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "notes_acp_action_cancel_consumed_after_on_close",
                        );
                        return;
                    }
                    dispatch_notes_acp_action(
                        this.upgrade(),
                        actions_target.clone(),
                        actions_generation,
                        &action_id,
                        window,
                        app_cx,
                    );
                });
            }
        })
        .detach();
    }

    /// Relaunch the cached Notes-hosted ACP surface with fresh session state.
    ///
    /// Use this for explicit note → ACP switches so the user does not land
    /// inside an unrelated prior conversation.
    pub(super) fn relaunch_embedded_acp(
        &mut self,
        initial_input: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let had_cached_view = self.embedded_acp_chat.is_some();

        if let Some(ref entity) = self.embedded_acp_chat {
            entity.update(cx, |chat, cx| {
                chat.prepare_for_host_hide(cx);
            });
        }
        self.embedded_acp_chat = None;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_acp_surface_relaunch_requested",
            had_cached_view,
            has_input = initial_input.as_ref().is_some_and(|s| !s.trim().is_empty()),
        );

        let result = self.open_or_focus_embedded_acp(initial_input, window, cx);
        match &result {
            Ok(()) => tracing::info!(
                target: "script_kit::tab_ai",
                event = "notes_acp_surface_relaunch_completed",
            ),
            Err(error) => tracing::warn!(
                target: "script_kit::tab_ai",
                event = "notes_acp_surface_relaunch_failed",
                error = %error,
            ),
        }
        result
    }

    /// Accessor for the current surface mode.
    pub(crate) fn surface_mode(&self) -> NotesSurfaceMode {
        self.surface_mode
    }

    pub(crate) fn embedded_acp_chat_entity(
        &self,
    ) -> Option<Entity<crate::ai::agent_chat::ui::AgentChatView>> {
        self.embedded_acp_chat.clone()
    }
}

/// Dispatch an ACP action from the Notes-hosted actions dialog popup.
///
/// Called from the async spawn inside `toggle_acp_actions`.  Receives
/// `&mut Window` and `&mut App` from `AsyncWindowContext::update`, so it
/// can interact with the ACP view entity and the Notes host state.
fn dispatch_notes_acp_action(
    entity: Option<Entity<NotesApp>>,
    acp_target: gpui::WeakEntity<crate::ai::agent_chat::ui::AgentChatView>,
    acp_generation: u64,
    action_id: &str,
    window: &mut Window,
    cx: &mut gpui::App,
) {
    let Some(entity) = entity else { return };
    let Some(acp_entity) = acp_target.upgrade() else {
        tracing::warn!(
            target: "script_kit::tab_ai",
            event = "notes_acp_action_stale_view",
            action_id = %action_id,
            reason = "target_dropped",
        );
        return;
    };
    if entity.read(cx).notes_acp_generation != acp_generation {
        tracing::warn!(
            target: "script_kit::tab_ai",
            event = "notes_acp_action_stale_view",
            action_id = %action_id,
            reason = "generation_mismatch",
            expected_generation = acp_generation,
            actual_generation = entity.read(cx).notes_acp_generation,
        );
        return;
    }

    tracing::info!(
        event = "notes_acp_action_dispatched",
        action_id = %action_id,
    );

    // For `acp_show_history`, open the Notes-anchored history popup.
    if action_id == "acp_show_history" {
        let opened = entity.update(cx, |app: &mut NotesApp, cx| {
            app.open_embedded_acp_history_popup(window, cx)
        });
        tracing::info!(event = "notes_acp_action_show_history", opened);
        return;
    }

    // For `acp_close`, route to the Notes host to switch surfaces.
    if action_id == "acp_close" {
        entity.update(cx, |app: &mut NotesApp, cx| {
            app.close_embedded_acp_via_host("acp_action_close", Some(window), cx);
        });
        return;
    }

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
                    crate::ai::acp::export::build_acp_conversation_markdown_from_thread(
                        thread.read(cx),
                    )
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
                if let Some(transcript) = &chat.transcript {
                    transcript.update(cx, |t, cx| t.clear_collapsed_ids(cx));
                }
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
                if let Some(transcript) = &chat.transcript {
                    transcript.read(cx).scroll_to(gpui::ListOffset {
                        item_ix: 0,
                        offset_in_item: px(0.),
                    });
                }
                cx.notify();
            });
        }
        "acp_scroll_to_bottom" => {
            acp_entity.update(cx, |chat, cx| {
                if let Some(transcript) = &chat.transcript {
                    transcript.read(cx).scroll_to_end();
                }
                cx.notify();
            });
        }
        "acp_expand_all" => {
            acp_entity.update(cx, |chat, cx| {
                if let Some(transcript) = &chat.transcript {
                    transcript.update(cx, |t, cx| t.clear_collapsed_ids(cx));
                }
                cx.notify();
            });
        }
        "acp_collapse_all" => {
            acp_entity.update(cx, |chat, cx| {
                if let Some(transcript) = &chat.transcript {
                    transcript.update(cx, |t, cx| t.clear_collapsed_ids(cx));
                }
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
            app.close_embedded_acp_via_host("external_close", None, cx);
        }
    });

    Ok(())
}
