use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(crate) const AGENT_PROMPT_HANDOFF_ACTION_PREFIX: &str = "agent_chat:handoff:";
pub(crate) const CMUX_CODEX_ADAPTER_ID: &str = "cmux_codex";
pub(crate) const CMUX_CODEX_ACTION_ID: &str = "agent_chat:handoff:cmux_codex";

const DRY_RUN_ENV: &str = "SCRIPT_KIT_AGENT_HANDOFF_DRY_RUN";
const RECEIPT_PATH_ENV: &str = "SCRIPT_KIT_AGENT_HANDOFF_RECEIPT_PATH";
const CMUX_BINARY_ENV: &str = "SCRIPT_KIT_CMUX_BINARY";
const CODEX_BINARY_ENV: &str = "SCRIPT_KIT_CODEX_BINARY";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentPromptHandoffAdapterId {
    CmuxCodex,
}

impl AgentPromptHandoffAdapterId {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::CmuxCodex => CMUX_CODEX_ADAPTER_ID,
        }
    }

    pub(crate) fn action_id(self) -> &'static str {
        match self {
            Self::CmuxCodex => CMUX_CODEX_ACTION_ID,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentPromptHandoffSource {
    AcpComposer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentPromptHandoffPayload {
    pub(crate) source: AgentPromptHandoffSource,
    pub(crate) adapter_id: AgentPromptHandoffAdapterId,
    pub(crate) raw_input: String,
    pub(crate) prompt: String,
    pub(crate) cwd: PathBuf,
    pub(crate) model_id: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) context_part_count: usize,
    pub(crate) prompt_builder_segment_count: usize,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentPromptHandoffReceipt {
    pub(crate) adapter_id: String,
    pub(crate) action_id: String,
    pub(crate) dry_run: bool,
    pub(crate) cwd: String,
    pub(crate) prompt_chars: usize,
    pub(crate) prompt_sha256: String,
    pub(crate) command_kind: String,
    pub(crate) cmux_binary: String,
    pub(crate) codex_binary: String,
    pub(crate) prompt_file_created: bool,
    pub(crate) script_file_created: bool,
    pub(crate) spawned: bool,
    pub(crate) pid: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AgentPromptHandoffError {
    SetupMode,
    EmptyPrompt,
    UnsupportedAdapter(String),
    Io(String),
    Spawn(String),
}

impl AgentPromptHandoffError {
    pub(crate) fn user_message(&self) -> String {
        match self {
            Self::SetupMode => "Agent Chat is in setup mode".to_string(),
            Self::EmptyPrompt => "No prompt to send".to_string(),
            Self::UnsupportedAdapter(adapter) => {
                format!("Prompt handoff adapter '{adapter}' is unavailable")
            }
            Self::Io(error) => format!("Failed to prepare prompt handoff: {error}"),
            Self::Spawn(error) => format!("Failed to launch cmux Codex: {error}"),
        }
    }
}

pub(crate) fn adapter_from_action_id(action_id: &str) -> Option<AgentPromptHandoffAdapterId> {
    match action_id {
        CMUX_CODEX_ACTION_ID => Some(AgentPromptHandoffAdapterId::CmuxCodex),
        _ => None,
    }
}

pub(crate) fn launch_prompt_handoff(
    payload: &AgentPromptHandoffPayload,
) -> Result<AgentPromptHandoffReceipt, AgentPromptHandoffError> {
    if payload.prompt.trim().is_empty() {
        return Err(AgentPromptHandoffError::EmptyPrompt);
    }

    match payload.adapter_id {
        AgentPromptHandoffAdapterId::CmuxCodex => launch_cmux_codex(payload),
    }
}

fn launch_cmux_codex(
    payload: &AgentPromptHandoffPayload,
) -> Result<AgentPromptHandoffReceipt, AgentPromptHandoffError> {
    let cmux_binary = std::env::var(CMUX_BINARY_ENV).unwrap_or_else(|_| "cmux".to_string());
    let codex_binary = std::env::var(CODEX_BINARY_ENV).unwrap_or_else(|_| "codex".to_string());
    let dry_run = env_truthy(DRY_RUN_ENV);

    let mut prompt_file_created = false;
    let mut script_file_created = false;
    let mut command_string = String::new();

    if !dry_run {
        let prepared = prepare_cmux_codex_wrapper(&payload.prompt, &codex_binary)?;
        prompt_file_created = true;
        script_file_created = true;
        command_string = prepared.command_string;
    }

    let prompt_chars = payload.prompt.chars().count();
    let prompt_sha256 = sha256_hex(&payload.prompt);
    let mut receipt = AgentPromptHandoffReceipt {
        adapter_id: payload.adapter_id.id().to_string(),
        action_id: payload.adapter_id.action_id().to_string(),
        dry_run,
        cwd: payload.cwd.to_string_lossy().to_string(),
        prompt_chars,
        prompt_sha256,
        command_kind: "cmux_new_workspace_command_wrapper".to_string(),
        cmux_binary: cmux_binary.clone(),
        codex_binary,
        prompt_file_created,
        script_file_created,
        spawned: false,
        pid: None,
    };

    if dry_run {
        write_receipt_if_requested(&receipt)?;
        return Ok(receipt);
    }

    let args = build_cmux_new_workspace_args(&payload.cwd, &command_string);
    let child = std::process::Command::new(&cmux_binary)
        .args(args)
        .spawn()
        .map_err(|error| AgentPromptHandoffError::Spawn(error.to_string()))?;

    receipt.spawned = true;
    receipt.pid = Some(child.id());
    write_receipt_if_requested(&receipt)?;
    Ok(receipt)
}

struct PreparedCmuxCodexWrapper {
    command_string: String,
}

fn prepare_cmux_codex_wrapper(
    prompt: &str,
    codex_binary: &str,
) -> Result<PreparedCmuxCodexWrapper, AgentPromptHandoffError> {
    let dir = std::env::temp_dir()
        .join("script-kit-agent-handoff")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&dir)
        .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?;

    let prompt_path = dir.join("prompt.md");
    let script_path = dir.join("run.zsh");
    std::fs::write(&prompt_path, prompt)
        .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?;

    let script = format!(
        "#!/bin/zsh\nset -euo pipefail\nprompt_file=\"${{0:A:h}}/prompt.md\"\nprompt=\"$(cat \"$prompt_file\")\"\nrm -f \"$prompt_file\"\nrm -f \"$0\"\nexec {} -- \"$prompt\"\n",
        shell_quote(codex_binary)
    );
    std::fs::write(&script_path, script)
        .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&script_path)
            .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script_path, permissions)
            .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?;
    }

    Ok(PreparedCmuxCodexWrapper {
        command_string: format!("/bin/zsh {}", shell_quote_path(&script_path)),
    })
}

