use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use agent_client_protocol::{self as acp, Client as _};
use anyhow::{Context as _, Result};
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, PtySize, PtySystem};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

const AGY_ADAPTER_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PRINT_TIMEOUT: &str = "2m";

#[derive(Debug, Default)]
struct AgySessionState {
    cwd: PathBuf,
    conversation_id: Option<String>,
    seen_lines: HashSet<String>,
}

type ActiveKillers = Arc<Mutex<HashMap<String, Box<dyn ChildKiller + Send + Sync>>>>;

pub(crate) fn run_stdio() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build AGY ACP Tokio runtime")?;

    let local_set = tokio::task::LocalSet::new();
    local_set.block_on(&runtime, async move {
        let incoming = tokio::io::stdin().compat();
        let outgoing = tokio::io::stdout().compat_write();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let (connection, io_task) =
            acp::AgentSideConnection::new(AgyAcpAgent::new(tx), outgoing, incoming, |future| {
                tokio::task::spawn_local(future);
            });

        tokio::task::spawn_local(async move {
            while let Some((notification, ack)) = rx.recv().await {
                let _ = connection.session_notification(notification).await;
                let _ = ack.send(());
            }
        });

        io_task.await
    })?;

    Ok(())
}

struct AgyAcpAgent {
    session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
    next_session_id: Cell<u64>,
    sessions: RefCell<HashMap<String, AgySessionState>>,
    active_killers: ActiveKillers,
}

