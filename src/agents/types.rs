//! Agent type definitions
//!
//! This module contains the core data types for mdflow agents:
//! - `Agent` - Represents an agent parsed from a .md file
//! - `AgentBackend` - Supported AI backends (Claude, Gemini, Codex, Copilot)
//! - `AgentFrontmatter` - Parsed YAML frontmatter from agent files
//! - `AgentMatch` - Scored match result for fuzzy search
//! - `MdflowInput` - Typed input specification from `_inputs`
//!
//! # Metadata Convention
//!
//! Script Kit metadata uses `_sk_*` prefixed keys in frontmatter to avoid
//! leaking to CLI flags. mdflow treats underscore-prefixed keys as template
//! variables that are consumed (not passed to the backend command).
//!
//! ```yaml
//! ---
//! _sk_name: "Review PR"
//! _sk_description: "Reviews staged changes via Claude"
//! _sk_icon: "git-pull-request"
//! _sk_alias: "review"
//! _sk_shortcut: "cmd shift r"
//! model: sonnet
//! ---
//! ```

// These types are public API for future integration - allow unused fields/variants for now
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

/// Represents an mdflow agent parsed from a .md file
///
/// Agents are markdown files in `~/.scriptkit/plugins/*/agents/` that can be executed
/// via the mdflow CLI. The filename determines the AI backend to use:
/// - `task.claude.md` → runs via `claude` CLI
/// - `task.gemini.md` → runs via `gemini` CLI
/// - `task.codex.md` → runs via `codex` CLI
/// - `task.copilot.md` → runs via `copilot` CLI
///
/// # Execution Model
///
/// Script Kit spawns `mdflow <file>` directly and lets mdflow handle:
/// - Frontmatter → CLI flag conversion
/// - Config merging (built-in → global → project → frontmatter)
/// - Template variable substitution
/// - `@file` imports and `!command` inlines
///
/// Script Kit only adds runtime overrides:
/// - `--_quiet --raw` for UI capture mode (suppress dashboard, clean output)
/// - `--_varname value` for user-provided template variables
/// - stdin piping for `{{ _stdin }}` support
#[derive(Clone, Debug)]
pub struct Agent {
    /// Display name (from `_sk_name` or derived from filename)
    pub name: String,
    /// File path to the .md file
    pub path: PathBuf,
    /// Backend inferred from filename (claude, gemini, codex, copilot)
    pub backend: AgentBackend,
    /// Whether this is interactive mode (has `.i.` in filename or `_interactive: true`)
    pub interactive: bool,
    /// Description from `_sk_description`
    pub description: Option<String>,
    /// Icon name from `_sk_icon` (falls back to backend default)
    pub icon: Option<String>,
    /// Keyboard shortcut from `_sk_shortcut`
    pub shortcut: Option<String>,
    /// Alias from `_sk_alias` for quick triggering
    pub alias: Option<String>,
    /// Parsed frontmatter (preserved raw for mdflow to interpret)
    pub frontmatter: AgentFrontmatter,
    /// The kit this agent belongs to (e.g., "main", "custom-kit")
    pub kit: Option<String>,
    /// Whether the file contains shell inlines (`!command`)
    pub has_shell_inlines: bool,
    /// Whether the file contains remote URL imports
    pub has_remote_imports: bool,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
            backend: AgentBackend::Generic,
            interactive: false,
            description: None,
            icon: None,
            shortcut: None,
            alias: None,
            frontmatter: AgentFrontmatter::default(),
            kit: None,
            has_shell_inlines: false,
            has_remote_imports: false,
        }
    }
}

/// Supported AI backends for mdflow agents
///
/// The backend is inferred from the filename pattern:
/// - `*.claude.md` → Claude
/// - `*.gemini.md` → Gemini
/// - `*.codex.md` → Codex
/// - `*.copilot.md` → Copilot
/// - `*.md` (no backend suffix) → Generic
///
/// The `Other(String)` variant supports custom commands via `--_command`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum AgentBackend {
    Claude,
    Gemini,
    Codex,
    Copilot,
    /// Generic backend - no specific command suffix in filename
    #[default]
    Generic,
    /// Custom command specified via `--_command` or `_command` frontmatter
    Other(String),
}

