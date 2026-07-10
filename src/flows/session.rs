//! Conversational flow sessions: metadata + command building.
//!
//! A session is an agent conversation — the real `flow-*` wrapper (or
//! `md <path> --_interactive`) running in an embedded PTY in the main window.
//! The live `Entity<TermPrompt>` handles live on `ScriptListApp`
//! (`flow_sessions`); this module owns the engine-agnostic parts so they stay
//! unit-testable without GPUI.
//!
//! Contract (Conversation Desk):
//! - Enter on a flow = start (or resume) a conversation.
//! - Backgrounding NEVER kills the process; re-entering an Active row
//!   restores the SAME PTY.
//! - Stop is an explicit Cmd+K verb, never a side effect of navigation.

/// Coarse session state, following Orca's four-state attention model.
/// v1 derives Working/Done from PTY liveness; hook-fed Blocked/Waiting
/// arrive with the mdflow session protocol later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Working,
    /// The agent asked something / is waiting on the user (hook-fed; unused
    /// until engines phone home).
    NeedsYou,
    /// Process exited with this code (None = signal/unknown).
    Done(Option<i32>),
}

impl SessionState {
    pub fn label(self) -> &'static str {
        match self {
            SessionState::Working => "working",
            SessionState::NeedsYou => "needs you",
            SessionState::Done(Some(0)) => "done",
            SessionState::Done(_) => "exited",
        }
    }

    pub fn is_live(self) -> bool {
        !matches!(self, SessionState::Done(_))
    }
}

/// Metadata for one conversation, independent of the GPUI entity.
#[derive(Debug, Clone)]
pub struct FlowSessionMeta {
    pub id: u64,
    pub flow_id: String,
    pub flow_name: String,
    pub friendly_name: String,
    pub origin: String,
    pub engine: String,
    /// The exact command typed into the PTY shell (receipt + restart).
    pub command: String,
    /// The user's initial task text, when the session started from typed
    /// input (Tab router / desk filter).
    pub initial_task: Option<String>,
    pub state: SessionState,
    pub started_at: std::time::Instant,
}

impl FlowSessionMeta {
    pub fn elapsed_label(&self) -> String {
        let secs = self.started_at.elapsed().as_secs();
        if secs < 60 {
            format!("{secs}s")
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else {
            format!("{}h", secs / 3600)
        }
    }
}

/// Single-quote shell escaping (POSIX): wraps in `'…'` with embedded quotes
/// escaped. Used for cwd and the initial task text.
pub fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '@'))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Build the conversation command for a flow.
///
/// Prefers the bun-linked wrapper (preserves package semantics exactly as in
/// a shell); falls back to `md <path>`. `--_interactive` makes mdflow exec
/// the engine's real interactive CLI. The initial task rides as the
/// positional prompt. The session id is stamped into the environment so
/// future hook/attribution work can attribute state to this pane (Orca's
/// pane-identity pattern).
pub fn build_conversation_command(
    session_id: u64,
    cwd: &str,
    wrapper_command: Option<&str>,
    flow_path: &str,
    initial_task: Option<&str>,
) -> String {
    let launch = match wrapper_command {
        Some(wrapper) => shell_quote(wrapper),
        None => format!("md {}", shell_quote(flow_path)),
    };
    let task = initial_task
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| format!(" {}", shell_quote(t)))
        .unwrap_or_default();
    format!(
        "cd {} && SCRIPT_KIT_FLOW_SESSION_ID={session_id} {launch}{task} --_interactive",
        shell_quote(cwd)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_command_preferred_over_md_path() {
        let cmd = build_conversation_command(
            7,
            "/Users/john/dev/app",
            Some("flow-gmail"),
            "/pkg/flows/flow-gmail.codex.md",
            Some("what did vercel email me?"),
        );
        assert_eq!(
            cmd,
            "cd /Users/john/dev/app && SCRIPT_KIT_FLOW_SESSION_ID=7 flow-gmail 'what did vercel email me?' --_interactive"
        );
    }

    #[test]
    fn md_fallback_without_wrapper_and_without_task() {
        let cmd = build_conversation_command(3, "/tmp/p", None, "/tmp/p/flows/x.codex.md", None);
        assert_eq!(
            cmd,
            "cd /tmp/p && SCRIPT_KIT_FLOW_SESSION_ID=3 md /tmp/p/flows/x.codex.md --_interactive"
        );
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(shell_quote(""), "''");
        assert_eq!(shell_quote("plain-path/ok.md"), "plain-path/ok.md");
    }

    #[test]
    fn state_labels_are_honest() {
        assert_eq!(SessionState::Working.label(), "working");
        assert_eq!(SessionState::Done(Some(0)).label(), "done");
        assert_eq!(SessionState::Done(Some(1)).label(), "exited");
        assert!(SessionState::Working.is_live());
        assert!(!SessionState::Done(None).is_live());
    }
}
