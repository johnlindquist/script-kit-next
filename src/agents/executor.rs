//! Agent execution
//!
//! This module handles executing mdflow agents:
//! - Spawn `mdflow <file>` directly (let mdflow interpret frontmatter)
//! - Add `--_quiet --raw` for UI capture mode
//! - Pass `--_varname value` for user-provided template variables
//! - Pipe stdin for `{{ _stdin }}` support
//! - Set working directory to agent file's parent directory
//!
//! # Execution Model
//!
//! We do NOT convert frontmatter to CLI flags. mdflow handles that.
//! Script Kit only adds:
//! - Mode flags (`--_quiet --raw` for UI capture)
//! - Runtime variable overrides (`--_varname value`)
//! - stdin piping

// These functions are public API for future integration - allow them to be unused for now
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result};
use tracing::{debug, warn};

use crate::agents::types::{Agent, AgentAvailability, AgentExecutionMode};
use crate::setup::get_kit_path;

const SAFE_AGENT_ENV_VARS: [&str; 8] = [
    "PATH",
    "HOME",
    "TMPDIR",
    "USER",
    "LANG",
    "TERM",
    "SHELL",
    "XDG_RUNTIME_DIR",
];

const RESERVED_MDFLOW_VARIABLE_KEYS: [&str; 4] = ["quiet", "context", "env", "command"];

/// Check if mdflow CLI is available in PATH
pub fn is_mdflow_available() -> bool {
    which::which("mdflow").is_ok() || which::which("md").is_ok()
}

/// Get the mdflow command name (prefers "mdflow", falls back to "md")
pub fn get_mdflow_command() -> Option<&'static str> {
    if which::which("mdflow").is_ok() {
        Some("mdflow")
    } else if which::which("md").is_ok() {
        Some("md")
    } else {
        None
    }
}

/// Check availability of an agent (mdflow + backend)
pub fn check_availability(agent: &Agent) -> AgentAvailability {
    let mdflow_available = is_mdflow_available();
    let backend_available = agent.backend.is_available();

    let error_message = if !mdflow_available {
        Some("mdflow not found. Install with: npm install -g mdflow".to_string())
    } else if !backend_available {
        agent.backend.command().map(|cmd| {
            format!(
                "{} CLI not found. Please install {} to use this agent.",
                agent.backend.label(),
                cmd
            )
        })
    } else {
        None
    };

    AgentAvailability {
        mdflow_available,
        backend_available,
        error_message,
    }
}

fn apply_agent_environment_allowlist(
    cmd: &mut Command,
    frontmatter_env: Option<&HashMap<String, String>>,
) {
    cmd.env_clear();

    for env_key in SAFE_AGENT_ENV_VARS {
        if let Some(env_value) = std::env::var_os(env_key) {
            cmd.env(env_key, env_value);
        }
    }

    // Frontmatter _env values are applied after the scrubbed baseline, but
    // protected baseline keys remain immutable to prevent command hijacking.
    if let Some(env_vars) = frontmatter_env {
        for (env_key, env_value) in env_vars {
            if SAFE_AGENT_ENV_VARS
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(env_key))
            {
                warn!(
                    key = %env_key,
                    "agent_env_allowlist_skip_protected_key"
                );
                continue;
            }
            cmd.env(env_key, env_value);
        }
    }
}

fn has_parent_dir_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn validate_agent_markdown_path(path: &Path) -> Result<PathBuf> {
    validate_agent_markdown_path_with_kit_root(path, &get_kit_path())
}