impl AgyAcpAgent {
    fn new(
        session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
    ) -> Self {
        Self {
            session_update_tx,
            next_session_id: Cell::new(1),
            sessions: RefCell::new(HashMap::new()),
            active_killers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn stop_worker(&self, session_id: &str) {
        let killer = self
            .active_killers
            .lock()
            .ok()
            .and_then(|mut killers| killers.remove(session_id));
        if let Some(mut killer) = killer {
            let _ = killer.kill();
        }
    }

    async fn send_agent_chunk(&self, session_id: &acp::SessionId, text: String) -> acp::Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }

        let (ack_tx, ack_rx) = oneshot::channel();
        self.session_update_tx
            .send((
                acp::SessionNotification::new(
                    session_id.clone(),
                    acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                        acp::ContentBlock::Text(acp::TextContent::new(text)),
                    )),
                ),
                ack_tx,
            ))
            .map_err(|_| acp::Error::internal_error())?;
        ack_rx.await.map_err(|_| acp::Error::internal_error())?;
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Agent for AgyAcpAgent {
    async fn initialize(
        &self,
        args: acp::InitializeRequest,
    ) -> acp::Result<acp::InitializeResponse> {
        Ok(acp::InitializeResponse::new(args.protocol_version)
            .agent_info(
                acp::Implementation::new("script-kit-agy-acp", AGY_ADAPTER_VERSION)
                    .title("Antigravity CLI"),
            )
            .agent_capabilities(
                acp::AgentCapabilities::new().prompt_capabilities(
                    acp::PromptCapabilities::new()
                        .image(false)
                        .audio(false)
                        .embedded_context(false),
                ),
            ))
    }

    async fn authenticate(
        &self,
        _args: acp::AuthenticateRequest,
    ) -> acp::Result<acp::AuthenticateResponse> {
        Ok(acp::AuthenticateResponse::default())
    }

    async fn new_session(
        &self,
        args: acp::NewSessionRequest,
    ) -> acp::Result<acp::NewSessionResponse> {
        let next = self.next_session_id.get();
        self.next_session_id.set(next.saturating_add(1));
        let session_id = format!("agy-{next}");

        self.sessions.borrow_mut().insert(
            session_id.clone(),
            AgySessionState {
                cwd: args.cwd,
                conversation_id: None,
                seen_lines: HashSet::new(),
            },
        );

        Ok(
            acp::NewSessionResponse::new(session_id).models(acp::SessionModelState::new(
                acp::ModelId::new("default"),
                vec![acp::ModelInfo::new("default", "Antigravity CLI Default")],
            )),
        )
    }

    async fn prompt(&self, args: acp::PromptRequest) -> acp::Result<acp::PromptResponse> {
        let session_id = args.session_id.0.to_string();
        self.stop_worker(&session_id);

        let prompt = text_from_prompt_blocks(&args.prompt);
        if prompt.trim().is_empty() {
            return Err(acp::Error::invalid_params());
        }

        let (cwd, conversation_id, seen_lines) = {
            let mut sessions = self.sessions.borrow_mut();
            let session = sessions.entry(session_id.clone()).or_default();
            (
                session.cwd.clone(),
                session.conversation_id.clone(),
                session.seen_lines.clone(),
            )
        };

        let active_killers = Arc::clone(&self.active_killers);
        let worker_session_id = session_id.clone();
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        std::thread::Builder::new()
            .name(format!("agy-acp-{session_id}"))
            .spawn(move || {
                run_agy_prompt(
                    worker_session_id,
                    cwd,
                    prompt,
                    conversation_id,
                    seen_lines,
                    active_killers,
                    event_tx,
                );
            })
            .map_err(|_| acp::Error::internal_error())?;

        let mut emitted_lines = HashSet::new();
        let mut next_conversation_id = None;
        let mut cancelled = false;

        while let Some(event) = event_rx.recv().await {
            match event {
                AgyPromptEvent::Chunk(text) => {
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            emitted_lines.insert(trimmed.to_string());
                        }
                    }
                    self.send_agent_chunk(&args.session_id, text).await?;
                }
                AgyPromptEvent::BoundConversation(id) => {
                    next_conversation_id = Some(id);
                }
                AgyPromptEvent::Done {
                    cancelled: was_cancelled,
                } => {
                    cancelled = was_cancelled;
                    break;
                }
                AgyPromptEvent::Error(message) => {
                    return Err(acp::Error::internal_error().data(message));
                }
            }
        }

        if let Ok(mut killers) = self.active_killers.lock() {
            killers.remove(&session_id);
        }

        if let Some(session) = self.sessions.borrow_mut().get_mut(&session_id) {
            if let Some(id) = next_conversation_id {
                session.conversation_id = Some(id);
            }
            session.seen_lines.extend(emitted_lines);
        }

        let stop_reason = if cancelled {
            acp::StopReason::Cancelled
        } else {
            acp::StopReason::EndTurn
        };
        Ok(acp::PromptResponse::new(stop_reason))
    }

    async fn cancel(&self, args: acp::CancelNotification) -> acp::Result<()> {
        self.stop_worker(&args.session_id.0);
        Ok(())
    }

    async fn set_session_model(
        &self,
        _args: acp::SetSessionModelRequest,
    ) -> acp::Result<acp::SetSessionModelResponse> {
        Ok(acp::SetSessionModelResponse::default())
    }
}

#[derive(Debug)]
enum AgyPromptEvent {
    Chunk(String),
    BoundConversation(String),
    Done { cancelled: bool },
    Error(String),
}

fn run_agy_prompt(
    session_id: String,
    cwd: PathBuf,
    prompt: String,
    conversation_id: Option<String>,
    seen_lines: HashSet<String>,
    active_killers: ActiveKillers,
    event_tx: mpsc::UnboundedSender<AgyPromptEvent>,
) {
    let result = run_agy_prompt_inner(
        &session_id,
        &cwd,
        &prompt,
        conversation_id.as_deref(),
        seen_lines,
        Arc::clone(&active_killers),
        event_tx.clone(),
    );

    if let Err(error) = result {
        let _ = event_tx.send(AgyPromptEvent::Error(format!("{error:#}")));
    }
    if let Ok(mut killers) = active_killers.lock() {
        killers.remove(&session_id);
    }
}

