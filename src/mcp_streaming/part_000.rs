use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
/// SSE event types supported by the MCP server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SseEventType {
    Progress,
    Output,
    Error,
    Complete,
}
impl SseEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SseEventType::Progress => "progress",
            SseEventType::Output => "output",
            SseEventType::Error => "error",
            SseEventType::Complete => "complete",
        }
    }
}
impl std::fmt::Display for SseEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
/// SSE Stream manager for broadcasting events to connected clients
#[derive(Debug)]
pub struct SseStream {
    /// Buffer of formatted SSE messages ready to send
    buffer: Vec<String>,
}
impl SseStream {
    /// Create a new SSE stream
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Format and queue an SSE event for broadcast
    ///
    /// Event format: `event: {type}\ndata: {json}\n\n`
    pub fn broadcast_event(&mut self, event_type: SseEventType, data: &serde_json::Value) {
        let formatted = format_sse_event(event_type, data);
        self.buffer.push(formatted);
    }

    /// Get all pending events and clear the buffer
    pub fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.buffer)
    }

    /// Get the number of pending events
    pub fn pending_count(&self) -> usize {
        self.buffer.len()
    }
}
impl Default for SseStream {
    fn default() -> Self {
        Self::new()
    }
}
/// Format a single SSE event
///
/// Format: `event: {type}\ndata: {json}\n\n`
pub fn format_sse_event(event_type: SseEventType, data: &serde_json::Value) -> String {
    let json_str = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    format!("event: {}\ndata: {}\n\n", event_type.as_str(), json_str)
}
/// Format an SSE heartbeat comment
///
/// Format: `: heartbeat\n\n`
pub fn format_sse_heartbeat() -> String {
    ": heartbeat\n\n".to_string()
}
/// Audit log entry for tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Method/tool name that was called
    pub method: String,
    /// Parameters passed to the method (as JSON)
    pub params: serde_json::Value,
    /// Duration of the call in milliseconds
    pub duration_ms: u64,
    /// Whether the call succeeded
    pub success: bool,
    /// Error message if the call failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl AuditLogEntry {
    /// Create a new successful audit log entry
    pub fn success(method: &str, params: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            timestamp: iso8601_now(),
            method: method.to_string(),
            params,
            duration_ms,
            success: true,
            error: None,
        }
    }

    /// Create a new failed audit log entry
    pub fn failure(method: &str, params: serde_json::Value, duration_ms: u64, error: &str) -> Self {
        Self {
            timestamp: iso8601_now(),
            method: method.to_string(),
            params,
            duration_ms,
            success: false,
            error: Some(error.to_string()),
        }
    }
}
/// Audit logger that writes to ~/.scriptkit/logs/mcp-audit.jsonl
pub struct AuditLogger {
    log_path: PathBuf,
}
impl AuditLogger {
    /// Create a new audit logger
    ///
    /// # Arguments
    /// * `kit_path` - Path to ~/.scriptkit directory
    pub fn new(kit_path: PathBuf) -> Self {
        let log_path = kit_path.join("logs").join("mcp-audit.jsonl");
        Self { log_path }
    }

    /// Create audit logger with default ~/.scriptkit path
    pub fn with_defaults() -> Result<Self> {
        let kit_path = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".scriptkit");
        Ok(Self::new(kit_path))
    }

    /// Get the log file path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// Write an audit log entry
    pub fn log(&self, entry: &AuditLogEntry) -> Result<()> {
        // Ensure logs directory exists
        if let Some(parent) = self.log_path.parent() {
            fs::create_dir_all(parent).context("Failed to create logs directory")?;
        }

        // Serialize entry to JSON
        let json = serde_json::to_string(entry).context("Failed to serialize audit log entry")?;

        // Append to log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .context("Failed to open audit log file")?;

        writeln!(file, "{}", json).context("Failed to write audit log entry")?;

        Ok(())
    }

    /// Log a successful tool call
    pub fn log_success(
        &self,
        method: &str,
        params: serde_json::Value,
        duration_ms: u64,
    ) -> Result<()> {
        let entry = AuditLogEntry::success(method, params, duration_ms);
        self.log(&entry)
    }

    /// Log a failed tool call
    pub fn log_failure(
        &self,
        method: &str,
        params: serde_json::Value,
        duration_ms: u64,
        error: &str,
    ) -> Result<()> {
        let entry = AuditLogEntry::failure(method, params, duration_ms, error);
        self.log(&entry)
    }
}
/// Get current timestamp in ISO 8601 format
fn iso8601_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    // Convert to datetime components (simplified - just for formatting)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year/month/day from days since epoch (1970-01-01)
    // Simplified calculation - good enough for logging purposes
    let mut year = 1970i32;
    let mut remaining_days = days_since_epoch as i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let (month, day) = day_of_year_to_month_day(remaining_days as u32 + 1, is_leap_year(year));

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
fn day_of_year_to_month_day(day_of_year: u32, leap: bool) -> (u32, u32) {
    let days_in_months: [u32; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining = day_of_year;
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining <= days {
            return ((i + 1) as u32, remaining);
        }
        remaining -= days;
    }
    (12, 31) // Fallback
}
