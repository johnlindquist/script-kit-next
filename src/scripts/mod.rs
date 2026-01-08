//! Scripts module - Script and scriptlet management
//!
//! This module provides functionality for:
//! - Loading scripts from ~/.scriptkit/*/scripts/
//! - Loading scriptlets from ~/.scriptkit/*/scriptlets/
//! - Fuzzy search across scripts, scriptlets, built-ins, apps, and windows
//! - Grouping results by frecency and type
//! - Registering scheduled scripts
//!
//! # Module Structure
//!
//! - `types` - Core data types (Script, Scriptlet, SearchResult, etc.)
//! - `metadata` - Metadata extraction from script files
//! - `loader` - Script loading from file system
//! - `scriptlet_loader` - Scriptlet loading and parsing
//! - `search` - Fuzzy search functionality
//! - `grouping` - Result grouping for the main menu
//! - `scheduling` - Script scheduling registration

#![allow(dead_code)]

mod grouping;
pub mod input_detection;
mod loader;
mod metadata;
mod scheduling;
mod scriptlet_loader;
pub mod search;
mod types;

// Re-export core types (always used)
pub use types::{AgentMatch, FallbackConfig, Script, Scriptlet, SearchResult};

// Re-export loader functions (always used)
pub use loader::read_scripts;

// Re-export scriptlet loader functions (always used)
pub use scriptlet_loader::{load_scriptlets, read_scriptlets, read_scriptlets_from_file};

// Re-export search functions (always used)
pub use search::{
    compute_match_indices_for_result, fuzzy_search_unified, fuzzy_search_unified_all, NucleoCtx,
};

// Re-export grouping functions (always used)
pub use grouping::get_grouped_results;

// Re-export scheduling functions (always used)
pub use scheduling::register_scheduled_scripts;

// Additional re-exports needed by tests (only compiled when testing)
#[cfg(test)]
pub use types::{BuiltInMatch, MatchIndices, ScriptMatch, ScriptletMatch, WindowMatch};

#[cfg(test)]
pub use metadata::{extract_full_metadata, extract_script_metadata, parse_metadata_line};

#[cfg(test)]
pub use search::{
    fuzzy_search_builtins, fuzzy_search_scriptlets, fuzzy_search_scripts,
    fuzzy_search_unified_with_builtins, fuzzy_search_unified_with_windows, fuzzy_search_windows,
};

// Re-export external types needed by tests via super::*
#[cfg(test)]
pub use crate::app_launcher::AppInfo;
#[cfg(test)]
pub use crate::builtins::BuiltInEntry;
#[cfg(test)]
pub use crate::frecency::FrecencyStore;
#[cfg(test)]
pub use crate::list_item::GroupedListItem;
#[cfg(test)]
pub use std::path::PathBuf;

// Internal re-exports for tests
#[cfg(test)]
pub(crate) use scriptlet_loader::{
    build_scriptlet_file_path, extract_code_block, extract_html_comment_metadata,
    extract_kit_from_path, parse_scriptlet_section,
};
#[cfg(test)]
pub(crate) use search::{
    contains_ignore_ascii_case, extract_filename, extract_scriptlet_display_path,
    find_ignore_ascii_case, fuzzy_match_with_indices, fuzzy_match_with_indices_ascii,
    is_fuzzy_match,
};

#[cfg(test)]
#[path = "../scripts_tests.rs"]
mod tests;
