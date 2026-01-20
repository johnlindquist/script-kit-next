//! Persistent Claude Code CLI Session Manager
//!
//! This module manages persistent Claude CLI processes for efficient multi-turn conversations.
//! Instead of spawning a new process for each message, we keep a single process alive per chat
//! and send messages via the `--input-format stream-json` protocol.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  ClaudeSessionManager (global singleton)                        │
//! │  ├── sessions: HashMap<session_id, ClaudeSession>              │
//! │  └── cleanup_interval: periodically removes stale sessions      │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ClaudeSession (per chat)                                       │
//! │  ├── child: Child process handle                                │
//! │  ├── stdin: BufWriter to send JSONL messages                    │
//! │  ├── response_rx: Channel to receive parsed responses           │
//! │  └── reader_thread: Background thread parsing stdout            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! let manager = ClaudeSessionManager::global();
//!
//! // Send a message (creates session if needed)
//! manager.send_message(
//!     "chat-uuid",
//!     "Hello!",
//!     "sonnet",
//!     Some("Be helpful"),
//!     |chunk| println!("Chunk: {}", chunk),
//! )?;
//!
//! // Close session when done
//! manager.close_session("chat-uuid");
//! ```

use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

/// Events from the stdout reader thread
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Streaming text chunk (partial response)
    TextChunk(String),
    /// Final result (response complete)
    Result(String),
    /// Error from CLI
    Error(String),
    /// Process exited
    Exited(i32),
}

/// A persistent Claude CLI session
pub struct ClaudeSession {
    /// Child process handle
    child: Child,
    /// Buffered writer to stdin
    stdin: BufWriter<ChildStdin>,
    /// Receiver for parsed events from stdout
    response_rx: Receiver<SessionEvent>,
    /// Last activity time (for cleanup)
    last_activity: Instant,
    /// Session ID
    session_id: String,
    /// Model ID
    model_id: String,
}

impl ClaudeSession {
    /// Send a user message and stream the response
    pub fn send_message(&mut self, content: &str, on_chunk: impl Fn(&str)) -> Result<String> {
        self.last_activity = Instant::now();

        // Build and send the JSONL message
        let msg = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": content
            }
        });
        let line = serde_json::to_string(&msg)?;

        tracing::debug!(
            session_id = %self.session_id,
            message_len = content.len(),
            "Sending message to persistent Claude session"
        );

        self.stdin.write_all(line.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;

        // Read events until we get a Result or Error
        #[allow(unused_assignments)]
        let mut final_result: Option<String> = None;
        let timeout = Duration::from_secs(120);
        let start = Instant::now();

        loop {
            // Check timeout
            if start.elapsed() > timeout {
                return Err(anyhow!("Claude session timed out after {:?}", timeout));
            }

            // Try to receive with timeout
            match self.response_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => match event {
                    SessionEvent::TextChunk(chunk) => {
                        on_chunk(&chunk);
                    }
                    SessionEvent::Result(result) => {
                        final_result = Some(result);
                        break;
                    }
                    SessionEvent::Error(err) => {
                        return Err(anyhow!("Claude session error: {}", err));
                    }
                    SessionEvent::Exited(code) => {
                        return Err(anyhow!("Claude session exited with code: {}", code));
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Continue waiting
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(anyhow!("Claude session reader disconnected"));
                }
            }
        }

        self.last_activity = Instant::now();
        Ok(final_result.unwrap_or_default())
    }

    /// Check if the session is still alive
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false, // Exited
            Ok(None) => true,     // Still running
            Err(_) => false,      // Error checking
        }
    }

    /// Kill the session
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ClaudeSession {
    fn drop(&mut self) {
        tracing::debug!(session_id = %self.session_id, "Dropping Claude session");
        self.kill();
    }
}

/// Configuration for spawning a Claude session
#[derive(Clone)]
pub struct SessionConfig {
    pub claude_path: String,
    pub model_id: String,
    pub system_prompt: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            claude_path: "claude".to_string(),
            model_id: "sonnet".to_string(),
            system_prompt: Some("You are a helpful AI assistant".to_string()),
        }
    }
}

/// Manager for persistent Claude CLI sessions
pub struct ClaudeSessionManager {
    sessions: Mutex<HashMap<String, ClaudeSession>>,
    /// Track session IDs that have been created (for --resume vs --session-id)
    created_sessions: Mutex<std::collections::HashSet<String>>,
    config: SessionConfig,
}

impl ClaudeSessionManager {
    /// Get the global session manager instance
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<ClaudeSessionManager> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let claude_path = std::env::var("SCRIPT_KIT_CLAUDE_CODE_PATH")
                .unwrap_or_else(|_| "claude".to_string());

