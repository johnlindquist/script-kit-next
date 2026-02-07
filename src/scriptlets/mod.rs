//! Scriptlet parsing and variable substitution module
//!
//! This module provides comprehensive support for parsing markdown files
//! containing scriptlets (code snippets with metadata) and performing
//! variable substitution in scriptlet content.
//!
//! # Types
//! - `Scriptlet`: Full scriptlet with all metadata
//! - `ScriptletMetadata`: Parsed HTML comment metadata
//!
//! # Features
//! - Parse markdown files with H1 groups and H2 scriptlets
//! - Extract metadata from HTML comments
//! - Handle nested code fences (``` inside ~~~ and vice versa)
//! - Variable substitution with named inputs, positional args, and conditionals

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
