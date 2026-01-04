🧩 Packing 5 file(s)...
📝 Files selected:
  • src/protocol/semantic_id.rs
  • src/protocol/mod.rs
  • src/protocol/io.rs
  • src/protocol/types.rs
  • src/protocol/message.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 5
</notes>
</file_summary>

<directory_structure>
src/protocol/semantic_id.rs
src/protocol/mod.rs
src/protocol/io.rs
src/protocol/types.rs
src/protocol/message.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/protocol/semantic_id.rs">
//! Semantic ID generation for AI-driven UX targeting
//!
//! Provides functions to generate semantic IDs for UI elements that can be
//! used by AI agents to target specific elements in the interface.

/// Generate a semantic ID for an element.
///
/// Format: {type}:{index}:{value_slug}
///
/// # Arguments
/// * `element_type` - The element type (e.g., "choice", "button", "input")
/// * `index` - The numeric index of the element
/// * `value` - The value to convert to a slug
///
/// # Returns
/// A semantic ID string in the format: type:index:slug
pub fn generate_semantic_id(element_type: &str, index: usize, value: &str) -> String {
    let slug = value_to_slug(value);
    format!("{}:{}:{}", element_type, index, slug)
}

/// Generate a semantic ID for named elements (no index).
///
/// Format: {type}:{name}
///
/// # Arguments
/// * `element_type` - The element type (e.g., "input", "panel", "window")
/// * `name` - The name of the element
///
/// # Returns
/// A semantic ID string in the format: type:name
pub fn generate_semantic_id_named(element_type: &str, name: &str) -> String {
    let slug = value_to_slug(name);
    format!("{}:{}", element_type, slug)
}

/// Convert a value string to a URL-safe slug suitable for semantic IDs.
///
/// - Converts to lowercase
/// - Replaces spaces and underscores with hyphens
/// - Removes non-alphanumeric characters (except hyphens)
/// - Collapses multiple hyphens to single
/// - Truncates to 20 characters
/// - Removes leading/trailing hyphens
pub fn value_to_slug(value: &str) -> String {
    let slug: String = value
        .to_lowercase()
        .chars()
        .map(|c| match c {
            ' ' | '_' => '-',
            c if c.is_alphanumeric() || c == '-' => c,
            _ => '-',
        })
        .collect();

    // Collapse multiple hyphens and trim
    let mut result = String::with_capacity(20);
    let mut prev_hyphen = false;

    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen && !result.is_empty() {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }

        if result.len() >= 20 {
            break;
        }
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    // Ensure non-empty
    if result.is_empty() {
        result.push_str("item");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_slug_basic() {
        assert_eq!(value_to_slug("apple"), "apple");
        assert_eq!(value_to_slug("Apple"), "apple");
        assert_eq!(value_to_slug("APPLE"), "apple");
    }

    #[test]
    fn test_value_to_slug_spaces() {
        assert_eq!(value_to_slug("red apple"), "red-apple");
        assert_eq!(value_to_slug("red  apple"), "red-apple"); // multiple spaces
        assert_eq!(value_to_slug("  apple  "), "apple"); // leading/trailing spaces become hyphens then trimmed
    }

    #[test]
    fn test_value_to_slug_special_chars() {
        assert_eq!(value_to_slug("apple_pie"), "apple-pie");
        assert_eq!(value_to_slug("apple@pie!"), "apple-pie");
        assert_eq!(value_to_slug("hello-world"), "hello-world");
    }

    #[test]
    fn test_value_to_slug_truncation() {
        let long_value = "this is a very long value that exceeds twenty characters";
        let slug = value_to_slug(long_value);
        assert!(slug.len() <= 20);
        assert_eq!(slug, "this-is-a-very-long");
    }

    #[test]
    fn test_value_to_slug_empty() {
        assert_eq!(value_to_slug(""), "item");
        assert_eq!(value_to_slug("   "), "item");
        assert_eq!(value_to_slug("@#$%"), "item"); // all special chars
    }

    #[test]
    fn test_generate_semantic_id() {
        assert_eq!(generate_semantic_id("choice", 0, "apple"), "choice:0:apple");
        assert_eq!(
            generate_semantic_id("choice", 5, "Red Apple"),
            "choice:5:red-apple"
        );
        assert_eq!(
            generate_semantic_id("button", 1, "Submit Form"),
            "button:1:submit-form"
        );
    }

    #[test]
    fn test_generate_semantic_id_named() {
        assert_eq!(
            generate_semantic_id_named("input", "filter"),
            "input:filter"
        );
        assert_eq!(
            generate_semantic_id_named("panel", "preview"),
            "panel:preview"
        );
        assert_eq!(
            generate_semantic_id_named("window", "Main Window"),
            "window:main-window"
        );
    }
}

</file>

<file path="src/protocol/mod.rs">
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

</file>

<file path="src/protocol/io.rs">
//! Protocol I/O for JSONL message parsing and serialization
//!
//! This module provides:
//! - `parse_message` / `parse_message_graceful` for parsing JSON messages
//! - `serialize_message` for serializing messages to JSON
//! - `JsonlReader` for streaming JSONL reads

use std::io::{BufRead, BufReader, Read};
use tracing::{debug, warn};
use uuid::Uuid;

use super::message::Message;

/// Maximum length for raw JSON in logs (prevents huge base64 data in logs)
const MAX_RAW_LOG_PREVIEW: usize = 200;

/// Get a truncated preview of raw JSON for logging
pub fn log_preview(raw: &str) -> (&str, usize) {
    let len = raw.len();
    if len > MAX_RAW_LOG_PREVIEW {
        (&raw[..MAX_RAW_LOG_PREVIEW], len)
    } else {
        (raw, len)
    }
}

/// Parse a single JSONL message from a string
///
/// # Arguments
/// * `line` - A JSON string (typically one line from JSONL)
///
/// # Returns
/// * `Result<Message, serde_json::Error>` - Parsed message or deserialization error
pub fn parse_message(line: &str) -> Result<Message, serde_json::Error> {
    serde_json::from_str(line).map_err(|e| {
        // Log the raw input and error for debugging
        warn!(
            raw_input = %line,
            error = %e,
            "Failed to parse JSONL message"
        );
        e
    })
}

/// Result type for graceful message parsing
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ParseResult {
    /// Successfully parsed a known message type
    Ok(Message),
    /// Message has no "type" field
    MissingType {
        /// Truncated raw JSON for debugging
        raw: String,
    },
    /// Unknown message type value - valid JSON with a "type" field we don't recognize
    UnknownType {
        /// The unrecognized type value
        message_type: String,
        /// Truncated raw JSON for debugging
        raw: String,
    },
    /// Known message type but invalid payload (wrong field types, missing required fields)
    InvalidPayload {
        /// The message type that was recognized
        message_type: String,
        /// Serde error message describing the problem
        error: String,
        /// Truncated raw JSON for debugging
        raw: String,
    },
    /// JSON parsing failed entirely (syntax error)
    ParseError(serde_json::Error),
}

/// Structured parse issue for user-facing error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseIssueKind {
    MissingType,
    UnknownType,
    InvalidPayload,
    ParseError,
}

#[derive(Debug, Clone)]
pub struct ParseIssue {
    pub correlation_id: String,
    pub kind: ParseIssueKind,
    pub message_type: Option<String>,
    pub error: Option<String>,
    pub raw_preview: String,
    pub raw_len: usize,
}

impl ParseIssue {
    fn new(
        kind: ParseIssueKind,
        message_type: Option<String>,
        error: Option<String>,
        raw_preview: String,
        raw_len: usize,
    ) -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            kind,
            message_type,
            error,
            raw_preview,
            raw_len,
        }
    }
}

/// Parse a message with graceful handling of unknown types
///
/// Unlike `parse_message`, this function handles unknown message types
/// gracefully by returning `ParseResult::UnknownType` instead of failing.
///
/// # Classification Logic
/// - Missing "type" field → `MissingType`
/// - Unknown type value → `UnknownType`
/// - Known type with invalid payload → `InvalidPayload`
/// - Invalid JSON syntax → `ParseError`
///
/// # Arguments
/// * `line` - A JSON string (typically one line from JSONL)
///
/// # Returns
/// * `ParseResult` - Classified parse result
///
/// # Performance
/// This function uses single-parse optimization: it parses to serde_json::Value
/// first, then converts to Message. This avoids double-parsing on unknown types.
///
/// # Security
/// Raw JSON is truncated to 200 chars in logs to prevent leaking sensitive data
/// (base64 screenshots, clipboard content, etc.)
pub fn parse_message_graceful(line: &str) -> ParseResult {
    let (preview, _raw_len) = log_preview(line);

    // P1-11 FIX: Single parse - parse to Value first, then convert
    // This avoids double-parsing: previously we tried Message first, then Value on failure
    let value: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            // Don't log here - caller (JsonlReader) handles logging
            return ParseResult::ParseError(e);
        }
    };

    // Check for type field and extract it as owned String before consuming value
    let msg_type: String = match value.get("type").and_then(|t| t.as_str()) {
        Some(t) => t.to_string(),
        None => {
            // Missing type field
            return ParseResult::MissingType {
                raw: preview.to_string(),
            };
        }
    };

    // Try to convert Value to Message (consumes value)
    match serde_json::from_value::<Message>(value) {
        Ok(msg) => ParseResult::Ok(msg),
        Err(e) => {
            let error_str = e.to_string();
            // Check if this is an "unknown variant" error (unknown type)
            // vs a field/payload error (known type, bad data)
            if error_str.contains("unknown variant") {
                ParseResult::UnknownType {
                    message_type: msg_type,
                    raw: preview.to_string(),
                }
            } else {
                // Known type but invalid payload
                ParseResult::InvalidPayload {
                    message_type: msg_type,
                    error: error_str,
                    raw: preview.to_string(),
                }
            }
        }
    }
}

/// Serialize a message to JSONL format
///
/// # Arguments
/// * `msg` - The message to serialize
///
/// # Returns
/// * `Result<String, serde_json::Error>` - JSON string (without newline)
pub fn serialize_message(msg: &Message) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}

/// JSONL reader for streaming/chunked message reads
///
/// Provides utilities to read messages one at a time from a reader.
///
/// # Performance
/// Uses a reusable line buffer to avoid allocating a new String per line read.
/// The buffer is cleared and reused between reads (P1-12 optimization).
pub struct JsonlReader<R: Read> {
    reader: BufReader<R>,
    /// Reusable line buffer - cleared and reused per read to avoid allocations
    line_buffer: String,
}

impl<R: Read> JsonlReader<R> {
    /// Create a new JSONL reader
    pub fn new(reader: R) -> Self {
        JsonlReader {
            reader: BufReader::new(reader),
            // Pre-allocate reasonable capacity for typical JSON messages
            line_buffer: String::with_capacity(1024),
        }
    }

