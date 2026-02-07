/// Extract code block from text, handling nested fences
/// Returns (tool, code) if found
pub fn extract_code_block_nested(text: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut in_fence = false;
    let mut fence_type: Option<FenceType> = None;
    let mut fence_count = 0;
    let mut tool = String::new();
    let mut code_lines = Vec::new();
    let mut found = false;

    for line in lines {
        let trimmed = line.trim_start();

        if !in_fence {
            // Check for opening fence
            if let Some(fence_info) = detect_fence_start(trimmed) {
                in_fence = true;
                fence_type = Some(fence_info.0);
                fence_count = fence_info.1;
                tool = fence_info.2;
                continue;
            }
        } else {
            // Check for closing fence (same type, same or more chars)
            if let Some(current_fence_type) = fence_type {
                if is_matching_fence_end(trimmed, current_fence_type, fence_count) {
                    found = true;
                    break;
                }
            }
            code_lines.push(line);
        }
    }

    if found {
        let code = code_lines.join("\n");
        Some((tool, code.trim().to_string()))
    } else if in_fence && !code_lines.is_empty() {
        // Unclosed fence, but we have content
        let code = code_lines.join("\n");
        Some((tool, code.trim().to_string()))
    } else {
        None
    }
}
/// Detect if a line starts a code fence, returns (fence_type, count, language)
fn detect_fence_start(line: &str) -> Option<(FenceType, usize, String)> {
    let backtick_count = line.chars().take_while(|&c| c == '`').count();
    if backtick_count >= 3 {
        let rest = &line[backtick_count..];
        let lang = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some((FenceType::Backticks, backtick_count, lang));
    }

    let tilde_count = line.chars().take_while(|&c| c == '~').count();
    if tilde_count >= 3 {
        let rest = &line[tilde_count..];
        let lang = rest.split_whitespace().next().unwrap_or("").to_string();
        return Some((FenceType::Tildes, tilde_count, lang));
    }

    None
}
/// Check if a line is a closing fence matching the opening
fn is_matching_fence_end(line: &str, fence_type: FenceType, min_count: usize) -> bool {
    let count = match fence_type {
        FenceType::Backticks => line.chars().take_while(|&c| c == '`').count(),
        FenceType::Tildes => line.chars().take_while(|&c| c == '~').count(),
    };

    if count < min_count {
        return false;
    }

    // Rest of line should be empty or whitespace
    let rest = &line[count..];
    rest.chars().all(|c| c.is_whitespace())
}
/// Extract H3 actions from a scriptlet section
///
/// H3 headers within an H2 section define actions that appear in the Actions Menu.
/// Each H3 must have a valid tool codefence to become an action.
///
/// # Example
/// ```markdown
/// ## My Scriptlet
/// ```bash
/// main code
/// ```
///
/// ### Copy to Clipboard
/// <!-- shortcut: cmd+c -->
/// ```bash
/// echo "{{selection}}" | pbcopy
/// ```
///
/// ### Open in Browser
/// ```open
/// https://example.com
/// ```
/// ```
fn extract_h3_actions(section_text: &str) -> Vec<ScriptletAction> {
    let mut actions = Vec::new();
    let lines: Vec<&str> = section_text.lines().collect();

    let mut i = 0;
    let mut found_main_code = false;

    // First, skip past the main H2 content until we find the main code block
    // The main code block is the first valid tool codefence after the H2 header
    while i < lines.len() {
        let trimmed = lines[i].trim_start();

        // Skip the H2 header itself
        if trimmed.starts_with("## ") {
            i += 1;
            continue;
        }

        // Check for code fence start
        if let Some(fence_info) = detect_fence_start(trimmed) {
            // Check if this is a valid tool or metadata/schema block
            let lang = &fence_info.2;
            if VALID_TOOLS.contains(&lang.as_str()) || lang.is_empty() {
                // This is the main scriptlet code, skip it
                found_main_code = true;
                // Skip to end of this fence
                let fence_type = fence_info.0;
                let fence_count = fence_info.1;
                i += 1;
                while i < lines.len() {
                    if is_matching_fence_end(lines[i].trim_start(), fence_type, fence_count) {
                        break;
                    }
                    i += 1;
                }
            }
        }

        // Once we've found and passed the main code, look for H3s
        if found_main_code && trimmed.starts_with("### ") {
            // Found an H3 - extract its content until the next H3 or end of section
            let Some(h3_name) = trimmed
                .strip_prefix("### ")
                .map(str::trim)
                .map(str::to_string)
            else {
                i += 1;
                continue;
            };
            if h3_name.is_empty() {
                i += 1;
                continue;
            }

            // Collect content until next H3 or end
            let mut h3_content = String::new();
            i += 1;
            while i < lines.len() {
                let line_trimmed = lines[i].trim_start();
                if line_trimmed.starts_with("### ") {
                    break;
                }
                h3_content.push_str(lines[i]);
                h3_content.push('\n');
                i += 1;
            }

            // Try to parse this H3 as an action
            if let Some(action) = parse_h3_action(&h3_name, &h3_content) {
                actions.push(action);
            }

            continue; // Don't increment i again
        }

        i += 1;
    }

    actions
}
/// Parse a single H3 section into a ScriptletAction
fn parse_h3_action(name: &str, content: &str) -> Option<ScriptletAction> {
    // Extract metadata from HTML comments
    let metadata = parse_html_comment_metadata(content);

    // Extract code block
    let (tool_str, code) = extract_code_block_nested(content)?;

    // Default to bash if no tool specified
    let tool = if tool_str.is_empty() {
        "bash".to_string()
    } else {
        tool_str
    };

    // Only create action if tool is valid
    if !VALID_TOOLS.contains(&tool.as_str()) {
        debug!(tool = %tool, action = %name, "Unknown tool type in scriptlet action, skipping");
        return None;
    }

    let inputs = extract_named_inputs(&code);
    let command = slugify(name);

    Some(ScriptletAction {
        name: name.to_string(),
        command,
        tool,
        code,
        inputs,
        shortcut: metadata.shortcut,
        description: metadata.description,
    })
}
/// Parse a `.actions.md` file into shared actions
///
/// # Format
/// - H1 headers (`# Group Name`) define groups (for organization only)
/// - H3 headers (`### Action Name`) define individual actions
/// - H2 headers are ignored (they're for regular scriptlets)
/// - HTML comments contain metadata (shortcut, description)
/// - Code fences contain the action code
///
/// All H3 actions are returned flat (group names are not used at runtime).
#[allow(dead_code)]
pub fn parse_actions_file(content: &str) -> Vec<ScriptletAction> {
    let mut actions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();

        // Look for H3 headers - these are actions
        if trimmed.starts_with("### ") {
            let Some(h3_name) = trimmed
                .strip_prefix("### ")
                .map(str::trim)
                .map(str::to_string)
            else {
                i += 1;
                continue;
            };
            if h3_name.is_empty() {
                i += 1;
                continue;
            }

            // Collect content until next H1, H2, H3, or end
            let mut h3_content = String::new();
            i += 1;
            while i < lines.len() {
                let line_trimmed = lines[i].trim_start();
                if line_trimmed.starts_with("# ")
                    || line_trimmed.starts_with("## ")
                    || line_trimmed.starts_with("### ")
                {
                    break;
                }
                h3_content.push_str(lines[i]);
                h3_content.push('\n');
                i += 1;
            }

            // Try to parse this H3 as an action
            if let Some(action) = parse_h3_action(&h3_name, &h3_content) {
                actions.push(action);
            }

            continue; // Don't increment i again
        }

        i += 1;
    }

    actions
}
/// Get the path to the companion actions file for a given markdown file
///
/// For `foo.md`, returns `foo.actions.md`
/// For `foo.bar.md`, returns `foo.bar.actions.md`
#[allow(dead_code)]
pub fn get_actions_file_path(md_path: &std::path::Path) -> std::path::PathBuf {
    md_path.with_extension("actions.md")
}
/// Load shared actions from a companion `.actions.md` file if it exists
///
/// For a scriptlet file at `/path/to/foo.md`, this checks for
/// `/path/to/foo.actions.md` and parses any actions defined there.
#[allow(dead_code)]
pub fn load_shared_actions(md_path: &std::path::Path) -> Vec<ScriptletAction> {
    let actions_path = get_actions_file_path(md_path);

    if actions_path.exists() {
        match std::fs::read_to_string(&actions_path) {
            Ok(content) => {
                let actions = parse_actions_file(&content);
                if !actions.is_empty() {
                    debug!(
                        path = %actions_path.display(),
                        count = actions.len(),
                        "Loaded shared actions from companion file"
                    );
                }
                actions
            }
            Err(e) => {
                warn!(
                    path = %actions_path.display(),
                    error = %e,
                    "Failed to read actions file"
                );
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}
/// Merge shared actions into a scriptlet
///
/// Shared actions are added to the scriptlet's actions list.
/// If an action with the same command already exists (inline action),
/// the inline action takes precedence and the shared one is skipped.
#[allow(dead_code)]
pub fn merge_shared_actions(scriptlet: &mut Scriptlet, shared_actions: &[ScriptletAction]) {
    for shared_action in shared_actions {
        // Check if an inline action with the same command already exists
        let exists = scriptlet
            .actions
            .iter()
            .any(|a| a.command == shared_action.command);

        if !exists {
            scriptlet.actions.push(shared_action.clone());
        }
    }
}
/// Parse a markdown file into scriptlets
///
/// # Format
/// - H1 headers (`# Group Name`) define groups
/// - H1 can have a code fence that prepends to all scriptlets in that group
/// - H2 headers (`## Scriptlet Name`) define individual scriptlets
/// - H3 headers (`### Action Name`) define actions for the parent scriptlet
/// - HTML comments contain metadata
/// - Code fences contain the scriptlet code
pub fn parse_markdown_as_scriptlets(content: &str, source_path: Option<&str>) -> Vec<Scriptlet> {
    let mut scriptlets = Vec::new();
    let mut current_group = String::new();
    let mut global_prepend = String::new();

    // Split by headers while preserving the header type
    let sections = split_by_headers(content);

    for section in sections {
        let section_text = section.text;
        let first_line = section_text.lines().next().unwrap_or("");

        if first_line.starts_with("## ") {
            // H2: Individual scriptlet
            let name = first_line
                .strip_prefix("## ")
                .unwrap_or("")
                .trim()
                .to_string();

            if name.is_empty() {
                continue;
            }

            // Try codefence metadata first (new format)
            let codefence_result = parse_codefence_metadata(section_text);
            let typed_metadata = codefence_result.metadata;
            let schema = codefence_result.schema;

            // Also parse HTML comment metadata (legacy format, for backward compatibility)
            let metadata = parse_html_comment_metadata(section_text);

            // Extract code block - prefer codefence result if available, else use legacy extraction
            let code_block = codefence_result
                .code
                .map(|code_block| (code_block.language, code_block.content))
                .or_else(|| extract_code_block_nested(section_text));

            if let Some((tool_str, mut code)) = code_block {
                // Prepend global code if exists and tool matches
                if !global_prepend.is_empty() {
                    code = format!("{}\n{}", global_prepend, code);
                }

                // Validate tool type
                let tool: String = if tool_str.is_empty() {
                    "ts".to_string()
                } else {
                    tool_str
                };

                // Check if tool is valid, warn if not
                if !VALID_TOOLS.contains(&tool.as_str()) {
                    debug!(tool = %tool, name = %name, "Unknown tool type in scriptlet");
                }

                let inputs = extract_named_inputs(&code);
                let command = slugify(&name);

                // Extract H3 actions from this section
                let actions = extract_h3_actions(section_text);

                scriptlets.push(Scriptlet {
                    name,
                    command,
                    tool,
                    scriptlet_content: code,
                    inputs,
                    group: current_group.clone(),
                    preview: None,
                    metadata,
                    typed_metadata,
                    schema,
                    kit: None,
                    source_path: source_path.map(|s| s.to_string()),
                    actions,
                });
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

    // Load and merge shared actions from companion .actions.md file
    if let Some(path_str) = source_path {
        let path = std::path::Path::new(path_str);
        // Only load shared actions for .md files (not .actions.md files themselves)
        if !path_str.ends_with(".actions.md") {
            let shared_actions = load_shared_actions(path);
            if !shared_actions.is_empty() {
                for scriptlet in &mut scriptlets {
                    merge_shared_actions(scriptlet, &shared_actions);
                }
            }
        }
    }

    scriptlets
}
/// Section of markdown content with its header level
struct MarkdownSection<'a> {
    text: &'a str,
}
