//! MCP JSON-RPC 2.0 Protocol Handler
//!
//! Implements the JSON-RPC 2.0 protocol for MCP (Model Context Protocol).
//! Handles request parsing, method routing, and response generation.
//!
//! JSON-RPC 2.0 format:
//! - Request: {"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
//! - Success: {"jsonrpc":"2.0","id":1,"result":{"tools":[]}}
//! - Error: {"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}

// Allow from_str name - we're not implementing FromStr trait as this returns Option, not Result
#![allow(clippy::should_implement_trait)]
// Allow large error variant - JsonRpcResponse needs to carry full error info for JSON-RPC spec
#![allow(clippy::result_large_err)]
// Allow dead code - this module provides complete MCP API surface; some methods for future use
#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");
