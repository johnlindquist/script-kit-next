use super::conversation_fixtures as chat;
use super::runtime::{Evaluator, Mounted, RootOwner, BATCH_STEP_REQUEST_ID_PREFIX};
use crate::protocol::{
    AgentChatFixtureCommand, AutomationTargetIdentitySnapshot, AutomationWindowTarget,
    FixtureControl, FlowFixtureCommand, SdkChatFixtureCommand,
};
use crate::AppView;
use anyhow::{ensure, Context as _, Result};
use gpui::Entity;
use serde_json::{json, Value};
use std::time::Duration;

struct ActionBaseline {
    identity: AutomationTargetIdentitySnapshot,
    observation: Value,
    windows: std::collections::BTreeSet<(String, u64)>,
    completion: Option<crate::prompt_completion::PromptCompletionBinding>,
    submission_sequence: Option<u64>,
}

impl Evaluator {
    pub(super) fn act(&mut self, request_id: &str, raw: &Value) -> Result<Value> {
        let target: AutomationWindowTarget = serde_json::from_value(raw["target"].clone())?;
        let expected: AutomationTargetIdentitySnapshot =
            serde_json::from_value(raw["expected"].clone())?;
        self.validate_expected(&target, &expected)?;
        // Ordinary GPUI event handling lives on Main, including standalone
        // Notes/Chat. Refuse stale targets before constructing that dispatcher.
        if raw["type"] == "simulateGpuiEvent" && self.main.is_none() {
            self.ensure_main()?;
        }
        let mounted = self.resolve(&target)?.clone();
        let baseline = self.begin_action(&mounted, &target)?;
        let mut reply = match raw["type"].as_str() {
            Some("batch") => self.registered_batch(request_id, &mounted, raw)?,
            Some("simulateGpuiEvent") => self.forward_main(request_id, raw, true)?,
            _ => anyhow::bail!("owned_action_not_permitted"),
        };
        self.drain_sdk_requests()?;
        self.tick(true)?;
        // Dispatch completion is deliberately not advertised as handler activation.
        // Changed state and immutable sink receipts remain the activation evidence.
        if self.resolve(&target).is_ok() {
            reply["targetIdentity"] = serde_json::to_value(self.identity(&target)?)?;
            reply["fixtureObservation"] = self.fixture_observation(&mounted)?;
        }
        let mut receipt = self.finish_action(
            request_id,
            raw["type"].as_str().unwrap_or("action"),
            &mounted,
            &target,
            baseline,
        )?;
        if raw["type"] == "simulateGpuiEvent" {
            receipt.was_deferred = reply["wasDeferred"].as_bool().unwrap_or(false);
            receipt.dispatch_completed = reply["dispatchCompleted"].as_bool().unwrap_or(false);
        }
        if reply["success"] == false
            && matches!(receipt.effect, crate::protocol::ObservedEffect::NoOp { .. })
        {
            let code = reply["results"]
                .as_array()
                .and_then(|results| {
                    results
                        .iter()
                        .find_map(|result| result["error"]["code"].as_str())
                })
                .or_else(|| reply["errorCode"].as_str())
                .unwrap_or("action_failed");
            receipt.effect = crate::protocol::ObservedEffect::Refused { code: code.into() };
            receipt.dispatch_completed = false;
        }
        reply["actionReceipt"] = serde_json::to_value(receipt)?;
        Ok(reply)
    }

    pub(super) fn drain_sdk_requests(&mut self) -> Result<()> {
        let Some(main) = self.main.clone() else {
            return Ok(());
        };
        let mut cx = self.cx.app.borrow_mut();
        let prompt = match &main.read(&cx).current_view {
            AppView::ChatPrompt { entity, .. } => entity.clone(),
            _ => return Ok(()),
        };
        if let Some(control) = self.sdk_controls.get_mut("main") {
            chat::drain_sdk_chat_fixture(&prompt, control, &mut cx)?;
        }
        Ok(())
    }

