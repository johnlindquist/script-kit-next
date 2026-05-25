use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use agent_client_protocol::{ContentBlock, TextContent};

use super::executor::InlineAgentExecutor;
use super::types::{
    InlineAgentProviderEvent, InlineAgentProviderRequest, InlineAgentSessionId, InlineAgentTurnId,
};
use crate::ai::agent_chat::events::AgentChatEvent;
use crate::ai::agent_chat::launch::{
    resolve_focused_text_pi_launch, warm_session_manager, PiAgentChatLaunch,
};
use crate::ai::agent_chat::runtime::{AgentChatConnection, AgentChatTurnRequest};
use crate::ai::agent_chat::warm_session::{
    AgentChatWarmSessionLease, AgentChatWarmSessionSnapshot, AgentChatWarmSessionState,
};

const PREPARE_IN_PROGRESS_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const FIRST_AGENT_DELTA_TIMEOUT: Duration = Duration::from_secs(30);
const TOTAL_TURN_TIMEOUT: Duration = Duration::from_secs(180);
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub(crate) struct AgentChatInlineAgentExecutor {
    connection: Arc<dyn AgentChatConnection>,
    ui_thread_id: String,
    cwd: PathBuf,
    model_id: Option<String>,
    warm_key: String,
    warm_generation: u64,
    warm_lease: Arc<Mutex<Option<AgentChatWarmSessionLease>>>,
}

impl AgentChatInlineAgentExecutor {
    pub(crate) fn new(launch: PiAgentChatLaunch, lease: AgentChatWarmSessionLease) -> Self {
        Self {
            connection: lease.connection.clone(),
            ui_thread_id: lease.ui_thread_id.clone(),
            cwd: lease.cwd.clone(),
            model_id: launch.selected_model_id,
            warm_key: lease.key.clone(),
            warm_generation: lease.generation,
            warm_lease: Arc::new(Mutex::new(Some(lease))),
        }
    }

    fn release_warm_lease(lease: &Mutex<Option<AgentChatWarmSessionLease>>, reason: &'static str) {
        let lease = match lease.lock() {
            Ok(mut guard) => guard.take(),
            Err(error) => error.into_inner().take(),
        };
        let Some(lease) = lease else {
            return;
        };
        let key = lease.key.clone();
        let generation = lease.generation;
        match warm_session_manager().dismiss_reset(lease) {
            Ok(snapshot) => {
                tracing::info!(
                    target: "script_kit::inline_agent",
                    event = "inline_agent_pi_warm_dismiss_reset",
                    reason,
                    warm_key = %key,
                    generation,
                    replacement_generation = snapshot.generation,
                    replacement_state = ?snapshot.state,
                );
            }
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::inline_agent",
                    event = "inline_agent_pi_warm_dismiss_reset_failed",
                    reason,
                    warm_key = %key,
                    generation,
                    error = %error,
                );
            }
        }
    }
}

impl Drop for AgentChatInlineAgentExecutor {
    fn drop(&mut self) {
        Self::release_warm_lease(&self.warm_lease, "drop");
    }
}

pub(crate) fn spawn_default_agent_chat_inline_agent_executor(
) -> Result<AgentChatInlineAgentExecutor, String> {
    let (launch, prepared) = prepare_default_agent_chat_inline_agent_warm_session()?;
    let warm_key = launch.warm_key.clone();
    let prepared = wait_for_prepared_warm_session(warm_key.as_str(), prepared);

    if prepared.state != AgentChatWarmSessionState::Ready {
        return Err(format!(
            "Pi Inline Agent warm session was not ready (state={:?})",
            prepared.state
        ));
    }

    let lease = warm_session_manager()
        .acquire_warm(&warm_key)
        .ok_or_else(|| "Pi Inline Agent warm session was not ready".to_string())?;
    tracing::info!(
        target: "script_kit::inline_agent",
        event = "inline_agent_pi_warm_acquired",
        warm_key = %lease.key,
        generation = lease.generation,
        ui_thread_id = %lease.ui_thread_id,
    );

    Ok(AgentChatInlineAgentExecutor::new(launch, lease))
}

