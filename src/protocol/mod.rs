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
//!
//! # API Visibility
//!
//! Parsing classification internals remain crate-private to avoid leaking parser
//! implementation details as a public contract.
//!
//! ```compile_fail
//! use script_kit_gpui::protocol::parse_message_graceful;
//! use script_kit_gpui::protocol::ParseResult;
//! ```

#![allow(dead_code)]

mod io;
mod message;
mod semantic_id;
mod types;

#[allow(unused_imports)]
pub use io::{serialize_message, JsonlReader, ParseIssueKind};
#[allow(unused_imports)]
pub use message::{capabilities, Message};
#[allow(unused_imports)]
pub use semantic_id::{generate_semantic_id, generate_semantic_id_named, value_to_slug};
#[allow(unused_imports)]
pub use types::{
    AiChatInfo, AiMessageInfo, BoxModelSides, ChatMessagePosition, ChatMessageRole,
    ChatPromptConfig, ChatPromptMessage, Choice, ClipboardAction, ClipboardEntryType,
    ClipboardFormat, ClipboardHistoryAction, ClipboardHistoryEntryData, ComputedBoxModel,
    ComputedFlexStyle, DisplayInfo, ElementInfo, ElementType, ExecOptions, Field,
    FileSearchResultEntry, GridColorScheme, GridDepthOption, GridOptions, LayoutBounds,
    LayoutComponentInfo, LayoutComponentType, LayoutInfo, MenuBarItemData, MouseAction, MouseData,
    ProtocolAction, ScriptErrorData, ScriptletData, ScriptletMetadataData, SubmitValue,
    SystemWindowInfo, TargetWindowBounds, TilePosition, WindowActionType,
};
