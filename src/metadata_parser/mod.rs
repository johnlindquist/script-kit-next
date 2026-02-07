//! Typed metadata parser for Script Kit scripts
//!
//! Parses the new typed `metadata = { ... }` global from scripts,
//! complementing the existing comment-based metadata parser in scripts.rs.
//!
//! Example script with typed metadata:
//! ```typescript
//! metadata = {
//!   name: "Create Note",
//!   description: "Creates a new note in the notes directory",
//!   author: "John Lindquist",
//!   enter: "Create",
//!   alias: "note",
//!   tags: ["productivity", "notes"],
//!   hidden: false
//! }
//! ```

include!("part_000.rs");
include!("part_001.rs");
