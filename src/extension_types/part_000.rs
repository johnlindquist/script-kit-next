use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
// ============================================================================
// Valid Categories (Raycast-compatible)
// ============================================================================

/// Valid categories for extensions (matches Raycast's fixed set)
pub const VALID_CATEGORIES: &[&str] = &[
    "Applications",
    "Communication",
    "Data",
    "Design Tools",
    "Developer Tools",
    "Documentation",
    "Finance",
    "Fun",
    "Media",
    "News",
    "Productivity",
    "Security",
    "System",
    "Web",
    "Other",
];
// ============================================================================
// Extension Manifest (Bundle-level metadata)
// ============================================================================

/// Extension bundle metadata (YAML frontmatter)
/// Compatible with Raycast manifest for easy porting
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    // === Required for publishing ===
    /// Unique URL-safe identifier (e.g., "cleanshot")
    #[serde(default)]
    pub name: String,
    /// Display name shown in UI (e.g., "CleanShot X")
    #[serde(default)]
    pub title: String,
    /// Full description
    #[serde(default)]
    pub description: String,
    /// Icon path or icon name (supports both)
    #[serde(default)]
    pub icon: String,
    /// Author's handle/username
    #[serde(default)]
    pub author: String,
    /// License identifier (e.g., "MIT")
    #[serde(default = "default_license")]
    pub license: String,
    /// Categories for discovery
    #[serde(default)]
    pub categories: Vec<String>,
    /// Supported platforms (accept but warn if not macOS)
    #[serde(default)]
    pub platforms: Vec<String>,

    // === Optional ===
    /// Additional search keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Active contributors
    #[serde(default)]
    pub contributors: Vec<String>,
    /// Extension version
    pub version: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Extension-wide preferences
    #[serde(default)]
    pub preferences: Vec<Preference>,

    // === Script Kit specific ===
    /// Required permissions (clipboard, accessibility, etc.)
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Minimum Script Kit version (semver)
    #[serde(alias = "min_version")]
    pub min_version: Option<String>,
    /// Schema version for future format evolution
    pub manifest_version: Option<u32>,

    /// Catch-all for unknown/future Raycast fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}
fn default_license() -> String {
    "MIT".to_string()
}
impl ExtensionManifest {
    /// Check if the extension targets macOS (or targets all platforms)
    pub fn supports_macos(&self) -> bool {
        self.platforms.is_empty() || self.platforms.iter().any(|p| p.to_lowercase() == "macos")
    }

    /// Validate that all categories are valid
    pub fn validate_categories(&self) -> Result<(), Vec<String>> {
        let invalid: Vec<String> = self
            .categories
            .iter()
            .filter(|c| !VALID_CATEGORIES.contains(&c.as_str()))
            .cloned()
            .collect();

        if invalid.is_empty() {
            Ok(())
        } else {
            Err(invalid)
        }
    }
}
// ============================================================================
// Command Mode
// ============================================================================

/// Command execution mode (Raycast compatible)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandMode {
    /// Shows a UI view (default)
    #[default]
    View,
    /// Runs without UI
    NoView,
    /// Shows in menu bar
    MenuBar,
}
// ============================================================================
// Argument Types
// ============================================================================

/// Argument input type (Raycast compatible)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    /// Plain text input
    #[default]
    Text,
    /// Password input (masked)
    Password,
    /// Dropdown selection
    Dropdown,
}
/// Typed argument definition (Raycast compatible)
/// Commands can have up to 3 arguments in Raycast
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    /// Argument identifier
    pub name: String,
    /// Input type
    #[serde(rename = "type")]
    pub arg_type: ArgumentType,
    /// Placeholder text
    pub placeholder: String,
    /// Whether this argument is required
    #[serde(default)]
    pub required: bool,
    /// Options for dropdown type
    #[serde(default)]
    pub data: Vec<DropdownOption>,
}
/// Dropdown option for arguments and preferences
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DropdownOption {
    /// Display title
    pub title: String,
    /// Stored value
    pub value: String,
}
// ============================================================================
// Preference Types
// ============================================================================