    pub(super) fn fixture_control(
        &mut self,
        request_id: &str,
        target: &AutomationWindowTarget,
        expected: &AutomationTargetIdentitySnapshot,
        control: FixtureControl,
    ) -> Result<Value> {
        self.validate_expected(target, expected)?;
        let mounted = self.resolve(target)?.clone();
        let baseline = self.begin_action(&mounted, target)?;
        let mut explicit_observation = None;
        match control {
            FixtureControl::Search(command) => {
                explicit_observation = Some(self.search_fixture_control(&mounted, command)?);
            }
            FixtureControl::AgentChat(command) => {
                let view = self.agent_chat_owner(&mounted)?;
                if let AgentChatFixtureCommand::MutateInputBeforePaint { text } = command {
                    let before = self.identity(target)?;
                    view.update(&mut **self.cx.app.borrow_mut(), |view, cx| {
                        view.set_input(text, cx)
                    });
                    let after = self.identity(target)?;
                    let request = crate::computer_use::runtime_bridge::ComputerUseCaptureRenderWindowRequest {
                        target: target.clone(), expected: Some(before.clone()), hi_dpi: true,
                        include_image: false, correlation_id: request_id.into(),
                        probes: Vec::new(),
                    };
                    let capture = match crate::computer_use::gpui_runtime_bridge::capture_render_window_on_gpui_thread(&request, &mut self.cx.app.borrow_mut()) {
                        Ok(snapshot) => serde_json::to_value(snapshot)?,
                        Err(error) => json!({"errorCode":error.error_code()}),
                    };
                    explicit_observation =
                        Some(json!({"before":before,"after":after,"oldCapture":capture,
                        "owner":self.fixture_observation(&mounted)?}));
                } else {
                    let action = match command {
                        AgentChatFixtureCommand::Submit { text } => {
                            chat::AgentChatFixtureAction::Submit { text }
                        }
                        AgentChatFixtureCommand::Retry {} => chat::AgentChatFixtureAction::Retry,
                        AgentChatFixtureCommand::Stop {} => chat::AgentChatFixtureAction::Stop,
                        AgentChatFixtureCommand::EmitText {
                            turn_generation,
                            text,
                        } => chat::AgentChatFixtureAction::EmitText {
                            turn_generation,
                            text,
                        },
                        AgentChatFixtureCommand::Complete { turn_generation } => {
                            chat::AgentChatFixtureAction::Complete { turn_generation }
                        }
                        AgentChatFixtureCommand::Fail { turn_generation } => {
                            chat::AgentChatFixtureAction::Fail { turn_generation }
                        }
                        AgentChatFixtureCommand::OpenHistory {} => {
                            chat::AgentChatFixtureAction::OpenHistory
                        }
                        AgentChatFixtureCommand::OpenSlashPicker {} => {
                            chat::AgentChatFixtureAction::OpenSlashPicker
                        }
                        AgentChatFixtureCommand::OpenProfilePicker {} => {
                            chat::AgentChatFixtureAction::OpenProfilePicker
                        }
                        AgentChatFixtureCommand::HoldDrain {} => {
                            chat::AgentChatFixtureAction::HoldDrain
                        }
                        AgentChatFixtureCommand::RetainDrain {} => {
                            chat::AgentChatFixtureAction::RetainDrain
                        }
                        AgentChatFixtureCommand::ReleaseDrain { turn_generation } => {
                            chat::AgentChatFixtureAction::ReleaseDrain { turn_generation }
                        }
                        AgentChatFixtureCommand::MutateInputBeforePaint { .. } => unreachable!(),
                    };
                    mounted
                        .handle
                        .update(&mut **self.cx.app.borrow_mut(), |_, window, cx| {
                            chat::drive_agent_chat_fixture(&view, action, window, cx)
                        })??;
                }
            }
            FixtureControl::Notes(_) => anyhow::bail!("no_notes_window"),
            FixtureControl::Flow(command) => {
                let RootOwner::Main(main) = &mounted.owner else {
                    anyhow::bail!("flow_host_required");
                };
                let (session_id, action) = match command {
                    FlowFixtureCommand::Submit { session_id, text } => {
                        (session_id, chat::FlowFixtureAction::Submit { text })
                    }
                    FlowFixtureCommand::Retry { session_id } => {
                        (session_id, chat::FlowFixtureAction::Retry)
                    }
                    FlowFixtureCommand::Stop { session_id } => {
                        (session_id, chat::FlowFixtureAction::Stop)
                    }
                    FlowFixtureCommand::Background { session_id } => {
                        (session_id, chat::FlowFixtureAction::Background)
                    }
                    FlowFixtureCommand::Resume { session_id } => {
                        (session_id, chat::FlowFixtureAction::Resume)
                    }
                    FlowFixtureCommand::EmitText {
                        session_id,
                        message_id,
                        text,
                    } => (
                        session_id,
                        chat::FlowFixtureAction::Text {
                            expected_message_id: message_id,
                            text,
                        },
                    ),
                    FlowFixtureCommand::Complete {
                        session_id,
                        message_id,
                    } => (
                        session_id,
                        chat::FlowFixtureAction::Complete {
                            expected_message_id: message_id,
                        },
                    ),
                    FlowFixtureCommand::Fail {
                        session_id,
                        message_id,
                    } => (
                        session_id,
                        chat::FlowFixtureAction::Fail {
                            expected_message_id: message_id,
                        },
                    ),
                };
                mounted
                    .handle
                    .update(&mut **self.cx.app.borrow_mut(), |_, window, cx| {
                        main.update(cx, |main, cx| {
                            ensure!(
                                main.conversations
                                    .flow_sessions
                                    .iter()
                                    .any(|(meta, _)| meta.id == session_id),
                                "flow_session_missing"
                            );
                            main.drive_flow_fixture(session_id, action, window, cx)
                        })
                    })??;
            }
            FixtureControl::SdkChat(command) => {
                let RootOwner::Main(main) = &mounted.owner else {
                    anyhow::bail!("sdk_chat_host_required");
                };
                let mut cx = self.cx.app.borrow_mut();
                let AppView::ChatPrompt { entity, .. } = &main.read(&cx).current_view else {
                    anyhow::bail!("sdk_chat_not_mounted");
                };
                let prompt = entity.clone();
                let control = self
                    .sdk_controls
                    .get_mut(&mounted.info.id)
                    .context("sdk_fixture_control_missing")?;
                let action = match command {
                    SdkChatFixtureCommand::Submit { text } => {
                        chat::SdkChatFixtureAction::Submit(text)
                    }
                    SdkChatFixtureCommand::Retry {} => chat::SdkChatFixtureAction::Retry,
                    SdkChatFixtureCommand::Stop {} => chat::SdkChatFixtureAction::Stop,
                    SdkChatFixtureCommand::EmitText { message_id, text } => {
                        chat::SdkChatFixtureAction::Text { message_id, text }
                    }
                    SdkChatFixtureCommand::Complete { message_id } => {
                        chat::SdkChatFixtureAction::Complete { message_id }
                    }
                    SdkChatFixtureCommand::Fail { message_id } => {
                        chat::SdkChatFixtureAction::Fail { message_id }
                    }
                };
                chat::drive_sdk_chat_fixture(&prompt, control, action, &mut cx)?;
            }
            FixtureControl::Theme(command) => {
                ensure!(
                    mounted.fixture_id == "main.theme-chooser",
                    "theme_chooser_fixture_required"
                );
                let RootOwner::Main(main) = &mounted.owner else {
                    anyhow::bail!("theme_chooser_host_required");
                };
                ensure!(
                    matches!(
                        &main.read(&self.cx.app.borrow()).current_view,
                        AppView::ThemeChooserView { .. }
                    ),
                    "theme_chooser_not_mounted"
                );
                explicit_observation = Some(
                    self.theme_fixture
                        .control(command, &mut self.cx.app.borrow_mut())?,
                );
            }
            FixtureControl::OneWindow(crate::protocol::OneWindowFixtureCommand::TalkTo {
                destination,
            }) => {
                let RootOwner::Main(main) = &mounted.owner else {
                    anyhow::bail!("one_window_host_required");
                };
                let dest = match destination {
                    crate::protocol::OneWindowTalkDestination::Chat => sk_one_window::Dest::Chat,
                    crate::protocol::OneWindowTalkDestination::Note => sk_one_window::Dest::Note,
                };
                main.update(&mut **self.cx.app.borrow_mut(), |app, cx| {
                    app.one_window_talk_began(false, Some(dest), cx)
                });
            }
            FixtureControl::Fault {
                operation: crate::protocol::ThemeFaultOperation::SuppressThemeNotification,
                target: fault_target,
            } => {
                let fault = self.resolve(&fault_target)?;
                crate::windows::automation_runtime_handles::suppress_next_theme_notification(
                    &fault.info.id,
                    fault.info.generation.context("fault_generation_missing")?,
                )?;
                explicit_observation = Some(json!({"suppressed":fault_target}));
            }
        }
        self.tick(true)?;
        let observation = match explicit_observation {
            Some(observation) => observation,
            None => self.fixture_observation(&mounted)?,
        };
        let action_receipt =
            self.finish_action(request_id, "fixtureControl", &mounted, target, baseline)?;
        Ok(
            json!({"operation":"fixtureControl","ok":true,"target":self.identity(target)?,
            "observation":observation,"actionReceipt":action_receipt}),
        )
    }

