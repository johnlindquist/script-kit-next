//! Script Kit tools for AI agent sessions, served over MCP streamable HTTP.
//!
//! goose sessions reach these tools through the app's MCP server at
//! `POST /mcp/agent`. Each session registers an [`AgentToolPolicy`] and sends
//! its id in the `X-Script-Kit-Agent-Policy` header; the policy decides which
//! tools the session sees, which paths they may touch, and how many web
//! searches one turn may run. Requests without a known policy see no tools.

mod files;
mod web;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use parking_lot::{Mutex, RwLock};
use serde_json::{json, Value};

use crate::ai::agent_chat::profiles::{ResolvedAgentChatProfile, BUILTIN_QUICK_AI_PROFILE_ID};

/// HTTP path of the agent-tools MCP endpoint on the app's MCP server.
pub(crate) const AGENT_TOOLS_PATH: &str = "/mcp/agent";
/// Lower-cased header carrying the session's policy id.
pub(crate) const POLICY_HEADER: &str = "x-script-kit-agent-policy";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentToolsEndpoint {
    pub uri: String,
    pub token: String,
}

static ENDPOINT: LazyLock<RwLock<Option<AgentToolsEndpoint>>> = LazyLock::new(|| RwLock::new(None));

/// Record where the running MCP server accepts agent-tool requests.
pub(crate) fn set_endpoint(port: u16, token: &str) {
    *ENDPOINT.write() = Some(AgentToolsEndpoint {
        uri: format!("http://127.0.0.1:{port}{AGENT_TOOLS_PATH}"),
        token: token.to_string(),
    });
}

/// The agent-tools endpoint, when the MCP server is running.
pub(crate) fn endpoint() -> Option<AgentToolsEndpoint> {
    ENDPOINT.read().clone()
}

/// What one agent session may do with Script Kit tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentToolPolicy {
    pub allowed_tools: Vec<String>,
    pub cwd: PathBuf,
    /// `None` = unrestricted; `Some([])` = no filesystem reads.
    pub read_roots: Option<Vec<PathBuf>>,
    /// `None` = unrestricted; `Some([])` = no filesystem writes.
    pub write_roots: Option<Vec<PathBuf>>,
    /// Web searches allowed per turn; `None` = unlimited.
    pub web_search_budget: Option<u8>,
}

impl AgentToolPolicy {
    pub(crate) fn from_profile(
        profile: &ResolvedAgentChatProfile,
        tools: &[&str],
        cwd: &Path,
    ) -> Self {
        let roots = |paths: Option<&Vec<String>>| {
            paths.map(|paths| {
                paths
                    .iter()
                    .map(|path| crate::ai::agent_chat::goose::binary::expand_tilde_path(path))
                    .collect::<Vec<_>>()
            })
        };
        let path_policy = profile.path_policy.as_ref();
        Self {
            allowed_tools: tools.iter().map(|tool| tool.to_string()).collect(),
            cwd: cwd.to_path_buf(),
            read_roots: roots(path_policy.and_then(|policy| policy.allow_read.as_ref())),
            write_roots: roots(path_policy.and_then(|policy| policy.allow_write.as_ref())),
            web_search_budget: (profile.id == BUILTIN_QUICK_AI_PROFILE_ID)
                .then_some(crate::ai::agent_chat::profiles::QUICK_AI_FOCUSED_SEARCH_BUDGET),
        }
    }

    pub(crate) fn allows(&self, tool: &str) -> bool {
        self.allowed_tools.iter().any(|allowed| allowed == tool)
    }
}

#[derive(Debug)]
struct PolicyState {
    policy: AgentToolPolicy,
    searches_this_turn: u8,
}

static REGISTRY: LazyLock<Mutex<HashMap<String, PolicyState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn register_policy(policy: AgentToolPolicy) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    REGISTRY.lock().insert(
        id.clone(),
        PolicyState {
            policy,
            searches_this_turn: 0,
        },
    );
    id
}

