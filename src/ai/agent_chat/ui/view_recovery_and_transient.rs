impl AgentChatView {
    pub(crate) fn retry_last_user_turn(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self
            .live_thread()
            .update(cx, |thread, cx| thread.retry_last_user_turn(cx))
        {
            tracing::warn!(
                target: "script_kit::agent_chat",
                event = "agent_chat_retry_failed",
                error = %error,
            );
        }
    }

    pub(crate) fn ai_recovery_actions(&self, cx: &App) -> Vec<crate::actions::Action> {
        let thread = self.live_thread().read(cx);
        let mut actions = Self::recovery_action_specs(thread.recovery_card_spec().as_ref());
        actions.extend(self.queue_actions(cx));
        if let Some(failure) = thread.brain_decision_failure() {
            for (operation, label) in [
                ("memoryInspect", "Review Held Question"),
                ("memoryRetry", "Retry Memory"),
                ("memoryContinue", "Continue Without Memory"),
            ] {
                let payload = serde_json::json!({"thread":thread.ui_thread_id(),"operation":operation,"target":thread.brain_decision_generation().to_string()});
                actions.push(
                    crate::actions::Action::new(
                        format!("agent_chat_queue:{payload}"),
                        label,
                        Some(format!("Memory unavailable. {} Use the original held question; any edited composer draft stays open.", failure.primary_message())),
                        crate::actions::ActionCategory::ScriptContext,
                    )
                    .with_section("Memory Recovery"),
                );
            }
        }
        actions
    }

    fn recovery_action_specs(
        spec: Option<&crate::ai::reliability::AiRecoveryCardSpec>,
    ) -> Vec<crate::actions::Action> {
        let Some(spec) = spec else {
            return Vec::new();
        };
        use crate::actions::{Action, ActionCategory};
        use crate::designs::icon_variations::IconName;

        spec.actions
            .iter()
            .filter(|recovery| recovery.enabled)
            .map(|recovery| {
                let icon = match recovery.action.kind() {
                    RecoveryActionKind::CopyDetails | RecoveryActionKind::CopyTranscript => {
                        IconName::Copy
                    }
                    RecoveryActionKind::Retry
                    | RecoveryActionKind::RetrySameDestination
                    | RecoveryActionKind::CheckAgain => IconName::Refresh,
                    RecoveryActionKind::ChooseCompatibleModel
                    | RecoveryActionKind::ChooseProvider
                    | RecoveryActionKind::ChooseProfile
                    | RecoveryActionKind::ConfigureProvider
                    | RecoveryActionKind::UpdateClient
                    | RecoveryActionKind::SignIn
                    | RecoveryActionKind::SwitchAccount
                    | RecoveryActionKind::RepairComponent
                    | RecoveryActionKind::ChooseDestination => IconName::Settings,
                    RecoveryActionKind::UseCurrentResults
                    | RecoveryActionKind::ContinueInAgentChat
                    | RecoveryActionKind::Reattach
                    | RecoveryActionKind::RethreadFlow
                    | RecoveryActionKind::RestartFlowRun
                    | RecoveryActionKind::TrimContext
                    | RecoveryActionKind::OpenDictationHistory => IconName::ArrowRight,
                };
                Action::new(
                    recovery.semantic_id,
                    recovery.label.to_string(),
                    Some("Recover this AI operation without losing your work".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_icon(icon)
                .with_section("Recovery")
            })
            .collect()
    }

    pub(crate) fn dispatch_ai_recovery_action(
        &mut self,
        action_id: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        let action = {
            let thread = self.live_thread().read(cx);
            thread
                .recovery_card_spec()
                .and_then(|spec| {
                    spec.actions
                        .into_iter()
                        .find(|action| action.semantic_id == action_id && action.enabled)
                })
                .map(|spec| spec.action)
        };
        let Some(action) = action else {
            return false;
        };
        self.dispatch_recovery_action(action, cx);
        true
    }

    fn dispatch_recovery_action(&mut self, action: AiRecoveryAction, cx: &mut Context<Self>) {
        let commands = match self
            .live_thread()
            .update(cx, |thread, cx| thread.select_recovery_action(action, cx))
        {
            Ok(commands) => commands,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::agent_chat",
                    event = "agent_chat_recovery_transition_rejected",
                    error_code = %error,
                );
                return;
            }
        };
        self.interpret_recovery_commands(commands, cx);
    }

    fn interpret_recovery_commands(&mut self, commands: Vec<AiCommand>, cx: &mut Context<Self>) {
        for command in commands {
            match command {
                AiCommand::CopyRedactedDiagnostics(_) => {
                    let receipt = {
                        let thread = self.live_thread().read(cx);
                        thread
                            .reliability_state()
                            .diagnostic
                            .as_ref()
                            .map(|diagnostic| {
                                format!(
                                    "AI recovery diagnostic\ncode={:?}\nfingerprint={}",
                                    match &thread.reliability_state().phase {
                                        sk_protocol::ai_reliability::AiPhase::AwaitingRecovery {
                                            failure,
                                            ..
                                        } => failure.code,
                                        _ => sk_protocol::ai_reliability::AiFailureCode::Unknown,
                                    },
                                    diagnostic.fingerprint.0
                                )
                            })
                    };
                    if let Some(receipt) = receipt {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(receipt));
                    }
                }
                AiCommand::LaunchAuthentication(auth) => {
                    let selected_model_id = self
                        .live_thread()
                        .read(cx)
                        .selected_model_id()
                        .map(str::to_string);
                    let provider =
                        crate::ai::agent_chat::goose::auth_recovery::resolve_auth_recovery_provider(
                            selected_model_id.as_deref(),
                        );
                    // Browser sign-in providers: retry the saved turn. The
                    // runtime starts the provider's login page on its own and
                    // the reply continues once the user finishes signing in.
                    if let Some(provider) = provider.as_deref().filter(|provider| {
                        crate::ai::agent_chat::goose::sign_in::is_browser_sign_in_provider(provider)
                    }) {
                        let forgotten = match auth.mode {
                            AuthRecoveryMode::SwitchAccount => {
                                crate::ai::agent_chat::goose::sign_in::forget(provider)
                            }
                            AuthRecoveryMode::SignIn => Ok(()),
                        };
                        self.live_thread().update(cx, |thread, cx| match forgotten {
                            Ok(()) => {
                                // Choosing Sign In is the consent: the retried
                                // turn opens the sign-in page without asking.
                                thread.prepare_sign_in_retry();
                                thread.complete_recovery_command(
                                    auth.command_id,
                                    RecoveryEffectResult::NoChange,
                                    cx,
                                );
                                if let Err(error) = thread.resume_recovered_turn_from_view(cx) {
                                    tracing::warn!(
                                        target: "script_kit::agent_chat",
                                        event = "agent_chat_sign_in_resume_failed",
                                        error_code = %error,
                                    );
                                }
                            }
                            Err(error) => {
                                let failure = crate::ai::reliability::provider_failure(
                                    sk_protocol::ai_reliability::ProtocolComponent::Goose,
                                    format!("stored sign-in could not be removed: {error}"),
                                );
                                thread.fail_recovery_command(auth.command_id, failure.failure, cx);
                            }
                        });
                        continue;
                    }
                    // API-key and other providers: goose's own setup, in Terminal.
                    let action = match auth.mode {
                        AuthRecoveryMode::SignIn => {
                            crate::ai::agent_chat::goose::auth_recovery::AuthRecoveryAction::SignInAgain
                        }
                        AuthRecoveryMode::SwitchAccount => {
                            crate::ai::agent_chat::goose::auth_recovery::AuthRecoveryAction::SwitchAccount
                        }
                    };
                    let launched = provider
                        .zip(crate::ai::agent_chat::goose::binary::default_goose_binary())
                        .is_some_and(|(provider, binary)| {
                            crate::ai::agent_chat::goose::auth_recovery::launch_auth_recovery(
                                binary, provider, action,
                            )
                            .is_ok()
                        });
                    if launched {
                        self.live_thread().update(cx, |thread, cx| {
                            thread.complete_recovery_command(
                                auth.command_id,
                                RecoveryEffectResult::ExternalActionLaunched,
                                cx,
                            );
                        });
                    } else {
                        let failure = crate::ai::reliability::provider_failure(
                            sk_protocol::ai_reliability::ProtocolComponent::Goose,
                            "authentication recovery could not start",
                        );
                        self.live_thread().update(cx, |thread, cx| {
                            thread.fail_recovery_command(auth.command_id, failure.failure, cx);
                        });
                    }
                }
                AiCommand::OpenConfiguration(target) => match target.kind {
                    ConfigurationTargetKind::Model
                    | ConfigurationTargetKind::Provider
                    | ConfigurationTargetKind::Profile => {
                        self.open_profile_trigger_picker(cx);
                    }
                    ConfigurationTargetKind::Context => {
                        self.live_thread().update(cx, |thread, cx| {
                            thread.clear_pending_context_for_new_entry_intent(cx);
                            thread.complete_recovery_command(
                                target.command_id,
                                RecoveryEffectResult::ContextTrimmed,
                                cx,
                            );
                        });
                    }
                    ConfigurationTargetKind::Mdflow => {}
                },
                AiCommand::RecheckClientCapability(check) => {
                    self.live_thread().update(cx, |thread, cx| {
                        thread.complete_recovery_command(
                            check.command_id,
                            RecoveryEffectResult::CapabilityRechecked,
                            cx,
                        );
                        if let Err(error) = thread.resume_recovered_turn_from_view(cx) {
                            tracing::warn!(
                                target: "script_kit::agent_chat",
                                event = "agent_chat_recovery_resume_failed",
                                error_code = %error,
                            );
                        }
                    });
                }
                AiCommand::OpenClientUpdate(update) => {
                    let url = match update.client {
                        sk_protocol::ai_reliability::ClientKind::Codex => {
                            "https://github.com/openai/codex/releases/latest"
                        }
                        sk_protocol::ai_reliability::ClientKind::Goose
                        | sk_protocol::ai_reliability::ClientKind::Mdflow
                        | sk_protocol::ai_reliability::ClientKind::LocalLlm
                        | sk_protocol::ai_reliability::ClientKind::Other => {
                            "https://scriptkit.com/"
                        }
                    };
                    match std::process::Command::new("open").arg(url).spawn() {
                        Ok(_) => {
                            self.live_thread().update(cx, |thread, cx| {
                                thread.complete_recovery_command(
                                    update.command_id,
                                    RecoveryEffectResult::ExternalActionLaunched,
                                    cx,
                                );
                            });
                        }
                        Err(error) => {
                            let failure = crate::ai::reliability::provider_failure(
                                sk_protocol::ai_reliability::ProtocolComponent::Codex,
                                format!("client update page could not open: {error}"),
                            );
                            self.live_thread().update(cx, |thread, cx| {
                                thread.fail_recovery_command(
                                    update.command_id,
                                    failure.failure,
                                    cx,
                                );
                            });
                        }
                    }
                }
                AiCommand::InstallOrRepairComponent(_) => {
                    self.open_profile_trigger_picker(cx);
                }
                AiCommand::RunSurfaceRecovery { command_id, action } => {
                    let failure = crate::ai::reliability::destination_failure(
                        true,
                        "surface-owned recovery callback is unavailable in Agent Chat",
                    );
                    tracing::warn!(
                        target: "script_kit::agent_chat",
                        event = "agent_chat_surface_recovery_rejected",
                        ?action,
                    );
                    self.live_thread().update(cx, |thread, cx| {
                        thread.fail_recovery_command(command_id, failure.failure, cx);
                    });
                }
                AiCommand::ContinueInAgentChat(escalation) => {
                    let seed = self.live_thread().read(cx).quick_ai_handoff_seed();
                    match (seed, self.on_continue_in_agent_chat.clone()) {
                        (Some(seed), Some(callback)) => {
                            self.live_thread().update(cx, |thread, cx| {
                                thread.complete_recovery_command(
                                    escalation.command_id,
                                    RecoveryEffectResult::AgentChatOpened,
                                    cx,
                                );
                            });
                            cx.defer(move |cx| callback(seed.clone(), cx));
                        }
                        _ => {
                            let failure = crate::ai::reliability::provider_failure(
                                sk_protocol::ai_reliability::ProtocolComponent::Provider,
                                "Quick AI handoff could not be prepared",
                            );
                            self.live_thread().update(cx, |thread, cx| {
                                thread.fail_recovery_command(
                                    escalation.command_id,
                                    failure.failure,
                                    cx,
                                );
                            });
                        }
                    }
                }
                AiCommand::PersistWork(_)
                | AiCommand::CheckCapabilities(_)
                | AiCommand::StartTurn(_)
                | AiCommand::CancelTurn { .. }
                | AiCommand::ScheduleBackoff { .. }
                | AiCommand::ApplySelection(_)
                | AiCommand::ReattachSession(_)
                | AiCommand::RethreadFlow(_)
                | AiCommand::RestartFlowRun(_)
                | AiCommand::ClearPendingWork(_)
                | AiCommand::ScheduleRecoveredDismiss => {}
            }
        }
    }

    fn render_reserved_transient_lane(
        id: &'static str,
        height_px: f32,
        content: Option<gpui::AnyElement>,
    ) -> gpui::AnyElement {
        let height_px = agent_chat_transient_lane_height(height_px, content.is_some());
        div()
            .id(id)
            .w_full()
            .h(px(height_px))
            .overflow_hidden()
            .when_some(content, |d, content| d.child(content))
            .into_any_element()
    }

    fn message_queue_lane_active(&self, cx: &App) -> bool {
        !self.live_thread().read(cx).queued_messages().is_empty()
    }

    fn context_bootstrap_note_lane_active(&self, cx: &App) -> bool {
        let thread = self.live_thread().read(cx);
        thread.sign_in_note().is_some()
            || thread
                .context_bootstrap_note()
                .is_some_and(|note| !note.trim().is_empty())
    }

    fn render_message_queue_strip(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let thread = self.live_thread().read(cx);
        let count = thread.queued_messages().len();
        let label = if thread.queue_paused() {
            format!("{count} queued · paused · Review in Actions")
        } else {
            format!("{count} queued · Review in Actions")
        };
        div()
            .id("agent_chat-message-queue-header")
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .px(px(
                crate::components::conversation_style::CONVERSATION_INPUT_PADDING_X,
            ))
            .text_xs()
            .text_color(rgb(theme::get_cached_theme().colors.text.primary))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, window, cx| this.trigger_toggle_actions(window, cx)))
            .child(label)
            .into_any_element()
    }

    pub(crate) fn queue_actions(&self, cx: &App) -> Vec<crate::actions::Action> {
        use crate::actions::{Action, ActionCategory};
        let thread = self.live_thread().read(cx);
        if thread.queued_messages().is_empty() {
            return Vec::new();
        }
        let revision = thread.queue_revision();
        let make = |operation: &str, target: &str, title: String, description: String| {
            let payload = serde_json::json!({"thread": thread.ui_thread_id(), "operation": operation, "target": target});
            Action::new(
                format!("agent_chat_queue:{payload}"),
                title,
                Some(description),
                ActionCategory::ScriptContext,
            )
            .with_section("Queued Messages")
        };
        let mut actions = vec![
            make(
                if thread.queue_paused() {
                    "resume"
                } else {
                    "pause"
                },
                &revision,
                if thread.queue_paused() {
                    "Resume Queue"
                } else {
                    "Keep Queue Paused"
                }
                .to_string(),
                "Sending resumes only when you choose Resume Queue.".to_string(),
            ),
            make(
                "clear",
                &revision,
                "Clear Queued Messages".to_string(),
                "Remove this exact queue snapshot; the current response continues.".to_string(),
            ),
        ];
        if thread.brain_decision_failure().is_some() {
            actions.remove(0);
        }
        for message in thread.queued_messages() {
            let preview: String = message.text.chars().take(64).collect();
            let context = message
                .context_items
                .iter()
                .map(|item| {
                    format!(
                        "{} ({:?}; {})",
                        item.part.label(),
                        item.part.source_kind(),
                        item.id.as_str()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            actions.push(make(
                "inspect",
                &message.id,
                format!("Inspect: {preview}"),
                format!(
                    "{} frozen context items · {context}",
                    message.context_items.len()
                ),
            ));
            actions.push(make(
                "remove",
                &message.id,
                format!("Remove: {preview}"),
                "Remove only this message. Already dispatched messages cannot be removed."
                    .to_string(),
            ));
        }
        actions
    }

    pub(crate) fn queue_semantic_elements(&self, cx: &App) -> Vec<crate::protocol::ElementInfo> {
        let mut elements = Vec::new();
        let thread = self.live_thread().read(cx);
        if let Some(failure) = thread.brain_decision_failure() {
            let mut failure_element =
                crate::protocol::ElementInfo::panel("agent_chat.memoryUnavailable");
            failure_element.text = Some(format!(
                "Memory unavailable · {} · Retry Memory or Continue Without Memory in Actions",
                failure.primary_message()
            ));
            failure_element.status_kind = Some("unavailable".to_string());
            failure_element.value =
                Some(serde_json::to_string(&failure.failure).unwrap_or_default());
            elements.push(failure_element);
        }
        if thread.queued_messages().is_empty() && thread.brain_decision_failure().is_none() {
            return elements;
        }
        let mut panel = crate::protocol::ElementInfo::panel("agent_chat.queue");
        panel.text = Some(format!(
            "{} queued · {}",
            thread.queued_messages().len(),
            if thread.queue_paused() {
                "paused"
            } else {
                "ready"
            }
        ));
        panel.value = Some(thread.queue_revision());
        elements.push(panel);
        for (index, action) in self
            .ai_recovery_actions(cx)
            .into_iter()
            .filter(|action| action.id.starts_with("agent_chat_queue:"))
            .enumerate()
        {
            let mut element = crate::protocol::ElementInfo::button(index, &action.title);
            element.semantic_id = action.id.clone();
            element.source = Some(action.id);
            element.value = action.description;
            element.selectable = Some(true);
            elements.push(element);
        }
        elements
    }

    pub(crate) fn execute_queue_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(payload) = action_id.strip_prefix("agent_chat_queue:") else {
            return false;
        };
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(payload) else {
            return false;
        };
        let operation = payload["operation"].as_str().unwrap_or("");
        let target = payload["target"].as_str().unwrap_or("");
        let thread = self.live_thread();
        if payload["thread"].as_str() != Some(thread.read(cx).ui_thread_id()) {
            return false;
        }
        if operation == "memoryRetry" || operation == "memoryContinue" {
            return target.parse::<u64>().ok().is_some_and(|generation| {
                thread.update(cx, |thread, cx| {
                    thread.resolve_brain_decision(operation == "memoryRetry", generation, cx)
                })
            });
        }
        if operation == "inspect" || operation == "memoryInspect" {
            let message = if operation == "memoryInspect" {
                target
                    .parse::<u64>()
                    .ok()
                    .and_then(|generation| thread.read(cx).held_brain_request(generation))
            } else {
                thread
                    .read(cx)
                    .queued_messages()
                    .iter()
                    .find(|message| message.id == target)
                    .cloned()
            };
            let Some(message) = message else {
                self.command_status = Some("This request changed or has already dispatched. Review the current requests in Actions.");
                self.notify_semantic_change(cx);
                return false;
            };
            let parts: Vec<_> = message
                .context_items
                .iter()
                .map(|item| &item.part)
                .collect();
            let body = format!(
                "{}\n\nFrozen context ({}):\n{}",
                message.text,
                parts.len(),
                serde_json::to_string_pretty(&parts).unwrap_or_default()
            );
            let owner = cx.entity().downgrade();
            crate::confirm::open_parent_confirm_dialog_for_entity(
                window,
                cx,
                owner,
                crate::confirm::ParentConfirmOptions {
                    title: if operation == "memoryInspect" {
                        "Held Question"
                    } else {
                        "Queued Message"
                    }
                    .into(),
                    body: body.into(),
                    confirm_text: "Copy request text".into(),
                    cancel_text: "Dismiss".into(),
                    confirm_variant: gpui_component::button::ButtonVariant::Primary,
                    width: px(crate::confirm::PARENT_MODAL_WIDTH_PX),
                },
                move |window, cx| {
                    use gpui_component::WindowExt as _;
                    let notification = match crate::platform::copy_text(&message.text) {
                        Ok(receipt) => gpui_component::notification::Notification::info(
                            receipt.feedback("Copied request text".to_string()),
                        ),
                        Err(error) => {
                            let diagnostic =
                                crate::logging::log_private_user_value(&error.to_string());
                            tracing::warn!(diagnostic_fingerprint = %diagnostic.sha256, "Request text copy failed");
                            gpui_component::notification::Notification::error(
                                "Couldn't copy this request. Review it in Actions and try again.",
                            )
                            .autohide(false)
                        }
                    };
                    window.push_notification(notification.id1::<AgentChatView>("request-copy"), cx);
                },
                |_, _| {},
            );
            return true;
        }
        let executed = thread.update(cx, |thread, cx| match operation {
            "remove" => thread.remove_queued_message(target, cx),
            "clear" => thread.clear_queued_messages(target, cx),
            "resume" | "pause" if thread.queue_revision() == target => {
                thread.set_queue_paused(operation == "pause", cx).is_ok()
            }
            _ => false,
        });
        if !executed {
            self.command_status = Some("The queue changed or this message was already dispatched. Review it again in Actions.");
        }
        self.notify_semantic_change(cx);
        executed
    }

    fn render_context_bootstrap_note(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        // Provider sign-in status outranks context capture: until the user is
        // signed in, nothing else in this lane can make progress.
        let (state, note) = {
            let thread = self.live_thread().read(cx);
            match (thread.sign_in_state(), thread.sign_in_note()) {
                // Sign-in status is information, not an alert: the quiet style.
                (Some(_), Some(note)) => (AgentChatContextBootstrapState::Ready, Some(note)),
                _ => (
                    thread.context_bootstrap_state(),
                    thread.context_bootstrap_note().map(|v| v.to_string()),
                ),
            }
        };

        let Some(note) = note.filter(|v| !v.trim().is_empty()) else {
            return div()
                .id("agent_chat-context-bootstrap-note-empty")
                .into_any_element();
        };

        let theme = theme::get_cached_theme();
        let accent = theme.colors.accent.selected;
        let border = theme.colors.ui.border;

        let (fg_color, bg, outline) = match state {
            AgentChatContextBootstrapState::Preparing => {
                (accent, (accent << 8) | 0x10, (accent << 8) | 0x24)
            }
            AgentChatContextBootstrapState::Ready => (
                theme.colors.text.muted,
                (border << 8) | 0x10,
                (border << 8) | 0x24,
            ),
            AgentChatContextBootstrapState::Failed => (
                theme.colors.text.primary,
                (border << 8) | 0x14,
                (border << 8) | 0x28,
            ),
        };

        div()
            .id("agent_chat-context-bootstrap-note")
            .px(px(12.0))
            .pb(px(6.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(6.0))
                    .bg(rgba(bg))
                    .border_1()
                    .border_color(rgba(outline))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(fg_color))
                            .child(SharedString::from(note)),
                    ),
            )
            .into_any_element()
    }
}
