//! Canonical Agent Chat content-block boundary.
//!
//! New Agent Chat code should import content-block types from here rather than
//! reaching for the external `agent_client_protocol` crate directly. The crate
//! is retained purely for its content-block schema (`ContentBlock`,
//! `TextContent`, `ImageContent`); it is **not** used as an active transport.
//! Centralizing the import keeps the dependency boundary visible and makes a
//! future swap or local wrapper a single-file change.

// Forward-looking boundary: import sites are migrated to this module in a
// later slice. Re-exports may be unused until then.
#[allow(unused_imports)]
pub(crate) use agent_client_protocol::{ContentBlock, ImageContent, TextContent};
