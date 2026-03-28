//! Agent Client Protocol (ACP) integration.
//!
//! Provides a generic transport layer for communicating with ACP-compatible
//! AI coding agents (Claude Code, Gemini CLI, Codex, OpenCode, etc.).
//!
//! # Module Layout
//!
//! - `config` — `AcpAgentConfig` for agent discovery and command configuration.
//! - `types` — Bridging types between ACP and Script Kit internals.
//! - `handlers` — Client-side handler implementing the ACP `Client` trait.
//! - `client` — ACP runtime: subprocess lifecycle, initialize, session/prompt loop.

pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod handlers;
pub(crate) mod types;

pub(crate) use client::AcpRuntime;
pub(crate) use config::AcpAgentConfig;
