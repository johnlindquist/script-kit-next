//! Scriptlet loading and parsing
//!
//! This module provides functions for loading scriptlets from markdown files
//! in the ~/.scriptkit/kit/*/extensions/ directories.

mod loading;
mod parsing;

pub use loading::{load_scriptlets, read_scriptlets_from_file};

pub(crate) use loading::extract_kit_from_path;
pub(crate) use parsing::parse_scriptlet_section;

#[cfg(test)]
mod tests;
