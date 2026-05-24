use std::path::PathBuf;

use crate::config::{AcpProfile, AgentChatBackend, AiPreferences};

pub const BUILTIN_GENERAL_PROFILE_ID: &str = "general";
pub const BUILTIN_SCRIPT_KIT_PROFILE_ID: &str = "script-kit";
pub const DEFAULT_PI_PROVIDER: &str = "openai-codex";
pub const DEFAULT_PI_MODEL: &str = "gpt-5.4";
pub const SCRIPT_KIT_PI_TOOLS: [&str; 8] = [
    "read",
    "write",
    "edit",
    "bash",
    "grep",
    "find",
    "ls",
    "hashline_edit",
];

const GENERAL_APPEND_SYSTEM_PROMPT: &str = "You are the General Agent Chat profile for Script Kit. Answer everyday questions directly and helpfully. Do not use file, shell, workspace, skill, template, or extension capabilities unless the user explicitly attaches context that requires them.";
const SCRIPT_KIT_APPEND_SYSTEM_PROMPT: &str = "You are the Script Kit Agent Chat profile. Help manage ~/.scriptkit, including config.ts, scripts, scriptlets, plugins, and package.json. Make focused minimal edits. Explain risks before destructive file operations. Do not install packages or run long commands unless the user asks.";

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAgentChatProfile {
    pub source: AgentChatProfileSource,
    pub id: String,
    pub name: String,
    pub backend: AgentChatBackend,
    pub agent: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub append_system_prompt: Option<String>,
    pub cwd: Option<PathBuf>,
    pub tools: Option<Vec<String>>,
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

pub fn built_in_general_profile(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_GENERAL_PROFILE_ID.to_string(),
        name: "General".to_string(),
        backend: AgentChatBackend::Pi,
        agent: None,
        provider: Some(DEFAULT_PI_PROVIDER.to_string()),
        model: Some(DEFAULT_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(GENERAL_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.general_cwd()),
        tools: Some(Vec::new()),
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
        backend: AgentChatBackend::Pi,
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

pub fn built_in_profiles(ctx: &AgentChatProfileContext) -> Vec<ResolvedAgentChatProfile> {
    vec![
        built_in_general_profile(ctx),
        built_in_script_kit_profile(ctx),
    ]
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

    apply_ai_fallbacks(built_in_general_profile(ctx), ai)
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
        backend,
        agent: clean_opt(profile.agent.as_deref()).map(str::to_string),
        provider: clean_opt(profile.provider.as_deref()).map(str::to_string),
        model: clean_opt(profile.model.as_deref()).map(str::to_string),
        system_prompt: clean_opt(profile.system_prompt.as_deref()).map(str::to_string),
        append_system_prompt: clean_opt(profile.append_system_prompt.as_deref())
            .map(str::to_string),
        cwd: clean_opt(profile.cwd.as_deref()).map(PathBuf::from),
        tools: profile.tools.as_ref().map(|tools| clean_list(tools)),
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

pub fn apply_ai_fallbacks(
    mut profile: ResolvedAgentChatProfile,
    ai: &AiPreferences,
) -> ResolvedAgentChatProfile {
    if profile.backend == AgentChatBackend::Acp && profile.agent.is_none() {
        profile.agent = clean_opt(ai.selected_acp_agent_id.as_deref()).map(str::to_string);
    }

    if let Some(selected_model) = clean_opt(ai.selected_model_id.as_deref()).map(str::to_string) {
        if profile.source == AgentChatProfileSource::BuiltIn || profile.model.is_none() {
            profile.model = Some(selected_model);
        }
    }

    profile
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
