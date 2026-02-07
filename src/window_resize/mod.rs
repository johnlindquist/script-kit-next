//! Dynamic Window Resizing Module
//!
//! Handles window height for different view types in Script Kit GPUI.
//!
//! **Key Rules:**
//! - ScriptList (main window with preview): FIXED at 500px, never resizes
//! - ArgPrompt with choices: Dynamic height based on choice count (capped at 500px)
//! - ArgPrompt without choices (input only): Compact input-only height
//! - Editor/Div/Term: Full height 700px

include!("part_000.rs");
include!("part_001.rs");
