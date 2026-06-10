use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};
use std::ops::Range;

fn profile_source_label(
    source: crate::ai::agent_chat::profiles::AgentChatProfileSource,
) -> &'static str {
    match source {
        crate::ai::agent_chat::profiles::AgentChatProfileSource::BuiltIn => "Built-in",
        crate::ai::agent_chat::profiles::AgentChatProfileSource::User => "Custom",
        crate::ai::agent_chat::profiles::AgentChatProfileSource::Plugin => "Plugin",
    }
}

pub(super) fn build_profile_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    let prefs = crate::config::load_user_preferences();
    let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
    let selected_id =
        crate::ai::agent_chat::profiles::selected_agent_chat_profile_picker_id(&prefs.ai, &ctx);

    crate::ai::agent_chat::profiles::agent_chat_profile_picker_entries(&prefs.ai, &ctx)
        .into_iter()
        .enumerate()
        .filter(|(_, entry)| {
            let pipe_text = format!("|{}", entry.id);
            matches_query(&entry.id, query)
                || matches_query(&entry.name, query)
                || matches_query(&pipe_text, query)
        })
        .map(|(rank, entry)| {
            let replacement = format!("|{}", entry.id);
            let source = profile_source_label(entry.source);
            let selected = entry.id == selected_id;
            let title = if selected {
                format!("{} \u{2713}", entry.name)
            } else {
                entry.name.clone()
            };
            let subtitle = if selected {
                format!("Current Agent Chat profile · {source} · Pi")
            } else {
                format!(
                    "Switch to this profile in a new chat · {source} · Pi · Starts a new chat when a conversation is already active"
                )
            };
            SpineListRow {
                id: ss(format!("spine:|:{}", entry.id)),
                kind: SpineListRowKind::Profile {
                    profile_id: ss(entry.id.clone()),
                },
                title: ss(title),
                subtitle: Some(ss(subtitle)),
                meta: Some(ss(format!("|{}", entry.id))),
                icon: Some(ss(entry
                    .icon_name
                    .unwrap_or_else(|| "user-round".to_string()))),
                badges: vec![ss("|")],
                score: i32::MAX.saturating_sub(rank as i32),
                is_selectable: true,
                action_label: None,
                action: SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range: segment_byte_range.clone(),
                    replacement: ss(replacement.clone()),
                    resolution_id: ss(entry.id),
                    resolution_label: ss(replacement),
                    resolution_source: ss("profile"),
                    trailing_space: true,
                },
            }
        })
        .collect()
}
