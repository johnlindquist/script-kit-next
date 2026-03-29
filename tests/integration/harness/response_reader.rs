//! Stdout response reader: background thread that captures structured JSONL
//! responses from the app's stdout.
//!
//! The app writes `QueryResponse` objects as single-line JSON to stdout.
//! This reader buffers them and supports blocking waits for specific responses
//! matched by `requestId`.

#![allow(dead_code)]

use std::io::{BufRead, BufReader};
use std::process::ChildStdout;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// A parsed structured response from the app's stdout.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QueryResponse {
    /// Snapshot of current prompt state.
    StateResult {
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "promptType")]
        prompt_type: String,
        #[serde(rename = "inputValue")]
        input_value: String,
        #[serde(rename = "choiceCount")]
        choice_count: usize,
        #[serde(rename = "visibleChoiceCount")]
        visible_choice_count: usize,
        #[serde(rename = "selectedIndex")]
        selected_index: i32,
        #[serde(rename = "selectedValue")]
        selected_value: Option<String>,
        #[serde(rename = "isFocused")]
        is_focused: bool,
        #[serde(rename = "windowVisible")]
        window_visible: bool,
    },
    /// Snapshot of the AI window's command-bar state.
    AiCommandBarResult {
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "aiWindowOpen")]
        ai_window_open: bool,
        #[serde(rename = "commandBarOpen")]
        command_bar_open: bool,
        #[serde(rename = "actionIds")]
        action_ids: Vec<String>,
        #[serde(rename = "selectedIndex")]
        selected_index: i32,
        #[serde(rename = "selectedActionId")]
        selected_action_id: Option<String>,
    },
    /// Snapshot of chat actions dialog state.
    ChatActionsResult {
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "isChatView")]
        is_chat_view: bool,
        #[serde(rename = "actionsPopupOpen")]
        actions_popup_open: bool,
        #[serde(rename = "actionIds")]
        action_ids: Vec<String>,
        #[serde(rename = "actionTitles")]
        action_titles: Vec<String>,
        #[serde(rename = "selectedActionIndex")]
        selected_action_index: i32,
        #[serde(rename = "selectedActionId")]
        selected_action_id: Option<String>,
        #[serde(rename = "chatModel")]
        chat_model: Option<String>,
        #[serde(rename = "messageCount")]
        message_count: usize,
        #[serde(rename = "hasResponse")]
        has_response: bool,
    },
}

impl QueryResponse {
    /// Get the requestId from any response variant.
    pub fn request_id(&self) -> &str {
        match self {
            Self::StateResult { request_id, .. }
            | Self::AiCommandBarResult { request_id, .. }
            | Self::ChatActionsResult { request_id, .. } => request_id,
        }
    }
}

/// Convenience struct for accessing `StateResult` fields.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub request_id: String,
    pub prompt_type: String,
    pub input_value: String,
    pub choice_count: usize,
    pub visible_choice_count: usize,
    pub selected_index: i32,
    pub selected_value: Option<String>,
    pub is_focused: bool,
    pub window_visible: bool,
}

impl StateSnapshot {
    /// Extract from a `QueryResponse::StateResult`.
    pub fn from_response(resp: QueryResponse) -> Option<Self> {
        match resp {
            QueryResponse::StateResult {
                request_id,
                prompt_type,
                input_value,
                choice_count,
                visible_choice_count,
                selected_index,
                selected_value,
                is_focused,
                window_visible,
            } => Some(Self {
                request_id,
                prompt_type,
                input_value,
                choice_count,
                visible_choice_count,
                selected_index,
                selected_value,
                is_focused,
                window_visible,
            }),
            _ => None,
        }
    }
}

/// Convenience struct for accessing `ChatActionsResult` fields.
#[derive(Debug, Clone)]
pub struct ChatActionsSnapshot {
    pub request_id: String,
    pub is_chat_view: bool,
    pub actions_popup_open: bool,
    pub action_ids: Vec<String>,
    pub action_titles: Vec<String>,
    pub selected_action_index: i32,
    pub selected_action_id: Option<String>,
    pub chat_model: Option<String>,
    pub message_count: usize,
    pub has_response: bool,
}

impl ChatActionsSnapshot {
    /// Extract from a `QueryResponse::ChatActionsResult`.
    pub fn from_response(resp: QueryResponse) -> Option<Self> {
        match resp {
            QueryResponse::ChatActionsResult {
                request_id,
                is_chat_view,
                actions_popup_open,
                action_ids,
                action_titles,
                selected_action_index,
                selected_action_id,
                chat_model,
                message_count,
                has_response,
            } => Some(Self {
                request_id,
                is_chat_view,
                actions_popup_open,
                action_ids,
                action_titles,
                selected_action_index,
                selected_action_id,
                chat_model,
                message_count,
                has_response,
            }),
            _ => None,
        }
    }
}

