//! Translates a resolved Agent Chat profile into a goose recipe.
//!
//! goose accepts a whole session configuration as an inline recipe passed in
//! `session/new` `_meta.recipeDeeplink` (URL-safe base64 of the recipe JSON,
//! no padding). The recipe carries the profile's instructions, its exact tool
//! surface, and the provider/model to start with. When a recipe declares
//! extensions, goose may add its configured default Developer. The runtime
//! reconciles the real session inventory before accepting it or sending text.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::{json, Value};

use crate::ai::agent_chat::profiles::ResolvedAgentChatProfile;

/// Name of the Script Kit tool extension inside goose sessions.
pub(crate) const SCRIPT_KIT_TOOLS_EXTENSION: &str = "scriptkit";
/// Name of goose's built-in developer extension (shell, write, edit).
pub(crate) const GOOSE_DEVELOPER_EXTENSION: &str = "developer";

/// Where one profile tool is served from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolHost {
    /// Served by Script Kit's MCP agent-tools endpoint under this name.
    ScriptKit(&'static str),
    /// Served by goose's developer extension under this name.
    Developer(&'static str),
    /// A goose platform extension (subagents, skills, todo list).
    Platform(&'static str),
    /// The MCP servers the user configured in `config.ts`.
    UserMcp,
}

/// Profile tool id for the user's own MCP servers.
pub(crate) const USER_MCP_TOOL_ID: &str = "mcp";

/// Map a profile tool id onto the extension that serves it.
///
/// Profile ids are the stable, user-facing tool names used in config.ts and
/// flow profiles. Unknown ids map to nothing and are reported by the caller.
pub(crate) fn tool_host(tool_id: &str) -> Option<ToolHost> {
    Some(match tool_id.trim() {
        "web_search" => ToolHost::ScriptKit("web_search"),
        "web_fetch" => ToolHost::ScriptKit("web_fetch"),
        "desktop_search" => ToolHost::ScriptKit("desktop_search"),
        "read" => ToolHost::ScriptKit("read"),
        "grep" => ToolHost::ScriptKit("grep"),
        "find" => ToolHost::ScriptKit("find"),
        "ls" => ToolHost::ScriptKit("ls"),
        "create_file" => ToolHost::ScriptKit("create_file"),
        // Script Kit serves file changes itself so a turn's changes can be
        // reverted and stay inside the profile's write roots.
        "write" => ToolHost::ScriptKit("write"),
        "edit" | "hashline_edit" => ToolHost::ScriptKit("edit"),
        "app" => ToolHost::ScriptKit("app"),
        "bash" | "shell" => ToolHost::Developer("shell"),
        "subagents" | "delegate" => ToolHost::Platform("summon"),
        "skills" => ToolHost::Platform("skills"),
        "todo" => ToolHost::Platform("todo"),
        USER_MCP_TOOL_ID => ToolHost::UserMcp,
        _ => return None,
    })
}

/// The profile tool id for a tool name goose reports. goose names tools
/// `<extension>__<tool>`; this is the inverse of [`tool_host`], so tool
/// policies and tool cards see the same ids profiles are written with.
pub(crate) fn profile_tool_id(goose_tool_name: &str) -> String {
    match goose_tool_name.split_once("__") {
        Some((SCRIPT_KIT_TOOLS_EXTENSION, tool)) => tool.to_string(),
        Some((GOOSE_DEVELOPER_EXTENSION, "shell")) => "bash".to_string(),
        Some((GOOSE_DEVELOPER_EXTENSION, tool)) => tool.to_string(),
        _ => goose_tool_name.to_string(),
    }
}

/// Connection details for Script Kit's MCP agent-tools endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptKitToolsEndpoint {
    pub uri: String,
    pub bearer_token: String,
    /// Identifies the per-session tool policy registered with the endpoint.
    pub policy_id: String,
}

/// Header carrying the per-session tool policy id to the agent-tools endpoint.
pub(crate) const AGENT_POLICY_HEADER: &str = "X-Script-Kit-Agent-Policy";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecipeTools {
    pub script_kit: Vec<&'static str>,
    pub developer: Vec<&'static str>,
    pub platform: Vec<&'static str>,
    pub user_mcp: bool,
    pub unknown: Vec<String>,
}

pub(crate) fn recipe_tools(tool_ids: &[String]) -> RecipeTools {
    let mut tools = RecipeTools::default();
    for tool_id in tool_ids {
        match tool_host(tool_id) {
            Some(ToolHost::ScriptKit(name)) if !tools.script_kit.contains(&name) => {
                tools.script_kit.push(name);
            }
            Some(ToolHost::Developer(name)) if !tools.developer.contains(&name) => {
                tools.developer.push(name);
            }
            Some(ToolHost::Platform(name)) if !tools.platform.contains(&name) => {
                tools.platform.push(name);
            }
            Some(ToolHost::UserMcp) => tools.user_mcp = true,
            Some(_) => {}
            None => tools.unknown.push(tool_id.clone()),
        }
    }
    tools
}

/// The profile's effective tool ids: the structured policy wins over the
/// legacy `tools` list; `None` means the profile did not restrict tools.
pub(crate) fn profile_tool_ids(profile: &ResolvedAgentChatProfile) -> Option<Vec<String>> {
    profile
        .tool_policy
        .as_ref()
        .and_then(|policy| policy.allow.clone())
        .or_else(|| profile.tools.clone())
}

/// Tools a profile gets when it does not restrict tools at all.
pub(crate) const UNRESTRICTED_PROFILE_TOOLS: [&str; 14] = [
    "web_search",
    "web_fetch",
    "read",
    "grep",
    "find",
    "ls",
    "write",
    "edit",
    "bash",
    "app",
    "mcp",
    "subagents",
    "skills",
    "todo",
];

/// The user's `config.ts` MCP servers as goose extensions. Names Script Kit
/// already uses for its own extensions are skipped.
pub(crate) fn user_mcp_extensions(config: &crate::config::McpConfig) -> Vec<Value> {
    let mut servers: Vec<_> = config.enabled_servers().collect();
    servers.sort_by_key(|(left, _)| *left);
    servers
        .into_iter()
        .filter(|(id, _)| {
            !matches!(
                id.as_str(),
                SCRIPT_KIT_TOOLS_EXTENSION
                    | GOOSE_DEVELOPER_EXTENSION
                    | "summon"
                    | "skills"
                    | "todo"
            )
        })
        .map(|(id, server)| match server {
            crate::config::McpServerConfig::Stdio(stdio) => {
                let mut extension = json!({
                    "type": "stdio",
                    "name": id,
                    "description": stdio.description.clone().unwrap_or_default(),
                    "cmd": stdio.command,
                    "args": stdio.args,
                    "envs": stdio.env,
                    "timeout": 300,
                });
                if let Some(cwd) = stdio.cwd.as_deref() {
                    extension["cwd"] = json!(cwd);
                }
                extension
            }
            crate::config::McpServerConfig::Http(http) => json!({
                "type": "streamable_http",
                "name": id,
                "description": http.description.clone().unwrap_or_default(),
                "uri": http.endpoint,
                "headers": http.headers,
                "timeout": 300,
            }),
        })
        .collect()
}

pub(crate) fn profile_instructions(profile: &ResolvedAgentChatProfile) -> String {
    let mut sections = Vec::new();
    if let Some(system) = profile.system_prompt.as_deref().map(str::trim) {
        if !system.is_empty() {
            sections.push(system.to_string());
        }
    }
    if let Some(append) = profile.append_system_prompt.as_deref().map(str::trim) {
        if !append.is_empty() {
            sections.push(append.to_string());
        }
    }
    if let Some(blocked) = profile.blocked_action_message.as_deref().map(str::trim) {
        if !blocked.is_empty() {
            sections.push(format!(
                "When the user asks for an action your tools cannot perform, reply exactly: {blocked}"
            ));
        }
    }
    if sections.is_empty() {
        sections.push(format!(
            "You are the {} assistant in Script Kit.",
            profile.name
        ));
    }
    protect_instructions(&sections.join("\n\n"))
}

/// goose renders recipe instructions as a template. Profile prompts are plain
/// text, so any template delimiters they contain must reach the model verbatim.
pub(crate) fn protect_instructions(text: &str) -> String {
    if text.contains("{{") || text.contains("{%") || text.contains("{#") {
        format!("{{% raw %}}{text}{{% endraw %}}")
    } else {
        text.to_string()
    }
}

pub(crate) struct RecipeInput<'a> {
    pub profile: &'a ResolvedAgentChatProfile,
    pub provider: Option<&'a str>,
    pub model: Option<&'a str>,
    pub tools: &'a RecipeTools,
    pub endpoint: Option<&'a ScriptKitToolsEndpoint>,
    /// Extensions for the user's MCP servers, used when the profile allows `mcp`.
    pub user_mcp: &'a [Value],
}

