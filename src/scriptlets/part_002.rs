/// Split markdown content by headers, preserving header lines
fn split_by_headers(content: &str) -> Vec<MarkdownSection<'_>> {
    let mut sections = Vec::new();
    let mut current_start = 0;
    let mut in_fence = false;
    let mut fence_type: Option<FenceType> = None;
    let mut fence_count = 0;

    let lines: Vec<&str> = content.lines().collect();
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        // Track fence state
        if !in_fence {
            if let Some(fence_info) = detect_fence_start(trimmed) {
                in_fence = true;
                fence_type = Some(fence_info.0);
                fence_count = fence_info.1;
                continue;
            }
        } else if let Some(current_fence_type) = fence_type {
            if is_matching_fence_end(trimmed, current_fence_type, fence_count) {
                in_fence = false;
                fence_type = None;
                fence_count = 0;
                continue;
            }
        }

        // Only split on headers outside of fences
        if !in_fence && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            if i > 0 {
                let start = line_starts[current_start];
                let end = line_starts[i];
                if end > start {
                    sections.push(MarkdownSection {
                        text: &content[start..end],
                    });
                }
            }
            current_start = i;
        }
    }

    // Add remaining content
    if current_start < lines.len() {
        let start = line_starts[current_start];
        sections.push(MarkdownSection {
            text: &content[start..],
        });
    }

    sections
}
// ============================================================================
// Validation-Aware Parsing
// ============================================================================

