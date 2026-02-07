// ============================================================================
// Validation Error Types
// ============================================================================

/// Error encountered during command validation.
/// Allows per-command validation with graceful degradation -
/// valid commands can still be loaded even when others fail.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CommandValidationError {
    /// Path to the source file
    pub file_path: PathBuf,
    /// Name of the command that failed (if identifiable)
    pub command_name: Option<String>,
    /// Line number where the error occurred (1-based)
    pub line_number: Option<usize>,
    /// Description of what went wrong
    pub error_message: String,
}
impl CommandValidationError {
    /// Create a new validation error
    pub fn new(
        file_path: impl Into<PathBuf>,
        command_name: Option<String>,
        line_number: Option<usize>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            command_name,
            line_number,
            error_message: error_message.into(),
        }
    }
}
impl std::fmt::Display for CommandValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.display())?;
        if let Some(line) = self.line_number {
            write!(f, ":{}", line)?;
        }
        if let Some(ref name) = self.command_name {
            write!(f, " [{}]", name)?;
        }
        write!(f, ": {}", self.error_message)
    }
}
/// Result of parsing commands from an extension file with validation.
/// Contains both successfully parsed commands and any validation errors encountered.
#[derive(Clone, Debug, Default)]
pub struct ExtensionParseResult {
    /// Successfully parsed commands
    pub commands: Vec<Command>,
    /// Validation errors for commands that failed to parse
    pub errors: Vec<CommandValidationError>,
    /// Extension-level manifest (if present)
    pub manifest: Option<ExtensionManifest>,
}
// ============================================================================
// Icon Resolution
// ============================================================================

/// Source of an icon (name or file path)
#[derive(Clone, Debug, PartialEq)]
pub enum IconSource {
    /// Named icon from built-in set
    Named(String),
    /// File path (relative or absolute)
    Path(String),
}
/// Resolve an icon value to either a named icon or file path
pub fn resolve_icon(value: &str) -> IconSource {
    if value.starts_with("./")
        || value.starts_with("/")
        || value.starts_with("../")
        || value.contains('/')
        || value.ends_with(".png")
        || value.ends_with(".svg")
        || value.ends_with(".icns")
    {
        IconSource::Path(value.to_string())
    } else {
        IconSource::Named(value.to_string())
    }
}
// ============================================================================
// Version Checking
// ============================================================================

/// Check if a version requirement is satisfied
/// Uses semver-style comparison
pub fn check_min_version(required: &str, current: &str) -> Result<(), String> {
    // Parse versions as semver
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.trim_start_matches('v').split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
            Some((major, minor, patch))
        } else {
            None
        }
    };

    let required_v = parse_version(required)
        .ok_or_else(|| format!("Invalid minVersion format: {}", required))?;
    let current_v =
        parse_version(current).ok_or_else(|| format!("Invalid current version: {}", current))?;

    if current_v >= required_v {
        Ok(())
    } else {
        Err(format!(
            "Extension requires Script Kit {} or newer (current: {})",
            required, current
        ))
    }
}