pub(crate) fn prepare_default_agent_chat_inline_agent_warm_session(
) -> Result<(PiAgentChatLaunch, AgentChatWarmSessionSnapshot), String> {
    let prefs = crate::config::load_user_preferences();
    let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
    let launch = resolve_focused_text_pi_launch(&prefs.ai, &ctx)
        .map_err(|error| format!("Failed to resolve Pi Text profile launch: {error}"))?;
    let warm_spec = launch.warm_spec();
    let prepared = warm_session_manager()
        .prepare_warm(warm_spec)
        .map_err(|error| format!("Failed to prepare warm Pi Inline Agent session: {error}"))?;
    tracing::info!(
        target: "script_kit::inline_agent",
        event = "inline_agent_pi_warm_prepared",
        warm_key = %prepared.key,
        generation = prepared.generation,
        state = ?prepared.state,
    );

    Ok((launch, prepared))
}

fn wait_for_prepared_warm_session(
    warm_key: &str,
    initial: AgentChatWarmSessionSnapshot,
) -> AgentChatWarmSessionSnapshot {
    if initial.state != AgentChatWarmSessionState::Preparing {
        return initial;
    }

    let started = Instant::now();
    while started.elapsed() < PREPARE_IN_PROGRESS_WAIT_TIMEOUT {
        if let Some(snapshot) = warm_session_manager().snapshot(warm_key) {
            if snapshot.state != AgentChatWarmSessionState::Preparing {
                return snapshot;
            }
        }
        std::thread::sleep(EVENT_POLL_INTERVAL);
    }

    warm_session_manager().snapshot(warm_key).unwrap_or(initial)
}