fn run_agy_prompt_inner(
    session_id: &str,
    cwd: &Path,
    prompt: &str,
    conversation_id: Option<&str>,
    seen_lines: HashSet<String>,
    active_killers: ActiveKillers,
    event_tx: mpsc::UnboundedSender<AgyPromptEvent>,
) -> Result<()> {
    let agy_path = find_agy_path();
    let started_at = SystemTime::now();

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("open AGY PTY")?;

    let mut command = CommandBuilder::new(agy_path);
    if std::env::var("AGY_SKIP_PERMISSIONS").ok().as_deref() == Some("1") {
        command.arg("--dangerously-skip-permissions");
    }
    if let Some(id) = conversation_id {
        command.arg("--conversation");
        command.arg(id);
    }
    command.arg("--print-timeout");
    command
        .arg(std::env::var("AGY_PRINT_TIMEOUT").unwrap_or_else(|_| DEFAULT_PRINT_TIMEOUT.into()));
    command.arg("-p");
    command.arg(prompt);
    command.cwd(cwd);
    command.env("PAGER", "cat");

    let mut child = pair.slave.spawn_command(command).context("spawn agy")?;
    if let Ok(mut killers) = active_killers.lock() {
        killers.insert(session_id.to_string(), child.clone_killer());
    }
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .context("clone AGY PTY reader")?;
    let mut filter = TranscriptFilter::new(seen_lines);
    let mut buffer = [0_u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let chunk = String::from_utf8_lossy(&buffer[..count]);
                let filtered = filter.push(&chunk, false);
                if !filtered.trim().is_empty() {
                    let _ = event_tx.send(AgyPromptEvent::Chunk(filtered));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }

    let tail = filter.push("", true);
    if !tail.trim().is_empty() {
        let _ = event_tx.send(AgyPromptEvent::Chunk(tail));
    }

    let status = child.wait().context("wait for agy")?;
    if conversation_id.is_none() {
        if let Some(id) = find_conversation_created_after(started_at) {
            let _ = event_tx.send(AgyPromptEvent::BoundConversation(id));
        }
    }

    let cancelled = !status.success();
    let _ = event_tx.send(AgyPromptEvent::Done { cancelled });
    Ok(())
}

fn text_from_prompt_blocks(blocks: &[acp::ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            acp::ContentBlock::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_agy_path() -> String {
    let home = dirs::home_dir().unwrap_or_default();
    [
        home.join(".local/bin/agy"),
        home.join(".npm-global/bin/agy"),
        PathBuf::from("/opt/homebrew/bin/agy"),
        PathBuf::from("/usr/local/bin/agy"),
    ]
    .into_iter()
    .find(|path| path.exists())
    .map(|path| path.to_string_lossy().to_string())
    .unwrap_or_else(|| "agy".to_string())
}

fn antigravity_conversation_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".gemini/antigravity-cli/conversations"))
}

fn find_conversation_created_after(started_at: SystemTime) -> Option<String> {
    let dir = antigravity_conversation_dir()?;
    let threshold = started_at
        .checked_sub(Duration::from_secs(1))
        .unwrap_or(started_at);
    let entries = std::fs::read_dir(dir).ok()?;
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("pb") {
                return None;
            }
            let modified = entry.metadata().ok()?.modified().ok()?;
            if modified < threshold {
                return None;
            }
            let id = path.file_stem()?.to_str()?.to_string();
            Some((modified, id))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, id)| id)
}

#[derive(Debug)]
struct TranscriptFilter {
    buffer: String,
    suppressing_tool_block: bool,
    ignored_lines: HashSet<String>,
}

impl TranscriptFilter {
    fn new(ignored_lines: HashSet<String>) -> Self {
        Self {
            buffer: String::new(),
            suppressing_tool_block: false,
            ignored_lines,
        }
    }

