//! Closed production conversation seeds and controls. No renderer lives here.
use super::fixture_ids::{FLOW_FIXTURE_IDS, SDK_CHAT_FIXTURE_IDS};
use gpui::{App, AppContext, Entity, Window};

pub(crate) fn seed_owned_flow_catalogue() -> anyhow::Result<Vec<crate::flows::model::FlowDescriptor>>
{
    let scope = crate::runtime_policy::owned_evaluation()
        .ok_or_else(|| anyhow::anyhow!("owned_flow_required"))?;
    let path = scope.root().join("conversation.fixture");
    scope.require_owned_path(&path)?;
    if !path.exists() {
        std::fs::write(
            &path,
            "---\nmodel: fixture-model\n---\nSummarize the supplied fixture text.",
        )?;
    }
    let flows = vec![crate::flows::model::FlowDescriptor {
        id: "owned:conversation".into(),
        path: path.to_string_lossy().into_owned(),
        source: crate::flows::model::FlowSource::Project,
        name: "flow-conversation".into(),
        description: Some("A controlled production Flow conversation".into()),
        engine: "codex".into(),
        engine_source: Some("fixture".into()),
        inputs: Vec::new(),
        is_workflow: false,
        interactive: false,
        mtime_ms: 0,
        origin: Some("Owned fixture".into()),
        wrapper_command: None,
    }];
    crate::flows::catalog::flow_catalog()
        .install_owned_roster(scope.root().to_string_lossy().into_owned(), flows.clone())?;
    Ok(flows)
}

