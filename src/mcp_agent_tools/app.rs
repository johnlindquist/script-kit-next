//! Script Kit's own app actions and data for agent sessions.
//!
//! The same tools Script Kit's MCP server gives external agents (notes,
//! clipboard history, scripts, config, launcher state, read-only computer
//! inspection, and scripts that declare an input schema) plus its resources
//! (calendar, clipboard history, notes, Brain, dictation history …) and the
//! user's past chats. Tool names swap `/` for `_` because model APIs only
//! accept `[A-Za-z0-9_-]` names. Read-only tools are annotated so the engine
//! runs them without asking; everything else asks under an approval mode.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::mcp_kit_tools::ToolDefinition;
use crate::mcp_protocol::{JsonRpcRequest, McpRuntimeContext};

/// goose prefixes tool names with `scriptkit__`; model APIs cap names at 64.
const MAX_TOOL_NAME: usize = 64 - "scriptkit__".len();
const MAX_RESOURCE_CHARS: usize = 60_000;
const MAX_CHAT_CHARS: usize = 40_000;

/// One app tool as the agent sees it.
#[derive(Debug, Clone)]
pub(super) struct AppTool {
    /// Model-safe name (`kit_notes_create`).
    pub name: String,
    /// Name the MCP server routes on (`kit/notes_create`).
    pub route: Option<String>,
    pub description: String,
    pub schema: Value,
    pub read_only: bool,
}

fn model_safe(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Whether a routed MCP tool only looks. Computer tools inspect apps, windows
/// and menus, but those that capture the screen ask first like a change
/// would; a handful of kit tools read state or config.
fn routed_tool_is_read_only(route: &str) -> bool {
    let captures_screen = matches!(
        route,
        crate::mcp_computer_use_tools::COMPUTER_SEE_TOOL
            | crate::mcp_computer_use_tools::COMPUTER_CAPTURE_NATIVE_WINDOW_TOOL
            | crate::mcp_computer_use_tools::COMPUTER_CAPTURE_RENDER_WINDOW_TOOL
    );
    (route.starts_with("computer/") && !captures_screen)
        || matches!(
            route,
            "kit/state"
                | "kit/config_get"
                | "kit/config_list"
                | "kit/config_validate"
                | "kit/config_validate_change"
        )
}

fn scripts() -> Vec<Arc<crate::scripts::Script>> {
    crate::scripts::read_scripts().unwrap_or_default()
}

fn local_tools() -> Vec<AppTool> {
    vec![
        AppTool {
            name: "kit_resources".into(),
            route: None,
            description: "List the data Script Kit can read for you (calendar, clipboard history, notes, Brain memory, dictation history, notifications, scripts, system, git …) with the uri to pass to kit_read.".into(),
            schema: json!({ "type": "object", "properties": {} }),
            read_only: true,
        },
        AppTool {
            name: "kit_read".into(),
            route: None,
            description: "Read one Script Kit resource by uri, e.g. kit://calendar, kit://clipboard-history, kit://notes, kit://brain, kit://dictation-history, scripts://.".into(),
            schema: json!({
                "type": "object",
                "properties": { "uri": { "type": "string" } },
                "required": ["uri"]
            }),
            read_only: true,
        },
        AppTool {
            name: "chats_search".into(),
            route: None,
            description: "Search the user's past chats with you by words. Returns ids, titles and previews; read one with chat_read.".into(),
            schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50 }
                },
                "required": ["query"]
            }),
            read_only: true,
        },
        AppTool {
            name: "chat_read".into(),
            route: None,
            description: "Read the transcript of one past chat by id (from chats_search).".into(),
            schema: json!({
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }),
            read_only: true,
        },
    ]
}

/// Every app tool, local ones first.
pub(super) fn tools() -> Vec<AppTool> {
    let mut definitions: Vec<ToolDefinition> =
        crate::mcp_control::build_default_mutation_registry().definitions();
    definitions.extend(crate::mcp_kit_tools::get_kit_tool_definitions());
    definitions.extend(crate::mcp_computer_use_tools::get_computer_use_tool_definitions());
    // A script schema alone is not an executable grant. Reviewed external commands
    // have separate per-principal, digest-bound discovery.
    let mut tools = local_tools();
    for definition in definitions {
        let name = model_safe(&definition.name);
        if name.len() > MAX_TOOL_NAME || tools.iter().any(|tool| tool.name == name) {
            continue;
        }
        tools.push(AppTool {
            read_only: routed_tool_is_read_only(&definition.name),
            name,
            route: Some(definition.name),
            description: definition.description,
            schema: definition.input_schema,
        });
    }
    tools
}

pub(super) fn find(name: &str) -> Option<AppTool> {
    tools().into_iter().find(|tool| tool.name == name)
}

