use super::*;

impl ScriptListApp {
    /// **Contract:** `AppView::AcpChatView` and `cx.notify()` happen
    /// *before* any deferred-capture await. The user sees the chat surface
    /// within one frame.
    pub(super) fn open_tab_ai_acp_view_from_request_impl(
        &mut self,
        request: TabAiLaunchRequest,
        capture_rx: TabAiDeferredCaptureRx,
        focused_part: Option<crate::ai::message_parts::AiContextPart>,
        use_ask_anything_fallback: bool,
        explicit_ambient_chip_label: Option<String>,
        force_acp_surface: bool,
        cx: &mut Context<Self>,
    ) {
        let open_started_at = std::time::Instant::now();
        let source_view = request.source_view.clone();
        let had_harness_session = self.tab_ai_harness.is_some();
        let pending_script_list_trigger = self.tab_ai_harness_script_list_trigger;

        // Compute canonical effective intent once, matching PTY path's normalization.
        let effective_intent = Self::tab_ai_effective_submission_intent(&request);
        let auto_submit = effective_intent.is_some();

        // Build ACP initial input via the shared helper, ensuring the same
        // verification contract as the PTY submission path.
        // When force_acp_surface is set (Auto Submit fallback), use the raw
        // intent without new-script guidance — the query may be general.
        let acp_initial_input = effective_intent
            .clone()
            .map(|intent| {
                if force_acp_surface {
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "tab_ai_acp_initial_input_built",
                        prompt_type = %request.ui_snapshot.prompt_type,
                        guidance_appended = false,
                        forced_by_script_list_submit = false,
                        force_acp_surface = true,
                    );
                    intent
                } else {
                    let initial_input = crate::ai::harness::build_tab_ai_acp_initial_input_for_prompt(
                        &request.ui_snapshot.prompt_type,
                        &intent,
                    );

                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "tab_ai_acp_initial_input_built",
                        prompt_type = %request.ui_snapshot.prompt_type,
                        guidance_appended = initial_input.guidance_appended,
                        forced_by_script_list_submit = initial_input.forced_by_script_list_submit,
                        includes_script_authoring_skill = initial_input.includes_script_authoring_skill,
                        includes_bun_build_verification = initial_input.includes_bun_build_verification,
                        includes_bun_execute_verification = initial_input.includes_bun_execute_verification,
                    );

                    initial_input.text
                }
            })
            .or_else(|| {
                Self::tab_ai_acp_initial_input_for_launch(
                    &request.ui_snapshot.prompt_type,
                    None,
                    pending_script_list_trigger,
                    force_acp_surface,
                )
            });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_begin",
            acp_chat_ui_variant = request.ui_variant.state_id(),
            auto_submit,
            has_entry_intent = request.entry_intent.is_some(),
            had_harness_session,
            pending_script_list_trigger = ?pending_script_list_trigger,
            prefilled_len = acp_initial_input.as_ref().map(|text| text.len()).unwrap_or(0),
        );

        // --- Permission broker + ACP connection ---
        let stage_started_at = std::time::Instant::now();
        let (broker, permission_rx) = crate::ai::acp::AcpPermissionBroker::new();
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_stage",
            stage = "permission_broker_new",
            stage_ms = stage_started_at.elapsed().as_millis() as u64,
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        // --- ACP agent catalog + preflight resolution ---
        let stage_started_at = std::time::Instant::now();
        let catalog = match crate::ai::acp::load_acp_agent_catalog_entries() {
            Ok(entries) => entries,
            Err(error) => {
                self.show_acp_catalog_load_failed_setup_view(source_view, error.to_string(), cx);
                return;
            }
        };

        // Check for an explicit retry payload from a setup card before
        // falling back to persisted preference and entry-path derivation.
        let retry_request = self.take_acp_retry_request_for_open(cx);
        let retry_draft_state = retry_request
            .as_ref()
            .and_then(|request| request.draft_state.clone());

        let focused_part = if retry_draft_state.is_some() {
            None
        } else {
            focused_part
        };

        let use_ask_anything_fallback = if retry_draft_state.is_some() {
            false
        } else {
            use_ask_anything_fallback
        };

        let preferred_agent_id = retry_request
            .as_ref()
            .and_then(|req| req.preferred_agent_id.clone())
            .or_else(crate::ai::acp::load_preferred_acp_agent_id);

        let requirements = retry_request
            .as_ref()
            .map(|req| req.launch_requirements)
            .unwrap_or_else(|| {
                let has_context_parts = focused_part.is_some();
                let needs_image = focused_part
                    .as_ref()
                    .map(|part| part.source().contains("screenshot=1"))
                    .unwrap_or(false);
                crate::ai::acp::AcpLaunchRequirements {
                    needs_embedded_context: has_context_parts,
                    needs_image,
                }
            });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_retry_request_consumed",
            had_retry_request = retry_request.is_some(),
            had_retry_draft_state = retry_draft_state.is_some(),
            preferred_agent_id = ?preferred_agent_id,
            needs_embedded_context = requirements.needs_embedded_context,
            needs_image = requirements.needs_image,
        );

        let acp_launch_resolution = if retry_request.is_some() {
            crate::ai::acp::resolve_explicit_acp_launch_with_requirements(
                &catalog,
                preferred_agent_id.as_deref(),
                requirements,
            )
        } else {
            crate::ai::acp::resolve_acp_launch_with_requirements(
                &catalog,
                preferred_agent_id.as_deref(),
                requirements,
            )
        };
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_launch_resolution",
            preferred_agent_id = ?preferred_agent_id,
            selected_agent_id = ?acp_launch_resolution.selected_agent_id(),
            blocker = ?acp_launch_resolution.blocker,
            stage_ms = stage_started_at.elapsed().as_millis() as u64,
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        if !acp_launch_resolution.is_ready() {
            self.show_acp_launch_blocked_setup_view(
                source_view,
                &acp_launch_resolution,
                requirements,
                cx,
            );
            return;
        }

        let agent = match acp_launch_resolution
            .selected_agent
            .as_ref()
            .and_then(|entry| entry.config.clone())
        {
            Some(config) => config,
            None => {
                tracing::error!(
                    target: "script_kit::tab_ai",
                    event = "acp_resolution_missing_config",
                );
                cx.notify();
                return;
            }
        };
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_stage",
            stage = "acp_agent_resolved",
            agent_id = %agent.id,
            stage_ms = stage_started_at.elapsed().as_millis() as u64,
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        let agent_display_name = agent.display_name().to_string();
        // Extract model info before `agent` is moved into spawn_with_approval.
        let agent_models = agent.models.clone();
        // Use the config-backed preferred model, falling back to the first model.
        let persisted_model = crate::config::load_user_preferences().ai.selected_model_id;
        let default_model_id = persisted_model
            .filter(|id| agent_models.iter().any(|m| m.id == *id))
            .or_else(|| agent_models.first().map(|m| m.id.clone()));

        let selected_agent_id_for_prewarm = acp_launch_resolution
            .selected_agent_id()
            .map(str::to_string);
        let prewarmed_acp_chat = self.take_prewarmed_acp_chat_for_launch(
            selected_agent_id_for_prewarm.as_deref(),
            requirements,
            retry_request.is_some(),
            cx,
        );

        let (thread, view_entity, used_prewarmed_acp) =
            if let Some((view_entity, thread)) = prewarmed_acp_chat {
                view_entity.update(cx, |view, cx| {
                    view.set_ui_variant(request.ui_variant, cx);
                });
                if let Some(initial_input) = acp_initial_input.clone() {
                    thread.update(cx, |thread, cx| {
                        thread.set_input(initial_input, cx);
                    });
                }

                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_open_stage",
                    stage = "acp_hot_prewarm_reused",
                    acp_chat_ui_variant = request.ui_variant.state_id(),
                    used_prewarmed_acp = true,
                    total_ms = open_started_at.elapsed().as_millis() as u64,
                );
                (thread, view_entity, true)
            } else {
                let stage_started_at = std::time::Instant::now();
                let connection = match crate::ai::acp::AcpConnection::spawn_with_approval(
                    agent,
                    Some(broker.approval_fn()),
                ) {
                    Ok(conn) => std::sync::Arc::new(conn),
                    Err(error) => {
                        tracing::error!(
                            event = "tab_ai_acp_spawn_failed",
                            error = %error,
                        );
                        self.toast_manager.push(
                            crate::components::toast::Toast::error(
                                format!("Failed to start ACP connection: {error}"),
                                &self.theme,
                            )
                            .duration_ms(Some(TOAST_ERROR_MS)),
                        );
                        cx.notify();
                        return;
                    }
                };
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_open_stage",
                    stage = "acp_connection_spawn_with_approval",
                    used_prewarmed_acp = false,
                    stage_ms = stage_started_at.elapsed().as_millis() as u64,
                    total_ms = open_started_at.elapsed().as_millis() as u64,
                );

                // Use ~/.scriptkit as cwd so Claude Code discovers CLAUDE.md and skills/.
                let cwd = crate::setup::get_kit_path();

                let stage_started_at = std::time::Instant::now();
                let thread = cx.new(|cx| {
                    crate::ai::acp::AcpThread::new(
                        connection,
                        permission_rx,
                        crate::ai::acp::AcpThreadInit {
                            ui_thread_id: uuid::Uuid::new_v4().to_string(),
                            cwd,
                            initial_input: acp_initial_input.clone(),
                            initial_context_parts: Vec::new(),
                            display_name: agent_display_name.into(),
                            selected_agent: acp_launch_resolution.selected_agent.clone(),
                            available_agents: acp_launch_resolution.catalog_entries.clone(),
                            launch_requirements: requirements,
                            available_models: agent_models,
                            selected_model_id: default_model_id,
                        },
                        cx,
                    )
                });
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_open_stage",
                    stage = "acp_thread_new",
                    used_prewarmed_acp = false,
                    stage_ms = stage_started_at.elapsed().as_millis() as u64,
                    total_ms = open_started_at.elapsed().as_millis() as u64,
                );

                let stage_started_at = std::time::Instant::now();
                let view_entity = cx.new(|cx| {
                    crate::ai::acp::AcpChatView::new(thread.clone(), cx)
                        .with_ui_variant(request.ui_variant)
                });
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_open_stage",
                    stage = "acp_chat_view_new",
                    acp_chat_ui_variant = request.ui_variant.state_id(),
                    used_prewarmed_acp = false,
                    stage_ms = stage_started_at.elapsed().as_millis() as u64,
                    total_ms = open_started_at.elapsed().as_millis() as u64,
                );

                (thread, view_entity, false)
            };

        // Only persist the selected agent when the launch is explicit (retry
        // request / first-run) or already aligned with the saved preference.
        // Automatic capability fallback should not silently rewrite the user's
        // preferred agent.
        let selected_agent_id = acp_launch_resolution
            .selected_agent_id()
            .map(str::to_string);
        let implicit_codex_default_active = retry_request.is_none()
            && preferred_agent_id.is_none()
            && selected_agent_id.as_deref() == Some(crate::ai::acp::config::CODEX_ACP_AGENT_ID)
            && crate::ai::acp::config::codex_acp_default_probe_state()
                .should_be_implicit_codex_default;
        let should_persist_selected_agent = retry_request.is_some()
            || (preferred_agent_id.is_none() && !implicit_codex_default_active)
            || preferred_agent_id.as_deref() == selected_agent_id.as_deref();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_preferred_agent_post_launch_persist_decision",
            had_retry_request = retry_request.is_some(),
            preferred_agent_id = ?preferred_agent_id,
            selected_agent_id = ?selected_agent_id,
            implicit_codex_default_active,
            blocker = ?acp_launch_resolution.blocker,
            should_persist_selected_agent,
            needs_embedded_context = requirements.needs_embedded_context,
            needs_image = requirements.needs_image,
        );

        if should_persist_selected_agent {
            crate::ai::acp::persist_preferred_acp_agent_id(selected_agent_id);
        } else {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_preferred_agent_preserved_during_fallback_launch",
                preferred_agent_id = ?preferred_agent_id,
                selected_agent_id = ?acp_launch_resolution.selected_agent_id(),
            );
        }

        self.wire_embedded_acp_footer_callbacks(&view_entity, cx);
        self.embedded_acp_chat = Some(view_entity.clone());
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_stage",
            stage = "acp_chat_view_ready",
            used_prewarmed_acp,
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        // Save originating surface for close/restore
        self.tab_ai_harness_return_view = Some(source_view.clone());
        self.tab_ai_harness_return_focus_target = Some(self.tab_ai_return_focus_target());

        // Seed apply-back route synchronously so focused-chip ACP sessions
        // retain the concrete target metadata even though they skip deferred
        // context capture.
        self.seed_tab_ai_apply_back_route(
            &request.source_view,
            &request.ui_snapshot,
            focused_part.as_ref(),
        );

        // --- View switch FIRST: user sees the ACP chat surface immediately ---
        let view_entity_for_staging = view_entity.clone();
        view_entity_for_staging.update(cx, |view, _cx| {
            view.opened_via_transient_trigger = pending_script_list_trigger;
        });
        self.enter_embedded_acp_chat_surface(view_entity, cx);
        cx.notify();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_view_switched",
            elapsed_ms = open_started_at.elapsed().as_millis() as u64,
        );

        let needs_deferred = self.stage_acp_initial_context_parts(
            retry_draft_state,
            &view_entity_for_staging,
            &thread,
            focused_part,
            use_ask_anything_fallback,
            explicit_ambient_chip_label.clone(),
            auto_submit,
            pending_script_list_trigger,
            request.suppress_focused_part,
            &source_view,
            cx,
        );

        self.schedule_acp_post_paint_harness_teardown(had_harness_session, open_started_at, cx);

        if !needs_deferred {
            return;
        }

        // --- Ask Anything fallback: spawn deferred context injection task ---
        // Mark capture as pending so the footer loading dot activates.
        view_entity_for_staging.update(cx, |view, _cx| {
            view.set_context_capture_pending(true);
        });

        self.spawn_acp_deferred_context_staging(
            view_entity_for_staging,
            thread,
            request,
            capture_rx,
            effective_intent,
            auto_submit,
            open_started_at,
            cx,
        );
    }

    /// Defer harness termination to after first paint so the user sees the
    /// chat surface before the synchronous teardown blocks the main thread.
    fn schedule_acp_post_paint_harness_teardown(
        &mut self,
        had_harness_session: bool,
        open_started_at: std::time::Instant,
        cx: &mut Context<Self>,
    ) {
        if !had_harness_session {
            return;
        }
        let app_weak_for_teardown = cx.entity().downgrade();
        let open_started_at_for_teardown = open_started_at;
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(16))
                .await;
            let _ = cx.update(|cx| {
                let Some(app) = app_weak_for_teardown.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| {
                    let stage_started_at = std::time::Instant::now();
                    this.terminate_tab_ai_harness_session(cx);
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_open_stage",
                        stage = "terminate_tab_ai_harness_session_post_paint",
                        stage_ms = stage_started_at.elapsed().as_millis() as u64,
                        total_ms = open_started_at_for_teardown.elapsed().as_millis() as u64,
                    );
                });
            });
        })
        .detach();
    }

    /// Extract a pending retry request from the current ACP chat view.
    ///
    /// Returns `None` if the current view is not an `AcpChatView` or if no
    /// retry request has been queued.
    pub(super) fn take_acp_retry_request_for_open(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<crate::ai::acp::AcpRetryRequest> {
        if let AppView::AcpChatView { entity } = &self.current_view {
            return entity.update(cx, |view, _cx| view.take_retry_request());
        }

        self.embedded_acp_chat
            .as_ref()
            .cloned()
            .and_then(|entity| entity.update(cx, |view, _cx| view.take_retry_request()))
    }

    /// Build the ACP composer text for the first render of a new launch.
    ///
    /// ScriptList-triggered `@` and `/` routes prefill the raw trigger so the
    /// ACP handoff never paints an empty composer before the picker opens.
    pub(super) fn tab_ai_acp_initial_input_for_launch(
        prompt_type: &str,
        effective_intent: Option<&str>,
        pending_script_list_trigger: Option<char>,
        force_acp_surface: bool,
    ) -> Option<String> {
        if let Some(intent) = effective_intent {
            if force_acp_surface {
                return Some(intent.to_string());
            }

            return Some(
                crate::ai::harness::build_tab_ai_acp_initial_input_for_prompt(prompt_type, intent)
                    .text,
            );
        }

        match (prompt_type, pending_script_list_trigger) {
            ("ScriptList", Some(trigger @ ('/' | '@'))) => Some(trigger.to_string()),
            _ => None,
        }
    }
}
