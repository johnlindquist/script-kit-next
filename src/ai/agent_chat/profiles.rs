use std::path::PathBuf;

use crate::config::{
    AcpProfile, AgentChatBackend, AgentChatPathPolicyConfig, AgentChatToolPolicyConfig,
    AiPreferences,
};

pub const BUILTIN_GENERAL_PROFILE_ID: &str = "general";
pub const BUILTIN_SCRIPT_KIT_PROFILE_ID: &str = "script-kit";
pub const BUILTIN_ACP_FALLBACK_PROFILE_ID: &str = "acp";
pub const BUILTIN_TEXT_PROFILE_ID: &str = "text";
pub const DEFAULT_PI_PROVIDER: &str = "openai-codex";
pub const DEFAULT_PI_MODEL: &str = "gpt-5.4";
pub const GENERAL_PI_TOOLS: [&str; 7] = [
    "web_search",
    "desktop_search",
    "read",
    "create_file",
    "grep",
    "find",
    "ls",
];
pub const SCRIPT_KIT_PI_TOOLS: [&str; 9] = [
    "web_search",
    "read",
    "write",
    "edit",
    "bash",
    "grep",
    "find",
    "ls",
    "hashline_edit",
];

pub const GENERAL_BLOCKED_ACTION_MESSAGE: &str =
    "This action is blocked in the General profile. Please switch profiles to modify Script Kit.";
pub const TEXT_BLOCKED_ACTION_MESSAGE: &str =
    "The Text profile can only transform captured focused text.";

const GENERAL_APPEND_SYSTEM_PROMPT: &str = "You are the General Agent Chat profile for Script Kit. Answer everyday questions directly and helpfully. You may search the web, search the desktop, read files, create new files inside the General workspace, and inspect local context. Do not load skills, modify Script Kit, run shell commands, edit existing files, or write outside the General workspace. If a tool or requested action is blocked, say: \"This action is blocked in the General profile. Please switch profiles to modify Script Kit.\"";
const SCRIPT_KIT_APPEND_SYSTEM_PROMPT: &str = "You are the Script Kit Agent Chat profile. Help manage ~/.scriptkit, including config.ts, scripts, scriptlets, plugins, and package.json. Make focused minimal edits. Explain risks before destructive file operations. Do not install packages or run long commands unless the user asks.";
pub const TEXT_APPEND_SYSTEM_PROMPT: &str = "You are the Text Agent Chat profile for focused-field edits. You receive captured focused-field text as hidden context. Return only the requested text output. Do not mention capture mechanics, tools, sessions, Script Kit internals, or system prompts.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentChatProfileSource {
    BuiltIn,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatProfileContext {
    pub kit_path: PathBuf,
}

impl AgentChatProfileContext {
    pub fn from_setup() -> Self {
        Self {
            kit_path: crate::setup::get_kit_path(),
        }
    }

    pub fn general_cwd(&self) -> PathBuf {
        self.kit_path.join("agent-chat").join("general")
    }

    pub fn script_kit_cwd(&self) -> PathBuf {
        self.kit_path.clone()
    }