fn validate_agent_markdown_path_with_kit_root(path: &Path, kit_root: &Path) -> Result<PathBuf> {
    if has_parent_dir_component(path) {
        warn!(
            path = %path.display(),
            "agent_path_validation_failed: reason=parent_dir_segment"
        );
        anyhow::bail!(
            "agent_path_validation_failed: path={} reason=parent_dir_segment",
            path.display()
        );
    }

    let canonical_path = std::fs::canonicalize(path).with_context(|| {
        format!(
            "agent_path_validation_failed: path={} reason=canonicalize_error",
            path.display()
        )
    })?;

    let metadata = std::fs::metadata(&canonical_path).with_context(|| {
        format!(
            "agent_path_validation_failed: path={} reason=metadata_error",
            canonical_path.display()
        )
    })?;
    if !metadata.is_file() {
        anyhow::bail!(
            "agent_path_validation_failed: path={} reason=not_file",
            canonical_path.display()
        );
    }

    let is_markdown_file = canonical_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
    if !is_markdown_file {
        anyhow::bail!(
            "agent_path_validation_failed: path={} reason=invalid_extension",
            canonical_path.display()
        );
    }

    let canonical_kit_root = std::fs::canonicalize(kit_root).with_context(|| {
        format!(
            "agent_path_validation_failed: kit_root={} reason=canonicalize_kit_root_error",
            kit_root.display()
        )
    })?;
    let kit_agents_root = canonical_kit_root.join("kit");

    if !canonical_path.starts_with(&kit_agents_root) {
        anyhow::bail!(
            "agent_path_validation_failed: path={} reason=outside_kit_root",
            canonical_path.display()
        );
    }

    let relative_path = canonical_path
        .strip_prefix(&kit_agents_root)
        .with_context(|| {
            format!(
                "agent_path_validation_failed: path={} reason=strip_prefix_error",
                canonical_path.display()
            )
        })?;

    let mut components = relative_path.components();
    let kit_name_component = components.next();
    let agents_dir_component = components.next();

    if !matches!(kit_name_component, Some(Component::Normal(_)))
        || !matches!(
            agents_dir_component,
            Some(Component::Normal(dir)) if dir == "agents"
        )
        || components.next().is_none()
    {
        anyhow::bail!(
            "agent_path_validation_failed: path={} reason=outside_agents_dir",
            canonical_path.display()
        );
    }

    Ok(canonical_path)
}

fn normalize_agent_variable_key(raw_key: &str) -> Result<String> {
    let key = raw_key.trim();
    if key.is_empty() {
        anyhow::bail!("agent_arg_validation_failed: reason=empty_variable_key");
    }

    let key_without_dashes = key.trim_start_matches('-');
    let key_without_underscores = key_without_dashes.trim_start_matches('_');
    if key_without_underscores.is_empty() {
        anyhow::bail!(
            "agent_arg_validation_failed: key={} reason=empty_variable_key",
            raw_key
        );
    }

    let normalized = key_without_underscores.to_ascii_lowercase();
    if RESERVED_MDFLOW_VARIABLE_KEYS.contains(&normalized.as_str()) {
        warn!(
            key = %raw_key,
            "agent_arg_validation_failed: reason=reserved_variable_key"
        );
        anyhow::bail!(
            "agent_arg_validation_failed: key={} reason=reserved_variable_key",
            raw_key
        );
    }

    if key_without_underscores
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        anyhow::bail!(
            "agent_arg_validation_failed: key={} reason=invalid_variable_key",
            raw_key
        );
    }

    Ok(format!("_{}", key_without_underscores))
}

fn validate_agent_variable_value(raw_key: &str, value: &str) -> Result<()> {
    if value
        .chars()
        .any(|character| character == '\n' || character == '\r' || character.is_control())
    {
        warn!(
            key = %raw_key,
            "agent_arg_validation_failed: reason=invalid_variable_value"
        );
        anyhow::bail!(
            "agent_arg_validation_failed: key={} reason=invalid_variable_value",
            raw_key
        );
    }

    Ok(())
}

fn apply_agent_variable_overrides(
    cmd: &mut Command,
    variables: &HashMap<String, String>,
) -> Result<()> {
    for (key, value) in variables {
        let normalized_key = normalize_agent_variable_key(key)?;
        validate_agent_variable_value(key, value)?;
        cmd.arg(format!("--{}", normalized_key));
        cmd.arg(value);
    }

    Ok(())
}

