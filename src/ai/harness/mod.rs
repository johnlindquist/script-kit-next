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
    #[serde(default = "default_tab_ai_harness_warm_on_startup")]
    pub warm_on_startup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub env: std::collections::BTreeMap<String, String>,
}

/// Default value for [`HarnessConfig::warm_on_startup`].
///
/// Returns `true` so that omitting the field from JSON (or using
/// `HarnessConfig::default()`) enables prewarm.  Users opt *out*
/// with `"warmOnStartup": false`.
fn default_tab_ai_harness_warm_on_startup() -> bool {
    true
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            schema_version: TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION,
            backend: HarnessBackendKind::ClaudeCode,
            command: "claude".to_string(),
            args: Vec::new(),
            warm_on_startup: default_tab_ai_harness_warm_on_startup(),
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

// ---------------------------------------------------------------------------
// Hints block (invocation receipt + suggested intents)
// ---------------------------------------------------------------------------

/// Serializable hints block appended after the context block.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TabAiHarnessHints<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    invocation_receipt: Option<&'a crate::ai::TabAiInvocationReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    suggested_intents: Vec<crate::ai::TabAiSuggestedIntentSpec>,
}

/// Build the optional `<scriptKitHints>` block from receipt + suggestions.
/// Returns `None` when both are empty (no block emitted).
fn build_tab_ai_harness_hints_block(
    invocation_receipt: Option<&crate::ai::TabAiInvocationReceipt>,
    suggested_intents: &[crate::ai::TabAiSuggestedIntentSpec],
) -> Result<Option<String>, String> {
    if invocation_receipt.is_none() && suggested_intents.is_empty() {
        return Ok(None);
    }
    let hints = TabAiHarnessHints {
        invocation_receipt,
        suggested_intents: suggested_intents.to_vec(),
    };
    let hints_json = serde_json::to_string_pretty(&hints)
        .map_err(|e| format!("tab_ai_harness_hints_serialize_failed: {e}"))?;
    Ok(Some(format!(
        "<scriptKitHints>\n\
         Use these to understand capture quality and suggest strong first actions.\n\
         ```json\n\
         {hints_json}\n\
         ```\n\
         </scriptKitHints>"
    )))
}

// ---------------------------------------------------------------------------
// Full submission builder
// ---------------------------------------------------------------------------

