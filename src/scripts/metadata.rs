//! Script metadata extraction
//!
//! This module provides functions for extracting metadata from script files,
//! including both comment-based metadata (// Name:, // Description:) and
//! typed metadata from `metadata = {...}` declarations.

use std::fs;
use std::path::PathBuf;
use tracing::debug;

use crate::metadata_parser::{extract_typed_metadata, TypedMetadata};
use crate::schema_parser::{extract_schema, Schema};

use super::types::{ScheduleMetadata, ScriptMetadata};

/// Parse a single metadata line with lenient matching
/// Supports patterns like:
/// - "//Name:Value"
/// - "//Name: Value"
/// - "// Name:Value"
/// - "// Name: Value"
/// - "//  Name:Value"
/// - "//  Name: Value"
/// - "//\tName:Value"
/// - "//\tName: Value"
///
/// Returns Some((key, value)) if the line is a valid metadata comment, None otherwise.
/// Key matching is case-insensitive.
pub fn parse_metadata_line(line: &str) -> Option<(String, String)> {
    // Must start with //
    let after_slashes = line.strip_prefix("//")?;

    // Skip any whitespace (spaces or tabs) after the slashes
    let trimmed = after_slashes.trim_start();

    // Find the colon that separates key from value
    let colon_pos = trimmed.find(':')?;

    // Key is before the colon (no spaces in key names like "Name", "Description")
    let key = trimmed[..colon_pos].trim();

    // Key must be a single word (no spaces)
    if key.is_empty() || key.contains(' ') {
        return None;
    }

    // Value is after the colon, trimmed
    let value = trimmed[colon_pos + 1..].trim();

    Some((key.to_string(), value.to_string()))
}

/// Extract metadata from script content
/// Parses lines looking for "// Name:", "// Description:", and "// Icon:" with lenient matching
/// Only checks the first 20 lines of the file
pub fn extract_script_metadata(content: &str) -> ScriptMetadata {
    let mut metadata = ScriptMetadata::default();

    for line in content.lines().take(20) {
        if let Some((key, value)) = parse_metadata_line(line) {
            match key.to_lowercase().as_str() {
                "name" => {
                    if metadata.name.is_none() && !value.is_empty() {
                        metadata.name = Some(value);
                    }
                }
                "description" => {
                    if metadata.description.is_none() && !value.is_empty() {
                        metadata.description = Some(value);
                    }
                }
                "icon" => {
                    if metadata.icon.is_none() && !value.is_empty() {
                        metadata.icon = Some(value);
                    }
                }
                "alias" => {
                    if metadata.alias.is_none() && !value.is_empty() {
                        metadata.alias = Some(value);
                    }
                }
                "shortcut" => {
                    if metadata.shortcut.is_none() && !value.is_empty() {
                        metadata.shortcut = Some(value);
                    }
                }
                _ => {} // Ignore other metadata keys for now
            }
        }
    }

    metadata
}

/// Extract full metadata from script content including typed metadata and schema
///
/// Priority order for metadata extraction:
/// 1. Try typed `metadata = {...}` first (new format)
/// 2. Fall back to `// Name:` comments (legacy format)
///
/// For fields present in typed metadata, those values take precedence.
/// For fields NOT in typed metadata but present in comments, comment values are used.
///
/// Returns (ScriptMetadata, Option<TypedMetadata>, Option<Schema>)
pub fn extract_full_metadata(
    content: &str,
) -> (ScriptMetadata, Option<TypedMetadata>, Option<Schema>) {
    // Extract typed metadata first
    let typed_result = extract_typed_metadata(content);
    let typed_meta = typed_result.metadata;

    // Extract schema
    let schema_result = extract_schema(content);
    let schema = schema_result.schema;

    // Extract comment-based metadata as fallback
    let comment_meta = extract_script_metadata(content);

    // Build final ScriptMetadata, preferring typed values when available
    let script_meta = if let Some(ref typed) = typed_meta {
        ScriptMetadata {
            name: typed.name.clone().or(comment_meta.name),
            description: typed.description.clone().or(comment_meta.description),
            icon: typed.icon.clone().or(comment_meta.icon),
            alias: typed.alias.clone().or(comment_meta.alias),
            shortcut: typed.shortcut.clone().or(comment_meta.shortcut),
        }
    } else {
        comment_meta
    };

    (script_meta, typed_meta, schema)
}

/// Extract metadata from script file comments
/// Looks for lines starting with "// Name:" and "// Description:" with lenient matching
pub(crate) fn extract_metadata(path: &PathBuf) -> ScriptMetadata {
    match fs::read_to_string(path) {
        Ok(content) => extract_script_metadata(&content),
        Err(e) => {
            debug!(
                error = %e,
                path = %path.display(),
                "Could not read script file for metadata extraction"
            );
            ScriptMetadata::default()
        }
    }
}

/// Extract full metadata from a script file path
/// Returns (ScriptMetadata, Option<TypedMetadata>, Option<Schema>)
pub(crate) fn extract_metadata_full(
    path: &PathBuf,
) -> (ScriptMetadata, Option<TypedMetadata>, Option<Schema>) {
    match fs::read_to_string(path) {
        Ok(content) => extract_full_metadata(&content),
        Err(e) => {
            debug!(
                error = %e,
                path = %path.display(),
                "Could not read script file for full metadata extraction"
            );
            (ScriptMetadata::default(), None, None)
        }
    }
}

/// Extract schedule metadata from script content
/// Parses lines looking for "// Cron:" and "// Schedule:" with lenient matching
/// Only checks the first 30 lines of the file
pub fn extract_schedule_metadata(content: &str) -> ScheduleMetadata {
    let mut metadata = ScheduleMetadata::default();

    for line in content.lines().take(30) {
        if let Some((key, value)) = parse_metadata_line(line) {
            match key.to_lowercase().as_str() {
                "cron" => {
                    if metadata.cron.is_none() && !value.is_empty() {
                        metadata.cron = Some(value);
                    }
                }
                "schedule" => {
                    if metadata.schedule.is_none() && !value.is_empty() {
                        metadata.schedule = Some(value);
                    }
                }
                _ => {} // Ignore other metadata keys
            }
        }
    }

    metadata
}

/// Extract schedule metadata from a script file path
pub fn extract_schedule_metadata_from_file(path: &PathBuf) -> ScheduleMetadata {
    match fs::read_to_string(path) {
        Ok(content) => extract_schedule_metadata(&content),
        Err(e) => {
            debug!(
                error = %e,
                path = %path.display(),
                "Could not read script file for schedule metadata extraction"
            );
            ScheduleMetadata::default()
        }
    }
}
