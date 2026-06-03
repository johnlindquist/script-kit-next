use crate::ai::agent_chat::profiles::{
    AgentChatProfileContext, AgentChatProfileSource, ResolvedAgentChatProfile,
};
use crate::config::{AgentChatBackend, AiPreferences};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileSearchResult {
    pub profile: ResolvedAgentChatProfile,
    pub selected: bool,
}

pub(crate) fn profile_search_results(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
    query: &str,
) -> Vec<ProfileSearchResult> {
    let selected_id =
        crate::ai::agent_chat::profiles::selected_agent_chat_profile_picker_id(ai, ctx);
    let query_lower = query.trim().to_ascii_lowercase();
    crate::ai::agent_chat::profiles::resolved_agent_chat_profile_picker_profiles(ai, ctx)
        .into_iter()
        .filter(|profile| {
            if query_lower.is_empty() {
                return true;
            }
            profile.name.to_ascii_lowercase().contains(&query_lower)
                || profile.id.to_ascii_lowercase().contains(&query_lower)
                || profile
                    .model
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&query_lower)
                || profile
                    .provider
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&query_lower)
        })
        .map(|profile| {
            let selected = profile.id == selected_id;
            ProfileSearchResult { profile, selected }
        })
        .collect()
}

pub(crate) fn persist_profile_search_selection(profile_id: &str) -> bool {
    let mut prefs = crate::config::load_user_preferences();
    let ctx = AgentChatProfileContext::from_setup();
    let found = crate::ai::agent_chat::profiles::persist_agent_chat_profile_selection(
        &mut prefs.ai,
        profile_id,
        &ctx,
    )
    .is_some();
    let persisted = found && crate::config::save_user_preferences(&prefs).is_ok();
    tracing::info!(
        target: "script_kit::spine",
        event = "profile_search_profile_persisted",
        profile_id,
        persisted,
        "Profile Search selection persisted"
    );
    persisted
}

pub(crate) fn source_label(source: AgentChatProfileSource) -> &'static str {
    match source {
        AgentChatProfileSource::BuiltIn => "Built-in",
        AgentChatProfileSource::User => "User",
        AgentChatProfileSource::Plugin => "Plugin",
    }
}

pub(crate) fn backend_label(backend: AgentChatBackend) -> &'static str {
    match backend {
        AgentChatBackend::Pi => "Pi",
    }
}

pub(crate) fn profile_model_label(profile: &ResolvedAgentChatProfile) -> String {
    match (profile.provider.as_deref(), profile.model.as_deref()) {
        (Some(provider), Some(model)) => format!("{provider} / {model}"),
        (Some(provider), None) => provider.to_string(),
        (None, Some(model)) => model.to_string(),
        (None, None) => "Default model".to_string(),
    }
}

pub(crate) fn profile_tools_label(profile: &ResolvedAgentChatProfile) -> String {
    profile
        .tools
        .as_ref()
        .filter(|tools| !tools.is_empty())
        .map(|tools| tools.join(", "))
        .unwrap_or_else(|| "No explicit tools".to_string())
}

pub(crate) fn profile_prompt_summary(profile: &ResolvedAgentChatProfile) -> String {
    let prompt = profile
        .system_prompt
        .as_deref()
        .or(profile.append_system_prompt.as_deref())
        .unwrap_or("No custom prompt text");
    const LIMIT: usize = 360;
    if prompt.len() <= LIMIT {
        prompt.to_string()
    } else {
        format!("{}...", prompt.chars().take(LIMIT).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_search_finds_builtin_profiles_by_name_and_id() {
        let prefs = AiPreferences::default();
        let ctx = AgentChatProfileContext::from_setup();

        let text = profile_search_results(&prefs, &ctx, "text");
        assert!(text.iter().any(|result| result.profile.id == "text"));

        let script_kit = profile_search_results(&prefs, &ctx, "script-kit");
        assert!(script_kit
            .iter()
            .any(|result| result.profile.id == "script-kit"));
    }

    #[test]
    fn profile_search_preview_labels_expose_model_and_tools() {
        let prefs = AiPreferences::default();
        let ctx = AgentChatProfileContext::from_setup();
        let result = profile_search_results(&prefs, &ctx, "general")
            .into_iter()
            .find(|result| result.profile.id == "general")
            .expect("general profile should exist");

        assert!(profile_model_label(&result.profile).contains("openai-codex"));
        assert!(profile_tools_label(&result.profile).contains("web_search"));
        assert!(!profile_prompt_summary(&result.profile).is_empty());
    }
}
