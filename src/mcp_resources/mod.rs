//! MCP Resources Handler
//!
//! Implements MCP resources for Script Kit:
//! - `kit://state` - Current app state as JSON
//! - `scripts://` - List of available scripts
//! - `scriptlets://` - List of available scriptlets
//!
//! Resources are read-only data that clients can access without tool calls.

include!("part_000.rs");
include!("part_001.rs");
