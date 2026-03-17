//! Unified keyboard shortcut system.
//!
//! This module provides:
//! - Shortcut type definitions and parsing
//! - Hotkey compatibility (keystroke conversion, normalization)
//! - User customization persistence

mod hotkey_compat;
mod persistence;
mod types;

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

// Re-export core types (allow unused during incremental development)
#[allow(unused_imports)]
pub use types::{
    canonicalize_key, is_known_key, Modifiers, Platform, Shortcut, ShortcutParseError,
};

// Re-export hotkey compatibility functions (used by hotkeys.rs, prompt_handler.rs, etc.)
pub use hotkey_compat::{keystroke_to_shortcut, normalize_shortcut, parse_shortcut};

// Re-export persistence types
#[allow(unused_imports)]
pub use persistence::{
    default_overrides_path, get_cached_shortcut_overrides, invalidate_shortcut_cache,
    load_shortcut_overrides, remove_shortcut_override, save_shortcut_override, PersistenceError,
    ShortcutOverrides,
};
