//! Process Manager Module
//!
//! Centralized process tracking for bun script processes.
#![allow(dead_code)] // Some methods reserved for future use
//!
//! This module provides:
//! - PID file at ~/.scriptkit/script-kit.pid for main app
//! - Active child PIDs file at ~/.scriptkit/active-bun-pids.json
//! - Thread-safe process registration/unregistration
//! - Orphan detection on startup
//! - Bulk kill for graceful shutdown
//!

include!("part_000.rs");
include!("part_001.rs");