/// Run one app tool. `Ok` is the MCP `tools/call` result.
pub(super) fn call(
    tool: &AppTool,
    args: Value,
    runtime: Option<&McpRuntimeContext>,
) -> Result<Value, String> {
    match tool.name.as_str() {
        "kit_resources" => return text(resources_text()),
        "kit_read" => return text(read_resource(&args)?),
        "chats_search" => return text(search_chats(&args)?),
        "chat_read" => return text(read_chat(&args)?),
        _ => {}
    }
    let route = tool.route.as_deref().ok_or("This tool has no route.")?;
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: json!({ "name": route, "arguments": args }),
    };
    let scripts = scripts();
    let runtime_engine = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let response = runtime_engine.block_on(
        crate::mcp_protocol::handle_tools_call_with_runtime_context(request, &scripts, runtime),
    );
    match (response.result, response.error) {
        (Some(result), _) => Ok(result),
        (None, Some(error)) => Err(error.message),
        (None, None) => Err("The tool returned nothing.".into()),
    }
}

fn text(body: String) -> Result<Value, String> {
    Ok(json!({ "content": [{ "type": "text", "text": body }], "isError": false }))
}

fn truncate(mut body: String, max: usize) -> String {
    if body.chars().count() > max {
        body = body.chars().take(max).collect();
        body.push_str("\n[truncated]");
    }
    body
}

fn resources_text() -> String {
    crate::mcp_resources::get_resource_definitions()
        .into_iter()
        .map(|resource| {
            format!(
                "{} — {}{}",
                resource.uri,
                resource.name,
                resource
                    .description
                    .map(|description| format!(": {description}"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_resource(args: &Value) -> Result<String, String> {
    let uri = super::arg_str(args, "uri").ok_or("kit_read needs a uri")?;
    let scripts = scripts();
    let scriptlets = crate::scripts::load_scriptlets()
        .map(|loaded| loaded.into_scriptlets())
        .unwrap_or_default();
    let content = crate::mcp_resources::read_resource(uri, &scripts, &scriptlets, None)?;
    Ok(truncate(content.text, MAX_RESOURCE_CHARS))
}

fn search_chats(args: &Value) -> Result<String, String> {
    let query = super::arg_str(args, "query").ok_or("chats_search needs a query")?;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map(|limit| limit.clamp(1, 50) as usize)
        .unwrap_or(10);
    let hits = crate::ai::agent_chat::ui::history::search_history(query, limit);
    if hits.is_empty() {
        return Ok(format!("No past chats match “{query}”."));
    }
    Ok(hits
        .iter()
        .map(|hit| {
            format!(
                "id: {}\ntitle: {}\nwhen: {}\npreview: {}",
                hit.entry.session_id,
                hit.entry.title_display(),
                hit.entry.timestamp,
                hit.entry.preview_display()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n"))
}

fn read_chat(args: &Value) -> Result<String, String> {
    let id = super::arg_str(args, "id").ok_or("chat_read needs an id")?;
    let conversation = crate::ai::agent_chat::ui::history::load_conversation(id)
        .ok_or_else(|| format!("No past chat has id {id}."))?;
    let body = conversation
        .messages
        .iter()
        .map(|message| format!("{}: {}", message.role, message.body))
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok(truncate(body, MAX_CHAT_CHARS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_tool_names_are_model_safe_and_unique() {
        let tools = tools();
        assert!(tools.iter().any(|tool| tool.name == "kit_notes_create"));
        assert!(tools.iter().any(|tool| tool.name == "kit_clipboard_copy"));
        assert!(tools.iter().any(|tool| tool.name == "computer_list_apps"));
        let mut names = std::collections::HashSet::new();
        for tool in &tools {
            assert!(
                tool.name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "{}",
                tool.name
            );
            assert!(tool.name.len() <= MAX_TOOL_NAME, "{}", tool.name);
            assert!(names.insert(tool.name.clone()), "duplicate {}", tool.name);
        }
    }

    #[test]
    fn only_looking_tools_are_read_only() {
        let tools = tools();
        let read_only = |name: &str| {
            tools
                .iter()
                .find(|tool| tool.name == name)
                .unwrap()
                .read_only
        };
        assert!(read_only("kit_read"));
        assert!(read_only("kit_state"));
        assert!(read_only("computer_list_apps"));
        assert!(!read_only("computer_see"), "screen capture asks first");
        assert!(!read_only("kit_notes_delete"));
        assert!(!read_only("kit_scripts_run"));
        assert!(!read_only("kit_config_set"));
    }

    #[test]
    fn resources_are_listed_with_their_uris() {
        let listed = resources_text();
        assert!(listed.contains("kit://clipboard-history"));
        assert!(listed.contains("kit://calendar"));
    }
}
