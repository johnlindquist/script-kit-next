//! Tab AI harness configuration and context formatting.
//!
//! Defines the contract for connecting Tab AI to an external CLI harness
//! (Claude Code, Codex, Gemini CLI, Copilot CLI, or a custom command).
//! The context assembly pipeline (`TabAiContextBlob`) is unchanged — this
//! module only consumes it.

use serde::{Deserialize, Serialize};

/// Schema version for `HarnessConfig` wire format.
pub const TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `<scriptKitContext>` block injected into harnesses.
pub const TAB_AI_HARNESS_CONTEXT_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which CLI harness to connect to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HarnessBackendKind {
    ClaudeCode,
    Codex,
    GeminiCli,
    CopilotCli,
    Custom,
}

/// Persisted configuration for the Tab AI harness.
///
/// Stored at `~/.scriptkit/harness.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessConfig {
    pub schema_version: u32,
    pub backend: HarnessBackendKind,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default)]
    pub warm_on_startup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub env: std::collections::BTreeMap<String, String>,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            schema_version: TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION,
            backend: HarnessBackendKind::ClaudeCode,
            command: "claude".to_string(),
            args: Vec::new(),
            warm_on_startup: false,
            working_directory: None,
            env: std::collections::BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Shell quoting
// ---------------------------------------------------------------------------

/// Minimally shell-quote a value.  Safe characters pass through; everything
/// else gets single-quoted with internal `'` escaped via `'"'"'`.
fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "/._-:=@".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', r#"'"'"'"#))
    }
}

fn is_valid_shell_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

impl HarnessConfig {
    /// Build a shell command line from this config.
    ///
    /// Includes env vars as a prefix and `cd <dir> &&` when a working
    /// directory is set.
    pub fn command_line(&self) -> String {
        let command_and_args = std::iter::once(shell_quote(&self.command))
            .chain(self.args.iter().map(|arg| shell_quote(arg)))
            .collect::<Vec<_>>()
            .join(" ");

        let with_env = if self.env.is_empty() {
            command_and_args
        } else {
            let env_prefix = self
                .env
                .iter()
                .filter(|(key, _)| is_valid_shell_env_key(key))
                .map(|(key, value)| format!("{key}={}", shell_quote(value)))
                .collect::<Vec<_>>()
                .join(" ");
            if env_prefix.is_empty() {
                command_and_args
            } else {
                format!("{env_prefix} {command_and_args}")
            }
        };

        match &self.working_directory {
            Some(dir) if !dir.trim().is_empty() => {
                format!("cd {} && {}", shell_quote(dir), with_env)
            }
            _ => with_env,
        }
    }
}

// ---------------------------------------------------------------------------
// Config I/O
// ---------------------------------------------------------------------------

/// Path to the harness config file.
pub fn tab_ai_harness_config_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "tab_ai_harness_config_path: HOME is not set".to_string())?;
    Ok(std::path::Path::new(&home)
        .join(".scriptkit")
        .join("harness.json"))
}

/// Read (or default) the harness config from disk.
pub fn read_tab_ai_harness_config() -> Result<HarnessConfig, String> {
    let path = tab_ai_harness_config_path()?;
    if !path.exists() {
        return Ok(HarnessConfig::default());
    }
    let json = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "tab_ai_harness_config_read_failed: path={} error={}",
            path.display(),
            e
        )
    })?;
    serde_json::from_str(&json).map_err(|e| {
        format!(
            "tab_ai_harness_config_parse_failed: path={} error={}",
            path.display(),
            e
        )
    })
}

// ---------------------------------------------------------------------------
// Config validation
// ---------------------------------------------------------------------------

