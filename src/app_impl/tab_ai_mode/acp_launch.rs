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
        let (_broker, permission_rx) = crate::ai::agent_chat::ui::AgentChatPermissionBroker::new();
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_open_stage",
            stage = "permission_broker_new",
            stage_ms = stage_started_at.elapsed().as_millis() as u64,
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        let profile_ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
        let ai_preferences = crate::config::load_user_preferences().ai;
        let effective_profile = crate::ai::agent_chat::profiles::resolve_effective_profile(
            &ai_preferences,
            &profile_ctx,
        );
        let focused_text_mini =
            request.ui_variant == crate::ai::acp::ui_variant::AcpChatUiVariant::FocusedTextMini;
        let pi_launch_result = if focused_text_mini {
            crate::ai::agent_chat::launch::resolve_focused_text_pi_launch(
                &ai_preferences,
                &profile_ctx,
            )
        } else {
            // Apply the Spine cwd chip as the agent's working directory. The Pi
            // runtime bakes cwd at spawn time and ignores per-turn cwd, so this
            // must be set on the launch spec, not via AcpThread::set_cwd.
            crate::ai::agent_chat::launch::PiAgentChatLaunch::from_profile_with_cwd_override(
                effective_profile.clone(),
                self.spine_cwd_for_acp_launch(),
            )
        };
        match pi_launch_result {
            Ok(pi_launch) => {
                self.open_tab_ai_pi_view_from_launch(
                    pi_launch,
                    request,
                    capture_rx,
                    focused_part,
                    use_ask_anything_fallback,
                    explicit_ambient_chip_label,
                    auto_submit,
                    effective_intent,
                    acp_initial_input,
                    permission_rx,
                    source_view,
                    had_harness_session,
                    pending_script_list_trigger,
                    open_started_at,
                    cx,
                );
                return;
            }
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "pi_agent_chat_launch_resolution_failed",
                    error = %error,
                    focused_text_mini,
                );
                if focused_text_mini {
                    self.toast_manager.push(
                        crate::components::toast::Toast::error(
                            "Pi Text profile is unavailable",
                            &self.theme,
                        )
                        .duration_ms(Some(TOAST_ERROR_MS)),
                    );
                }
                self.show_pi_agent_chat_unavailable_setup_view(source_view, error.to_string(), cx);
                return;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn open_tab_ai_pi_view_from_launch(
        &mut self,
        pi_launch: crate::ai::agent_chat::launch::PiAgentChatLaunch,
        request: TabAiLaunchRequest,
        capture_rx: TabAiDeferredCaptureRx,
        focused_part: Option<crate::ai::message_parts::AiContextPart>,
        use_ask_anything_fallback: bool,
        explicit_ambient_chip_label: Option<String>,
        auto_submit: bool,
        effective_intent: Option<String>,
        acp_initial_input: Option<String>,
        permission_rx: async_channel::Receiver<crate::ai::acp::AcpApprovalRequest>,
        source_view: AppView,
        had_harness_session: bool,
        pending_script_list_trigger: Option<char>,
        open_started_at: std::time::Instant,
        cx: &mut Context<Self>,
    ) {
        let requirements = crate::ai::agent_chat::ui::AgentChatLaunchRequirements {
            needs_embedded_context: focused_part.is_some(),
            needs_image: focused_part
                .as_ref()
                .map(|part| part.source().contains("screenshot=1"))
                .unwrap_or(false),
        };
        let warm_spec = pi_launch.warm_spec();
        let manager = crate::ai::agent_chat::launch::warm_session_manager();
        let Some(lease) = manager.acquire_warm_ready(&pi_launch.warm_key) else {
            match manager.prepare_warm_background(warm_spec) {
                Ok(snapshot) => {
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "pi_agent_chat_warm_prepare_background",
                        profile_id = %pi_launch.profile.id,
                        warm_key = %pi_launch.warm_key,
                        generation = snapshot.generation,
                        state = ?snapshot.state,
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "pi_agent_chat_warm_prepare_failed",
                        profile_id = %pi_launch.profile.id,
                        warm_key = %pi_launch.warm_key,
                        error = %error,
                    );
                }
            }
            self.toast_manager.push(
                crate::components::toast::Toast::info(
                    "Starting Pi Agent Chat. Try again in a moment.",
                    &self.theme,
                )
                .duration_ms(Some(TOAST_ERROR_MS)),
            );
            cx.notify();
            return;
        };

        let connection = lease.connection.clone();
        let cwd = lease.cwd.clone();
        let ui_thread_id = lease.ui_thread_id.clone();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "pi_agent_chat_warm_acquired",
            profile_id = %pi_launch.profile.id,
            profile_name = %pi_launch.profile.name,
            warm_key = %pi_launch.warm_key,
            generation = lease.generation,
            ui_thread_id = %ui_thread_id,
            cwd = %cwd.display(),
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        let thread = cx.new(|cx| {
            crate::ai::agent_chat::ui::AgentChatThread::new(
                connection,
                permission_rx,
                crate::ai::agent_chat::ui::AgentChatThreadInit {
                    ui_thread_id,
                    cwd,
                    initial_input: acp_initial_input.clone(),
                    initial_context_parts: Vec::new(),
                    display_name: pi_launch.profile.name.clone().into(),
                    profile_display_name: Some(pi_launch.profile.name.clone().into()),
                    profile_icon_name: pi_launch.profile.icon_name.clone(),
                    selected_agent: None,
                    available_agents: Vec::new(),
                    launch_requirements: requirements,
                    available_models: pi_launch.available_models.clone(),
                    selected_model_id: pi_launch.selected_model_id.clone(),
                },
                cx,
            )
        });

        let view_entity = cx.new(|cx| {
            crate::ai::agent_chat::ui::AgentChatView::new(thread.clone(), cx).with_ui_variant(request.ui_variant)
        });

        self.active_agent_chat_warm_lease = Some(lease);
        self.wire_embedded_acp_footer_callbacks(&view_entity, cx);
        self.embedded_acp_chat = Some(view_entity.clone());
        self.tab_ai_harness_return_view = Some(source_view.clone());
        self.tab_ai_harness_return_focus_target = Some(self.tab_ai_return_focus_target());
        self.seed_tab_ai_apply_back_route(
            &request.source_view,
            &request.ui_snapshot,
            focused_part.as_ref(),
        );

        let view_entity_for_staging = view_entity.clone();
        view_entity_for_staging.update(cx, |view, _cx| {
            view.opened_via_transient_trigger = pending_script_list_trigger;
        });
        self.enter_embedded_acp_chat_surface(view_entity, cx);
        cx.notify();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "pi_agent_chat_view_switched",
            profile_id = %pi_launch.profile.id,
            acp_chat_ui_variant = request.ui_variant.state_id(),
            total_ms = open_started_at.elapsed().as_millis() as u64,
        );

        let needs_deferred = self.stage_acp_initial_context_parts(
            None,
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
    ) -> Option<crate::ai::agent_chat::ui::AgentChatRetryRequest> {
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
    /// ScriptList-triggered `@`, `/`, and `|` routes prefill the raw trigger so the
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
            ("ScriptList", Some(trigger @ ('/' | '@' | '|'))) => Some(trigger.to_string()),
            _ => None,
        }
    }

    /// Persist the current `spine_cwd` to user preferences (`ai.cwd`) so it is
    /// restored on the next app launch. Non-fatal and off the UI thread.
    pub(crate) fn persist_spine_cwd(&self) {
        let cwd = self
            .spine_cwd
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        std::thread::Builder::new()
            .name("persist-spine-cwd".into())
            .spawn(move || {
                let mut prefs = crate::config::load_user_preferences();
                if prefs.ai.cwd == cwd {
                    return;
                }
                prefs.ai.cwd = cwd.clone();
                if let Err(error) = crate::config::save_user_preferences(&prefs) {
                    tracing::warn!(
                        target: "script_kit::spine",
                        event = "persist_spine_cwd_failed",
                        error = %error,
                    );
                } else {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "persist_spine_cwd",
                        cwd = ?cwd,
                    );
                }
            })
            .ok();
    }

    /// The working directory to launch the agent in, derived from the Spine cwd
    /// chip. Returns `None` (use the profile/default cwd) unless the user has
    /// *explicitly* picked a cwd (revision > 0) and it is still a directory.
    ///
    /// The startup default (`~/.scriptkit`, revision 0) intentionally does not
    /// override the profile's launch cwd, so default launches keep hitting the
    /// startup-warmed session and the General profile's scratch directory.
    pub(crate) fn spine_cwd_for_acp_launch(&self) -> Option<std::path::PathBuf> {
        if self.spine_cwd_revision == 0 {
            return None;
        }
        let cwd = self.spine_cwd.as_ref()?;
        if cwd.is_dir() {
            Some(cwd.clone())
        } else {
            tracing::warn!(
                target: "script_kit::spine",
                event = "spine_cwd_for_acp_launch_not_a_dir",
                cwd = %cwd.display(),
                "Spine cwd is not a directory; falling back to profile cwd"
            );
            None
        }
    }

    /// Start warming a Pi Agent Chat session for the current Spine cwd so a
    /// later Cmd+Enter acquires a ready warm session with the correct working
    /// directory instead of missing (which would surface the "try again"
    /// toast). Invoked when the user picks a cwd.
    pub(crate) fn prewarm_acp_for_spine_cwd(&self, cx: &mut Context<Self>) {
        let _ = cx;
        let profile_ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
        let ai_preferences = crate::config::load_user_preferences().ai;
        let effective_profile = crate::ai::agent_chat::profiles::resolve_effective_profile(
            &ai_preferences,
            &profile_ctx,
        );
        match crate::ai::agent_chat::launch::PiAgentChatLaunch::from_profile_with_cwd_override(
            effective_profile,
            self.spine_cwd_for_acp_launch(),
        ) {
            Ok(pi_launch) => {
                let manager = crate::ai::agent_chat::launch::warm_session_manager();
                if let Err(error) = manager.prepare_warm_background(pi_launch.warm_spec()) {
                    tracing::warn!(
                        target: "script_kit::spine",
                        event = "prewarm_acp_for_spine_cwd_failed",
                        warm_key = %pi_launch.warm_key,
                        error = %error,
                    );
                } else {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "prewarm_acp_for_spine_cwd",
                        warm_key = %pi_launch.warm_key,
                        cwd = %pi_launch.cwd.display(),
                    );
                }
            }
            Err(error) => {
                tracing::debug!(
                    target: "script_kit::spine",
                    event = "prewarm_acp_for_spine_cwd_resolution_failed",
                    error = %error,
                );
            }
        }
    }
}
