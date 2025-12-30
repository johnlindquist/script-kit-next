//! Scriptlet parsing and variable substitution module
//!
//! This module provides comprehensive support for parsing markdown files
//! containing scriptlets (code snippets with metadata) and performing
//! variable substitution in scriptlet content.
//!
//! # Types
//! - `Scriptlet`: Full scriptlet with all metadata
//! - `ScriptletMetadata`: Parsed HTML comment metadata
//!
//! # Features
//! - Parse markdown files with H1 groups and H2 scriptlets
//! - Extract metadata from HTML comments
//! - Handle nested code fences (``` inside ~~~ and vice versa)
//! - Variable substitution with named inputs, positional args, and conditionals

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Valid tool types that can be used in code fences
pub const VALID_TOOLS: &[&str] = &[
    "bash",
    "python",
    "kit",
    "ts",
    "js",
    "transform",
    "template",
    "open",
    "edit",
    "paste",
    "type",
    "submit",
    "applescript",
    "ruby",
    "perl",
    "php",
    "node",
    "deno",
    "bun",
    // Shell variants
    "zsh",
    "sh",
    "fish",
    "cmd",
    "powershell",
    "pwsh",
];

/// Shell tools (tools that execute in a shell environment)
pub const SHELL_TOOLS: &[&str] = &["bash", "zsh", "sh", "fish", "cmd", "powershell", "pwsh"];

/// Metadata extracted from HTML comments in scriptlets
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ScriptletMetadata {
    /// Trigger text that activates this scriptlet
    pub trigger: Option<String>,
    /// Keyboard shortcut (e.g., "cmd shift k")
    pub shortcut: Option<String>,
    /// Raw cron expression (e.g., "*/5 * * * *")
    pub cron: Option<String>,
    /// Natural language schedule (e.g., "every tuesday at 2pm") - converted to cron internally
    pub schedule: Option<String>,
    /// Whether to run in background
    pub background: Option<bool>,
    /// File paths to watch for changes
    pub watch: Option<String>,
    /// System event to trigger on
    pub system: Option<String>,
    /// Description of the scriptlet
    pub description: Option<String>,
    /// Text expansion trigger (e.g., "type,,")
    pub expand: Option<String>,
    /// Alias trigger - when user types alias + space, immediately run script
    pub alias: Option<String>,
    /// Any additional metadata key-value pairs
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// A scriptlet parsed from a markdown file
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Scriptlet {
    /// Name of the scriptlet (from H2 header)
    pub name: String,
    /// Command identifier (slugified name)
    pub command: String,
    /// Tool type (bash, python, ts, etc.)
    pub tool: String,
    /// The actual code content
    pub scriptlet_content: String,
    /// Named input placeholders (e.g., ["variableName", "otherVar"])
    pub inputs: Vec<String>,
    /// Group name (from H1 header)
    pub group: String,
    /// HTML preview content (if any)
    pub preview: Option<String>,
    /// Parsed metadata from HTML comments
    pub metadata: ScriptletMetadata,
    /// The kenv this scriptlet belongs to
    pub kenv: Option<String>,
    /// Source file path
    pub source_path: Option<String>,
}

#[allow(dead_code)]
impl Scriptlet {
    /// Create a new scriptlet with minimal required fields
    pub fn new(name: String, tool: String, content: String) -> Self {
        let command = slugify(&name);
        let inputs = extract_named_inputs(&content);

        Scriptlet {
            name,
            command,
            tool,
            scriptlet_content: content,
            inputs,
            group: String::new(),
            preview: None,
            metadata: ScriptletMetadata::default(),
            kenv: None,
            source_path: None,
        }
    }

    /// Check if this scriptlet uses a shell tool
    pub fn is_shell(&self) -> bool {
        SHELL_TOOLS.contains(&self.tool.as_str())
    }

    /// Check if the tool type is valid
    pub fn is_valid_tool(&self) -> bool {
        VALID_TOOLS.contains(&self.tool.as_str())
    }
}

/// Convert a name to a command slug (lowercase, spaces to hyphens)
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract named input placeholders from scriptlet content
/// Finds all {{variableName}} patterns
fn extract_named_inputs(content: &str) -> Vec<String> {
    let mut inputs = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {
            let mut name = String::new();

            // Skip if it's a conditional ({{#if, {{else, {{/if)
            if chars.peek() == Some(&'#') || chars.peek() == Some(&'/') {
                continue;
            }

            // Collect the variable name
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    break;
                }
                name.push(ch);
                chars.next();
            }

            // Skip closing }}
            if chars.peek() == Some(&'}') {
                chars.next();
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
            }

            // Add if valid identifier and not already present
            let trimmed = name.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with('/')
                && trimmed != "else"
                && !inputs.contains(&trimmed.to_string())
            {
                inputs.push(trimmed.to_string());
            }
        }
    }

    inputs
}

/// Parse metadata from HTML comments
/// Supports format: <!-- key: value\nkey2: value2 -->
pub fn parse_html_comment_metadata(text: &str) -> ScriptletMetadata {
    let mut metadata = ScriptletMetadata::default();

    // Find all HTML comment blocks
    let mut remaining = text;
    while let Some(start) = remaining.find("<!--") {
        if let Some(end) = remaining[start..].find("-->") {
            let comment_content = &remaining[start + 4..start + end];

            // Parse key: value pairs
            for line in comment_content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Some(colon_pos) = trimmed.find(':') {
                    let key = trimmed[..colon_pos].trim().to_lowercase();
                    let value = trimmed[colon_pos + 1..].trim().to_string();

                    if value.is_empty() {
                        continue;
                    }

                    match key.as_str() {
                        "trigger" => metadata.trigger = Some(value),
                        "shortcut" => metadata.shortcut = Some(value),
                        "cron" => metadata.cron = Some(value),
                        "schedule" => metadata.schedule = Some(value),
                        "background" => {
                            metadata.background =
                                Some(value.to_lowercase() == "true" || value == "1")
                        }
                        "watch" => metadata.watch = Some(value),
                        "system" => metadata.system = Some(value),
                        "description" => metadata.description = Some(value),
                        "expand" => metadata.expand = Some(value),
                        "alias" => metadata.alias = Some(value),
                        _ => {
                            metadata.extra.insert(key, value);
                        }
                    }
                }
            }

            remaining = &remaining[start + end + 3..];
        } else {
            break;
        }
    }

    metadata
}

