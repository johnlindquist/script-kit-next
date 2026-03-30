//! Tab AI harness configuration and context formatting.
//!
//! Defines the contract for connecting Tab AI to an external CLI harness
//! (Claude Code, Codex, Gemini CLI, Copilot CLI, or a custom command).
//! The context assembly pipeline (`TabAiContextBlob`) is unchanged — this
//! module only consumes it.

pub(crate) mod screenshot_files;

pub use screenshot_files::{
    capture_tab_ai_focused_window_screenshot_file, capture_tab_ai_screen_screenshot_file,
    cleanup_old_tab_ai_screenshot_files, cleanup_old_tab_ai_screenshot_files_in_dir,
    tab_ai_screenshot_prefix, TabAiScreenshotFile, TAB_AI_SCREENSHOT_MAX_KEEP,
};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Capture kind
// ---------------------------------------------------------------------------

/// Declares what kind of pre-switch capture the harness launch should perform.
///
/// Threaded through [`TabAiLaunchRequest`] → [`spawn_tab_ai_pre_switch_capture`]
/// so each explicit AI command gets the appropriate screenshot/context capture
/// instead of always defaulting to focused-window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAiCaptureKind {
    /// Default Tab/Shift+Tab path: focused-window screenshot + full desktop context.
    DefaultContext,
    /// Full-screen screenshot (e.g. `SendScreenToAi`).
    FullScreen,
    /// Focused-window screenshot (e.g. `SendFocusedWindowToAi`).
    FocusedWindow,
    /// Selected text context only — no screenshot (e.g. `SendSelectedTextToAi`).
    SelectedText,
    /// Browser tab URL context only — no screenshot (e.g. `SendBrowserTabToAi`).
    BrowserTab,
}

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
        let working_directory = Some(crate::setup::get_kit_path().to_string_lossy().into_owned());
        Self {
            schema_version: TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION,
            backend: HarnessBackendKind::ClaudeCode,
            command: "claude".to_string(),
            args: Vec::new(),
            warm_on_startup: default_tab_ai_harness_warm_on_startup(),
            working_directory,
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

/// Whether a harness session is a fresh prewarm (reusable once) or has been
/// consumed by a user-initiated Tab entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAiHarnessWarmState {
    /// Silently prewarmed — can be reused exactly once by the next Tab press.
    FreshPrewarm,
    /// Already consumed by a user interaction — must be torn down before reuse.
    Consumed,
}

