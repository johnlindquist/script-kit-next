//! Transaction flight recorder executor for `waitFor` and `batch` commands.
//!
//! Executes deterministic UI transactions against a [`TransactionStateProvider`],
//! producing per-command receipts with before/after snapshots, poll observations,
//! elapsed timings, and actionable failure suggestions.

use crate::protocol::transaction_trace::{
    append_transaction_trace, now_epoch_ms, should_include_trace,
};
use crate::protocol::types::batch_wait::{
    BatchCommand, BatchOptions, BatchResultEntry, StateMatchSpec, TransactionCommandTrace,
    TransactionError, TransactionErrorCode, TransactionTrace, TransactionTraceMode,
    TransactionTraceStatus, UiStateSnapshot, WaitCondition, WaitDetailedCondition,
    WaitNamedCondition, WaitPollObservation,
};
use anyhow::Result;
use std::path::Path;
use std::time::{Duration, Instant};

// ── Default constants ──────────────────────────────────────────────────────

const DEFAULT_WAIT_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_WAIT_POLL_INTERVAL_MS: u64 = 25;

// ── Provider trait ─────────────────────────────────────────────────────────

/// Abstraction over the live UI state, allowing the executor to be tested
/// without a running GPUI window.
pub trait TransactionStateProvider {
    /// Take a snapshot of the current UI state.
    fn snapshot(&self) -> UiStateSnapshot;
    /// Set the input/filter field text.
    fn set_input(&mut self, text: &str) -> Result<()>;
    /// Select a choice by value, optionally submitting. Returns the matched
    /// value or `None` if no choice matched.
    fn select_by_value(&mut self, value: &str, submit: bool) -> Result<Option<String>>;
}

// ── Condition matching ─────────────────────────────────────────────────────

fn matches_state(snapshot: &UiStateSnapshot, spec: &StateMatchSpec) -> bool {
    if let Some(ref expected) = spec.input_value {
        if snapshot.input_value.as_deref() != Some(expected.as_str()) {
            return false;
        }
    }
    if let Some(ref expected) = spec.selected_value {
        if snapshot.selected_value.as_deref() != Some(expected.as_str()) {
            return false;
        }
    }
    if let Some(expected) = spec.window_visible {
        if snapshot.window_visible != expected {
            return false;
        }
    }
    // prompt_type is not in UiStateSnapshot, skip it
    true
}

fn matches_condition(snapshot: &UiStateSnapshot, condition: &WaitCondition) -> (bool, Vec<String>) {
    match condition {
        WaitCondition::Named(WaitNamedCondition::ChoicesRendered) => {
            (snapshot.choice_count > 0, Vec::new())
        }
        WaitCondition::Named(WaitNamedCondition::InputEmpty) => (
            snapshot.input_value.as_deref().unwrap_or("").is_empty(),
            Vec::new(),
        ),
        WaitCondition::Named(WaitNamedCondition::WindowVisible) => {
            (snapshot.window_visible, Vec::new())
        }
        WaitCondition::Named(WaitNamedCondition::WindowFocused) => {
            (snapshot.window_focused, Vec::new())
        }
        WaitCondition::Detailed(WaitDetailedCondition::ElementExists { semantic_id })
        | WaitCondition::Detailed(WaitDetailedCondition::ElementVisible { semantic_id }) => {
            let matched: Vec<String> = snapshot
                .visible_semantic_ids
                .iter()
                .filter(|id| *id == semantic_id)
                .cloned()
                .collect();
            (!matched.is_empty(), matched)
        }
        WaitCondition::Detailed(WaitDetailedCondition::ElementFocused { semantic_id }) => {
            let ok = snapshot.focused_semantic_id.as_deref() == Some(semantic_id.as_str());
            (
                ok,
                if ok {
                    vec![semantic_id.clone()]
                } else {
                    Vec::new()
                },
            )
        }
        WaitCondition::Detailed(WaitDetailedCondition::StateMatch { state }) => {
            (matches_state(snapshot, state), Vec::new())
        }
    }
}