/// State for parsing code fences
#[derive(Clone, Copy, PartialEq)]
enum FenceType {
    Backticks, // ```
    Tildes,    // ~~~
}

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
            if is_matching_fence_end(trimmed, fence_type.unwrap(), fence_count) {
                found = true;
                break;
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

/// Parse a markdown file into scriptlets
///
/// # Format
/// - H1 headers (`# Group Name`) define groups
/// - H1 can have a code fence that prepends to all scriptlets in that group
/// - H2 headers (`## Scriptlet Name`) define individual scriptlets
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

            // Parse metadata from HTML comments
            let metadata = parse_html_comment_metadata(section_text);

            // Extract code block
            if let Some((tool, mut code)) = extract_code_block_nested(section_text) {
                // Prepend global code if exists and tool matches
                if !global_prepend.is_empty() {
                    code = format!("{}\n{}", global_prepend, code);
                }

                // Validate tool type
                let tool = if tool.is_empty() {
                    "ts".to_string()
                } else {
                    tool
                };

                // Check if tool is valid, warn if not
                if !VALID_TOOLS.contains(&tool.as_str()) {
                    debug!(tool = %tool, name = %name, "Unknown tool type in scriptlet");
                }

                let inputs = extract_named_inputs(&code);
                let command = slugify(&name);

                scriptlets.push(Scriptlet {
                    name,
                    command,
                    tool,
                    scriptlet_content: code,
                    inputs,
                    group: current_group.clone(),
                    preview: None,
                    metadata,
                    kenv: None,
                    source_path: source_path.map(|s| s.to_string()),
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

    scriptlets
}

/// Section of markdown content with its header level
struct MarkdownSection<'a> {
    text: &'a str,
}

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
        } else if is_matching_fence_end(trimmed, fence_type.unwrap(), fence_count) {
            in_fence = false;
            fence_type = None;
            fence_count = 0;
            continue;
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

                if directive.starts_with("if ") {
                    let flag_name = directive.strip_prefix("if ").unwrap().trim();
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
            result.push(content[i..].chars().next().unwrap());
            i += content[i..].chars().next().unwrap().len_utf8();
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

/// Process a single if block, returning (result, bytes_consumed)
fn process_if_block(
    content: &str,
    flag_name: &str,
    flags: &HashMap<String, bool>,
) -> (String, usize) {
    let flag_value = flags.get(flag_name).copied().unwrap_or(false);

    let mut depth = 1;
    let mut if_content = String::new();
    let mut else_content = String::new();
    let mut else_if_chains: Vec<(String, String)> = Vec::new(); // (flag, content)
    let mut in_else = false;
    let mut current_else_if_flag: Option<String> = None;
    let mut consumed = 0;

    let mut chars = content.chars().peekable();
    let mut pos = 0;

    while let Some(c) = chars.next() {
        pos += c.len_utf8();

        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            pos += 1;

            // Read what's inside
            let mut inner = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
                chars.next();
                pos += ch.len_utf8();
            }

            // Skip closing }}
            if chars.peek() == Some(&'}') {
                chars.next();
                pos += 1;
                if chars.peek() == Some(&'}') {
                    chars.next();
                    pos += 1;
                }
            }

            let inner_trimmed = inner.trim();

            if inner_trimmed.starts_with("#if ") {
                depth += 1;
                // Add to current content - inner already contains the #
                let tag = format!("{{{{{}}}}}", inner_trimmed);
                if in_else {
                    if current_else_if_flag.is_some() {
                        else_if_chains.last_mut().unwrap().1.push_str(&tag);
                    } else {
                        else_content.push_str(&tag);
                    }
                } else {
                    if_content.push_str(&tag);
                }
            } else if inner_trimmed == "/if" {
                depth -= 1;
                if depth == 0 {
                    consumed = pos;
                    break;
                } else {
                    let tag = "{{/if}}";
                    if in_else {
                        if current_else_if_flag.is_some() {
                            else_if_chains.last_mut().unwrap().1.push_str(tag);
                        } else {
                            else_content.push_str(tag);
                        }
                    } else {
                        if_content.push_str(tag);
                    }
                }
            } else if inner_trimmed == "else" && depth == 1 {
                in_else = true;
                current_else_if_flag = None;
            } else if inner_trimmed.starts_with("else if ") && depth == 1 {
                let else_if_flag = inner_trimmed
                    .strip_prefix("else if ")
                    .unwrap()
                    .trim()
                    .to_string();
                in_else = true;
                current_else_if_flag = Some(else_if_flag.clone());
                else_if_chains.push((else_if_flag, String::new()));
            } else {
                // Some other tag, add to current content
                let tag = format!("{{{{{}}}}}", inner);
                if in_else {
                    if current_else_if_flag.is_some() {
                        else_if_chains.last_mut().unwrap().1.push_str(&tag);
                    } else {
                        else_content.push_str(&tag);
                    }
                } else {
                    if_content.push_str(&tag);
                }
            }
        } else if in_else {
            if current_else_if_flag.is_some() {
                else_if_chains.last_mut().unwrap().1.push(c);
            } else {
                else_content.push(c);
            }
        } else {
            if_content.push(c);
        }
    }

    // Determine which content to use
    let result = if flag_value {
        // Process nested conditionals in if_content
        process_conditionals(&if_content, flags)
    } else {
        // Check else-if chains
        let mut found = false;
        let mut selected_content = String::new();

        for (chain_flag, chain_content) in &else_if_chains {
            if flags.get(chain_flag).copied().unwrap_or(false) {
                selected_content = process_conditionals(chain_content, flags);
                found = true;
                break;
            }
        }

        if !found {
            // Use else content
            process_conditionals(&else_content, flags)
        } else {
            selected_content
        }
    };

    (result, consumed)
}

// ============================================================================
// Interpreter Tool Constants and Error Helpers
// ============================================================================

/// Interpreter tools that require an external interpreter to execute
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub const INTERPRETER_TOOLS: &[&str] = &["python", "ruby", "perl", "php", "node"];

/// Get the interpreter command for a given tool
///
/// # Arguments
/// * `tool` - The tool name (e.g., "python", "ruby")
///
/// # Returns
/// The interpreter command to use (e.g., "python3" for "python")
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn get_interpreter_command(tool: &str) -> String {
    match tool {
        "python" => "python3".to_string(),
        "ruby" => "ruby".to_string(),
        "perl" => "perl".to_string(),
        "php" => "php".to_string(),
        "node" => "node".to_string(),
        _ => tool.to_string(),
    }
}

/// Get platform-specific installation instructions for an interpreter
///
/// # Arguments
/// * `interpreter` - The interpreter name (e.g., "python3", "ruby")
///
/// # Returns
/// A user-friendly error message with installation instructions
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn interpreter_not_found_message(interpreter: &str) -> String {
    let tool_name = match interpreter {
        "python3" | "python" => "Python",
        "ruby" => "Ruby",
        "perl" => "Perl",
        "php" => "PHP",
        "node" | "nodejs" => "Node.js",
        _ => interpreter,
    };

    let install_instructions = get_platform_install_instructions(interpreter);

    format!(
        "{} interpreter not found.\n\n{}\n\nAfter installation, restart Script Kit.",
        tool_name, install_instructions
    )
}

/// Get platform-specific installation instructions
///
/// # Arguments
/// * `interpreter` - The interpreter name
///
/// # Returns
/// Platform-specific installation command suggestions
#[allow(dead_code)] // Used by interpreter_not_found_message
fn get_platform_install_instructions(interpreter: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        get_macos_install_instructions(interpreter)
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_install_instructions(interpreter)
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_install_instructions(interpreter)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        format!(
            "Please install {} using your system's package manager.",
            interpreter
        )
    }
}

