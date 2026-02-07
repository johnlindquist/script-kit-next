//! MCP Server-Sent Events (SSE) Streaming and Audit Logging
//!
//! Provides:
//! - SSE streaming for real-time event delivery to clients
//! - Audit logging for tool calls to ~/.scriptkit/logs/mcp-audit.jsonl
//!
//! Event format: `event: {type}\ndata: {json}\n\n`

// Allow dead code - SSE streaming and audit logging infrastructure for future features
#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");
