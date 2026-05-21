use super::source_classification::{build_tab_ai_apply_back_hint, detect_tab_ai_source_type};
use super::*;
use crate::ai::acp::AcpContextBootstrapState;

fn should_stage_focused_part_for_retry_draft_restore(has_retry_draft_state: bool) -> bool {
    // A retry-draft restore is authoritative: it already contains the user's
    // draft text, cursor, pending context parts, pasted-token metadata, typed
    // aliases, and inline-owned tokens. Do not mix in a freshly focused host
    // target during an agent-switch relaunch.
    !has_retry_draft_state
}

fn clear_stale_acp_ambient_bootstrap(
    thread_entity: &gpui::Entity<crate::ai::acp::AcpThread>,
    cx: &mut gpui::App,
) {
    thread_entity.update(cx, |thread, cx| {
        if thread.context_bootstrap_state() != AcpContextBootstrapState::Preparing {
            return;
        }
        if thread.pending_ambient_context_enabled() {
            thread.mark_context_bootstrap_failed(
                "Desktop context capture was interrupted. You can still send.",
                cx,
            );
        } else {
            thread.mark_context_bootstrap_ready(cx);
        }
    });
}

impl ScriptListApp {
    /// Stage synchronous context parts onto the ACP thread/view immediately
    /// after the view switch.
    ///
    /// Handles: retry-draft restore, slash priming, focused chip, ask-anything
    /// chip, ambient chip, and the immediate `mark_context_bootstrap_ready` /
    /// auto-submit path.
    ///
    /// Returns `true` when deferred (async) context staging is still needed
    /// (i.e. the ask-anything or explicit-ambient paths), `false` when all
    /// context is already committed synchronously.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn stage_acp_initial_context_parts(
        &mut self,
        retry_draft_state: Option<crate::ai::acp::view::AcpRetryDraftState>,
        view_entity_for_staging: &gpui::Entity<crate::ai::acp::AcpChatView>,
        thread: &gpui::Entity<crate::ai::acp::AcpThread>,
        focused_part: Option<crate::ai::message_parts::AiContextPart>,
        use_ask_anything_fallback: bool,
        explicit_ambient_chip_label: Option<String>,
        auto_submit: bool,
        pending_script_list_trigger: Option<char>,
        suppress_focused_part: bool,
        source_view: &AppView,
        cx: &mut Context<Self>,
    ) -> bool {
        let has_retry_draft_state = retry_draft_state.is_some();
        let should_stage_focused_part =
            should_stage_focused_part_for_retry_draft_restore(has_retry_draft_state);
        let focused_part = if should_stage_focused_part {
            focused_part
        } else {
            None
        };

        // Restore retry draft state (suppresses focused part, slash priming, and ask-anything).
        if let Some(draft_state) = retry_draft_state {
            view_entity_for_staging.update(cx, |view, cx| {
                view.restore_retry_draft_state(draft_state, cx);
            });
        }

        // Prime the slash command picker with /new-script when ACP opens
        // without auto-submit, explicit empty-composer intent, or context.
        if !has_retry_draft_state
            && Self::should_prime_script_authoring_slash(
                auto_submit,
                focused_part.is_some(),
                use_ask_anything_fallback,
                pending_script_list_trigger,
                suppress_focused_part,
            )
        {
            view_entity_for_staging.update(cx, |view, cx| {
                view.prime_slash_entry("new-script", cx);
            });
        }

        // --- Stage context part + insert inline @type:name token ---
        if let Some(part) = focused_part.clone() {
            let inline_token = crate::ai::context_mentions::part_to_inline_token(&part);
            let _ = thread.update(cx, |thread, cx| {
                if let Some(ref token) = inline_token {
                    let text = format!("{token} ");
                    thread.input.set_text(text.clone());
                    thread.input.set_cursor(text.len());
                }
                thread.add_context_part(part.clone(), cx);
            });
            if let Some(token) = inline_token {
                view_entity_for_staging.update(cx, |view, _cx| {
                    view.register_inline_owned_context_part(token, part);
                });
            }
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_focused_chip_staged_on_thread",
                source_view = match source_view {
                    AppView::ScriptList => "ScriptList",
                    _ => "Other",
                },
            );
        } else if use_ask_anything_fallback && explicit_ambient_chip_label.is_none() {
            // Stage a minimal desktop context resource as the Ask Anything inline token.
            let part = crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: crate::ai::message_parts::ASK_ANYTHING_RESOURCE_URI.to_string(),
                label: crate::ai::message_parts::ASK_ANYTHING_LABEL.to_string(),
            };
            let _ = thread.update(cx, |thread, cx| {
                thread.add_context_part(part, cx);
            });
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_ask_anything_chip_staged_on_thread",
                source_view = match source_view {
                    AppView::ScriptList => "ScriptList",
                    _ => "Other",
                },
            );
        } else if let Some(ref label) = explicit_ambient_chip_label {
            // Stage a labeled ambient capture inline token for explicit AI commands.
            let chip_label = label.clone();
            let part = crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: crate::ai::message_parts::ASK_ANYTHING_RESOURCE_URI.to_string(),
                label: chip_label,
            };
            let _ = thread.update(cx, |thread, cx| {
                thread.add_context_part(part, cx);
            });
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_ambient_capture_chip_staged_on_thread",
                source_view = match source_view {
                    AppView::ScriptList => "ScriptList",
                    _ => "Other",
                },
                chip_label = %label,
            );
        }

        // --- Focused-target path: mark bootstrap ready immediately ---
        // No deferred capture needed; the chip is already staged.
        if !use_ask_anything_fallback && explicit_ambient_chip_label.is_none() {
            let _ = thread.update(cx, |thread, cx| {
                thread.mark_context_bootstrap_ready(cx);
                // Auto-submit if effective intent was resolved (Shift+Tab path)
                if auto_submit {
                    if let Err(e) = thread.submit_input(cx) {
                        tracing::warn!(
                            event = "tab_ai_acp_focused_auto_submit_failed",
                            error = %e,
                        );
                    }
                }
            });
            return false;
        }

        // Deferred context staging is needed (ask-anything / explicit ambient).
        true
    }

    /// Spawn the async task that waits for the deferred desktop capture and
    /// injects the enriched context onto the ACP thread.
    ///
    /// Must be called only when `needs_deferred` is true (ask-anything /
    /// explicit-ambient paths). The view's capture-pending state must be set
    /// to `true` before calling this function.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn spawn_acp_deferred_context_staging(
        &mut self,
        view_entity_for_staging: gpui::Entity<crate::ai::acp::AcpChatView>,
        thread: gpui::Entity<crate::ai::acp::AcpThread>,
        request: TabAiLaunchRequest,
        capture_rx: TabAiDeferredCaptureRx,
        effective_intent: Option<String>,
        auto_submit: bool,
        open_started_at: std::time::Instant,
        cx: &mut Context<Self>,
    ) {
        let app_weak = cx.entity().downgrade();
        let view_weak_for_capture = view_entity_for_staging.downgrade();
        let thread_weak = thread.downgrade();
        let capture_gen = request.capture_generation;
        let effective_intent_for_capture = effective_intent;

        let first_paint_delay_ms = Self::ACP_CONTEXT_FIRST_PAINT_DELAY_MS;
        let first_paint_delay = std::time::Duration::from_millis(first_paint_delay_ms);

        cx.spawn(async move |_this, cx| {
            // Let the ACP chat render once before any heavy context staging runs.
            cx.background_executor().timer(first_paint_delay).await;
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_context_stage",
                stage = "first_paint_gate_released",
                stage_ms = first_paint_delay_ms,
                total_ms = open_started_at.elapsed().as_millis() as u64,
            );

            // Wait for deferred capture (bounded — OS providers can hang without a timeout).
            let capture_wait_started_at = std::time::Instant::now();
            let capture_timeout =
                std::time::Duration::from_secs(ScriptListApp::TAB_AI_DEFERRED_CAPTURE_TIMEOUT_SEC);
            let capture_result = loop {
                match capture_rx.try_recv() {
                    Ok(result) => break result,
                    Err(async_channel::TryRecvError::Closed) => {
                        break Err("deferred capture channel closed".to_string());
                    }
                    Err(async_channel::TryRecvError::Empty) => {
                        if capture_wait_started_at.elapsed() >= capture_timeout {
                            tracing::warn!(
                                target: "script_kit::tab_ai",
                                event = "tab_ai_acp_deferred_capture_timeout",
                                timeout_ms = capture_timeout.as_millis() as u64,
                            );
                            break Err("deferred capture timed out".to_string());
                        }
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(100))
                            .await;
                    }
                }
            };
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_context_stage",
                stage = "capture_rx_recv",
                stage_ms = capture_wait_started_at.elapsed().as_millis() as u64,
                total_ms = open_started_at.elapsed().as_millis() as u64,
            );

            let artifacts = match capture_result {
                Ok(a) => a,
                Err(e) => {
                    tracing::warn!(
                        event = "tab_ai_acp_deferred_capture_failed",
                        error = %e,
                    );
                    TabAiDeferredCaptureArtifacts::default()
                }
            };

            // Apply the captured context
            let _ = cx.update(|cx| {
                let Some(app) = app_weak.upgrade() else {
                    return;
                };
                let Some(thread_entity) = thread_weak.upgrade() else {
                    return;
                };

                // Clear capture-pending on the ACP view so the footer dot calms.
                if let Some(view_entity) = view_weak_for_capture.upgrade() {
                    view_entity.update(cx, |view, _cx| {
                        view.set_context_capture_pending(false);
                    });
                }

                app.update(cx, |this, cx| {
                    // Stale generation check
                    if this.tab_ai_harness_capture_generation != capture_gen {
                        tracing::debug!(
                            event = "tab_ai_acp_deferred_capture_stale",
                            expected = capture_gen,
                            current = this.tab_ai_harness_capture_generation,
                        );
                        clear_stale_acp_ambient_bootstrap(&thread_entity, cx);
                        return;
                    }

                    let context_stage_started_at = std::time::Instant::now();
                    let resolved = this.build_tab_ai_context_from(
                        effective_intent_for_capture.clone().unwrap_or_default(),
                        request.source_view.clone(),
                        request.ui_snapshot.clone(),
                        artifacts.desktop,
                        request.invocation_receipt.clone(),
                        cx,
                    );
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_context_stage",
                        stage = "build_tab_ai_context_from",
                        stage_ms = context_stage_started_at.elapsed().as_millis() as u64,
                        total_ms = open_started_at.elapsed().as_millis() as u64,
                    );

                    let source_type = detect_tab_ai_source_type(
                        &request.source_view,
                        &resolved.context.desktop,
                        resolved.context.focused_target.as_ref(),
                    );
                    let apply_back_hint = build_tab_ai_apply_back_hint(source_type.as_ref());

                    // Persist the apply-back route
                    this.tab_ai_harness_apply_back_route = source_type
                        .clone()
                        .zip(apply_back_hint.clone())
                        .map(|(source_type, hint)| crate::ai::TabAiApplyBackRoute {
                            source_type,
                            hint,
                            focused_target: resolved.context.focused_target.clone(),
                        });

                    let context = resolved.context.with_deferred_capture_fields(
                        source_type,
                        artifacts.screenshot_path,
                        apply_back_hint,
                    );

                    // Stage context on the AcpThread
                    let stage_context_started_at = std::time::Instant::now();
                    let _ = thread_entity.update(cx, |thread, cx| {
                        if let Err(e) = thread.stage_ask_anything_context(&context, cx) {
                            tracing::warn!(
                                event = "tab_ai_acp_stage_context_failed",
                                error = %e,
                            );
                            thread.mark_context_bootstrap_failed(
                                "Some desktop context could not be attached. You can still send.",
                                cx,
                            );
                        }

                        // Auto-submit if effective intent was resolved
                        // (Shift+Tab path or quick-submit plan)
                        if auto_submit {
                            if let Err(e) = thread.submit_input(cx) {
                                tracing::warn!(
                                    event = "tab_ai_acp_auto_submit_failed",
                                    error = %e,
                                );
                            }
                        }
                    });
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_context_stage",
                        stage = "thread_stage_context",
                        stage_ms = stage_context_started_at.elapsed().as_millis() as u64,
                        total_ms = open_started_at.elapsed().as_millis() as u64,
                    );
                });
            });
        })
        .detach();
    }

    pub(super) fn should_prime_script_authoring_slash(
        auto_submit: bool,
        has_focused_part: bool,
        use_ask_anything_fallback: bool,
        pending_script_list_trigger: Option<char>,
        suppress_focused_part: bool,
    ) -> bool {
        !auto_submit
            && !has_focused_part
            && !use_ask_anything_fallback
            && !suppress_focused_part
            && !matches!(pending_script_list_trigger, Some('/' | '@'))
    }
}