    fn main_batch_command(
        &mut self,
        request_id: &str,
        mounted: &Mounted,
        command: &crate::protocol::BatchCommand,
        expected: &AutomationTargetIdentitySnapshot,
        timeout: Duration,
    ) -> Result<Option<String>> {
        let RootOwner::Main(main) = &mounted.owner else {
            anyhow::bail!("main_owner_required");
        };
        let handled = crate::windows::with_runtime_window_dispatch(mounted.handle, || {
            mounted
                .handle
                .update(&mut **self.cx.app.borrow_mut(), |_, window, cx| {
                    let actual = Self::snapshot_for(mounted, window, cx)?;
                    crate::validate_gpui_expected_identity(expected, &actual)
                        .map_err(anyhow::Error::msg)?;
                    main.update(cx, |app, cx| match command {
                        crate::protocol::BatchCommand::SelectBySemanticId {
                            semantic_id,
                            submit,
                        } => super::main_fixtures::select_main_overlay_element(
                            app,
                            semantic_id,
                            *submit,
                            window,
                            cx,
                        ),
                        _ => Ok(false),
                    })
                })
        })??;
        if handled {
            return Ok(match command {
                crate::protocol::BatchCommand::SelectBySemanticId { semantic_id, .. } => {
                    Some(semantic_id.clone())
                }
                _ => None,
            });
        }
        if let crate::protocol::BatchCommand::SelectBySemanticId {
            semantic_id,
            submit,
        } = command
        {
            let app_cx = self.cx.app.borrow().to_async();
            let selected = app_cx.update(|cx| {
                let actual = mounted
                    .handle
                    .update(cx, |_, window, cx| Self::snapshot_for(mounted, window, cx))??;
                crate::validate_gpui_expected_identity(expected, &actual)
                    .map_err(anyhow::Error::msg)?;
                crate::windows::with_runtime_window_dispatch(mounted.handle, || {
                    crate::apply_registered_root_layer_selection(
                        &mounted.info,
                        semantic_id,
                        *submit,
                        cx,
                    )
                })
            })?;
            if selected.is_some() {
                return Ok(selected);
            }
        }
        // Direct batch commands must not activate controls below a modal Root
        // layer. Real GPUI keys continue through the actual modal focus route.
        let covered = mounted
            .handle
            .update(&mut **self.cx.app.borrow_mut(), |_, window, cx| {
                !gpui_component::Root::read(window, cx)
                    .layer_snapshot(cx)
                    .dialogs
                    .is_empty()
            })?;
        ensure!(!covered, "covered_by_root_dialog");
        let request = json!({"type":"batch","requestId":request_id,"target":Self::instance(mounted)?,
            "expected":expected,"commands":[command],"options":{"stopOnError":true,"timeout":timeout.as_millis() as u64}});
        let reply = self.forward_main(request_id, &request, true)?;
        let entry: crate::protocol::BatchResultEntry = serde_json::from_value(
            reply["results"]
                .as_array()
                .and_then(|results| results.first())
                .context("main_batch_step_result_missing")?
                .clone(),
        )?;
        if !entry.success {
            return Err(entry.error.context("main_batch_step_error_missing")?.into());
        }
        Ok(entry.value)
    }

