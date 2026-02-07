use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;
use tracing::debug;
/// Result of parsing codefence metadata from a scriptlet
#[derive(Debug, Clone, Default)]
pub struct CodefenceParseResult {
    /// Parsed metadata from ```metadata block
    pub metadata: Option<TypedMetadata>,
    /// Parsed schema from ```schema block
    pub schema: Option<Schema>,
    /// The code content from the main code block (e.g., ```ts)
    pub code: Option<CodeBlock>,
    /// Parse errors encountered
    pub errors: Vec<String>,
}
/// A code block with its language and content
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// The language identifier (e.g., "ts", "bash", "python")
    pub language: String,
    /// The code content
    pub content: String,
}
/// Parse codefence blocks from markdown scriptlet content
///
/// Looks for:
/// - `\`\`\`metadata\n{...}\n\`\`\`` - JSON metadata block
/// - `\`\`\`schema\n{...}\n\`\`\`` - JSON schema block  
/// - `\`\`\`<lang>\n...\n\`\`\`` - Main code block
///
/// # Arguments
/// * `content` - The markdown content to parse
///
/// # Returns
/// `CodefenceParseResult` with parsed metadata, schema, code, and any errors
pub fn parse_codefence_metadata(content: &str) -> CodefenceParseResult {
    let mut result = CodefenceParseResult::default();

    let blocks = extract_all_codefence_blocks(content);

    for (language, block_content) in blocks {
        match language.as_str() {
            "metadata" => {
                // Try JSON first, then fall back to simple key: value format
                if let Ok(metadata) = serde_json::from_str::<TypedMetadata>(&block_content) {
                    debug!(
                        name = ?metadata.name,
                        description = ?metadata.description,
                        "Parsed codefence metadata (JSON)"
                    );
                    result.metadata = Some(metadata);
                } else if let Some(metadata) = parse_simple_metadata(&block_content) {
                    debug!(
                        keyword = ?metadata.keyword,
                        "Parsed codefence metadata (simple format)"
                    );
                    result.metadata = Some(metadata);
                } else {
                    result.errors.push(
                        "Failed to parse metadata: not valid JSON or simple key: value format"
                            .to_string(),
                    );
                }
            }
            "schema" => match serde_json::from_str::<Schema>(&block_content) {
                Ok(schema) => {
                    debug!(
                        input_fields = schema.input.len(),
                        output_fields = schema.output.len(),
                        "Parsed codefence schema"
                    );
                    result.schema = Some(schema);
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to parse schema JSON: {}", e));
                }
            },
            // Skip empty language specifier
            "" => {}
            // Any other language is treated as code
            lang => {
                // Only capture the first non-metadata/schema code block
                if result.code.is_none() {
                    result.code = Some(CodeBlock {
                        language: lang.to_string(),
                        content: block_content,
                    });
                }
            }
        }
    }

    result
}
/// Extract all codefence blocks from content
/// Returns Vec of (language, content) tuples
fn extract_all_codefence_blocks(content: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Check for opening fence (``` or ~~~)
        if let Some((fence_char, fence_count, language)) = detect_fence_opening(trimmed) {
            let mut block_lines = Vec::new();
            i += 1;

            // Collect content until closing fence
            while i < lines.len() {
                let current = lines[i].trim_start();
                if is_closing_fence(current, fence_char, fence_count) {
                    break;
                }
                block_lines.push(lines[i]);
                i += 1;
            }

            let block_content = block_lines.join("\n");
            blocks.push((language, block_content.trim().to_string()));
        }

        i += 1;
    }

    blocks
}
/// Detect opening fence, returns (fence_char, count, language)
fn detect_fence_opening(line: &str) -> Option<(char, usize, String)> {
    // Try backticks
    let backtick_count = line.chars().take_while(|&c| c == '`').count();
    if backtick_count >= 3 {
        let rest = &line[backtick_count..];
        let language = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some(('`', backtick_count, language));
    }

    // Try tildes
    let tilde_count = line.chars().take_while(|&c| c == '~').count();
    if tilde_count >= 3 {
        let rest = &line[tilde_count..];
        let language = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some(('~', tilde_count, language));
    }

    None
}
/// Check if line is a closing fence
fn is_closing_fence(line: &str, fence_char: char, min_count: usize) -> bool {
    let count = line.chars().take_while(|&c| c == fence_char).count();
    if count < min_count {
        return false;
    }

    // Rest of line should be empty or whitespace
    let rest = &line[count..];
    rest.chars().all(|c| c.is_whitespace())
}
/// Parse simple key: value metadata format
///
/// Supports lines like:
/// ```text
/// keyword: !testing
/// name: My Script
/// description: Does something useful
/// ```
///
/// The value is everything after `: ` (colon-space).
/// Lines starting with `//` are treated as comments and ignored.
/// Empty lines are ignored.
///
/// Special handling:
/// - `keyword`, `expand`, `snippet` all map to the `keyword` field
fn parse_simple_metadata(content: &str) -> Option<TypedMetadata> {
    use std::collections::HashMap;

    let mut fields: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Look for `key: value` pattern (colon followed by space)
        if let Some(colon_pos) = line.find(": ") {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 2..].trim().to_string();

            if !value.is_empty() {
                fields.insert(key, value);
            }
        }
    }

    // If no fields were parsed, return None
    if fields.is_empty() {
        return None;
    }

    // Build TypedMetadata from parsed fields
    let mut metadata = TypedMetadata::default();

    for (key, value) in fields {
        match key.as_str() {
            "name" => metadata.name = Some(value),
            "description" => metadata.description = Some(value),
            "author" => metadata.author = Some(value),
            "enter" => metadata.enter = Some(value),
            "alias" => metadata.alias = Some(value),
            "keyword" | "expand" | "snippet" => metadata.keyword = Some(value),
            "icon" => metadata.icon = Some(value),
            "shortcut" => metadata.shortcut = Some(value),
            "placeholder" => metadata.placeholder = Some(value),
            "cron" => metadata.cron = Some(value),
            "schedule" => metadata.schedule = Some(value),
            "hidden" => metadata.hidden = value.to_lowercase() == "true" || value == "1",
            "background" => metadata.background = value.to_lowercase() == "true" || value == "1",
            "system" => metadata.system = value.to_lowercase() == "true" || value == "1",
            "fallback" => metadata.fallback = value.to_lowercase() == "true" || value == "1",
            "fallback_label" => metadata.fallback_label = Some(value),
            // Unknown fields go to extra
            _ => {
                metadata.extra.insert(key, serde_json::Value::String(value));
            }
        }
    }

    Some(metadata)
}