    /// Read the next message from the stream
    ///
    /// # Returns
    /// * `Ok(Some(Message))` - Successfully parsed message
    /// * `Ok(None)` - End of stream
    /// * `Err(e)` - Parse error
    pub fn next_message(&mut self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        // Use loop instead of recursion to prevent stack overflow on many empty lines
        loop {
            // P1-12 FIX: Reuse buffer instead of allocating new String each call
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer)? {
                0 => {
                    debug!("Reached end of JSONL stream");
                    return Ok(None);
                }
                bytes_read => {
                    debug!(bytes_read, "Read line from JSONL stream");
                    let trimmed = self.line_buffer.trim();
                    if trimmed.is_empty() {
                        debug!("Skipping empty line in JSONL stream");
                        continue; // Skip empty lines (loop instead of recursion)
                    }
                    let msg = parse_message(trimmed)?;
                    return Ok(Some(msg));
                }
            }
        }
    }

    /// Read the next message with graceful unknown type handling
    ///
    /// Unlike `next_message`, this method uses `parse_message_graceful` to
    /// handle unknown message types without errors. Unknown types are logged
    /// and skipped, continuing to read the next message.
    ///
    /// # Logging
    /// All logging is consolidated here (reader layer). The parse_message_graceful
    /// function does not log - it returns structured results for this layer to handle.
    ///
    /// # Returns
    /// * `Ok(Some(Message))` - Successfully parsed known message
    /// * `Ok(None)` - End of stream
    /// * `Err(e)` - IO error (not parse errors for unknown types)
    pub fn next_message_graceful(&mut self) -> Result<Option<Message>, std::io::Error> {
        self.next_message_graceful_with_handler(|_| {})
    }

    /// Read the next message with graceful unknown type handling, reporting parse issues.
    pub fn next_message_graceful_with_handler<F>(
        &mut self,
        mut on_issue: F,
    ) -> Result<Option<Message>, std::io::Error>
    where
        F: FnMut(ParseIssue),
    {
        loop {
            // P1-12 FIX: Reuse buffer instead of allocating new String each iteration
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer)? {
                0 => {
                    debug!("Reached end of JSONL stream");
                    return Ok(None);
                }
                _ => {
                    let trimmed = self.line_buffer.trim();
                    if trimmed.is_empty() {
                        debug!("Skipping empty line in JSONL stream");
                        continue;
                    }

                    // Get preview for logging (security: truncate large payloads)
                    let (preview, raw_len) = log_preview(trimmed);

                    match parse_message_graceful(trimmed) {
                        ParseResult::Ok(msg) => {
                            debug!(message_id = ?msg.id(), "Successfully parsed message");
                            return Ok(Some(msg));
                        }
                        ParseResult::MissingType { .. } => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::MissingType,
                                None,
                                None,
                                preview.to_string(),
                                raw_len,
                            );
                            warn!(
                                correlation_id = %issue.correlation_id,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping message with missing 'type' field"
                            );
                            on_issue(issue);
                            continue;
                        }
                        ParseResult::UnknownType { message_type, .. } => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::UnknownType,
                                Some(message_type.clone()),
                                None,
                                preview.to_string(),
                                raw_len,
                            );
                            warn!(
                                correlation_id = %issue.correlation_id,
                                message_type = %message_type,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping unknown message type"
                            );
                            on_issue(issue);
                            continue;
                        }
                        ParseResult::InvalidPayload {
                            message_type,
                            error,
                            ..
                        } => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::InvalidPayload,
                                Some(message_type.clone()),
                                Some(error.clone()),
                                preview.to_string(),
                                raw_len,
                            );
                            warn!(
                                correlation_id = %issue.correlation_id,
                                message_type = %message_type,
                                error = %error,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping message with invalid payload"
                            );
                            on_issue(issue);
                            continue;
                        }
                        ParseResult::ParseError(e) => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::ParseError,
                                None,
                                Some(e.to_string()),
                                preview.to_string(),
                                raw_len,
                            );
                            // Log but continue - graceful degradation
                            warn!(
                                correlation_id = %issue.correlation_id,
                                error = %e,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping malformed JSON message"
                            );
                            on_issue(issue);
                            continue;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_preview_truncation() {
        // Short string - should not be truncated
        let short = "hello";
        let (preview, len) = log_preview(short);
        assert_eq!(preview, "hello");
        assert_eq!(len, 5);

        // Long string - should be truncated to 200 chars
        let long = "a".repeat(500);
        let (preview, len) = log_preview(&long);
        assert_eq!(preview.len(), 200);
        assert_eq!(len, 500);
    }

    #[test]
    fn test_parse_message_graceful_known_type() {
        let json = r#"{"type":"arg","id":"1","placeholder":"Pick","choices":[]}"#;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::Arg { id, .. }) => {
                assert_eq!(id, "1");
            }
            _ => panic!("Expected ParseResult::Ok with Arg message"),
        }
    }

    #[test]
    fn test_parse_message_graceful_unknown_type() {
        let json = r#"{"type":"futureFeature","id":"1","data":"test"}"#;
        match parse_message_graceful(json) {
            ParseResult::UnknownType { message_type, raw } => {
                assert_eq!(message_type, "futureFeature");
                assert_eq!(raw, json);
            }
            _ => panic!("Expected ParseResult::UnknownType"),
        }
    }

    #[test]
    fn test_parse_message_graceful_invalid_json() {
        let json = "not valid json at all";
        match parse_message_graceful(json) {
            ParseResult::ParseError(_) => {}
            _ => panic!("Expected ParseResult::ParseError"),
        }
    }

    #[test]
    fn test_parse_message_graceful_missing_type_field() {
        let json = r#"{"id":"1","data":"test"}"#;
        match parse_message_graceful(json) {
            ParseResult::MissingType { raw } => {
                // raw should be truncated preview (but this is short enough to be full)
                assert!(raw.contains("id"));
            }
            other => panic!("Expected ParseResult::MissingType, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_message_graceful_invalid_payload() {
        // Known type "arg" but missing required "placeholder" field
        let json = r#"{"type":"arg","id":"1"}"#;
        match parse_message_graceful(json) {
            ParseResult::InvalidPayload {
                message_type,
                error,
                ..
            } => {
                assert_eq!(message_type, "arg");
                assert!(error.contains("placeholder")); // should mention missing field
            }
            other => panic!("Expected ParseResult::InvalidPayload, got {:?}", other),
        }
    }

    #[test]
    fn test_jsonl_reader_skips_empty_lines() {
        use std::io::Cursor;

        let jsonl = "\n{\"type\":\"beep\"}\n\n{\"type\":\"show\"}\n";
        let cursor = Cursor::new(jsonl);
        let mut reader = JsonlReader::new(cursor);

        // First message should be beep (skipping initial empty line)
        let msg1 = reader.next_message().unwrap();
        assert!(matches!(msg1, Some(Message::Beep {})));

        // Second message should be show (skipping intermediate empty lines)
        let msg2 = reader.next_message().unwrap();
        assert!(matches!(msg2, Some(Message::Show {})));

        // Should be EOF
        let msg3 = reader.next_message().unwrap();
        assert!(msg3.is_none());
    }

    #[test]
    fn test_jsonl_reader_graceful_skips_unknown() {
        use std::io::Cursor;

        let jsonl = r#"{"type":"unknownType","id":"1"}
{"type":"beep"}
{"type":"anotherUnknown","data":"test"}
{"type":"show"}
"#;
        let cursor = Cursor::new(jsonl);
        let mut reader = JsonlReader::new(cursor);

        // Should skip unknownType and return beep
        let msg1 = reader.next_message_graceful().unwrap();
        assert!(matches!(msg1, Some(Message::Beep {})));

        // Should skip anotherUnknown and return show
        let msg2 = reader.next_message_graceful().unwrap();
        assert!(matches!(msg2, Some(Message::Show {})));

        // Should be EOF
        let msg3 = reader.next_message_graceful().unwrap();
        assert!(msg3.is_none());
    }

    #[test]
    fn test_jsonl_reader_reports_invalid_payload() {
        use std::io::Cursor;

        let jsonl = r#"{"type":"arg","id":"1"}
{"type":"beep"}
"#;
        let cursor = Cursor::new(jsonl);
        let mut reader = JsonlReader::new(cursor);
        let mut issues: Vec<ParseIssue> = Vec::new();

        let msg = reader
            .next_message_graceful_with_handler(|issue| issues.push(issue))
            .unwrap();

        assert!(matches!(msg, Some(Message::Beep {})));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, ParseIssueKind::InvalidPayload);
        assert_eq!(issues[0].message_type.as_deref(), Some("arg"));
        assert!(issues[0]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("placeholder"));
    }

    // ============================================================
    // Debug Grid Message Tests
    // ============================================================

    #[test]
    fn test_show_grid_default_options() {
        let json = r#"{"type":"showGrid"}"#;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::ShowGrid { options }) => {
                assert_eq!(options.grid_size, 8); // default
                assert!(!options.show_bounds);
                assert!(!options.show_box_model);
                assert!(!options.show_alignment_guides);
            }
            other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
        }
    }

    #[test]
    fn test_show_grid_with_options() {
        let json = r#"{"type":"showGrid","gridSize":16,"showBounds":true,"showBoxModel":true}"#;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::ShowGrid { options }) => {
                assert_eq!(options.grid_size, 16);
                assert!(options.show_bounds);
                assert!(options.show_box_model);
                assert!(!options.show_alignment_guides);
            }
            other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
        }
    }

    #[test]
    fn test_show_grid_with_depth_preset() {
        use crate::protocol::types::GridDepthOption;

        let json = r#"{"type":"showGrid","depth":"all"}"#;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::ShowGrid { options }) => match options.depth {
                GridDepthOption::Preset(s) => assert_eq!(s, "all"),
                _ => panic!("Expected Preset depth"),
            },
            other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
        }
    }

    #[test]
    fn test_show_grid_with_depth_components() {
        use crate::protocol::types::GridDepthOption;

        let json = r#"{"type":"showGrid","depth":["header","list","footer"]}"#;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::ShowGrid { options }) => match options.depth {
                GridDepthOption::Components(components) => {
                    assert_eq!(components.len(), 3);
                    assert!(components.contains(&"header".to_string()));
                    assert!(components.contains(&"list".to_string()));
                    assert!(components.contains(&"footer".to_string()));
                }
                _ => panic!("Expected Components depth"),
            },
            other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
        }
    }

    #[test]
    fn test_show_grid_with_color_scheme() {
        let json = r##"{"type":"showGrid","colorScheme":{"gridLines":"#FF0000AA","promptBounds":"#00FF00"}}"##;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::ShowGrid { options }) => {
                let colors = options.color_scheme.expect("Expected color scheme");
                assert_eq!(colors.grid_lines, Some("#FF0000AA".to_string()));
                assert_eq!(colors.prompt_bounds, Some("#00FF00".to_string()));
                assert!(colors.input_bounds.is_none());
            }
            other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
        }
    }

    #[test]
    fn test_hide_grid() {
        let json = r#"{"type":"hideGrid"}"#;
        match parse_message_graceful(json) {
            ParseResult::Ok(Message::HideGrid) => {}
            other => panic!("Expected ParseResult::Ok with HideGrid, got {:?}", other),
        }
    }

    #[test]
    fn test_show_grid_roundtrip() {
        use crate::protocol::types::{GridColorScheme, GridDepthOption, GridOptions};

        let options = GridOptions {
            grid_size: 16,
            show_bounds: true,
            show_box_model: false,
            show_alignment_guides: true,
            show_dimensions: true,
            depth: GridDepthOption::Components(vec!["header".to_string(), "list".to_string()]),
            color_scheme: Some(GridColorScheme {
                grid_lines: Some("#FF0000".to_string()),
                prompt_bounds: None,
                input_bounds: None,
                button_bounds: None,
                list_bounds: None,
                padding_fill: Some("#00FF0040".to_string()),
                margin_fill: None,
                alignment_guide: None,
            }),
        };

        let msg = Message::show_grid_with_options(options);
        let serialized = serde_json::to_string(&msg).expect("Failed to serialize");

        // Verify the serialized JSON has the expected type
        assert!(serialized.contains(r##""type":"showGrid""##));
        assert!(serialized.contains(r##""gridSize":16"##));
        assert!(serialized.contains(r##""showBounds":true"##));
        assert!(serialized.contains(r##""showAlignmentGuides":true"##));

        // Deserialize back and verify
        let deserialized: Message =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        match deserialized {
            Message::ShowGrid { options } => {
                assert_eq!(options.grid_size, 16);
                assert!(options.show_bounds);
                assert!(!options.show_box_model);
                assert!(options.show_alignment_guides);
            }
            _ => panic!("Expected ShowGrid message"),
        }
    }
}

</file>

<file path="src/protocol/types.rs">
//! Protocol types for Script Kit GPUI
//!
//! Contains all the helper types used in protocol messages:
//! - Choice, Field for prompts
//! - Clipboard, Keyboard, Mouse action types
//! - ExecOptions, MouseEventData
//! - ScriptletData, ProtocolAction
//! - Element types for UI querying
//! - Error data types

use serde::{Deserialize, Serialize};

use super::semantic_id::{generate_semantic_id, generate_semantic_id_named};

/// A choice option for arg() prompts
///
/// Supports Script Kit API: name, value, and optional description.
/// Semantic IDs are generated for AI-driven UX targeting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Semantic ID for AI targeting. Format: choice:{index}:{value_slug}
    /// This field is typically generated at render time, not provided by scripts.
    #[serde(skip_serializing_if = "Option::is_none", rename = "semanticId")]
    pub semantic_id: Option<String>,
}

impl Choice {
    pub fn new(name: String, value: String) -> Self {
        Choice {
            name,
            value,
            description: None,
            semantic_id: None,
        }
    }

    pub fn with_description(name: String, value: String, description: String) -> Self {
        Choice {
            name,
            value,
            description: Some(description),
            semantic_id: None,
        }
    }

    /// Generate and set the semantic ID for this choice.
    /// Format: choice:{index}:{value_slug}
    ///
    /// The value_slug is created by:
    /// - Converting to lowercase
    /// - Replacing spaces and underscores with hyphens
    /// - Removing non-alphanumeric characters (except hyphens)
    /// - Truncating to 20 characters
    pub fn with_semantic_id(mut self, index: usize) -> Self {
        self.semantic_id = Some(generate_semantic_id("choice", index, &self.value));
        self
    }

    /// Set the semantic ID directly (for custom IDs)
    pub fn set_semantic_id(&mut self, id: String) {
        self.semantic_id = Some(id);
    }

    /// Generate the semantic ID without setting it (for external use)
    pub fn generate_id(&self, index: usize) -> String {
        generate_semantic_id("choice", index, &self.value)
    }
}

/// A field definition for form/fields prompts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl Field {
    pub fn new(name: String) -> Self {
        Field {
            name,
            label: None,
            field_type: None,
            placeholder: None,
            value: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_type(mut self, field_type: String) -> Self {
        self.field_type = Some(field_type);
        self
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = Some(placeholder);
        self
    }
}

/// Clipboard action type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardAction {
    Read,
    Write,
}

/// Clipboard format type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardFormat {
    Text,
    Image,
}

/// Keyboard action type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardAction {
    Type,
    Tap,
}

/// Mouse action type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MouseAction {
    Move,
    Click,
    SetPosition,
}

/// Clipboard entry type for clipboard history
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardEntryType {
    Text,
    Image,
}

/// Clipboard history action type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardHistoryAction {
    List,
    Pin,
    Unpin,
    Remove,
    Clear,
    #[serde(rename = "trimOversize")]
    TrimOversize,
}

/// Window action type for window management
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WindowActionType {
    Focus,
    Close,
    Minimize,
    Maximize,
    Resize,
    Move,
}

/// Mouse event data for the mouse action
///
/// Contains coordinates and optional button for click events.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MouseEventData {
    /// Move to position
    Move { x: f64, y: f64 },
    /// Click at position with optional button
    Click {
        x: f64,
        y: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
    /// Set absolute position
    SetPosition { x: f64, y: f64 },
}

impl MouseEventData {
    /// Get coordinates as (x, y) tuple
    pub fn coordinates(&self) -> (f64, f64) {
        match self {
            MouseEventData::Move { x, y } => (*x, *y),
            MouseEventData::Click { x, y, .. } => (*x, *y),
            MouseEventData::SetPosition { x, y } => (*x, *y),
        }
    }
}

/// Exec command options
///
/// Options for the exec command including working directory, environment, and timeout.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecOptions {
    /// Working directory for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Environment variables (key-value pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Whether to capture stdout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_stdout: Option<bool>,
    /// Whether to capture stderr
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_stderr: Option<bool>,
}

/// Window bounds for window management (integer-based for system windows)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TargetWindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Clipboard history entry data for list responses
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClipboardHistoryEntryData {
    #[serde(rename = "entryId")]
    pub entry_id: String,
    pub content: String,
    #[serde(rename = "contentType")]
    pub content_type: ClipboardEntryType,
    pub timestamp: String,
    pub pinned: bool,
}

/// System window information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SystemWindowInfo {
    #[serde(rename = "windowId")]
    pub window_id: u32,
    pub title: String,
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<TargetWindowBounds>,
    #[serde(rename = "isMinimized", skip_serializing_if = "Option::is_none")]
    pub is_minimized: Option<bool>,
    #[serde(rename = "isActive", skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// File search result entry
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileSearchResultEntry {
    pub path: String,
    pub name: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

/// Element type for UI element querying (getElements)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ElementType {
    Choice,
    Input,
    Button,
    Panel,
    List,
}

/// Information about a UI element returned by getElements
///
/// Contains semantic ID, type, text content, and state information
/// for AI-driven UX targeting.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ElementInfo {
    /// Semantic ID for targeting (e.g., "choice:0:apple")
    pub semantic_id: String,
    /// Element type (choice, input, button, panel, list)
    #[serde(rename = "type")]
    pub element_type: ElementType,
    /// Display text of the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Value (for choices/inputs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Whether this element is currently selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    /// Whether this element is currently focused
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
    /// Index in parent container (for list items)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

impl ElementInfo {
    /// Create a new ElementInfo for a choice element
    pub fn choice(index: usize, name: &str, value: &str, selected: bool) -> Self {
        ElementInfo {
            semantic_id: generate_semantic_id("choice", index, value),
            element_type: ElementType::Choice,
            text: Some(name.to_string()),
            value: Some(value.to_string()),
            selected: Some(selected),
            focused: None,
            index: Some(index),
        }
    }

    /// Create a new ElementInfo for an input element
    pub fn input(name: &str, value: Option<&str>, focused: bool) -> Self {
        ElementInfo {
            semantic_id: generate_semantic_id_named("input", name),
            element_type: ElementType::Input,
            text: None,
            value: value.map(|s| s.to_string()),
            selected: None,
            focused: Some(focused),
            index: None,
        }
    }

    /// Create a new ElementInfo for a button element
    pub fn button(index: usize, label: &str) -> Self {
        ElementInfo {
            semantic_id: generate_semantic_id("button", index, label),
            element_type: ElementType::Button,
            text: Some(label.to_string()),
            value: None,
            selected: None,
            focused: None,
            index: Some(index),
        }
    }

    /// Create a new ElementInfo for a panel element
    pub fn panel(name: &str) -> Self {
        ElementInfo {
            semantic_id: generate_semantic_id_named("panel", name),
            element_type: ElementType::Panel,
            text: None,
            value: None,
            selected: None,
            focused: None,
            index: None,
        }
    }

    /// Create a new ElementInfo for a list element
    pub fn list(name: &str, item_count: usize) -> Self {
        ElementInfo {
            semantic_id: generate_semantic_id_named("list", name),
            element_type: ElementType::List,
            text: Some(format!("{} items", item_count)),
            value: None,
            selected: None,
            focused: None,
            index: None,
        }
    }
}

/// Protocol action for the Actions API
///
/// Represents an action that can be displayed in the ActionsDialog.
/// The `has_action` field is CRITICAL - it determines the routing behavior:
/// - `has_action=true`: Rust sends ActionTriggered back to SDK (for actions with onAction handlers)
/// - `has_action=false`: Rust submits the value directly (for simple actions)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolAction {
    /// Display name of the action
    pub name: String,
    /// Optional description shown below the name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional keyboard shortcut (e.g., "cmd+c")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// Value to submit or pass to the action handler
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// CRITICAL: If true, send ActionTriggered to SDK; if false, submit value directly
    #[serde(default)]
    pub has_action: bool,
    /// Whether this action is visible in the list
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// Whether to close the dialog after triggering
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close: Option<bool>,
}