/// Per-process policy id and bearer are attached only when actually spawning the engine.
pub(super) fn script_kit_extension(endpoint: &ScriptKitToolsEndpoint) -> Value {
    json!({
        "type": "streamable_http",
        "name": SCRIPT_KIT_TOOLS_EXTENSION,
        "description": "Script Kit tools",
        "uri": endpoint.uri,
        "headers": {
            "Authorization": format!("Bearer {}", endpoint.bearer_token),
            AGENT_POLICY_HEADER: endpoint.policy_id,
        },
        "timeout": 300,
    })
}

pub(crate) fn build_recipe(input: RecipeInput<'_>) -> Value {
    let mut extensions = Vec::new();
    if !input.tools.script_kit.is_empty() {
        if let Some(endpoint) = input.endpoint {
            extensions.push(script_kit_extension(endpoint));
        }
    }
    for name in &input.tools.platform {
        extensions.push(json!({
            "type": "platform",
            "name": name,
            "description": "",
            "bundled": true,
        }));
    }
    if input.tools.user_mcp {
        extensions.extend(input.user_mcp.iter().cloned());
    }
    if !input.tools.developer.is_empty() {
        extensions.push(json!({
            "type": "platform",
            "name": GOOSE_DEVELOPER_EXTENSION,
            "description": "",
            "display_name": "Developer",
            "bundled": true,
            "available_tools": input.tools.developer,
        }));
    }

    let mut settings = serde_json::Map::new();
    if let Some(provider) = input.provider.filter(|value| !value.trim().is_empty()) {
        settings.insert("goose_provider".to_string(), json!(provider));
    }
    if let Some(model) = input.model.filter(|value| !value.trim().is_empty()) {
        settings.insert("goose_model".to_string(), json!(model));
    }

    let mut recipe = json!({
        "version": "1.0.0",
        "title": input.profile.name,
        "description": format!("Script Kit {} profile", input.profile.name),
        "instructions": profile_instructions(input.profile),
        "extensions": extensions,
    });
    if !settings.is_empty() {
        recipe["settings"] = Value::Object(settings);
    }
    recipe
}

