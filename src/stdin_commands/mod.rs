//! External command handling via stdin.
//!
//! This module provides the ability to control the Script Kit app via stdin JSONL commands.
//! This is primarily used for testing and automation.
//!
//! # Protocol
//!
//! Commands are sent as JSON objects, one per line (JSONL format):
//!
//! ```json
//! {"type": "run", "path": "/path/to/script.ts"}
//! {"type": "show"}
//! {"type": "hide"}
//! {"type": "setFilter", "text": "search term"}
//! {"type": "triggerBuiltin", "name": "clipboardHistory"}
//! {"type": "simulateKey", "key": "enter", "modifiers": ["cmd"]}
//! ```
//!
//! # Example Usage
//!
//! ```bash
//! # Run a script via stdin
//! echo '{"type": "run", "path": "/path/to/script.ts"}' | ./script-kit-gpui
//!
//! # Show/hide the window
//! echo '{"type": "show"}' | ./script-kit-gpui
//! echo '{"type": "hide"}' | ./script-kit-gpui
//!
//! # Filter the script list (for testing)
//! echo '{"type": "setFilter", "text": "hello"}' | ./script-kit-gpui
//! ```

// --- merged from part_000.rs ---
use crate::logging;
use crate::protocol::GridDepthOption;
use crate::setup;
use itertools::Itertools;
use std::io::BufRead;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;
/// Default grid size for ShowGrid command
fn default_grid_size() -> u32 {
    8
}
/// Maximum bytes accepted for a single external stdin JSONL command.
const MAX_STDIN_COMMAND_BYTES: usize = 16 * 1024;
const CAPTURE_WINDOW_RELATIVE_ROOTS: [&str; 2] = [".test-screenshots", "test-screenshots"];
const CAPTURE_WINDOW_SCRIPTKIT_ROOT: &str = "screenshots";
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct ExternalCommandRequestId(String);
impl ExternalCommandRequestId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl From<String> for ExternalCommandRequestId {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl std::fmt::Display for ExternalCommandRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl AsRef<str> for ExternalCommandRequestId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::ops::Deref for ExternalCommandRequestId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyModifier {
    #[serde(alias = "meta", alias = "command")]
    Cmd,
    Shift,
    #[serde(alias = "option")]
    Alt,
    #[serde(alias = "control")]
    Ctrl,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureWindowPathPolicyError {
    EmptyPath,
    PathOutsideAllowedRoots {
        resolved_path: PathBuf,
        allowed_roots: Vec<PathBuf>,
    },
    SymlinkInPath {
        resolved_path: PathBuf,
        symlink_path: PathBuf,
    },
    InvalidExtension {
        resolved_path: PathBuf,
    },
    PathResolutionIo {
        operation: &'static str,
        source: String,
    },
}
impl std::fmt::Display for CaptureWindowPathPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "captureWindow path must not be empty"),
            Self::PathOutsideAllowedRoots {
                resolved_path,
                allowed_roots,
            } => {
                let roots = allowed_roots
                    .iter()
                    .map(|root| root.display().to_string())
                    .join(", ");
                write!(
                    f,
                    "resolved path '{}' is outside allowed roots [{}]",
                    resolved_path.display(),
                    roots
                )
            }
            Self::SymlinkInPath {
                resolved_path,
                symlink_path,
            } => write!(
                f,
                "resolved path '{}' contains symlink component '{}'",
                resolved_path.display(),
                symlink_path.display()
            ),
            Self::InvalidExtension { resolved_path } => write!(
                f,
                "resolved path '{}' must end with .png",
                resolved_path.display()
            ),
            Self::PathResolutionIo { operation, source } => {
                write!(f, "path resolution failed during {}: {}", operation, source)
            }
        }
    }
}
impl std::error::Error for CaptureWindowPathPolicyError {}
pub fn validate_capture_window_output_path(
    raw_path: &str,
) -> Result<PathBuf, CaptureWindowPathPolicyError> {
    let cwd =
        std::env::current_dir().map_err(|err| CaptureWindowPathPolicyError::PathResolutionIo {
            operation: "current_dir",
            source: err.to_string(),
        })?;
    let kit_root = setup::get_kit_path();
    validate_capture_window_output_path_with_roots(raw_path, &cwd, &kit_root)
}
fn validate_capture_window_output_path_with_roots(
    raw_path: &str,
    cwd: &Path,
    kit_root: &Path,
) -> Result<PathBuf, CaptureWindowPathPolicyError> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err(CaptureWindowPathPolicyError::EmptyPath);
    }

    let expanded = PathBuf::from(shellexpand::tilde(trimmed).as_ref());
    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        cwd.join(expanded)
    };
    let normalized = normalize_absolute_path(&absolute);

    let allowed_roots = capture_window_allowed_roots(cwd, kit_root);
    let is_allowed = allowed_roots
        .iter()
        .any(|allowed_root| normalized.starts_with(allowed_root));
    if !is_allowed {
        return Err(CaptureWindowPathPolicyError::PathOutsideAllowedRoots {
            resolved_path: normalized,
            allowed_roots,
        });
    }

    let has_png_extension = normalized
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("png"))
        .unwrap_or(false);
    if !has_png_extension {
        return Err(CaptureWindowPathPolicyError::InvalidExtension {
            resolved_path: normalized,
        });
    }

    match first_symlink_component(&normalized) {
        Ok(Some(symlink_path)) => Err(CaptureWindowPathPolicyError::SymlinkInPath {
            resolved_path: normalized,
            symlink_path,
        }),
        Ok(None) => Ok(normalized),
        Err(err) => Err(CaptureWindowPathPolicyError::PathResolutionIo {
            operation: "symlink_metadata",
            source: err.to_string(),
        }),
    }
}
fn capture_window_allowed_roots(cwd: &Path, kit_root: &Path) -> Vec<PathBuf> {
    let normalized_cwd = normalize_absolute_path(cwd);
    let normalized_kit_root = normalize_absolute_path(kit_root);
    let mut roots = CAPTURE_WINDOW_RELATIVE_ROOTS
        .iter()
        .map(|root| normalize_absolute_path(&normalized_cwd.join(root)))
        .collect::<Vec<_>>();
    roots.push(normalize_absolute_path(
        &normalized_kit_root.join(CAPTURE_WINDOW_SCRIPTKIT_ROOT),
    ));
    roots.sort();
    roots.dedup();
    roots
}
fn normalize_absolute_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                let popped = normalized.pop();
                if !popped && !normalized.has_root() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
}
fn first_symlink_component(path: &Path) -> std::io::Result<Option<PathBuf>> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());

        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Ok(Some(current.clone()));
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        }
    }
    Ok(None)
}
enum StdinLineRead {
    Eof,
    Line(String),
    TooLong { raw: String, raw_len: usize },
}
fn read_stdin_line_bounded<R: BufRead>(
    reader: &mut R,
    byte_buffer: &mut Vec<u8>,
    max_line_bytes: usize,
) -> std::io::Result<StdinLineRead> {
    byte_buffer.clear();
    let mut total_bytes = 0usize;
    let mut saw_any_data = false;

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if !saw_any_data {
                return Ok(StdinLineRead::Eof);
            }

            let line = String::from_utf8_lossy(byte_buffer).into_owned();
            return Ok(StdinLineRead::Line(line));
        }

        saw_any_data = true;
        let newline_pos = available.iter().position(|&byte| byte == b'\n');
        let consumed_len = newline_pos.map_or(available.len(), |idx| idx + 1);

        if byte_buffer.len() < max_line_bytes {
            let remaining = max_line_bytes - byte_buffer.len();
            let copy_len = remaining.min(consumed_len);
            byte_buffer.extend_from_slice(&available[..copy_len]);
        }

        reader.consume(consumed_len);
        total_bytes = total_bytes.saturating_add(consumed_len);

        if total_bytes > max_line_bytes {
            // Drain the rest of this oversized command to recover.
            if newline_pos.is_none() {
                loop {
                    let remaining = reader.fill_buf()?;
                    if remaining.is_empty() {
                        break;
                    }
                    if let Some(next_newline_pos) = remaining.iter().position(|&byte| byte == b'\n')
                    {
                        reader.consume(next_newline_pos + 1);
                        total_bytes = total_bytes.saturating_add(next_newline_pos + 1);
                        break;
                    }
                    let chunk_len = remaining.len();
                    reader.consume(chunk_len);
                    total_bytes = total_bytes.saturating_add(chunk_len);
                }
            }

            let raw = String::from_utf8_lossy(byte_buffer).into_owned();
            return Ok(StdinLineRead::TooLong {
                raw,
                raw_len: total_bytes,
            });
        }

        if newline_pos.is_some() {
            let line = String::from_utf8_lossy(byte_buffer).into_owned();
            return Ok(StdinLineRead::Line(line));
        }
    }
}
/// External commands that can be sent to the app via stdin
///
/// All commands support an optional `requestId` field for correlation.
/// When present, the request_id is logged with all related operations,
/// making it easy for AI agents to trace command execution through logs.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ExternalCommand {
    /// Run a script by path
    Run {
        path: String,
        /// Optional request ID for correlation in logs
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Show the window
    Show {
        /// Optional request ID for correlation in logs
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Hide the window
    Hide {
        /// Optional request ID for correlation in logs
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Set the filter text (for testing)
    SetFilter {
        text: String,
        /// Optional request ID for correlation in logs
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Trigger a built-in feature by name (for testing)
    TriggerBuiltin {
        name: String,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Simulate a key press (for testing)
    /// key: Key name like "enter", "escape", "up", "down", "k", etc.
    /// modifiers: Optional array of modifiers ["cmd", "shift", "alt", "ctrl"]
    SimulateKey {
        key: String,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Open the Notes window (for testing)
    OpenNotes,
    /// Open the AI Chat window (for testing)
    OpenAi,
    /// Open the Mini AI Chat window (for testing)
    OpenMiniAi,
    /// Open the AI Chat window with mock data (for visual testing)
    /// This inserts sample conversations to test the UI layout
    OpenAiWithMockData,
    /// Open the Mini AI Chat window with mock data (for visual testing)
    OpenMiniAiWithMockData,
    /// Show the AI command bar (Cmd+K menu) for testing the refactored ActionsDialog
    ShowAiCommandBar,
    /// Simulate a key press in the AI window (for testing command bar navigation)
    /// key: Key name like "enter", "escape", "up", "down", "k", etc.
    /// modifiers: Optional array of modifiers ["cmd", "shift", "alt", "ctrl"]
    SimulateAiKey {
        key: String,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Capture a screenshot of a window by title pattern and save to file (for testing)
    /// title: Title pattern to match (e.g., "Script Kit AI" for the AI window)
    /// path: File path to save the PNG screenshot
    CaptureWindow {
        title: String,
        path: String,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Set the AI window search filter (for testing chat search)
    /// text: Search query to filter chats
    SetAiSearch {
        text: String,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Set the AI window input text and optionally submit (for testing streaming)
    /// text: Message text to set in the input field
    /// submit: If true, submit the message after setting (triggers streaming)
    SetAiInput {
        text: String,
        #[serde(default)]
        submit: bool,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Set the ACP chat input text and optionally submit (for testing ACP composer behavior)
    /// text: Message text to set in the ACP input field
    /// submit: If true, submit the message after setting
    SetAcpInput {
        text: String,
        #[serde(default)]
        submit: bool,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Show the debug grid overlay with options (for visual testing)
    ShowGrid {
        #[serde(default = "default_grid_size", rename = "gridSize")]
        grid_size: u32,
        #[serde(default, rename = "showBounds")]
        show_bounds: bool,
        #[serde(default, rename = "showBoxModel")]
        show_box_model: bool,
        #[serde(default, rename = "showAlignmentGuides")]
        show_alignment_guides: bool,
        #[serde(default, rename = "showDimensions")]
        show_dimensions: bool,
        #[serde(default)]
        depth: GridDepthOption,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Hide the debug grid overlay
    HideGrid,
    /// Show the shortcut recorder modal (for testing)
    /// command_id: ID of the command (e.g., "test/my-command")
    /// command_name: Display name (e.g., "My Command")
    ShowShortcutRecorder {
        #[serde(rename = "commandId")]
        command_id: String,
        #[serde(rename = "commandName")]
        command_name: String,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Query the AI window state as a machine-readable JSON snapshot.
    ///
    /// Returns structural metadata (mode, overlay visibility, counts) via structured
    /// tracing at info level. Never exposes conversation content or PII.
    GetAiWindowState {
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
    /// Execute a fallback action (e.g., Search Google, Copy to Clipboard)
    /// This is triggered when a fallback item is selected from the UI
    ExecuteFallback {
        /// The fallback ID (e.g., "search-google", "copy-to-clipboard")
        #[serde(rename = "fallbackId")]
        fallback_id: String,
        /// The user's input text to use with the fallback action
        input: String,
        #[serde(default, rename = "requestId")]
        request_id: Option<ExternalCommandRequestId>,
    },
}
impl ExternalCommand {
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Run { request_id, .. }
            | Self::Show { request_id }
            | Self::Hide { request_id }
            | Self::SetFilter { request_id, .. }
            | Self::TriggerBuiltin { request_id, .. }
            | Self::SimulateKey { request_id, .. }
            | Self::SimulateAiKey { request_id, .. }
            | Self::CaptureWindow { request_id, .. }
            | Self::SetAiSearch { request_id, .. }
            | Self::SetAiInput { request_id, .. }
            | Self::SetAcpInput { request_id, .. }
            | Self::GetAiWindowState { request_id, .. }
            | Self::ShowGrid { request_id, .. }
            | Self::ShowShortcutRecorder { request_id, .. }
            | Self::ExecuteFallback { request_id, .. } => {
                request_id.as_ref().map(ExternalCommandRequestId::as_str)
            }
            _ => None,
        }
    }

    pub fn command_type(&self) -> &'static str {
        match self {
            Self::Run { .. } => "run",
            Self::Show { .. } => "show",
            Self::Hide { .. } => "hide",
            Self::SetFilter { .. } => "setFilter",
            Self::TriggerBuiltin { .. } => "triggerBuiltin",
            Self::SimulateKey { .. } => "simulateKey",
            Self::OpenNotes => "openNotes",
            Self::OpenAi => "openAi",
            Self::OpenMiniAi => "openMiniAi",
            Self::OpenAiWithMockData => "openAiWithMockData",
            Self::OpenMiniAiWithMockData => "openMiniAiWithMockData",
            Self::ShowAiCommandBar => "showAiCommandBar",
            Self::SimulateAiKey { .. } => "simulateAiKey",
            Self::CaptureWindow { .. } => "captureWindow",
            Self::SetAiSearch { .. } => "setAiSearch",
            Self::SetAiInput { .. } => "setAiInput",
            Self::SetAcpInput { .. } => "setAcpInput",
            Self::GetAiWindowState { .. } => "getAiWindowState",
            Self::ShowGrid { .. } => "showGrid",
            Self::HideGrid => "hideGrid",
            Self::ShowShortcutRecorder { .. } => "showShortcutRecorder",
            Self::ExecuteFallback { .. } => "executeFallback",
        }
    }
}
#[derive(Debug, Clone)]
pub struct ExternalCommandEnvelope {
    pub command: ExternalCommand,
    pub correlation_id: String,
}
// --- merged from part_001.rs ---
/// Start a thread that listens on stdin for external JSONL commands.
/// Returns an async_channel::Receiver that can be awaited without polling.
///
/// # Channel Capacity
///
/// Uses a bounded channel with capacity of 100 to prevent unbounded memory growth.
/// This is generous for stdin commands which typically arrive at < 10/sec.
///
/// # Thread Safety
///
/// Spawns a background thread that reads stdin line-by-line. When the channel
/// is closed (receiver dropped), the thread will exit gracefully.
#[tracing::instrument(skip_all)]
pub fn start_stdin_listener() -> async_channel::Receiver<ExternalCommandEnvelope> {
    // P1-6: Use bounded channel to prevent unbounded memory growth
    // Capacity of 100 is generous for stdin commands (typically < 10/sec)
    let (tx, rx) = async_channel::bounded(100);

    std::thread::spawn(move || {
        let listener_correlation_id = format!("stdin:listener:{}", Uuid::new_v4());
        let _listener_guard = logging::set_correlation_id(listener_correlation_id.clone());
        tracing::info!(
            category = "STDIN",
            event_type = "stdin_listener_started",
            correlation_id = %listener_correlation_id,
            "External command listener started"
        );

        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut byte_buffer = Vec::with_capacity(1024);

        loop {
            match read_stdin_line_bounded(&mut reader, &mut byte_buffer, MAX_STDIN_COMMAND_BYTES) {
                Ok(StdinLineRead::Eof) => break,
                Ok(StdinLineRead::Line(line)) => {
                    let trimmed = line.trim_end_matches(['\r', '\n']);
                    if trimmed.trim().is_empty() {
                        continue;
                    }

                    let summary = logging::summarize_payload(trimmed);
                    match serde_json::from_str::<ExternalCommand>(trimmed) {
                        Ok(cmd) => {
                            let correlation_id = cmd
                                .request_id()
                                .filter(|id| !id.trim().is_empty())
                                .map(|id| format!("stdin:req:{}", id))
                                .unwrap_or_else(|| format!("stdin:{}", Uuid::new_v4()));
                            let _guard = logging::set_correlation_id(correlation_id.clone());

                            tracing::info!(
                                category = "STDIN",
                                event_type = "stdin_command_parsed",
                                command_type = cmd.command_type(),
                                line_len = trimmed.len(),
                                payload_summary = %summary,
                                correlation_id = %correlation_id,
                                "Parsed external command"
                            );

                            // send_blocking is used since we're in a sync thread
                            if tx
                                .send_blocking(ExternalCommandEnvelope {
                                    command: cmd,
                                    correlation_id: correlation_id.clone(),
                                })
                                .is_err()
                            {
                                tracing::warn!(
                                    category = "STDIN",
                                    event_type = "stdin_channel_closed",
                                    correlation_id = %correlation_id,
                                    "Command channel closed, exiting"
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            let correlation_id = format!("stdin:parse:{}", Uuid::new_v4());
                            let _guard = logging::set_correlation_id(correlation_id.clone());
                            tracing::warn!(
                                category = "STDIN",
                                event_type = "stdin_parse_failed",
                                line_len = trimmed.len(),
                                payload_summary = %summary,
                                error = %e,
                                correlation_id = %correlation_id,
                                "Failed to parse external command"
                            );
                        }
                    }
                }
                Ok(StdinLineRead::TooLong { raw, raw_len }) => {
                    let correlation_id = format!("stdin:oversize:{}", Uuid::new_v4());
                    let _guard = logging::set_correlation_id(correlation_id.clone());
                    let summary = logging::summarize_payload(&raw);
                    tracing::warn!(
                        category = "STDIN",
                        event_type = "stdin_command_too_large",
                        raw_len = raw_len,
                        max_line_bytes = MAX_STDIN_COMMAND_BYTES,
                        payload_summary = %summary,
                        correlation_id = %correlation_id,
                        "Skipping oversized external command"
                    );
                }
                Err(e) => {
                    let correlation_id = format!("stdin:read:{}", Uuid::new_v4());
                    let _guard = logging::set_correlation_id(correlation_id.clone());
                    tracing::error!(
                        category = "STDIN",
                        event_type = "stdin_read_error",
                        error = %e,
                        correlation_id = %correlation_id,
                        "Error reading stdin"
                    );
                    break;
                }
            }
        }
        tracing::info!(
            category = "STDIN",
            event_type = "stdin_listener_exiting",
            "External command listener exiting"
        );
    });

    rx
}
// --- merged from part_002.rs ---
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use std::io::Cursor;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_read_stdin_line_bounded_skips_oversized_line_and_recovers() -> anyhow::Result<()> {
        let oversized_payload = "x".repeat(20_000);
        let input = format!(
            r#"{{"type":"setFilter","text":"{}"}}
{{"type":"show"}}
"#,
            oversized_payload
        );

        let mut reader = Cursor::new(input);
        let mut byte_buffer = Vec::new();

        let first = read_stdin_line_bounded(&mut reader, &mut byte_buffer, MAX_STDIN_COMMAND_BYTES)
            .context("Expected bounded line reader to process input")?;
        match first {
            StdinLineRead::TooLong { raw_len, .. } => {
                assert!(raw_len > MAX_STDIN_COMMAND_BYTES);
            }
            _ => panic!("Expected first line to be marked too long"),
        }

        let second =
            read_stdin_line_bounded(&mut reader, &mut byte_buffer, MAX_STDIN_COMMAND_BYTES)
                .context("Expected second line to be readable")?;
        match second {
            StdinLineRead::Line(line) => {
                assert_eq!(line.trim_end(), r#"{"type":"show"}"#);
            }
            _ => panic!("Expected second line to be a valid command"),
        }

        Ok(())
    }

    #[test]
    fn test_external_command_run_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "run", "path": "/path/to/script.ts"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::Run { path, request_id } => {
                assert_eq!(path, "/path/to/script.ts");
                assert!(request_id.is_none());
            }
            _ => panic!("Expected Run command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_run_with_request_id() -> anyhow::Result<()> {
        let json = r#"{"type": "run", "path": "/path/to/script.ts", "requestId": "req-123"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::Run { path, request_id } => {
                assert_eq!(path, "/path/to/script.ts");
                assert_eq!(request_id, Some("req-123".to_string().into()));
            }
            _ => panic!("Expected Run command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_show_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "show"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::Show { request_id: None }));
        Ok(())
    }

    #[test]
    fn test_external_command_show_with_request_id() -> anyhow::Result<()> {
        let json = r#"{"type": "show", "requestId": "req-456"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::Show { request_id } => {
                assert_eq!(request_id, Some("req-456".to_string().into()));
            }
            _ => panic!("Expected Show command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_hide_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "hide"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::Hide { request_id: None }));
        Ok(())
    }

    #[test]
    fn test_external_command_set_filter_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "setFilter", "text": "hello world"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::SetFilter { text, request_id } => {
                assert_eq!(text, "hello world");
                assert!(request_id.is_none());
            }
            _ => panic!("Expected SetFilter command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_set_filter_with_request_id() -> anyhow::Result<()> {
        let json = r#"{"type": "setFilter", "text": "hello", "requestId": "req-789"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::SetFilter { text, request_id } => {
                assert_eq!(text, "hello");
                assert_eq!(request_id, Some("req-789".to_string().into()));
            }
            _ => panic!("Expected SetFilter command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_trigger_builtin_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "triggerBuiltin", "name": "clipboardHistory"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::TriggerBuiltin { name, .. } => assert_eq!(name, "clipboardHistory"),
            _ => panic!("Expected TriggerBuiltin command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_simulate_key_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "simulateKey", "key": "enter", "modifiers": ["cmd", "shift"]}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::SimulateKey { key, modifiers, .. } => {
                assert_eq!(key, "enter");
                assert_eq!(modifiers, vec![KeyModifier::Cmd, KeyModifier::Shift]);
            }
            _ => panic!("Expected SimulateKey command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_simulate_key_no_modifiers() -> anyhow::Result<()> {
        let json = r#"{"type": "simulateKey", "key": "escape"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::SimulateKey { key, modifiers, .. } => {
                assert_eq!(key, "escape");
                assert!(modifiers.is_empty());
            }
            _ => panic!("Expected SimulateKey command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_simulate_key_modifier_aliases() -> anyhow::Result<()> {
        let json = r#"{"type":"simulateKey","key":"k","modifiers":["meta","option","control"]}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::SimulateKey { modifiers, .. } => {
                assert_eq!(
                    modifiers,
                    vec![KeyModifier::Cmd, KeyModifier::Alt, KeyModifier::Ctrl]
                );
            }
            _ => panic!("Expected SimulateKey command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_simulate_key_unknown_modifier_rejected() {
        let json = r#"{"type":"simulateKey","key":"enter","modifiers":["capslock"]}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_invalid_json_fails() {
        let json = r#"{"type": "unknown"}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_rejects_unknown_fields() {
        let json = r#"{"type":"show","unexpected":"field"}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_missing_required_field_fails() {
        // Run command requires path field
        let json = r#"{"type": "run"}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_clone() {
        let cmd = ExternalCommand::Run {
            path: "/test".to_string(),
            request_id: None,
        };
        let cloned = cmd.clone();
        match cloned {
            ExternalCommand::Run { path, .. } => assert_eq!(path, "/test"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_external_command_debug() {
        let cmd = ExternalCommand::Show { request_id: None };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Show"));
    }

    #[test]
    fn test_external_command_request_id_accessor() {
        let cmd = ExternalCommand::SetFilter {
            text: "hello".to_string(),
            request_id: Some("req-42".to_string().into()),
        };
        assert_eq!(cmd.request_id(), Some("req-42"));
    }

    #[test]
    fn test_external_command_type_accessor() {
        let cmd = ExternalCommand::Show { request_id: None };
        assert_eq!(cmd.command_type(), "show");
    }

    #[test]
    fn test_external_command_open_notes_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "openNotes"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::OpenNotes));
        Ok(())
    }

    #[test]
    fn test_external_command_open_ai_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "openAi"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::OpenAi));
        Ok(())
    }

    #[test]
    fn test_external_command_open_mini_ai_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "openMiniAi"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::OpenMiniAi));
        Ok(())
    }

    #[test]
    fn test_external_command_open_ai_with_mock_data_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "openAiWithMockData"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::OpenAiWithMockData));
        Ok(())
    }

    #[test]
    fn test_external_command_open_mini_ai_with_mock_data_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "openMiniAiWithMockData"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::OpenMiniAiWithMockData));
        Ok(())
    }

    #[test]
    fn test_external_command_get_ai_window_state_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "getAiWindowState"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(
            cmd,
            ExternalCommand::GetAiWindowState { request_id: None }
        ));
        assert_eq!(cmd.command_type(), "getAiWindowState");
        Ok(())
    }

    #[test]
    fn test_external_command_get_ai_window_state_with_request_id() -> anyhow::Result<()> {
        let json = r#"{"type": "getAiWindowState", "requestId": "req-42"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert_eq!(cmd.request_id(), Some("req-42"));
        Ok(())
    }

    #[test]
    fn test_external_command_set_acp_input_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "setAcpInput", "text": "hello world", "submit": true}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert_eq!(cmd.command_type(), "setAcpInput");
        match cmd {
            ExternalCommand::SetAcpInput {
                text,
                submit,
                request_id,
            } => {
                assert_eq!(text, "hello world");
                assert!(submit);
                assert!(request_id.is_none());
            }
            _ => panic!("Expected SetAcpInput command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_set_acp_input_with_request_id() -> anyhow::Result<()> {
        let json = r#"{"type": "setAcpInput", "text": "hello", "requestId": "req-acp"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert_eq!(cmd.request_id(), Some("req-acp"));
        Ok(())
    }

    #[test]
    fn test_external_command_capture_window_deserialization() -> anyhow::Result<()> {
        let json =
            r#"{"type": "captureWindow", "title": "Script Kit AI", "path": "/tmp/screenshot.png"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::CaptureWindow { title, path, .. } => {
                assert_eq!(title, "Script Kit AI");
                assert_eq!(path, "/tmp/screenshot.png");
            }
            _ => panic!("Expected CaptureWindow command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_show_grid_defaults() -> anyhow::Result<()> {
        let json = r#"{"type": "showGrid"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::ShowGrid {
                grid_size,
                show_bounds,
                show_box_model,
                show_alignment_guides,
                show_dimensions,
                depth,
                ..
            } => {
                assert_eq!(grid_size, 8); // default
                assert!(!show_bounds); // default false
                assert!(!show_box_model); // default false
                assert!(!show_alignment_guides); // default false
                assert!(!show_dimensions); // default false
                assert!(matches!(depth, GridDepthOption::Preset(_))); // default
            }
            _ => panic!("Expected ShowGrid command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_show_grid_with_options() -> anyhow::Result<()> {
        let json = r#"{"type": "showGrid", "gridSize": 16, "showBounds": true, "showBoxModel": true, "showAlignmentGuides": true, "showDimensions": true, "depth": "all"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::ShowGrid {
                grid_size,
                show_bounds,
                show_box_model,
                show_alignment_guides,
                show_dimensions,
                depth,
                ..
            } => {
                assert_eq!(grid_size, 16);
                assert!(show_bounds);
                assert!(show_box_model);
                assert!(show_alignment_guides);
                assert!(show_dimensions);
                match depth {
                    GridDepthOption::Preset(s) => assert_eq!(s, "all"),
                    _ => panic!("Expected Preset depth"),
                }
            }
            _ => panic!("Expected ShowGrid command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_show_grid_with_components() -> anyhow::Result<()> {
        let json = r#"{"type": "showGrid", "depth": ["header", "footer"]}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::ShowGrid { depth, .. } => match depth {
                GridDepthOption::Components(components) => {
                    assert_eq!(components, vec!["header", "footer"]);
                }
                _ => panic!("Expected Components depth"),
            },
            _ => panic!("Expected ShowGrid command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_hide_grid_deserialization() -> anyhow::Result<()> {
        let json = r#"{"type": "hideGrid"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        assert!(matches!(cmd, ExternalCommand::HideGrid));
        Ok(())
    }

    #[test]
    fn test_external_command_execute_fallback_deserialization() -> anyhow::Result<()> {
        let json =
            r#"{"type": "executeFallback", "fallbackId": "search-google", "input": "hello world"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::ExecuteFallback {
                fallback_id, input, ..
            } => {
                assert_eq!(fallback_id, "search-google");
                assert_eq!(input, "hello world");
            }
            _ => panic!("Expected ExecuteFallback command"),
        }
        Ok(())
    }

    #[test]
    fn test_external_command_execute_fallback_copy() -> anyhow::Result<()> {
        let json = r#"{"type": "executeFallback", "fallbackId": "copy-to-clipboard", "input": "test text"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json)?;
        match cmd {
            ExternalCommand::ExecuteFallback {
                fallback_id, input, ..
            } => {
                assert_eq!(fallback_id, "copy-to-clipboard");
                assert_eq!(input, "test text");
            }
            _ => panic!("Expected ExecuteFallback command"),
        }
        Ok(())
    }

    #[test]
    fn test_validate_capture_window_output_path_allows_dot_test_screenshots() -> anyhow::Result<()>
    {
        let temp = TempDir::new().context("create temp dir")?;
        let cwd = std::fs::canonicalize(temp.path()).context("canonicalize temp dir")?;
        let kit_root = cwd.join("kit-root");
        std::fs::create_dir_all(&kit_root).context("create kit root")?;

        let resolved = validate_capture_window_output_path_with_roots(
            ".test-screenshots/shot.png",
            &cwd,
            &kit_root,
        )
        .context("path should be accepted")?;

        assert_eq!(resolved, cwd.join(".test-screenshots/shot.png"));
        Ok(())
    }

    #[test]
    fn test_validate_capture_window_output_path_rejects_traversal() -> anyhow::Result<()> {
        let temp = TempDir::new().context("create temp dir")?;
        let cwd = temp.path();
        let kit_root = cwd.join("kit-root");
        std::fs::create_dir_all(&kit_root).context("create kit root")?;

        let error = validate_capture_window_output_path_with_roots(
            ".test-screenshots/../escape.png",
            cwd,
            &kit_root,
        )
        .err()
        .context("path traversal should be rejected")?;

        assert!(matches!(
            error,
            CaptureWindowPathPolicyError::PathOutsideAllowedRoots { .. }
        ));
        Ok(())
    }

    #[test]
    fn test_validate_capture_window_output_path_rejects_symlink_parent() -> anyhow::Result<()> {
        let temp = TempDir::new().context("create temp dir")?;
        let cwd = temp.path();
        let kit_root = cwd.join("kit-root");
        std::fs::create_dir_all(&kit_root).context("create kit root")?;

        let screenshots_root = cwd.join(".test-screenshots");
        std::fs::create_dir_all(&screenshots_root).context("create screenshots root")?;

        let outside = cwd.join("outside");
        std::fs::create_dir_all(&outside).context("create outside dir")?;

        let symlink_path = screenshots_root.join("linked");
        create_symlink(&outside, &symlink_path)?;

        let error = validate_capture_window_output_path_with_roots(
            ".test-screenshots/linked/shot.png",
            cwd,
            &kit_root,
        )
        .err()
        .context("symlink target should be rejected")?;

        assert!(matches!(
            error,
            CaptureWindowPathPolicyError::SymlinkInPath { .. }
        ));
        Ok(())
    }

    #[test]
    fn test_validate_capture_window_output_path_allows_scriptkit_screenshots_root(
    ) -> anyhow::Result<()> {
        let temp = TempDir::new().context("create temp dir")?;
        let cwd = std::fs::canonicalize(temp.path()).context("canonicalize temp dir")?;
        let kit_root = cwd.join("kit-root");
        let screenshots_root = kit_root.join("screenshots");
        std::fs::create_dir_all(&screenshots_root).context("create screenshots root")?;

        let target = screenshots_root.join("shot.png");
        let resolved = validate_capture_window_output_path_with_roots(
            target.to_string_lossy().as_ref(),
            &cwd,
            &kit_root,
        )
        .context("path should be accepted")?;

        assert_eq!(resolved, target);
        Ok(())
    }

    #[cfg(unix)]
    fn create_symlink(target: &Path, link: &Path) -> anyhow::Result<()> {
        std::os::unix::fs::symlink(target, link).context("create symlink")?;
        Ok(())
    }

    #[cfg(windows)]
    fn create_symlink(target: &Path, link: &Path) -> anyhow::Result<()> {
        std::os::windows::fs::symlink_dir(target, link).context("create symlink")?;
        Ok(())
    }
}