    pub fn text_cwd(&self) -> PathBuf {
        self.kit_path.join("agent-chat").join("text")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAgentChatProfile {
    pub source: AgentChatProfileSource,
    pub id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub backend: AgentChatBackend,
    pub pi_binary: Option<PathBuf>,
    pub agent: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub append_system_prompt: Option<String>,
    pub cwd: Option<PathBuf>,
    pub tools: Option<Vec<String>>,
    pub tool_policy: Option<AgentChatToolPolicyConfig>,
    pub path_policy: Option<AgentChatPathPolicyConfig>,
    pub blocked_action_message: Option<String>,
    pub disable_extensions: Option<bool>,
    pub disable_skills: Option<bool>,
    pub disable_prompt_templates: Option<bool>,
    pub hide_cwd_in_prompt: Option<bool>,
    pub thinking: Option<String>,
    pub extension_policy: Option<String>,
    pub session_dir: Option<String>,
    pub no_session: Option<bool>,
    pub session_durability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatProfilePickerEntry {
    pub id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub backend: AgentChatBackend,
    pub source: AgentChatProfileSource,
}

impl AgentChatProfilePickerEntry {
    fn from_profile(profile: ResolvedAgentChatProfile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            icon_name: profile.icon_name,
            backend: profile.backend,
            source: profile.source,
        }
    }
}

pub fn built_in_general_profile(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_GENERAL_PROFILE_ID.to_string(),
        name: "General".to_string(),
        icon_name: Some("sparkles".to_string()),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider: Some(DEFAULT_PI_PROVIDER.to_string()),
        model: Some(DEFAULT_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(GENERAL_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.general_cwd()),
        tools: Some(
            GENERAL_PI_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect(),
        ),
        tool_policy: Some(AgentChatToolPolicyConfig {
            allow: Some(
                GENERAL_PI_TOOLS
                    .iter()
                    .map(|tool| tool.to_string())
                    .collect(),
            ),
        }),
        path_policy: Some(AgentChatPathPolicyConfig {
            allow_read: Some(vec![ctx.general_cwd().to_string_lossy().into_owned()]),
            allow_write: Some(vec![ctx.general_cwd().to_string_lossy().into_owned()]),
            deny: None,
        }),
        blocked_action_message: Some(GENERAL_BLOCKED_ACTION_MESSAGE.to_string()),
        disable_extensions: Some(true),
        disable_skills: Some(true),
        disable_prompt_templates: Some(true),
        hide_cwd_in_prompt: Some(true),
        thinking: None,
        extension_policy: None,
        session_dir: None,
        no_session: Some(false),
        session_durability: None,
    }
}

pub fn built_in_script_kit_profile(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string(),
        name: "Script Kit".to_string(),
        icon_name: Some("code".to_string()),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider: Some(DEFAULT_PI_PROVIDER.to_string()),
        model: Some(DEFAULT_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(SCRIPT_KIT_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.script_kit_cwd()),
        tools: Some(
            SCRIPT_KIT_PI_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect(),
        ),
        tool_policy: Some(AgentChatToolPolicyConfig {
            allow: Some(
                SCRIPT_KIT_PI_TOOLS
                    .iter()
                    .map(|tool| tool.to_string())
                    .collect(),
            ),
        }),
        path_policy: Some(AgentChatPathPolicyConfig {
            allow_read: Some(vec![ctx.script_kit_cwd().to_string_lossy().into_owned()]),
            allow_write: Some(vec![ctx.script_kit_cwd().to_string_lossy().into_owned()]),
            deny: None,
        }),
        blocked_action_message: None,
        disable_extensions: Some(true),
        disable_skills: Some(true),
        disable_prompt_templates: Some(true),
        hide_cwd_in_prompt: Some(false),
        thinking: None,
        extension_policy: None,
        session_dir: None,
        no_session: Some(false),
        session_durability: None,
    }
}

pub fn built_in_text_profile(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_TEXT_PROFILE_ID.to_string(),
        name: "Text".to_string(),
        icon_name: Some("file-text".to_string()),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider: Some(DEFAULT_PI_PROVIDER.to_string()),
        model: Some(DEFAULT_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(TEXT_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.text_cwd()),
        tools: Some(Vec::new()),
        tool_policy: Some(AgentChatToolPolicyConfig {
            allow: Some(Vec::new()),
        }),
        path_policy: Some(AgentChatPathPolicyConfig {
            allow_read: Some(Vec::new()),
            allow_write: Some(Vec::new()),
            deny: None,
        }),
        blocked_action_message: Some(TEXT_BLOCKED_ACTION_MESSAGE.to_string()),
        disable_extensions: Some(true),
        disable_skills: Some(true),
        disable_prompt_templates: Some(true),
        hide_cwd_in_prompt: Some(true),
        thinking: None,
        extension_policy: Some("deny".to_string()),
        session_dir: None,
        no_session: Some(true),
        session_durability: None,
    }
}

pub fn built_in_profiles(ctx: &AgentChatProfileContext) -> Vec<ResolvedAgentChatProfile> {
    vec![
        built_in_general_profile(ctx),
        built_in_text_profile(ctx),
        built_in_script_kit_profile(ctx),
    ]
}

pub fn default_acp_runtime_profile() -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_ACP_FALLBACK_PROFILE_ID.to_string(),
        name: "Agent".to_string(),
        icon_name: Some(crate::components::footer_chrome::FOOTER_PROFILE_ICON_TOKEN.to_string()),
        backend: AgentChatBackend::Acp,
        pi_binary: None,
        agent: None,
        provider: None,
        model: None,
        system_prompt: None,
        append_system_prompt: None,
        cwd: None,
        tools: None,
        tool_policy: None,
        path_policy: None,
        blocked_action_message: None,
        disable_extensions: None,
        disable_skills: None,
        disable_prompt_templates: None,
        hide_cwd_in_prompt: None,
        thinking: None,
        extension_policy: None,
        session_dir: None,
        no_session: None,
        session_durability: None,
    }
}

