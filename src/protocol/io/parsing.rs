// Protocol I/O for JSONL message parsing and serialization
//
// This module provides:
// - `parse_message` / `parse_message_graceful` for parsing JSON messages
// - `serialize_message` for serializing messages to JSON
// - `JsonlReader` for streaming JSONL reads

use std::io::{BufRead, BufReader, ErrorKind, Read};
use tracing::{debug, warn};
use uuid::Uuid;

use super::message::Message;

/// Maximum length for raw JSON in logs (prevents huge base64 data in logs)
const MAX_RAW_LOG_PREVIEW: usize = 200;
/// Maximum JSONL message size accepted from script stdout.
///
/// Large lines are treated as malformed input to prevent memory amplification.
const MAX_PROTOCOL_LINE_BYTES: usize = 64 * 1024;

enum LineRead {
    Eof,
    Line(String),
    TooLong { raw: String, raw_len: usize },
}

/// Get a truncated preview of raw JSON for logging
///
/// # Safety
/// This function handles UTF-8 correctly by finding a valid char boundary
/// when truncating. It will never panic on multi-byte UTF-8 characters.
fn log_preview(raw: &str) -> (&str, usize) {
    let len = raw.len();
    if len > MAX_RAW_LOG_PREVIEW {
        // Find a valid UTF-8 char boundary at or before MAX_RAW_LOG_PREVIEW
        // This prevents panics on multi-byte characters (emoji, CJK, etc.)
        let mut end = MAX_RAW_LOG_PREVIEW;
        while end > 0 && !raw.is_char_boundary(end) {
            end -= 1;
        }
        (&raw[..end], len)
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
///
/// # Security
/// Raw JSON is truncated to 200 chars in error logs to prevent leaking sensitive data
/// (base64 screenshots, clipboard content, etc.)
#[tracing::instrument(skip_all, fields(line_len = line.len()))]
fn parse_message(line: &str) -> Result<Message, serde_json::Error> {
    serde_json::from_str(line).map_err(|e| {
        // SECURITY: Use truncated preview to avoid logging sensitive data
        // (base64 screenshots, clipboard content, user text, etc.)
        let (preview, raw_len) = log_preview(line);
        warn!(
            raw_preview = %preview,
            raw_len = raw_len,
            error = %e,
            "Failed to parse JSONL message"
        );
        e
    })
}

fn is_unknown_message_type_error(error: &str, message_type: &str) -> bool {
    if !error.contains("unknown variant") {
        return false;
    }

    let quoted_markers = [
        format!("`{message_type}`"),
        format!("'{message_type}'"),
        format!("\"{message_type}\""),
    ];

    quoted_markers.iter().any(|marker| error.contains(marker))
        || error.contains(&format!("unknown variant {message_type}"))
}

/// Result type for graceful message parsing
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum ParseResult {
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
    LineTooLong,
}

#[derive(Debug, Clone)]
pub(crate) struct ParseIssue {
    pub(crate) correlation_id: String,
    pub(crate) kind: ParseIssueKind,
    pub(crate) message_type: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) raw_preview: String,
    pub(crate) raw_len: usize,
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
#[tracing::instrument(skip_all, fields(line_len = line.len()))]
fn parse_message_graceful(line: &str) -> ParseResult {
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
            // Classify as UnknownType only when the unknown variant token matches
            // the top-level message type field; other unknown variants are payload errors.
            if is_unknown_message_type_error(&error_str, &msg_type) {
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
