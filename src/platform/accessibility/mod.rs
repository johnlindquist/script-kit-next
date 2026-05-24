//! Focused text accessibility boundary for the inline agent.
//!
//! This module owns macOS truth for whole focused-field capture and mutation.
//! UI and AI code exchange session IDs and DTOs with this boundary, never raw
//! AX handles.

pub mod app_identity;
pub mod ax;
pub mod clipboard;
pub mod double_modifier_trigger;
pub mod focused_text;
pub mod geometry;
pub mod metrics;
pub mod mutation;
pub mod permissions;

#[allow(unused_imports)]
pub use self::app_identity::ActiveAppIdentity;
#[allow(unused_imports)]
pub use self::focused_text::{
    capture_focused_text_field, CaptureFocusedTextOptions, FocusedTextCapabilities,
    FocusedTextError, FocusedTextSessionId, FocusedTextSnapshot, FocusedTextTargetDescriptor,
    TextRangeUtf16,
};
#[allow(unused_imports)]
pub use self::geometry::{DisplayBounds, FocusedFieldGeometry};
#[allow(unused_imports)]
pub use self::metrics::TextMetrics;
#[allow(unused_imports)]
pub use self::mutation::{
    append_focused_text, copy_text_output, replace_focused_text, TextMutationOptions,
    TextMutationResult,
};
