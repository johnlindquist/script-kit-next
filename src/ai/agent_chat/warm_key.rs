//! A process-local cache identity, not a credential. Bind the *effective* MCP policy
//! as well as the recipe: Script Kit's tool list is not in the recipe until spawn.
use crate::{ai::agent_chat::goose::GooseLaunchSpec, mcp_agent_tools::AgentToolPolicy};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::LazyLock};
use uuid::Uuid;

// External MCP definitions may contain headers and environment values. Neither they nor a
// stable/dictionary-testable fingerprint of them should appear in logs or survive a process.
static PROCESS_SALT: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
fn paths(roots: &Option<Vec<PathBuf>>) -> Option<Vec<&[u8]>> {
    roots
        .as_ref()
        .map(|r| r.iter().map(|p| p.as_os_str().as_encoded_bytes()).collect())
}
pub(crate) fn launch_warm_key(
    spec: &GooseLaunchSpec,
    profile: &str,
    policy: &AgentToolPolicy,
    endpoint: Option<Uuid>,
) -> String {
    let shape = json!({
        "binary": spec.command.as_os_str().as_encoded_bytes(),
        "args": spec.args,
        "cwd": spec.cwd.as_os_str().as_encoded_bytes(),
        "recipe": spec.recipe,
        "thinking": spec.thinking,
        "hidden": spec.hidden_session,
        "gooseRoot": spec.env.get("GOOSE_PATH_ROOT"),
        "contextFiles": spec.env.get("CONTEXT_FILE_NAMES"),
        "launcherAuthor": spec.launcher_author.map(|access| access.profile_tool()),
        "approval": spec.approval_mode.id(),
        "profile": profile,
        "endpoint": endpoint,
        "tools": policy.allowed_tools,
        "readRoots": paths(&policy.read_roots),
        "writeRoots": paths(&policy.write_roots),
        "webBudget": policy.web_search_budget,
    });
    let mut hash = Sha256::new();
    hash.update(PROCESS_SALT.as_bytes());
    hash.update(shape.to_string().as_bytes());
    format!("goose-warm-v2:{:x}", hash.finalize())
}
