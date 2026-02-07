use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;
use crate::scriptlet_metadata::parse_codefence_metadata;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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
pub fn resolve_scriptlet_icon<'a>(
    metadata: &'a ScriptletMetadata,
    frontmatter: Option<&'a BundleFrontmatter>,
    tool: &str,
) -> Cow<'a, str> {
    // Check scriptlet metadata first (via extra field for now)
    if let Some(icon) = metadata.extra.get("icon") {
        return Cow::Borrowed(icon.as_str());
    }

    // Check bundle frontmatter
    if let Some(fm) = frontmatter {
        if let Some(ref icon) = fm.icon {
            return Cow::Borrowed(icon.as_str());
        }
    }

    // Fall back to tool default
    Cow::Borrowed(tool_type_to_icon(tool))
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
                && !inputs.iter().any(|existing| existing == trimmed)
            {
                inputs.push(trimmed.to_owned());
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
