//! GitHub sync worker for the Script Kit workspace.

use anyhow::{bail, Context, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub const SCRIPT_KIT_SYNC_DRY_RUN_ENV: &str = "SCRIPT_KIT_SYNC_DRY_RUN";

const DEFAULT_REPO_NAME: &str = "scriptkit-sync";
const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(60);
const PUSH_COMMAND_TIMEOUT: Duration = Duration::from_secs(120);
const GITIGNORE_MARKER: &str = "# Script Kit Sync sensitive exclusions";

#[derive(Debug, Clone)]
pub struct SyncToGithubReport {
    pub workspace: PathBuf,
    pub dry_run: bool,
    pub steps: Vec<SyncStepReport>,
}

impl SyncToGithubReport {
    pub fn summary_message(&self) -> String {
        let skipped = self.steps.iter().filter(|step| step.skipped).count();
        if self.dry_run {
            format!("GitHub sync dry-run complete ({skipped} remote step(s) skipped)")
        } else {
            "Synced Script Kit to GitHub".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncStepReport {
    pub label: String,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub skipped: bool,
}

#[derive(Debug, Clone)]
struct ShellCommand {
    label: String,
    program: String,
    args: Vec<String>,
    timeout: Duration,
    skip_when_dry_run: bool,
}

impl ShellCommand {
    fn new(
        label: impl Into<String>,
        program: impl Into<String>,
        args: Vec<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            label: label.into(),
            program: program.into(),
            args,
            timeout,
            skip_when_dry_run: false,
        }
    }

    fn shell(label: impl Into<String>, script: String, timeout: Duration) -> Self {
        Self::new(label, "/bin/sh", vec!["-c".to_string(), script], timeout)
    }

    fn skip_when_dry_run(mut self) -> Self {
        self.skip_when_dry_run = true;
        self
    }

    fn display_command(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(quote_for_log(&self.program));
        for arg in &self.args {
            parts.push(quote_for_log(arg));
        }
        parts.join(" ")
    }
}

pub fn sync_to_github_workspace() -> Result<SyncToGithubReport> {
    sync_to_github_at_path(crate::setup::get_kit_path())
}

pub fn sync_to_github_at_path(workspace: PathBuf) -> Result<SyncToGithubReport> {
    if !workspace.exists() {
        bail!(
            "Script Kit workspace does not exist: {}",
            workspace.display()
        );
    }
    if !workspace.is_dir() {
        bail!(
            "Script Kit workspace is not a directory: {}",
            workspace.display()
        );
    }

    let dry_run = sync_dry_run_enabled();
    let commands = build_sync_commands(DEFAULT_REPO_NAME.to_string());
    let mut steps = Vec::with_capacity(commands.len());

    tracing::info!(
        target: "script_kit::sync",
        workspace = %workspace.display(),
        dry_run,
        command_count = commands.len(),
        "sync.github.start"
    );

    for command in commands {
        let step = run_shell_command(&workspace, &command, dry_run)?;
        steps.push(step);
    }

    tracing::info!(
        target: "script_kit::sync",
        workspace = %workspace.display(),
        dry_run,
        step_count = steps.len(),
        "sync.github.finished"
    );

    Ok(SyncToGithubReport {
        workspace,
        dry_run,
        steps,
    })
}

pub fn sensitive_exclusion_patterns() -> Vec<String> {
    [
        "agent-token",
        "server.json",
        ".env",
        ".env.*",
        "secrets/",
        ".pem",
        ".key",
        ".p12",
        ".pfx",
        "agent_chat/auth/",
        "logs/",
        "*.log",
        ".DS_Store",
        "node_modules/",
        ".cache/",
        "tmp/",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn sync_dry_run_enabled() -> bool {
    std::env::var(SCRIPT_KIT_SYNC_DRY_RUN_ENV)
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn build_sync_commands(repo_name: String) -> Vec<ShellCommand> {
    vec![
        gitignore_write_command(),
        ShellCommand::new(
            "git init",
            "git",
            vec!["init".to_string()],
            DEFAULT_COMMAND_TIMEOUT,
        ),
        ShellCommand::new(
            "git add",
            "git",
            vec!["add".to_string(), ".".to_string()],
            DEFAULT_COMMAND_TIMEOUT,
        ),
        ShellCommand::shell("git commit", git_commit_script(), DEFAULT_COMMAND_TIMEOUT),
        ShellCommand::shell(
            "gh repo create",
            gh_repo_create_script(repo_name),
            DEFAULT_COMMAND_TIMEOUT,
        )
        .skip_when_dry_run(),
        ShellCommand::new(
            "git push",
            "git",
            vec![
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                "HEAD".to_string(),
            ],
            PUSH_COMMAND_TIMEOUT,
        )
        .skip_when_dry_run(),
    ]
}

fn gitignore_write_command() -> ShellCommand {
    let marker = quote_for_shell(GITIGNORE_MARKER);
    let block = gitignore_exclusion_block();
    let script = format!(
        "set -eu\n\
         touch .gitignore\n\
         if ! grep -Fq {marker} .gitignore; then\n\
         cat >> .gitignore <<'SCRIPT_KIT_SYNC_GITIGNORE'\n\
         {block}SCRIPT_KIT_SYNC_GITIGNORE\n\
         fi\n"
    );

    ShellCommand::shell("gitignore write", script, DEFAULT_COMMAND_TIMEOUT)
}

fn gitignore_exclusion_block() -> String {
    let mut block = String::new();
    block.push('\n');
    block.push_str(GITIGNORE_MARKER);
    block.push('\n');
    for pattern in sensitive_exclusion_patterns() {
        block.push_str(&pattern);
        block.push('\n');
    }
    block
}

fn git_commit_script() -> String {
    "set -eu\n\
     if git diff --cached --quiet; then\n\
     echo 'No Script Kit changes to commit'\n\
     else\n\
     git commit -m 'Sync Script Kit'\n\
     fi\n"
        .to_string()
}

fn gh_repo_create_script(repo_name: String) -> String {
    let repo_name = quote_for_shell(&repo_name);
    format!(
        "set -eu\n\
         if git remote get-url origin >/dev/null 2>&1; then\n\
         echo 'origin remote already configured'\n\
         else\n\
         gh repo create {repo_name} --private --source . --remote origin\n\
         fi\n"
    )
}

fn run_shell_command(
    workspace: &Path,
    command: &ShellCommand,
    dry_run: bool,
) -> Result<SyncStepReport> {
    let command_text = command.display_command();
    tracing::info!(
        target: "script_kit::sync",
        label = %command.label,
        command = %command_text,
        timeout_ms = command.timeout.as_millis() as u64,
        dry_run,
        skip_when_dry_run = command.skip_when_dry_run,
        "sync.github.command"
    );

    if dry_run && command.skip_when_dry_run {
        tracing::info!(
            target: "script_kit::sync",
            label = %command.label,
            command = %command_text,
            "sync.github.command_skipped_dry_run"
        );
        return Ok(SyncStepReport {
            label: command.label.clone(),
            command: command_text,
            stdout: String::new(),
            stderr: String::new(),
            skipped: true,
        });
    }

    let output = run_command_with_timeout(workspace, command, &command_text)?;
    if !output.status_success {
        bail!(
            "Command failed: {}\nstdout:\n{}\nstderr:\n{}",
            command_text,
            output.stdout,
            output.stderr
        );
    }

    Ok(SyncStepReport {
        label: command.label.clone(),
        command: command_text,
        stdout: output.stdout,
        stderr: output.stderr,
        skipped: false,
    })
}

struct TimedCommandOutput {
    status_success: bool,
    stdout: String,
    stderr: String,
}

fn run_command_with_timeout(
    workspace: &Path,
    command: &ShellCommand,
    command_text: &str,
) -> Result<TimedCommandOutput> {
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .current_dir(workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn command: {command_text}"))?;

    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .with_context(|| format!("Failed to poll command: {command_text}"))?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .with_context(|| format!("Failed to collect command output: {command_text}"))?;
            return Ok(TimedCommandOutput {
                status_success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        if started.elapsed() >= command.timeout {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .with_context(|| format!("Failed to collect timed-out command: {command_text}"))?;
            bail!(
                "Command timed out after {} ms: {}\nstdout:\n{}\nstderr:\n{}",
                command.timeout.as_millis(),
                command_text,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

fn quote_for_log(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | '='))
    {
        value.to_string()
    } else {
        quote_for_shell(value)
    }
}

fn quote_for_shell(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}
