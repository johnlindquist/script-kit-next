use crate::ai::agent_chat::profiles::{
    AgentChatProfileContext, AgentChatProfileSource, ResolvedAgentChatProfile,
};
use crate::config::{AgentChatBackend, AiPreferences};

/// Synthetic action row id: Enter on it writes a fresh markdown profile from
/// the mdflow template and opens it in the editor.
pub(crate) const CREATE_PROFILE_ROW_ID: &str = "create-new-profile";

/// Trailing "Create New Profile…" action row. It reuses the profile row
/// shape so the list/preview machinery needs no special cases; the preview's
/// Instructions section explains what Enter does.
fn create_profile_action_row(ctx: &AgentChatProfileContext) -> ResolvedAgentChatProfile {
    ResolvedAgentChatProfile {
        source: AgentChatProfileSource::Mdflow,
        id: CREATE_PROFILE_ROW_ID.to_string(),
        name: "Create New Profile…".to_string(),
        icon_name: Some("plus".to_string()),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider: None,
        model: None,
        system_prompt: None,
        append_system_prompt: Some(format!(
            "Creates a starter profile at {} and opens it in your editor. Profiles are single markdown files (mdflow format): YAML frontmatter for the model and tools, body for the instructions.",
            crate::ai::agent_chat::mdflow_profiles::mdflow_profiles_dir(ctx)
                .join("my-profile.md")
                .display()
        )),
        cwd: None,
        tools: None,
        tool_policy: None,
        path_policy: None,
        blocked_action_message: None,
        disable_extensions: None,
        disable_skills: None,
        disable_prompt_templates: None,
        disable_context_files: None,
        hide_cwd_in_prompt: None,
        thinking: None,
        extension_policy: None,
        session_dir: None,
        no_session: None,
        session_durability: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileSearchResult {
    pub profile: ResolvedAgentChatProfile,
    pub selected: bool,
    /// True when this profile is the one Quick AI (launcher Tab-with-text)
    /// currently launches with.
    pub quick_ai: bool,
    pub name_highlight_indices: Option<Vec<usize>>,
    pub description_highlight_indices: Option<Vec<usize>>,
}

pub(crate) fn profile_search_results(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
    query: &str,
) -> Vec<ProfileSearchResult> {
    let selected_id =
        crate::ai::agent_chat::profiles::selected_agent_chat_profile_picker_id(ai, ctx);
    let quick_ai_id = crate::ai::agent_chat::profiles::resolve_quick_ai_profile(ai, ctx).id;
    let query = query.trim();
    let query_lower = query.to_ascii_lowercase();
    // The built-in Quick AI profile leads the list: it isn't a pickable Agent
    // Chat default, but Tab ("Use for Quick AI") needs it as the way back to
    // the fast zero-context default.
    std::iter::once(crate::ai::agent_chat::profiles::built_in_quick_ai_profile(
        ctx,
    ))
    .chain(crate::ai::agent_chat::profiles::resolved_agent_chat_profile_picker_profiles(ai, ctx))
    .chain(std::iter::once(create_profile_action_row(ctx)))
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
        let quick_ai = profile.id == quick_ai_id;
        let description = profile_search_result_description(&profile);
        let (_, name_indices) = crate::scripts::search::highlight_indices_for(query, &profile.name);
        let (_, description_indices) =
            crate::scripts::search::highlight_indices_for(query, &description);
        ProfileSearchResult {
            profile,
            selected,
            quick_ai,
            name_highlight_indices: non_empty_indices(name_indices),
            description_highlight_indices: non_empty_indices(description_indices),
        }
    })
    .collect()
}