impl ProtocolAction {
    /// Create a new ProtocolAction with just a name
    pub fn new(name: String) -> Self {
        ProtocolAction {
            name,
            description: None,
            shortcut: None,
            value: None,
            has_action: false,
            visible: None,
            close: None,
        }
    }

    /// Default visibility is true when unset.
    /// Actions with `visible: false` should be filtered out of the UI.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible.unwrap_or(true)
    }

    /// Default close behavior is true when unset.
    /// Actions with `close: false` should keep the dialog open after triggering.
    #[inline]
    pub fn should_close(&self) -> bool {
        self.close.unwrap_or(true)
    }

    /// Create a ProtocolAction with a value that submits directly
    pub fn with_value(name: String, value: String) -> Self {
        ProtocolAction {
            name,
            description: None,
            shortcut: None,
            value: Some(value),
            has_action: false,
            visible: None,
            close: None,
        }
    }

    /// Create a ProtocolAction that triggers an SDK handler
    pub fn with_handler(name: String) -> Self {
        ProtocolAction {
            name,
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: None,
            close: None,
        }
    }
}

/// Scriptlet metadata for protocol serialization
///
/// Matches the ScriptletMetadata struct from scriptlets.rs but optimized
/// for JSON protocol transmission.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptletMetadataData {
    /// Trigger text that activates this scriptlet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// Keyboard shortcut (e.g., "cmd shift k")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// Raw cron expression (e.g., "*/5 * * * *")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// Natural language schedule (e.g., "every tuesday at 2pm") - converted to cron internally
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    /// Whether to run in background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    /// File paths to watch for changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch: Option<String>,
    /// System event to trigger on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Description of the scriptlet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Text expansion trigger (e.g., "type,,")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
}

