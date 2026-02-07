//! Script scheduling module for cron-based and natural language script execution.
//!
//! This module provides functionality to schedule scripts for automatic execution
//! based on cron expressions or natural language schedules (e.g., "every tuesday at 2pm").
//!
//! # Metadata Keys
//! Scripts can specify schedules using two metadata formats:
//! - `// Cron: */5 * * * *` - Raw cron patterns (minute precision)
//! - `// Schedule: every tuesday at 2pm` - Natural language schedules
//!

include!("part_000.rs");
include!("part_001.rs");