/// Get macOS installation instructions (Homebrew)
#[cfg(target_os = "macos")]
#[allow(dead_code)] // Used by get_platform_install_instructions
fn get_macos_install_instructions(interpreter: &str) -> String {
    let brew_package = match interpreter {
        "python3" | "python" => "python",
        "ruby" => "ruby",
        "perl" => "perl",
        "php" => "php",
        "node" | "nodejs" => "node",
        _ => interpreter,
    };

    format!(
        "Install using Homebrew:\n  brew install {}\n\nOr download from the official website.",
        brew_package
    )
}

/// Get Linux installation instructions (apt/dnf)
#[cfg(target_os = "linux")]
fn get_linux_install_instructions(interpreter: &str) -> String {
    let (apt_package, dnf_package) = match interpreter {
        "python3" | "python" => ("python3", "python3"),
        "ruby" => ("ruby", "ruby"),
        "perl" => ("perl", "perl"),
        "php" => ("php", "php-cli"),
        "node" | "nodejs" => ("nodejs", "nodejs"),
        _ => (interpreter, interpreter),
    };

    format!(
        "Install using your package manager:\n\n  Debian/Ubuntu:\n    sudo apt install {}\n\n  Fedora/RHEL:\n    sudo dnf install {}",
        apt_package, dnf_package
    )
}

/// Get Windows installation instructions
#[cfg(target_os = "windows")]
fn get_windows_install_instructions(interpreter: &str) -> String {
    let (choco_package, download_url) = match interpreter {
        "python3" | "python" => ("python", "https://www.python.org/downloads/"),
        "ruby" => ("ruby", "https://rubyinstaller.org/"),
        "perl" => ("strawberryperl", "https://strawberryperl.com/"),
        "php" => ("php", "https://windows.php.net/download/"),
        "node" | "nodejs" => ("nodejs", "https://nodejs.org/"),
        _ => (interpreter, ""),
    };

    if download_url.is_empty() {
        format!(
            "Install using Chocolatey:\n  choco install {}",
            choco_package
        )
    } else {
        format!(
            "Install using Chocolatey:\n  choco install {}\n\nOr download from:\n  {}",
            choco_package, download_url
        )
    }
}

/// Check if a tool is an interpreter tool
///
/// # Arguments
/// * `tool` - The tool name to check
///
/// # Returns
/// `true` if the tool requires an external interpreter
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn is_interpreter_tool(tool: &str) -> bool {
    INTERPRETER_TOOLS.contains(&tool)
}

/// Get the file extension for a given interpreter tool
///
/// # Arguments
/// * `tool` - The tool name
///
/// # Returns
/// The appropriate file extension for scripts of that type
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn get_interpreter_extension(tool: &str) -> &'static str {
    match tool {
        "python" => "py",
        "ruby" => "rb",
        "perl" => "pl",
        "php" => "php",
        "node" => "js",
        _ => "txt",
    }
}