/// Scriptlet data for protocol transmission
///
/// Represents a parsed scriptlet from markdown files, containing
/// the code content, tool type, metadata, and variable inputs.
/// Used to pass scriptlet data between Rust and SDK/bun.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptletData {
    /// Name of the scriptlet (from H2 header)
    pub name: String,
    /// Command identifier (slugified name)
    pub command: String,
    /// Tool type (bash, python, ts, etc.)
    pub tool: String,
    /// The actual code content
    pub content: String,
    /// Named input placeholders (e.g., ["variableName", "otherVar"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,
    /// Group name (from H1 header)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// HTML preview content (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    /// Parsed metadata from HTML comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ScriptletMetadataData>,
    /// The kit this scriptlet belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kit: Option<String>,
    /// Source file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// Whether this is a scriptlet.
    /// Defaults to `false` when deserialized (for backwards compatibility).
    /// The `ScriptletData::new()` constructor sets this to `true`.
    #[serde(default)]
    pub is_scriptlet: bool,
}

impl ScriptletData {
    /// Create a new ScriptletData with required fields
    pub fn new(name: String, command: String, tool: String, content: String) -> Self {
        ScriptletData {
            name,
            command,
            tool,
            content,
            inputs: Vec::new(),
            group: None,
            preview: None,
            metadata: None,
            kit: None,
            source_path: None,
            is_scriptlet: true,
        }
    }

    /// Add inputs
    pub fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Add group
    pub fn with_group(mut self, group: String) -> Self {
        self.group = Some(group);
        self
    }

    /// Add preview HTML
    pub fn with_preview(mut self, preview: String) -> Self {
        self.preview = Some(preview);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: ScriptletMetadataData) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add kit
    pub fn with_kit(mut self, kit: String) -> Self {
        self.kit = Some(kit);
        self
    }

    /// Add source path
    pub fn with_source_path(mut self, path: String) -> Self {
        self.source_path = Some(path);
        self
    }
}

// ============================================================
// DEBUG GRID OVERLAY
// ============================================================

/// Options for the debug grid overlay
///
/// Used with ShowGrid message to configure the visual debugging overlay
/// that displays grid lines, component bounds, and alignment guides.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridOptions {
    /// Grid line spacing in pixels (8 or 16)
    #[serde(default = "default_grid_size")]
    pub grid_size: u32,

    /// Show component bounding boxes with labels
    #[serde(default)]
    pub show_bounds: bool,

    /// Show CSS box model (padding/margin) visualization
    #[serde(default)]
    pub show_box_model: bool,

    /// Show alignment guides between components
    #[serde(default)]
    pub show_alignment_guides: bool,

    /// Show component dimensions in labels (e.g., "Header (500x45)")
    #[serde(default)]
    pub show_dimensions: bool,

    /// Which components to show bounds for
    /// - "prompts": Top-level prompts only
    /// - "all": All rendered elements
    /// - ["name1", "name2"]: Specific component names
    #[serde(default)]
    pub depth: GridDepthOption,

    /// Optional custom color scheme
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<GridColorScheme>,
}

fn default_grid_size() -> u32 {
    8
}

/// Depth option for grid bounds display
///
/// Controls which components have their bounding boxes shown.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GridDepthOption {
    /// Preset mode: "prompts" or "all"
    Preset(String),
    /// Specific component names to show bounds for
    Components(Vec<String>),
}

impl Default for GridDepthOption {
    fn default() -> Self {
        GridDepthOption::Preset("prompts".to_string())
    }
}

/// Custom color scheme for the debug grid overlay
///
/// All colors are in "#RRGGBBAA" or "#RRGGBB" hex format.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridColorScheme {
    /// Color for grid lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_lines: Option<String>,

    /// Color for prompt bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_bounds: Option<String>,

    /// Color for input bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_bounds: Option<String>,

    /// Color for button bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_bounds: Option<String>,

    /// Color for list bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_bounds: Option<String>,

    /// Fill color for padding visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_fill: Option<String>,

    /// Fill color for margin visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_fill: Option<String>,

    /// Color for alignment guide lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment_guide: Option<String>,
}

// ============================================================
// ERROR DATA
// ============================================================

/// Script error data for structured error reporting
///
/// Sent when a script execution fails, providing detailed error information
/// for display in the UI with actionable suggestions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptErrorData {
    /// User-friendly error message
    pub error_message: String,
    /// Raw stderr output if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_output: Option<String>,
    /// Process exit code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Parsed stack trace if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Path to the script that failed
    pub script_path: String,
    /// Actionable fix suggestions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    /// When the error occurred (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl ScriptErrorData {
    /// Create a new ScriptErrorData with required fields
    pub fn new(error_message: String, script_path: String) -> Self {
        ScriptErrorData {
            error_message,
            stderr_output: None,
            exit_code: None,
            stack_trace: None,
            script_path,
            suggestions: Vec::new(),
            timestamp: None,
        }
    }

    /// Add stderr output
    pub fn with_stderr(mut self, stderr: String) -> Self {
        self.stderr_output = Some(stderr);
        self
    }

    /// Add exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Add stack trace
    pub fn with_stack_trace(mut self, trace: String) -> Self {
        self.stack_trace = Some(trace);
        self
    }

    /// Add suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Add a single suggestion
    pub fn add_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add timestamp
    pub fn with_timestamp(mut self, timestamp: String) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

// ============================================================
// LAYOUT INFO (AI Agent Debugging)
// ============================================================

/// Computed box model for a component (padding, margin, gap)
///
/// All values are in pixels. This provides the "why" behind spacing -
/// AI agents can understand if space comes from padding, margin, or gap.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComputedBoxModel {
    /// Padding values (inner spacing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<BoxModelSides>,
    /// Margin values (outer spacing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<BoxModelSides>,
    /// Gap between flex/grid children
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
}

/// Box model sides (top, right, bottom, left)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BoxModelSides {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl BoxModelSides {
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Computed flex properties for a component
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComputedFlexStyle {
    /// Flex direction: "row" or "column"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Flex grow value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grow: Option<f32>,
    /// Flex shrink value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shrink: Option<f32>,
    /// Align items: "start", "center", "end", "stretch"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,
    /// Justify content: "start", "center", "end", "space-between", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,
}

/// Bounding rectangle in pixels
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LayoutBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Component type for categorization
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayoutComponentType {
    Prompt,
    Input,
    Button,
    List,
    ListItem,
    Header,
    #[default]
    Container,
    Panel,
    Other,
}

/// Information about a single component in the layout tree
///
/// This is the core data structure for `getLayoutInfo()`.
/// It provides everything an AI agent needs to understand "why"
/// a component is positioned/sized the way it is.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutComponentInfo {
    /// Component name/identifier
    pub name: String,
    /// Component type for categorization
    #[serde(rename = "type")]
    pub component_type: LayoutComponentType,
    /// Bounding rectangle (absolute position and size)
    pub bounds: LayoutBounds,
    /// Computed box model (padding, margin, gap)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub box_model: Option<ComputedBoxModel>,
    /// Computed flex properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex: Option<ComputedFlexStyle>,
    /// Nesting depth (0 = root, 1 = child of root, etc.)
    pub depth: u32,
    /// Parent component name (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Child component names
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    /// Human-readable explanation of why this component has its current size/position
    /// Example: "Height is 45px = padding(8) + content(28) + padding(8) + divider(1)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl LayoutComponentInfo {
    pub fn new(name: impl Into<String>, component_type: LayoutComponentType) -> Self {
        Self {
            name: name.into(),
            component_type,
            bounds: LayoutBounds::default(),
            box_model: None,
            flex: None,
            depth: 0,
            parent: None,
            children: Vec::new(),
            explanation: None,
        }
    }

    pub fn with_bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.bounds = LayoutBounds {
            x,
            y,
            width,
            height,
        };
        self
    }

    pub fn with_padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let box_model = self.box_model.get_or_insert_with(ComputedBoxModel::default);
        box_model.padding = Some(BoxModelSides {
            top,
            right,
            bottom,
            left,
        });
        self
    }

    pub fn with_margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let box_model = self.box_model.get_or_insert_with(ComputedBoxModel::default);
        box_model.margin = Some(BoxModelSides {
            top,
            right,
            bottom,
            left,
        });
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        let box_model = self.box_model.get_or_insert_with(ComputedBoxModel::default);
        box_model.gap = Some(gap);
        self
    }

    pub fn with_flex_column(mut self) -> Self {
        let flex = self.flex.get_or_insert_with(ComputedFlexStyle::default);
        flex.direction = Some("column".to_string());
        self
    }

    pub fn with_flex_row(mut self) -> Self {
        let flex = self.flex.get_or_insert_with(ComputedFlexStyle::default);
        flex.direction = Some("row".to_string());
        self
    }

    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        let flex = self.flex.get_or_insert_with(ComputedFlexStyle::default);
        flex.grow = Some(grow);
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }
}

/// Full layout information for the current UI state
///
/// Returned by `getLayoutInfo()` SDK function.
/// Contains the component tree and window-level information.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutInfo {
    /// Window dimensions
    pub window_width: f32,
    pub window_height: f32,
    /// Current prompt type (e.g., "arg", "div", "editor", "mainMenu")
    pub prompt_type: String,
    /// All components in the layout tree
    pub components: Vec<LayoutComponentInfo>,
    /// Timestamp when layout was captured (ISO 8601)
    pub timestamp: String,
}

// ============================================================
// MENU BAR INTEGRATION
// ============================================================

/// A menu bar item with its children and metadata
///
/// Used for serializing menu bar data between the app and SDK.
/// Represents a single menu item in the application's menu bar hierarchy.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MenuBarItemData {
    /// The display title of the menu item
    pub title: String,
    /// Whether the menu item is enabled (clickable)
    pub enabled: bool,
    /// Keyboard shortcut string if any (e.g., "⌘S")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// Child menu items (for submenus)
    #[serde(default)]
    pub children: Vec<MenuBarItemData>,
    /// Path of menu titles to reach this item (e.g., ["File", "New", "Window"])
    #[serde(default)]
    pub menu_path: Vec<String>,
}

impl MenuBarItemData {
    /// Create a new MenuBarItemData
    pub fn new(title: String, enabled: bool) -> Self {
        MenuBarItemData {
            title,
            enabled,
            shortcut: None,
            children: Vec::new(),
            menu_path: Vec::new(),
        }
    }

    /// Add a keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: String) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Add child menu items
    pub fn with_children(mut self, children: Vec<MenuBarItemData>) -> Self {
        self.children = children;
        self
    }

    /// Set the menu path
    pub fn with_menu_path(mut self, path: Vec<String>) -> Self {
        self.menu_path = path;
        self
    }
}

</file>

<file path="src/protocol/message.rs">
//! Protocol Message enum for Script Kit GPUI
//!
//! This module contains the main Message enum that represents all possible
//! protocol messages exchanged between scripts and the GPUI app.

use serde::{Deserialize, Serialize};

use super::types::*;

