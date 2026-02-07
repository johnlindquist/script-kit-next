//! Scriptlet loading and parsing
//!
//! This module provides functions for loading scriptlets from markdown files
//! in the ~/.scriptkit/kit/*/extensions/ directories.

mod loading;
mod parsing;

pub use loading::{load_scriptlets, read_scriptlets, read_scriptlets_from_file};

pub(crate) use loading::{build_scriptlet_file_path, extract_kit_from_path};
pub(crate) use parsing::{
    extract_code_block, extract_html_comment_metadata, parse_scriptlet_section, slugify_name,
};

#[cfg(test)]
mod tests;