pub(crate) fn recipe_deeplink(recipe: &Value) -> String {
    URL_SAFE_NO_PAD.encode(recipe.to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::profiles::{
        built_in_quick_ai_profile, built_in_script_kit_profile, AgentChatProfileContext,
    };
    use std::path::PathBuf;

    fn ctx() -> AgentChatProfileContext {
        AgentChatProfileContext {
            kit_path: PathBuf::from("/tmp/kit"),
        }
    }

    fn endpoint() -> ScriptKitToolsEndpoint {
        ScriptKitToolsEndpoint {
            uri: "http://127.0.0.1:43210/mcp/agent".to_string(),
            bearer_token: "token".to_string(),
            policy_id: "policy-1".to_string(),
        }
    }

    #[test]
    fn quick_ai_recipe_exposes_only_web_search_and_no_developer_tools() {
        let profile = built_in_quick_ai_profile(&ctx());
        let tools = recipe_tools(&profile_tool_ids(&profile).expect("quick ai restricts tools"));
        let recipe = build_recipe(RecipeInput {
            profile: &profile,
            provider: Some("chatgpt_codex"),
            model: Some("gpt-6-luna"),
            tools: &tools,
            endpoint: Some(&endpoint()),
            user_mcp: &[json!({"type": "stdio", "name": "github"})],
        });

        let extensions = recipe["extensions"].as_array().expect("extensions");
        assert_eq!(extensions.len(), 1, "{recipe:#}");
        assert_eq!(extensions[0]["name"], SCRIPT_KIT_TOOLS_EXTENSION);
        assert_eq!(tools.script_kit, vec!["web_search"]);
        assert_eq!(extensions[0]["headers"][AGENT_POLICY_HEADER], "policy-1");
        assert_eq!(recipe["settings"]["goose_provider"], "chatgpt_codex");
    }

    #[test]
    fn script_kit_recipe_serves_edits_itself_and_adds_agent_extensions() {
        let profile = built_in_script_kit_profile(&ctx());
        let tools = recipe_tools(&profile_tool_ids(&profile).expect("script kit restricts tools"));
        assert_eq!(tools.developer, vec!["shell"]);
        for tool in ["read", "write", "edit", "app"] {
            assert!(tools.script_kit.contains(&tool), "{tool}");
        }
        assert_eq!(tools.platform, vec!["summon", "skills", "todo"]);
        assert!(tools.user_mcp);
        assert!(tools.unknown.is_empty(), "{:?}", tools.unknown);
        let recipe = build_recipe(RecipeInput {
            profile: &profile,
            provider: None,
            model: None,
            tools: &tools,
            endpoint: Some(&endpoint()),
            user_mcp: &[json!({"type": "stdio", "name": "github"})],
        });
        let names: Vec<_> = recipe["extensions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|extension| extension["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            names,
            vec![
                "scriptkit",
                "summon",
                "skills",
                "todo",
                "github",
                "developer"
            ]
        );
    }

    #[test]
    fn a_profile_without_tools_gets_no_extensions() {
        let profile = built_in_quick_ai_profile(&ctx());
        let recipe = build_recipe(RecipeInput {
            profile: &profile,
            provider: None,
            model: None,
            tools: &RecipeTools::default(),
            endpoint: Some(&endpoint()),
            user_mcp: &[],
        });
        assert_eq!(recipe["extensions"], json!([]));
        assert!(recipe.get("settings").is_none());
    }

    #[test]
    fn config_mcp_servers_become_goose_extensions() {
        let config: crate::config::McpConfig = serde_json::from_value(json!({
            "servers": {
                "github": {"transport": "stdio", "command": "npx", "args": ["-y", "gh-mcp"], "env": {"TOKEN": "t"}},
                "linear": {"transport": "http", "endpoint": "https://mcp.linear.app/mcp", "headers": {"Authorization": "Bearer x"}},
                "off": {"transport": "stdio", "command": "x", "enabled": false},
                "scriptkit": {"transport": "stdio", "command": "x"}
            }
        }))
        .unwrap();
        let extensions = user_mcp_extensions(&config);
        assert_eq!(extensions.len(), 2, "{extensions:#?}");
        assert_eq!(extensions[0]["type"], "stdio");
        assert_eq!(extensions[0]["cmd"], "npx");
        assert_eq!(extensions[0]["envs"]["TOKEN"], "t");
        assert_eq!(extensions[1]["type"], "streamable_http");
        assert_eq!(extensions[1]["uri"], "https://mcp.linear.app/mcp");
    }

    #[test]
    fn template_delimiters_in_prompts_are_protected() {
        assert_eq!(
            protect_instructions("use {{ name }}"),
            "{% raw %}use {{ name }}{% endraw %}"
        );
        assert_eq!(protect_instructions("plain"), "plain");
    }

    #[test]
    fn deeplink_is_unpadded_url_safe_base64_of_the_recipe_json() {
        let recipe = json!({"title": "t?>"});
        let link = recipe_deeplink(&recipe);
        assert!(!link.contains('='));
        let decoded = URL_SAFE_NO_PAD.decode(link).expect("decodes");
        assert_eq!(serde_json::from_slice::<Value>(&decoded).unwrap(), recipe);
    }
}
