use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ChoiceDisplayMetadata {
    pub(super) description: Option<String>,
    pub(super) item_type: Option<String>,
    pub(super) shortcut: Option<String>,
    pub(super) last_run: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct SelectChoiceIndex {
    pub(super) metadata: ChoiceDisplayMetadata,
    pub(super) name_lower: String,
    pub(super) description_lower: String,
    pub(super) value_lower: String,
    pub(super) item_type_lower: String,
    pub(super) last_run_lower: String,
    pub(super) shortcut_lower: String,
    pub(super) stable_semantic_id: String,
}

impl SelectChoiceIndex {
    pub(super) fn from_choice(choice: &Choice, source_index: usize) -> Self {
        let metadata = ChoiceDisplayMetadata::from_choice(choice);

        SelectChoiceIndex {
            name_lower: choice.name.to_lowercase(),
            description_lower: choice
                .description
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            value_lower: choice.value.to_lowercase(),
            item_type_lower: metadata
                .item_type
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            last_run_lower: metadata
                .last_run
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            shortcut_lower: metadata
                .shortcut
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            stable_semantic_id: fallback_select_semantic_id(source_index, &choice.value),
            metadata,
        }
    }
}

impl ChoiceDisplayMetadata {
    pub(super) fn from_choice(choice: &Choice) -> Self {
        let mut metadata = Self::default();
        let mut description_parts = Vec::new();

        if let Some(description) = choice.description.as_deref() {
            for token in description
                .split(['•', '|', '\n'])
                .map(str::trim)
                .filter(|token| !token.is_empty())
            {
                if metadata.shortcut.is_none() {
                    if let Some(shortcut) = extract_shortcut_token(token) {
                        metadata.shortcut = Some(shortcut);
                        continue;
                    }
                }

                if metadata.item_type.is_none() {
                    if let Some(item_type) = extract_script_type_token(token) {
                        metadata.item_type = Some(item_type);
                        continue;
                    }
                }

                if metadata.last_run.is_none() {
                    if let Some(last_run) = extract_last_run_token(token) {
                        metadata.last_run = Some(last_run);
                        continue;
                    }
                }

                description_parts.push(token.to_string());
            }
        }

        if !description_parts.is_empty() {
            metadata.description = Some(description_parts.join(" • "));
        }

        if metadata.item_type.is_none() {
            metadata.item_type = infer_script_type(choice);
        }

        metadata
    }

    pub(super) fn subtitle_text(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(description) = self.description.as_deref() {
            if !description.is_empty() {
                parts.push(description.to_string());
            }
        }

        let mut metadata_parts = Vec::new();
        if let Some(item_type) = self.item_type.as_deref() {
            metadata_parts.push(item_type.to_string());
        }
        if let Some(last_run) = self.last_run.as_deref() {
            metadata_parts.push(last_run.to_string());
        }

        if !metadata_parts.is_empty() {
            parts.push(metadata_parts.join(" · "));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" • "))
        }
    }
}

fn infer_script_type(choice: &Choice) -> Option<String> {
    let name_lower = choice.name.to_lowercase();
    let value_lower = choice.value.to_lowercase();
    let description_lower = choice
        .description
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    let combined = format!("{} {} {}", name_lower, description_lower, value_lower);

    if combined.contains("scriptlet")
        || value_lower.contains(".md#")
        || value_lower.contains("/snippets/")
    {
        return Some("Scriptlet".to_string());
    }

    if combined.contains("extension")
        || value_lower.contains("/extensions/")
        || value_lower.contains("/extension/")
    {
        return Some("Extension".to_string());
    }

    if combined.contains("agent") {
        return Some("Agent".to_string());
    }

    let script_extensions = [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".sh", ".py", ".rb", ".ps1", ".zsh", ".bash",
    ];
    if combined.contains("script")
        || script_extensions
            .iter()
            .any(|ext| value_lower.ends_with(ext) || value_lower.contains(&format!("{ext}#")))
    {
        return Some("Script".to_string());
    }

    None
}

fn looks_like_shortcut(token: &str) -> bool {
    let lower = token.to_lowercase();
    if token.len() > 28 || token.is_empty() {
        return false;
    }

    let has_modifier = [
        "cmd", "command", "ctrl", "control", "alt", "option", "shift", "meta", "⌘", "⌃", "⌥", "⇧",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    let has_key_like = token.chars().any(|ch| ch.is_ascii_alphanumeric())
        || token.contains('↵')
        || token.contains('⌫')
        || token.contains('↑')
        || token.contains('↓')
        || token.contains('←')
        || token.contains('→');

    has_modifier && has_key_like
}

fn normalize_shortcut_label(raw: &str) -> String {
    if raw.chars().any(|ch| "⌘⌥⌃⇧↵⌫↑↓←→".contains(ch)) {
        return raw.trim().replace(' ', "");
    }

    let mut normalized = raw.to_lowercase();
    normalized = normalized
        .replace("command", "cmd")
        .replace("control", "ctrl")
        .replace("option", "alt");

    normalized
        .split(['+', '-', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| match part {
            "cmd" | "meta" => "⌘".to_string(),
            "ctrl" => "⌃".to_string(),
            "alt" | "opt" => "⌥".to_string(),
            "shift" => "⇧".to_string(),
            "enter" | "return" => "↵".to_string(),
            "delete" | "backspace" => "⌫".to_string(),
            "up" | "arrowup" => "↑".to_string(),
            "down" | "arrowdown" => "↓".to_string(),
            "left" | "arrowleft" => "←".to_string(),
            "right" | "arrowright" => "→".to_string(),
            _ => part.to_ascii_uppercase(),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_shortcut_token(token: &str) -> Option<String> {
    let lower = token.to_lowercase();

    if lower.starts_with("shortcut")
        || lower.starts_with("key")
        || lower.starts_with("hotkey")
        || lower.starts_with("shortcut ")
    {
        let shortcut_value = token
            .split_once(':')
            .or_else(|| token.split_once('='))
            .map(|(_, value)| value.trim())
            .unwrap_or_default();
        if !shortcut_value.is_empty() {
            return Some(normalize_shortcut_label(shortcut_value));
        }
    }

    if looks_like_shortcut(token) {
        return Some(normalize_shortcut_label(token));
    }

    None
}

fn extract_script_type_token(token: &str) -> Option<String> {
    let lower = token.trim().to_lowercase();
    if lower == "script" || lower.starts_with("type: script") {
        return Some("Script".to_string());
    }
    if lower == "scriptlet" || lower.starts_with("type: scriptlet") {
        return Some("Scriptlet".to_string());
    }
    if lower == "extension" || lower.starts_with("type: extension") {
        return Some("Extension".to_string());
    }
    if lower == "agent" || lower.starts_with("type: agent") {
        return Some("Agent".to_string());
    }
    None
}

fn extract_last_run_token(token: &str) -> Option<String> {
    let trimmed = token.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("last run") || lower.starts_with("last ran") {
        return Some(trimmed.to_string());
    }
    if (lower.starts_with("ran ") || lower.contains(" ago"))
        && (lower.contains("run") || lower.contains("ran"))
    {
        return Some(trimmed.to_string());
    }
    None
}
pub(super) fn fallback_select_semantic_id(source_index: usize, value: &str) -> String {
    generate_semantic_id("select", source_index, value)
}
