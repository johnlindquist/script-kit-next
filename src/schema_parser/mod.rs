//! Schema parser for Script Kit scripts
#![allow(dead_code)]
//!
//! Parses the `schema = { input: {...}, output: {...} }` global from scripts.
//! This defines the typed interface for input() and output() functions,
//! enabling MCP tool generation and AI agent integration.
//!
//! Example script with schema:
//! ```typescript
//! schema = {
//!   input: {
//!     title: { type: "string", required: true, description: "Note title" },
//!     tags: { type: "array", items: "string", description: "Tags for the note" }
//!   },
//!   output: {
//!     path: { type: "string", description: "Path to created file" },
//!     wordCount: { type: "number" }
//!   }
//! }
//!
//! const { title, tags } = await input();
//! // ... create note ...
//! output({ path: notePath, wordCount: content.split(' ').length });
//! ```

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
