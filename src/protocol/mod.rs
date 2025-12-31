//! JSONL Protocol for Script Kit GPUI
//!
//! Defines message types for bidirectional communication between scripts and the GPUI app.
//! Messages are exchanged as newline-delimited JSON (JSONL), with each message tagged by a `type` field.
//!
//! # Message Categories
//!
//! ## Prompts (script → app, await user input)
//! - `arg`: Choice selection with optional search
//! - `div`: Display HTML/markdown content
//! - `editor`: Code/text editor
//! - `fields`: Multi-field form
//! - `form`: Custom form layout
//! - `path`: File/directory picker
//! - `drop`: Drag-and-drop target
//! - `hotkey`: Keyboard shortcut capture
//! - `term`: Terminal emulator
//! - `chat`, `mic`, `webcam`: Media prompts
//!
//! ## Responses (app → script)
//! - `submit`: User selection or form submission
//! - `update`: Live updates (keystrokes, selections)
//!
//! ## System Control
//! - `exit`: Terminate script
//! - `show`/`hide`: Window visibility
//! - `setPosition`, `setSize`, `setAlwaysOnTop`: Window management
//! - `setPanel`, `setPreview`, `setPrompt`, `setInput`: UI updates
//! - `setActions`, `actionTriggered`: Actions menu
//!
//! ## State Queries (request/response pattern)
//! - `getState`/`stateResult`: App state
//! - `getSelectedText`/`selectedText`: System selection
//! - `captureScreenshot`/`screenshotResult`: Window capture
//! - `getWindowBounds`/`windowBounds`: Window geometry
//! - `clipboardHistory`/`clipboardHistoryResult`: Clipboard access
//!
//! ## Scriptlets
//! - `runScriptlet`, `getScriptlets`, `scriptletList`, `scriptletResult`
//!
//! # Module Structure
//!
//! - `types`: Helper types (Choice, Field, ClipboardAction, MouseEventData, ExecOptions, etc.)
//! - `message`: The main Message enum (59+ variants) and constructors
//! - `semantic_id`: Semantic ID generation for AI-driven UX
//! - `io`: JSONL parsing with graceful error handling, serialization, streaming readers

#![allow(dead_code)]

mod io;
mod message;
mod semantic_id;
mod types;

// Re-export all public types
pub use io::*;
pub use message::*;
pub use semantic_id::*;
pub use types::*;