fn non_empty_indices(indices: Vec<usize>) -> Option<Vec<usize>> {
    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

pub(crate) fn profile_search_result_description(profile: &ResolvedAgentChatProfile) -> String {
    format!(
        "{} · {}",
        source_label(profile.source),
        profile_model_label(profile)
    )
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

/// Persist the profile Quick AI launches with (Tab = "Use for Quick AI" in
/// Profile Search). Accepts the built-in `quick-ai` id to restore the fast
/// zero-context default.
pub(crate) fn persist_quick_ai_profile_search_selection(profile_id: &str) -> bool {
    let mut prefs = crate::config::load_user_preferences();
    let ctx = AgentChatProfileContext::from_setup();
    let found = crate::ai::agent_chat::profiles::persist_quick_ai_profile_selection(
        &mut prefs.ai,
        profile_id,
        &ctx,
    )
    .is_some();
    let persisted = found && crate::config::save_user_preferences(&prefs).is_ok();
    tracing::info!(
        target: "script_kit::spine",
        event = "profile_search_quick_ai_profile_persisted",
        profile_id,
        persisted,
        "Quick AI profile selection persisted"
    );
    persisted
}

pub(crate) fn source_label(source: AgentChatProfileSource) -> &'static str {
    match source {
        AgentChatProfileSource::BuiltIn => "Built-in",
        AgentChatProfileSource::User => "User",
        AgentChatProfileSource::Mdflow => "Markdown",
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

pub(crate) fn profile_preview_explanation() -> &'static str {
    "Profiles define the instructions, model/provider, tools, and working directory used by Agent Chat when starting new conversations."
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

    #[test]
    fn profile_search_highlight_indices_match_name_and_description() {
        let prefs = AiPreferences::default();
        let ctx = AgentChatProfileContext::from_setup();

        let text = profile_search_results(&prefs, &ctx, "tex")
            .into_iter()
            .find(|result| result.profile.id == "text")
            .expect("text profile should match by name");
        assert_eq!(text.name_highlight_indices, Some(vec![0, 1, 2]));

        let codex = profile_search_results(&prefs, &ctx, "codex")
            .into_iter()
            .find(|result| result.profile.id == "general")
            .expect("general profile should match by model description");
        assert!(codex.description_highlight_indices.is_some());
    }

    #[test]
    fn profile_search_leads_with_quick_ai_row_marked_as_quick_ai_default() {
        let prefs = AiPreferences::default();
        let ctx = AgentChatProfileContext::from_setup();

        let results = profile_search_results(&prefs, &ctx, "");
        let first = results.first().expect("profile search must have rows");
        assert_eq!(first.profile.id, "quick-ai");
        assert!(
            first.quick_ai,
            "quick-ai row is the default Quick AI target"
        );
        assert!(
            !first.selected,
            "quick-ai must never read as the Agent Chat default"
        );
        // Exactly one row carries the Quick AI marker.
        assert_eq!(results.iter().filter(|result| result.quick_ai).count(), 1);
    }

    #[test]
    fn profile_search_marks_picked_quick_ai_profile() {
        let prefs = AiPreferences {
            quick_ai_profile_id: Some("text".to_string()),
            ..AiPreferences::default()
        };
        let ctx = AgentChatProfileContext::from_setup();

        let results = profile_search_results(&prefs, &ctx, "");
        let text = results
            .iter()
            .find(|result| result.profile.id == "text")
            .expect("text profile row");
        assert!(text.quick_ai);
        let quick_ai_row = results
            .iter()
            .find(|result| result.profile.id == "quick-ai")
            .expect("quick-ai row");
        assert!(!quick_ai_row.quick_ai);
    }

    #[test]
    fn profile_search_ends_with_create_profile_action_row() {
        let prefs = AiPreferences::default();
        let ctx = AgentChatProfileContext::from_setup();

        let results = profile_search_results(&prefs, &ctx, "");
        let last = results.last().expect("profile search must have rows");
        assert_eq!(last.profile.id, CREATE_PROFILE_ROW_ID);
        assert!(!last.selected);
        assert!(!last.quick_ai);

        // It stays reachable by search.
        let filtered = profile_search_results(&prefs, &ctx, "create");
        assert!(filtered
            .iter()
            .any(|result| result.profile.id == CREATE_PROFILE_ROW_ID));
    }

    #[test]
    fn profile_search_preview_explanation_is_structured_copy() {
        assert!(profile_preview_explanation().contains("Profiles define"));
        assert!(profile_preview_explanation().contains("working directory"));
    }
}