fn apply_positional_args(cmd: &mut Command, positional_args: &[String]) {
    if positional_args.is_empty() {
        return;
    }

    // Ensure positional arguments are not parsed as mdflow options.
    cmd.arg("--");
    for arg in positional_args {
        cmd.arg(arg);
    }
}

/// Execute an agent
///
/// # Arguments
///
/// * `agent` - The agent to execute
/// * `mode` - Execution mode (UI capture, interactive, dry run, explain)
/// * `variables` - Runtime variable overrides (passed as `--_varname value`)
/// * `positional_args` - Positional arguments appended after the file path
/// * `stdin_input` - Optional input to pipe to stdin (for `{{ _stdin }}`)
///
/// # Returns
///
/// A spawned child process. The caller is responsible for:
/// - Reading stdout/stderr
/// - Waiting for completion
/// - Handling errors
pub fn execute_agent(
    agent: &Agent,
    mode: AgentExecutionMode,
    variables: &HashMap<String, String>,
    positional_args: &[String],
    stdin_input: Option<&str>,
) -> Result<Child> {
    let mdflow_cmd =
        get_mdflow_command().context("mdflow not found. Install with: npm install -g mdflow")?;
    let canonical_agent_path = validate_agent_markdown_path(&agent.path).with_context(|| {
        format!(
            "agent_execution_validation_failed: agent={} path={}",
            agent.name,
            agent.path.display()
        )
    })?;
    debug!(
        agent = %agent.name,
        mode = ?mode,
        path = %canonical_agent_path.display(),
        "agent_execute_start"
    );

    let mut cmd = Command::new(mdflow_cmd);

    // Add the agent file path
    cmd.arg(&canonical_agent_path);

    // Add mode-specific flags
    match mode {
        AgentExecutionMode::UiCapture => {
            // Suppress dashboard, clean output for embedding in UI
            cmd.arg("--_quiet");
            cmd.arg("--raw");
        }
        AgentExecutionMode::Interactive => {
            // No special flags - mdflow handles interactive mode
        }
        AgentExecutionMode::DryRun => {
            cmd.arg("--_dry-run");
        }
        AgentExecutionMode::Explain => {
            // Use "md explain" subcommand instead
            // Note: This requires special handling - we actually run "md explain <file>"
            // For now, we use --_context which shows context without running
            cmd.arg("--_context");
        }
    }

    // Add runtime variable overrides
    apply_agent_variable_overrides(&mut cmd, variables)?;

    // Add positional arguments
    apply_positional_args(&mut cmd, positional_args);

    // Set working directory to agent file's parent
    // This is important for @./relative imports to work correctly
    if let Some(parent) = canonical_agent_path.parent() {
        cmd.current_dir(parent);
    }

    // Set up I/O
    cmd.stdin(if stdin_input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    apply_agent_environment_allowlist(&mut cmd, agent.frontmatter.env.as_ref());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn mdflow for agent: {}", agent.path.display()))?;

    // Write stdin if provided
    if let Some(input) = stdin_input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .with_context(|| format!("Failed to write stdin for agent: {}", agent.name))?;
            // Drop stdin to close it
        }
    }

    Ok(child)
}

/// Run `md explain` to get context preview for an agent
///
/// This is useful for showing what the agent will send to the AI
/// before actually running it.
pub fn explain_agent(agent: &Agent) -> Result<String> {
    let mdflow_cmd =
        get_mdflow_command().context("mdflow not found. Install with: npm install -g mdflow")?;
    let canonical_agent_path = validate_agent_markdown_path(&agent.path).with_context(|| {
        format!(
            "agent_explain_validation_failed: agent={} path={}",
            agent.name,
            agent.path.display()
        )
    })?;
    debug!(
        agent = %agent.name,
        path = %canonical_agent_path.display(),
        "agent_explain_start"
    );

    let mut cmd = Command::new(mdflow_cmd);
    cmd.arg("explain");
    cmd.arg(&canonical_agent_path);

    // Set working directory
    if let Some(parent) = canonical_agent_path.parent() {
        cmd.current_dir(parent);
    }
    apply_agent_environment_allowlist(&mut cmd, agent.frontmatter.env.as_ref());

    let output = cmd
        .output()
        .with_context(|| format!("Failed to run md explain for agent: {}", agent.name))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("md explain failed: {}", stderr)
    }
}

