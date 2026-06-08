//! Safe, compact result-card derivation for Agent Chat assistant output.
//!
//! This module is intentionally pure. It only recognizes explicit artifacts
//! and follow-up prompts from completed assistant text; UI rendering and action
//! dispatch stay in Agent Chat surfaces.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RESULT_CARD_MAX_ARTIFACTS: usize = 3;
pub const RESULT_CARD_MAX_FOLLOW_UPS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatResultCards {
    pub artifacts: Vec<AgentChatResultArtifact>,
    pub follow_ups: Vec<AgentChatResultFollowUp>,
}

impl AgentChatResultCards {
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty() && self.follow_ups.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatResultArtifact {
    pub kind: AgentChatResultArtifactKind,
    pub title: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentChatResultArtifactKind {
    File,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatResultFollowUp {
    pub label: String,
    pub prompt: String,
}

pub fn derive_agent_chat_result_cards_from_assistant_message(body: &str) -> AgentChatResultCards {
    let mut artifacts = Vec::new();
    let mut seen_artifacts = HashSet::new();

    collect_markdown_links(body, &mut artifacts, &mut seen_artifacts);
    collect_plain_absolute_paths(body, &mut artifacts, &mut seen_artifacts);

    AgentChatResultCards {
        artifacts,
        follow_ups: collect_next_actions(body),
    }
}

fn collect_markdown_links(
    body: &str,
    artifacts: &mut Vec<AgentChatResultArtifact>,
    seen: &mut HashSet<String>,
) {
    for line in body.lines() {
        let mut remaining = line;
        while let Some(label_start) = remaining.find('[') {
            let after_label_start = &remaining[label_start + 1..];
            let Some(label_end) = after_label_start.find(']') else {
                break;
            };
            let label = &after_label_start[..label_end];
            let after_label = &after_label_start[label_end + 1..];
            if !after_label.starts_with('(') {
                remaining = after_label;
                continue;
            }
            let after_target_start = &after_label[1..];
            let Some(target_end) = after_target_start.find(')') else {
                break;
            };
            let target = after_target_start[..target_end].trim();
            push_artifact(label, target, artifacts, seen);
            remaining = &after_target_start[target_end + 1..];
            if artifacts.len() >= RESULT_CARD_MAX_ARTIFACTS {
                return;
            }
        }
    }
}

fn collect_plain_absolute_paths(
    body: &str,
    artifacts: &mut Vec<AgentChatResultArtifact>,
    seen: &mut HashSet<String>,
) {
    if artifacts.len() >= RESULT_CARD_MAX_ARTIFACTS {
        return;
    }

    for token in body.split_whitespace() {
        let candidate = token
            .trim_matches(|ch: char| matches!(ch, ',' | '.' | ';' | ':' | ')' | '(' | '"' | '\''));
        if candidate.starts_with('/') {
            push_artifact("", candidate, artifacts, seen);
            if artifacts.len() >= RESULT_CARD_MAX_ARTIFACTS {
                return;
            }
        }
    }
}

fn push_artifact(
    label: &str,
    target: &str,
    artifacts: &mut Vec<AgentChatResultArtifact>,
    seen: &mut HashSet<String>,
) {
    if artifacts.len() >= RESULT_CARD_MAX_ARTIFACTS {
        return;
    }

    if let Some(link) = normalize_http_link(target) {
        if seen.insert(link.clone()) {
            artifacts.push(AgentChatResultArtifact {
                kind: AgentChatResultArtifactKind::Link,
                title: sanitized_title(label, &link),
                target: link,
            });
        }
        return;
    }

    if let Some(path) = normalize_existing_absolute_path(target) {
        let target = path.to_string_lossy().to_string();
        if seen.insert(target.clone()) {
            artifacts.push(AgentChatResultArtifact {
                kind: AgentChatResultArtifactKind::File,
                title: sanitized_title(label, &target),
                target,
            });
        }
    }
}

fn normalize_http_link(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn normalize_existing_absolute_path(target: &str) -> Option<PathBuf> {
    let path = Path::new(target);
    if !path.is_absolute() {
        return None;
    }
    path.canonicalize()
        .ok()
        .filter(|canonical| canonical.exists())
}

fn collect_next_actions(body: &str) -> Vec<AgentChatResultFollowUp> {
    let mut follow_ups = Vec::new();
    let mut in_section = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_section {
                break;
            }
            continue;
        }

        if is_next_actions_heading(trimmed) {
            in_section = true;
            continue;
        }

        if !in_section {
            continue;
        }

        if looks_like_heading(trimmed) {
            break;
        }

        if let Some(prompt) = bullet_text(trimmed) {
            if prompt_contains_reserved_action_id(prompt) {
                continue;
            }
            let prompt = sanitize_inline(prompt);
            if prompt.is_empty() {
                continue;
            }
            let label = truncate_chars(&prompt, 48);
            follow_ups.push(AgentChatResultFollowUp { label, prompt });
            if follow_ups.len() >= RESULT_CARD_MAX_FOLLOW_UPS {
                break;
            }
        }
    }

    follow_ups
}

fn is_next_actions_heading(line: &str) -> bool {
    let normalized = line
        .trim_matches(|ch: char| matches!(ch, '#' | ':' | '*'))
        .trim()
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "next_actions" | "next actions" | "follow ups" | "follow-ups"
    )
}

fn looks_like_heading(line: &str) -> bool {
    line.starts_with('#') || (line.ends_with(':') && bullet_text(line).is_none())
}

fn bullet_text(line: &str) -> Option<&str> {
    for prefix in ["- ", "* ", "1. ", "2. ", "3. ", "4. "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim());
        }
    }
    None
}

fn prompt_contains_reserved_action_id(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    lower.contains("sdk:") || lower.contains("action:")
}

fn sanitized_title(label: &str, fallback: &str) -> String {
    let candidate = sanitize_inline(label);
    if candidate.is_empty() {
        fallback_title(fallback)
    } else {
        truncate_chars(&candidate, 80)
    }
}

fn fallback_title(target: &str) -> String {
    Path::new(target)
        .file_name()
        .and_then(|name| name.to_str())
        .map(sanitize_inline)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| truncate_chars(target, 80))
}

fn sanitize_inline(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