/// Convenience struct for accessing `AiCommandBarResult` fields.
#[derive(Debug, Clone)]
pub struct AiCommandBarSnapshot {
    pub request_id: String,
    pub ai_window_open: bool,
    pub command_bar_open: bool,
    pub action_ids: Vec<String>,
    pub selected_index: i32,
    pub selected_action_id: Option<String>,
}

impl AiCommandBarSnapshot {
    /// Extract from a `QueryResponse::AiCommandBarResult`.
    pub fn from_response(resp: QueryResponse) -> Option<Self> {
        match resp {
            QueryResponse::AiCommandBarResult {
                request_id,
                ai_window_open,
                command_bar_open,
                action_ids,
                selected_index,
                selected_action_id,
            } => Some(Self {
                request_id,
                ai_window_open,
                command_bar_open,
                action_ids,
                selected_index,
                selected_action_id,
            }),
            _ => None,
        }
    }
}

/// Shared state between the reader thread and test code.
struct ReaderState {
    responses: Vec<QueryResponse>,
    /// Raw lines that failed to parse (kept for debugging).
    unparsed: Vec<String>,
    /// Set to true when stdout EOF is reached.
    done: bool,
}

/// Background reader that captures stdout JSONL responses into a searchable buffer.
pub struct ResponseReader {
    state: Arc<(Mutex<ReaderState>, Condvar)>,
    _handle: std::thread::JoinHandle<()>,
}

impl ResponseReader {
    /// Start reading stdout from a child process.
    ///
    /// Spawns a background thread that reads lines until EOF, parsing each
    /// as a `QueryResponse`.
    pub fn new(stdout: ChildStdout) -> Self {
        let state = Arc::new((
            Mutex::new(ReaderState {
                responses: Vec::new(),
                unparsed: Vec::new(),
                done: false,
            }),
            Condvar::new(),
        ));

        let state_clone = Arc::clone(&state);
        let handle = std::thread::Builder::new()
            .name("integration-test-response-reader".into())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(text) => {
                            let trimmed = text.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            let (lock, cvar) = &*state_clone;
                            let mut state = lock.lock().unwrap();

                            match serde_json::from_str::<QueryResponse>(trimmed) {
                                Ok(resp) => {
                                    eprintln!(
                                        "[harness/stdout] Response: type={}, requestId={}",
                                        match &resp {
                                            QueryResponse::StateResult { .. } => "stateResult",
                                            QueryResponse::AiCommandBarResult { .. } =>
                                                "aiCommandBarResult",
                                            QueryResponse::ChatActionsResult { .. } =>
                                                "chatActionsResult",
                                        },
                                        resp.request_id()
                                    );
                                    state.responses.push(resp);
                                }
                                Err(_) => {
                                    // Not a QueryResponse — might be other stdout output
                                    eprintln!("[harness/stdout] Unparsed: {}", trimmed);
                                    state.unparsed.push(text);
                                }
                            }

                            cvar.notify_all();
                        }
                        Err(_) => break,
                    }
                }
                let (lock, cvar) = &*state_clone;
                let mut state = lock.lock().unwrap();
                state.done = true;
                cvar.notify_all();
            })
            .expect("failed to spawn response reader thread");

        Self {
            state,
            _handle: handle,
        }
    }

    /// Wait for a response with the given `requestId`.
    ///
    /// Searches existing buffered responses first, then blocks until a match
    /// arrives or `timeout` elapses.
    pub fn wait_for_response(
        &self,
        request_id: &str,
        timeout: Duration,
    ) -> anyhow::Result<QueryResponse> {
        let deadline = std::time::Instant::now() + timeout;
        let (lock, cvar) = &*self.state;

        let mut checked_up_to = 0;

        loop {
            let state = lock.lock().unwrap();

            // Check new responses since last scan
            for resp in &state.responses[checked_up_to..] {
                if resp.request_id() == request_id {
                    return Ok(resp.clone());
                }
            }
            checked_up_to = state.responses.len();

            if state.done {
                anyhow::bail!(
                    "stdout closed without response for requestId: {:?}\n\
                     Total responses captured: {}\n\
                     Unparsed lines: {}",
                    request_id,
                    state.responses.len(),
                    state.unparsed.len(),
                );
            }

            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                anyhow::bail!(
                    "timed out after {:?} waiting for response with requestId: {:?}\n\
                     Total responses captured: {}\n\
                     Available requestIds: {:?}",
                    timeout,
                    request_id,
                    state.responses.len(),
                    state
                        .responses
                        .iter()
                        .map(|r| r.request_id().to_string())
                        .collect::<Vec<_>>(),
                );
            }

            // Wait for new responses or EOF
            let _state = cvar.wait_timeout(state, remaining).unwrap().0;
        }
    }

    /// Get all captured responses so far (snapshot).
    pub fn responses(&self) -> Vec<QueryResponse> {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        state.responses.clone()
    }

    /// Check if stdout has been closed.
    pub fn is_done(&self) -> bool {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        state.done
    }
}