/// Validate that a tool name is a known interpreter
///
/// # Arguments
/// * `tool` - The tool name to validate
///
/// # Returns
/// `Ok(())` if valid, `Err` with descriptive message if not
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn validate_interpreter_tool(tool: &str) -> Result<(), String> {
    if is_interpreter_tool(tool) {
        Ok(())
    } else if VALID_TOOLS.contains(&tool) {
        Err(format!(
            "'{}' is a valid tool but not an interpreter tool",
            tool
        ))
    } else {
        Err(format!("'{}' is not a recognized tool type", tool))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Type and Constant Tests
    // ========================================

    #[test]
    fn test_valid_tools_contains_common_tools() {
        assert!(VALID_TOOLS.contains(&"bash"));
        assert!(VALID_TOOLS.contains(&"python"));
        assert!(VALID_TOOLS.contains(&"ts"));
        assert!(VALID_TOOLS.contains(&"js"));
        assert!(VALID_TOOLS.contains(&"kit"));
        assert!(VALID_TOOLS.contains(&"paste"));
        assert!(VALID_TOOLS.contains(&"template"));
    }

    #[test]
    fn test_shell_tools_contains_shells() {
        assert!(SHELL_TOOLS.contains(&"bash"));
        assert!(SHELL_TOOLS.contains(&"zsh"));
        assert!(SHELL_TOOLS.contains(&"sh"));
        assert!(SHELL_TOOLS.contains(&"fish"));
        assert!(SHELL_TOOLS.contains(&"powershell"));
    }

    #[test]
    fn test_scriptlet_new_basic() {
        let scriptlet = Scriptlet::new(
            "My Test Script".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );

        assert_eq!(scriptlet.name, "My Test Script");
        assert_eq!(scriptlet.command, "my-test-script");
        assert_eq!(scriptlet.tool, "bash");
        assert_eq!(scriptlet.scriptlet_content, "echo hello");
        assert!(scriptlet.inputs.is_empty());
    }

    #[test]
    fn test_scriptlet_new_with_inputs() {
        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "ts".to_string(),
            "const name = '{{name}}'; const age = {{age}};".to_string(),
        );

        assert_eq!(scriptlet.inputs.len(), 2);
        assert!(scriptlet.inputs.contains(&"name".to_string()));
        assert!(scriptlet.inputs.contains(&"age".to_string()));
    }

    #[test]
    fn test_scriptlet_is_shell() {
        let bash = Scriptlet::new("test".to_string(), "bash".to_string(), "echo".to_string());
        let ts = Scriptlet::new(
            "test".to_string(),
            "ts".to_string(),
            "console.log()".to_string(),
        );

        assert!(bash.is_shell());
        assert!(!ts.is_shell());
    }

    #[test]
    fn test_scriptlet_is_valid_tool() {
        let valid = Scriptlet::new("test".to_string(), "bash".to_string(), "echo".to_string());
        let invalid = Scriptlet::new(
            "test".to_string(),
            "invalid_tool".to_string(),
            "echo".to_string(),
        );

        assert!(valid.is_valid_tool());
        assert!(!invalid.is_valid_tool());
    }

    // ========================================
    // Slugify Tests
    // ========================================

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My Script"), "my-script");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("test@123"), "test-123");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("Hello   World"), "hello-world");
        assert_eq!(slugify("  Leading Trailing  "), "leading-trailing");
    }

    // ========================================
    // Extract Named Inputs Tests
    // ========================================

    #[test]
    fn test_extract_named_inputs_basic() {
        let inputs = extract_named_inputs("Hello {{name}}!");
        assert_eq!(inputs, vec!["name"]);
    }

    #[test]
    fn test_extract_named_inputs_multiple() {
        let inputs = extract_named_inputs("{{first}} and {{second}}");
        assert_eq!(inputs.len(), 2);
        assert!(inputs.contains(&"first".to_string()));
        assert!(inputs.contains(&"second".to_string()));
    }

    #[test]
    fn test_extract_named_inputs_no_duplicates() {
        let inputs = extract_named_inputs("{{name}} is {{name}}");
        assert_eq!(inputs, vec!["name"]);
    }

    #[test]
    fn test_extract_named_inputs_ignores_conditionals() {
        let inputs = extract_named_inputs("{{#if flag}}{{name}}{{/if}}");
        assert_eq!(inputs, vec!["name"]);
        assert!(!inputs.contains(&"#if flag".to_string()));
        assert!(!inputs.contains(&"/if".to_string()));
    }

    #[test]
    fn test_extract_named_inputs_empty() {
        let inputs = extract_named_inputs("No placeholders here");
        assert!(inputs.is_empty());
    }

    // ========================================
    // Metadata Parsing Tests
    // ========================================

    #[test]
    fn test_parse_metadata_basic() {
        let metadata = parse_html_comment_metadata("<!-- shortcut: cmd k -->");
        assert_eq!(metadata.shortcut, Some("cmd k".to_string()));
    }

    #[test]
    fn test_parse_metadata_multiple_fields() {
        let metadata = parse_html_comment_metadata(
            "<!--\nshortcut: cmd k\ndescription: My script\ntrigger: test\n-->",
        );
        assert_eq!(metadata.shortcut, Some("cmd k".to_string()));
        assert_eq!(metadata.description, Some("My script".to_string()));
        assert_eq!(metadata.trigger, Some("test".to_string()));
    }

    #[test]
    fn test_parse_metadata_background_bool() {
        let metadata = parse_html_comment_metadata("<!-- background: true -->");
        assert_eq!(metadata.background, Some(true));

        let metadata = parse_html_comment_metadata("<!-- background: false -->");
        assert_eq!(metadata.background, Some(false));
    }

    #[test]
    fn test_parse_metadata_extra_fields() {
        let metadata = parse_html_comment_metadata("<!-- custom_field: value -->");
        assert_eq!(
            metadata.extra.get("custom_field"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_parse_metadata_empty() {
        let metadata = parse_html_comment_metadata("No comments here");
        assert!(metadata.shortcut.is_none());
        assert!(metadata.description.is_none());
    }

    #[test]
    fn test_parse_metadata_colons_in_value() {
        let metadata =
            parse_html_comment_metadata("<!-- description: Visit https://example.com for info -->");
        assert_eq!(
            metadata.description,
            Some("Visit https://example.com for info".to_string())
        );
    }

    // ========================================
    // Expand Metadata Tests
    // ========================================

    #[test]
    fn test_parse_metadata_expand_basic() {
        let metadata = parse_html_comment_metadata("<!-- expand: :sig -->");
        assert_eq!(metadata.expand, Some(":sig".to_string()));
    }

    #[test]
    fn test_parse_metadata_expand_with_punctuation() {
        let metadata = parse_html_comment_metadata("<!-- expand: !email -->");
        assert_eq!(metadata.expand, Some("!email".to_string()));
    }

    #[test]
    fn test_parse_metadata_expand_with_double_suffix() {
        // Common pattern: keyword followed by double char like "ddate,,"
        let metadata = parse_html_comment_metadata("<!-- expand: ddate,, -->");
        assert_eq!(metadata.expand, Some("ddate,,".to_string()));
    }

    #[test]
    fn test_parse_metadata_expand_with_other_fields() {
        let metadata = parse_html_comment_metadata(
            "<!--\nexpand: :addr\nshortcut: cmd e\ndescription: Insert address\n-->",
        );
        assert_eq!(metadata.expand, Some(":addr".to_string()));
        assert_eq!(metadata.shortcut, Some("cmd e".to_string()));
        assert_eq!(metadata.description, Some("Insert address".to_string()));
    }

    #[test]
    fn test_parse_metadata_expand_empty_value() {
        // Empty expand value should not be stored
        let metadata = parse_html_comment_metadata("<!-- expand: -->");
        assert_eq!(metadata.expand, None);
    }

    #[test]
    fn test_parse_markdown_scriptlet_with_expand() {
        let markdown = r#"## Email Signature

<!-- expand: :sig -->

```type
Best regards,
John Doe
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert_eq!(scriptlets[0].name, "Email Signature");
        assert_eq!(scriptlets[0].metadata.expand, Some(":sig".to_string()));
        assert_eq!(scriptlets[0].tool, "type");
    }

    #[test]
    fn test_parse_markdown_multiple_scriptlets_with_expand() {
        let markdown = r#"# Snippets

## Date Insert

<!-- expand: :date -->

```type
{{date}}
```

## Email Template

<!-- expand: !email -->

```type
Hello {{name}},
```

## No Expand

```type
Plain text
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 3);

        assert_eq!(scriptlets[0].metadata.expand, Some(":date".to_string()));
        assert_eq!(scriptlets[1].metadata.expand, Some("!email".to_string()));
        assert_eq!(scriptlets[2].metadata.expand, None);
    }

    #[test]
    fn test_expand_metadata_serialization() {
        let metadata = ScriptletMetadata {
            expand: Some(":test".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"expand\":\":test\""));

        let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.expand, Some(":test".to_string()));
    }

    #[test]
    fn test_expand_metadata_deserialization_missing() {
        // When expand is not present in JSON, it should be None
        let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
        let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.expand, None);
    }

    // ========================================
    // Alias Metadata Tests
    // ========================================

    #[test]
    fn test_parse_metadata_alias_basic() {
        let metadata = parse_html_comment_metadata("<!-- alias: goog -->");
        assert_eq!(metadata.alias, Some("goog".to_string()));
    }

    #[test]
    fn test_parse_metadata_alias_with_punctuation() {
        let metadata = parse_html_comment_metadata("<!-- alias: g! -->");
        assert_eq!(metadata.alias, Some("g!".to_string()));
    }

    #[test]
    fn test_parse_metadata_alias_with_numbers() {
        let metadata = parse_html_comment_metadata("<!-- alias: cmd123 -->");
        assert_eq!(metadata.alias, Some("cmd123".to_string()));
    }

    #[test]
    fn test_parse_metadata_alias_with_other_fields() {
        let metadata = parse_html_comment_metadata(
            "<!--\nalias: search\nshortcut: cmd s\ndescription: Search the web\n-->",
        );
        assert_eq!(metadata.alias, Some("search".to_string()));
        assert_eq!(metadata.shortcut, Some("cmd s".to_string()));
        assert_eq!(metadata.description, Some("Search the web".to_string()));
    }

    #[test]
    fn test_parse_metadata_alias_empty_value() {
        // Empty alias value should not be stored
        let metadata = parse_html_comment_metadata("<!-- alias: -->");
        assert_eq!(metadata.alias, None);
    }

    #[test]
    fn test_parse_markdown_scriptlet_with_alias() {
        let markdown = r#"## Google Search

<!-- alias: goog -->

```bash
open "https://www.google.com/search?q=$1"
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert_eq!(scriptlets[0].name, "Google Search");
        assert_eq!(scriptlets[0].metadata.alias, Some("goog".to_string()));
        assert_eq!(scriptlets[0].tool, "bash");
    }

    #[test]
    fn test_parse_markdown_multiple_scriptlets_with_alias() {
        let markdown = r#"# Launchers

## Google Search

<!-- alias: goog -->

```open
https://google.com
```

## GitHub

<!-- alias: gh -->

```open
https://github.com
```

## No Alias

```open
https://example.com
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 3);

        assert_eq!(scriptlets[0].metadata.alias, Some("goog".to_string()));
        assert_eq!(scriptlets[1].metadata.alias, Some("gh".to_string()));
        assert_eq!(scriptlets[2].metadata.alias, None);
    }

    #[test]
    fn test_alias_metadata_serialization() {
        let metadata = ScriptletMetadata {
            alias: Some("test".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"alias\":\"test\""));

        let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.alias, Some("test".to_string()));
    }

    #[test]
    fn test_alias_metadata_deserialization_missing() {
        // When alias is not present in JSON, it should be None
        let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
        let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.alias, None);
    }

    #[test]
    fn test_alias_and_expand_together() {
        // Both alias and expand can coexist on the same scriptlet
        let metadata = parse_html_comment_metadata("<!--\nalias: goog\nexpand: :google\n-->");
        assert_eq!(metadata.alias, Some("goog".to_string()));
        assert_eq!(metadata.expand, Some(":google".to_string()));
    }

    // ========================================
    // Code Block Extraction Tests
    // ========================================

    #[test]
    fn test_extract_code_block_basic_backticks() {
        let result = extract_code_block_nested("```ts\nconst x = 1;\n```");
        assert!(result.is_some());
        let (tool, code) = result.unwrap();
        assert_eq!(tool, "ts");
        assert_eq!(code, "const x = 1;");
    }

    #[test]
    fn test_extract_code_block_basic_tildes() {
        let result = extract_code_block_nested("~~~bash\necho hello\n~~~");
        assert!(result.is_some());
        let (tool, code) = result.unwrap();
        assert_eq!(tool, "bash");
        assert_eq!(code, "echo hello");
    }

    #[test]
    fn test_extract_code_block_nested_backticks_in_tildes() {
        let content = "~~~md\nHere's code:\n```ts\nconst x = 1;\n```\nDone!\n~~~";
        let result = extract_code_block_nested(content);
        assert!(result.is_some());
        let (tool, code) = result.unwrap();
        assert_eq!(tool, "md");
        assert!(code.contains("```ts"));
        assert!(code.contains("const x = 1;"));
    }

    #[test]
    fn test_extract_code_block_no_language() {
        let result = extract_code_block_nested("```\ncode here\n```");
        assert!(result.is_some());
        let (tool, code) = result.unwrap();
        assert_eq!(tool, "");
        assert_eq!(code, "code here");
    }

    #[test]
    fn test_extract_code_block_none_without_fence() {
        let result = extract_code_block_nested("No code fence here");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_code_block_multiline() {
        let result = extract_code_block_nested("```python\ndef foo():\n    return 42\n```");
        assert!(result.is_some());
        let (tool, code) = result.unwrap();
        assert_eq!(tool, "python");
        assert!(code.contains("def foo():"));
        assert!(code.contains("return 42"));
    }

    // ========================================
    // Markdown Parsing Tests
    // ========================================

    #[test]
    fn test_parse_markdown_basic_scriptlet() {
        let markdown = r#"## Test Script

```ts
console.log("hello");
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert_eq!(scriptlets[0].name, "Test Script");
        assert_eq!(scriptlets[0].tool, "ts");
        assert!(scriptlets[0].scriptlet_content.contains("console.log"));
    }

    #[test]
    fn test_parse_markdown_with_group() {
        let markdown = r#"# My Group

## Script One

```bash
echo one
```

## Script Two

```bash
echo two
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 2);
        assert_eq!(scriptlets[0].group, "My Group");
        assert_eq!(scriptlets[1].group, "My Group");
    }

    #[test]
    fn test_parse_markdown_with_metadata() {
        let markdown = r#"## Shortcut Script

<!-- shortcut: cmd k -->

```ts
console.log("triggered");
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert_eq!(scriptlets[0].metadata.shortcut, Some("cmd k".to_string()));
    }

    #[test]
    fn test_parse_markdown_with_global_prepend() {
        let markdown = r#"# Shell Scripts

```bash
#!/bin/bash
set -e
```

## Script A

```bash
echo "A"
```

## Script B

```bash
echo "B"
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 2);

        // Both should have the prepended content
        assert!(scriptlets[0].scriptlet_content.contains("#!/bin/bash"));
        assert!(scriptlets[0].scriptlet_content.contains("set -e"));
        assert!(scriptlets[0].scriptlet_content.contains("echo \"A\""));

        assert!(scriptlets[1].scriptlet_content.contains("#!/bin/bash"));
        assert!(scriptlets[1].scriptlet_content.contains("echo \"B\""));
    }

    #[test]
    fn test_parse_markdown_default_tool() {
        let markdown = r#"## No Language

```
just code
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        // Empty tool should default to "ts"
        assert_eq!(scriptlets[0].tool, "ts");
    }

    #[test]
    fn test_parse_markdown_extracts_inputs() {
        let markdown = r#"## Template

```ts
console.log("Hello {{name}}! You are {{age}} years old.");
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert!(scriptlets[0].inputs.contains(&"name".to_string()));
        assert!(scriptlets[0].inputs.contains(&"age".to_string()));
    }

    #[test]
    fn test_parse_markdown_source_path() {
        let markdown = "## Test\n\n```bash\necho\n```";
        let scriptlets = parse_markdown_as_scriptlets(markdown, Some("/path/to/file.md"));
        assert_eq!(
            scriptlets[0].source_path,
            Some("/path/to/file.md".to_string())
        );
    }

    #[test]
    fn test_parse_markdown_empty() {
        let scriptlets = parse_markdown_as_scriptlets("", None);
        assert!(scriptlets.is_empty());
    }

    #[test]
    fn test_parse_markdown_no_code_block() {
        let markdown = "## Title\n\nJust text, no code.";
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert!(scriptlets.is_empty());
    }

    // ========================================
    // Variable Substitution Tests
    // ========================================

    #[test]
    fn test_format_scriptlet_named_inputs() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), "Alice".to_string());
        inputs.insert("greeting".to_string(), "Hello".to_string());

        let result = format_scriptlet("{{greeting}}, {{name}}!", &inputs, &[], false);

        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_format_scriptlet_positional_unix() {
        let result = format_scriptlet(
            "echo $1 and $2",
            &HashMap::new(),
            &["first".to_string(), "second".to_string()],
            false,
        );

        assert_eq!(result, "echo first and second");
    }

    #[test]
    fn test_format_scriptlet_positional_windows() {
        let result = format_scriptlet(
            "echo %1 and %2",
            &HashMap::new(),
            &["first".to_string(), "second".to_string()],
            true,
        );

        assert_eq!(result, "echo first and second");
    }

    #[test]
    fn test_format_scriptlet_all_args_unix() {
        let result = format_scriptlet(
            "echo $@",
            &HashMap::new(),
            &["one".to_string(), "two".to_string(), "three".to_string()],
            false,
        );

        assert_eq!(result, r#"echo "one" "two" "three""#);
    }

    #[test]
    fn test_format_scriptlet_all_args_windows() {
        let result = format_scriptlet(
            "echo %*",
            &HashMap::new(),
            &["one".to_string(), "two".to_string()],
            true,
        );

        assert_eq!(result, r#"echo "one" "two""#);
    }

    #[test]
    fn test_format_scriptlet_combined() {
        let mut inputs = HashMap::new();
        inputs.insert("prefix".to_string(), "Result:".to_string());

        let result = format_scriptlet(
            "{{prefix}} $1 and $2",
            &inputs,
            &["A".to_string(), "B".to_string()],
            false,
        );

        assert_eq!(result, "Result: A and B");
    }

    #[test]
    fn test_format_scriptlet_escape_quotes() {
        let result = format_scriptlet(
            "echo $@",
            &HashMap::new(),
            &["has\"quote".to_string()],
            false,
        );

        assert_eq!(result, r#"echo "has\"quote""#);
    }

    // ========================================
    // Conditional Processing Tests
    // ========================================

    #[test]
    fn test_process_conditionals_if_true() {
        let mut flags = HashMap::new();
        flags.insert("show".to_string(), true);

        let result = process_conditionals("{{#if show}}visible{{/if}}", &flags);
        assert_eq!(result, "visible");
    }

    #[test]
    fn test_process_conditionals_if_false() {
        let mut flags = HashMap::new();
        flags.insert("show".to_string(), false);

        let result = process_conditionals("{{#if show}}visible{{/if}}", &flags);
        assert_eq!(result, "");
    }

    #[test]
    fn test_process_conditionals_if_missing_flag() {
        let flags = HashMap::new();

        let result = process_conditionals("{{#if undefined}}visible{{/if}}", &flags);
        assert_eq!(result, "");
    }

    #[test]
    fn test_process_conditionals_if_else_true() {
        let mut flags = HashMap::new();
        flags.insert("flag".to_string(), true);

        let result = process_conditionals("{{#if flag}}yes{{else}}no{{/if}}", &flags);
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_process_conditionals_if_else_false() {
        let mut flags = HashMap::new();
        flags.insert("flag".to_string(), false);

        let result = process_conditionals("{{#if flag}}yes{{else}}no{{/if}}", &flags);
        assert_eq!(result, "no");
    }

    #[test]
    fn test_process_conditionals_else_if() {
        let mut flags = HashMap::new();
        flags.insert("a".to_string(), false);
        flags.insert("b".to_string(), true);

        let result = process_conditionals("{{#if a}}A{{else if b}}B{{else}}C{{/if}}", &flags);
        assert_eq!(result, "B");
    }

    #[test]
    fn test_process_conditionals_nested() {
        let mut flags = HashMap::new();
        flags.insert("outer".to_string(), true);
        flags.insert("inner".to_string(), true);

        let result =
            process_conditionals("{{#if outer}}[{{#if inner}}nested{{/if}}]{{/if}}", &flags);
        assert_eq!(result, "[nested]");
    }

    #[test]
    fn test_process_conditionals_preserves_other_content() {
        let mut flags = HashMap::new();
        flags.insert("show".to_string(), true);

        let result = process_conditionals("Before {{#if show}}middle{{/if}} after", &flags);
        assert_eq!(result, "Before middle after");
    }

    #[test]
    fn test_process_conditionals_with_variables() {
        let mut flags = HashMap::new();
        flags.insert("useTitle".to_string(), true);

        let result = process_conditionals("{{#if useTitle}}Hello {{name}}{{/if}}", &flags);
        assert_eq!(result, "Hello {{name}}");
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_full_scriptlet_workflow() {
        let markdown = r#"# Tools

## Greeter

<!-- 
description: Greets a person
shortcut: cmd g
-->

```ts
const name = "{{name}}";
{{#if formal}}console.log(`Dear ${name}`);{{else}}console.log(`Hey ${name}!`);{{/if}}
```
"#;

        let scriptlets = parse_markdown_as_scriptlets(markdown, Some("/test.md"));
        assert_eq!(scriptlets.len(), 1);

        let scriptlet = &scriptlets[0];
        assert_eq!(scriptlet.name, "Greeter");
        assert_eq!(scriptlet.group, "Tools");
        assert_eq!(
            scriptlet.metadata.description,
            Some("Greets a person".to_string())
        );
        assert_eq!(scriptlet.metadata.shortcut, Some("cmd g".to_string()));
        assert!(scriptlet.inputs.contains(&"name".to_string()));

        // Test variable substitution
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), "Alice".to_string());

        let mut flags = HashMap::new();
        flags.insert("formal".to_string(), true);

        let content = process_conditionals(&scriptlet.scriptlet_content, &flags);
        let result = format_scriptlet(&content, &inputs, &[], false);

        assert!(result.contains("Alice"));
        assert!(result.contains("Dear"));
        assert!(!result.contains("Hey"));
    }

    #[test]
    fn test_complex_markdown_parsing() {
        let markdown = r#"# Productivity

## Open URL

<!-- shortcut: cmd u -->

```open
https://example.com
```

## Type Date

<!-- expand: ddate,, -->

```type
{{#if iso}}{{date}}{{else}}{{formattedDate}}{{/if}}
```

# Development

```bash
# Common setup
export PATH="$HOME/bin:$PATH"
```

## Run Tests

```bash
npm test $@
```

## Build

```bash
npm run build $1
```
"#;

        let scriptlets = parse_markdown_as_scriptlets(markdown, None);

        // Should have 4 scriptlets: Open URL, Type Date, Run Tests, Build
        assert_eq!(scriptlets.len(), 4);

        // First two belong to "Productivity" group
        assert_eq!(scriptlets[0].group, "Productivity");
        assert_eq!(scriptlets[0].name, "Open URL");
        assert_eq!(scriptlets[0].tool, "open");

        assert_eq!(scriptlets[1].group, "Productivity");
        assert_eq!(scriptlets[1].name, "Type Date");
        assert_eq!(scriptlets[1].metadata.expand, Some("ddate,,".to_string()));

        // Last two belong to "Development" group and have the common setup prepended
        assert_eq!(scriptlets[2].group, "Development");
        assert_eq!(scriptlets[2].name, "Run Tests");
        assert!(scriptlets[2].scriptlet_content.contains("export PATH"));
        assert!(scriptlets[2].scriptlet_content.contains("npm test"));

        assert_eq!(scriptlets[3].group, "Development");
        assert_eq!(scriptlets[3].name, "Build");
        assert!(scriptlets[3].scriptlet_content.contains("export PATH"));
    }

    #[test]
    fn test_scriptlet_metadata_serialization() {
        let metadata = ScriptletMetadata {
            shortcut: Some("cmd k".to_string()),
            description: Some("Test".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.shortcut, deserialized.shortcut);
        assert_eq!(metadata.description, deserialized.description);
    }

    #[test]
    fn test_scriptlet_serialization() {
        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );

        let json = serde_json::to_string(&scriptlet).unwrap();
        let deserialized: Scriptlet = serde_json::from_str(&json).unwrap();

        assert_eq!(scriptlet.name, deserialized.name);
        assert_eq!(scriptlet.tool, deserialized.tool);
        assert_eq!(scriptlet.scriptlet_content, deserialized.scriptlet_content);
    }

    // ========================================
    // Interpreter Tool Tests
    // ========================================

    #[test]
    fn test_interpreter_tools_constant() {
        // Verify all expected interpreters are in the list
        assert!(INTERPRETER_TOOLS.contains(&"python"));
        assert!(INTERPRETER_TOOLS.contains(&"ruby"));
        assert!(INTERPRETER_TOOLS.contains(&"perl"));
        assert!(INTERPRETER_TOOLS.contains(&"php"));
        assert!(INTERPRETER_TOOLS.contains(&"node"));

        // Verify count
        assert_eq!(INTERPRETER_TOOLS.len(), 5);
    }

    #[test]
    fn test_is_interpreter_tool() {
        // Positive cases
        assert!(is_interpreter_tool("python"));
        assert!(is_interpreter_tool("ruby"));
        assert!(is_interpreter_tool("perl"));
        assert!(is_interpreter_tool("php"));
        assert!(is_interpreter_tool("node"));

        // Negative cases - shell tools
        assert!(!is_interpreter_tool("bash"));
        assert!(!is_interpreter_tool("sh"));
        assert!(!is_interpreter_tool("zsh"));

        // Negative cases - other tools
        assert!(!is_interpreter_tool("ts"));
        assert!(!is_interpreter_tool("kit"));
        assert!(!is_interpreter_tool("open"));
        assert!(!is_interpreter_tool("paste"));
        assert!(!is_interpreter_tool("unknown"));
    }

    #[test]
    fn test_get_interpreter_command() {
        // Python uses python3
        assert_eq!(get_interpreter_command("python"), "python3");

        // Others use their direct name
        assert_eq!(get_interpreter_command("ruby"), "ruby");
        assert_eq!(get_interpreter_command("perl"), "perl");
        assert_eq!(get_interpreter_command("php"), "php");
        assert_eq!(get_interpreter_command("node"), "node");

        // Unknown returns as-is
        assert_eq!(get_interpreter_command("unknown"), "unknown");
    }

    #[test]
    fn test_get_interpreter_extension() {
        assert_eq!(get_interpreter_extension("python"), "py");
        assert_eq!(get_interpreter_extension("ruby"), "rb");
        assert_eq!(get_interpreter_extension("perl"), "pl");
        assert_eq!(get_interpreter_extension("php"), "php");
        assert_eq!(get_interpreter_extension("node"), "js");

        // Unknown returns txt
        assert_eq!(get_interpreter_extension("unknown"), "txt");
    }

    #[test]
    fn test_validate_interpreter_tool_valid() {
        assert!(validate_interpreter_tool("python").is_ok());
        assert!(validate_interpreter_tool("ruby").is_ok());
        assert!(validate_interpreter_tool("perl").is_ok());
        assert!(validate_interpreter_tool("php").is_ok());
        assert!(validate_interpreter_tool("node").is_ok());
    }

    #[test]
    fn test_validate_interpreter_tool_non_interpreter() {
        // bash is valid but not an interpreter tool
        let result = validate_interpreter_tool("bash");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not an interpreter tool"));
    }

    #[test]
    fn test_validate_interpreter_tool_unknown() {
        let result = validate_interpreter_tool("unknown_tool");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a recognized tool type"));
    }

    #[test]
    fn test_interpreter_not_found_message_python() {
        let msg = interpreter_not_found_message("python3");

        // Should contain the tool name
        assert!(msg.contains("Python"));
        assert!(msg.contains("interpreter not found"));

        // Should have installation instructions
        #[cfg(target_os = "macos")]
        {
            assert!(msg.contains("brew install python"));
        }
        #[cfg(target_os = "linux")]
        {
            assert!(msg.contains("apt install python3") || msg.contains("dnf install python3"));
        }
        #[cfg(target_os = "windows")]
        {
            assert!(msg.contains("choco install python"));
        }

        // Should mention restart
        assert!(msg.contains("restart Script Kit"));
    }

    #[test]
    fn test_interpreter_not_found_message_ruby() {
        let msg = interpreter_not_found_message("ruby");

        assert!(msg.contains("Ruby"));
        assert!(msg.contains("interpreter not found"));

        #[cfg(target_os = "macos")]
        {
            assert!(msg.contains("brew install ruby"));
        }
    }

    #[test]
    fn test_interpreter_not_found_message_node() {
        let msg = interpreter_not_found_message("node");

        assert!(msg.contains("Node.js"));
        assert!(msg.contains("interpreter not found"));

        #[cfg(target_os = "macos")]
        {
            assert!(msg.contains("brew install node"));
        }
    }

    #[test]
    fn test_interpreter_not_found_message_perl() {
        let msg = interpreter_not_found_message("perl");

        assert!(msg.contains("Perl"));
        assert!(msg.contains("interpreter not found"));
    }

    #[test]
    fn test_interpreter_not_found_message_php() {
        let msg = interpreter_not_found_message("php");

        assert!(msg.contains("PHP"));
        assert!(msg.contains("interpreter not found"));
    }

    #[test]
    fn test_interpreter_tools_are_valid_tools() {
        // All interpreter tools should also be in VALID_TOOLS
        for tool in INTERPRETER_TOOLS {
            assert!(
                VALID_TOOLS.contains(tool),
                "Interpreter tool '{}' should be in VALID_TOOLS",
                tool
            );
        }
    }

    #[test]
    fn test_interpreter_tools_disjoint_from_shell_tools() {
        // Interpreter tools should not overlap with shell tools
        for tool in INTERPRETER_TOOLS {
            assert!(
                !SHELL_TOOLS.contains(tool),
                "Interpreter tool '{}' should not be in SHELL_TOOLS",
                tool
            );
        }
    }

    #[test]
    fn test_scriptlet_with_interpreter_tool() {
        // Test creating a scriptlet with an interpreter tool
        let scriptlet = Scriptlet::new(
            "Python Script".to_string(),
            "python".to_string(),
            "print('Hello, World!')".to_string(),
        );

        assert_eq!(scriptlet.tool, "python");
        assert!(is_interpreter_tool(&scriptlet.tool));
        assert!(scriptlet.is_valid_tool());
        assert!(!scriptlet.is_shell());
    }

    #[test]
    fn test_parse_markdown_with_interpreter_tools() {
        let markdown = r#"# Scripts

## Python Hello

```python
print("Hello from Python")
```

## Ruby Greeting

```ruby
puts "Hello from Ruby"
```

## Node Script

```node
console.log("Hello from Node");
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);

        assert_eq!(scriptlets.len(), 3);

        // Python
        assert_eq!(scriptlets[0].tool, "python");
        assert!(is_interpreter_tool(&scriptlets[0].tool));
        assert!(scriptlets[0].scriptlet_content.contains("print"));

        // Ruby
        assert_eq!(scriptlets[1].tool, "ruby");
        assert!(is_interpreter_tool(&scriptlets[1].tool));
        assert!(scriptlets[1].scriptlet_content.contains("puts"));

        // Node
        assert_eq!(scriptlets[2].tool, "node");
        assert!(is_interpreter_tool(&scriptlets[2].tool));
        assert!(scriptlets[2].scriptlet_content.contains("console.log"));
    }

    #[test]
    fn test_interpreter_extension_matches_tool_extension() {
        // get_interpreter_extension should match the tool_extension for interpreter tools
        // This ensures consistency between the two functions
        assert_eq!(get_interpreter_extension("python"), "py");
        assert_eq!(get_interpreter_extension("ruby"), "rb");
        assert_eq!(get_interpreter_extension("perl"), "pl");
        assert_eq!(get_interpreter_extension("php"), "php");
        assert_eq!(get_interpreter_extension("node"), "js");
    }

    // ========================================
    // Cron and Schedule Metadata Tests
    // ========================================

    #[test]
    fn test_parse_metadata_cron_basic() {
        let metadata = parse_html_comment_metadata("<!-- cron: */5 * * * * -->");
        assert_eq!(metadata.cron, Some("*/5 * * * *".to_string()));
    }

    #[test]
    fn test_parse_metadata_cron_hourly() {
        let metadata = parse_html_comment_metadata("<!-- cron: 0 * * * * -->");
        assert_eq!(metadata.cron, Some("0 * * * *".to_string()));
    }

    #[test]
    fn test_parse_metadata_cron_daily() {
        let metadata = parse_html_comment_metadata("<!-- cron: 0 9 * * * -->");
        assert_eq!(metadata.cron, Some("0 9 * * *".to_string()));
    }

    #[test]
    fn test_parse_metadata_cron_weekly() {
        let metadata = parse_html_comment_metadata("<!-- cron: 0 9 * * 1 -->");
        assert_eq!(metadata.cron, Some("0 9 * * 1".to_string()));
    }

    #[test]
    fn test_parse_metadata_schedule_natural_language() {
        let metadata = parse_html_comment_metadata("<!-- schedule: every hour -->");
        assert_eq!(metadata.schedule, Some("every hour".to_string()));
    }

    #[test]
    fn test_parse_metadata_schedule_every_tuesday() {
        let metadata = parse_html_comment_metadata("<!-- schedule: every tuesday at 2pm -->");
        assert_eq!(metadata.schedule, Some("every tuesday at 2pm".to_string()));
    }

    #[test]
    fn test_parse_metadata_schedule_every_day() {
        let metadata = parse_html_comment_metadata("<!-- schedule: every day at 9am -->");
        assert_eq!(metadata.schedule, Some("every day at 9am".to_string()));
    }

    #[test]
    fn test_parse_metadata_cron_with_other_fields() {
        let metadata = parse_html_comment_metadata(
            "<!--\ncron: 0 */6 * * *\ndescription: Runs every 6 hours\nbackground: true\n-->",
        );
        assert_eq!(metadata.cron, Some("0 */6 * * *".to_string()));
        assert_eq!(metadata.description, Some("Runs every 6 hours".to_string()));
        assert_eq!(metadata.background, Some(true));
    }

    #[test]
    fn test_parse_metadata_schedule_with_other_fields() {
        let metadata = parse_html_comment_metadata("<!--\nschedule: every weekday at 9am\ndescription: Morning task\nbackground: true\n-->");
        assert_eq!(metadata.schedule, Some("every weekday at 9am".to_string()));
        assert_eq!(metadata.description, Some("Morning task".to_string()));
        assert_eq!(metadata.background, Some(true));
    }

    #[test]
    fn test_parse_metadata_cron_and_schedule_together() {
        // Both can exist, though typically only one would be used
        let metadata =
            parse_html_comment_metadata("<!--\ncron: 0 9 * * *\nschedule: every day at 9am\n-->");
        assert_eq!(metadata.cron, Some("0 9 * * *".to_string()));
        assert_eq!(metadata.schedule, Some("every day at 9am".to_string()));
    }

    #[test]
    fn test_parse_metadata_cron_empty_value() {
        // Empty cron value should not be stored
        let metadata = parse_html_comment_metadata("<!-- cron: -->");
        assert_eq!(metadata.cron, None);
    }

    #[test]
    fn test_parse_metadata_schedule_empty_value() {
        // Empty schedule value should not be stored
        let metadata = parse_html_comment_metadata("<!-- schedule: -->");
        assert_eq!(metadata.schedule, None);
    }

    #[test]
    fn test_parse_markdown_scriptlet_with_cron() {
        let markdown = r#"## Hourly Backup

<!-- cron: 0 * * * * -->

```bash
backup.sh
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert_eq!(scriptlets[0].name, "Hourly Backup");
        assert_eq!(scriptlets[0].metadata.cron, Some("0 * * * *".to_string()));
        assert_eq!(scriptlets[0].tool, "bash");
    }

    #[test]
    fn test_parse_markdown_scriptlet_with_schedule() {
        let markdown = r#"## Weekly Report

<!-- schedule: every monday at 8am -->

```bash
generate-report.sh
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 1);
        assert_eq!(scriptlets[0].name, "Weekly Report");
        assert_eq!(
            scriptlets[0].metadata.schedule,
            Some("every monday at 8am".to_string())
        );
        assert_eq!(scriptlets[0].tool, "bash");
    }

    #[test]
    fn test_parse_markdown_multiple_scriptlets_with_cron_and_schedule() {
        let markdown = r#"# Scheduled Tasks

## Every 5 Minutes Check

<!-- cron: */5 * * * * -->

```bash
check-status.sh
```

## Daily Cleanup

<!-- schedule: every day at midnight -->

```bash
cleanup.sh
```

## No Schedule

```bash
manual-task.sh
```
"#;
        let scriptlets = parse_markdown_as_scriptlets(markdown, None);
        assert_eq!(scriptlets.len(), 3);

        assert_eq!(scriptlets[0].metadata.cron, Some("*/5 * * * *".to_string()));
        assert_eq!(scriptlets[0].metadata.schedule, None);

        assert_eq!(scriptlets[1].metadata.cron, None);
        assert_eq!(
            scriptlets[1].metadata.schedule,
            Some("every day at midnight".to_string())
        );

        assert_eq!(scriptlets[2].metadata.cron, None);
        assert_eq!(scriptlets[2].metadata.schedule, None);
    }

    #[test]
    fn test_cron_metadata_serialization() {
        let metadata = ScriptletMetadata {
            cron: Some("0 9 * * 1-5".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"cron\":\"0 9 * * 1-5\""));

        let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cron, Some("0 9 * * 1-5".to_string()));
    }

    #[test]
    fn test_schedule_metadata_serialization() {
        let metadata = ScriptletMetadata {
            schedule: Some("every friday at 5pm".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"schedule\":\"every friday at 5pm\""));

        let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.schedule,
            Some("every friday at 5pm".to_string())
        );
    }

    #[test]
    fn test_cron_metadata_deserialization_missing() {
        // When cron is not present in JSON, it should be None
        let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
        let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.cron, None);
    }

    #[test]
    fn test_cron_complex_expression() {
        // Test parsing complex cron expressions with ranges and lists
        let metadata = parse_html_comment_metadata("<!-- cron: 0 9,12,18 * * 1-5 -->");
        assert_eq!(metadata.cron, Some("0 9,12,18 * * 1-5".to_string()));
    }

    #[test]
    fn test_cron_six_field_expression() {
        // Some cron parsers support seconds as the first field
        let metadata = parse_html_comment_metadata("<!-- cron: 0 30 9 * * * -->");
        assert_eq!(metadata.cron, Some("0 30 9 * * *".to_string()));
    }
}