/// Parse markdown file into scriptlets with validation and graceful degradation.
///
/// Unlike `parse_markdown_as_scriptlets`, this function:
/// - Returns both valid scriptlets AND validation errors
/// - Continues parsing after individual scriptlet validation failures
/// - Parses bundle-level frontmatter
/// - Resolves icons using the priority order (scriptlet > frontmatter > tool default)
#[allow(dead_code)] // Public API for future use
pub fn parse_scriptlets_with_validation(
    content: &str,
    source_path: Option<&str>,
) -> ScriptletParseResult {
    let mut result = ScriptletParseResult::default();
    let file_path = PathBuf::from(source_path.unwrap_or("<unknown>"));

    // Parse bundle-level frontmatter
    result.frontmatter = parse_bundle_frontmatter(content);

    let mut current_group = String::new();
    let mut global_prepend = String::new();

    // Split by headers while preserving the header type and line numbers
    let sections = split_by_headers_with_line_numbers(content);

    for section in sections {
        let section_text = section.text;
        let section_start_line = section.line_number;
        let first_line = section_text.lines().next().unwrap_or("");

        if first_line.starts_with("## ") {
            // H2: Individual scriptlet
            let name = first_line
                .strip_prefix("## ")
                .unwrap_or("")
                .trim()
                .to_string();

            if name.is_empty() {
                result.errors.push(ScriptletValidationError::new(
                    &file_path,
                    None,
                    Some(section_start_line),
                    "Empty scriptlet name (H2 header with no text)",
                ));
                continue;
            }

            // Try to parse this scriptlet, catching any validation errors
            match parse_single_scriptlet(
                section_text,
                &name,
                &current_group,
                &global_prepend,
                source_path,
                result.frontmatter.as_ref(),
                section_start_line,
                &file_path,
            ) {
                Ok(scriptlet) => result.scriptlets.push(scriptlet),
                Err(error) => result.errors.push(error),
            }
        } else if first_line.starts_with("# ") {
            // H1: Group header
            let group_name = first_line
                .strip_prefix("# ")
                .unwrap_or("")
                .trim()
                .to_string();
            current_group = group_name;

            // Check for global prepend code block
            if let Some((_, code)) = extract_code_block_nested(section_text) {
                global_prepend = code;
            } else {
                global_prepend.clear();
            }
        }
    }

    result
}
/// Parse a single scriptlet from a section, returning either a Scriptlet or a validation error
#[allow(dead_code)] // Used by parse_scriptlets_with_validation
#[allow(clippy::too_many_arguments)]
fn parse_single_scriptlet(
    section_text: &str,
    name: &str,
    current_group: &str,
    global_prepend: &str,
    source_path: Option<&str>,
    frontmatter: Option<&BundleFrontmatter>,
    section_start_line: usize,
    file_path: &PathBuf,
) -> Result<Scriptlet, ScriptletValidationError> {
    // Try codefence metadata first (new format)
    let codefence_result = parse_codefence_metadata(section_text);
    let typed_metadata = codefence_result.metadata;
    let schema = codefence_result.schema;

    // Check for codefence parse errors - log but don't fail
    for error in &codefence_result.errors {
        debug!(error = %error, scriptlet = %name, "Codefence parse warning");
    }

    // Also parse HTML comment metadata (legacy format, for backward compatibility)
    let metadata = parse_html_comment_metadata(section_text);

    // Extract code block - prefer codefence result if available
    let code_block = codefence_result
        .code
        .map(|code_block| (code_block.language, code_block.content))
        .or_else(|| extract_code_block_nested(section_text));

    let (tool_str, mut code) = code_block.ok_or_else(|| {
        ScriptletValidationError::new(
            file_path,
            Some(name.to_string()),
            Some(section_start_line),
            "No code block found in scriptlet",
        )
    })?;

    // Prepend global code if exists
    if !global_prepend.is_empty() {
        code = format!("{}\n{}", global_prepend, code);
    }

    // Default tool type to "ts" if empty
    let tool = if tool_str.is_empty() {
        "ts".to_string()
    } else {
        tool_str
    };

    // Check if tool is valid - emit warning but don't fail
    if !VALID_TOOLS.contains(&tool.as_str()) {
        debug!(tool = %tool, name = %name, "Unknown tool type in scriptlet");
    }

    // Resolve icon using priority order
    let _resolved_icon = resolve_scriptlet_icon(&metadata, frontmatter, &tool);

    let inputs = extract_named_inputs(&code);
    let command = slugify(name);

    // Extract H3 actions from this section
    let actions = extract_h3_actions(section_text);

    Ok(Scriptlet {
        name: name.to_string(),
        command,
        tool,
        scriptlet_content: code,
        inputs,
        group: current_group.to_string(),
        preview: None,
        metadata,
        typed_metadata,
        schema,
        kit: None,
        source_path: source_path.map(|s| s.to_string()),
        actions,
    })
}
/// Section of markdown content with its header level and line number
#[allow(dead_code)] // Used by split_by_headers_with_line_numbers
struct MarkdownSectionWithLine<'a> {
    text: &'a str,
    line_number: usize, // 1-based line number
}
/// Split markdown content by headers, preserving header lines and line numbers
#[allow(dead_code)] // Used by parse_scriptlets_with_validation
fn split_by_headers_with_line_numbers(content: &str) -> Vec<MarkdownSectionWithLine<'_>> {
    let mut sections = Vec::new();
    let mut current_start = 0;
    let mut current_start_line = 1; // 1-based
    let mut in_fence = false;
    let mut fence_type: Option<FenceType> = None;
    let mut fence_count = 0;

    let lines: Vec<&str> = content.lines().collect();
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        // Track fence state
        if !in_fence {
            if let Some(fence_info) = detect_fence_start(trimmed) {
                in_fence = true;
                fence_type = Some(fence_info.0);
                fence_count = fence_info.1;
                continue;
            }
        } else if let Some(current_fence_type) = fence_type {
            if is_matching_fence_end(trimmed, current_fence_type, fence_count) {
                in_fence = false;
                fence_type = None;
                fence_count = 0;
                continue;
            }
        }

        // Only split on headers outside of fences
        if !in_fence && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            if i > 0 {
                let start = line_starts[current_start];
                let end = line_starts[i];
                if end > start {
                    sections.push(MarkdownSectionWithLine {
                        text: &content[start..end],
                        line_number: current_start_line,
                    });
                }
            }
            current_start = i;
            current_start_line = i + 1; // Convert to 1-based
        }
    }

    // Add remaining content
    if current_start < lines.len() {
        let start = line_starts[current_start];
        sections.push(MarkdownSectionWithLine {
            text: &content[start..],
            line_number: current_start_line,
        });
    }

    sections
}
// ============================================================================
// Variable Substitution
// ============================================================================