pub(crate) fn unregister_policy(id: &str) {
    REGISTRY.lock().remove(id);
}

/// A new user turn starts: per-turn budgets reset.
pub(crate) fn begin_turn(id: &str) {
    if let Some(state) = REGISTRY.lock().get_mut(id) {
        state.searches_this_turn = 0;
    }
}

fn policy_for(id: Option<&str>) -> Option<AgentToolPolicy> {
    REGISTRY.lock().get(id?).map(|state| state.policy.clone())
}

/// Claim one web search against the turn budget. `false` = budget spent.
fn claim_search(id: Option<&str>) -> bool {
    let Some(id) = id else {
        return false;
    };
    let mut registry = REGISTRY.lock();
    let Some(state) = registry.get_mut(id) else {
        return false;
    };
    match state.policy.web_search_budget {
        Some(budget) if state.searches_this_turn >= budget => false,
        _ => {
            state.searches_this_turn = state.searches_this_turn.saturating_add(1);
            true
        }
    }
}

/// Reply to one HTTP POST on the agent-tools endpoint.
#[derive(Debug, PartialEq)]
pub(crate) enum AgentRpcReply {
    /// JSON-RPC response body (HTTP 200).
    Json(String),
    /// Notification acknowledged (HTTP 202, empty body).
    Accepted,
}

pub(crate) fn handle_rpc(policy_id: Option<&str>, body: &str) -> AgentRpcReply {
    let message: Value = match serde_json::from_str(body) {
        Ok(message) => message,
        Err(error) => {
            return AgentRpcReply::Json(
                json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {error}") },
                })
                .to_string(),
            );
        }
    };
    let Some(id) = message.get("id").filter(|id| !id.is_null()).cloned() else {
        // Notifications (`notifications/initialized`, cancellations) need no answer.
        return AgentRpcReply::Accepted;
    };
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let outcome = match method {
        "initialize" => Ok(json!({
            "protocolVersion": params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(MCP_PROTOCOL_VERSION),
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "script-kit", "version": env!("CARGO_PKG_VERSION") },
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_list(policy_for(policy_id).as_ref()) })),
        "tools/call" => Ok(call_tool(policy_id, &params)),
        _ => Err((-32601, format!("Method not found: {method}"))),
    };
    let reply = match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message },
        }),
    };
    AgentRpcReply::Json(reply.to_string())
}

struct ToolDef {
    name: &'static str,
    description: &'static str,
    schema: fn() -> Value,
}

const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "web_search",
        description: "Search the web. Returns titles, URLs, and snippets for the top results. Use for live or time-sensitive facts.",
        schema: || json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "One focused search query" },
                "max_results": { "type": "integer", "minimum": 1, "maximum": 10 }
            },
            "required": ["query"]
        }),
    },
    ToolDef {
        name: "web_fetch",
        description: "Fetch a public web page and return its readable text.",
        schema: || json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "http(s) URL to fetch" },
                "max_chars": { "type": "integer", "minimum": 500, "maximum": 50000 }
            },
            "required": ["url"]
        }),
    },
    ToolDef {
        name: "read",
        description: "Read a text file. Returns numbered lines; use offset and limit for large files.",
        schema: || json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "minimum": 1, "description": "First line (1-based)" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 2000 }
            },
            "required": ["path"]
        }),
    },
    ToolDef {
        name: "ls",
        description: "List the entries of a directory.",
        schema: || json!({
            "type": "object",
            "properties": { "path": { "type": "string", "description": "Directory (defaults to the working directory)" } }
        }),
    },
    ToolDef {
        name: "find",
        description: "Find files whose path matches a glob pattern such as **/*.ts.",
        schema: || json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "path": { "type": "string", "description": "Directory to search (defaults to the working directory)" }
            },
            "required": ["pattern"]
        }),
    },
    ToolDef {
        name: "grep",
        description: "Search file contents with a regular expression. Returns path:line: text matches.",
        schema: || json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Regular expression" },
                "path": { "type": "string", "description": "File or directory (defaults to the working directory)" },
                "glob": { "type": "string", "description": "Only search files matching this glob" }
            },
            "required": ["pattern"]
        }),
    },
    ToolDef {
        name: "create_file",
        description: "Create a new file with the given content. Fails if the file already exists.",
        schema: || json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        }),
    },
    ToolDef {
        name: "desktop_search",
        description: "Search files on this Mac by name or content using Spotlight.",
        schema: || json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        }),
    },
];

