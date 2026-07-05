use std::path::PathBuf;

use crate::config::{
    AgentChatBackend, AgentChatPathPolicyConfig, AgentChatProfile, AgentChatToolPolicyConfig,
    AiPreferences,
};
use crate::plugins::profiles::{
    discover_plugin_profiles, prompt_file_text, resolved_artifact_tools, validate_profile,
    PluginProfile, ProfilePromptMode,
};

pub const BUILTIN_GENERAL_PROFILE_ID: &str = "general";
pub const BUILTIN_SCRIPT_KIT_PROFILE_ID: &str = "script-kit";
pub const BUILTIN_TEXT_PROFILE_ID: &str = "text";
pub const BUILTIN_BRAIN_PROFILE_ID: &str = "brain";
pub const BUILTIN_QUICK_AI_PROFILE_ID: &str = "quick-ai";
pub const DEFAULT_PI_PROVIDER: &str = "openai-codex";
pub const DEFAULT_PI_MODEL: &str = "gpt-5.4";
/// Quick AI (launcher Tab-with-text) is pinned to the fastest Codex model so
/// answers stream back with minimal latency. It intentionally ignores the
/// user's selected Agent Chat model.
pub const QUICK_AI_PI_MODEL: &str = "gpt-5.3-codex-spark";
/// The Text/rewrite mini surface is likewise pinned to the fastest Codex
/// model: the instant rewrite flow fires three variation turns per submit and
/// they must stream back near-instantly — speed over depth. Resolved via
/// `resolve_focused_text_pi_launch`, which refuses the global model override.
pub const TEXT_PI_MODEL: &str = QUICK_AI_PI_MODEL;

/// A curated Pi provider ("Agent") and its selectable models for Agent Chat
/// model pickers.
///
/// The live provider/model catalog is advertised dynamically by the `pi` agent
/// at runtime (`get_available_models`); this static fallback lets the launcher
/// pre-select a provider/model WITHOUT a live session. The primary launcher
/// catalog is Codex-only; alternative providers are available through advanced
/// configuration paths. Selections persist as the namespaced
/// `selectedModelId = "<provider>/<model>"` that the Pi launch reads (see
/// [`parse_provider_model_selection`]).
pub struct PiProviderCatalogEntry {
    pub id: &'static str,
    pub display_name: &'static str,
    /// `(model_id, model_display_name)` pairs; `model_id` is the bare model
    /// (namespaced with the provider id at selection time).
    pub models: Vec<(&'static str, &'static str)>,
}

/// Primary static Pi provider → model catalog for the launcher picker.
pub fn pi_provider_model_catalog() -> Vec<PiProviderCatalogEntry> {
    vec![PiProviderCatalogEntry {
        id: "openai-codex",
        display_name: "Codex",
        models: vec![
            ("gpt-5.5", "GPT-5.5"),
            ("gpt-5.4", "GPT-5.4"),
            ("gpt-5-mini", "GPT-5 mini"),
        ],
    }]
}

/// Advanced static provider catalog retained for settings/configuration flows.
pub fn advanced_pi_provider_model_catalog() -> Vec<PiProviderCatalogEntry> {
    vec![
        PiProviderCatalogEntry {
            id: "anthropic",
            display_name: "Claude",
            models: vec![
                ("claude-opus-4-6", "Opus 4.6"),
                ("claude-sonnet-4-6", "Sonnet 4.6"),
                ("claude-sonnet-4-5", "Sonnet 4.5"),
                ("claude-haiku-4-5", "Haiku 4.5"),
            ],
        },
        PiProviderCatalogEntry {
            id: "google",
            display_name: "Gemini",
            models: vec![
                ("gemini-2.5-pro", "Gemini 2.5 Pro"),
                ("gemini-2.5-flash", "Gemini 2.5 Flash"),
            ],
        },
    ]
}
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
/// The Text/mini profile is a locked-down text rewriter, but users also ask it
/// live-info questions (e.g. "When is the next nba game?") and expect web access.
/// It gets exactly one read-only network tool — `web_search` — and nothing else
/// (no fs, no skills, no extensions); see `built_in_text_profile`.
pub const TEXT_PI_TOOLS: [&str; 1] = ["web_search"];

