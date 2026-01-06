//! Fallback commands module
//!
//! Provides Raycast-style fallback commands that appear when no scripts match
//! the user's input. Includes built-in fallbacks and support for custom fallbacks.

pub mod builtins;
pub mod collector;

#[allow(unused_imports)]
pub use builtins::{
    get_applicable_fallbacks, get_builtin_fallbacks, BuiltinFallback, FallbackAction,
    FallbackCondition,
};
#[allow(unused_imports)]
pub use collector::{
    collect_builtin_fallbacks, collect_fallbacks, collect_script_fallbacks, FallbackItem,
};