fn build_wait_suggestion(condition: &WaitCondition, snapshot: &UiStateSnapshot) -> Option<String> {
    match condition {
        WaitCondition::Named(WaitNamedCondition::ChoicesRendered) if snapshot.choice_count == 0 => {
            Some(
                "No choices were visible at timeout. Verify the preceding setInput \
                 changed the filter, or inspect getAccessibilityTree before selecting."
                    .to_string(),
            )
        }
        WaitCondition::Named(WaitNamedCondition::WindowFocused) if !snapshot.window_focused => {
            Some(
                "The window never became focused. Wait for windowVisible first, \
                 then retry windowFocused."
                    .to_string(),
            )
        }
        WaitCondition::Detailed(WaitDetailedCondition::ElementExists { semantic_id })
        | WaitCondition::Detailed(WaitDetailedCondition::ElementVisible { semantic_id })
            if !snapshot
                .visible_semantic_ids
                .iter()
                .any(|id| id == semantic_id) =>
        {
            Some(format!(
                "Element '{semantic_id}' was not visible at timeout. Inspect \
                 getAccessibilityTree or switch to stateMatch if the exact \
                 semanticId is unstable."
            ))
        }
        WaitCondition::Detailed(WaitDetailedCondition::ElementFocused { semantic_id })
            if snapshot.focused_semantic_id.as_deref() != Some(semantic_id.as_str()) =>
        {
            Some(format!(
                "Element '{semantic_id}' never received focus. Add a focus action \
                 before waiting for elementFocused."
            ))
        }
        _ => None,
    }
}

// ── Command name helper ────────────────────────────────────────────────────

fn command_name(command: &BatchCommand) -> &'static str {
    match command {
        BatchCommand::SetInput { .. } => "setInput",
        BatchCommand::WaitFor { .. } => "waitFor",
        BatchCommand::SelectByValue { .. } => "selectByValue",
        BatchCommand::FilterAndSelect { .. } => "filterAndSelect",
        BatchCommand::TypeAndSubmit { .. } => "typeAndSubmit",
    }
}

// ── Wait-for polling loop ──────────────────────────────────────────────────

struct WaitResult {
    success: bool,
    elapsed_ms: u64,
    error: Option<TransactionError>,
    trace: TransactionCommandTrace,
}

fn run_wait_for_command<P: TransactionStateProvider>(
    provider: &mut P,
    index: usize,
    condition: &WaitCondition,
    timeout: u64,
    poll_interval: u64,
) -> WaitResult {
    let started_at_ms = now_epoch_ms();
    let started = Instant::now();
    let before = provider.snapshot();
    let mut polls = Vec::new();

    tracing::info!(
        target: "script_kit::transaction",
        index = index,
        timeout_ms = timeout,
        poll_interval_ms = poll_interval,
        "transaction_wait_start"
    );

    loop {
        let elapsed_ms = started.elapsed().as_millis() as u64;
        let snapshot = provider.snapshot();
        let (ok, matched_ids) = matches_condition(&snapshot, condition);

        polls.push(WaitPollObservation {
            attempt: polls.len() + 1,
            elapsed_ms,
            condition_satisfied: ok,
            snapshot: snapshot.clone(),
            matched_semantic_ids: matched_ids,
        });

        if ok {
            tracing::info!(
                target: "script_kit::transaction",
                index = index,
                elapsed_ms = elapsed_ms,
                "transaction_wait_complete"
            );
            return WaitResult {
                success: true,
                elapsed_ms,
                error: None,
                trace: TransactionCommandTrace {
                    index,
                    command: "waitFor".to_string(),
                    started_at_ms,
                    elapsed_ms,
                    before,
                    after: snapshot,
                    polls,
                    error: None,
                },
            };
        }

        if elapsed_ms >= timeout {
            let error = TransactionError {
                code: TransactionErrorCode::WaitConditionTimeout,
                message: format!("Timeout after {timeout}ms waiting for {condition:?}"),
                suggestion: build_wait_suggestion(condition, &snapshot),
            };

            tracing::warn!(
                target: "script_kit::transaction",
                index = index,
                elapsed_ms = elapsed_ms,
                message = %error.message,
                "transaction_wait_timeout"
            );

            return WaitResult {
                success: false,
                elapsed_ms,
                error: Some(error.clone()),
                trace: TransactionCommandTrace {
                    index,
                    command: "waitFor".to_string(),
                    started_at_ms,
                    elapsed_ms,
                    before,
                    after: snapshot,
                    polls,
                    error: Some(error),
                },
            };
        }

        std::thread::sleep(Duration::from_millis(poll_interval.max(1)));
    }
}

// ── Trace persistence helper ───────────────────────────────────────────────

