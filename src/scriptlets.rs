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

use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;
use crate::scriptlet_metadata::parse_codefence_metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, warn};

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

// ============================================================================
// Bundle Frontmatter (YAML at top of markdown files)
// ============================================================================

/// Frontmatter metadata for a scriptlet bundle (markdown file)
/// This is parsed from YAML at the top of the file, delimited by `---`
#[allow(dead_code)] // Public API for future use
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleFrontmatter {
    /// Bundle name
    pub name: Option<String>,
    /// Bundle description
    pub description: Option<String>,
    /// Author of the bundle
    pub author: Option<String>,
    /// Default icon for scriptlets in this bundle
    pub icon: Option<String>,
    /// Any additional fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Parse YAML frontmatter from the beginning of markdown content
///
/// Frontmatter is delimited by `---` at the start and end:
/// ```markdown
/// ---
/// name: My Bundle
/// icon: Star
/// ---
/// # Content starts here
/// ```
#[allow(dead_code)] // Public API for future use
pub fn parse_bundle_frontmatter(content: &str) -> Option<BundleFrontmatter> {
    let trimmed = content.trim_start();

    // Must start with ---
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;

    let yaml_content = &after_first[..end_pos].trim();

    match serde_yaml::from_str::<BundleFrontmatter>(yaml_content) {
        Ok(fm) => Some(fm),
        Err(e) => {
            debug!(error = %e, "Failed to parse bundle frontmatter");
            None
        }
    }
}

/// Get a default icon for a tool type
#[allow(dead_code)] // Public API for future use
pub fn tool_type_to_icon(tool: &str) -> &'static str {
    match tool {
        "bash" | "zsh" | "sh" | "fish" => "terminal",
        "python" => "snake",
        "ruby" => "gem",
        "node" | "js" | "ts" | "kit" => "file-code",
        "open" => "external-link",
        "edit" => "edit",
        "paste" => "clipboard",
        "type" => "keyboard",
        "template" => "file-text",
        "transform" => "refresh-cw",
        "applescript" => "apple",
        "powershell" | "pwsh" | "cmd" => "terminal",
        "perl" => "code",
        "php" => "code",
        "deno" | "bun" => "file-code",
        _ => "file",
    }
}

/// Resolve the icon for a scriptlet using priority order:
/// 1. Scriptlet-level metadata icon
/// 2. Bundle frontmatter default icon
/// 3. Tool-type default icon
#[allow(dead_code)] // Public API for future use
pub fn resolve_scriptlet_icon(
    metadata: &ScriptletMetadata,
    frontmatter: Option<&BundleFrontmatter>,
    tool: &str,
) -> String {
    // Check scriptlet metadata first (via extra field for now)
    if let Some(icon) = metadata.extra.get("icon") {
        return icon.clone();
    }

    // Check bundle frontmatter
    if let Some(fm) = frontmatter {
        if let Some(ref icon) = fm.icon {
            return icon.clone();
        }
    }

    // Fall back to tool default
    tool_type_to_icon(tool).to_string()
}

// ============================================================================
// Validation Error Types
// ============================================================================

/// Error encountered during scriptlet validation.
/// Allows per-scriptlet validation with graceful degradation -
/// valid scriptlets can still be loaded even when others fail.
#[allow(dead_code)] // Public API for future use
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScriptletValidationError {
    /// Path to the source file
    pub file_path: PathBuf,
    /// Name of the scriptlet that failed (if identifiable)
    pub scriptlet_name: Option<String>,
    /// Line number where the error occurred (1-based)
    pub line_number: Option<usize>,
    /// Description of what went wrong
    pub error_message: String,
}

#[allow(dead_code)] // Public API for future use
impl ScriptletValidationError {
    /// Create a new validation error
    pub fn new(
        file_path: impl Into<PathBuf>,
        scriptlet_name: Option<String>,
        line_number: Option<usize>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            scriptlet_name,
            line_number,
            error_message: error_message.into(),
        }
    }
}

impl std::fmt::Display for ScriptletValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.display())?;
        if let Some(line) = self.line_number {
            write!(f, ":{}", line)?;
        }
        if let Some(ref name) = self.scriptlet_name {
            write!(f, " [{}]", name)?;
        }
        write!(f, ": {}", self.error_message)
    }
}

/// Result of parsing scriptlets from a markdown file with validation.
/// Contains both successfully parsed scriptlets and any validation errors encountered.
#[allow(dead_code)] // Public API for future use
#[derive(Clone, Debug, Default)]
pub struct ScriptletParseResult {
    /// Successfully parsed scriptlets
    pub scriptlets: Vec<Scriptlet>,
    /// Validation errors for scriptlets that failed to parse
    pub errors: Vec<ScriptletValidationError>,
    /// Bundle-level frontmatter (if present)
    pub frontmatter: Option<BundleFrontmatter>,
}

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
    pub keyword: Option<String>,
    /// Alias trigger - when user types alias + space, immediately run script
    pub alias: Option<String>,
    /// Any additional metadata key-value pairs
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// An action defined within a scriptlet via H3 header + codefence
///
/// These actions appear in the Actions Menu when the scriptlet is focused.
/// Example markdown:
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
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScriptletAction {
    /// Name from H3 header (e.g., "Copy to Clipboard")
    pub name: String,
    /// Slugified command identifier
    pub command: String,
    /// Tool type from codefence (e.g., "bash", "open", "transform")
    pub tool: String,
    /// Code content from codefence
    pub code: String,
    /// Named input placeholders (e.g., ["selection", "clipboard"])
    pub inputs: Vec<String>,
    /// Optional keyboard shortcut hint (e.g., "cmd+c")
    pub shortcut: Option<String>,
    /// Optional description
    pub description: Option<String>,
}

impl ScriptletAction {
    /// Create action ID for the Actions Menu (prefixed to avoid collisions)
    #[allow(dead_code)] // Will be used when integrating with ActionsDialog
    pub fn action_id(&self) -> String {
        format!("scriptlet_action:{}", self.command)
    }
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
    /// Parsed metadata from HTML comments (legacy format)
    pub metadata: ScriptletMetadata,
    /// Typed metadata from codefence ```metadata block (new format)
    pub typed_metadata: Option<TypedMetadata>,
    /// Schema definition from codefence ```schema block
    pub schema: Option<Schema>,
    /// The kit this scriptlet belongs to
    pub kit: Option<String>,
    /// Source file path
    pub source_path: Option<String>,
    /// Actions defined via H3 headers within this scriptlet
    pub actions: Vec<ScriptletAction>,
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
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: None,
            actions: Vec::new(),
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
                        "keyword" | "expand" => metadata.keyword = Some(value),
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
            let h3_name = trimmed.strip_prefix("### ").unwrap().trim().to_string();
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
            let h3_name = trimmed.strip_prefix("### ").unwrap().trim().to_string();
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
    let stem = md_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = md_path.parent().unwrap_or(std::path::Path::new(""));
    parent.join(format!("{}.actions.md", stem))
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
            let code_block = if let Some(ref code_block) = codefence_result.code {
                Some((code_block.language.clone(), code_block.content.clone()))
            } else {
                extract_code_block_nested(section_text)
            };

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
    let code_block = if let Some(ref code_block) = codefence_result.code {
        Some((code_block.language.clone(), code_block.content.clone()))
    } else {
        extract_code_block_nested(section_text)
    };

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
#[path = "scriptlet_tests.rs"]
mod tests;