/// Protocol message with type discrimination via serde tag
///
/// This enum uses the "type" field to discriminate between message kinds.
/// Each variant corresponds to a message kind in the Script Kit v1 API.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    // ============================================================
    // CORE PROMPTS (existing)
    // ============================================================
    /// Script sends arg prompt with choices and optional actions
    #[serde(rename = "arg")]
    Arg {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Script sends div (HTML display)
    #[serde(rename = "div")]
    Div {
        id: String,
        html: String,
        /// Tailwind classes for the content container
        #[serde(rename = "containerClasses", skip_serializing_if = "Option::is_none")]
        container_classes: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
        /// Placeholder text (shown in header)
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Hint text
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
        /// Footer text
        #[serde(skip_serializing_if = "Option::is_none")]
        footer: Option<String>,
        /// Container background color: "transparent", "#RRGGBB", "#RRGGBBAA", or Tailwind color name
        #[serde(rename = "containerBg", skip_serializing_if = "Option::is_none")]
        container_bg: Option<String>,
        /// Container padding in pixels, or "none" to disable
        #[serde(rename = "containerPadding", skip_serializing_if = "Option::is_none")]
        container_padding: Option<serde_json::Value>,
        /// Container opacity (0-100)
        #[serde(skip_serializing_if = "Option::is_none")]
        opacity: Option<u8>,
    },

    /// App responds with submission (selected value or null)
    #[serde(rename = "submit")]
    Submit { id: String, value: Option<String> },

    /// App sends live update
    #[serde(rename = "update")]
    Update {
        id: String,
        #[serde(flatten)]
        data: serde_json::Value,
    },

    /// Signal termination
    #[serde(rename = "exit")]
    Exit {
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Force submit the current prompt with a value (from SDK's submit() function)
    #[serde(rename = "forceSubmit")]
    ForceSubmit { value: serde_json::Value },

    /// Set the current prompt's input text
    #[serde(rename = "setInput")]
    SetInput { text: String },

    // ============================================================
    // TEXT INPUT PROMPTS
    // ============================================================
    /// Code/text editor with syntax highlighting
    #[serde(rename = "editor")]
    Editor {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// VSCode-style snippet template with tabstops (e.g., "Hello ${1:name}!")
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<String>,
        #[serde(rename = "onInit", skip_serializing_if = "Option::is_none")]
        on_init: Option<String>,
        #[serde(rename = "onSubmit", skip_serializing_if = "Option::is_none")]
        on_submit: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Compact arg prompt (same as Arg but compact display)
    #[serde(rename = "mini")]
    Mini {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
    },

    /// Tiny arg prompt (same as Arg but tiny display)
    #[serde(rename = "micro")]
    Micro {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
    },

    // ============================================================
    // SELECTION PROMPTS
    // ============================================================
    /// Select from choices with optional multiple selection
    #[serde(rename = "select")]
    Select {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        #[serde(skip_serializing_if = "Option::is_none")]
        multiple: Option<bool>,
    },

    // ============================================================
    // FORM PROMPTS
    // ============================================================
    /// Multiple input fields
    #[serde(rename = "fields")]
    Fields {
        id: String,
        fields: Vec<Field>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Custom HTML form
    #[serde(rename = "form")]
    Form {
        id: String,
        html: String,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    // ============================================================
    // FILE/PATH PROMPTS
    // ============================================================
    /// File/folder path picker
    #[serde(rename = "path")]
    Path {
        id: String,
        #[serde(rename = "startPath", skip_serializing_if = "Option::is_none")]
        start_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
    },

    /// File drop zone
    #[serde(rename = "drop")]
    Drop { id: String },

    // ============================================================
    // INPUT CAPTURE PROMPTS
    // ============================================================
    /// Hotkey capture
    #[serde(rename = "hotkey")]
    Hotkey {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
    },

    // ============================================================
    // TEMPLATE/TEXT PROMPTS
    // ============================================================
    /// Template string with placeholders
    #[serde(rename = "template")]
    Template { id: String, template: String },

    /// Environment variable prompt
    #[serde(rename = "env")]
    Env {
        id: String,
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        secret: Option<bool>,
    },

    // ============================================================
    // MEDIA PROMPTS
    // ============================================================
    /// Chat interface
    #[serde(rename = "chat")]
    Chat { id: String },

    /// Terminal emulator
    #[serde(rename = "term")]
    Term {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Custom widget with HTML
    #[serde(rename = "widget")]
    Widget {
        id: String,
        html: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<serde_json::Value>,
    },

    /// Webcam capture
    #[serde(rename = "webcam")]
    Webcam { id: String },

    /// Microphone recording
    #[serde(rename = "mic")]
    Mic { id: String },

    // ============================================================
    // NOTIFICATION/FEEDBACK MESSAGES
    // ============================================================
    /// System notification
    #[serde(rename = "notify")]
    Notify {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
    },

    /// System beep sound
    #[serde(rename = "beep")]
    Beep {},

    /// Text-to-speech
    #[serde(rename = "say")]
    Say {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        voice: Option<String>,
    },

    /// Status bar update
    #[serde(rename = "setStatus")]
    SetStatus {
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// HUD (heads-up display) overlay message
    #[serde(rename = "hud")]
    Hud {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },

    // ============================================================
    // SYSTEM CONTROL MESSAGES
    // ============================================================
    /// Menu bar icon/scripts
    #[serde(rename = "menu")]
    Menu {
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        scripts: Option<Vec<String>>,
    },

    /// Clipboard operations
    #[serde(rename = "clipboard")]
    Clipboard {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        action: ClipboardAction,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<ClipboardFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },

    /// Keyboard simulation
    #[serde(rename = "keyboard")]
    Keyboard {
        action: KeyboardAction,
        #[serde(skip_serializing_if = "Option::is_none")]
        keys: Option<String>,
    },

    /// Mouse control
    #[serde(rename = "mouse")]
    Mouse {
        action: MouseAction,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },

    /// Show window
    #[serde(rename = "show")]
    Show {},

    /// Hide window
    #[serde(rename = "hide")]
    Hide {},

    /// Open URL in default browser
    #[serde(rename = "browse")]
    Browse { url: String },

    /// Execute shell command
    #[serde(rename = "exec")]
    Exec {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<serde_json::Value>,
    },

    // ============================================================
    // UI UPDATE MESSAGES
    // ============================================================
    /// Set panel HTML content
    #[serde(rename = "setPanel")]
    SetPanel { html: String },

    /// Set preview HTML content
    #[serde(rename = "setPreview")]
    SetPreview { html: String },

    /// Set prompt HTML content
    #[serde(rename = "setPrompt")]
    SetPrompt { html: String },

    // ============================================================
    // SELECTED TEXT OPERATIONS
    // ============================================================
    /// Get currently selected text from focused application
    #[serde(rename = "getSelectedText")]
    GetSelectedText {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Set (replace) currently selected text in focused application
    #[serde(rename = "setSelectedText")]
    SetSelectedText {
        text: String,
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Check if accessibility permissions are granted
    #[serde(rename = "checkAccessibility")]
    CheckAccessibility {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Request accessibility permissions (shows system dialog)
    #[serde(rename = "requestAccessibility")]
    RequestAccessibility {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    // ============================================================
    // WINDOW INFORMATION
    // ============================================================
    /// Get current window bounds (position and size)
    #[serde(rename = "getWindowBounds")]
    GetWindowBounds {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with window bounds
    #[serde(rename = "windowBounds")]
    WindowBounds {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        #[serde(rename = "requestId")]
        request_id: String,
    },

    // ============================================================
    // SELECTED TEXT RESPONSES
    // ============================================================
    /// Response with selected text
    #[serde(rename = "selectedText")]
    SelectedText {
        text: String,
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response after setting text
    #[serde(rename = "textSet")]
    TextSet {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with accessibility permission status
    #[serde(rename = "accessibilityStatus")]
    AccessibilityStatus {
        granted: bool,
        #[serde(rename = "requestId")]
        request_id: String,
    },

    // ============================================================
    // CLIPBOARD HISTORY
    // ============================================================
    /// Request clipboard history operation
    #[serde(rename = "clipboardHistory")]
    ClipboardHistory {
        #[serde(rename = "requestId")]
        request_id: String,
        action: ClipboardHistoryAction,
        /// Entry ID for pin/unpin/remove operations
        #[serde(rename = "entryId", skip_serializing_if = "Option::is_none")]
        entry_id: Option<String>,
    },

    /// Response with a clipboard history entry
    #[serde(rename = "clipboardHistoryEntry")]
    ClipboardHistoryEntry {
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "entryId")]
        entry_id: String,
        content: String,
        #[serde(rename = "contentType")]
        content_type: ClipboardEntryType,
        timestamp: String,
        pinned: bool,
    },

    /// Response with list of clipboard history entries
    #[serde(rename = "clipboardHistoryList")]
    ClipboardHistoryList {
        #[serde(rename = "requestId")]
        request_id: String,
        entries: Vec<ClipboardHistoryEntryData>,
    },

    /// Response for clipboard history action result
    #[serde(rename = "clipboardHistoryResult")]
    ClipboardHistoryResult {
        #[serde(rename = "requestId")]
        request_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ============================================================
    // WINDOW MANAGEMENT (System Windows)
    // ============================================================
    /// Request list of all system windows
    #[serde(rename = "windowList")]
    WindowList {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Perform action on a system window
    #[serde(rename = "windowAction")]
    WindowAction {
        #[serde(rename = "requestId")]
        request_id: String,
        action: WindowActionType,
        #[serde(rename = "windowId", skip_serializing_if = "Option::is_none")]
        window_id: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bounds: Option<TargetWindowBounds>,
    },

    /// Response with list of system windows
    #[serde(rename = "windowListResult")]
    WindowListResult {
        #[serde(rename = "requestId")]
        request_id: String,
        windows: Vec<SystemWindowInfo>,
    },

    /// Response for window action result
    #[serde(rename = "windowActionResult")]
    WindowActionResult {
        #[serde(rename = "requestId")]
        request_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ============================================================
    // FILE SEARCH
    // ============================================================
    /// Request file search
    #[serde(rename = "fileSearch")]
    FileSearch {
        #[serde(rename = "requestId")]
        request_id: String,
        query: String,
        #[serde(rename = "onlyin", skip_serializing_if = "Option::is_none")]
        only_in: Option<String>,
    },

    /// Response with file search results
    #[serde(rename = "fileSearchResult")]
    FileSearchResult {
        #[serde(rename = "requestId")]
        request_id: String,
        files: Vec<FileSearchResultEntry>,
    },

    // ============================================================
    // SCREENSHOT CAPTURE
    // ============================================================
    /// Request to capture app window screenshot
    #[serde(rename = "captureScreenshot")]
    CaptureScreenshot {
        #[serde(rename = "requestId")]
        request_id: String,
        /// If true, return full retina resolution (2x). If false (default), scale down to 1x.
        #[serde(rename = "hiDpi", skip_serializing_if = "Option::is_none")]
        hi_dpi: Option<bool>,
    },

    /// Response with screenshot data as base64 PNG
    #[serde(rename = "screenshotResult")]
    ScreenshotResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Base64-encoded PNG data
        data: String,
        width: u32,
        height: u32,
    },

    // ============================================================
    // STATE QUERY
    // ============================================================
    /// Request current UI state without modifying it
    #[serde(rename = "getState")]
    GetState {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with current UI state
    #[serde(rename = "stateResult")]
    StateResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Current prompt type
        #[serde(rename = "promptType")]
        prompt_type: String,
        /// Prompt ID if active
        #[serde(rename = "promptId", skip_serializing_if = "Option::is_none")]
        prompt_id: Option<String>,
        /// Placeholder text if applicable
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Current input/filter value
        #[serde(rename = "inputValue")]
        input_value: String,
        /// Total number of choices
        #[serde(rename = "choiceCount")]
        choice_count: usize,
        /// Number of visible/filtered choices
        #[serde(rename = "visibleChoiceCount")]
        visible_choice_count: usize,
        /// Currently selected index (-1 if none)
        #[serde(rename = "selectedIndex")]
        selected_index: i32,
        /// Value of the selected choice
        #[serde(rename = "selectedValue", skip_serializing_if = "Option::is_none")]
        selected_value: Option<String>,
        /// Whether the window has focus
        #[serde(rename = "isFocused")]
        is_focused: bool,
        /// Whether the window is visible
        #[serde(rename = "windowVisible")]
        window_visible: bool,
    },

    // ============================================================
    // ELEMENT QUERY (AI-driven UX)
    // ============================================================
    /// Request visible UI elements with semantic IDs
    #[serde(rename = "getElements")]
    GetElements {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Maximum number of elements to return (default: 50)
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<usize>,
    },

    /// Response with list of visible UI elements
    #[serde(rename = "elementsResult")]
    ElementsResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// List of visible UI elements
        elements: Vec<ElementInfo>,
        /// Total number of elements (may be larger than returned if limit applied)
        #[serde(rename = "totalCount")]
        total_count: usize,
    },

    // ============================================================
    // LAYOUT INFO (AI Agent Debugging)
    // ============================================================
    /// Request layout information with component tree and computed styles
    ///
    /// Returns detailed information about every component's position,
    /// size, padding, margin, gap, and flex properties. Designed to
    /// help AI agents understand "why" components are positioned/sized.
    #[serde(rename = "getLayoutInfo")]
    GetLayoutInfo {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with full layout information
    #[serde(rename = "layoutInfoResult")]
    LayoutInfoResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Full layout information including component tree
        #[serde(flatten)]
        info: LayoutInfo,
    },

    // ============================================================
    // ERROR REPORTING
    // ============================================================
    /// Script error with structured error information
    #[serde(rename = "setError")]
    SetError {
        /// User-friendly error message
        #[serde(rename = "errorMessage")]
        error_message: String,
        /// Raw stderr output if available
        #[serde(rename = "stderrOutput", skip_serializing_if = "Option::is_none")]
        stderr_output: Option<String>,
        /// Process exit code if available
        #[serde(rename = "exitCode", skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        /// Parsed stack trace if available
        #[serde(rename = "stackTrace", skip_serializing_if = "Option::is_none")]
        stack_trace: Option<String>,
        /// Path to the script that failed
        #[serde(rename = "scriptPath")]
        script_path: String,
        /// Actionable fix suggestions
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        suggestions: Vec<String>,
        /// When the error occurred (ISO 8601 format)
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    // ============================================================
    // SCRIPTLET OPERATIONS
    // ============================================================
    /// Run a scriptlet with variable substitution
    #[serde(rename = "runScriptlet")]
    RunScriptlet {
        #[serde(rename = "requestId")]
        request_id: String,
        /// The scriptlet data to execute
        scriptlet: ScriptletData,
        /// Named input values for {{variable}} substitution
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inputs: Option<serde_json::Value>,
        /// Positional arguments for $1, $2, etc.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
    },

    /// Request list of available scriptlets
    #[serde(rename = "getScriptlets")]
    GetScriptlets {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Optional kit to filter by
        #[serde(skip_serializing_if = "Option::is_none")]
        kit: Option<String>,
        /// Optional group to filter by
        #[serde(skip_serializing_if = "Option::is_none")]
        group: Option<String>,
    },

    /// Response with list of scriptlets
    #[serde(rename = "scriptletList")]
    ScriptletList {
        #[serde(rename = "requestId")]
        request_id: String,
        /// List of scriptlets
        scriptlets: Vec<ScriptletData>,
    },

    /// Result of scriptlet execution
    #[serde(rename = "scriptletResult")]
    ScriptletResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether execution succeeded
        success: bool,
        /// Output from the scriptlet (stdout)
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        /// Exit code if available
        #[serde(rename = "exitCode", skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
    },

    // ============================================================
    // TEST INFRASTRUCTURE
    // ============================================================
    /// Simulate a mouse click at specific coordinates (for testing)
    ///
    /// This message is used by test infrastructure to simulate mouse clicks
    /// at specified window-relative coordinates. It enables automated visual
    /// testing of click behaviors without requiring actual user interaction.
    #[serde(rename = "simulateClick")]
    SimulateClick {
        #[serde(rename = "requestId")]
        request_id: String,
        /// X coordinate relative to the window
        x: f64,
        /// Y coordinate relative to the window
        y: f64,
        /// Optional button: "left" (default), "right", or "middle"
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },

    /// Response after simulating a click
    #[serde(rename = "simulateClickResult")]
    SimulateClickResult {
        #[serde(rename = "requestId")]
        request_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ============================================================
    // DEBUG/VISUAL TESTING
    // ============================================================
    /// Show the debug grid overlay with options
    ///
    /// Displays a grid overlay for visual debugging and layout verification.
    /// The grid shows alignment lines, component bounds, and box model visualization.
    #[serde(rename = "showGrid")]
    ShowGrid {
        /// Grid configuration options (flattened into the message)
        #[serde(flatten)]
        options: GridOptions,
    },

    /// Hide the debug grid overlay
    #[serde(rename = "hideGrid")]
    HideGrid,

    // ============================================================
    // ACTIONS API
    // ============================================================
    /// Set actions to display in the ActionsDialog (incoming from SDK)
    ///
    /// Scripts define actions with optional onAction handlers. The `has_action`
    /// field on each action determines routing:
    /// - `has_action=true`: Send ActionTriggered back to SDK
    /// - `has_action=false`: Submit value directly
    #[serde(rename = "setActions")]
    SetActions {
        /// List of actions to display
        actions: Vec<ProtocolAction>,
    },

    /// Notify SDK that an action was triggered (outgoing to SDK)
    ///
    /// Sent when an action with `has_action=true` is triggered.
    /// The SDK's onAction handler will receive this.
    #[serde(rename = "actionTriggered")]
    ActionTriggered {
        /// Name of the triggered action
        action: String,
        /// Value associated with the action (if any)
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        /// Current input/filter text at time of trigger
        input: String,
    },

    // ============================================================
    // MENU BAR INTEGRATION
    // ============================================================
    /// Request menu bar items from the frontmost app or a specific app
    ///
    /// SDK sends this to get the menu bar hierarchy from an application.
    /// If bundle_id is None, uses the frontmost application.
    #[serde(rename = "getMenuBar")]
    GetMenuBar {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Optional bundle ID to get menu bar from a specific app
        #[serde(rename = "bundleId", skip_serializing_if = "Option::is_none")]
        bundle_id: Option<String>,
    },

    /// Response with menu bar items
    ///
    /// App sends this back to SDK with the menu bar hierarchy.
    #[serde(rename = "menuBarResult")]
    MenuBarResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// The menu bar items (hierarchical)
        items: Vec<super::types::MenuBarItemData>,
    },

    /// Execute a menu action by path
    ///
    /// SDK sends this to click a menu item in a specific application.
    #[serde(rename = "executeMenuAction")]
    ExecuteMenuAction {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Bundle ID of the target application
        #[serde(rename = "bundleId")]
        bundle_id: String,
        /// Path of menu titles to the target item (e.g., ["File", "New", "Window"])
        path: Vec<String>,
    },

    /// Result of a menu action execution
    ///
    /// App sends this back to SDK after attempting to execute a menu action.
    #[serde(rename = "menuActionResult")]
    MenuActionResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the action succeeded
        success: bool,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

impl Message {
    /// Create an arg prompt message
    pub fn arg(id: String, placeholder: String, choices: Vec<Choice>) -> Self {
        Message::Arg {
            id,
            placeholder,
            choices,
            actions: None,
        }
    }

    /// Create an arg prompt message with actions
    pub fn arg_with_actions(
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Vec<ProtocolAction>,
    ) -> Self {
        Message::Arg {
            id,
            placeholder,
            choices,
            actions: if actions.is_empty() {
                None
            } else {
                Some(actions)
            },
        }
    }

    /// Create a div (HTML display) message
    pub fn div(id: String, html: String) -> Self {
        Message::Div {
            id,
            html,
            container_classes: None,
            actions: None,
            placeholder: None,
            hint: None,
            footer: None,
            container_bg: None,
            container_padding: None,
            opacity: None,
        }
    }

    /// Create a div message with container classes
    pub fn div_with_classes(id: String, html: String, container_classes: String) -> Self {
        Message::Div {
            id,
            html,
            container_classes: Some(container_classes),
            actions: None,
            placeholder: None,
            hint: None,
            footer: None,
            container_bg: None,
            container_padding: None,
            opacity: None,
        }
    }

    /// Create a submit response message
    pub fn submit(id: String, value: Option<String>) -> Self {
        Message::Submit { id, value }
    }

    /// Create an exit message
    pub fn exit(code: Option<i32>, message: Option<String>) -> Self {
        Message::Exit { code, message }
    }

    /// Get the prompt ID for prompt-type messages (arg, div, editor, etc.)
    ///
    /// These messages have an `id` field that identifies the prompt session.
    /// Returns None for non-prompt messages.
    pub fn prompt_id(&self) -> Option<&str> {
        match self {
            // Core prompts
            Message::Arg { id, .. }
            | Message::Div { id, .. }
            | Message::Submit { id, .. }
            | Message::Update { id, .. }
            // Text input prompts
            | Message::Editor { id, .. }
            | Message::Mini { id, .. }
            | Message::Micro { id, .. }
            // Selection prompts
            | Message::Select { id, .. }
            // Form prompts
            | Message::Fields { id, .. }
            | Message::Form { id, .. }
            // File/path prompts
            | Message::Path { id, .. }
            | Message::Drop { id, .. }
            // Input capture prompts
            | Message::Hotkey { id, .. }
            // Template/text prompts
            | Message::Template { id, .. }
            | Message::Env { id, .. }
            // Media prompts
            | Message::Chat { id, .. }
            | Message::Term { id, .. }
            | Message::Widget { id, .. }
            | Message::Webcam { id, .. }
            | Message::Mic { id, .. } => Some(id),
            // Clipboard has optional id
            Message::Clipboard { id, .. } => id.as_deref(),
            // All other messages don't have prompt IDs
            _ => None,
        }
    }

    /// Get the request ID for request/response type messages
    ///
    /// These messages have a `request_id` field for correlating requests with responses.
    /// Returns None for non-request messages.
    pub fn request_id(&self) -> Option<&str> {
        match self {
            // Selected text operations
            Message::GetSelectedText { request_id, .. }
            | Message::SetSelectedText { request_id, .. }
            | Message::CheckAccessibility { request_id, .. }
            | Message::RequestAccessibility { request_id, .. }
            | Message::SelectedText { request_id, .. }
            | Message::TextSet { request_id, .. }
            | Message::AccessibilityStatus { request_id, .. }
            // Window information
            | Message::GetWindowBounds { request_id, .. }
            | Message::WindowBounds { request_id, .. }
            // Clipboard history
            | Message::ClipboardHistory { request_id, .. }
            | Message::ClipboardHistoryEntry { request_id, .. }
            | Message::ClipboardHistoryList { request_id, .. }
            | Message::ClipboardHistoryResult { request_id, .. }
            // Window management
            | Message::WindowList { request_id, .. }
            | Message::WindowAction { request_id, .. }
            | Message::WindowListResult { request_id, .. }
            | Message::WindowActionResult { request_id, .. }
            // File search
            | Message::FileSearch { request_id, .. }
            | Message::FileSearchResult { request_id, .. }
            // Screenshot capture
            | Message::CaptureScreenshot { request_id, .. }
            | Message::ScreenshotResult { request_id, .. }
            // State query
            | Message::GetState { request_id, .. }
            | Message::StateResult { request_id, .. }
            // Element query
            | Message::GetElements { request_id, .. }
            | Message::ElementsResult { request_id, .. }
            // Layout info
            | Message::GetLayoutInfo { request_id, .. }
            | Message::LayoutInfoResult { request_id, .. }
            // Scriptlet operations
            | Message::RunScriptlet { request_id, .. }
            | Message::GetScriptlets { request_id, .. }
            | Message::ScriptletList { request_id, .. }
            | Message::ScriptletResult { request_id, .. }
            // Test infrastructure
            | Message::SimulateClick { request_id, .. }
            | Message::SimulateClickResult { request_id, .. }
            // Menu bar
            | Message::GetMenuBar { request_id, .. }
            | Message::MenuBarResult { request_id, .. }
            | Message::ExecuteMenuAction { request_id, .. }
            | Message::MenuActionResult { request_id, .. } => Some(request_id),
            // All other messages don't have request IDs
            _ => None,
        }
    }

    /// Get the message ID (works for message types that have an ID)
    ///
    /// This is a unified accessor that returns either prompt_id or request_id,
    /// whichever is applicable for the message type.
    pub fn id(&self) -> Option<&str> {
        // Try prompt_id first, then request_id
        self.prompt_id().or_else(|| self.request_id())
    }

    // ============================================================
    // Constructor methods for new message types
    // ============================================================

    /// Create an editor prompt message
    pub fn editor(id: String) -> Self {
        Message::Editor {
            id,
            content: None,
            language: None,
            template: None,
            on_init: None,
            on_submit: None,
            actions: None,
        }
    }

    /// Create an editor with content and language
    pub fn editor_with_content(id: String, content: String, language: Option<String>) -> Self {
        Message::Editor {
            id,
            content: Some(content),
            language,
            template: None,
            on_init: None,
            on_submit: None,
            actions: None,
        }
    }

    /// Create an editor with a VSCode-style snippet template
    pub fn editor_with_template(id: String, template: String, language: Option<String>) -> Self {
        Message::Editor {
            id,
            content: None,
            language,
            template: Some(template),
            on_init: None,
            on_submit: None,
            actions: None,
        }
    }

    /// Create a mini prompt message
    pub fn mini(id: String, placeholder: String, choices: Vec<Choice>) -> Self {
        Message::Mini {
            id,
            placeholder,
            choices,
        }
    }

    /// Create a micro prompt message
    pub fn micro(id: String, placeholder: String, choices: Vec<Choice>) -> Self {
        Message::Micro {
            id,
            placeholder,
            choices,
        }
    }

    /// Create a select prompt message
    pub fn select(id: String, placeholder: String, choices: Vec<Choice>, multiple: bool) -> Self {
        Message::Select {
            id,
            placeholder,
            choices,
            multiple: if multiple { Some(true) } else { None },
        }
    }

    /// Create a fields prompt message
    pub fn fields(id: String, fields: Vec<Field>) -> Self {
        Message::Fields {
            id,
            fields,
            actions: None,
        }
    }

    /// Create a form prompt message
    pub fn form(id: String, html: String) -> Self {
        Message::Form {
            id,
            html,
            actions: None,
        }
    }

    /// Create a path prompt message
    pub fn path(id: String, start_path: Option<String>) -> Self {
        Message::Path {
            id,
            start_path,
            hint: None,
        }
    }

    /// Create a drop zone message
    pub fn drop(id: String) -> Self {
        Message::Drop { id }
    }

    /// Create a hotkey prompt message
    pub fn hotkey(id: String) -> Self {
        Message::Hotkey {
            id,
            placeholder: None,
        }
    }

    /// Create a template prompt message
    pub fn template(id: String, template: String) -> Self {
        Message::Template { id, template }
    }

    /// Create an env prompt message
    pub fn env(id: String, key: String, secret: bool) -> Self {
        Message::Env {
            id,
            key,
            secret: if secret { Some(true) } else { None },
        }
    }

    /// Create a chat prompt message
    pub fn chat(id: String) -> Self {
        Message::Chat { id }
    }

    /// Create a term prompt message
    pub fn term(id: String, command: Option<String>) -> Self {
        Message::Term {
            id,
            command,
            actions: None,
        }
    }

    /// Create a widget message
    pub fn widget(id: String, html: String) -> Self {
        Message::Widget {
            id,
            html,
            options: None,
        }
    }

    /// Create a webcam prompt message
    pub fn webcam(id: String) -> Self {
        Message::Webcam { id }
    }

    /// Create a mic prompt message
    pub fn mic(id: String) -> Self {
        Message::Mic { id }
    }

    /// Create a notify message
    pub fn notify(title: Option<String>, body: Option<String>) -> Self {
        Message::Notify { title, body }
    }

    /// Create a beep message
    pub fn beep() -> Self {
        Message::Beep {}
    }

    /// Create a say message
    pub fn say(text: String, voice: Option<String>) -> Self {
        Message::Say { text, voice }
    }

    /// Create a set status message
    pub fn set_status(status: String, message: Option<String>) -> Self {
        Message::SetStatus { status, message }
    }

    /// Create a HUD overlay message
    pub fn hud(text: String, duration_ms: Option<u64>) -> Self {
        Message::Hud { text, duration_ms }
    }

    /// Create a menu message
    pub fn menu(icon: Option<String>, scripts: Option<Vec<String>>) -> Self {
        Message::Menu { icon, scripts }
    }

    /// Create a clipboard read message
    pub fn clipboard_read(format: Option<ClipboardFormat>) -> Self {
        Message::Clipboard {
            id: None,
            action: ClipboardAction::Read,
            format,
            content: None,
        }
    }

    /// Create a clipboard write message
    pub fn clipboard_write(content: String, format: Option<ClipboardFormat>) -> Self {
        Message::Clipboard {
            id: None,
            action: ClipboardAction::Write,
            format,
            content: Some(content),
        }
    }

    /// Create a keyboard type message
    pub fn keyboard_type(keys: String) -> Self {
        Message::Keyboard {
            action: KeyboardAction::Type,
            keys: Some(keys),
        }
    }

    /// Create a keyboard tap message
    pub fn keyboard_tap(keys: String) -> Self {
        Message::Keyboard {
            action: KeyboardAction::Tap,
            keys: Some(keys),
        }
    }

    /// Create a mouse message
    pub fn mouse(action: MouseAction, data: Option<serde_json::Value>) -> Self {
        Message::Mouse { action, data }
    }

    /// Create a show message
    pub fn show() -> Self {
        Message::Show {}
    }

    /// Create a hide message
    pub fn hide() -> Self {
        Message::Hide {}
    }

    /// Create a browse message to open URL in default browser
    pub fn browse(url: String) -> Self {
        Message::Browse { url }
    }

    /// Create an exec message
    pub fn exec(command: String, options: Option<serde_json::Value>) -> Self {
        Message::Exec { command, options }
    }

    /// Create a set panel message
    pub fn set_panel(html: String) -> Self {
        Message::SetPanel { html }
    }

    /// Create a set preview message
    pub fn set_preview(html: String) -> Self {
        Message::SetPreview { html }
    }

    /// Create a set prompt message
    pub fn set_prompt(html: String) -> Self {
        Message::SetPrompt { html }
    }

    // ============================================================
    // Constructor methods for selected text operations
    // ============================================================

    /// Create a get selected text request
    pub fn get_selected_text(request_id: String) -> Self {
        Message::GetSelectedText { request_id }
    }

    /// Create a set selected text request
    pub fn set_selected_text_msg(text: String, request_id: String) -> Self {
        Message::SetSelectedText { text, request_id }
    }

    /// Create a check accessibility request
    pub fn check_accessibility(request_id: String) -> Self {
        Message::CheckAccessibility { request_id }
    }

    /// Create a request accessibility request
    pub fn request_accessibility(request_id: String) -> Self {
        Message::RequestAccessibility { request_id }
    }

    /// Create a selected text response
    pub fn selected_text_response(text: String, request_id: String) -> Self {
        Message::SelectedText { text, request_id }
    }

    /// Create a text set response (success)
    pub fn text_set_success(request_id: String) -> Self {
        Message::TextSet {
            success: true,
            error: None,
            request_id,
        }
    }

    /// Create a text set response (error)
    pub fn text_set_error(error: String, request_id: String) -> Self {
        Message::TextSet {
            success: false,
            error: Some(error),
            request_id,
        }
    }

    /// Create an accessibility status response
    pub fn accessibility_status(granted: bool, request_id: String) -> Self {
        Message::AccessibilityStatus {
            granted,
            request_id,
        }
    }

    // ============================================================
    // Constructor methods for window information
    // ============================================================

    /// Create a get window bounds request
    pub fn get_window_bounds(request_id: String) -> Self {
        Message::GetWindowBounds { request_id }
    }

    /// Create a window bounds response
    pub fn window_bounds(x: f64, y: f64, width: f64, height: f64, request_id: String) -> Self {
        Message::WindowBounds {
            x,
            y,
            width,
            height,
            request_id,
        }
    }

    // ============================================================
    // Constructor methods for clipboard history
    // ============================================================

    /// Create a clipboard history list request
    pub fn clipboard_history_list(request_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::List,
            entry_id: None,
        }
    }

    /// Create a clipboard history pin request
    pub fn clipboard_history_pin(request_id: String, entry_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Pin,
            entry_id: Some(entry_id),
        }
    }

    /// Create a clipboard history unpin request
    pub fn clipboard_history_unpin(request_id: String, entry_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Unpin,
            entry_id: Some(entry_id),
        }
    }

    /// Create a clipboard history remove request
    pub fn clipboard_history_remove(request_id: String, entry_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Remove,
            entry_id: Some(entry_id),
        }
    }

    /// Create a clipboard history clear request
    pub fn clipboard_history_clear(request_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Clear,
            entry_id: None,
        }
    }

    /// Create a clipboard history trim oversize request
    pub fn clipboard_history_trim_oversize(request_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::TrimOversize,
            entry_id: None,
        }
    }

    /// Create a clipboard history entry response
    pub fn clipboard_history_entry(
        request_id: String,
        entry_id: String,
        content: String,
        content_type: ClipboardEntryType,
        timestamp: String,
        pinned: bool,
    ) -> Self {
        Message::ClipboardHistoryEntry {
            request_id,
            entry_id,
            content,
            content_type,
            timestamp,
            pinned,
        }
    }

    /// Create a clipboard history list response
    pub fn clipboard_history_list_response(
        request_id: String,
        entries: Vec<ClipboardHistoryEntryData>,
    ) -> Self {
        Message::ClipboardHistoryList {
            request_id,
            entries,
        }
    }

    /// Create a clipboard history result (success)
    pub fn clipboard_history_success(request_id: String) -> Self {
        Message::ClipboardHistoryResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a clipboard history result (error)
    pub fn clipboard_history_error(request_id: String, error: String) -> Self {
        Message::ClipboardHistoryResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }

    // ============================================================
    // Constructor methods for window management
    // ============================================================

    /// Create a window list request
    pub fn window_list(request_id: String) -> Self {
        Message::WindowList { request_id }
    }

    /// Create a window action request
    pub fn window_action(
        request_id: String,
        action: WindowActionType,
        window_id: Option<u32>,
        bounds: Option<TargetWindowBounds>,
    ) -> Self {
        Message::WindowAction {
            request_id,
            action,
            window_id,
            bounds,
        }
    }

    /// Create a window list response
    pub fn window_list_result(request_id: String, windows: Vec<SystemWindowInfo>) -> Self {
        Message::WindowListResult {
            request_id,
            windows,
        }
    }

    /// Create a window action result (success)
    pub fn window_action_success(request_id: String) -> Self {
        Message::WindowActionResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a window action result (error)
    pub fn window_action_error(request_id: String, error: String) -> Self {
        Message::WindowActionResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }

    // ============================================================
    // Constructor methods for file search
    // ============================================================

    /// Create a file search request
    pub fn file_search(request_id: String, query: String, only_in: Option<String>) -> Self {
        Message::FileSearch {
            request_id,
            query,
            only_in,
        }
    }

    /// Create a file search result response
    pub fn file_search_result(request_id: String, files: Vec<FileSearchResultEntry>) -> Self {
        Message::FileSearchResult { request_id, files }
    }

    // ============================================================
    // Constructor methods for screenshot capture
    // ============================================================

    /// Create a capture screenshot request
    pub fn capture_screenshot(request_id: String) -> Self {
        Message::CaptureScreenshot {
            request_id,
            hi_dpi: None,
        }
    }

    /// Create a capture screenshot request with hi_dpi option
    pub fn capture_screenshot_with_options(request_id: String, hi_dpi: Option<bool>) -> Self {
        Message::CaptureScreenshot { request_id, hi_dpi }
    }

    /// Create a screenshot result response
    pub fn screenshot_result(request_id: String, data: String, width: u32, height: u32) -> Self {
        Message::ScreenshotResult {
            request_id,
            data,
            width,
            height,
        }
    }

    // ============================================================
    // Constructor methods for state query
    // ============================================================

    /// Create a get state request
    pub fn get_state(request_id: String) -> Self {
        Message::GetState { request_id }
    }

    /// Create a state result response
    #[allow(clippy::too_many_arguments)]
    pub fn state_result(
        request_id: String,
        prompt_type: String,
        prompt_id: Option<String>,
        placeholder: Option<String>,
        input_value: String,
        choice_count: usize,
        visible_choice_count: usize,
        selected_index: i32,
        selected_value: Option<String>,
        is_focused: bool,
        window_visible: bool,
    ) -> Self {
        Message::StateResult {
            request_id,
            prompt_type,
            prompt_id,
            placeholder,
            input_value,
            choice_count,
            visible_choice_count,
            selected_index,
            selected_value,
            is_focused,
            window_visible,
        }
    }

    // ============================================================
    // Constructor methods for element query
    // ============================================================

    /// Create a get elements request
    pub fn get_elements(request_id: String) -> Self {
        Message::GetElements {
            request_id,
            limit: None,
        }
    }

    /// Create a get elements request with limit
    pub fn get_elements_with_limit(request_id: String, limit: usize) -> Self {
        Message::GetElements {
            request_id,
            limit: Some(limit),
        }
    }

    /// Create an elements result response
    pub fn elements_result(
        request_id: String,
        elements: Vec<ElementInfo>,
        total_count: usize,
    ) -> Self {
        Message::ElementsResult {
            request_id,
            elements,
            total_count,
        }
    }

    // ============================================================
    // Constructor methods for layout info
    // ============================================================

    /// Create a get layout info request
    pub fn get_layout_info(request_id: String) -> Self {
        Message::GetLayoutInfo { request_id }
    }

    /// Create a layout info result response
    pub fn layout_info_result(request_id: String, info: LayoutInfo) -> Self {
        Message::LayoutInfoResult { request_id, info }
    }

    // ============================================================
    // Constructor methods for error reporting
    // ============================================================

    /// Create a script error message from ScriptErrorData
    pub fn set_error(error_data: ScriptErrorData) -> Self {
        Message::SetError {
            error_message: error_data.error_message,
            stderr_output: error_data.stderr_output,
            exit_code: error_data.exit_code,
            stack_trace: error_data.stack_trace,
            script_path: error_data.script_path,
            suggestions: error_data.suggestions,
            timestamp: error_data.timestamp,
        }
    }

    /// Create a simple script error message with just the message and path
    pub fn script_error(error_message: String, script_path: String) -> Self {
        Message::SetError {
            error_message,
            stderr_output: None,
            exit_code: None,
            stack_trace: None,
            script_path,
            suggestions: Vec::new(),
            timestamp: None,
        }
    }

    /// Create a full script error message with all optional fields
    pub fn script_error_full(
        error_message: String,
        script_path: String,
        stderr_output: Option<String>,
        exit_code: Option<i32>,
        stack_trace: Option<String>,
        suggestions: Vec<String>,
        timestamp: Option<String>,
    ) -> Self {
        Message::SetError {
            error_message,
            stderr_output,
            exit_code,
            stack_trace,
            script_path,
            suggestions,
            timestamp,
        }
    }

    // ============================================================
    // Constructor methods for scriptlet operations
    // ============================================================

    /// Create a run scriptlet request
    pub fn run_scriptlet(
        request_id: String,
        scriptlet: ScriptletData,
        inputs: Option<serde_json::Value>,
        args: Vec<String>,
    ) -> Self {
        Message::RunScriptlet {
            request_id,
            scriptlet,
            inputs,
            args,
        }
    }

    /// Create a get scriptlets request
    pub fn get_scriptlets(request_id: String) -> Self {
        Message::GetScriptlets {
            request_id,
            kit: None,
            group: None,
        }
    }

    /// Create a get scriptlets request with filters
    pub fn get_scriptlets_filtered(
        request_id: String,
        kit: Option<String>,
        group: Option<String>,
    ) -> Self {
        Message::GetScriptlets {
            request_id,
            kit,
            group,
        }
    }

    /// Create a scriptlet list response
    pub fn scriptlet_list(request_id: String, scriptlets: Vec<ScriptletData>) -> Self {
        Message::ScriptletList {
            request_id,
            scriptlets,
        }
    }

    /// Create a successful scriptlet result
    pub fn scriptlet_result_success(
        request_id: String,
        output: Option<String>,
        exit_code: Option<i32>,
    ) -> Self {
        Message::ScriptletResult {
            request_id,
            success: true,
            output,
            error: None,
            exit_code,
        }
    }

    /// Create a failed scriptlet result
    pub fn scriptlet_result_error(
        request_id: String,
        error: String,
        exit_code: Option<i32>,
    ) -> Self {
        Message::ScriptletResult {
            request_id,
            success: false,
            output: None,
            error: Some(error),
            exit_code,
        }
    }

    // ============================================================
    // Constructor methods for test infrastructure
    // ============================================================

    /// Create a simulate click request
    ///
    /// Coordinates are relative to the window's content area.
    pub fn simulate_click(request_id: String, x: f64, y: f64) -> Self {
        Message::SimulateClick {
            request_id,
            x,
            y,
            button: None,
        }
    }

    /// Create a simulate click request with a specific button
    ///
    /// Coordinates are relative to the window's content area.
    /// Button can be "left", "right", or "middle".
    pub fn simulate_click_with_button(request_id: String, x: f64, y: f64, button: String) -> Self {
        Message::SimulateClick {
            request_id,
            x,
            y,
            button: Some(button),
        }
    }

    /// Create a successful simulate click result
    pub fn simulate_click_success(request_id: String) -> Self {
        Message::SimulateClickResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a failed simulate click result
    pub fn simulate_click_error(request_id: String, error: String) -> Self {
        Message::SimulateClickResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }

    // ============================================================
    // Constructor methods for Actions API
    // ============================================================

    /// Create an ActionTriggered message to send to SDK
    ///
    /// This is sent when an action with `has_action=true` is triggered.
    pub fn action_triggered(action: String, value: Option<String>, input: String) -> Self {
        Message::ActionTriggered {
            action,
            value,
            input,
        }
    }

    /// Create a SetActions message
    pub fn set_actions(actions: Vec<ProtocolAction>) -> Self {
        Message::SetActions { actions }
    }

    /// Create a SetInput message
    pub fn set_input(text: String) -> Self {
        Message::SetInput { text }
    }

    // ============================================================
    // Constructor methods for debug grid
    // ============================================================

    /// Create a ShowGrid message with default options
    pub fn show_grid() -> Self {
        Message::ShowGrid {
            options: GridOptions::default(),
        }
    }

    /// Create a ShowGrid message with custom options
    pub fn show_grid_with_options(options: GridOptions) -> Self {
        Message::ShowGrid { options }
    }

    /// Create a HideGrid message
    pub fn hide_grid() -> Self {
        Message::HideGrid
    }

    // ============================================================
    // Constructor methods for menu bar integration
    // ============================================================

    /// Create a GetMenuBar request message
    pub fn get_menu_bar(request_id: String, bundle_id: Option<String>) -> Self {
        Message::GetMenuBar {
            request_id,
            bundle_id,
        }
    }

    /// Create a MenuBarResult response message
    pub fn menu_bar_result(request_id: String, items: Vec<super::types::MenuBarItemData>) -> Self {
        Message::MenuBarResult { request_id, items }
    }

    /// Create an ExecuteMenuAction request message
    pub fn execute_menu_action(request_id: String, bundle_id: String, path: Vec<String>) -> Self {
        Message::ExecuteMenuAction {
            request_id,
            bundle_id,
            path,
        }
    }

    /// Create a successful MenuActionResult response message
    pub fn menu_action_success(request_id: String) -> Self {
        Message::MenuActionResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a failed MenuActionResult response message
    pub fn menu_action_error(request_id: String, error: String) -> Self {
        Message::MenuActionResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 5 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~27.3K (27,253 exact)
  Total Chars: 128,455 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     12.7K - src/protocol/message.rs
      7.8K - src/protocol/types.rs
      5.0K - src/protocol/io.rs
      1.1K - src/protocol/semantic_id.rs
       530 - src/protocol/mod.rs