fn build_cmux_new_workspace_args(cwd: &Path, command_string: &str) -> Vec<String> {
    vec![
        "new-workspace".to_string(),
        "--cwd".to_string(),
        cwd.to_string_lossy().to_string(),
        "--command".to_string(),
        command_string.to_string(),
    ]
}

fn write_receipt_if_requested(
    receipt: &AgentPromptHandoffReceipt,
) -> Result<(), AgentPromptHandoffError> {
    let Ok(path) = std::env::var(RECEIPT_PATH_ENV) else {
        return Ok(());
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?;
    }
    let json = serde_json::to_string_pretty(receipt)
        .map_err(|error| AgentPromptHandoffError::Io(error.to_string()))?;
    std::fs::write(path, json).map_err(|error| AgentPromptHandoffError::Io(error.to_string()))
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn sha256_hex(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.to_string_lossy())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_from_action_id_recognizes_cmux_codex() {
        assert_eq!(
            adapter_from_action_id(CMUX_CODEX_ACTION_ID),
            Some(AgentPromptHandoffAdapterId::CmuxCodex)
        );
        assert_eq!(adapter_from_action_id("agent_chat:handoff:other"), None);
    }

    #[test]
    fn cmux_args_do_not_include_raw_prompt() {
        let prompt = "summarize this private prompt";
        let args = build_cmux_new_workspace_args(
            Path::new("/Users/example/project"),
            "/bin/zsh '/tmp/script-kit-agent-handoff/abc/run.zsh'",
        );
        let joined = args.join(" ");
        assert!(!joined.contains(prompt));
        assert!(joined.contains("new-workspace"));
        assert!(joined.contains("--command"));
    }

    #[test]
    fn receipt_does_not_serialize_raw_prompt() {
        let receipt = AgentPromptHandoffReceipt {
            adapter_id: CMUX_CODEX_ADAPTER_ID.to_string(),
            action_id: CMUX_CODEX_ACTION_ID.to_string(),
            dry_run: true,
            cwd: "/tmp".to_string(),
            prompt_chars: 26,
            prompt_sha256: sha256_hex("summarize private prompt"),
            command_kind: "cmux_new_workspace_command_wrapper".to_string(),
            cmux_binary: "cmux".to_string(),
            codex_binary: "codex".to_string(),
            prompt_file_created: false,
            script_file_created: false,
            spawned: false,
            pid: None,
        };
        let json = serde_json::to_string(&receipt).expect("serialize receipt");
        assert!(!json.contains("summarize private prompt"));
        assert!(json.contains("\"promptSha256\""));
        assert!(json.contains("\"promptChars\""));
    }
}