/// Runtime state for a live harness terminal session.
#[derive(Clone)]
pub struct TabAiHarnessSessionState {
    pub config: HarnessConfig,
    pub entity: gpui::Entity<crate::term_prompt::TermPrompt>,
    pub id: String,
    pub warm_state: TabAiHarnessWarmState,
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
            warm_state: TabAiHarnessWarmState::Consumed,
        }
    }

    /// Returns `true` if this session is a fresh prewarm that has not yet been
    /// consumed by a user Tab press.
    pub fn is_fresh_prewarm(&self) -> bool {
        matches!(self.warm_state, TabAiHarnessWarmState::FreshPrewarm)
    }

    /// Mark the session as a newly created prewarm that may be reused once.
    pub fn mark_fresh_prewarm(&mut self) {
        self.warm_state = TabAiHarnessWarmState::FreshPrewarm;
    }

    /// Mark the session as consumed so it cannot be reused again.
    pub fn mark_consumed(&mut self) {
        self.warm_state = TabAiHarnessWarmState::Consumed;
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
        // Default working_directory resolves to the Script Kit root (~/.scriptkit)
        assert!(
            config.working_directory.is_some(),
            "default working_directory should be set to scriptkit root"
        );
        let wd = config.working_directory.as_ref().unwrap();
        assert!(
            wd.contains("scriptkit") || wd.contains("script-kit"),
            "working_directory should point to scriptkit root, got: {wd}"
        );
        assert!(config.env.is_empty());
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
            working_directory: None,
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

    #[test]
    fn claude_md_documents_quick_terminal_as_primary_tab_surface() {
        let doc = include_str!("../../../CLAUDE.md");
        assert!(
            doc.contains("Shift+Tab` in `AppView::ScriptList` with non-empty filter text"),
            "CLAUDE.md must document Shift+Tab entry-intent routing"
        );
        assert!(
            doc.contains("`Tab` / `Shift+Tab` inside `AppView::QuickTerminalView`"),
            "CLAUDE.md must document PTY-owned Tab handling inside QuickTerminalView"
        );
        assert!(
            doc.contains("Legacy compatibility only"),
            "CLAUDE.md must describe TabAiChat as compatibility-only"
        );
    }

    #[test]
    fn standard_startup_shift_tab_routes_into_harness_entry_intent() {
        let source = include_str!("../../app_impl/startup.rs");
        // Split at the test module boundary so assertions only inspect
        // production code, not their own string literals.
        let production = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("file has content before #[cfg(test)]");
        assert!(
            production.contains("open_tab_ai_chat_with_entry_intent(Some(query), cx)"),
            "Shift+Tab in ScriptList must route the filter text into harness entry intent"
        );
        let legacy_call = format!("{}(query, cx)", "dispatch_ai_script_generation_from_query");
        assert!(
            !production.contains(&legacy_call),
            "Standard startup must not keep the legacy Shift+Tab script-generation path"
        );
    }

    fn extract_tab_ai_quick_terminal_section(doc: &str) -> &str {
        let start = doc
            .find("### Tab AI — Quick Terminal with Context Injection")
            .expect("doc must contain Tab AI quick terminal section");
        let rest = &doc[start..];
        let end = rest[1..]
            .find("\n### ")
            .map(|idx| idx + 1)
            .unwrap_or(rest.len());
        &rest[..end]
    }

    #[test]
    fn agent_docs_keep_quick_terminal_section_identical() {
        const CLAUDE_DOC: &str = include_str!("../../../CLAUDE.md");
        const AGENTS_DOC: &str = include_str!("../../../AGENTS.md");
        assert_eq!(
            extract_tab_ai_quick_terminal_section(CLAUDE_DOC),
            extract_tab_ai_quick_terminal_section(AGENTS_DOC),
            "CLAUDE.md and AGENTS.md must keep the Tab AI quick-terminal section byte-for-byte identical"
        );
    }

    #[test]
    fn agent_docs_match_actual_lifecycle_and_submit_semantics() {
        const CLAUDE_DOC: &str = include_str!("../../../CLAUDE.md");
        const AGENTS_DOC: &str = include_str!("../../../AGENTS.md");
        for (label, text) in [("CLAUDE.md", CLAUDE_DOC), ("AGENTS.md", AGENTS_DOC)] {
            let section = extract_tab_ai_quick_terminal_section(text);
            assert!(
                section.contains("silently prewarms the configured harness at app launch"),
                "{label} must describe startup prewarm as the default path"
            );
            assert!(
                section.contains("Await the user's next terminal input."),
                "{label} must describe sentinel behavior"
            );
            assert!(
                !section.contains("First Tab press spawns the configured harness CLI in a PTY"),
                "{label} must not claim first-Tab spawn as the default lifecycle"
            );
            assert!(
                !section.contains(
                    "`Submit` — used when a non-empty entry intent is supplied. Appends a sentinel asking the harness to wait"
                ),
                "{label} must not claim Submit-with-intent appends the wait sentinel"
            );
        }
    }

    #[test]
    fn agent_docs_describe_quick_terminal_contract() {
        const CLAUDE_DOC: &str = include_str!("../../../CLAUDE.md");
        const AGENTS_DOC: &str = include_str!("../../../AGENTS.md");

        for (label, text) in [("CLAUDE.md", CLAUDE_DOC), ("AGENTS.md", AGENTS_DOC)] {
            assert!(
                text.contains("QuickTerminalView"),
                "{label} must mention QuickTerminalView"
            );
            assert!(
                text.contains("build_tab_ai_harness_submission"),
                "{label} must mention harness submission"
            );
            assert!(
                text.contains("CaptureContextOptions::tab_ai_submit()"),
                "{label} must mention text-safe PTY capture"
            );
            assert!(
                text.contains("~/.scriptkit/harness.json"),
                "{label} must mention harness config path"
            );
            assert!(
                text.contains("warmOnStartup"),
                "{label} must mention warmOnStartup"
            );
            assert!(
                text.contains("Cmd+W"),
                "{label} must document wrapper close"
            );
            assert!(
                text.contains("Escape"),
                "{label} must mention PTY escape passthrough"
            );
            assert!(
                !text.contains(
                    "AppView::TabAiChat` variant \u{2014} full-view replacement (primary path via `open_tab_ai_chat()`)"
                ),
                "{label} must not describe TabAiChat as the default Tab path"
            );
            assert!(
                !text.contains("inline AI chat opens"),
                "{label} must not describe inline chat as the default Tab destination"
            );
            assert!(
                !text.contains("dispatch_ai_script_generation_from_query"),
                "{label} must not advertise the legacy Shift+Tab generation bypass"
            );
        }
    }

    #[test]
    fn agent_docs_match_current_context_builder_contract() {
        const CLAUDE_DOC: &str = include_str!("../../../CLAUDE.md");
        const AGENTS_DOC: &str = include_str!("../../../AGENTS.md");
        for (label, text) in [("CLAUDE.md", CLAUDE_DOC), ("AGENTS.md", AGENTS_DOC)] {
            let section = extract_tab_ai_quick_terminal_section(text);
            assert!(
                section.contains("`build_tab_ai_context_from()`"),
                "{label} must describe the current context builder entrypoint"
            );
            assert!(
                section.contains("CaptureContextOptions::tab_ai_submit()"),
                "{label} must reference text-safe PTY capture profile"
            );
            assert!(
                !section.contains("`build_tab_ai_context()`"),
                "{label} must not mention the removed build_tab_ai_context() wording"
            );
            assert!(
                !section.contains("bundle_id + warning count"),
                "{label} must not describe the old TabAiResolvedContext shape"
            );
        }
    }

    #[test]
    fn install_time_root_claude_md_contains_current_quick_terminal_contract() {
        const ROOT_CLAUDE_DOC: &str = include_str!("../../../kit-init/ROOT_CLAUDE.md");
        assert!(
            ROOT_CLAUDE_DOC.contains("`build_tab_ai_context_from()`"),
            "ROOT_CLAUDE.md must describe the current context builder entrypoint"
        );
        assert!(
            ROOT_CLAUDE_DOC.contains("CaptureContextOptions::tab_ai_submit()"),
            "ROOT_CLAUDE.md must reference text-safe PTY capture profile"
        );
        assert!(
            ROOT_CLAUDE_DOC.contains("~/.scriptkit/harness.json"),
            "ROOT_CLAUDE.md must mention harness config path"
        );
        assert!(
            ROOT_CLAUDE_DOC.contains("warmOnStartup"),
            "ROOT_CLAUDE.md must mention warmOnStartup"
        );
        assert!(
            !ROOT_CLAUDE_DOC.contains("`build_tab_ai_context()`"),
            "ROOT_CLAUDE.md must not mention the removed build_tab_ai_context() wording"
        );
        assert!(
            !ROOT_CLAUDE_DOC.contains("bundle_id + warning count"),
            "ROOT_CLAUDE.md must not describe the old TabAiResolvedContext shape"
        );
    }

    #[test]
    fn standard_startup_quick_terminal_tab_writes_directly_to_pty() {
        let source = include_str!("../../app_impl/startup.rs");
        assert!(
            source.contains("b\"\\t\""),
            "QuickTerminal must forward Tab directly to the PTY"
        );
        assert!(
            source.contains("b\"\\x1b[Z\""),
            "QuickTerminal must forward Shift+Tab/backtab directly to the PTY"
        );
        assert!(
            source.contains("term.terminal.input(bytes)"),
            "QuickTerminal Tab handling must write raw bytes to the PTY"
        );
    }
}