    fn registered_batch(
        &mut self,
        request_id: &str,
        mounted: &Mounted,
        raw: &Value,
    ) -> Result<Value> {
        let commands: Vec<crate::protocol::BatchCommand> =
            serde_json::from_value(raw["commands"].clone())?;
        let expected: AutomationTargetIdentitySnapshot =
            serde_json::from_value(raw["expected"].clone())?;
        ensure!(commands.len() <= 256, "batch_command_limit");
        let options = raw.get("options").cloned().unwrap_or_else(|| json!({}));
        let stop_on_error = options["stopOnError"].as_bool().unwrap_or(true);
        let timeout = Duration::from_millis(options["timeout"].as_u64().unwrap_or(5_000));
        ensure!(
            timeout <= Duration::from_millis(self.bootstrap.limits.max_lifetime_ms),
            "batch_timeout_exceeds_session_limit"
        );
        let started = std::time::Instant::now();
        ensure!(
            mounted.info.generation.is_some(),
            "window_generation_missing"
        );
        let target = Self::instance(mounted)?;
        let mut results = Vec::with_capacity(commands.len());
        let mut failed_at = None;
        for (index, command) in commands.iter().enumerate() {
            let command_started = std::time::Instant::now();
            let name = serde_json::to_value(command)?["type"]
                .as_str()
                .unwrap_or("unknown")
                .to_owned();
            let result = (|| -> Result<Option<String>> {
                ensure!(started.elapsed() < timeout, "batch_timeout");
                self.validate_expected(&target, &expected)?;
                if let crate::protocol::BatchCommand::WaitFor {
                    condition,
                    timeout: wait_timeout,
                    poll_interval,
                } = command
                {
                    let step_id = format!("{BATCH_STEP_REQUEST_ID_PREFIX}{index}:{request_id}");
                    let remaining = timeout.saturating_sub(started.elapsed()).as_millis() as u64;
                    let request = json!({"type":"waitFor","requestId":step_id,"target":target,
                        "condition":condition,"timeout":wait_timeout.unwrap_or(5_000).min(remaining),"pollInterval":poll_interval.unwrap_or(25)});
                    let response = self.wait_registered(&step_id, mounted, &request)?;
                    if response["success"] != true {
                        let error: crate::protocol::TransactionError =
                            serde_json::from_value(response["error"].clone())?;
                        return Err(error.into());
                    }
                    return Ok(None);
                }
                if matches!(mounted.owner, RootOwner::Main(_)) {
                    // Main's transaction transport rejects any reused ID, even
                    // after completion. Each internal step owns a distinct reply.
                    let step_id = format!("{BATCH_STEP_REQUEST_ID_PREFIX}{index}:{request_id}");
                    return self.main_batch_command(
                        &step_id,
                        mounted,
                        command,
                        &expected,
                        timeout.saturating_sub(started.elapsed()),
                    );
                }
                crate::windows::with_runtime_window_dispatch(mounted.handle, || {
                    crate::apply_registered_surface_command(
                        &mounted.info,
                        command,
                        &mut self.cx.app.borrow_mut(),
                    )
                })
            })();
            let success = result.is_ok();
            let (value, error) = match result {
                Ok(value) => (value, None),
                Err(error) => (
                    None,
                    Some(
                        error
                            .downcast_ref::<crate::protocol::TransactionError>()
                            .cloned()
                            .unwrap_or_else(|| {
                                crate::protocol::TransactionError::action_failed(error.to_string())
                            }),
                    ),
                ),
            };
            if !success && failed_at.is_none() {
                failed_at = Some(index);
            }
            results.push(crate::protocol::BatchResultEntry {
                index,
                success,
                command: name,
                elapsed: Some(command_started.elapsed().as_millis() as u64),
                value,
                error,
            });
            self.drain_sdk_requests()?;
            self.tick(true)?;
            if !success && stop_on_error {
                break;
            }
        }
        serde_json::to_value(crate::protocol::Message::batch_result(
            request_id.into(),
            failed_at.is_none(),
            results,
            failed_at,
            started.elapsed().as_millis() as u64,
        ))
        .map_err(Into::into)
    }

