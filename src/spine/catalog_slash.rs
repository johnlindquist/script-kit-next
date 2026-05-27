use std::ops::Range;

use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};

struct SlashCommandSpec {
    name: &'static str,
    description: &'static str,
    icon: &'static str,
}

const SPINE_SLASH_COMMANDS: &[SlashCommandSpec] = &[
    SlashCommandSpec {
        name: "rewrite",
        description: "Rewrite the prompt or selected context",
        icon: "pencil",
    },
    SlashCommandSpec {
        name: "summarize",
        description: "Summarize attached context",
        icon: "align-left",
    },
    SlashCommandSpec {
        name: "explain",
        description: "Explain the attached context",
        icon: "book-open",
    },
    SlashCommandSpec {
        name: "fix",
        description: "Fix grammar, code, or formatting",
        icon: "wrench",
    },
    SlashCommandSpec {
        name: "translate",
        description: "Translate the text",
        icon: "languages",
    },
    SlashCommandSpec {
        name: "review",
        description: "Review code changes",
        icon: "git-pull-request",
    },
];

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

pub(super) fn build_slash_command_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    SPINE_SLASH_COMMANDS
        .iter()
        .enumerate()
        .filter(|(_, spec)| {
            let slash_text = format!("/{}", spec.name);
            matches_query(spec.name, query)
                || matches_query(&slash_text, query)
                || matches_query(spec.description, query)
        })
        .map(|(rank, spec)| {
            let replacement = format!("/{}", spec.name);
            let capitalized = capitalize(spec.name);
            SpineListRow {
                id: ss(format!("spine:/:{}", spec.name)),
                kind: SpineListRowKind::SlashCommand {
                    command: ss(spec.name),
                },
                title: ss(capitalized),
                subtitle: Some(ss(spec.description)),
                meta: None,
                icon: Some(ss(spec.icon)),
                badges: vec![ss("/")],
                score: i32::MAX.saturating_sub(rank as i32),
                is_selectable: true,
                action_label: Some(ss("Insert")),
                action: SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range: segment_byte_range.clone(),
                    replacement: ss(replacement.clone()),
                    resolution_id: ss(format!("default:{}", spec.name)),
                    resolution_label: ss(replacement),
                    resolution_source: ss("slash-command-default"),
                    trailing_space: true,
                },
            }
        })
        .collect()
}