// -----------------------------------------------------------------
// Source-level cleanup contract audits
//
// These tests use `include_str!` to verify that the harness-first
// cleanup contracts remain intact: lifecycle teardown, prewarm,
// fallback routing, and legacy command redirection.
// -----------------------------------------------------------------
#[cfg(test)]
mod cleanup_contract_audits {
    fn compact(text: &str) -> String {
        text.chars().filter(|ch| !ch.is_whitespace()).collect()
    }

    #[test]
    fn close_tab_ai_harness_terminal_clears_session_and_rewarms() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("pub(crate) fn close_tab_ai_harness_terminal")
            .expect("close_tab_ai_harness_terminal should exist");
        let rest = &source[start..];
        // Scope to the next function definition so we only audit the close fn.
        let end = rest
            .find("fn schedule_tab_ai_harness_prewarm")
            .expect("schedule_tab_ai_harness_prewarm should follow close fn");
        let body = compact(&rest[..end]);

        for needle in [
            "self.tab_ai_harness_capture_generation+=1;",
            "self.tab_ai_harness_apply_back_route=None;",
            "letsession=self.tab_ai_harness.take();",
            "term.terminate_session()",
            "self.schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250),cx);",
        ] {
            assert!(
                body.contains(&compact(needle)),
                "close_tab_ai_harness_terminal must contain: {needle}"
            );
        }
    }

    #[test]
    fn selection_fallback_send_to_ai_opens_harness_with_query() {
        let source = compact(include_str!("../../app_impl/selection_fallback.rs"));

        assert!(
            source.contains(&compact("FallbackResult::SendToAiHarness { query } =>")),
            "selection fallback must handle the harness-native send-to-ai result"
        );
        assert!(
            source.contains(&compact(
                "self.open_tab_ai_chat_with_entry_intent(Some(normalized), cx);"
            )),
            "non-empty send-to-ai fallback queries must open the harness with entry intent"
        );
    }

    #[test]
    fn builtin_execution_routes_generate_script_to_harness() {
        let source = compact(include_str!("../../app_execute/builtin_execution.rs"));

        assert!(
            source.contains(&compact("AiCommandType::GenerateScript =>")),
            "GenerateScript arm should exist in builtin execution"
        );
        assert!(
            source.contains(&compact(
                "self.open_tab_ai_chat_with_entry_intent(Some(trimmed), cx);"
            )),
            "GenerateScript should submit through the harness"
        );
        assert!(
            !source.contains("show_script_generation_chat"),
            "builtin execution must not call the legacy script-generation chat"
        );
    }

    #[test]
    fn explicit_tab_entry_always_forces_fresh_session() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("fn open_tab_ai_harness_terminal_from_request")
            .expect("open_tab_ai_harness_terminal_from_request should exist");
        let rest = &source[start..];
        let end = rest
            .find("pub(crate) fn warm_tab_ai_harness_on_startup")
            .expect("warm_tab_ai_harness_on_startup should follow open fn");
        let body = compact(&rest[..end]);

        // Every explicit Tab entry must always force a fresh harness session.
        assert!(
            body.contains(&compact(
                "match self.ensure_tab_ai_harness_terminal(true, cx)"
            )),
            "explicit Tab entry must always force a fresh harness session"
        );
        assert!(
            !body.contains("reuse_fresh_prewarm"),
            "literal fresh-session contract must not keep one-time prewarm reuse in the explicit open path"
        );

        // Verify the terminal becomes visible before deferred context injection.
        let full_body = &rest[..end];
        let view_switch = full_body
            .find("self.current_view = AppView::QuickTerminalView")
            .expect("must switch to quick terminal");
        let deferred_inject = full_body
            .rfind("cx.spawn(async move |_this, cx|")
            .expect("must spawn deferred injection task");
        assert!(
            view_switch < deferred_inject,
            "the terminal must become visible before deferred context injection begins"
        );
    }

    #[test]
    fn prewarm_tags_cold_start_sessions_as_fresh() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("fn warm_tab_ai_harness_silently")
            .expect("warm_tab_ai_harness_silently should exist");
        let rest = &source[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .or_else(|| rest[1..].find("\n    pub"))
            .unwrap_or(rest.len());
        let body = compact(&rest[..end]);

        assert!(
            body.contains("mark_fresh_prewarm"),
            "silent prewarm must use the encapsulated mark_fresh_prewarm() helper"
        );
        assert!(
            body.contains(&compact("ensure_tab_ai_harness_terminal(false, cx)")),
            "silent prewarm must use force_fresh=false to avoid killing existing sessions"
        );
    }

    #[test]
    fn session_state_exposes_explicit_one_shot_prewarm_api() {
        let source = include_str!("mod.rs");
        assert!(
            source.contains("pub enum TabAiHarnessWarmState"),
            "session state enum must exist"
        );
        assert!(
            source.contains("FreshPrewarm"),
            "FreshPrewarm variant must exist"
        );
        assert!(source.contains("Consumed"), "Consumed variant must exist");
        assert!(
            source.contains("pub fn is_fresh_prewarm(&self) -> bool"),
            "session must expose is_fresh_prewarm()"
        );
        assert!(
            source.contains("pub fn mark_fresh_prewarm(&mut self)"),
            "session must expose mark_fresh_prewarm()"
        );
        assert!(
            source.contains("pub fn mark_consumed(&mut self)"),
            "session must expose mark_consumed()"
        );
    }

    #[test]
    fn startup_prewarm_delegates_to_silent_helper_with_opt_out() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("pub(crate) fn warm_tab_ai_harness_on_startup")
            .expect("warm_tab_ai_harness_on_startup should exist");
        let rest = &source[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .or_else(|| rest[1..].find("\n    pub"))
            .unwrap_or(rest.len());
        let body = compact(&rest[..end]);

        assert!(
            body.contains(&compact("self.warm_tab_ai_harness_silently(true, cx);")),
            "startup prewarm must delegate to silent helper with respect_startup_opt_out=true"
        );
    }

    #[test]
    fn silent_prewarm_helper_uses_encapsulated_helpers_not_raw_field_writes() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("fn warm_tab_ai_harness_silently")
            .expect("warm_tab_ai_harness_silently should exist");
        let rest = &source[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .or_else(|| rest[1..].find("\n    pub"))
            .unwrap_or(rest.len());
        let body = &rest[..end];

        assert!(
            !body.contains("warm_state ="),
            "silent prewarm must not directly write warm_state — use mark_fresh_prewarm() instead"
        );
        assert!(
            body.contains("mark_fresh_prewarm()"),
            "silent prewarm must use the encapsulated mark_fresh_prewarm() helper"
        );
    }

    #[test]
    fn close_path_tears_down_session_and_reprewarms() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("pub(crate) fn close_tab_ai_harness_terminal")
            .expect("close_tab_ai_harness_terminal should exist");
        let rest = &source[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .or_else(|| rest[1..].find("\n    pub"))
            .unwrap_or(rest.len());
        let body = compact(&rest[..end]);

        assert!(
            body.contains(&compact("let session = self.tab_ai_harness.take();")),
            "close must clear the live harness session"
        );
        assert!(
            body.contains("terminate_session"),
            "close must kill the PTY"
        );
        assert!(
            body.contains(&compact(
                "self.schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250), cx);"
            )),
            "close must queue a silent fresh prewarm for the next Tab press"
        );
    }

    #[test]
    fn prompt_ai_dispatch_routes_script_generation_to_harness() {
        let source = compact(include_str!("../../app_impl/prompt_ai.rs"));

        assert!(
            source.contains(&compact(
                "pub(crate) fn dispatch_ai_script_generation_from_query("
            )),
            "dispatch_ai_script_generation_from_query should exist"
        );
        assert!(
            source.contains(&compact(
                "self.open_tab_ai_chat_with_entry_intent(Some(query), cx);"
            )),
            "dispatch_ai_script_generation_from_query must route to the harness"
        );
        assert!(
            !source.contains(&compact("show_script_generation_chat()")),
            "dispatch_ai_script_generation_from_query must not call the legacy chat"
        );
    }

    #[test]
    fn force_fresh_path_propagates_terminate_failures() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let start = source
            .find("fn ensure_tab_ai_harness_terminal")
            .expect("ensure_tab_ai_harness_terminal should exist");
        let rest = &source[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .or_else(|| rest[1..].find("\n    pub"))
            .unwrap_or(rest.len());
        let body = compact(&rest[..end]);

        // The force-fresh path must propagate terminate failures via `?`
        // instead of silently discarding them with `let _ = ...`.
        assert!(
            body.contains(&compact(
                "existing.entity.update(cx, |term, _cx| { term.terminate_session().map_err(|e| e.to_string()) })?;"
            )),
            "force-fresh path must propagate terminate_session failures with `?`"
        );
        assert!(
            !body.contains(&compact("let _ = existing.entity.update")),
            "force-fresh path must not discard terminate failures"
        );
        // Handle must NOT be cleared before terminate succeeds.
        assert!(
            !body.contains(&compact("self.tab_ai_harness.take()")),
            "force-fresh path must not use .take() which clears the handle before terminate"
        );
    }

    // ── Acceptance-criteria contract tests ──────────────────────

    fn extract_fn_body(source: &str, signature: &str) -> String {
        let start = source.find(signature).expect("signature must exist");
        let rest = &source[start..];
        let open = rest.find('{').expect("function body must open");
        let mut depth = 0usize;
        let mut end = None;
        for (idx, ch) in rest[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + idx + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        rest[..end.expect("function body must close")].to_string()
    }

    #[test]
    fn tab_ai_open_path_always_forces_fresh_session_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "fn open_tab_ai_harness_terminal_from_request",
        ));

        assert!(
            body.contains(&compact(
                "match self.ensure_tab_ai_harness_terminal(true, cx)"
            )),
            "explicit Tab entry must always force a fresh harness session"
        );
        assert!(
            !body.contains("reuse_fresh_prewarm"),
            "literal fresh-session contract must not keep one-time prewarm reuse in the explicit open path"
        );
    }

    #[test]
    fn force_fresh_path_clears_session_only_after_successful_terminate_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "fn ensure_tab_ai_harness_terminal",
        ));

        let terminate_pos = body
            .find(&compact(
                "existing.entity.update(cx, |term, _cx| { term.terminate_session().map_err(|e| e.to_string()) })?;"
            ))
            .expect("terminate_session call must exist in force-fresh path");

        let clear_pos = body
            .find(&compact("self.tab_ai_harness = None;"))
            .expect("session clear must exist after terminate success");

        assert!(
            terminate_pos < clear_pos,
            "force-fresh path must clear self.tab_ai_harness only after terminate_session succeeds"
        );
    }

    #[test]
    fn tab_ai_silent_prewarm_is_marked_fresh_on_cold_start_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "fn warm_tab_ai_harness_silently",
        ));
        assert!(
            body.contains(&compact("if was_cold_start {")),
            "silent prewarm helper must gate FreshPrewarm tagging on a newly created session"
        );
        assert!(
            body.contains(&compact("session.mark_fresh_prewarm();")),
            "cold-started prewarm must be marked reusable once"
        );
        assert!(
            body.contains(&compact("self.ensure_tab_ai_harness_terminal(false, cx)")),
            "silent prewarm helper must never force-fresh kill an existing live session"
        );
    }

    #[test]
    fn tab_ai_close_path_reseeds_future_prewarm_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "pub(crate) fn close_tab_ai_harness_terminal",
        ));
        assert!(
            body.contains(&compact(
                "let session = self.tab_ai_harness.take();"
            )),
            "close path must clear the current PTY session"
        );
        assert!(
            body.contains(&compact(
                "self.schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250), cx);"
            )),
            "close path must schedule a fresh prewarm for the next Tab press"
        );
    }

    #[test]
    fn tab_ai_open_path_switches_view_before_waiting_for_capture_contract() {
        let body = extract_fn_body(
            include_str!("../../app_impl/tab_ai_mode.rs"),
            "fn open_tab_ai_harness_terminal_from_request",
        );

        let view_switch = body
            .find("self.current_view = AppView::QuickTerminalView")
            .expect("QuickTerminalView switch must exist");

        // Find the cx.notify() that comes AFTER the view switch (not the
        // error-path notify that precedes it).
        let notify = body[view_switch..]
            .find("cx.notify()")
            .map(|offset| view_switch + offset)
            .expect("cx.notify() must follow the view switch");

        let capture_wait = body
            .find("capture_rx.recv().await")
            .expect("deferred capture await must exist");

        assert!(
            view_switch < notify,
            "the harness view must be selected before notifying the UI"
        );
        assert!(
            notify < capture_wait,
            "the terminal must become visible before waiting for deferred capture"
        );
    }

    // ── Post-close prewarm split contracts ─────────────────────

    #[test]
    fn post_close_prewarm_uses_dedicated_helper_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let schedule_body = compact(&extract_fn_body(
            source,
            "fn schedule_tab_ai_harness_prewarm",
        ));

        assert!(
            schedule_body.contains(&compact("this.warm_tab_ai_harness_after_close(cx);")),
            "close-cycle scheduler must call warm_tab_ai_harness_after_close()"
        );
        assert!(
            !schedule_body.contains(&compact("this.warm_tab_ai_harness_on_startup(cx);")),
            "close-cycle scheduler must not route through startup-only prewarm"
        );
    }

    #[test]
    fn startup_and_post_close_prewarm_split_opt_out_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");

        let startup_body = compact(&extract_fn_body(
            source,
            "pub(crate) fn warm_tab_ai_harness_on_startup",
        ));
        assert!(
            startup_body.contains(&compact("self.warm_tab_ai_harness_silently(true, cx);")),
            "startup prewarm must continue respecting warmOnStartup=false via true arg"
        );

        let after_close_body = compact(&extract_fn_body(
            source,
            "fn warm_tab_ai_harness_after_close",
        ));
        assert!(
            after_close_body.contains(&compact("self.warm_tab_ai_harness_silently(false, cx);")),
            "post-close prewarm must bypass the startup-only opt-out via false arg"
        );
    }

    #[test]
    fn silent_prewarm_helper_still_marks_cold_start_as_fresh_contract() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "fn warm_tab_ai_harness_silently",
        ));

        assert!(
            body.contains(&compact("if was_cold_start {")),
            "silent prewarm helper must still gate fresh tagging on newly created sessions"
        );
        assert!(
            body.contains(&compact("session.mark_fresh_prewarm();")),
            "silent prewarm helper must still mark cold-started sessions as reusable once"
        );
        assert!(
            body.contains(&compact("self.ensure_tab_ai_harness_terminal(false, cx)")),
            "silent prewarm helper must never force-fresh kill an existing live session"
        );
    }

    // ── Source/apply-back provenance unification contracts ─────

    // ── Screenshot helper & builtin registry audits ────────────

    const SCREENSHOT_FILES_SOURCE: &str = include_str!("screenshot_files.rs");
    const BUILTINS_SOURCE: &str = include_str!("../../builtins/mod.rs");

    #[test]
    fn full_screen_capture_helper_contract_is_preserved() {
        assert!(
            SCREENSHOT_FILES_SOURCE
                .contains("pub fn capture_tab_ai_screen_screenshot_file()"),
            "full-screen screenshot helper must exist as a public function",
        );
        assert!(
            SCREENSHOT_FILES_SOURCE.contains("capture_screen_screenshot()"),
            "full-screen screenshot helper must call the platform full-screen screenshot API",
        );
        assert!(
            SCREENSHOT_FILES_SOURCE.contains("cleanup_old_tab_ai_screenshot_files"),
            "full-screen screenshot helper must clean up old screenshot temp files",
        );
        assert!(
            SCREENSHOT_FILES_SOURCE.contains("TAB_AI_SCREENSHOT_MAX_KEEP"),
            "full-screen screenshot helper must use the shared screenshot retention limit",
        );
        assert!(
            SCREENSHOT_FILES_SOURCE.contains("title: \"Full Screen\".to_string()"),
            "full-screen screenshot helper must label the artifact as Full Screen",
        );
        assert!(
            SCREENSHOT_FILES_SOURCE.contains("used_fallback: false"),
            "full-screen screenshot helper must set used_fallback to false",
        );
    }

    #[test]
    fn builtin_registry_keeps_harness_entries_and_manual_paths_only() {
        let fn_start = BUILTINS_SOURCE
            .find("pub fn get_builtin_entries(")
            .expect("get_builtin_entries must exist");
        let fn_body = &BUILTINS_SOURCE[fn_start..];
        let fn_end = fn_body
            .find("\n#[cfg(test)]")
            .unwrap_or(fn_body.len());
        let registration_section = &fn_body[..fn_end];

        for legacy_id in [
            "builtin-open-ai-chat",
            "builtin-mini-ai-chat",
            "builtin-new-conversation",
            "builtin-clear-conversation",
            "builtin-send-screen-area-to-ai",
        ] {
            let quoted = format!("\"{}\"", legacy_id);
            assert!(
                !registration_section.contains(&quoted),
                "{legacy_id} must not be registered in the main builtin list",
            );
        }

        for kept_id in [
            "builtin-generate-script-with-ai",
            "builtin-generate-script-from-current-app",
            "builtin-send-screen-to-ai",
            "builtin-send-selected-text-to-ai",
            "builtin-send-browser-tab-to-ai",
            "builtin-new-script",
            "builtin-new-extension",
        ] {
            let quoted = format!("\"{}\"", kept_id);
            assert!(
                registration_section.contains(&quoted),
                "{kept_id} must stay registered in the main builtin list",
            );
        }
    }

    #[test]
    fn focused_window_builtin_uses_canonical_id() {
        let fn_start = BUILTINS_SOURCE
            .find("pub fn get_builtin_entries(")
            .expect("get_builtin_entries must exist");
        let fn_body = &BUILTINS_SOURCE[fn_start..];
        let fn_end = fn_body
            .find("\n#[cfg(test)]")
            .unwrap_or(fn_body.len());
        let registration_section = &fn_body[..fn_end];

        assert!(
            registration_section.contains("\"builtin-send-focused-window-to-ai\""),
            "SendFocusedWindowToAi must use the canonical focused-window builtin id",
        );
        assert!(
            !registration_section.contains("\"builtin-send-window-to-ai\""),
            "legacy short focused-window builtin id must not remain in the main builtin list",
        );
    }

    #[test]
    fn detect_source_type_delegates_to_canonical_function() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "fn detect_tab_ai_source_type(",
        ));

        assert!(
            body.contains(&compact(
                "crate::ai::detect_tab_ai_source_type_from_prompt("
            )),
            "detect_tab_ai_source_type must delegate to canonical crate::ai function"
        );
        assert!(
            body.contains(&compact("app_view_to_prompt_type_str(source_view),")),
            "detect_tab_ai_source_type must convert AppView via app_view_to_prompt_type_str"
        );
    }

    #[test]
    fn build_apply_back_hint_delegates_to_canonical_function() {
        let source = include_str!("../../app_impl/tab_ai_mode.rs");
        let body = compact(&extract_fn_body(
            source,
            "fn build_tab_ai_apply_back_hint(",
        ));

        assert!(
            body.contains(&compact(
                "crate::ai::build_tab_ai_apply_back_hint_from_source(source_type)"
            )),
            "build_tab_ai_apply_back_hint must delegate to canonical crate::ai function"
        );
    }
}