/// Format a scriptlet by substituting variables
///
/// # Variable Types
/// - `{{variableName}}` - Named input, replaced with value from inputs map
/// - `$1`, `$2`, etc. (Unix) or `%1`, `%2`, etc. (Windows) - Positional args
/// - `$@` (Unix) or `%*` (Windows) - All arguments
///
/// # Arguments
/// * `content` - The scriptlet content with placeholders
/// * `inputs` - Map of variable names to values
/// * `positional_args` - List of positional arguments
/// * `windows` - If true, use Windows-style placeholders (%1, %*)
pub fn format_scriptlet(
    content: &str,
    inputs: &HashMap<String, String>,
    positional_args: &[String],
    windows: bool,
) -> String {
    let mut result = content.to_string();

    // Replace named inputs {{variableName}}
    for (name, value) in inputs {
        let placeholder = format!("{{{{{}}}}}", name);
        result = result.replace(&placeholder, value);
    }

    // Replace positional arguments
    if windows {
        // Windows style: %1, %2, etc.
        for (i, arg) in positional_args.iter().enumerate() {
            let placeholder = format!("%{}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Replace %* with all args quoted
        let all_args = positional_args
            .iter()
            .map(|a| format!("\"{}\"", a.replace('\"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ");
        result = result.replace("%*", &all_args);
    } else {
        // Unix style: $1, $2, etc.
        for (i, arg) in positional_args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Replace $@ with all args quoted
        let all_args = positional_args
            .iter()
            .map(|a| format!("\"{}\"", a.replace('\"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ");
        result = result.replace("$@", &all_args);
    }

    result
}
/// Process conditional blocks in scriptlet content
///
/// Supports:
/// - `{{#if flag}}...{{/if}}` - Include content if flag is truthy
/// - `{{#if flag}}...{{else}}...{{/if}}` - If-else
/// - `{{#if flag}}...{{else if other}}...{{else}}...{{/if}}` - If-else-if chains
///
/// # Arguments
/// * `content` - The scriptlet content with conditionals
/// * `flags` - Map of flag names to boolean values
pub fn process_conditionals(content: &str, flags: &HashMap<String, bool>) -> String {
    process_conditionals_impl(content, flags)
}
/// Internal implementation that handles the recursive conditional processing
fn process_conditionals_impl(content: &str, flags: &HashMap<String, bool>) -> String {
    let mut result = String::with_capacity(content.len());
    let mut i = 0;
    let bytes = content.as_bytes();

    while i < bytes.len() {
        // Check for {{#if
        if i + 5 < bytes.len() && &bytes[i..i + 3] == b"{{#" {
            // Find the closing }}
            if let Some(end_tag) = find_closing_braces(content, i + 3) {
                let directive = &content[i + 3..end_tag];

                if let Some(flag_name) = directive.strip_prefix("if ").map(str::trim) {
                    let remaining = &content[end_tag + 2..];
                    let (processed, consumed) = process_if_block(remaining, flag_name, flags);
                    result.push_str(&processed);
                    i = end_tag + 2 + consumed;
                    continue;
                }
            }
        }

        // Not a conditional, just copy the character
        if i < content.len() {
            if let Some(next_char) = content[i..].chars().next() {
                result.push(next_char);
                i += next_char.len_utf8();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}
/// Find the position of closing }} starting from a given position
fn find_closing_braces(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut i = start;

    while i + 1 < bytes.len() {
        if bytes[i] == b'}' && bytes[i + 1] == b'}' {
            return Some(i);
        }
        i += 1;
    }

    None
}
