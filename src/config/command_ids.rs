//! Command ID parsing, building, validation, and deeplink round-tripping.
//!
//! Canonical command IDs use the format `{category}/{identifier}` where category
//! is one of: `builtin`, `app`, `script`, `scriptlet`.

use anyhow::{anyhow, bail, Result};

/// The supported command ID categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Builtin,
    App,
    Script,
    Scriptlet,
}

/// All runtime-supported command categories.
pub const SUPPORTED_COMMAND_CATEGORIES: &[CommandCategory] = &[
    CommandCategory::Builtin,
    CommandCategory::App,
    CommandCategory::Script,
    CommandCategory::Scriptlet,
];

impl CommandCategory {
    /// Returns the string prefix for this category.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::App => "app",
            Self::Script => "script",
            Self::Scriptlet => "scriptlet",
        }
    }
}

/// Parse a command ID into its category and identifier.
///
/// Valid format: `{category}/{identifier}` where category is one of the
/// supported categories and identifier is non-empty.
pub fn parse_command_id(value: &str) -> Result<(CommandCategory, &str)> {
    let (category, identifier) = value
        .split_once('/')
        .ok_or_else(|| anyhow!("command id must be {{category}}/{{identifier}}: {}", value))?;

    if identifier.is_empty() {
        bail!("command id identifier cannot be empty: {}", value);
    }

    let category = match category {
        "builtin" => CommandCategory::Builtin,
        "app" => CommandCategory::App,
        "script" => CommandCategory::Script,
        "scriptlet" => CommandCategory::Scriptlet,
        _ => bail!("unsupported command id category: {}", category),
    };

    Ok((category, identifier))
}

/// Check if a string is a valid canonical command ID.
pub fn is_valid_command_id(value: &str) -> bool {
    parse_command_id(value).is_ok()
}

/// Build a canonical command ID from a category and identifier.
pub fn build_command_id(category: CommandCategory, identifier: &str) -> Result<String> {
    if identifier.is_empty() {
        bail!("identifier cannot be empty");
    }
    Ok(format!("{}/{}", category.as_str(), identifier))
}

/// Extract the bare identifier from a builtin value, stripping any `builtin/` or `builtin-` prefix.
pub fn normalize_builtin_identifier(value: &str) -> &str {
    let value = value.strip_prefix("builtin/").unwrap_or(value);
    value.strip_prefix("builtin-").unwrap_or(value)
}

/// Convert any builtin ID form to canonical `builtin/{identifier}`.
///
/// Handles:
/// - `"builtin-clipboard-history"` → `"builtin/clipboard-history"`
/// - `"clipboard-history"` → `"builtin/clipboard-history"`
/// - `"builtin/clipboard-history"` → `"builtin/clipboard-history"` (no-op)
pub fn canonical_builtin_command_id(value: &str) -> String {
    format!("builtin/{}", normalize_builtin_identifier(value))
}

/// Convert a command ID to its deeplink URL.
///
/// Format: `scriptkit://commands/{command_id}`
pub fn command_id_to_deeplink(value: &str) -> Result<String> {
    parse_command_id(value)?;
    Ok(format!("scriptkit://commands/{}", value))
}

/// Extract a command ID from a deeplink URL.
///
/// Expects format: `scriptkit://commands/{command_id}`
pub fn command_id_from_deeplink(url: &str) -> Result<String> {
    let command_id = url
        .strip_prefix("scriptkit://commands/")
        .ok_or_else(|| anyhow!("unsupported deeplink: {}", url))?;
    parse_command_id(command_id)?;
    Ok(command_id.to_string())
}