impl crate::ScriptListApp {
    pub(crate) fn mount_flow_fixture(
        &mut self,
        fixture_id: &str,
        cx: &mut gpui::Context<Self>,
    ) -> anyhow::Result<Option<u64>> {
        anyhow::ensure!(
            FLOW_FIXTURE_IDS.contains(&fixture_id),
            "unknown_flow_fixture"
        );
        let flows = seed_owned_flow_catalogue()?;
        let variant = match fixture_id {
            "flow.desk.dispatch" => crate::flows::model::FlowUxVariant::Dispatch,
            "flow.desk.lens" => crate::flows::model::FlowUxVariant::Lens,
            _ => crate::flows::model::FlowUxVariant::Flash,
        };
        // Give sessions a real Desk return route. A direct-open session closes
        // through the native hide barrier, which owned-hidden evaluation refuses.
        self.current_view = crate::AppView::FlowUxView {
            variant,
            filter: String::new(),
            selected_index: 0,
            inline_run: None,
        };
        self.note_main_route_changed();
        if fixture_id == "flow.session" {
            return self.create_owned_flow_session(&flows[0], cx).map(Some);
        }
        cx.notify();
        Ok(None)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FlowFixtureAction {
    Submit {
        text: String,
    },
    Text {
        expected_message_id: String,
        text: String,
    },
    Complete {
        expected_message_id: String,
    },
    Fail {
        expected_message_id: String,
    },
    Retry,
    Stop,
    Background,
    Resume,
}

impl crate::ScriptListApp {
    pub(crate) fn drive_flow_fixture(
        &mut self,
        session_id: u64,
        action: FlowFixtureAction,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> anyhow::Result<()> {
        crate::runtime_policy::WindowHostPolicy::OwnedHidden.validate()?;
        use crate::flows::codex_client::FlowThreadEvent;
        match action {
            FlowFixtureAction::Submit { text } => {
                anyhow::ensure!(
                    !text.trim().is_empty() && text.len() <= 64 * 1024,
                    "invalid_flow_fixture_input"
                );
                let result = self.submit_flow_chat_message(session_id, text, cx);
                anyhow::ensure!(
                    matches!(result, crate::FlowChatSubmitResult::Dispatched),
                    "flow_submit_not_accepted"
                );
            }
            FlowFixtureAction::Text {
                expected_message_id,
                text,
            } => {
                anyhow::ensure!(text.len() <= 64 * 1024, "flow_fixture_text_limit");
                self.apply_owned_flow_event(
                    session_id,
                    &expected_message_id,
                    FlowThreadEvent::AgentDelta {
                        session_id,
                        item_id: expected_message_id.clone(),
                        delta: text,
                    },
                    cx,
                )?;
            }
            FlowFixtureAction::Complete {
                expected_message_id,
            } => self.apply_owned_flow_event(
                session_id,
                &expected_message_id,
                FlowThreadEvent::TurnCompleted {
                    session_id,
                    outcome: crate::ai::reliability::AiTurnRuntimeOutcome::Completed {
                        stop_reason: Some("fixture".into()),
                    },
                },
                cx,
            )?,
            FlowFixtureAction::Fail {
                expected_message_id,
            } => self.apply_owned_flow_event(
                session_id,
                &expected_message_id,
                FlowThreadEvent::TurnFailed {
                    session_id,
                    failure: crate::ai::reliability::runtime_closed_failure(
                        sk_protocol::ai_reliability::ProtocolComponent::Codex,
                        "controlled Flow stream closed",
                    ),
                },
                cx,
            )?,
            FlowFixtureAction::Retry => self.dispatch_flow_recovery_action(
                session_id,
                sk_protocol::ai_reliability::AiRecoveryAction::Retry,
                window,
                cx,
            ),
            FlowFixtureAction::Stop => self.stop_flow_session(session_id, cx),
            FlowFixtureAction::Background => self.background_flow_session(window, cx),
            FlowFixtureAction::Resume => self.open_flow_session(session_id, cx),
        }
        Ok(())
    }
}

enum SdkFixtureRequest {
    Submit(crate::prompts::chat::ChatPromptPreparedRequest),
    Retry(crate::prompts::chat::ChatPromptPreparedRequest),
    Stop,
}

pub(crate) struct SdkChatFixtureControl {
    requests: async_channel::Receiver<SdkFixtureRequest>,
    pub accepted_requests: Vec<String>,
    pub sink_requests: Vec<serde_json::Value>,
    pub stop_requests: usize,
    next_response: u64,
}

pub(crate) enum SdkChatFixtureAction {
    Submit(String),
    Retry,
    Stop,
    Text { message_id: String, text: String },
    Complete { message_id: String },
    Fail { message_id: String },
}

pub(crate) fn create_sdk_chat_fixture(
    id: &str,
    _window: &mut Window,
    cx: &mut App,
) -> anyhow::Result<(Entity<crate::prompts::ChatPrompt>, SdkChatFixtureControl)> {
    crate::runtime_policy::WindowHostPolicy::OwnedHidden.validate()?;
    anyhow::ensure!(
        SDK_CHAT_FIXTURE_IDS.contains(&id),
        "unknown_sdk_chat_fixture"
    );
    let (tx, requests) = async_channel::bounded(16);
    let retry_tx = tx.clone();
    let stop_tx = tx.clone();
    let prompt = cx.new(|cx| {
        crate::prompts::ChatPrompt::new_sdk(
            id.to_string(),
            Some("Message the fixture".into()),
            Vec::new(),
            None,
            None,
            cx.focus_handle(),
            Some(std::sync::Arc::new(move |request| {
                tx.try_send(SdkFixtureRequest::Submit(request))
                    .map_err(|_| "sdk_fixture_sink_full_or_closed".into())
            })),
            false,
            std::sync::Arc::new(crate::theme::get_cached_theme()),
        )
        .with_retry_callback(std::sync::Arc::new(move |request| {
            retry_tx
                .try_send(SdkFixtureRequest::Retry(request.prepared))
                .map_err(|_| {
                    Box::new(crate::ai::reliability::runtime_closed_failure(
                        sk_protocol::ai_reliability::ProtocolComponent::Provider,
                        "SDK fixture retry sink full or closed",
                    ))
                })
        }))
        .with_stop_callback(std::sync::Arc::new(move |_| {
            stop_tx.try_send(SdkFixtureRequest::Stop).map_err(|_| {
                Box::new(crate::ai::reliability::runtime_closed_failure(
                    sk_protocol::ai_reliability::ProtocolComponent::Provider,
                    "SDK fixture Stop sink full or closed",
                ))
            })
        }))
    });
    let mut control = SdkChatFixtureControl {
        requests,
        accepted_requests: Vec::new(),
        sink_requests: Vec::new(),
        stop_requests: 0,
        next_response: 0,
    };
    if id != "sdk-chat.empty" {
        drive_sdk_chat_fixture(
            &prompt,
            &mut control,
            SdkChatFixtureAction::Submit("Fixture request".into()),
            cx,
        )?;
        let message_id = prompt
            .read(cx)
            .current_stream_message_id()
            .ok_or_else(|| anyhow::anyhow!("sdk_fixture_stream_missing"))?
            .to_string();
        if id == "sdk-chat.retryable-failure" {
            drive_sdk_chat_fixture(
                &prompt,
                &mut control,
                SdkChatFixtureAction::Fail { message_id },
                cx,
            )?;
        } else {
            drive_sdk_chat_fixture(
                &prompt,
                &mut control,
                SdkChatFixtureAction::Text {
                    message_id,
                    text: "Streaming fixture response".into(),
                },
                cx,
            )?;
        }
    }
    Ok((prompt, control))
}

pub(crate) fn drive_sdk_chat_fixture(
    prompt: &Entity<crate::prompts::ChatPrompt>,
    control: &mut SdkChatFixtureControl,
    action: SdkChatFixtureAction,
    cx: &mut App,
) -> anyhow::Result<()> {
    crate::runtime_policy::WindowHostPolicy::OwnedHidden.validate()?;
    match action {
        SdkChatFixtureAction::Submit(text) => {
            anyhow::ensure!(
                !text.trim().is_empty() && text.len() <= 64 * 1024,
                "invalid_sdk_fixture_text"
            );
            prompt.update(cx, |prompt, cx| {
                prompt.set_input(text, cx);
                prompt.submit(cx);
            });
        }
        SdkChatFixtureAction::Retry => {
            let result =
                prompt.update(cx, |prompt, cx| {
                    prompt.execute_conversation_command(
                crate::components::conversation_actions::ChatPromptConversationCommand::Retry, cx)
                });
            anyhow::ensure!(
                matches!(
                    result,
                    crate::components::conversation_actions::ConversationCommandExecution::Executed
                ),
                "sdk_retry_not_dispatched"
            );
        }
        SdkChatFixtureAction::Stop => prompt.update(cx, |prompt, cx| prompt.stop_streaming(cx)),
        SdkChatFixtureAction::Text { message_id, text } => {
            anyhow::ensure!(
                prompt.read(cx).current_stream_message_id() == Some(message_id.as_str()),
                "stale_sdk_stream"
            );
            anyhow::ensure!(text.len() <= 64 * 1024, "sdk_fixture_text_limit");
            prompt.update(cx, |prompt, cx| prompt.append_chunk(&message_id, &text, cx));
        }
        SdkChatFixtureAction::Complete { message_id } => {
            anyhow::ensure!(
                prompt.read(cx).current_stream_message_id() == Some(message_id.as_str()),
                "stale_sdk_stream"
            );
            prompt.update(cx, |prompt, cx| prompt.complete_streaming(&message_id, cx));
        }
        SdkChatFixtureAction::Fail { message_id } => {
            anyhow::ensure!(
                prompt.read(cx).current_stream_message_id() == Some(message_id.as_str()),
                "stale_sdk_stream"
            );
            let failure = crate::ai::reliability::runtime_closed_failure(
                sk_protocol::ai_reliability::ProtocolComponent::Provider,
                "controlled SDK stream closed",
            );
            let summary = failure.primary_message().to_string();
            prompt.update(cx, |prompt, cx| {
                prompt.set_message_failure(&message_id, failure.failure, summary, cx)
            });
        }
    }
    drain_sdk_chat_fixture(prompt, control, cx)
}

pub(crate) fn drain_sdk_chat_fixture(
    prompt: &Entity<crate::prompts::ChatPrompt>,
    control: &mut SdkChatFixtureControl,
    cx: &mut App,
) -> anyhow::Result<()> {
    while let Ok(request) = control.requests.try_recv() {
        match request {
            SdkFixtureRequest::Submit(request) | SdkFixtureRequest::Retry(request) => {
                anyhow::ensure!(
                    control.accepted_requests.len() < 32,
                    "sdk_fixture_turn_limit"
                );
                control
                    .accepted_requests
                    .push(request.request_ref().0.clone());
                // Project only the request actually received by the callback sink.
                control.sink_requests.push(serde_json::json!({
                    "requestRef": request.request_ref().0,
                    "promptId": request.prompt_id(),
                    "displayText": request.display_text(),
                    "outboundText": request.outbound_text(),
                    "payloadFingerprint": request.payload_fingerprint().0,
                }));
                control.next_response += 1;
                let response_id = format!("sdk-fixture-response-{}", control.next_response);
                prompt
                    .update(cx, |prompt, cx| {
                        if prompt.accepted_sdk_request().is_some() {
                            prompt.add_message(
                                crate::protocol::ChatPromptMessage::user(request.display_text()),
                                cx,
                            );
                        }
                        prompt.start_sdk_response(request, response_id, cx)
                    })
                    .map_err(anyhow::Error::msg)?;
                crate::runtime_policy::record_completed_fixture_effect();
            }
            SdkFixtureRequest::Stop => control.stop_requests += 1,
        }
    }
    Ok(())
}