impl AgentBackend {
    /// Parse backend from filename pattern
    ///
    /// # Examples
    /// ```
    /// use script_kit_gpui::agents::AgentBackend;
    ///
    /// assert_eq!(AgentBackend::from_filename("review.claude.md"), AgentBackend::Claude);
    /// assert_eq!(AgentBackend::from_filename("task.gemini.md"), AgentBackend::Gemini);
    /// assert_eq!(AgentBackend::from_filename("analyze.i.codex.md"), AgentBackend::Codex);
    /// assert_eq!(AgentBackend::from_filename("generic.md"), AgentBackend::Generic);
    /// ```
    pub fn from_filename(filename: &str) -> Self {
        let name = filename.to_lowercase();
        if name.contains(".claude.") || name.ends_with(".claude.md") {
            Self::Claude
        } else if name.contains(".gemini.") || name.ends_with(".gemini.md") {
            Self::Gemini
        } else if name.contains(".codex.") || name.ends_with(".codex.md") {
            Self::Codex
        } else if name.contains(".copilot.") || name.ends_with(".copilot.md") {
            Self::Copilot
        } else {
            Self::Generic
        }
    }

    /// Get the CLI command for this backend
    ///
    /// Returns `None` for Generic backend which requires explicit command specification.
    pub fn command(&self) -> Option<&str> {
        match self {
            Self::Claude => Some("claude"),
            Self::Gemini => Some("gemini"),
            Self::Codex => Some("codex"),
            Self::Copilot => Some("copilot"),
            Self::Generic => None,
            Self::Other(cmd) => Some(cmd.as_str()),
        }
    }

    /// Get the default icon for this backend
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Claude => "bot",
            Self::Gemini => "sparkles",
            Self::Codex => "terminal",
            Self::Copilot => "github",
            Self::Generic => "bot",
            Self::Other(_) => "cpu",
        }
    }

    /// Get the display label for this backend
    pub fn label(&self) -> &str {
        match self {
            Self::Claude => "Claude",
            Self::Gemini => "Gemini",
            Self::Codex => "Codex",
            Self::Copilot => "Copilot",
            Self::Generic => "Agent",
            Self::Other(cmd) => cmd.as_str(),
        }
    }

    /// Check if the backend CLI is available in PATH
    pub fn is_available(&self) -> bool {
        match self.command() {
            Some(cmd) => which::which(cmd).is_ok(),
            None => true, // Generic is always "available" (mdflow handles it)
        }
    }
}

/// Parsed frontmatter from agent .md file
///
/// This struct preserves the raw frontmatter for mdflow to interpret,
/// while extracting Script Kit-specific metadata (`_sk_*` keys) and
/// mdflow system keys (`_inputs`, `_interactive`, etc.).
///
/// # mdflow System Keys
///
/// - `_inputs` - Typed interactive prompts (text/select/number/confirm/password)
/// - `_interactive` / `_i` - Interactive mode override
/// - `_cwd` - Override working directory for inline commands
/// - `_subcommand` - Prepend subcommands to backend invocation
/// - `_env` - Environment variables for the spawned command
/// - `_command` - Override the backend command
///
/// # Script Kit Metadata Keys
///
/// - `_sk_name` - Display name in Script Kit
/// - `_sk_description` - Description shown in UI
/// - `_sk_icon` - Icon name for the list item
/// - `_sk_alias` - Quick trigger alias
/// - `_sk_shortcut` - Keyboard shortcut
#[derive(Clone, Debug, Default)]
pub struct AgentFrontmatter {
    /// Raw frontmatter as YAML - passed to mdflow as-is
    pub raw: HashMap<String, serde_yaml::Value>,

    // === Script Kit metadata (extracted from _sk_* keys) ===
    /// Display name from `_sk_name`
    pub sk_name: Option<String>,
    /// Description from `_sk_description`
    pub sk_description: Option<String>,
    /// Icon from `_sk_icon`
    pub sk_icon: Option<String>,
    /// Alias from `_sk_alias`
    pub sk_alias: Option<String>,
    /// Shortcut from `_sk_shortcut`
    pub sk_shortcut: Option<String>,

    // === mdflow system keys (extracted for UI/logic) ===
    /// Typed input specifications from `_inputs`
    pub inputs: Option<MdflowInputs>,
    /// Interactive mode from `_interactive` or `_i`
    pub interactive: Option<bool>,
    /// Working directory override from `_cwd`
    pub cwd: Option<String>,
    /// Custom command from `_command`
    pub command: Option<String>,
    /// Environment variables from `_env`
    pub env: Option<HashMap<String, String>>,
}