            ClaudeSessionManager {
                sessions: Mutex::new(HashMap::new()),
                created_sessions: Mutex::new(std::collections::HashSet::new()),
                config: SessionConfig {
                    claude_path,
                    ..Default::default()
                },
            }
        })
    }

    /// Send a message to a session (creating it if needed)
    pub fn send_message(
        &self,
        session_id: &str,
        content: &str,
        model_id: &str,
        system_prompt: Option<&str>,
        on_chunk: impl Fn(&str),
    ) -> Result<String> {
        tracing::debug!(
            session_id = %session_id,
            content_len = content.len(),
            model_id = %model_id,
            "ClaudeSessionManager.send_message called"
        );

        // Try to get existing session
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;

        tracing::debug!(
            session_id = %session_id,
            active_sessions = sessions.len(),
            "Acquired session lock"
        );

        // Check if we have a valid session
        let needs_new_session = match sessions.get_mut(session_id) {
            Some(session) => {
                let alive = session.is_alive();
                tracing::debug!(
                    session_id = %session_id,
                    is_alive = alive,
                    "Found existing session"
                );
                !alive
            }
            None => {
                tracing::debug!(session_id = %session_id, "No existing session found");
                true
            }
        };

        if needs_new_session {
            // Remove dead session if present
            sessions.remove(session_id);

            // Create new session
            tracing::info!(
                session_id = %session_id,
                model_id = %model_id,
                "Creating new persistent Claude session"
            );

            let session = self.spawn_session(session_id, model_id, system_prompt)?;
            sessions.insert(session_id.to_string(), session);
            tracing::debug!(session_id = %session_id, "Session created and inserted");
        }

        // Get the session and send message
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found after creation"))?;

        tracing::debug!(session_id = %session_id, "Sending message to session");
        let result = session.send_message(content, on_chunk);
        tracing::debug!(
            session_id = %session_id,
            success = result.is_ok(),
            "Message send completed"
        );
        result
    }

    /// Spawn a new Claude CLI session
    fn spawn_session(
        &self,
        session_id: &str,
        model_id: &str,
        system_prompt: Option<&str>,
    ) -> Result<ClaudeSession> {
        // Check if this session was created before (to use --resume vs --session-id)
        let session_existed = self
            .created_sessions
            .lock()
            .map(|set| set.contains(session_id))
            .unwrap_or(false);

        let mut cmd = Command::new(&self.config.claude_path);

        // Assistant mode configuration
        cmd.arg("--setting-sources").arg("");
        cmd.arg("--settings")
            .arg(r#"{"disableAllHooks": true, "permissions": {"allow": ["WebSearch", "WebFetch", "Read"]}}"#);
        cmd.arg("--tools").arg("WebSearch, WebFetch, Read");
        cmd.arg("--no-chrome");
        cmd.arg("--disable-slash-commands");

        // Streaming mode - IMPORTANT: --verbose is required for stream-json output
        cmd.arg("--print")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--output-format")
            .arg("stream-json");

        // Session persistence:
        // - First time: use --session-id to CREATE the session
        // - If process died and we're recreating: use --resume to CONTINUE the session
        if session_existed {
            tracing::info!(
                session_id = %session_id,
                "Resuming existing Claude session (process died, recreating)"
            );
            cmd.arg("--resume").arg(session_id);
        } else {
            tracing::info!(
                session_id = %session_id,
                "Creating new Claude session"
            );
            cmd.arg("--session-id").arg(session_id);

            // Mark this session as created
            if let Ok(mut set) = self.created_sessions.lock() {
                set.insert(session_id.to_string());
            }
        }

        // Model
        if !model_id.is_empty() && model_id != "default" {
            cmd.arg("--model").arg(model_id);
        }

        // System prompt (only effective on new sessions, ignored on resume)
        let effective_prompt = system_prompt
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("You are a helpful AI assistant");
        cmd.arg("--system-prompt").arg(effective_prompt);

        // Set up pipes
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::debug!(
            session_id = %session_id,
            model_id = %model_id,
            "Spawning persistent Claude CLI process"
        );

        let mut child = cmd.spawn().context("Failed to spawn claude CLI")?;

        // Take stdin
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdin"))?;
        let stdin = BufWriter::new(stdin);

        // Take stdout and spawn reader thread
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout"))?;

        // Take stderr for logging
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stderr"))?;

        // Create channel for events
        let (tx, rx) = mpsc::channel::<SessionEvent>();

        // Spawn stdout reader thread
        let session_id_clone = session_id.to_string();
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) if line.trim().is_empty() => continue,
                    Ok(line) => {
                        if let Some(event) = parse_claude_event(&line) {
                            if tx_clone.send(event).is_err() {
                                break; // Receiver dropped
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            session_id = %session_id_clone,
                            error = %e,
                            "Error reading Claude stdout"
                        );
                        let _ = tx_clone.send(SessionEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
            tracing::debug!(session_id = %session_id_clone, "Claude stdout reader exited");
        });

        // Spawn stderr reader thread (just for logging)
        let session_id_clone2 = session_id.to_string();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    tracing::trace!(
                        session_id = %session_id_clone2,
                        stderr = %line,
                        "Claude stderr"
                    );
                }
            }
        });

        Ok(ClaudeSession {
            child,
            stdin,
            response_rx: rx,
            last_activity: Instant::now(),
            session_id: session_id.to_string(),
            model_id: model_id.to_string(),
        })
    }

    /// Close a specific session
    pub fn close_session(&self, session_id: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(mut session) = sessions.remove(session_id) {
                tracing::info!(session_id = %session_id, "Closing Claude session");
                session.kill();
            }
        }
    }

    /// Close all sessions
    pub fn close_all_sessions(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for (id, mut session) in sessions.drain() {
                tracing::info!(session_id = %id, "Closing Claude session (cleanup)");
                session.kill();
            }
        }
    }

    /// Get count of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.lock().map(|s| s.len()).unwrap_or(0)
    }

    /// Cleanup stale sessions (not used recently)
    pub fn cleanup_stale_sessions(&self, max_idle: Duration) {
        if let Ok(mut sessions) = self.sessions.lock() {
            let stale_ids: Vec<String> = sessions
                .iter()
                .filter(|(_, s)| s.last_activity.elapsed() > max_idle)
                .map(|(id, _)| id.clone())
                .collect();

            for id in stale_ids {
                if let Some(mut session) = sessions.remove(&id) {
                    tracing::info!(
                        session_id = %id,
                        idle_secs = session.last_activity.elapsed().as_secs(),
                        "Cleaning up stale Claude session"
                    );
                    session.kill();
                }
            }
        }
    }
}

