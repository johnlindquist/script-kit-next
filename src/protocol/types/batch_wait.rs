//! Typed protocol contracts for wait/batch deterministic transaction layer.
//!
//! These types enable AI agents to execute verifiable UI transactions:
//! - `WaitCondition`: poll until a UI state predicate is satisfied
//! - `BatchCommand`: atomic UI actions (set input, select, submit)
//! - `BatchOptions` / `BatchResultEntry`: transaction control and result reporting

use serde::{Deserialize, Serialize};

/// Specification for matching against current UI state.
///
/// All fields are optional; omitted fields are treated as "don't care".
/// A match succeeds when every present field equals the corresponding live value.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StateMatchSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_visible: Option<bool>,
}

/// Simple named conditions that agents can wait on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WaitNamedCondition {
    ChoicesRendered,
    InputEmpty,
    WindowVisible,
    WindowFocused,
}

/// Detailed conditions requiring additional parameters.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum WaitDetailedCondition {
    ElementExists {
        #[serde(rename = "semanticId", alias = "semantic_id")]
        semantic_id: String,
    },
    ElementVisible {
        #[serde(rename = "semanticId", alias = "semantic_id")]
        semantic_id: String,
    },
    ElementFocused {
        #[serde(rename = "semanticId", alias = "semantic_id")]
        semantic_id: String,
    },
    StateMatch { state: StateMatchSpec },
}

/// Union of named and detailed wait conditions.
///
/// Uses `#[serde(untagged)]` because the two variants are structurally
/// distinct: `WaitNamedCondition` is a bare string, while
/// `WaitDetailedCondition` is always an object with a `type` field.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum WaitCondition {
    Named(WaitNamedCondition),
    Detailed(WaitDetailedCondition),
}

/// Atomic UI commands that can be executed individually or inside a batch.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum BatchCommand {
    SetInput {
        text: String,
    },
    WaitFor {
        condition: WaitCondition,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(
            rename = "pollInterval",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        poll_interval: Option<u64>,
    },
    SelectByValue {
        value: String,
        #[serde(default)]
        submit: bool,
    },
    FilterAndSelect {
        filter: String,
        #[serde(rename = "selectFirst", default)]
        select_first: bool,
        #[serde(default)]
        submit: bool,
    },
    TypeAndSubmit {
        text: String,
    },
}

/// Options controlling batch execution behavior.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchOptions {
    /// Stop executing commands after the first failure (default: true).
    #[serde(default = "default_stop_on_error")]
    pub stop_on_error: bool,
    /// Reserved for future use: rollback side effects on error.
    #[serde(default)]
    pub rollback_on_error: bool,
    /// Overall batch timeout in milliseconds (default: 5000).
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

/// Trace mode for transaction requests.
///
/// Controls whether trace receipts are included in results:
/// - `off`: no trace (default)
/// - `on`: always include trace
/// - `onFailure`: include trace only when the transaction fails
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransactionTraceMode {
    #[default]
    Off,
    On,
    OnFailure,
}

/// Machine-readable error code for transaction failures.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionErrorCode {
    WaitConditionTimeout,
    ElementNotFound,
    SelectionNotFound,
    InvalidCondition,
    UnsupportedCommand,
    ActionFailed,
}

/// Structured error with machine-readable code and actionable suggestion.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionError {
    pub code: TransactionErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl TransactionError {
    /// Create an action-failed error from a message string.
    pub fn action_failed(message: impl Into<String>) -> Self {
        Self {
            code: TransactionErrorCode::ActionFailed,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a wait-condition-timeout error from a message string.
    pub fn wait_timeout(message: impl Into<String>) -> Self {
        Self {
            code: TransactionErrorCode::WaitConditionTimeout,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a selection-not-found error from a message string.
    pub fn selection_not_found(message: impl Into<String>) -> Self {
        Self {
            code: TransactionErrorCode::SelectionNotFound,
            message: message.into(),
            suggestion: None,
        }
    }
}

/// Snapshot of UI state at a point in time during a transaction.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiStateSnapshot {
    pub window_visible: bool,
    pub window_focused: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_semantic_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visible_semantic_ids: Vec<String>,
    #[serde(default)]
    pub choice_count: usize,
}

/// A single poll observation during a waitFor command.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WaitPollObservation {
    pub attempt: usize,
    pub elapsed_ms: u64,
    pub condition_satisfied: bool,
    pub snapshot: UiStateSnapshot,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_semantic_ids: Vec<String>,
}

/// Status of a completed transaction trace.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionTraceStatus {
    Ok,
    Failed,
    Timeout,
}

/// Trace data for a single command within a transaction.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCommandTrace {
    pub index: usize,
    pub command: String,
    pub started_at_ms: u64,
    pub elapsed_ms: u64,
    pub before: UiStateSnapshot,
    pub after: UiStateSnapshot,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub polls: Vec<WaitPollObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TransactionError>,
}

/// Full transaction trace receipt, optionally embedded in results.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTrace {
    pub request_id: String,
    pub status: TransactionTraceStatus,
    pub started_at_ms: u64,
    pub total_elapsed_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<TransactionCommandTrace>,
}

/// Result entry for a single command within a batch.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchResultEntry {
    /// Zero-based index of this command in the batch.
    pub index: usize,
    /// Whether this command succeeded.
    pub success: bool,
    /// The command type name (e.g., "setInput", "waitFor").
    pub command: String,
    /// Wall-clock time this command took, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed: Option<u64>,
    /// The value produced by this command, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Structured error if the command failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<TransactionError>,
}

/// Helper for serde skip_serializing_if — returns true when trace mode is Off (default).
pub fn is_trace_off(mode: &TransactionTraceMode) -> bool {
    *mode == TransactionTraceMode::Off
}

fn default_stop_on_error() -> bool {
    true
}

fn default_timeout() -> u64 {
    5_000
}
