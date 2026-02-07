//! Keyword Manager - Text expansion system integration
//!
//! This module ties together all the components of the text expansion system:
//! - KeyboardMonitor: Global keystroke capture
//! - KeywordMatcher: Trigger detection with rolling buffer
//! - TextInjector: Backspace deletion + clipboard paste
//! - Scriptlets: Source of keyword triggers and replacement text
//!
//! # Architecture
//!
//! The KeywordManager:
//! 1. Loads scriptlets with `keyword` metadata from ~/.scriptkit/scriptlets/
//! 2. Registers each keyword trigger with the KeywordMatcher
//! 3. Starts the KeyboardMonitor with a callback that feeds keystrokes to the matcher
//! 4. When a match is found, performs the expansion:
//!    a. Stops keyboard monitor (avoid capturing our own keystrokes)
//!    b. Deletes trigger characters with backspaces
//!    c. Pastes replacement text via clipboard
//!    d. Resumes keyboard monitor
//!

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