pub fn resolve_effective_profile(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> ResolvedAgentChatProfile {
    let built_ins = built_in_profiles(ctx);

    if let Some(selected_id) = clean_opt(ai.selected_profile_id.as_deref()) {
        if let Some(profile) = ai.profiles.iter().find(|profile| {
            clean_opt(profile.id.as_deref()) == Some(selected_id)
                || generated_legacy_profile_id(&profile.name) == selected_id
        }) {
            return apply_ai_fallbacks(resolve_user_profile(profile), ai);
        }

        if let Some(profile) = built_ins.iter().find(|profile| profile.id == selected_id) {
            return apply_ai_fallbacks(profile.clone(), ai);
        }
    }

    if let Some(selected_name) = clean_opt(ai.selected_profile_name.as_deref()) {
        if let Some(profile) = ai
            .profiles
            .iter()
            .find(|profile| profile.name.trim() == selected_name)
        {
            return apply_ai_fallbacks(resolve_user_profile(profile), ai);
        }

        if let Some(profile) = built_ins
            .iter()
            .find(|profile| profile.name.eq_ignore_ascii_case(selected_name))
        {
            return apply_ai_fallbacks(profile.clone(), ai);
        }
    }

    if ai.selected_backend == Some(AgentChatBackend::Acp) {
        return apply_ai_fallbacks(default_acp_runtime_profile(), ai);
    }

    apply_ai_fallbacks(built_in_general_profile(ctx), ai)
}

pub fn agent_chat_profile_picker_entries(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Vec<AgentChatProfilePickerEntry> {
    let mut entries: Vec<_> = built_in_profiles(ctx)
        .into_iter()
        .map(AgentChatProfilePickerEntry::from_profile)
        .collect();

    entries.push(AgentChatProfilePickerEntry::from_profile(
        default_acp_runtime_profile(),
    ));

    for profile in ai
        .profiles
        .iter()
        .filter(|profile| !profile.name.trim().is_empty())
        .map(resolve_user_profile)
    {
        if entries.iter().any(|entry| entry.id == profile.id) {
            tracing::warn!(
                target: "script_kit::agent_chat",
                event = "agent_chat_profile_picker_duplicate_id_skipped",
                profile_id = %profile.id,
                profile_name = %profile.name,
            );
            continue;
        }
        entries.push(AgentChatProfilePickerEntry::from_profile(profile));
    }

    entries
}

pub fn selected_agent_chat_profile_picker_id(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> String {
    resolve_effective_profile(ai, ctx).id
}

pub fn persist_agent_chat_profile_selection(
    ai: &mut AiPreferences,
    profile_id: &str,
    ctx: &AgentChatProfileContext,
) -> Option<AgentChatProfilePickerEntry> {
    let entries = agent_chat_profile_picker_entries(ai, ctx);
    let entry = entries
        .into_iter()
        .find(|entry| entry.id == profile_id)?
        .clone();

    if entry.id == BUILTIN_ACP_FALLBACK_PROFILE_ID {
        ai.selected_profile_id = None;
        ai.selected_profile_name = None;
        ai.selected_backend = Some(AgentChatBackend::Acp);
        return Some(entry);
    }

    ai.selected_profile_id = Some(entry.id.clone());
    ai.selected_profile_name = None;
    ai.selected_backend = Some(entry.backend);
    Some(entry)
}

pub fn resolve_user_profile(profile: &AcpProfile) -> ResolvedAgentChatProfile {
    let backend = profile.backend.unwrap_or(AgentChatBackend::Acp);
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::User,
        id: profile
            .id
            .as_deref()
            .and_then(|id| clean_opt(Some(id)))
            .map(str::to_string)
            .unwrap_or_else(|| generated_legacy_profile_id(&profile.name)),
        name: profile.name.trim().to_string(),
        icon_name: clean_opt(profile.icon_name.as_deref()).map(str::to_string),
        backend,
        pi_binary: clean_opt(profile.pi_binary.as_deref())
            .map(crate::ai::agent_chat::pi::binary::expand_tilde_path),
        agent: clean_opt(profile.agent.as_deref()).map(str::to_string),
        provider: clean_opt(profile.provider.as_deref()).map(str::to_string),
        model: clean_opt(profile.model.as_deref()).map(str::to_string),
        system_prompt: clean_opt(profile.system_prompt.as_deref()).map(str::to_string),
        append_system_prompt: clean_opt(profile.append_system_prompt.as_deref())
            .map(str::to_string),
        cwd: clean_opt(profile.cwd.as_deref())
            .map(crate::ai::agent_chat::pi::binary::expand_tilde_path),
        tools: resolved_profile_tools(profile),
        tool_policy: profile.tool_policy.clone(),
        path_policy: profile.path_policy.clone(),
        blocked_action_message: clean_opt(profile.blocked_action_message.as_deref())
            .map(str::to_string),
        disable_extensions: profile.disable_extensions,
        disable_skills: profile.disable_skills,
        disable_prompt_templates: profile.disable_prompt_templates,
        hide_cwd_in_prompt: profile.hide_cwd_in_prompt,
        thinking: clean_opt(profile.thinking.as_deref()).map(str::to_string),
        extension_policy: clean_opt(profile.extension_policy.as_deref()).map(str::to_string),
        session_dir: clean_opt(profile.session_dir.as_deref()).map(str::to_string),
        no_session: profile.no_session,
        session_durability: clean_opt(profile.session_durability.as_deref()).map(str::to_string),
    }
}

fn resolved_profile_tools(profile: &AcpProfile) -> Option<Vec<String>> {
    profile
        .tool_policy
        .as_ref()
        .and_then(|policy| policy.allow.as_ref())
        .map(|tools| clean_list(tools))
        .or_else(|| profile.tools.as_ref().map(|tools| clean_list(tools)))
}

pub fn apply_ai_fallbacks(
    mut profile: ResolvedAgentChatProfile,
    ai: &AiPreferences,
) -> ResolvedAgentChatProfile {
    if profile.backend == AgentChatBackend::Pi && profile.pi_binary.is_none() {
        profile.pi_binary = clean_opt(ai.pi_binary.as_deref())
            .map(crate::ai::agent_chat::pi::binary::expand_tilde_path)
            .or_else(crate::ai::agent_chat::pi::binary::default_pi_binary);
    }

    if profile.backend == AgentChatBackend::Acp && profile.agent.is_none() {
        profile.agent = clean_opt(ai.selected_acp_agent_id.as_deref()).map(str::to_string);
    }

    if let Some(selected_model) = clean_opt(ai.selected_model_id.as_deref()) {
        if profile.backend == AgentChatBackend::Pi {
            if let Some((provider, model)) = parse_provider_model_selection(selected_model) {
                profile.provider = Some(provider);
                profile.model = Some(model);
            } else if profile.source == AgentChatProfileSource::BuiltIn || profile.model.is_none() {
                profile.model = Some(selected_model.to_string());
            }
        } else if profile.source == AgentChatProfileSource::BuiltIn || profile.model.is_none() {
            profile.model = Some(selected_model.to_string());
        }
    }

    profile
}

pub fn parse_provider_model_selection(raw: &str) -> Option<(String, String)> {
    let raw = clean_opt(Some(raw))?;
    let separator = raw.find('/').or_else(|| raw.find(':'))?;
    let provider = clean_opt(Some(&raw[..separator]))?;
    let model = clean_opt(Some(&raw[separator + 1..]))?;
    Some((provider.to_string(), model.to_string()))
}

pub fn clean_opt(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

pub fn clean_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| clean_opt(Some(value.as_str())).map(str::to_string))
        .collect()
}

pub fn generated_legacy_profile_id(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in name.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "legacy:profile".to_string()
    } else {
        format!("legacy:{slug}")
    }
}