/// The Brain profile answers from memory first: same safe toolset as General
/// (read-only beyond its workspace), tuned for quick palette-launched
/// questions that lean on staged brain recall.
pub const BRAIN_PI_TOOLS: [&str; 7] = [
    "web_search",
    "desktop_search",
    "read",
    "create_file",
    "grep",
    "find",
    "ls",
];

pub const GENERAL_BLOCKED_ACTION_MESSAGE: &str =
    "This action is blocked in the General profile. Please switch profiles to modify Script Kit.";
pub const BRAIN_BLOCKED_ACTION_MESSAGE: &str =
    "This action is blocked in the Brain profile. Please switch profiles to modify Script Kit.";
pub const TEXT_BLOCKED_ACTION_MESSAGE: &str = "The Text profile can only transform captured focused text or search the web for public current information.";

const BRAIN_APPEND_SYSTEM_PROMPT: &str = "You are the Brain profile: Script Kit's memory-aware assistant. Turns may include a 'Brain recall' block — relevant excerpts auto-retrieved from the user's local knowledge (their notes and past conversations with you). Treat recall as your own memory: answer personal or project questions from it directly and confidently, mention naturally where a memory came from (e.g. 'your note Egghead publish checklist'), and prefer newer memories when they conflict. If recall doesn't cover the question, say so plainly and answer from general knowledge or search — never invent memories. Users usually arrive here by typing a quick question into the launcher, so lead with the answer and keep replies tight. You may search the web, search the desktop, read files, and create files inside your workspace. Do not load skills, modify Script Kit, run shell commands, or write outside your workspace.";

const GENERAL_APPEND_SYSTEM_PROMPT: &str = "You are the General Agent Chat profile for Script Kit. Answer everyday questions directly and helpfully. You may search the web, search the desktop, read files, create new files inside the General workspace, and inspect local context. Do not load skills, modify Script Kit, run shell commands, edit existing files, or write outside the General workspace. If a tool or requested action is blocked, say: \"This action is blocked in the General profile. Please switch profiles to modify Script Kit.\"";
const SCRIPT_KIT_APPEND_SYSTEM_PROMPT: &str = "You are the Script Kit Agent Chat profile. Help manage ~/.scriptkit, including config.ts, scripts, scriptlets, plugins, and package.json. Make focused minimal edits. Explain risks before destructive file operations. Do not install packages or run long commands unless the user asks.";
pub const QUICK_AI_BLOCKED_ACTION_MESSAGE: &str =
    "Quick AI answers from the model only — no tools, files, or context. Open Agent Chat for anything more.";

pub const QUICK_AI_APPEND_SYSTEM_PROMPT: &str = "You are Quick AI: a zero-context, instant-answer mode launched by pressing Tab on a query typed into the Script Kit launcher. You receive only the user's typed text — no files, no selection, no screenshots, no memories, no tools. Lead with the answer in the first sentence and keep the whole reply tight. Prefer plain prose; use a short list or fenced code block only when the answer genuinely needs one. If the question depends on live or post-cutoff information you cannot verify, say so in one clause and give your best answer anyway. Never mention tools, sessions, context mechanics, Script Kit internals, or system prompts.";