    fn begin_action(
        &mut self,
        mounted: &Mounted,
        target: &AutomationWindowTarget,
    ) -> Result<ActionBaseline> {
        let completion = match &mounted.owner {
            RootOwner::Main(main) => main.read(&self.cx.app.borrow()).prompt_completion.clone(),
            _ => None,
        };
        let submission_sequence = completion.as_ref().and_then(|binding| {
            binding
                .observation()
                .receipt
                .map(|receipt| receipt.sequence)
        });
        Ok(ActionBaseline {
            identity: self.identity(target)?,
            observation: self.fixture_observation(mounted)?,
            windows: self
                .mounted
                .values()
                .filter_map(|window| {
                    window
                        .info
                        .generation
                        .map(|generation| (window.info.id.clone(), generation))
                })
                .collect(),
            completion,
            submission_sequence,
        })
    }

    fn finish_action(
        &mut self,
        request_id: &str,
        operation: &str,
        mounted: &Mounted,
        target: &AutomationWindowTarget,
        baseline: ActionBaseline,
    ) -> Result<crate::protocol::ScopedActionReceipt> {
        use crate::protocol::ObservedEffect;
        let after = if self.resolve(target).is_ok() {
            Some(self.identity(target)?)
        } else {
            None
        };
        let observation = if after.is_none() && matches!(mounted.owner, RootOwner::Main(_)) {
            Value::Null
        } else {
            self.fixture_observation(mounted)?
        };
        let mut delivered = None;
        if let Some(binding) = baseline.completion {
            if let Some(receipt) = binding.observation().receipt {
                if Some(receipt.sequence) != baseline.submission_sequence {
                    delivered = Some(ObservedEffect::SubmissionDelivered {
                        owner: mounted.info.id.clone(),
                        receipt_id: format!(
                            "{}:{}:{}",
                            receipt.prompt.id, receipt.prompt.generation, receipt.sequence
                        ),
                        prompt_instance_id: format!(
                            "{}:{}",
                            receipt.prompt.id, receipt.prompt.generation
                        ),
                        delivery_count: receipt.sequence,
                    });
                }
            }
        }
        if delivered.is_none() {
            for field in ["deliveredActionIds", "acceptedRequests", "completedActions"] {
                let before_count = baseline.observation[field].as_array().map_or(0, Vec::len);
                if let Some(values) = observation[field]
                    .as_array()
                    .filter(|values| values.len() > before_count)
                {
                    if let Some(id) = values.last().and_then(Value::as_str) {
                        delivered = Some(ObservedEffect::SubmissionDelivered {
                            owner: mounted.info.id.clone(),
                            receipt_id: id.into(),
                            prompt_instance_id: format!(
                                "{}:{}",
                                mounted.info.id,
                                baseline.identity.window_generation.unwrap_or(0)
                            ),
                            delivery_count: values.len() as u64,
                        });
                    }
                }
            }
        }
        if delivered.is_none()
            && ["completed", "actionCompleted"]
                .iter()
                .any(|field| observation[*field] == true && baseline.observation[*field] != true)
        {
            delivered = Some(ObservedEffect::SubmissionDelivered {
                owner: mounted.info.id.clone(),
                receipt_id: format!(
                    "{}:{}:completion",
                    mounted.info.id,
                    baseline.identity.window_generation.unwrap_or(0)
                ),
                prompt_instance_id: format!(
                    "{}:{}",
                    mounted.info.id,
                    baseline.identity.window_generation.unwrap_or(0)
                ),
                delivery_count: 1,
            });
        }
        let effect = if let Some(delivered) = delivered {
            delivered
        } else if after.is_none() {
            if mounted.info.parent_window_id.is_some() {
                ObservedEffect::PopupClosed {
                    target: target.clone(),
                }
            } else {
                ObservedEffect::RootClosed {
                    target: target.clone(),
                }
            }
        } else if let Some(opened) = self.mounted.values().find(|window| {
            window.info.parent_window_id.as_deref() == Some(&mounted.info.id)
                && window.info.generation.is_some_and(|generation| {
                    !baseline
                        .windows
                        .contains(&(window.info.id.clone(), generation))
                })
        }) {
            ObservedEffect::PopupOpened {
                target: Self::instance(opened)?,
            }
        } else if let Some(after) = after.as_ref().filter(|after| {
            after.data_generation != baseline.identity.data_generation
                || after.surface_generation != baseline.identity.surface_generation
                || after.presentation_revision != baseline.identity.presentation_revision
        }) {
            ObservedEffect::StateChanged {
                owner: mounted.info.id.clone(),
                revision: after.data_generation,
            }
        } else {
            ObservedEffect::NoOp {
                reason: "no_owner_change_observed".into(),
            }
        };
        Ok(crate::protocol::ScopedActionReceipt {
            request_id: request_id.into(),
            operation_id: format!("{request_id}:{operation}"),
            before: baseline.identity,
            after,
            dispatch_completed: true,
            was_deferred: false,
            effect,
        })
    }

