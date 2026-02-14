/// Map a file extension to a human-readable language/tool name.
/// Used as a last-resort fallback description for scripts with no other context.
pub(crate) fn extension_language_label(extension: &str) -> Option<&'static str> {
    match extension {
        "ts" | "tsx" => Some("TypeScript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("JavaScript"),
        "sh" | "bash" => Some("Shell script"),
        "zsh" => Some("Zsh script"),
        "py" => Some("Python script"),
        "rb" => Some("Ruby script"),
        "applescript" | "scpt" => Some("AppleScript"),
        _ => None,
    }
}

fn truncate_str_chars(s: &str, max_chars: usize) -> &str {
    s.char_indices()
        .nth(max_chars)
        .map_or(s, |(index, _)| &s[..index])
}

/// Auto-generate a fallback description for scripts that have no explicit description.
/// Priority: schedule expression > cron expression > watch pattern > background > system > filename
pub(crate) fn auto_description_for_script(script: &crate::scripts::Script) -> Option<String> {
    // If the script has an explicit description, return it as-is
    if script.description.is_some() {
        return script.description.clone();
    }

    // Try metadata-based descriptions
    if let Some(ref meta) = script.typed_metadata {
        if let Some(ref schedule) = meta.schedule {
            return Some(format!("Scheduled: {}", schedule));
        }
        if let Some(ref cron) = meta.cron {
            return Some(format!("Cron: {}", cron));
        }
        if let Some(first_pattern) = meta.watch.first() {
            let display = if first_pattern.chars().count() > 40 {
                format!("{}...", truncate_str_chars(first_pattern, 37))
            } else {
                first_pattern.clone()
            };
            return Some(format!("Watches: {}", display));
        }
        if meta.background {
            return Some("Background process".to_string());
        }
        if meta.system {
            return Some("System event handler".to_string());
        }
    }

    // Fallback: show filename when it differs from the display name
    let filename = crate::scripts::search::extract_filename(&script.path);
    if !filename.is_empty() && filename != script.name {
        Some(filename)
    } else {
        // Last resort: show language name based on extension
        extension_language_label(&script.extension).map(|s| s.to_string())
    }
}

/// Determine the grouped-view source hint for a script.
/// Priority: alias (when shortcut is badge) > tags > kit name (non-main)
pub(crate) fn grouped_view_hint_for_script(script: &crate::scripts::Script) -> Option<String> {
    if script.shortcut.is_some() {
        // Shortcut is the badge -> show alias as trigger hint, then tags, then kit
        script
            .alias
            .as_ref()
            .map(|a| format!("/{}", a))
            .or_else(|| {
                script.typed_metadata.as_ref().and_then(|meta| {
                    if !meta.tags.is_empty() {
                        Some(
                            meta.tags
                                .iter()
                                .take(2)
                                .map(|t| t.as_str())
                                .collect::<Vec<_>>()
                                .join(" · "),
                        )
                    } else {
                        None
                    }
                })
            })
    } else if script.alias.is_some() {
        // Alias is the badge -> show tags, then kit
        script
            .typed_metadata
            .as_ref()
            .and_then(|meta| {
                if !meta.tags.is_empty() {
                    Some(
                        meta.tags
                            .iter()
                            .take(2)
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                } else {
                    None
                }
            })
            .or_else(|| {
                script
                    .kit_name
                    .as_deref()
                    .filter(|k| *k != "main")
                    .map(|k| k.to_string())
            })
    } else {
        // No badge -> show tags, then kit name, then custom enter text as action hint
        script
            .typed_metadata
            .as_ref()
            .and_then(|meta| {
                if !meta.tags.is_empty() {
                    Some(
                        meta.tags
                            .iter()
                            .take(2)
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                } else {
                    None
                }
            })
            .or_else(|| {
                script
                    .kit_name
                    .as_deref()
                    .filter(|k| *k != "main")
                    .map(|k| k.to_string())
            })
            .or_else(|| {
                // Final fallback: custom enter text as action hint (e.g., "-> Execute")
                script
                    .typed_metadata
                    .as_ref()
                    .and_then(|m| m.enter.as_deref())
                    .filter(|e| *e != "Run" && *e != "Run Script")
                    .map(|e| format!("→ {}", e))
            })
    }
}

/// Determine the grouped-view source hint for a scriptlet.
/// Priority: hidden trigger keyword/alias > group name (non-main)
pub(crate) fn grouped_view_hint_for_scriptlet(
    scriptlet: &crate::scripts::Scriptlet,
) -> Option<String> {
    if scriptlet.shortcut.is_some() {
        scriptlet
            .keyword
            .as_ref()
            .or(scriptlet.alias.as_ref())
            .map(|k| format!("/{}", k))
    } else if scriptlet.keyword.is_some() {
        scriptlet.alias.as_ref().map(|a| format!("/{}", a))
    } else {
        scriptlet
            .group
            .as_deref()
            .filter(|g| *g != "main")
            .map(|g| g.to_string())
    }
}

/// Generate a code preview for scriptlets without explicit descriptions.
/// Shows the first meaningful line(s) of code, truncated to fit the description area.
/// For paste/snippet tools, this shows the pasted content; for open, the URL;
/// for code tools, the first non-comment line.
/// When the first line is very short (< 20 chars), appends the second line for richer context.
pub(crate) fn code_preview_for_scriptlet(scriptlet: &crate::scripts::Scriptlet) -> Option<String> {
    let code = &scriptlet.code;
    if code.is_empty() {
        return None;
    }

    // Collect meaningful (non-empty, non-comment) lines
    let meaningful_lines: Vec<&str> = code
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !l.starts_with("//")
                && !l.starts_with("/*")
                && !l.starts_with('*')
                && !l.starts_with("#!/")
        })
        .collect();

    let first_line = meaningful_lines.first()?;
    if first_line.is_empty() {
        return None;
    }

    let first_len = first_line.chars().count();

    // For very short first lines, append the second line for richer context
    // e.g., "cd ~/projects -> npm start"
    let preview = if first_len < 20 {
        if let Some(second_line) = meaningful_lines.get(1) {
            let combined = format!("{} → {}", first_line, second_line);
            let combined_len = combined.chars().count();
            if combined_len > 60 {
                let truncated: String = combined.chars().take(57).collect();
                format!("{}...", truncated)
            } else {
                combined
            }
        } else {
            first_line.to_string()
        }
    } else if first_len > 60 {
        let truncated: String = first_line.chars().take(57).collect();
        format!("{}...", truncated)
    } else {
        first_line.to_string()
    };

    Some(preview)
}

#[cfg(test)]
mod truncate_str_chars_tests {
    use super::truncate_str_chars;

    #[test]
    fn test_truncate_str_chars_returns_original_when_string_is_shorter() {
        assert_eq!(truncate_str_chars("short", 10), "short");
    }

    #[test]
    fn test_truncate_str_chars_truncates_at_char_boundary_when_utf8_input_is_long() {
        let input = "你好🙂abc";
        assert_eq!(truncate_str_chars(input, 3), "你好🙂");
    }
}
