//! Script scheduling registration
//!
//! This module provides functions for registering scripts that have
//! schedule metadata (cron expressions or natural language schedules).

use std::fs;
use std::path::PathBuf;
use tracing::{info, instrument, warn};

use glob::glob;

use crate::scheduler::Scheduler;
use crate::setup::get_kit_path;

use super::metadata::extract_schedule_metadata_from_file;

/// Scan scripts directory and register scripts with schedule metadata
///
/// Walks through ~/.sk/kit/*/scripts/ looking for .ts/.js files with
/// `// Cron:` or `// Schedule:` metadata comments, and registers them
/// with the provided scheduler.
///
/// Returns the count of scripts successfully registered.
#[instrument(level = "debug", skip(scheduler))]
pub fn register_scheduled_scripts(scheduler: &Scheduler) -> usize {
    let kit_path = get_kit_path();

    // Glob pattern to find scripts in all kits
    let pattern = kit_path.join("*/scripts");
    let pattern_str = pattern.to_string_lossy().to_string();

    // Find all kit script directories
    let script_dirs: Vec<PathBuf> = match glob(&pattern_str) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(e) => {
            warn!(error = %e, pattern = %pattern_str, "Failed to glob script directories for scheduling");
            return 0;
        }
    };

    let mut registered_count = 0;

    for scripts_dir in script_dirs {
        if !scripts_dir.exists() {
            continue;
        }

        match fs::read_dir(&scripts_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Ok(file_metadata) = entry.metadata() {
                        if file_metadata.is_file() {
                            let path = entry.path();

                            // Only process .ts and .js files
                            let is_script = path
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|ext| ext == "ts" || ext == "js")
                                .unwrap_or(false);

                            if !is_script {
                                continue;
                            }

                            // Extract schedule metadata
                            let schedule_meta = extract_schedule_metadata_from_file(&path);

                            // Skip if no schedule metadata
                            if schedule_meta.cron.is_none() && schedule_meta.schedule.is_none() {
                                continue;
                            }

                            // Register with scheduler
                            match scheduler.add_script(
                                path.clone(),
                                schedule_meta.cron.clone(),
                                schedule_meta.schedule.clone(),
                            ) {
                                Ok(()) => {
                                    registered_count += 1;
                                    info!(
                                        path = %path.display(),
                                        cron = ?schedule_meta.cron,
                                        schedule = ?schedule_meta.schedule,
                                        "Registered scheduled script"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        path = %path.display(),
                                        "Failed to register scheduled script"
                                    );
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
                    "Failed to read scripts directory for scheduling"
                );
            }
        }
    }

    if registered_count > 0 {
        info!(count = registered_count, "Registered scheduled scripts");
    }

    registered_count
}
