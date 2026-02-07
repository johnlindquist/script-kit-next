//! Menu Cache Layer
//!
//! SQLite-backed persistence for caching application menu bar data.
//! Caches menu hierarchies by bundle_id to avoid expensive rescanning.
//! Follows the same patterns as notes/storage.rs for consistency.

include!("part_000.rs");
include!("part_001.rs");