/// Run `md --_dry-run` to see what would be executed
///
/// Shows the full command that would be run without actually running it.
pub fn dry_run_agent(agent: &Agent) -> Result<String> {
    let mdflow_cmd =
        get_mdflow_command().context("mdflow not found. Install with: npm install -g mdflow")?;
    let canonical_agent_path = validate_agent_markdown_path(&agent.path).with_context(|| {
        format!(
            "agent_dry_run_validation_failed: agent={} path={}",
            agent.name,
            agent.path.display()
        )
    })?;
    debug!(
        agent = %agent.name,
        path = %canonical_agent_path.display(),
        "agent_dry_run_start"
    );

    let mut cmd = Command::new(mdflow_cmd);
    cmd.arg(&canonical_agent_path);
    cmd.arg("--_dry-run");

    // Set working directory
    if let Some(parent) = canonical_agent_path.parent() {
        cmd.current_dir(parent);
    }
    apply_agent_environment_allowlist(&mut cmd, agent.frontmatter.env.as_ref());

    let output = cmd
        .output()
        .with_context(|| format!("Failed to run dry-run for agent: {}", agent.name))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("dry-run failed: {}", stderr)
    }
}

/// Build command args for external terminal execution
///
/// For interactive agents, we may want to open a system terminal.
/// This returns the command and arguments to run.
pub fn build_terminal_command(agent: &Agent) -> Result<(String, Vec<String>)> {
    let canonical_agent_path = validate_agent_markdown_path(&agent.path).with_context(|| {
        format!(
            "agent_terminal_command_validation_failed: agent={} path={}",
            agent.name,
            agent.path.display()
        )
    })?;
    let mdflow_cmd =
        get_mdflow_command().context("mdflow command not found in PATH or kit node_modules")?;
    let args = vec![canonical_agent_path.to_string_lossy().to_string()];

    // For interactive terminal, don't add --_quiet or --raw
    // Let mdflow show its full terminal UX

    Ok((mdflow_cmd.to_string(), args))
}

