//! Deterministic AI-relevant desktop context snapshots.
//!
//! Gathers selected text, frontmost app, menu bar, browser URL, and focused
//! window metadata into a single schema-versioned `AiContextSnapshot`.
//! Individual providers that fail produce warning strings rather than
//! failing the entire snapshot.

mod capture;
mod inspection;
mod types;

#[allow(unused_imports)] // Used via lib crate; binary only needs capture_context_snapshot_json
pub use capture::{
    capture_context_snapshot, capture_context_snapshot_json, enable_deterministic_context_capture,
};
#[allow(unused_imports)] // Public API surface for lib consumers and MCP
pub use inspection::{
    build_inspection_hud_message, build_inspection_receipt, ContextSnapshotInspectionReceipt,
};
#[allow(unused_imports)] // Public API surface for lib consumers and MCP
pub use types::{
    AiContextSnapshot, Base64PngContext, BrowserContext, CaptureContextOptions,
    FocusedWindowContext, FrontmostAppContext, MenuBarItemSummary,
    AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
};

#[cfg(test)]
mod tests;