fn tool_list(policy: Option<&AgentToolPolicy>) -> Vec<Value> {
    let Some(policy) = policy else {
        return Vec::new();
    };
    TOOLS
        .iter()
        .filter(|tool| policy.allows(tool.name))
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": (tool.schema)(),
            })
        })
        .collect()
}

fn call_tool(policy_id: Option<&str>, params: &Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let Some(policy) = policy_for(policy_id).filter(|policy| policy.allows(name)) else {
        return tool_error(format!("The {name} tool is not available in this session."));
    };
    let result = match name {
        "web_search" => {
            if !claim_search(policy_id) {
                return tool_error(
                    "The web search budget for this answer is used up. Answer from the results you already have.",
                );
            }
            web::web_search(&args)
        }
        "web_fetch" => web::web_fetch(&args),
        "read" => files::read(&policy, &args),
        "ls" => files::ls(&policy, &args),
        "find" => files::find(&policy, &args),
        "grep" => files::grep(&policy, &args),
        "create_file" => files::create_file(&policy, &args),
        "desktop_search" => files::desktop_search(&args),
        _ => Err(format!("Unknown tool: {name}")),
    };
    match result {
        Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        Err(message) => tool_error(message),
    }
}

fn tool_error(message: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": message.into() }], "isError": true })
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(tools: &[&str], budget: Option<u8>) -> AgentToolPolicy {
        AgentToolPolicy {
            allowed_tools: tools.iter().map(|tool| tool.to_string()).collect(),
            cwd: std::env::temp_dir(),
            read_roots: Some(Vec::new()),
            write_roots: Some(Vec::new()),
            web_search_budget: budget,
        }
    }

    fn rpc(policy_id: Option<&str>, body: Value) -> Value {
        match handle_rpc(policy_id, &body.to_string()) {
            AgentRpcReply::Json(text) => serde_json::from_str(&text).unwrap(),
            AgentRpcReply::Accepted => Value::Null,
        }
    }

    #[test]
    fn notifications_are_accepted_without_a_body() {
        assert_eq!(
            handle_rpc(
                None,
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
            ),
            AgentRpcReply::Accepted
        );
    }

    #[test]
    fn tools_list_shows_only_the_policy_tools() {
        let id = register_policy(policy(&["web_search"], None));
        let listed = rpc(
            Some(&id),
            json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        );
        let names: Vec<_> = listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["web_search"]);
        let unknown = rpc(
            Some("missing"),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        );
        assert_eq!(unknown["result"]["tools"], json!([]));
        unregister_policy(&id);
    }

    #[test]
    fn a_tool_outside_the_policy_is_refused() {
        let id = register_policy(policy(&["web_search"], None));
        let call = rpc(
            Some(&id),
            json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read","arguments":{"path":"/etc/hosts"}}}),
        );
        assert_eq!(call["result"]["isError"], true);
        unregister_policy(&id);
    }

    #[test]
    fn the_search_budget_resets_each_turn() {
        let id = register_policy(policy(&["web_search"], Some(1)));
        assert!(claim_search(Some(&id)));
        assert!(!claim_search(Some(&id)));
        begin_turn(&id);
        assert!(claim_search(Some(&id)));
        unregister_policy(&id);
        assert!(!claim_search(Some(&id)));
    }
}