pub const TEXT_APPEND_SYSTEM_PROMPT: &str = "You are the Text Agent Chat profile for focused-field edits and compact one-off questions. You receive captured focused-field text as hidden context. For rewrite, edit, format, translate, summarize, or variation requests, return only the requested final text; do not add commentary, labels, markdown fences, citations, or explanations unless the user explicitly asks for them. You may use web_search, and only web_search, for live or time-sensitive public facts such as schedules, dates, prices, news, releases, current availability, or anything likely to have changed. For live-info questions, search before answering, answer directly, and include concise source URLs when available. If search fails or results are insufficient, say what is uncertain without claiming you have no web access. Do not mention capture mechanics, tool names, sessions, Script Kit internals, or system prompts.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentChatProfileSource {
    BuiltIn,
    User,
    Plugin,
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

    pub fn brain_cwd(&self) -> PathBuf {
        self.kit_path.join("agent-chat").join("brain")
    }

    pub fn quick_ai_cwd(&self) -> PathBuf {
        self.kit_path.join("agent-chat").join("quick-ai")
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
    pub disable_context_files: Option<bool>,
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

pub fn built_in_brain_profile(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_BRAIN_PROFILE_ID.to_string(),
        name: "Brain".to_string(),
        icon_name: Some("brain".to_string()),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider: Some(DEFAULT_PI_PROVIDER.to_string()),
        model: Some(DEFAULT_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(BRAIN_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.brain_cwd()),
        tools: Some(BRAIN_PI_TOOLS.iter().map(|tool| tool.to_string()).collect()),
        tool_policy: Some(AgentChatToolPolicyConfig {
            allow: Some(BRAIN_PI_TOOLS.iter().map(|tool| tool.to_string()).collect()),
        }),
        path_policy: Some(AgentChatPathPolicyConfig {
            allow_read: Some(vec![ctx.brain_cwd().to_string_lossy().into_owned()]),
            allow_write: Some(vec![ctx.brain_cwd().to_string_lossy().into_owned()]),
            deny: None,
        }),
        blocked_action_message: Some(BRAIN_BLOCKED_ACTION_MESSAGE.to_string()),
        disable_extensions: Some(true),
        disable_skills: Some(true),
        disable_prompt_templates: Some(true),
        disable_context_files: Some(true),
        hide_cwd_in_prompt: Some(true),
        thinking: None,
        extension_policy: None,
        session_dir: None,
        no_session: Some(false),
        session_durability: None,
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
        disable_context_files: Some(true),
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
        disable_context_files: Some(true),
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
        model: Some(TEXT_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(TEXT_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.text_cwd()),
        tools: Some(TEXT_PI_TOOLS.iter().map(|tool| tool.to_string()).collect()),
        tool_policy: Some(AgentChatToolPolicyConfig {
            allow: Some(TEXT_PI_TOOLS.iter().map(|tool| tool.to_string()).collect()),
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
        disable_context_files: Some(true),
        hide_cwd_in_prompt: Some(true),
        thinking: None,
        extension_policy: Some("deny".to_string()),
        session_dir: None,
        no_session: Some(true),
        session_durability: None,
    }
}

/// Zero-context profile behind the launcher's Tab-with-text "Quick AI" mode.
///
/// Everything is stripped: no tools (`--no-tools`), no extensions, no skills,
/// no prompt templates, no context files, no session persistence, and an
/// empty path policy. The model is pinned to [`QUICK_AI_PI_MODEL`] and must
/// not be overridden by the user's Agent Chat model selection — resolve it
/// via `resolve_quick_ai_pi_launch`, never through `apply_ai_fallbacks`.
///
/// Intentionally NOT listed in [`built_in_profiles`]: Quick AI is a launch
/// mode, not a pickable profile.
pub fn built_in_quick_ai_profile(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::BuiltIn,
        id: BUILTIN_QUICK_AI_PROFILE_ID.to_string(),
        name: "Quick AI".to_string(),
        icon_name: Some("zap".to_string()),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider: Some(DEFAULT_PI_PROVIDER.to_string()),
        model: Some(QUICK_AI_PI_MODEL.to_string()),
        system_prompt: None,
        append_system_prompt: Some(QUICK_AI_APPEND_SYSTEM_PROMPT.to_string()),
        cwd: Some(ctx.quick_ai_cwd()),
        tools: Some(Vec::new()),
        tool_policy: Some(AgentChatToolPolicyConfig {
            allow: Some(Vec::new()),
        }),
        path_policy: Some(AgentChatPathPolicyConfig {
            allow_read: Some(Vec::new()),
            allow_write: Some(Vec::new()),
            deny: None,
        }),
        blocked_action_message: Some(QUICK_AI_BLOCKED_ACTION_MESSAGE.to_string()),
        disable_extensions: Some(true),
        disable_skills: Some(true),
        disable_prompt_templates: Some(true),
        disable_context_files: Some(true),
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
        built_in_brain_profile(ctx),
        built_in_general_profile(ctx),
        built_in_text_profile(ctx),
        built_in_script_kit_profile(ctx),
    ]
}

pub fn resolve_effective_profile(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> ResolvedAgentChatProfile {
    let built_ins = built_in_profiles(ctx);
    let plugin_profiles = resolved_plugin_profiles(ctx);

    if let Some(selected_id) = clean_opt(ai.selected_profile_id.as_deref()) {
        if selected_id.starts_with("plugin:") {
            if let Some(profile) = plugin_profiles
                .iter()
                .find(|profile| profile.id == selected_id)
            {
                return apply_ai_fallbacks(profile.clone(), ai);
            }
            return apply_ai_fallbacks(built_in_general_profile(ctx), ai);
        }

        if let Some(profile) = ai.profiles.iter().find(|profile| {
            clean_opt(profile.id.as_deref()) == Some(selected_id)
                || generated_legacy_profile_id(&profile.name) == selected_id
        }) {
            return apply_ai_fallbacks(resolve_user_profile(profile), ai);
        }

        if let Some(profile) = plugin_profiles
            .iter()
            .find(|profile| profile.id == selected_id)
        {
            return apply_ai_fallbacks(profile.clone(), ai);
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

        if let Some(profile) = plugin_profiles
            .iter()
            .find(|profile| profile.name.eq_ignore_ascii_case(selected_name))
        {
            return apply_ai_fallbacks(profile.clone(), ai);
        }

        if let Some(profile) = built_ins
            .iter()
            .find(|profile| profile.name.eq_ignore_ascii_case(selected_name))
        {
            return apply_ai_fallbacks(profile.clone(), ai);
        }
    }

    // No explicit selection: the memory-aware Brain profile is the default —
    // quick questions typed into the launcher should land on the profile that
    // knows the user.
    apply_ai_fallbacks(built_in_brain_profile(ctx), ai)
}

pub fn agent_chat_profile_picker_entries(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Vec<AgentChatProfilePickerEntry> {
    resolved_agent_chat_profile_picker_profiles(ai, ctx)
        .into_iter()
        .map(AgentChatProfilePickerEntry::from_profile)
        .collect()
}

pub fn resolved_agent_chat_profile_picker_profiles(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Vec<ResolvedAgentChatProfile> {
    let mut entries = built_in_profiles(ctx);

    for profile in resolved_plugin_profiles(ctx) {
        if entries.iter().any(|entry| entry.id == profile.id) {
            tracing::warn!(
                target: "script_kit::agent_chat",
                event = "agent_chat_profile_picker_duplicate_id_skipped",
                profile_id = %profile.id,
                profile_name = %profile.name,
            );
            continue;
        }
        entries.push(profile);
    }

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
        entries.push(profile);
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

    ai.selected_profile_id = Some(entry.id.clone());
    ai.selected_profile_name = None;
    ai.selected_backend = Some(AgentChatBackend::Pi);
    Some(entry)
}

pub fn resolve_user_profile(profile: &AgentChatProfile) -> ResolvedAgentChatProfile {
    let backend = profile.backend.unwrap_or(AgentChatBackend::Pi);
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
        disable_context_files: profile.disable_context_files,
        hide_cwd_in_prompt: profile.hide_cwd_in_prompt,
        thinking: clean_opt(profile.thinking.as_deref()).map(str::to_string),
        extension_policy: clean_opt(profile.extension_policy.as_deref()).map(str::to_string),
        session_dir: clean_opt(profile.session_dir.as_deref()).map(str::to_string),
        no_session: profile.no_session,
        session_durability: clean_opt(profile.session_durability.as_deref()).map(str::to_string),
    }
}

/// Plugin-profile resolution walks `<kit>/plugins/**` and reads every
/// manifest and prompt file from disk. Hot paths resolve profiles several
/// times per keystroke/frame (Profile Search navigation, the Agent Chat
/// profile picker, footer label refresh, automation state snapshots), so the
/// disk-backed pass is memoized here. Plugin installs/removals surface after
/// the TTL elapses; callers that mutate plugins on disk and need the change
/// immediately should call [`invalidate_plugin_profile_cache`].
const PLUGIN_PROFILE_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(2);

struct PluginProfileCacheEntry {
    plugins_dir: PathBuf,
    kit_path: PathBuf,
    refreshed_at: std::time::Instant,
    profiles: Vec<ResolvedAgentChatProfile>,
}

static PLUGIN_PROFILE_CACHE: std::sync::Mutex<Option<PluginProfileCacheEntry>> =
    std::sync::Mutex::new(None);

pub fn invalidate_plugin_profile_cache() {
    if let Ok(mut cache) = PLUGIN_PROFILE_CACHE.lock() {
        *cache = None;
    }
}

fn plugin_profile_cache_entry_is_fresh(
    entry: &PluginProfileCacheEntry,
    plugins_dir: &std::path::Path,
    kit_path: &std::path::Path,
    now: std::time::Instant,
) -> bool {
    entry.plugins_dir == plugins_dir
        && entry.kit_path == kit_path
        && now.duration_since(entry.refreshed_at) < PLUGIN_PROFILE_CACHE_TTL
}

pub fn resolved_plugin_profiles(ctx: &AgentChatProfileContext) -> Vec<ResolvedAgentChatProfile> {
    let plugins_dir = crate::plugins::plugins_container_dir();
    if let Ok(cache) = PLUGIN_PROFILE_CACHE.lock() {
        if let Some(entry) = cache.as_ref() {
            if plugin_profile_cache_entry_is_fresh(
                entry,
                &plugins_dir,
                &ctx.kit_path,
                std::time::Instant::now(),
            ) {
                return entry.profiles.clone();
            }
        }
    }

    let profiles = resolved_plugin_profiles_uncached(ctx);
    if let Ok(mut cache) = PLUGIN_PROFILE_CACHE.lock() {
        *cache = Some(PluginProfileCacheEntry {
            plugins_dir,
            kit_path: ctx.kit_path.clone(),
            refreshed_at: std::time::Instant::now(),
            profiles: profiles.clone(),
        });
    }
    profiles
}

fn resolved_plugin_profiles_uncached(
    ctx: &AgentChatProfileContext,
) -> Vec<ResolvedAgentChatProfile> {
    let profiles = match discover_plugin_profiles() {
        Ok(profiles) => profiles,
        Err(error) => {
            tracing::warn!(
                target: "script_kit::agent_chat",
                error = %error,
                "agent_chat_plugin_profile_discovery_failed"
            );
            return Vec::new();
        }
    };
    resolve_plugin_profile_entries(profiles, ctx)
}

pub fn resolve_plugin_profile_entries(
    profiles: Vec<PluginProfile>,
    ctx: &AgentChatProfileContext,
) -> Vec<ResolvedAgentChatProfile> {
    let mut resolved = Vec::new();
    for profile in profiles {
        match resolve_plugin_profile(&profile, ctx) {
            Ok(profile) => {
                if resolved
                    .iter()
                    .any(|entry: &ResolvedAgentChatProfile| entry.id == profile.id)
                {
                    tracing::warn!(
                        target: "script_kit::agent_chat",
                        event = "agent_chat_plugin_profile_duplicate_id_skipped",
                        profile_id = %profile.id,
                        profile_name = %profile.name,
                    );
                    continue;
                }
                resolved.push(profile);
            }
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::agent_chat",
                    plugin_id = %profile.plugin_id,
                    profile_id = %profile.profile_id,
                    error = %error,
                    "agent_chat_plugin_profile_resolve_failed"
                );
            }
        }
    }
    resolved
}

pub fn resolve_plugin_profile(
    profile: &PluginProfile,
    ctx: &AgentChatProfileContext,
) -> anyhow::Result<ResolvedAgentChatProfile> {
    validate_profile(profile)?;
    let prompt_text = plugin_prompt_with_policy(profile)?;
    let artifact = &profile.artifact;
    let cwd = clean_opt(artifact.cwd.as_deref())
        .map(crate::ai::agent_chat::pi::binary::expand_tilde_path)
        .unwrap_or_else(|| {
            ctx.kit_path
                .join("agent-chat")
                .join("profiles")
                .join(&profile.profile_id)
        });

    let mut resolved = ResolvedAgentChatProfile {
        source: AgentChatProfileSource::Plugin,
        id: format!("plugin:{}/{}", profile.plugin_id, profile.profile_id),
        name: artifact.name.trim().to_string(),
        icon_name: clean_opt(artifact.icon_name.as_deref()).map(str::to_string),
        backend: artifact.backend.unwrap_or(AgentChatBackend::Pi),
        pi_binary: None,
        agent: None,
        provider: clean_opt(artifact.provider.as_deref()).map(str::to_string),
        model: clean_opt(artifact.model.as_deref()).map(str::to_string),
        system_prompt: None,
        append_system_prompt: None,
        cwd: Some(cwd),
        tools: resolved_artifact_tools(artifact),
        tool_policy: artifact.tool_policy.clone(),
        path_policy: Some(artifact.path_policy.clone()),
        blocked_action_message: clean_opt(artifact.blocked_action_message.as_deref())
            .map(str::to_string),
        disable_extensions: Some(artifact.disable_extensions.unwrap_or(true)),
        disable_skills: Some(artifact.disable_skills.unwrap_or(true)),
        disable_prompt_templates: Some(artifact.disable_prompt_templates.unwrap_or(true)),
        disable_context_files: Some(artifact.disable_context_files.unwrap_or(true)),
        hide_cwd_in_prompt: Some(artifact.hide_cwd_in_prompt.unwrap_or(true)),
        thinking: clean_opt(artifact.thinking.as_deref()).map(str::to_string),
        extension_policy: clean_opt(artifact.extension_policy.as_deref())
            .map(str::to_string)
            .or_else(|| Some("deny".to_string())),
        session_dir: clean_opt(artifact.session_dir.as_deref()).map(str::to_string),
        no_session: Some(artifact.no_session.unwrap_or(false)),
        session_durability: clean_opt(artifact.session_durability.as_deref()).map(str::to_string),
    };

    match artifact.prompt.mode {
        ProfilePromptMode::Replace => resolved.system_prompt = Some(prompt_text),
        ProfilePromptMode::Append => resolved.append_system_prompt = Some(prompt_text),
    }

    Ok(resolved)
}

fn plugin_prompt_with_policy(profile: &PluginProfile) -> anyhow::Result<String> {
    let prompt = prompt_file_text(profile)?;
    Ok(format!(
        "{}\n\n{}",
        prompt.trim_end(),
        plugin_profile_policy_appendix(profile)
    ))
}

fn plugin_profile_policy_appendix(profile: &PluginProfile) -> String {
    let artifact = &profile.artifact;
    let tools = resolved_artifact_tools(artifact).unwrap_or_default();
    let allow_read = artifact.path_policy.allow_read.clone().unwrap_or_default();
    let allow_write = artifact.path_policy.allow_write.clone().unwrap_or_default();
    let deny = artifact.path_policy.deny.clone().unwrap_or_default();
    let blocked = artifact
        .blocked_action_message
        .as_deref()
        .and_then(|message| clean_opt(Some(message)))
        .unwrap_or("This action is outside the selected Script Kit profile.");

    format!(
        "[Script Kit profile contract]\nProfile id: plugin:{}/{}\nAllowed tools: {}\nAllowed read paths: {}\nAllowed write paths: {}\nDenied paths: {}\nIf the user requests work outside this contract, refuse briefly and say: \"{}\"",
        profile.plugin_id,
        profile.profile_id,
        format_policy_list(&tools),
        format_policy_list(&allow_read),
        format_policy_list(&allow_write),
        format_policy_list(&deny),
        blocked
    )
}

fn format_policy_list(values: &[String]) -> String {
    let cleaned = clean_list(values);
    if cleaned.is_empty() {
        "none".to_string()
    } else {
        cleaned.join(", ")
    }
}

fn resolved_profile_tools(profile: &AgentChatProfile) -> Option<Vec<String>> {
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
    if profile.pi_binary.is_none() {
        profile.pi_binary = clean_opt(ai.pi_binary.as_deref())
            .map(crate::ai::agent_chat::pi::binary::expand_tilde_path)
            .or_else(crate::ai::agent_chat::pi::binary::default_pi_binary);
    }

    if let Some(selected_model) = clean_opt(ai.selected_model_id.as_deref()) {
        if let Some((provider, model)) = parse_provider_model_selection(selected_model) {
            profile.provider = Some(provider);
            profile.model = Some(model);
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

#[cfg(test)]
mod built_in_profile_tests {
    use super::*;

    #[test]
    fn only_brain_profile_prompt_contract_mentions_brain_recall() {
        let ctx = AgentChatProfileContext {
            kit_path: std::path::PathBuf::from("/tmp/script-kit-profile-test"),
        };
        let brain = built_in_brain_profile(&ctx);
        let general = built_in_general_profile(&ctx);
        let script_kit = built_in_script_kit_profile(&ctx);
        let text = built_in_text_profile(&ctx);

        assert!(
            brain
                .append_system_prompt
                .as_deref()
                .is_some_and(|prompt| prompt.contains("Brain recall")),
            "Brain profile must describe the recall block it receives"
        );
        for profile in [general, script_kit, text] {
            assert!(
                !profile
                    .append_system_prompt
                    .as_deref()
                    .unwrap_or_default()
                    .contains("Brain recall"),
                "{} profile must not claim a Brain recall contract",
                profile.name
            );
        }
    }
}

#[cfg(test)]
mod plugin_profile_cache_tests {
    use super::*;
    use std::path::Path;
    use std::time::{Duration, Instant};

    fn entry(
        plugins_dir: &Path,
        kit_path: &Path,
        refreshed_at: Instant,
    ) -> PluginProfileCacheEntry {
        PluginProfileCacheEntry {
            plugins_dir: plugins_dir.to_path_buf(),
            kit_path: kit_path.to_path_buf(),
            refreshed_at,
            profiles: Vec::new(),
        }
    }

    /// Profile Search navigation and the Agent Chat profile picker resolve
    /// plugin profiles several times per keystroke; a fresh same-key entry
    /// must be served from memory instead of re-walking `<kit>/plugins/**`.
    #[test]
    fn fresh_same_key_entry_hits() {
        let now = Instant::now();
        let e = entry(Path::new("/kit/plugins"), Path::new("/kit"), now);
        assert!(plugin_profile_cache_entry_is_fresh(
            &e,
            Path::new("/kit/plugins"),
            Path::new("/kit"),
            now + Duration::from_millis(50),
        ));
    }

    #[test]
    fn expired_entry_misses() {
        let now = Instant::now();
        let e = entry(Path::new("/kit/plugins"), Path::new("/kit"), now);
        assert!(!plugin_profile_cache_entry_is_fresh(
            &e,
            Path::new("/kit/plugins"),
            Path::new("/kit"),
            now + PLUGIN_PROFILE_CACHE_TTL,
        ));
    }

    #[test]
    fn different_kit_path_misses() {
        let now = Instant::now();
        let e = entry(Path::new("/kit/plugins"), Path::new("/kit"), now);
        assert!(!plugin_profile_cache_entry_is_fresh(
            &e,
            Path::new("/kit/plugins"),
            Path::new("/other-kit"),
            now,
        ));
    }

    #[test]
    fn different_plugins_dir_misses() {
        let now = Instant::now();
        let e = entry(Path::new("/kit/plugins"), Path::new("/kit"), now);
        assert!(!plugin_profile_cache_entry_is_fresh(
            &e,
            Path::new("/other-kit/plugins"),
            Path::new("/kit"),
            now,
        ));
    }

    #[test]
    fn invalidate_clears_cache() {
        if let Ok(mut cache) = PLUGIN_PROFILE_CACHE.lock() {
            *cache = Some(entry(
                Path::new("/kit/plugins"),
                Path::new("/kit"),
                Instant::now(),
            ));
        }
        invalidate_plugin_profile_cache();
        assert!(PLUGIN_PROFILE_CACHE.lock().unwrap().is_none());
    }
}