/// Validate that a harness config is usable: command is non-empty and the
/// binary is on PATH. Returns an actionable error message on failure.
pub fn validate_tab_ai_harness_config(config: &HarnessConfig) -> Result<(), String> {
    if config.command.trim().is_empty() {
        return Err(
            "Harness command is empty. Set a command in ~/.scriptkit/harness.json \
             or delete the file to use the default (claude)."
                .to_string(),
        );
    }
    if which::which(&config.command).is_err() {
        return Err(format!(
            "'{}' not found on PATH. Install the CLI or update the \
             \"command\" field in ~/.scriptkit/harness.json.",
            config.command,
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Session state
// ---------------------------------------------------------------------------

/// Runtime state for a live harness terminal session.
#[derive(Clone)]
pub struct TabAiHarnessSessionState {
    pub config: HarnessConfig,
    pub entity: gpui::Entity<crate::term_prompt::TermPrompt>,
    pub id: String,
}

impl TabAiHarnessSessionState {
    pub fn new(
        config: HarnessConfig,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        id: impl Into<String>,
    ) -> Self {
        Self {
            config,
            entity,
            id: id.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Context formatting
// ---------------------------------------------------------------------------

/// Whether to submit context as a full turn or stage it for later input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAiHarnessSubmissionMode {
    /// Submit immediately as a full harness turn.
    Submit,
    /// Paste/stage context only; user will type intent next.
    PasteOnly,
}

/// Build a `<scriptKitContext>` XML block from a resolved context blob.
pub fn build_tab_ai_harness_context_block(
    context: &crate::ai::TabAiContextBlob,
) -> Result<String, String> {
    let context_json = serde_json::to_string_pretty(context)
        .map_err(|e| format!("tab_ai_harness_context_serialize_failed: {e}"))?;
    Ok(format!(
        "<scriptKitContext schemaVersion=\"{schema}\">\n\
         Use this as ambient context for the next user request.\n\
         Do not quote the whole block back unless the user asks.\n\
         Prefer focusedTarget over visibleTargets when the user says \"this\", \"it\", or \"selected\".\n\
         ```json\n\
         {context_json}\n\
         ```\n\
         </scriptKitContext>",
        schema = TAB_AI_HARNESS_CONTEXT_SCHEMA_VERSION,
    ))
}

/// Build a full harness submission: context block + optional user intent.
///
/// Behavior depends on `mode`:
/// - `Submit` without intent: appends a sentinel asking the harness to wait.
/// - `PasteOnly` without intent: stages context only, no synthetic turn text.
/// - With intent (either mode): appends the intent as `User intent:`.
pub fn build_tab_ai_harness_submission(
    context: &crate::ai::TabAiContextBlob,
    intent: Option<&str>,
    mode: TabAiHarnessSubmissionMode,
) -> Result<String, String> {
    let mut output = build_tab_ai_harness_context_block(context)?;
    match intent.map(str::trim).filter(|v| !v.is_empty()) {
        Some(intent) => {
            output.push_str("\n\nUser intent:\n");
            output.push_str(intent);
            output.push('\n');
        }
        None if matches!(mode, TabAiHarnessSubmissionMode::Submit) => {
            output.push_str("\n\nAwait the user's next terminal input.\n");
        }
        None => {
            // PasteOnly: stage context only, no synthetic turn text.
        }
    }
    Ok(output)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_config_default_is_claude_code() {
        let config = HarnessConfig::default();
        assert_eq!(config.schema_version, TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION);
        assert_eq!(config.backend, HarnessBackendKind::ClaudeCode);
        assert_eq!(config.command, "claude");
        assert!(config.args.is_empty());
        assert!(!config.warm_on_startup);
        assert!(config.working_directory.is_none());
        assert!(config.env.is_empty());
        assert_eq!(config.command_line(), "claude");
    }

    #[test]
    fn harness_config_command_line_quotes_args_and_directory() {
        let config = HarnessConfig {
            schema_version: TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION,
            backend: HarnessBackendKind::Custom,
            command: "claude".to_string(),
            args: vec!["--resume".to_string(), "project with space".to_string()],
            warm_on_startup: false,
            working_directory: Some("/tmp/my dir".to_string()),
            env: std::collections::BTreeMap::from([("FOO".to_string(), "bar baz".to_string())]),
        };
        assert_eq!(
            config.command_line(),
            "cd '/tmp/my dir' && FOO='bar baz' claude --resume 'project with space'"
        );
    }

    #[test]
    fn harness_config_command_line_ignores_invalid_env_keys() {
        let config = HarnessConfig {
            schema_version: TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION,
            backend: HarnessBackendKind::Custom,
            command: "claude".to_string(),
            args: Vec::new(),
            warm_on_startup: false,
            working_directory: None,
            env: std::collections::BTreeMap::from([
                ("GOOD_KEY".to_string(), "1".to_string()),
                ("BAD-KEY".to_string(), "2".to_string()),
            ]),
        };

        assert_eq!(config.command_line(), "GOOD_KEY=1 claude");
    }

    #[test]
    fn harness_config_command_line_no_working_directory() {
        let config = HarnessConfig {
            command: "codex".to_string(),
            args: vec!["--fast".to_string()],
            ..HarnessConfig::default()
        };
        assert_eq!(config.command_line(), "codex --fast");
    }

    #[test]
    fn harness_config_serde_roundtrip() {
        let config = HarnessConfig {
            schema_version: 1,
            backend: HarnessBackendKind::ClaudeCode,
            command: "claude".to_string(),
            args: vec!["--resume".to_string()],
            warm_on_startup: false,
            working_directory: None,
            env: std::collections::BTreeMap::new(),
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: HarnessConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config, parsed);
    }

    #[test]
    fn harness_submission_wraps_context_and_optional_intent() {
        let context = crate::ai::TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "FileSearch".to_string(),
                input_text: Some("readme".to_string()),
                ..Default::default()
            },
            Default::default(),
            vec![],
            None,
            vec![],
            vec![],
            "2026-03-29T04:39:58Z".to_string(),
        );

        // With intent (Submit mode)
        let with_intent = build_tab_ai_harness_submission(
            &context,
            Some("rename this file"),
            TabAiHarnessSubmissionMode::Submit,
        )
        .expect("should build");
        assert!(with_intent.contains("<scriptKitContext schemaVersion=\"1\">"));
        assert!(with_intent.contains("\"promptType\": \"FileSearch\""));
        assert!(with_intent.contains("User intent:\nrename this file"));
        assert!(!with_intent.contains("Await the user"));

        // Without intent (Submit mode) — sentinel present
        let without_intent =
            build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::Submit)
                .expect("should build");
        assert!(without_intent.contains("<scriptKitContext schemaVersion=\"1\">"));
        assert!(without_intent.contains("Await the user's next terminal input."));
        assert!(!without_intent.contains("User intent:"));
    }

    #[test]
    fn harness_paste_only_omits_sentinel() {
        let context = crate::ai::TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                ..Default::default()
            },
            Default::default(),
            vec![],
            None,
            vec![],
            vec![],
            "2026-03-29T07:07:06Z".to_string(),
        );

        let paste =
            build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::PasteOnly)
                .expect("should build");
        assert!(paste.contains("<scriptKitContext schemaVersion=\"1\">"));
        assert!(!paste.contains("Await the user's next terminal input."));
        assert!(!paste.contains("User intent:"));
    }

    #[test]
    fn harness_paste_only_with_intent_still_includes_intent() {
        let context = crate::ai::TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                ..Default::default()
            },
            Default::default(),
            vec![],
            None,
            vec![],
            vec![],
            "2026-03-29T07:07:06Z".to_string(),
        );

        let paste = build_tab_ai_harness_submission(
            &context,
            Some("open settings"),
            TabAiHarnessSubmissionMode::PasteOnly,
        )
        .expect("should build");
        assert!(paste.contains("User intent:\nopen settings"));
        assert!(!paste.contains("Await the user's next terminal input."));
    }

    #[test]
    fn validate_rejects_empty_command() {
        let config = HarnessConfig {
            command: "".to_string(),
            ..HarnessConfig::default()
        };
        let err = validate_tab_ai_harness_config(&config).unwrap_err();
        assert!(
            err.contains("harness.json"),
            "must mention config file: {err}"
        );
    }

    #[test]
    fn validate_rejects_whitespace_only_command() {
        let config = HarnessConfig {
            command: "   ".to_string(),
            ..HarnessConfig::default()
        };
        let err = validate_tab_ai_harness_config(&config).unwrap_err();
        assert!(err.contains("empty"), "must say command is empty: {err}");
    }

    #[test]
    fn validate_rejects_missing_binary() {
        let config = HarnessConfig {
            command: "nonexistent-binary-xyz-42".to_string(),
            ..HarnessConfig::default()
        };
        let err = validate_tab_ai_harness_config(&config).unwrap_err();
        assert!(
            err.contains("not found on PATH"),
            "must mention PATH: {err}"
        );
        assert!(
            err.contains("harness.json"),
            "must mention config file: {err}"
        );
    }

    #[test]
    fn validate_accepts_known_binary() {
        // `sh` is universally available
        let config = HarnessConfig {
            command: "sh".to_string(),
            ..HarnessConfig::default()
        };
        assert!(validate_tab_ai_harness_config(&config).is_ok());
    }

    #[test]
    fn shell_quote_handles_edge_cases() {
        // Safe string passes through
        assert_eq!(shell_quote("hello"), "hello");
        assert_eq!(shell_quote("/usr/bin/claude"), "/usr/bin/claude");
        assert_eq!(shell_quote("FOO=bar"), "FOO=bar");

        // Spaces get quoted
        assert_eq!(shell_quote("hello world"), "'hello world'");

        // Single quotes get escaped
        assert_eq!(shell_quote("it's"), "'it'\"'\"'s'");
    }
}
