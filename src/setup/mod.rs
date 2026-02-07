//! Script Kit environment setup and initialization.
//!
//! Ensures ~/.scriptkit exists with required directories and starter files.
//! The path can be overridden via the SK_PATH environment variable.
//! Idempotent: user-owned files are never overwritten; app-owned files may be refreshed.

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
include!("part_004.rs");
include!("part_005.rs");
include!("part_006.rs");
