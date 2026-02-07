//! MCP Server Foundation
//!
//! Provides an HTTP server for MCP (Model Context Protocol) integration.
//! Features:
//! - HTTP server on localhost:43210
//! - Bearer token authentication from ~/.scriptkit/agent-token
//! - Health endpoint at GET /health
//! - Discovery file at ~/.scriptkit/server.json

// Allow dead code - ServerHandle methods provide full lifecycle API for future use
#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");