    fn push(&mut self, chunk: &str, flush: bool) -> String {
        self.buffer.push_str(&strip_known_cli_noise(chunk));
        let mut output = String::new();

        while let Some(newline_index) = self.buffer.find('\n') {
            let line = self.buffer[..=newline_index].to_string();
            self.buffer = self.buffer[newline_index + 1..].to_string();
            if let Some(filtered) = self.filter_line(&line) {
                output.push_str(&filtered);
            }
        }

        if flush && !self.buffer.is_empty() {
            let line = std::mem::take(&mut self.buffer);
            if let Some(filtered) = self.filter_line(&line) {
                output.push_str(&filtered);
            }
        }

        output
    }

    fn filter_line(&mut self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            self.suppressing_tool_block = false;
            return Some(line.to_string());
        }

        if let Some(stripped) = self.strip_ignored_prefix(line) {
            return if stripped.trim().is_empty() {
                None
            } else {
                Some(stripped)
            };
        }

        if is_task_output_header(trimmed) {
            self.suppressing_tool_block = true;
            return None;
        }

        if self.suppressing_tool_block && is_tool_output_line(trimmed) {
            return None;
        }

        if self.suppressing_tool_block && !is_tool_output_line(trimmed) {
            self.suppressing_tool_block = false;
        }

        if is_tool_output_line(trimmed) {
            None
        } else {
            Some(line.to_string())
        }
    }

    fn strip_ignored_prefix(&self, line: &str) -> Option<String> {
        let trimmed_start_len = line.len() - line.trim_start().len();
        let leading = &line[..trimmed_start_len];
        let body = &line[trimmed_start_len..];
        let body_trimmed = body.trim_end_matches(['\r', '\n']);
        let line_end = &body[body_trimmed.len()..];

        let mut ignored = self.ignored_lines.iter().collect::<Vec<_>>();
        ignored.sort_by_key(|entry| std::cmp::Reverse(entry.len()));
        for ignored_line in ignored {
            if body_trimmed == ignored_line {
                return Some(String::new());
            }
            if let Some(rest) = body_trimmed.strip_prefix(ignored_line) {
                let rest = rest.trim_start();
                if !rest.is_empty() {
                    return Some(format!("{leading}{rest}{line_end}"));
                }
            }
        }
        None
    }
}

fn strip_known_cli_noise(text: &str) -> String {
    let without_ansi = strip_ansi_escape_sequences(text);
    without_ansi
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("Warning: conversation \"") && trimmed.contains("\" not found"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_ansi_escape_sequences(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn is_task_output_header(line: &str) -> bool {
    line.starts_with("[Task ") && line.contains(" output]")
}

fn is_tool_output_line(line: &str) -> bool {
    line.starts_with("/Users/")
        || line.starts_with("file:///Users/")
        || line.contains("/Cache/")
        || line.contains("/Cache_Data/")
        || line.contains("/IndexedDB/")
        || line.contains("/Library/")
        || line.contains("/Application Support/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_text_joins_only_text_blocks() {
        let blocks = vec![
            acp::ContentBlock::Text(acp::TextContent::new("hello")),
            acp::ContentBlock::Text(acp::TextContent::new("world")),
        ];
        assert_eq!(text_from_prompt_blocks(&blocks), "hello\nworld");
    }

    #[test]
    fn transcript_filter_removes_replay_and_tool_paths() {
        let mut ignored = HashSet::new();
        ignored.insert("previous answer".to_string());
        let mut filter = TranscriptFilter::new(ignored);
        let output = filter.push(
            "previous answer\n[Task abc output]\n/Users/me/Library/Cache/file\nfresh\n",
            true,
        );
        assert_eq!(output, "fresh");
    }

    #[test]
    fn transcript_filter_removes_replayed_prefix_without_newline() {
        let mut ignored = HashSet::new();
        ignored.insert("saved".to_string());
        let mut filter = TranscriptFilter::new(ignored);
        assert_eq!(filter.push("savedkumquat", true), "kumquat");
    }

    #[test]
    fn strip_known_noise_removes_warning_and_ansi() {
        let output = strip_known_cli_noise(
            "\u{1b}[31mred\u{1b}[0m\nWarning: conversation \"x\" not found.\n",
        );
        assert_eq!(output, "red");
    }
}
