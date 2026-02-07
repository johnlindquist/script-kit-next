#![allow(dead_code)]
//! Structured JSONL logging for AI agents and human-readable stderr output.
//!
//! This module provides dual-output logging:
//! - **JSONL to file** (~/.scriptkit/logs/script-kit-gpui.jsonl) - structured for AI agent parsing
//! - **Pretty to stderr** - human-readable for developers
//! - **Compact AI mode** (SCRIPT_KIT_AI_LOG=1) - ultra-compact line format for AI context
//!
//! # Compact AI Format
//!
//! When `SCRIPT_KIT_AI_LOG=1` is set, stderr uses compact format:
//! ```text
//! SS.mmm|L|C|message
//! ```
//! Where:
//! - SS.mmm = seconds.milliseconds within current minute (resets each minute)
//! - L = single char level (i/w/e/d/t)
//! - C = single char category code (see AGENTS.md for legend)
//!
//!
//! # JSONL Output Format
//!
//! Each line is a valid JSON object:
//! ```json
//! {"timestamp":"2024-12-25T10:30:45.123Z","level":"INFO","target":"script_kit_gpui::main","message":"Script executed","fields":{"event_type":"script_event","script_id":"abc","duration_ms":42}}
//! ```

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
include!("part_004.rs");
