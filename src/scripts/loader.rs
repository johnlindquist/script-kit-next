//! Script loading from file system
//!
//! This module provides functions for loading scripts from the
//! ~/.scriptkit/*/scripts/ directories.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, instrument, warn};

use glob::glob;
use rayon::prelude::*;

use crate::setup::get_kit_path;

use super::metadata::extract_metadata_full;
use super::scriptlet_loader::extract_kit_from_path;
use super::types::Script;

/// Reads scripts from ~/.scriptkit/*/scripts/ directories
/// Returns a sorted list of Arc-wrapped Script structs for .ts and .js files
/// Returns empty vec if directory doesn't exist or is inaccessible
///
/// H1 Optimization: Returns Arc<Script> to avoid expensive clones during filter operations.
/// Uses rayon for parallel file scanning across kit directories.
#[instrument(level = "debug", skip_all)]
pub fn read_scripts() -> Vec<Arc<Script>> {
    let kit_path = get_kit_path();

    // Glob pattern to find scripts in all kits (under kit/ subdirectory)
    let pattern = kit_path.join("kit/*/scripts");
    let pattern_str = pattern.to_string_lossy().to_string();

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

    let load_started = std::time::Instant::now();

    // Read scripts from each kit's scripts directory in parallel
    let mut scripts: Vec<Arc<Script>> = script_dirs
        .par_iter()
        .flat_map_iter(|scripts_dir| read_scripts_from_dir(scripts_dir.as_path(), &kit_path))
        .collect();

    // Sort by name for deterministic ordering
    scripts.sort_by(|a, b| a.name.cmp(&b.name));

    crate::logging::log(
        "FILTER_PERF",
        &format!(
            "[SCRIPT_BODY_INDEX] scripts={} dirs={} parallel=true elapsed_ms={:.2}",
            scripts.len(),
            script_dirs.len(),
            load_started.elapsed().as_secs_f64() * 1000.0
        ),
    );

    debug!(
        count = scripts.len(),
        dirs = script_dirs.len(),
        elapsed_ms = load_started.elapsed().as_secs_f64() * 1000.0,
        "Loaded scripts from all kits with parallel body indexing"
    );
    scripts
}

/// Read scripts from a single directory.
/// Returns a Vec of loaded scripts for parallel collection.
///
/// H1 Optimization: Creates Arc-wrapped Scripts for cheap cloning.
///
/// # Arguments
/// * `scripts_dir` - Path to the scripts directory (e.g., ~/.scriptkit/kit/main/scripts)
/// * `kit_path` - Root kit path for extracting kit name (e.g., ~/.scriptkit)
pub(crate) fn read_scripts_from_dir(scripts_dir: &Path, kit_path: &Path) -> Vec<Arc<Script>> {
    let entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(scripts_dir) {
        Ok(entries) => entries.filter_map(|entry| entry.ok()).collect(),
        Err(e) => {
            warn!(
                error = %e,
                path = %scripts_dir.display(),
                "Failed to read scripts directory"
            );
            return Vec::new();
        }
    };

    entries
        .into_par_iter()
        .filter_map(|entry| load_script_entry(entry, kit_path))
        .collect()
}

/// Load a single script entry from a directory entry.
fn load_script_entry(entry: std::fs::DirEntry, kit_path: &Path) -> Option<Arc<Script>> {
    let file_metadata = entry.metadata().ok()?;
    if !file_metadata.is_file() {
        return None;
    }

    let path = entry.path();
    let ext_str = path.extension()?.to_str()?;
    if ext_str != "ts" && ext_str != "js" {
        return None;
    }

    let filename_str = path.file_stem()?.to_str()?;

    // Extract full metadata including typed and schema
    let (script_metadata, typed_metadata, schema) = extract_metadata_full(&path);

    // Use metadata name if available, otherwise filename
    let name = script_metadata
        .name
        .unwrap_or_else(|| filename_str.to_string());

    // Extract kit name from path
    let kit_name = extract_kit_from_path(&path, kit_path);

    // Read file body for content search indexing
    let body = match std::fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(e) => {
            debug!(
                error = %e,
                path = %path.display(),
                "Failed to read script body for content indexing"
            );
            None
        }
    };

    Some(Arc::new(Script {
        name,
        path: path.clone(),
        extension: ext_str.to_string(),
        description: script_metadata.description,
        icon: script_metadata.icon,
        alias: script_metadata.alias,
        shortcut: script_metadata.shortcut,
        typed_metadata,
        schema,
        kit_name,
        body,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("script-kit-gpui-{}-{}", label, nonce))
    }

    #[test]
    fn read_scripts_from_dir_reloads_updated_body_content() {
        let root = unique_test_dir("loader-body-reload");
        let scripts_dir = root.join("kit").join("main").join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir should be created for test");

        let script_path = scripts_dir.join("demo.ts");
        fs::write(&script_path, "console.log('alphaUniqueToken');\n")
            .expect("first write should succeed");

        let first = read_scripts_from_dir(&scripts_dir, &root);
        assert_eq!(first.len(), 1);
        assert_eq!(
            first[0].body.as_deref(),
            Some("console.log('alphaUniqueToken');\n")
        );

        fs::write(&script_path, "console.log('betaUniqueToken');\n")
            .expect("second write should succeed");

        let second = read_scripts_from_dir(&scripts_dir, &root);
        assert_eq!(second.len(), 1);
        assert_eq!(
            second[0].body.as_deref(),
            Some("console.log('betaUniqueToken');\n")
        );

        let _ = fs::remove_dir_all(&root);
    }
}