/// Parse a JSONL line from Claude CLI into a SessionEvent
fn parse_claude_event(line: &str) -> Option<SessionEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;

    match v.get("type")?.as_str()? {
        "stream_event" => {
            // Streaming events from --include-partial-messages
            // Format: {"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"..."}}}
            let event = v.get("event")?;
            if event.get("type")?.as_str()? == "content_block_delta" {
                let delta = event.get("delta")?;
                if delta.get("type")?.as_str()? == "text_delta" {
                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                        return Some(SessionEvent::TextChunk(text.to_string()));
                    }
                }
            }
            None
        }
        "assistant" => {
            // Full assistant message (also sent after streaming completes)
            // We can ignore this since we get the chunks via stream_event
            // Format: {"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}
            None
        }
        "result" => {
            // Final result
            let result = v.get("result")?.as_str()?.to_string();
            Some(SessionEvent::Result(result))
        }
        "error" => {
            let error = v
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            Some(SessionEvent::Error(error))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_event_result() {
        let line = r#"{"type":"result","subtype":"success","result":"Hello there!"}"#;
        let event = parse_claude_event(line);
        assert!(matches!(event, Some(SessionEvent::Result(r)) if r == "Hello there!"));
    }

    #[test]
    fn test_parse_claude_event_stream_delta() {
        // Streaming events come as stream_event with content_block_delta
        let line = r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}}"#;
        let event = parse_claude_event(line);
        assert!(matches!(event, Some(SessionEvent::TextChunk(t)) if t == "Hello"));
    }

    #[test]
    fn test_parse_claude_event_assistant_ignored() {
        // Assistant messages are ignored (we get content via stream_event)
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hi"}]}}"#;
        let event = parse_claude_event(line);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_claude_event_error() {
        let line = r#"{"type":"error","error":"Something went wrong"}"#;
        let event = parse_claude_event(line);
        assert!(matches!(event, Some(SessionEvent::Error(e)) if e == "Something went wrong"));
    }

    #[test]
    fn test_parse_claude_event_unknown() {
        let line = r#"{"type":"unknown","data":"stuff"}"#;
        let event = parse_claude_event(line);
        assert!(event.is_none());
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.claude_path, "claude");
        assert_eq!(config.model_id, "sonnet");
        assert!(config.system_prompt.is_some());
    }
}