fn maybe_persist_trace(
    mode: TransactionTraceMode,
    success: bool,
    trace: &TransactionTrace,
    log_path: Option<&Path>,
) -> Result<bool> {
    if !should_include_trace(mode, success) {
        return Ok(false);
    }
    append_transaction_trace(log_path, trace)?;
    Ok(true)
}

// ── Public executor entry points ───────────────────────────────────────────

/// Result of executing a single `waitFor` command.
pub struct WaitForOutput {
    pub request_id: String,
    pub success: bool,
    pub elapsed: u64,
    pub error: Option<TransactionError>,
    pub trace: Option<TransactionTrace>,
}

/// Execute a standalone `waitFor` command.
pub fn execute_wait_for<P: TransactionStateProvider>(
    provider: &mut P,
    request_id: String,
    condition: &WaitCondition,
    timeout: Option<u64>,
    poll_interval: Option<u64>,
    trace_mode: TransactionTraceMode,
) -> Result<WaitForOutput> {
    let timeout = timeout.unwrap_or(DEFAULT_WAIT_TIMEOUT_MS);
    let poll_interval = poll_interval.unwrap_or(DEFAULT_WAIT_POLL_INTERVAL_MS);

    let result = run_wait_for_command(provider, 0, condition, timeout, poll_interval);

    let trace = TransactionTrace {
        request_id: request_id.clone(),
        status: if result.success {
            TransactionTraceStatus::Ok
        } else {
            TransactionTraceStatus::Timeout
        },
        started_at_ms: result.trace.started_at_ms,
        total_elapsed_ms: result.elapsed_ms,
        failed_at: if result.success { None } else { Some(0) },
        commands: vec![result.trace],
    };

    let include_trace = maybe_persist_trace(trace_mode, result.success, &trace, None)?;

    Ok(WaitForOutput {
        request_id,
        success: result.success,
        elapsed: result.elapsed_ms,
        error: result.error,
        trace: if include_trace { Some(trace) } else { None },
    })
}

/// Result of executing a `batch` command.
pub struct BatchOutput {
    pub request_id: String,
    pub success: bool,
    pub results: Vec<BatchResultEntry>,
    pub failed_at: Option<usize>,
    pub total_elapsed: u64,
    pub trace: Option<TransactionTrace>,
}

