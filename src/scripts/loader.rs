//! Script loading from file system
//!
//! This module provides functions for loading scripts from the
//! ~/.scriptkit/*/scripts/ directories.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, instrument, warn};

use glob::glob;

use crate::setup::get_kit_path;

use super::metadata::extract_metadata_full;
use super::types::Script;

/// Extract kit name from a scripts directory path
/// e.g., ~/.scriptkit/kit/my-kit/scripts -> Some("my-kit")
pub(crate) fn extract_kit_from_scripts_dir(scripts_dir: &Path, kit_root: &Path) -> Option<String> {
    let kit_prefix = format!("{}/kit/", kit_root.to_string_lossy());
    let path_str = scripts_dir.to_string_lossy().to_string();

    if path_str.starts_with(&kit_prefix) {
        // Extract the kit name from the path
        // Path structure is: <kit_root>/kit/<kit-name>/scripts
        let relative = &path_str[kit_prefix.len()..];
        let parts: Vec<&str> = relative.split('/').collect();

        if !parts.is_empty() {
            return Some(parts[0].to_string());
        }
    }
    None
}

/// Reads scripts from ~/.scriptkit/*/scripts/ directories
/// Returns a sorted list of Arc-wrapped Script structs for .ts and .js files
/// Returns empty vec if directory doesn't exist or is inaccessible
///
/// H1 Optimization: Returns Arc<Script> to avoid expensive clones during filter operations.
#[instrument(level = "debug", skip_all)]
pub fn read_scripts() -> Vec<Arc<Script>> {
    let kit_path = get_kit_path();

    // Glob pattern to find scripts in all kits (under kit/ subdirectory)
    let pattern = kit_path.join("kit/*/scripts");
    let pattern_str = pattern.to_string_lossy().to_string();

    let mut scripts = Vec::new();

    // Find all kit script directories
    let script_dirs: Vec<PathBuf> = match glob(&pattern_str) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(e) => {
            warn!(error = %e, pattern = %pattern_str, "Failed to glob script directories");
            return vec![];
        }
    };

    if script_dirs.is_empty() {
        debug!(pattern = %pattern_str, "No script directories found");
        return vec![];
    }

    // Read scripts from each kit's scripts directory
    for scripts_dir in script_dirs {
        // Extract kit name from the directory path
        let kit_name = extract_kit_from_scripts_dir(&scripts_dir, &kit_path);
        read_scripts_from_dir(&scripts_dir, kit_name.as_deref(), &mut scripts);
    }

    // Sort by name
    scripts.sort_by(|a, b| a.name.cmp(&b.name));

    debug!(count = scripts.len(), "Loaded scripts from all kits");
    scripts
}

/// Read scripts from a single directory and append to the scripts vector
/// H1 Optimization: Creates Arc-wrapped Scripts for cheap cloning.
///
/// # Arguments
/// * `scripts_dir` - Path to the scripts directory
/// * `kit_name` - Optional kit name extracted from the directory path
/// * `scripts` - Vector to append loaded scripts to
pub(crate) fn read_scripts_from_dir(
    scripts_dir: &PathBuf,
    kit_name: Option<&str>,
    scripts: &mut Vec<Arc<Script>>,
) {
    // Read the directory contents
    match std::fs::read_dir(scripts_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(file_metadata) = entry.metadata() {
                    if file_metadata.is_file() {
                        let path = entry.path();

                        // Check extension
                        if let Some(ext) = path.extension() {
                            if let Some(ext_str) = ext.to_str() {
                                if ext_str == "ts" || ext_str == "js" {
                                    // Get filename without extension as fallback
                                    if let Some(file_name) = path.file_stem() {
                                        if let Some(filename_str) = file_name.to_str() {
                                            // Extract full metadata including typed and schema
                                            let (script_metadata, typed_metadata, schema) =
                                                extract_metadata_full(&path);

                                            // Use metadata name if available, otherwise filename
                                            let name = script_metadata
                                                .name
                                                .unwrap_or_else(|| filename_str.to_string());

                                            scripts.push(Arc::new(Script {
                                                name,
                                                path: path.clone(),
                                                extension: ext_str.to_string(),
                                                description: script_metadata.description,
                                                icon: script_metadata.icon,
                                                alias: script_metadata.alias,
                                                shortcut: script_metadata.shortcut,
                                                typed_metadata,
                                                schema,
                                                group: kit_name.map(|s| s.to_string()),
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                path = %scripts_dir.display(),
                "Failed to read scripts directory"
            );
        }
    }
}