/// Get install instructions for missing dependencies
pub fn get_install_instructions(availability: &AgentAvailability) -> String {
    if !availability.mdflow_available {
        return "mdflow is not installed.\n\n\
            Install with npm:\n  npm install -g mdflow\n\n\
            Or with bun:\n  bun install -g mdflow\n\n\
            For more info: https://github.com/johnlindquist/mdflow"
            .to_string();
    }

    if !availability.backend_available {
        if let Some(ref error) = availability.error_message {
            return error.clone();
        }
    }

    "Unknown installation issue".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::types::{AgentBackend, AgentFrontmatter};
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn with_temp_kit_agents_dir(test_fn: impl FnOnce(&std::path::Path, &std::path::Path)) {
        static SK_PATH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        struct SkPathGuard {
            previous: Option<OsString>,
        }

        impl Drop for SkPathGuard {
            fn drop(&mut self) {
                if let Some(previous) = &self.previous {
                    std::env::set_var(crate::setup::SK_PATH_ENV, previous);
                } else {
                    std::env::remove_var(crate::setup::SK_PATH_ENV);
                }
            }
        }

        let _lock = SK_PATH_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("SK_PATH lock poisoned");

        let temp_dir = TempDir::new().expect("create temp dir");
        let kit_root = temp_dir.path().join("scriptkit");
        let agents_dir = kit_root.join("kit/main/agents");
        fs::create_dir_all(&agents_dir).expect("create agents directory");

        let previous = std::env::var_os(crate::setup::SK_PATH_ENV);
        std::env::set_var(crate::setup::SK_PATH_ENV, &kit_root);
        let _guard = SkPathGuard { previous };

        test_fn(&kit_root, &agents_dir);
    }

    fn create_test_agent(backend: AgentBackend) -> Agent {
        Agent {
            name: "Test Agent".to_string(),
            path: PathBuf::from("/tmp/test.claude.md"),
            backend,
            interactive: false,
            description: Some("Test description".to_string()),
            icon: None,
            shortcut: None,
            alias: None,
            frontmatter: AgentFrontmatter::default(),
            kit: Some("main".to_string()),
            has_shell_inlines: false,
            has_remote_imports: false,
        }
    }

    #[test]
    fn test_check_availability_mdflow_missing() {
        // This test will pass if mdflow is not installed
        // or behave correctly if it is installed
        let agent = create_test_agent(AgentBackend::Claude);
        let availability = check_availability(&agent);

        // We can't guarantee mdflow is installed, so just check the struct is valid
        assert!(availability.mdflow_available || availability.error_message.is_some());
    }

    #[test]
    fn test_availability_struct() {
        let avail = AgentAvailability {
            mdflow_available: true,
            backend_available: true,
            error_message: None,
        };
        assert!(avail.is_available());

        let avail2 = AgentAvailability {
            mdflow_available: false,
            backend_available: true,
            error_message: Some("mdflow not found".to_string()),
        };
        assert!(!avail2.is_available());
    }

    #[test]
    fn test_get_install_instructions_mdflow() {
        let avail = AgentAvailability {
            mdflow_available: false,
            backend_available: true,
            error_message: Some("mdflow not found".to_string()),
        };

        let instructions = get_install_instructions(&avail);
        assert!(instructions.contains("npm install -g mdflow"));
        assert!(instructions.contains("bun install -g mdflow"));
    }

    #[test]
    fn test_get_install_instructions_backend() {
        let avail = AgentAvailability {
            mdflow_available: true,
            backend_available: false,
            error_message: Some("claude CLI not found".to_string()),
        };

        let instructions = get_install_instructions(&avail);
        assert!(instructions.contains("claude CLI not found"));
    }

    #[test]
    fn test_build_terminal_command() {
        with_temp_kit_agents_dir(|_kit_root, agents_dir| {
            let agent_file = agents_dir.join("test.claude.md");
            fs::write(&agent_file, "# test").expect("write agent file");

            let mut agent = create_test_agent(AgentBackend::Claude);
            agent.path = agent_file.clone();

            let (cmd, args) = build_terminal_command(&agent)
                .expect("valid agent path should produce terminal command");

            assert!(cmd == "mdflow" || cmd == "md");
            assert_eq!(args.len(), 1);

            let expected = fs::canonicalize(&agent_file).expect("canonicalize expected path");
            assert_eq!(PathBuf::from(&args[0]), expected);
        });
    }

    #[test]
    fn test_build_terminal_command_rejects_path_with_parent_segments() {
        with_temp_kit_agents_dir(|kit_root, _agents_dir| {
            let kit_main = kit_root.join("kit/main");
            fs::write(kit_main.join("escape.claude.md"), "# test")
                .expect("write outside agent file");

            let mut agent = create_test_agent(AgentBackend::Claude);
            agent.path = kit_main.join("agents/../escape.claude.md");

            let error = build_terminal_command(&agent)
                .expect_err("paths with parent dir segments should be rejected");
            assert!(
                error.to_string().contains("reason=parent_dir_segment"),
                "error should include parent_dir_segment reason"
            );
        });
    }

    #[test]
    fn test_apply_agent_environment_allowlist_includes_frontmatter_env() {
        let mut cmd = Command::new("echo");
        cmd.env("AGENT_SHOULD_BE_REMOVED", "1");

        let mut frontmatter_env = HashMap::new();
        frontmatter_env.insert("AGENT_FRONTMATTER_TOKEN".to_string(), "abc123".to_string());

        apply_agent_environment_allowlist(&mut cmd, Some(&frontmatter_env));

        let envs: Vec<(String, Option<std::ffi::OsString>)> = cmd
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().to_string(),
                    value.map(|v| v.to_os_string()),
                )
            })
            .collect();

        assert!(
            envs.iter().all(|(key, value)| {
                value.is_none()
                    || SAFE_AGENT_ENV_VARS
                        .iter()
                        .any(|allowed| allowed.eq_ignore_ascii_case(key))
                    || key == "AGENT_FRONTMATTER_TOKEN"
            }),
            "command environment should only contain allowlisted keys plus frontmatter _env"
        );

        assert!(
            !envs
                .iter()
                .any(|(key, value)| value.is_some() && key == "AGENT_SHOULD_BE_REMOVED"),
            "non-allowlisted variables should be removed by env_clear()"
        );

        assert!(
            envs.iter().any(|(key, value)| {
                key == "AGENT_FRONTMATTER_TOKEN"
                    && value
                        .as_ref()
                        .is_some_and(|val| val == &std::ffi::OsString::from("abc123"))
            }),
            "frontmatter _env should be applied after the allowlist scrub"
        );
    }

    #[test]
    fn test_apply_agent_environment_allowlist_rejects_frontmatter_path_override() {
        if std::env::var_os("PATH").is_none() {
            return;
        }

        let mut cmd = Command::new("echo");
        let mut frontmatter_env = HashMap::new();
        frontmatter_env.insert("PATH".to_string(), "/tmp/malicious".to_string());

        apply_agent_environment_allowlist(&mut cmd, Some(&frontmatter_env));

        let path_value = cmd
            .get_envs()
            .find_map(|(key, value)| {
                if key.eq_ignore_ascii_case("PATH") {
                    value.map(|v| v.to_os_string())
                } else {
                    None
                }
            })
            .expect("PATH should remain in allowlisted env");

        assert_ne!(
            path_value,
            std::ffi::OsString::from("/tmp/malicious"),
            "frontmatter PATH override should be ignored"
        );
    }

    #[test]
    fn test_normalize_agent_variable_key_rejects_reserved_mdflow_keys() {
        assert!(normalize_agent_variable_key("quiet").is_err());
        assert!(normalize_agent_variable_key("_context").is_err());
        assert!(normalize_agent_variable_key("--_env").is_err());
        assert!(normalize_agent_variable_key("COMMAND").is_err());
    }

    #[test]
    fn test_normalize_agent_variable_key_prefixes_underscore_for_user_vars() {
        assert_eq!(
            normalize_agent_variable_key("feature").expect("feature key should be valid"),
            "_feature"
        );
        assert_eq!(
            normalize_agent_variable_key("_feature").expect("underscored key should be valid"),
            "_feature"
        );
    }

    #[test]
    fn test_validate_agent_variable_value_rejects_control_characters() {
        assert!(validate_agent_variable_value("name", "safe-value").is_ok());
        assert!(validate_agent_variable_value("name", "line1\nline2").is_err());
        assert!(validate_agent_variable_value("name", "line1\rline2").is_err());
        assert!(validate_agent_variable_value("name", "bell\u{0007}").is_err());
    }

    #[test]
    fn test_apply_positional_args_inserts_argument_terminator() {
        let mut cmd = Command::new("mdflow");
        let args = vec!["--looks-like-flag".to_string(), "value".to_string()];
        apply_positional_args(&mut cmd, &args);

        let parsed_args: Vec<String> = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();
        assert_eq!(parsed_args, vec!["--", "--looks-like-flag", "value"]);
    }

    #[test]
    fn test_apply_agent_variable_overrides_rejects_reserved_keys() {
        let mut cmd = Command::new("mdflow");
        let mut variables = HashMap::new();
        variables.insert("quiet".to_string(), "true".to_string());

        let error = apply_agent_variable_overrides(&mut cmd, &variables)
            .expect_err("reserved mdflow key should be rejected");
        assert!(
            error.to_string().contains("reason=reserved_variable_key"),
            "error should include reserved_variable_key reason"
        );
    }

    #[test]
    fn test_apply_agent_variable_overrides_rejects_control_chars_in_values() {
        let mut cmd = Command::new("mdflow");
        let mut variables = HashMap::new();
        variables.insert("feature".to_string(), "bad\nvalue".to_string());

        let error = apply_agent_variable_overrides(&mut cmd, &variables)
            .expect_err("control characters in values should be rejected");
        assert!(
            error.to_string().contains("reason=invalid_variable_value"),
            "error should include invalid_variable_value reason"
        );
    }

    #[test]
    fn test_apply_agent_variable_overrides_formats_mdflow_args() {
        let mut cmd = Command::new("mdflow");
        let mut variables = HashMap::new();
        variables.insert("feature".to_string(), "safe".to_string());

        apply_agent_variable_overrides(&mut cmd, &variables)
            .expect("valid variable should be accepted");

        let parsed_args: Vec<String> = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();
        assert_eq!(parsed_args, vec!["--_feature", "safe"]);
    }

    #[test]
    fn test_validate_agent_markdown_path_accepts_file_inside_kit_agents_dir() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let kit_root = temp_dir.path().join("scriptkit");
        let agent_dir = kit_root.join("kit/main/agents");
        fs::create_dir_all(&agent_dir).expect("create agents directory");

        let agent_file = agent_dir.join("review.claude.md");
        fs::write(&agent_file, "# test").expect("write agent file");

        let resolved = validate_agent_markdown_path_with_kit_root(&agent_file, &kit_root)
            .expect("agent under kit agents should be accepted");
        let expected = fs::canonicalize(&agent_file).expect("canonicalize expected path");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_validate_agent_markdown_path_rejects_path_with_parent_segments() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let kit_root = temp_dir.path().join("scriptkit");
        let kit_main = kit_root.join("kit/main");
        fs::create_dir_all(kit_main.join("agents")).expect("create agents directory");
        fs::write(kit_main.join("escape.claude.md"), "# test").expect("write outside agent file");

        let path_with_parent = kit_main.join("agents/../escape.claude.md");
        let error = validate_agent_markdown_path_with_kit_root(&path_with_parent, &kit_root)
            .expect_err("paths with parent dir segments should be rejected");
        assert!(
            error.to_string().contains("reason=parent_dir_segment"),
            "error should include parent_dir_segment reason"
        );
    }

    #[test]
    fn test_validate_agent_markdown_path_rejects_file_outside_agents_directory() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let kit_root = temp_dir.path().join("scriptkit");
        let script_dir = kit_root.join("kit/main/scripts");
        fs::create_dir_all(&script_dir).expect("create scripts directory");

        let script_file = script_dir.join("not-agent.md");
        fs::write(&script_file, "# test").expect("write script file");

        let error = validate_agent_markdown_path_with_kit_root(&script_file, &kit_root)
            .expect_err("file outside agents dir should be rejected");
        assert!(
            error.to_string().contains("reason=outside_agents_dir"),
            "error should include outside_agents_dir reason"
        );
    }

    #[test]
    fn test_validate_agent_markdown_path_rejects_non_markdown_extension() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let kit_root = temp_dir.path().join("scriptkit");
        let agent_dir = kit_root.join("kit/main/agents");
        fs::create_dir_all(&agent_dir).expect("create agents directory");

        let binary_file = agent_dir.join("review.claude.txt");
        fs::write(&binary_file, "test").expect("write non-markdown file");

        let error = validate_agent_markdown_path_with_kit_root(&binary_file, &kit_root)
            .expect_err("non-markdown file should be rejected");
        assert!(
            error.to_string().contains("reason=invalid_extension"),
            "error should include invalid_extension reason"
        );
    }

    // Note: We can't easily test execute_agent without mdflow installed
    // and an actual agent file. These would be integration tests.

    #[test]
    fn test_execution_mode_default() {
        assert_eq!(AgentExecutionMode::default(), AgentExecutionMode::UiCapture);
    }
}
