//! MCP Script Namespace Tools
//!
//! Auto-generates MCP tools from Script Kit scripts that have schema definitions.
//! Scripts with `schema = { input: {...} }` are exposed as `scripts/{script-name}` tools.
//!
//! Example script:
//! ```typescript
//! // Name: Create Note
//! // Description: Creates a new note
//! schema = {
//!   input: {
//!     title: { type: "string", required: true, description: "Note title" },
//!     content: { type: "string", description: "Note content" }
//!   }
//! }
//! ```
//!
//! This becomes MCP tool: `scripts/create-note`
//! With inputSchema derived from schema.input

// Allow dead code - ScriptTool struct and generate_script_tool for future tool execution
#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");