/// Build a full harness submission: context block + optional hints + optional user intent.
///
/// Behavior depends on `mode`:
/// - `Submit` without intent: appends a sentinel asking the harness to wait.
/// - `PasteOnly` without intent: stages context only, no synthetic turn text.
/// - With intent (either mode): appends the intent as `User intent:`.
///
/// When `invocation_receipt` or `suggested_intents` are provided, a
/// `<scriptKitHints>` block is appended between the context and intent.
pub fn build_tab_ai_harness_submission(
    context: &crate::ai::TabAiContextBlob,
    intent: Option<&str>,
    mode: TabAiHarnessSubmissionMode,
    invocation_receipt: Option<&crate::ai::TabAiInvocationReceipt>,
    suggested_intents: &[crate::ai::TabAiSuggestedIntentSpec],
) -> Result<String, String> {
    let mut output = build_tab_ai_harness_context_block(context)?;

    if let Some(hints_block) =
        build_tab_ai_harness_hints_block(invocation_receipt, suggested_intents)?
    {
        output.push_str("\n\n");
        output.push_str(&hints_block);
    }

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
            // PasteOnly: stage context only, but leave the cursor on a fresh
            // line so the user's next keystrokes do not join the closing tag.
            if !output.ends_with('\n') {
                output.push('\n');
            }
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
        assert!(config.warm_on_startup);
        assert!(config.working_directory.is_none());
        assert!(config.env.is_empty());
        assert_eq!(config.command_line(), "claude");
    }

    #[test]
    fn harness_config_missing_warm_on_startup_field_defaults_to_true() {
        let json = r#"{
            "schemaVersion": 1,
            "backend": "claudeCode",
            "command": "claude"
        }"#;
        let parsed: HarnessConfig = serde_json::from_str(json).expect("deserialize");
        assert!(parsed.warm_on_startup);
    }

    #[test]
    fn harness_config_explicit_false_preserves_opt_out() {
        let json = r#"{
            "schemaVersion": 1,
            "backend": "claudeCode",
            "command": "claude",
            "warmOnStartup": false
        }"#;
        let parsed: HarnessConfig = serde_json::from_str(json).expect("deserialize");
        assert!(!parsed.warm_on_startup);
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
            None,
            &[],
        )
        .expect("should build");
        assert!(with_intent.contains("<scriptKitContext schemaVersion=\"1\">"));
        assert!(with_intent.contains("\"promptType\": \"FileSearch\""));
        assert!(with_intent.contains("User intent:\nrename this file"));
        assert!(!with_intent.contains("Await the user"));

        // Without intent (Submit mode) — sentinel present
        let without_intent = build_tab_ai_harness_submission(
            &context,
            None,
            TabAiHarnessSubmissionMode::Submit,
            None,
            &[],
        )
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

        let paste = build_tab_ai_harness_submission(
            &context,
            None,
            TabAiHarnessSubmissionMode::PasteOnly,
            None,
            &[],
        )
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
            None,
            &[],
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

    fn sample_context_with_focused_window() -> crate::ai::TabAiContextBlob {
        crate::ai::TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                input_text: Some("finder".to_string()),
                ..Default::default()
            },
            crate::context_snapshot::AiContextSnapshot {
                focused_window: Some(crate::context_snapshot::FocusedWindowContext {
                    title: "Finder — Downloads".to_string(),
                    width: 1440,
                    height: 900,
                    used_fallback: false,
                }),
                ..Default::default()
            },
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
            "2026-03-29T18:10:15Z".to_string(),
        )
    }

    #[test]
    fn paste_only_submission_stages_context_without_sentinel() {
        let submission = build_tab_ai_harness_submission(
            &sample_context_with_focused_window(),
            None,
            TabAiHarnessSubmissionMode::PasteOnly,
            None,
            &[],
        )
        .expect("submission");
        let expected_open = format!(
            "<scriptKitContext schemaVersion=\"{}\">",
            TAB_AI_HARNESS_CONTEXT_SCHEMA_VERSION
        );
        assert!(submission.contains(&expected_open));
        assert!(submission.contains("\"focusedWindow\""));
        assert!(!submission.contains("\"focusedWindowImage\""));
        assert!(!submission.contains("Await the user's next terminal input."));
        assert!(!submission.contains("User intent:"));
    }

    #[test]
    fn submit_without_intent_appends_wait_sentinel() {
        let submission = build_tab_ai_harness_submission(
            &sample_context_with_focused_window(),
            None,
            TabAiHarnessSubmissionMode::Submit,
            None,
            &[],
        )
        .expect("submission");
        assert!(submission.contains("Await the user's next terminal input."));
    }

    #[test]
    fn paste_only_submission_ends_on_fresh_line() {
        let submission = build_tab_ai_harness_submission(
            &sample_context_with_focused_window(),
            None,
            TabAiHarnessSubmissionMode::PasteOnly,
            None,
            &[],
        )
        .expect("submission");
        assert!(
            submission.ends_with("</scriptKitContext>\n"),
            "PasteOnly must leave the cursor on the next line after the context block: {submission:?}"
        );
        assert!(!submission.contains("Await the user's next terminal input."));
        assert!(!submission.contains("User intent:"));
    }

    #[test]
    fn paste_only_submission_keeps_next_user_input_separate_from_context_block() {
        let submission = build_tab_ai_harness_submission(
            &sample_context_with_focused_window(),
            None,
            TabAiHarnessSubmissionMode::PasteOnly,
            None,
            &[],
        )
        .expect("submission");
        let composed = format!("{submission}rename this file\n");
        assert!(
            composed.contains("</scriptKitContext>\nrename this file\n"),
            "user input must start on a fresh line after the context block: {composed:?}"
        );
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

    #[test]
    fn paste_only_submission_includes_hints_block_when_receipt_or_suggestions_exist() {
        let context = crate::ai::TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "FileSearch".to_string(),
                ..Default::default()
            },
            Default::default(),
            vec![],
            None,
            vec![],
            vec![],
            "2026-03-29T18:10:15Z".to_string(),
        );

        let receipt = crate::ai::TabAiInvocationReceipt {
            schema_version: crate::ai::TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION,
            prompt_type: "FileSearch".to_string(),
            input_status: crate::ai::TabAiFieldStatus::Captured,
            focus_status: crate::ai::TabAiFieldStatus::Captured,
            elements_status: crate::ai::TabAiFieldStatus::Captured,
            element_count: 3,
            warning_count: 0,
            has_focus_target: true,
            has_input_text: false,
            degradation_reasons: vec![],
            rich: true,
        };

        let suggestions = vec![
            crate::ai::TabAiSuggestedIntentSpec::new("Summarize", "summarize this file"),
            crate::ai::TabAiSuggestedIntentSpec::new("Rename", "rename this file"),
        ];

        let submission = build_tab_ai_harness_submission(
            &context,
            None,
            TabAiHarnessSubmissionMode::PasteOnly,
            Some(&receipt),
            &suggestions,
        )
        .expect("submission");

        assert!(submission.contains("<scriptKitHints>"));
        assert!(submission.contains("\"promptType\": \"FileSearch\""));
        assert!(submission.contains("\"intent\": \"summarize this file\""));
        assert!(submission.ends_with('\n'));
    }

    #[test]
    fn paste_only_submission_omits_hints_block_when_no_receipt_or_suggestions() {
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
            "2026-03-29T18:10:15Z".to_string(),
        );

        let submission = build_tab_ai_harness_submission(
            &context,
            None,
            TabAiHarnessSubmissionMode::PasteOnly,
            None,
            &[],
        )
        .expect("submission");

        assert!(!submission.contains("<scriptKitHints>"));
        assert!(submission.contains("<scriptKitContext"));
    }
}
