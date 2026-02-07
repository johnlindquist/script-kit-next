//! File Search Module using macOS Spotlight (mdfind)
//!
//! This module provides file search functionality using macOS's mdfind command,
//! which interfaces with the Spotlight index for fast file searching.
//!
//! # Streaming API
//!
//! For real-time search UX, use `search_files_streaming()` with a cancel token.
//! This allows:
//! - Cancellation of in-flight searches when query changes
//! - Batched UI updates without blocking on full results
//! - Proper cleanup of mdfind processes
//!
//! # Performance Notes
//!
//! - Metadata (size, modified) is fetched per-result which can be slow
//! - For faster "time to first result", consider skipping metadata in streaming mode
//!   and hydrating it lazily for visible rows only

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
include!("part_004.rs");
