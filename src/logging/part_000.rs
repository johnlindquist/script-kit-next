use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde_json::{Map, Value};
use tracing::field::{Field, Visit};
use tracing::{Level, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, MakeWriter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;
// =============================================================================
// SESSION IDENTITY & LOG PATHS
// =============================================================================
/// Path to the session-specific log file (latest-session.jsonl).
static SESSION_LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
/// Unique ID for the current session, generated on init.
static SESSION_ID: OnceLock<String> = OnceLock::new();
/// Get the path to the session-specific log file.
pub fn session_log_path() -> PathBuf {
    SESSION_LOG_PATH
        .get()
        .cloned()
        .unwrap_or_else(|| get_log_dir().join("latest-session.jsonl"))
}
/// Get the current session ID.
pub fn session_id() -> &'static str {
    SESSION_ID.get().map(|s| s.as_str()).unwrap_or("unknown")
}
// =============================================================================
// CORRELATION ID (MANDATORY FIELD)
// =============================================================================
// Global default correlation_id for events where no per-run/context value is set.
static DEFAULT_CORRELATION_ID: OnceLock<String> = OnceLock::new();
// Thread-local override used for request/interaction scoped correlation_ids.
thread_local! {
    static CORRELATION_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}
/// Set the correlation_id for the current thread, returning a guard that restores the previous
/// value on drop.
pub fn set_correlation_id(id: impl Into<String>) -> CorrelationGuard {
    let new_id = id.into();
    let previous = CORRELATION_ID.with(|cell| cell.borrow_mut().replace(new_id));
    CorrelationGuard { previous }
}
/// Guard that restores the previous correlation_id when dropped.
pub struct CorrelationGuard {
    previous: Option<String>,
}
impl Drop for CorrelationGuard {
    fn drop(&mut self) {
        let prev = self.previous.take();
        CORRELATION_ID.with(|cell| {
            *cell.borrow_mut() = prev;
        });
    }
}
/// Get the current correlation_id (thread-local if set, otherwise the global default).
pub fn current_correlation_id() -> String {
    CORRELATION_ID.with(|cell| {
        cell.borrow().clone().unwrap_or_else(|| {
            DEFAULT_CORRELATION_ID
                .get_or_init(|| Uuid::new_v4().to_string())
                .clone()
        })
    })
}
// =============================================================================
// BENCHMARKING UTILITIES (for hotkey → prompt latency analysis)
// =============================================================================
// Stores the instant when benchmarking started (as nanos since process start)
static BENCH_START_NANOS: AtomicU64 = AtomicU64::new(0);
static BENCH_EPOCH: OnceLock<Instant> = OnceLock::new();
/// Start a benchmark session. Call this when a hotkey is triggered.
/// Returns the benchmark ID (timestamp) for correlation.
pub fn bench_start(label: &str) -> u64 {
    let epoch = BENCH_EPOCH.get_or_init(Instant::now);
    let now = epoch.elapsed().as_nanos() as u64;
    BENCH_START_NANOS.store(now, Ordering::SeqCst);
    let id = now / 1_000_000; // Use millis as ID
    log("BENCH", &format!("▶ START [{}] {}", id, label));
    id
}
/// Log a benchmark checkpoint with elapsed time from bench_start().
/// Format: [+XXXms] step_name
pub fn bench_log(step: &str) {
    let epoch = match BENCH_EPOCH.get() {
        Some(e) => e,
        None => {
            log("BENCH", &format!("⚠ {} (no bench_start called)", step));
            return;
        }
    };
    let start = BENCH_START_NANOS.load(Ordering::SeqCst);
    if start == 0 {
        log("BENCH", &format!("⚠ {} (bench not started)", step));
        return;
    }
    let now = epoch.elapsed().as_nanos() as u64;
    let elapsed_ms = (now - start) / 1_000_000;
    log("BENCH", &format!("[+{:>4}ms] {}", elapsed_ms, step));
}
/// Log a benchmark checkpoint with a custom elapsed time (for cross-process timing).
pub fn bench_log_with_elapsed(step: &str, elapsed_ms: u64) {
    log("BENCH", &format!("[+{:>4}ms] {}", elapsed_ms, step));
}
/// Get elapsed milliseconds since bench_start().
pub fn bench_elapsed_ms() -> u64 {
    let epoch = match BENCH_EPOCH.get() {
        Some(e) => e,
        None => return 0,
    };
    let start = BENCH_START_NANOS.load(Ordering::SeqCst);
    if start == 0 {
        return 0;
    }
    let now = epoch.elapsed().as_nanos() as u64;
    (now - start) / 1_000_000
}
/// End the benchmark and log total time.
pub fn bench_end(label: &str) {
    let elapsed = bench_elapsed_ms();
    log("BENCH", &format!("◼ END [+{}ms] {}", elapsed, label));
    BENCH_START_NANOS.store(0, Ordering::SeqCst);
}
// =============================================================================
// COMPACT AI FORMAT (SCRIPT_KIT_AI_LOG=1)
// =============================================================================