impl InlineAgentExecutor for AgentChatInlineAgentExecutor {
    fn start_turn(
        &self,
        request: InlineAgentProviderRequest,
    ) -> anyhow::Result<async_channel::Receiver<InlineAgentProviderEvent>> {
        let session_id = request.session_id.0.clone();
        let turn_id = request.turn_id.0.clone();
        let prompt_chars = request.prompt.chars().count();
        let submit_started = Instant::now();
        tracing::info!(
            target: "script_kit::inline_agent",
            event = "inline_agent_pi_start_turn_dispatch",
            session_id = %session_id,
            turn_id = %turn_id,
            warm_key = %self.warm_key,
            warm_generation = self.warm_generation,
            ui_thread_id = %self.ui_thread_id,
            prompt_chars,
        );

        let agent_chat_events = match self.connection.start_turn(AgentChatTurnRequest {
            ui_thread_id: self.ui_thread_id.clone(),
            cwd: self.cwd.clone(),
            blocks: vec![ContentBlock::Text(TextContent::new(request.prompt))],
            model_id: self.model_id.clone(),
        }) {
            Ok(events) => events,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::inline_agent",
                    event = "inline_agent_pi_start_turn_dispatch_failed",
                    session_id = %session_id,
                    turn_id = %turn_id,
                    warm_key = %self.warm_key,
                    warm_generation = self.warm_generation,
                    elapsed_ms = submit_started.elapsed().as_millis() as u64,
                );
                Self::release_warm_lease(&self.warm_lease, "start_turn_failed");
                return Err(error);
            }
        };

        let (provider_tx, provider_rx) = async_channel::bounded(256);
        let warm_lease = self.warm_lease.clone();
        let warm_key = self.warm_key.clone();
        let warm_generation = self.warm_generation;
        let connection = self.connection.clone();
        let ui_thread_id = self.ui_thread_id.clone();
        std::thread::spawn(move || {
            let mut released = false;
            let mut first_agent_delta_logged = false;
            loop {
                let event = match agent_chat_events.try_recv() {
                    Ok(event) => event,
                    Err(async_channel::TryRecvError::Empty) => {
                        let elapsed = submit_started.elapsed();
                        let timeout =
                            if !first_agent_delta_logged && elapsed >= FIRST_AGENT_DELTA_TIMEOUT {
                                Some((
                                    "first_agent_delta_timeout",
                                    "Pi Inline Agent response timed out",
                                ))
                            } else if elapsed >= TOTAL_TURN_TIMEOUT {
                                Some(("total_turn_timeout", "Pi Inline Agent turn timed out"))
                            } else {
                                None
                            };

                        if let Some((timeout_kind, message)) = timeout {
                            tracing::warn!(
                                target: "script_kit::inline_agent",
                                event = "inline_agent_pi_turn_timeout",
                                session_id = %session_id,
                                turn_id = %turn_id,
                                warm_key = %warm_key,
                                warm_generation,
                                timeout_kind,
                                elapsed_ms = elapsed.as_millis() as u64,
                                first_agent_delta_logged,
                            );
                            let _ = connection.cancel_turn(ui_thread_id.clone());
                            let _ = provider_tx.send_blocking(InlineAgentProviderEvent::Failed {
                                message: message.to_string(),
                            });
                            AgentChatInlineAgentExecutor::release_warm_lease(
                                &warm_lease,
                                timeout_kind,
                            );
                            released = true;
                            break;
                        }

                        std::thread::sleep(EVENT_POLL_INTERVAL);
                        continue;
                    }
                    Err(async_channel::TryRecvError::Closed) => break,
                };
                let Some(provider_event) = map_agent_chat_event(event) else {
                    continue;
                };
                let terminal = matches!(
                    provider_event,
                    InlineAgentProviderEvent::TurnFinished
                        | InlineAgentProviderEvent::Failed { .. }
                );
                if let InlineAgentProviderEvent::AgentMessageDelta { text } = &provider_event {
                    if !first_agent_delta_logged && !text.is_empty() {
                        first_agent_delta_logged = true;
                        tracing::info!(
                            target: "script_kit::inline_agent",
                            event = "inline_agent_pi_first_agent_delta",
                            session_id = %session_id,
                            turn_id = %turn_id,
                            warm_key = %warm_key,
                            warm_generation,
                            elapsed_ms = submit_started.elapsed().as_millis() as u64,
                            delta_chars = text.chars().count(),
                        );
                    }
                }
                if terminal {
                    let terminal_kind = match &provider_event {
                        InlineAgentProviderEvent::TurnFinished => "finished",
                        InlineAgentProviderEvent::Failed { .. } => "failed",
                        _ => "unknown",
                    };
                    tracing::info!(
                        target: "script_kit::inline_agent",
                        event = "inline_agent_pi_turn_terminal",
                        session_id = %session_id,
                        turn_id = %turn_id,
                        warm_key = %warm_key,
                        warm_generation,
                        terminal_kind,
                        elapsed_ms = submit_started.elapsed().as_millis() as u64,
                        first_agent_delta_logged,
                    );
                }
                if provider_tx.send_blocking(provider_event).is_err() {
                    AgentChatInlineAgentExecutor::release_warm_lease(
                        &warm_lease,
                        "receiver_closed",
                    );
                    released = true;
                    break;
                }
                if terminal {
                    AgentChatInlineAgentExecutor::release_warm_lease(&warm_lease, "terminal_event");
                    released = true;
                    break;
                }
            }
            if !released {
                AgentChatInlineAgentExecutor::release_warm_lease(
                    &warm_lease,
                    "event_stream_closed",
                );
            }
        });

        Ok(provider_rx)
    }

    fn cancel_turn(
        &self,
        _session_id: InlineAgentSessionId,
        _turn_id: InlineAgentTurnId,
    ) -> anyhow::Result<()> {
        let result = self.connection.cancel_turn(self.ui_thread_id.clone());
        Self::release_warm_lease(&self.warm_lease, "cancel_turn");
        result
    }
}

fn map_agent_chat_event(event: AgentChatEvent) -> Option<InlineAgentProviderEvent> {
    match event {
        AgentChatEvent::AgentMessageDelta(text) => {
            Some(InlineAgentProviderEvent::AgentMessageDelta { text })
        }
        AgentChatEvent::AgentThoughtDelta(text) => {
            Some(InlineAgentProviderEvent::AgentThoughtDelta { text })
        }
        AgentChatEvent::UsageUpdated { .. } => Some(InlineAgentProviderEvent::UsageUpdated),
        AgentChatEvent::TurnFinished { .. } => Some(InlineAgentProviderEvent::TurnFinished),
        AgentChatEvent::Failed { error } => {
            Some(InlineAgentProviderEvent::Failed { message: error })
        }
        AgentChatEvent::SetupRequired { reason, .. } => {
            Some(InlineAgentProviderEvent::Failed { message: reason })
        }
        AgentChatEvent::UserMessageDelta(_)
        | AgentChatEvent::ToolCallStarted { .. }
        | AgentChatEvent::ToolCallUpdated { .. }
        | AgentChatEvent::PlanUpdated { .. }
        | AgentChatEvent::AvailableCommandsUpdated { .. }
        | AgentChatEvent::ModeChanged { .. }
        | AgentChatEvent::ModelsAvailable { .. } => None,
    }
}
