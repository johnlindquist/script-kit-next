//! VSCode snippet syntax parser for template() SDK function
//!
//! Parses snippet syntax into a structured data model for tabstop navigation.
//!
//! Supported syntax:
//! - `$1`, `$2`, `$3` - Simple tabstops (numbered positions)
//! - `${1:default}` - Tabstops with placeholder text
//! - `${1|a,b,c|}` - Choice tabstops (dropdown options)
//! - `$0` - Final cursor position
//! - `$$` - Escaped literal dollar sign

/// Represents a parsed part of a snippet template
include!("part_000.rs");
include!("part_001.rs");
