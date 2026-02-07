use crate::logging;
use crate::protocol::GridDepthOption;
use crate::setup;
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
                    .collect::<Vec<_>>()
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
    TriggerBuiltin { name: String },
    /// Simulate a key press (for testing)
    /// key: Key name like "enter", "escape", "up", "down", "k", etc.
    /// modifiers: Optional array of modifiers ["cmd", "shift", "alt", "ctrl"]
    SimulateKey {
        key: String,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    /// Open the Notes window (for testing)
    OpenNotes,
    /// Open the AI Chat window (for testing)
    OpenAi,
    /// Open the AI Chat window with mock data (for visual testing)
    /// This inserts sample conversations to test the UI layout
    OpenAiWithMockData,
    /// Show the AI command bar (Cmd+K menu) for testing the refactored ActionsDialog
    ShowAiCommandBar,
    /// Simulate a key press in the AI window (for testing command bar navigation)
    /// key: Key name like "enter", "escape", "up", "down", "k", etc.
    /// modifiers: Optional array of modifiers ["cmd", "shift", "alt", "ctrl"]
    SimulateAiKey {
        key: String,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    /// Capture a screenshot of a window by title pattern and save to file (for testing)
    /// title: Title pattern to match (e.g., "Script Kit AI" for the AI window)
    /// path: File path to save the PNG screenshot
    CaptureWindow { title: String, path: String },
    /// Set the AI window search filter (for testing chat search)
    /// text: Search query to filter chats
    SetAiSearch { text: String },
    /// Set the AI window input text and optionally submit (for testing streaming)
    /// text: Message text to set in the input field
    /// submit: If true, submit the message after setting (triggers streaming)
    SetAiInput {
        text: String,
        #[serde(default)]
        submit: bool,
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
    },
    /// Execute a fallback action (e.g., Search Google, Copy to Clipboard)
    /// This is triggered when a fallback item is selected from the UI
    ExecuteFallback {
        /// The fallback ID (e.g., "search-google", "copy-to-clipboard")
        #[serde(rename = "fallbackId")]
        fallback_id: String,
        /// The user's input text to use with the fallback action
        input: String,
    },
}
impl ExternalCommand {
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Run { request_id, .. }
            | Self::Show { request_id }
            | Self::Hide { request_id }
            | Self::SetFilter { request_id, .. } => {
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
            Self::OpenAiWithMockData => "openAiWithMockData",
            Self::ShowAiCommandBar => "showAiCommandBar",
            Self::SimulateAiKey { .. } => "simulateAiKey",
            Self::CaptureWindow { .. } => "captureWindow",
            Self::SetAiSearch { .. } => "setAiSearch",
            Self::SetAiInput { .. } => "setAiInput",
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
