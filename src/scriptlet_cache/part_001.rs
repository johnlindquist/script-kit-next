/// Format a HUD message for scriptlet validation errors
///
/// # Returns
/// A user-friendly message suitable for HUD display
///
/// # Examples
/// - Single error: "Failed to parse 'My Script' in snippets.md"
/// - Multiple errors in one file: "Failed to parse 2 scriptlet(s) in snippets.md"
/// - Multiple files: "Parse errors in 3 file(s). Check logs for details."
pub fn format_parse_error_message(errors: &[ScriptletValidationError]) -> String {
    if errors.is_empty() {
        return String::new();
    }

    // Group errors by file path
    let mut by_file: HashMap<&Path, Vec<&ScriptletValidationError>> = HashMap::new();
    for error in errors {
        by_file.entry(&error.file_path).or_default().push(error);
    }

    let file_count = by_file.len();

    if file_count == 1 {
        // Single file - show more detail
        let (path, file_errors) = by_file.iter().next().unwrap();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

        if file_errors.len() == 1 {
            // Single error in single file - show scriptlet name if available
            if let Some(ref name) = file_errors[0].scriptlet_name {
                format!("Failed to parse '{}' in {}", name, filename)
            } else {
                format!("Failed to parse scriptlet in {}", filename)
            }
        } else {
            // Multiple errors in single file
            format!(
                "Failed to parse {} scriptlet(s) in {}",
                file_errors.len(),
                filename
            )
        }
    } else {
        // Multiple files with errors
        let total_errors: usize = by_file.values().map(|v| v.len()).sum();
        format!(
            "Parse errors in {} file(s) ({} total). Check logs.",
            file_count, total_errors
        )
    }
}
/// Log validation errors to JSONL for debugging
///
/// Logs each error with structured fields for easy filtering:
/// - category: "SCRIPTLET_PARSE"
/// - file_path: Source file
/// - scriptlet_name: Name of failing scriptlet (if known)
/// - line_number: Line where error occurred (if known)
/// - error_message: Description of the error
pub fn log_validation_errors(errors: &[ScriptletValidationError]) {
    use crate::logging;

    for error in errors {
        let scriptlet_info = error
            .scriptlet_name
            .as_ref()
            .map(|n| format!(" [{}]", n))
            .unwrap_or_default();

        let line_info = error
            .line_number
            .map(|l| format!(":{}", l))
            .unwrap_or_default();

        let message = format!(
            "{}{}{}:{}",
            error.file_path.display(),
            line_info,
            scriptlet_info,
            error.error_message
        );

        // Use the logging module to log to JSONL
        logging::log("SCRIPTLET_PARSE", &message);

        // Also log with tracing for structured fields
        tracing::warn!(
            category = "SCRIPTLET_PARSE",
            file_path = %error.file_path.display(),
            scriptlet_name = ?error.scriptlet_name,
            line_number = ?error.line_number,
            error_message = %error.error_message,
            "Scriptlet validation error"
        );
    }
}
/// Convert a Scriptlet to a CachedScriptlet for caching
pub fn scriptlet_to_cached(
    scriptlet: &crate::scriptlets::Scriptlet,
    file_path: &Path,
) -> CachedScriptlet {
    // Create file_path with anchor from scriptlet.command (the kebab-case identifier)
    // Note: This uses `command` not `name` - command is the stable identifier for anchors
    let anchor = scriptlet.command.clone();
    let full_path = format!("{}#{}", file_path.display(), anchor);

    CachedScriptlet::new(
        &scriptlet.name,
        scriptlet.metadata.shortcut.clone(),
        scriptlet
            .typed_metadata
            .as_ref()
            .and_then(|t| t.keyword.clone())
            .or(scriptlet.metadata.keyword.clone()),
        scriptlet.metadata.alias.clone(),
        full_path,
    )
}
/// Load and cache scriptlets from a markdown file with validation
///
/// This is the main entry point for loading scriptlets with error handling.
/// It:
/// 1. Parses the file using `parse_scriptlets_with_validation`
/// 2. Logs any validation errors to JSONL
/// 3. Returns the parse result for HUD notification and caching
///
/// # Arguments
/// * `content` - The markdown file content
/// * `file_path` - Path to the source file (for error reporting)
///
/// # Returns
/// `ScriptletParseResult` containing valid scriptlets and any errors
pub fn load_scriptlets_with_validation(content: &str, file_path: &Path) -> ScriptletParseResult {
    use crate::scriptlets::parse_scriptlets_with_validation;

    let source_path = file_path.to_str();
    let result = parse_scriptlets_with_validation(content, source_path);

    // Log any errors to JSONL
    if !result.errors.is_empty() {
        log_validation_errors(&result.errors);

        tracing::info!(
            category = "SCRIPTLET_PARSE",
            file_path = %file_path.display(),
            valid_count = result.scriptlets.len(),
            error_count = result.errors.len(),
            "Loaded scriptlets with {} valid, {} errors",
            result.scriptlets.len(),
            result.errors.len()
        );
    }

    result
}
/// Summary of errors suitable for creating cache + HUD notification
pub struct ParseErrorSummary {
    /// User-friendly message for HUD display
    pub hud_message: String,
    /// Total number of errors
    pub error_count: usize,
    /// Path to log file for "Open Logs" action
    pub log_file_path: PathBuf,
}
/// Create a summary of parse errors for HUD notification
///
/// # Arguments
/// * `errors` - Validation errors from parsing
///
/// # Returns
/// `Some(ParseErrorSummary)` if there are errors, `None` otherwise
pub fn create_error_summary(errors: &[ScriptletValidationError]) -> Option<ParseErrorSummary> {
    if errors.is_empty() {
        return None;
    }

    Some(ParseErrorSummary {
        hud_message: format_parse_error_message(errors),
        error_count: errors.len(),
        log_file_path: get_log_file_path(),
    })
}
