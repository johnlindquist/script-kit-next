use std::path::Path;

use crate::scriptlet_metadata::parse_codefence_metadata;

use super::super::types::Scriptlet;

pub(crate) fn extract_html_comment_metadata(
    text: &str,
) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut metadata = HashMap::new();

    // Find HTML comment blocks
    if let Some(start) = text.find("<!--") {
        if let Some(end) = text.find("-->") {
            if start < end {
                let comment_content = &text[start + 4..end];
                // Parse key: value pairs
                for line in comment_content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Handle format: "key: value"
                        if let Some(colon_pos) = trimmed.find(':') {
                            let key = trimmed[..colon_pos].trim().to_string();
                            let value = trimmed[colon_pos + 1..].trim().to_string();
                            metadata.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    metadata
}

/// Extract code block from markdown text
/// Looks for ```language ... ``` pattern and returns (language, code)
/// Skips `metadata` and `schema` blocks which are used for configuration
pub(crate) fn extract_code_block(text: &str) -> Option<(String, String)> {
    let mut search_start = 0;

    while let Some(fence_offset) = text[search_start..].find("```") {
        let start = search_start + fence_offset;
        let after_fence = &text[start + 3..];

        // Get the language specifier (rest of line)
        if let Some(newline_pos) = after_fence.find('\n') {
            let language = after_fence[..newline_pos].trim();
            let code_start = start + 3 + newline_pos + 1;

            // Find closing fence
            if let Some(end_pos) = text[code_start..].find("```") {
                // Skip metadata and schema blocks - these are config, not code
                if language == "metadata" || language == "schema" {
                    // Move past this block and continue searching
                    search_start = code_start + end_pos + 3;
                    continue;
                }

                let code = text[code_start..code_start + end_pos].trim().to_string();
                return Some((language.to_owned(), code));
            }
        }

        // Couldn't parse this fence, move past it
        search_start = start + 3;
    }

    None
}

/// Convert a name to a command slug (lowercase, spaces/special chars to hyphens)
pub(crate) fn slugify_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Parse a single scriptlet section from markdown
/// Input should be text from ## Name to the next ## or end of file
/// `source_path` is the path to the .md file containing the scriptlet
pub(crate) fn parse_scriptlet_section(
    section: &str,
    source_path: Option<&Path>,
) -> Option<Scriptlet> {
    let lines: Vec<&str> = section.lines().collect();
    if lines.is_empty() {
        return None;
    }

    // First line should be ## Name
    let first_line = lines[0];
    if !first_line.starts_with("##") {
        return None;
    }

    let name = first_line
        .strip_prefix("##")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if name.is_empty() {
        return None;
    }

    // Extract metadata from HTML comments
    let html_metadata = extract_html_comment_metadata(section);

    // Also try codefence metadata (```metadata ... ```)
    let codefence_result = parse_codefence_metadata(section);
    let typed_metadata = codefence_result.metadata;

    // Log what we found for debugging
    tracing::debug!(
        category = "KEYWORD",
        name = %name,
        html_keyword = ?html_metadata.get("keyword"),
        codefence_keyword = ?typed_metadata.as_ref().and_then(|t| t.keyword.as_ref()),
        "Parsing scriptlet section metadata"
    );

    // Extract code block
    let (tool, code) = extract_code_block(section)?;

    // Generate command slug from name
    let command = slugify_name(&name);

    // Build file_path with anchor if source_path is provided
    let file_path = source_path.map(|p| format!("{}#{}", p.display(), command));

    // Prefer codefence metadata over HTML comment metadata for keyword
    let keyword = typed_metadata
        .as_ref()
        .and_then(|t| t.keyword.clone())
        .or_else(|| html_metadata.get("keyword").cloned());

    if keyword.is_some() {
        tracing::debug!(
            category = "KEYWORD",
            name = %name,
            keyword = ?keyword,
            "Found keyword trigger in scriptlet"
        );
    }

    Some(Scriptlet {
        name,
        description: typed_metadata
            .as_ref()
            .and_then(|t| t.description.clone())
            .or_else(|| html_metadata.get("description").cloned()),
        code,
        tool,
        shortcut: typed_metadata
            .as_ref()
            .and_then(|t| t.shortcut.clone())
            .or_else(|| html_metadata.get("shortcut").cloned()),
        keyword,
        group: None,
        file_path,
        command: Some(command),
        alias: typed_metadata
            .as_ref()
            .and_then(|t| t.alias.clone())
            .or_else(|| html_metadata.get("alias").cloned()),
    })
}