/// mdflow `_inputs` specification
///
/// Supports both the typed object format and legacy array format:
///
/// ```yaml
/// # Typed format
/// _inputs:
///   feature_name:
///     type: text
///     message: "Feature name?"
///     default: "New Feature"
///   confirm_deploy:
///     type: confirm
///     message: "Deploy to production?"
///
/// # Legacy array format
/// _inputs:
///   - feature_name
///   - confirm_deploy
/// ```
#[derive(Clone, Debug)]
pub enum MdflowInputs {
    /// Typed inputs with full specification
    Typed(HashMap<String, MdflowInput>),
    /// Legacy array of variable names (prompts for text input)
    Legacy(Vec<String>),
}

/// A single typed input specification
#[derive(Clone, Debug)]
pub struct MdflowInput {
    /// Input type: text, select, number, confirm, password
    pub input_type: MdflowInputType,
    /// Prompt message shown to user
    pub message: Option<String>,
    /// Default value
    pub default: Option<String>,
    /// Choices for select type
    pub choices: Option<Vec<String>>,
    /// Whether this input is required
    pub required: Option<bool>,
}

impl Default for MdflowInput {
    fn default() -> Self {
        Self {
            input_type: MdflowInputType::Text,
            message: None,
            default: None,
            choices: None,
            required: None,
        }
    }
}

/// Input types supported by mdflow's `_inputs`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MdflowInputType {
    #[default]
    Text,
    Select,
    Number,
    Confirm,
    Password,
}

impl MdflowInputType {
    /// Parse input type from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "select" => Self::Select,
            "number" => Self::Number,
            "confirm" | "boolean" => Self::Confirm,
            "password" | "secret" => Self::Password,
            _ => Self::Text,
        }
    }
}

// Note: AgentMatch is defined in scripts::types to avoid circular dependencies
// It's re-exported from the agents module for convenience

/// Execution mode for running agents
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AgentExecutionMode {
    /// UI capture mode: `--_quiet --raw` for clean output in Script Kit panel
    #[default]
    UiCapture,
    /// Interactive mode: full PTY for terminal interaction
    Interactive,
    /// Dry run: `--_dry-run` to show what would be executed
    DryRun,
    /// Explain mode: `md explain` to show context/prompt preview
    Explain,
}

/// Result of checking agent/backend availability
#[derive(Clone, Debug)]
pub struct AgentAvailability {
    /// Whether mdflow CLI is available
    pub mdflow_available: bool,
    /// Whether the backend CLI is available
    pub backend_available: bool,
    /// Error message if not available
    pub error_message: Option<String>,
}