    pub(super) fn agent_chat_owner(
        &self,
        mounted: &Mounted,
    ) -> Result<Entity<crate::ai::agent_chat::ui::AgentChatView>> {
        match &mounted.owner {
            RootOwner::AgentChat(view) => Ok(view.clone()),
            RootOwner::Main(main) => match &main.read(&self.cx.app.borrow()).current_view {
                AppView::AgentChatView { entity } => Ok(entity.clone()),
                _ => anyhow::bail!("agent_chat_not_mounted"),
            },
            _ => anyhow::bail!("agent_chat_host_required"),
        }
    }

    pub(super) fn mount_auxiliary(
        &mut self,
        fixture_id: &str,
        family: &str,
        parent: Option<&AutomationWindowTarget>,
    ) -> Result<Mounted> {
        let mut parent = match parent {
            Some(target) => self.resolve(target)?.clone(),
            None => self.ensure_main()?,
        };
        if family == "agentChatPopup" {
            if self.agent_chat_owner(&parent).is_err() {
                let RootOwner::Main(main) = &parent.owner else {
                    anyhow::bail!("agent_chat_popup_parent_required");
                };
                parent.handle.update(
                    &mut **self.cx.app.borrow_mut(),
                    |_, window, cx| -> Result<()> {
                        let view = chat::create_agent_chat_fixture(
                            "agent-chat.standard.populated",
                            window,
                            cx,
                        )?;
                        main.update(cx, |main, cx| {
                            main.transition_current_view_and_rekey_main_automation_surface(
                                AppView::AgentChatView { entity: view },
                            );
                            main.bind_owned_surface_revision_observers(cx);
                            cx.notify();
                        });
                        Ok(())
                    },
                )??;
            }
            let view = self.agent_chat_owner(&parent)?;
            parent
                .handle
                .update(&mut **self.cx.app.borrow_mut(), |_, window, cx| {
                    chat::open_agent_chat_popup_fixture(fixture_id, &view, window, cx)
                })??;
            self.tick(true)?;
            if fixture_id != "agent-chat.popup.history" {
                parent.fixture_id = fixture_id.into();
                return Ok(parent);
            }
            let info = crate::windows::list_automation_windows().into_iter().find(|info| {
                info.parent_window_id.as_deref() == Some(&parent.info.id)
                    && info.parent_window_generation == parent.info.generation
                    && info.id == crate::ai::agent_chat::ui::history_popup::AGENT_CHAT_HISTORY_POPUP_AUTOMATION_ID
            }).context("history_popup_registration_missing")?;
            let handle = crate::windows::get_runtime_window_handle_for_generation(
                &info.id,
                info.generation.context("popup_generation_missing")?,
            )
            .context("popup_runtime_missing")?;
            return Ok(Mounted {
                fixture_id: fixture_id.into(),
                info,
                handle,
                owner: RootOwner::ConversationPopup,
            });
        }
        anyhow::bail!("unknown_auxiliary_family")
    }
}