/// Preference input type (Raycast compatible)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreferenceType {
    /// Single-line text field
    #[default]
    Textfield,
    /// Password field (stored securely)
    Password,
    /// Boolean checkbox
    Checkbox,
    /// Dropdown selection
    Dropdown,
    /// Application picker
    #[serde(rename = "appPicker")]
    AppPicker,
    /// File picker
    File,
    /// Directory picker
    Directory,
}
/// Preference definition (Raycast compatible)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    /// Preference identifier
    pub name: String,
    /// Display title
    pub title: String,
    /// Description/tooltip
    pub description: String,
    /// Input type
    #[serde(rename = "type")]
    pub pref_type: PreferenceType,
    /// Whether this preference is required
    #[serde(default)]
    pub required: bool,
    /// Default value
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    /// Placeholder text (for text fields)
    pub placeholder: Option<String>,
    /// Label for checkbox type
    pub label: Option<String>,
    /// Options for dropdown type
    #[serde(default)]
    pub data: Vec<DropdownOption>,
}
// ============================================================================
// Command Metadata (Per-H2 section)
// ============================================================================

/// Command metadata (per-H2 section)
/// Mirrors Raycast command properties
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata {
    // === Core fields ===
    /// Description of what the command does
    pub description: Option<String>,
    /// Subtitle shown next to title
    pub subtitle: Option<String>,
    /// Command-specific icon (overrides extension icon)
    pub icon: Option<String>,
    /// Additional search keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Command mode: "view" (default), "no-view", "menu-bar"
    #[serde(default)]
    pub mode: CommandMode,

    /// Background interval for no-view/menu-bar commands (e.g., "1m", "1h", "1d")
    pub interval: Option<String>,
    /// Cron expression (Script Kit extension)
    pub cron: Option<String>,
    /// Natural language schedule (Script Kit extension)
    pub schedule: Option<String>,

    /// Typed arguments (up to 3 in Raycast)
    #[serde(default)]
    pub arguments: Vec<Argument>,

    /// Command-level preferences (override/extend extension prefs)
    #[serde(default)]
    pub preferences: Vec<Preference>,

    /// If true, user must enable manually
    #[serde(default)]
    pub disabled_by_default: bool,

    // === Script Kit extensions ===
    /// Keyboard shortcut (e.g., "cmd shift k")
    pub shortcut: Option<String>,
    /// Alias trigger
    pub alias: Option<String>,
    /// Text expansion trigger
    pub keyword: Option<String>,
    /// Whether to hide from main list
    #[serde(default)]
    pub hidden: bool,
    /// Trigger text
    pub trigger: Option<String>,
    /// Whether to run in background
    #[serde(default)]
    pub background: bool,
    /// File paths to watch
    pub watch: Option<String>,
    /// System event trigger
    pub system: Option<String>,

    /// Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
// ============================================================================
// Command (formerly Scriptlet)
// ============================================================================

/// A command parsed from an extension file (formerly called Scriptlet)
///
/// Each H2 section in an extension markdown file becomes a Command.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Command {
    /// Display name of the command (from H2 header)
    pub name: String,
    /// URL-safe identifier (slugified name)
    pub command: String,
    /// Tool/language type (bash, python, ts, etc.)
    pub tool: String,
    /// The actual code content
    pub content: String,
    /// Named input placeholders (e.g., ["variableName", "otherVar"])
    pub inputs: Vec<String>,
    /// Group name (from H1 header)
    pub group: String,
    /// HTML preview content (if any)
    pub preview: Option<String>,
    /// Typed metadata from codefence ```metadata block (new format)
    pub typed_metadata: Option<TypedMetadata>,
    /// Schema definition from codefence ```schema block
    pub schema: Option<Schema>,
    /// The extension this command belongs to
    pub extension: Option<String>,
    /// Source file path
    pub source_path: Option<PathBuf>,

    // === Raycast-compatible command metadata ===
    /// Command metadata (mode, arguments, preferences, etc.)
    pub metadata: CommandMetadata,
}
impl Default for Command {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            tool: "ts".to_string(),
            content: String::new(),
            inputs: Vec::new(),
            group: String::new(),
            preview: None,
            typed_metadata: None,
            schema: None,
            extension: None,
            source_path: None,
            metadata: CommandMetadata::default(),
        }
    }
}
impl Command {
    /// Create a new command with minimal required fields
    pub fn new(name: String, tool: String, content: String) -> Self {
        let command = slugify(&name);
        let inputs = extract_named_inputs(&content);

        Command {
            name,
            command,
            tool,
            content,
            inputs,
            ..Default::default()
        }
    }

    /// Check if this command uses a shell tool
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
/// Extract named input placeholders from command content
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
// ============================================================================
// Tool Constants
// ============================================================================

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