/// Execute a batch of commands as a transaction.
pub fn execute_batch<P: TransactionStateProvider>(
    provider: &mut P,
    request_id: String,
    commands: &[BatchCommand],
    options: Option<&BatchOptions>,
    trace_mode: TransactionTraceMode,
) -> Result<BatchOutput> {
    let stop_on_error = options.is_none_or(|o| o.stop_on_error);
    let started_at_ms = now_epoch_ms();
    let started = Instant::now();
    let mut results = Vec::new();
    let mut command_traces = Vec::new();
    let mut failed_at: Option<usize> = None;

    tracing::info!(
        target: "script_kit::transaction",
        request_id = %request_id,
        command_count = commands.len(),
        stop_on_error = stop_on_error,
        "transaction_batch_start"
    );

    for (index, command) in commands.iter().enumerate() {
        match command {
            BatchCommand::SetInput { text } => {
                let cmd_started_at = now_epoch_ms();
                let cmd_started = Instant::now();
                let before = provider.snapshot();
                let mut error = None;

                let success = match provider.set_input(text) {
                    Ok(()) => true,
                    Err(e) => {
                        error = Some(TransactionError {
                            code: TransactionErrorCode::ActionFailed,
                            message: format!("setInput failed: {e}"),
                            suggestion: Some(
                                "Verify the active prompt exposes a writable input \
                                 field before issuing setInput."
                                    .to_string(),
                            ),
                        });
                        false
                    }
                };

                let elapsed_ms = cmd_started.elapsed().as_millis() as u64;
                let after = provider.snapshot();

                results.push(BatchResultEntry {
                    index,
                    success,
                    command: command_name(command).to_string(),
                    elapsed: Some(elapsed_ms),
                    value: None,
                    error: error.clone(),
                });
                command_traces.push(TransactionCommandTrace {
                    index,
                    command: command_name(command).to_string(),
                    started_at_ms: cmd_started_at,
                    elapsed_ms,
                    before,
                    after,
                    polls: Vec::new(),
                    error,
                });

                if !success {
                    failed_at = Some(index);
                    if stop_on_error {
                        break;
                    }
                }
            }

            BatchCommand::WaitFor {
                condition,
                timeout,
                poll_interval,
            } => {
                let t = timeout.unwrap_or(DEFAULT_WAIT_TIMEOUT_MS);
                let pi = poll_interval.unwrap_or(DEFAULT_WAIT_POLL_INTERVAL_MS);

                let wr = run_wait_for_command(provider, index, condition, t, pi);

                results.push(BatchResultEntry {
                    index,
                    success: wr.success,
                    command: command_name(command).to_string(),
                    elapsed: Some(wr.elapsed_ms),
                    value: None,
                    error: wr.error.clone(),
                });
                command_traces.push(wr.trace);

                if !wr.success {
                    failed_at = Some(index);
                    if stop_on_error {
                        break;
                    }
                }
            }

            BatchCommand::SelectByValue { value, submit } => {
                let cmd_started_at = now_epoch_ms();
                let cmd_started = Instant::now();
                let before = provider.snapshot();
                let mut error = None;

                let selected = match provider.select_by_value(value, *submit) {
                    Ok(Some(v)) => Some(v),
                    Ok(None) => {
                        error = Some(TransactionError {
                            code: TransactionErrorCode::SelectionNotFound,
                            message: format!("selectByValue could not find value '{value}'"),
                            suggestion: Some(
                                "Run waitFor choicesRendered before selecting, or \
                                 inspect getAccessibilityTree to confirm the value \
                                 is present."
                                    .to_string(),
                            ),
                        });
                        None
                    }
                    Err(e) => {
                        error = Some(TransactionError {
                            code: TransactionErrorCode::ActionFailed,
                            message: format!("selectByValue failed: {e}"),
                            suggestion: Some(
                                "Verify the current choice is selectable and the \
                                 window is focused before selecting."
                                    .to_string(),
                            ),
                        });
                        None
                    }
                };

                let elapsed_ms = cmd_started.elapsed().as_millis() as u64;
                let after = provider.snapshot();
                let success = error.is_none();

                results.push(BatchResultEntry {
                    index,
                    success,
                    command: command_name(command).to_string(),
                    elapsed: Some(elapsed_ms),
                    value: selected,
                    error: error.clone(),
                });
                command_traces.push(TransactionCommandTrace {
                    index,
                    command: command_name(command).to_string(),
                    started_at_ms: cmd_started_at,
                    elapsed_ms,
                    before,
                    after,
                    polls: Vec::new(),
                    error,
                });

                if !success {
                    failed_at = Some(index);
                    if stop_on_error {
                        break;
                    }
                }
            }

            BatchCommand::FilterAndSelect { .. } | BatchCommand::TypeAndSubmit { .. } => {
                // These compound commands are not yet wired to the executor.
                // Record as unsupported so the caller gets a clear signal.
                let error = TransactionError {
                    code: TransactionErrorCode::UnsupportedCommand,
                    message: format!(
                        "{} is not yet supported by the transaction executor",
                        command_name(command)
                    ),
                    suggestion: Some(
                        "Use the equivalent primitive commands (setInput + waitFor \
                         + selectByValue) instead."
                            .to_string(),
                    ),
                };
                results.push(BatchResultEntry {
                    index,
                    success: false,
                    command: command_name(command).to_string(),
                    elapsed: Some(0),
                    value: None,
                    error: Some(error),
                });
                failed_at = Some(index);
                if stop_on_error {
                    break;
                }
            }
        }
    }

    let success = failed_at.is_none();
    let total_elapsed_ms = started.elapsed().as_millis() as u64;

    tracing::info!(
        target: "script_kit::transaction",
        request_id = %request_id,
        success = success,
        total_elapsed_ms = total_elapsed_ms,
        failed_at = ?failed_at,
        "transaction_batch_complete"
    );

    let trace = TransactionTrace {
        request_id: request_id.clone(),
        status: if success {
            TransactionTraceStatus::Ok
        } else {
            TransactionTraceStatus::Failed
        },
        started_at_ms,
        total_elapsed_ms,
        failed_at,
        commands: command_traces,
    };

    let include_trace = maybe_persist_trace(trace_mode, success, &trace, None)?;

    Ok(BatchOutput {
        request_id,
        success,
        results,
        failed_at,
        total_elapsed: total_elapsed_ms,
        trace: if include_trace { Some(trace) } else { None },
    })
}