/// Category code mapping for compact AI logs.
/// See AGENTS.md for the full legend.
fn category_to_code(category: &str) -> char {
    match category.to_uppercase().as_str() {
        "POSITION" => 'P',
        "APP" => 'A',
        "UI" => 'U',
        "STDIN" => 'S',
        "HOTKEY" => 'H',
        "VISIBILITY" => 'V',
        "EXEC" => 'E',
        "KEY" => 'K',
        "FOCUS" => 'F',
        "THEME" => 'T',
        "CACHE" => 'C',
        "PERF" => 'R',
        "WINDOW_MGR" => 'W',
        "ERROR" => 'X',
        "MOUSE_HOVER" => 'M',
        "SCROLL_STATE" => 'L',
        "SCROLL_PERF" => 'Q',
        "SCRIPT" => 'G', // G for script loaGing (changed from B)
        "CONFIG" => 'N', // N for coNfig
        "RESIZE" => 'Z',
        "TRAY" => 'H',   // Tray is part of Hotkey subsystem
        "DESIGN" => 'D', // Design system
        "BENCH" => 'B',  // B for Benchmark timing
        "WARN" | "WARNING" => 'X',
        "WINDOW_STATE" | "WINDOW_OPS" | "WINDOW_REG" => 'W',
        "CHAT" | "AI" | "ACTIONS" | "ACTIONS_THEME" | "COMMAND_BAR" | "PROMPTS" | "PANEL"
        | "EDITOR" | "DIV" | "FIELD" | "SHORTCUT" | "ALIAS" | "ALIAS_INPUT" | "CONFIRM"
        | "SEARCH" | "FALLBACK" | "KEYWORD" | "HUD" => 'U',
        "DEBUG_GRID" => 'D',
        "SCRIPTLET_PARSE" => 'G',
        "CLICK" => 'M',
        "MCP" => 'S',
        "CLIPBOARD" | "DEEPLINK" | "TEST" | "SCHEDULER" | "SHUTDOWN" | "SECRETS" | "PROC" => 'A',
        "OCR" | "FONT" | "VIBRANCY" => 'T',
        _ => '-', // Unknown category
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegacyLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl LegacyLogLevel {
    fn as_json_label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}
fn legacy_level_for_category(category: &str) -> LegacyLogLevel {
    match category.to_uppercase().as_str() {
        "ERROR" => LegacyLogLevel::Error,
        "WARN" | "WARNING" => LegacyLogLevel::Warn,
        "DEBUG" => LegacyLogLevel::Debug,
        "TRACE" => LegacyLogLevel::Trace,
        _ => LegacyLogLevel::Info,
    }
}
/// Convert tracing Level to single char
fn level_to_char(level: Level) -> char {
    match level {
        Level::ERROR => 'e',
        Level::WARN => 'w',
        Level::INFO => 'i',
        Level::DEBUG => 'd',
        Level::TRACE => 't',
    }
}
/// Infer category code from tracing target path
fn infer_category_from_target(target: &str) -> char {
    // Match by module name in the target path
    // Group patterns by their category code to satisfy clippy
    if target.contains("executor") {
        'E' // Execution
    } else if target.contains("theme") {
        'T' // Theme
    } else if target.contains("window_manager") || target.contains("window_control") {
        'W' // Window manager
    } else if target.contains("stdin") || target.contains("protocol") || target.contains("mcp") {
        'S' // Stdin/protocol
    } else if target.contains("hotkey") || target.contains("tray") {
        'H' // Hotkey
    } else if target.contains("scripts") || target.contains("file_search") {
        'G' // Script loaGing (not execution)
    } else if target.contains("window_state")
        || target.contains("window_ops")
        || target.contains("window_reg")
    {
        'W' // Window state/ops
    } else if target.contains("config") {
        'N' // coNfig
    } else if target.contains("watcher")
        || target.contains("clipboard")
        || target.contains("logging")
        || target.contains("main")
        || target.contains("deeplink")
        || target.contains("scheduler")
        || target.contains("shutdown")
        || target.contains("window")
    {
        'A' // App lifecycle/subsystems
    } else if target.contains("panel")
        || target.contains("prompts")
        || target.contains("editor")
        || target.contains("terminal")
        || target.contains("term_prompt")
        || target.contains("pty")
        || target.contains("syntax")
        || target.contains("app_impl")
        || target.contains("actions")
        || target.contains("ai")
        || target.contains("notes")
    {
        'U' // UI components
    } else if target.contains("perf") {
        'R' // peRformance
    } else if target.contains("resize") {
        'Z' // resiZe
    } else {
        '-' // Unknown
    }
}
/// Get seconds.milliseconds within current minute
fn get_minute_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_millis = now.as_millis();
    let millis_in_minute = total_millis % 60_000;
    let seconds = millis_in_minute / 1000;
    let millis = millis_in_minute % 1000;
    format!("{:02}.{:03}", seconds, millis)
}
/// Visitor to extract category field from tracing events
struct CategoryExtractor {
    category: Option<String>,
    message: String,
    correlation_id: Option<String>,
}
impl CategoryExtractor {
    fn new() -> Self {
        Self {
            category: None,
            message: String::new(),
            correlation_id: None,
        }
    }
}
impl Visit for CategoryExtractor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "category" => self.category = Some(value.to_string()),
            "message" => self.message = value.to_string(),
            "correlation_id" => self.correlation_id = Some(value.to_string()),
            // Skip legacy field
            "legacy" => {}
            _ => {
                // Append other fields to message
                if !self.message.is_empty() {
                    self.message.push(' ');
                }
                let _ = write!(self.message, "{}={}", field.name(), value);
            }
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "category" => self.category = Some(format!("{:?}", value)),
            "message" => self.message = format!("{:?}", value),
            "correlation_id" => self.correlation_id = Some(format!("{:?}", value)),
            // Skip legacy field
            "legacy" => {}
            _ => {
                if !self.message.is_empty() {
                    self.message.push(' ');
                }
                let _ = write!(self.message, "{}={:?}", field.name(), value);
            }
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() != "legacy" {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{}={}", field.name(), value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() != "legacy" {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{}={}", field.name(), value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() != "legacy" {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{}={}", field.name(), value);
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name() != "legacy" {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            let _ = write!(self.message, "{}={:.2}", field.name(), value);
        }
    }
}
/// Compact AI formatter for stderr output.
/// Format: `SS.mmm|L|C|message`
pub struct CompactAiFormatter;
impl<S, N> FormatEvent<S, N> for CompactAiFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let timestamp = get_minute_timestamp();
        let level_char = level_to_char(*event.metadata().level());

        // Extract category and message from fields
        let mut extractor = CategoryExtractor::new();
        event.record(&mut extractor);

        // Infer category from target if not explicitly set
        let category_code = if let Some(ref cat) = extractor.category {
            category_to_code(cat)
        } else {
            // Try to infer from target (e.g., script_kit_gpui::executor -> E)
            let target = event.metadata().target();
            infer_category_from_target(target)
        };

        // Ensure correlation_id is always present
        let correlation_id = extractor
            .correlation_id
            .unwrap_or_else(current_correlation_id);

        // Build the compact line
        writeln!(
            writer,
            "{}|{}|{}|cid={} {}",
            timestamp, level_char, category_code, correlation_id, extractor.message
        )
    }
}
// =============================================================================
// JSON FORMATTER WITH CORRELATION ID INJECTION
// =============================================================================

#[derive(Default)]
struct JsonFieldCollector {
    fields: Map<String, Value>,
}
impl JsonFieldCollector {
    fn take(self) -> Map<String, Value> {
        self.fields
    }
}
impl Visit for JsonFieldCollector {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), Value::String(value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), Value::Bool(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), Value::Number(value.into()));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if let Some(num) = serde_json::Number::from_f64(value) {
            self.fields
                .insert(field.name().to_string(), Value::Number(num));
        } else {
            self.fields.insert(
                field.name().to_string(),
                Value::String(format!("{:.2}", value)),
            );
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            Value::String(format!("{:?}", value)),
        );
    }
}
fn value_to_string(value: Value) -> String {
    match value {
        Value::String(s) => s,
        other => other.to_string(),
    }
}
/// Ensures every JSON log line includes a correlation_id and message field.
#[derive(Default)]
pub struct JsonWithCorrelation;