impl AgentAvailability {
    pub fn is_available(&self) -> bool {
        self.mdflow_available && self.backend_available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AgentBackend tests ===

    #[test]
    fn test_backend_from_filename_claude() {
        assert_eq!(
            AgentBackend::from_filename("review.claude.md"),
            AgentBackend::Claude
        );
        assert_eq!(
            AgentBackend::from_filename("task.claude.md"),
            AgentBackend::Claude
        );
        assert_eq!(
            AgentBackend::from_filename("REVIEW.CLAUDE.MD"),
            AgentBackend::Claude
        );
    }

    #[test]
    fn test_backend_from_filename_gemini() {
        assert_eq!(
            AgentBackend::from_filename("task.gemini.md"),
            AgentBackend::Gemini
        );
        assert_eq!(
            AgentBackend::from_filename("analyze.gemini.md"),
            AgentBackend::Gemini
        );
    }

    #[test]
    fn test_backend_from_filename_codex() {
        assert_eq!(
            AgentBackend::from_filename("code.codex.md"),
            AgentBackend::Codex
        );
        assert_eq!(
            AgentBackend::from_filename("analyze.i.codex.md"),
            AgentBackend::Codex
        );
    }

    #[test]
    fn test_backend_from_filename_copilot() {
        assert_eq!(
            AgentBackend::from_filename("help.copilot.md"),
            AgentBackend::Copilot
        );
    }

    #[test]
    fn test_backend_from_filename_generic() {
        assert_eq!(
            AgentBackend::from_filename("generic.md"),
            AgentBackend::Generic
        );
        assert_eq!(
            AgentBackend::from_filename("my-agent.md"),
            AgentBackend::Generic
        );
    }

    #[test]
    fn test_backend_from_filename_with_interactive_marker() {
        // .i. marker should not affect backend detection
        assert_eq!(
            AgentBackend::from_filename("task.i.claude.md"),
            AgentBackend::Claude
        );
        assert_eq!(
            AgentBackend::from_filename("code.i.gemini.md"),
            AgentBackend::Gemini
        );
    }

    #[test]
    fn test_backend_commands() {
        assert_eq!(AgentBackend::Claude.command(), Some("claude"));
        assert_eq!(AgentBackend::Gemini.command(), Some("gemini"));
        assert_eq!(AgentBackend::Codex.command(), Some("codex"));
        assert_eq!(AgentBackend::Copilot.command(), Some("copilot"));
        assert_eq!(AgentBackend::Generic.command(), None);
        assert_eq!(
            AgentBackend::Other("custom".to_string()).command(),
            Some("custom")
        );
    }

    #[test]
    fn test_backend_labels() {
        assert_eq!(AgentBackend::Claude.label(), "Claude");
        assert_eq!(AgentBackend::Gemini.label(), "Gemini");
        assert_eq!(AgentBackend::Codex.label(), "Codex");
        assert_eq!(AgentBackend::Copilot.label(), "Copilot");
        assert_eq!(AgentBackend::Generic.label(), "Agent");
        assert_eq!(AgentBackend::Other("ollama".to_string()).label(), "ollama");
    }

    #[test]
    fn test_backend_icons() {
        assert_eq!(AgentBackend::Claude.icon(), "bot");
        assert_eq!(AgentBackend::Gemini.icon(), "sparkles");
        assert_eq!(AgentBackend::Codex.icon(), "terminal");
        assert_eq!(AgentBackend::Copilot.icon(), "github");
        assert_eq!(AgentBackend::Generic.icon(), "bot");
        assert_eq!(AgentBackend::Other("custom".to_string()).icon(), "cpu");
    }

    // === Interactive marker tests ===

    #[test]
    fn test_interactive_marker_detection() {
        assert!(
            "task.i.claude.md".contains(".i."),
            "Should detect interactive marker"
        );
        assert!(
            !"task.claude.md".contains(".i."),
            "Should not detect interactive marker without .i."
        );
        assert!(
            "interactive.i.gemini.md".contains(".i."),
            "Should detect interactive marker in gemini file"
        );
    }

    // === MdflowInputType tests ===

    #[test]
    fn test_input_type_from_str() {
        assert_eq!(MdflowInputType::parse("text"), MdflowInputType::Text);
        assert_eq!(MdflowInputType::parse("TEXT"), MdflowInputType::Text);
        assert_eq!(MdflowInputType::parse("select"), MdflowInputType::Select);
        assert_eq!(MdflowInputType::parse("number"), MdflowInputType::Number);
        assert_eq!(MdflowInputType::parse("confirm"), MdflowInputType::Confirm);
        assert_eq!(MdflowInputType::parse("boolean"), MdflowInputType::Confirm);
        assert_eq!(
            MdflowInputType::parse("password"),
            MdflowInputType::Password
        );
        assert_eq!(MdflowInputType::parse("secret"), MdflowInputType::Password);
        assert_eq!(MdflowInputType::parse("unknown"), MdflowInputType::Text);
    }

    // === AgentExecutionMode tests ===

    #[test]
    fn test_execution_mode_default() {
        assert_eq!(AgentExecutionMode::default(), AgentExecutionMode::UiCapture);
    }

    // === AgentAvailability tests ===

    #[test]
    fn test_availability_both_available() {
        let avail = AgentAvailability {
            mdflow_available: true,
            backend_available: true,
            error_message: None,
        };
        assert!(avail.is_available());
    }

    #[test]
    fn test_availability_mdflow_missing() {
        let avail = AgentAvailability {
            mdflow_available: false,
            backend_available: true,
            error_message: Some("mdflow not found".to_string()),
        };
        assert!(!avail.is_available());
    }

    #[test]
    fn test_availability_backend_missing() {
        let avail = AgentAvailability {
            mdflow_available: true,
            backend_available: false,
            error_message: Some("claude not found".to_string()),
        };
        assert!(!avail.is_available());
    }

    // === Agent default tests ===

    #[test]
    fn test_agent_default() {
        let agent = Agent::default();
        assert!(agent.name.is_empty());
        assert!(agent.path.as_os_str().is_empty());
        assert_eq!(agent.backend, AgentBackend::Generic);
        assert!(!agent.interactive);
        assert!(agent.description.is_none());
        assert!(agent.icon.is_none());
        assert!(!agent.has_shell_inlines);
        assert!(!agent.has_remote_imports);
    }

    // === AgentFrontmatter default tests ===

    #[test]
    fn test_frontmatter_default() {
        let fm = AgentFrontmatter::default();
        assert!(fm.raw.is_empty());
        assert!(fm.sk_name.is_none());
        assert!(fm.sk_description.is_none());
        assert!(fm.inputs.is_none());
        assert!(fm.interactive.is_none());
        assert!(fm.cwd.is_none());
        assert!(fm.command.is_none());
        assert!(fm.env.is_none());
    }
}
