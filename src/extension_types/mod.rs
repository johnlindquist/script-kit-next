//! Extension type definitions for Raycast-compatible extensions
//!
//! This module defines the core types for the extension system:
//! - `ExtensionManifest`: Bundle-level metadata (YAML frontmatter)
//! - `CommandMetadata`: Per-command metadata (H2 section metadata)
//! - `Command`: A runnable command within an extension (formerly `Scriptlet`)
//! - Supporting types: `Preference`, `Argument`, `CommandMode`, etc.
//!
//! # Terminology
//! - **Extension**: A markdown file containing one or more commands
//! - **Command**: An individual runnable entry (H2 section) - formerly called "Scriptlet"
//! - **Manifest**: The YAML frontmatter at the top of an extension file

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
