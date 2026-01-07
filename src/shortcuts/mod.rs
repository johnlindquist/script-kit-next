//! Unified keyboard shortcut system.
//!
//! This module provides:
//! - Centralized shortcut definitions
//! - Deterministic context-aware matching
//! - Conflict detection and handling
//! - User customization support
//! - Platform-aware display formatting
//!
//! # Architecture
//!
//! The shortcut system uses an ordered context stack for deterministic routing:
//! - Most specific context (e.g., ActionsDialog) is checked first
//! - Falls through to less specific contexts (e.g., Global)
//! - Prevents "Global eats arrow keys in editor" bugs
//!
//! # Example
//!
//! ```ignore
//! use script_kit_gpui::shortcuts::{Shortcut, ShortcutParseError, ContextStack, ViewType};
//!
//! let shortcut = Shortcut::parse("cmd+shift+k")?;
//! println!("Display: {}", shortcut.display()); // ⌘⇧K on macOS
//!
//! // Build context stack from UI state
//! let stack = ContextStack::from_state(ViewType::Editor, false);
//! // Shortcuts are matched against contexts in order
//! ```

mod context;
mod hotkey_compat;
mod types;

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

// Re-export core types (allow unused during incremental development)
#[allow(unused_imports)]
pub use types::{
    canonicalize_key, is_known_key, Modifiers, Platform, Shortcut, ShortcutParseError,
};

// Re-export context types
#[allow(unused_imports)]
pub use context::{ContextStack, ShortcutContext, ViewType};

// Re-export hotkey compatibility functions (used by hotkeys.rs, prompt_handler.rs, etc.)
pub use hotkey_compat::{keystroke_to_shortcut, normalize_shortcut, parse_shortcut};
