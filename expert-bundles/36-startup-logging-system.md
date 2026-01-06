# Script Kit GPUI - Expert Review Request

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Architecture:**
- **GPUI** for UI rendering (custom immediate-mode reactive UI framework from Zed)
- **Bun** as the TypeScript runtime for user scripts
- **Stdin/stdout JSON protocol** for bidirectional script ↔ app communication
- **SQLite** for persistence (clipboard history, notes, chat)
- **macOS-first** with floating panel window behavior

**Key Constraints:**
- Must maintain backwards compatibility with existing Script Kit scripts
- Performance-critical: launcher must appear instantly, list scrolling at 60fps
- Multi-window: main launcher + Notes window + AI chat window (all independent)
- Theme hot-reload across all windows

---

## Bundle: Logging System (src/logging.rs)

This bundle documents the dual-output logging system optimized for both AI agents and human developers.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Logging Output Destinations                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────┐                                                   │
│   │    tracing::info!   │                                                   │
│   │    tracing::error!  │                                                   │
│   │    logging::log()   │                                                   │
│   └──────────┬──────────┘                                                   │
│              │                                                              │
│              ▼                                                              │
│   ┌─────────────────────────────────────────────────────────────┐          │
│   │              tracing-subscriber Registry                     │          │
│   └─────────────────────────────────────────────────────────────┘          │
│              │                                     │                        │
│              ▼                                     ▼                        │
│   ┌─────────────────────┐           ┌─────────────────────────────┐        │
│   │   JSON Layer        │           │   Stderr Layer               │        │
│   │   (file output)     │           │   (human/AI output)          │        │
│   └─────────────────────┘           └─────────────────────────────┘        │
│              │                                     │                        │
│              ▼                                     ▼                        │
│   ┌─────────────────────┐           ┌─────────────────────────────┐        │
│   │  ~/.scriptkit/logs/ │           │  SCRIPT_KIT_AI_LOG=1?       │        │
│   │  script-kit-gpui.   │           │  ├─ Yes: Compact AI format  │        │
│   │  jsonl              │           │  └─ No: Pretty format       │        │
│   └─────────────────────┘           └─────────────────────────────┘        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Output Formats

### 1. JSONL File Output

Path: `~/.scriptkit/logs/script-kit-gpui.jsonl`

```json
{"timestamp":"2024-12-25T10:30:45.123Z","level":"INFO","target":"script_kit_gpui::main","message":"Script executed","fields":{"event_type":"script_event","script_id":"hello.ts","duration_ms":42}}
```

**Features:**
- One JSON object per line (JSONL format)
- RFC 3339 timestamp
- Structured fields for machine parsing
- Non-blocking file writes

---

### 2. Pretty Stderr (Default)

```
2025-12-27T15:22:13.150640Z  INFO script_kit_gpui::logging: Application logging initialized
```

**Features:**
- Full timestamp
- Colored output (ANSI)
- Module path target
- Human-readable

---

### 3. Compact AI Format (SCRIPT_KIT_AI_LOG=1)

```
13.150|i|P|Selected display origin=(0,0)
```

Format: `SS.mmm|L|C|message`

| Part | Description |
|------|-------------|
| `SS.mmm` | Seconds.milliseconds within current minute |
| `L` | Level: i/w/e/d/t (info/warn/error/debug/trace) |
| `C` | Category code (single char) |
| `message` | The log message |

**Category Codes:**

| Code | Category | Description |
|------|----------|-------------|
| P | POSITION | Window positioning |
| A | APP | App lifecycle |
| U | UI | UI components |
| S | STDIN | Stdin protocol |
| H | HOTKEY | Hotkeys and tray |
| V | VISIBILITY | Window show/hide |
| E | EXEC | Script execution |
| K | KEY | Keyboard events |
| F | FOCUS | Focus management |
| T | THEME | Theme loading |
| C | CACHE | Caching |
| R | PERF | Performance |
| W | WINDOW_MGR | Window manager |
| X | ERROR | Errors |
| M | MOUSE_HOVER | Mouse hover |
| L | SCROLL_STATE | Scroll state |
| Q | SCROLL_PERF | Scroll performance |
| B | SCRIPT | Script loading |
| N | CONFIG | Configuration |
| Z | RESIZE | Window resize |
| D | DESIGN | Design system |

---

## Initialization

```rust
pub fn init() -> LoggingGuard {
    // Initialize in-memory buffer for UI display
    let _ = LOG_BUFFER.set(Mutex::new(VecDeque::with_capacity(50)));

    // Check for AI compact mode
    let ai_log_mode = std::env::var("SCRIPT_KIT_AI_LOG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // Create log directory
    let log_dir = ~/.scriptkit/logs/;
    fs::create_dir_all(&log_dir)?;

    // Open log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("script-kit-gpui.jsonl"))?;

    // Non-blocking writer (prevents UI freeze)
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file);

    // Environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,gpui=warn,hyper=warn,reqwest=warn"));

    // JSON layer for file
    let json_layer = fmt::layer()
        .json()
        .with_writer(non_blocking_file)
        .with_timer(fmt::time::UtcTime::rfc_3339());

    // Stderr layer (compact or pretty)
    if ai_log_mode {
        let ai_layer = fmt::layer()
            .with_writer(StderrWriter)
            .with_ansi(false)
            .event_format(CompactAiFormatter);
        
        registry.with(json_layer).with(ai_layer).init();
    } else {
        let pretty_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .compact();
        
        registry.with(json_layer).with(pretty_layer).init();
    }

    LoggingGuard { _file_guard: file_guard }
}
```

---

## LoggingGuard

```rust
pub struct LoggingGuard {
    _file_guard: WorkerGuard,
}
```

**CRITICAL**: The guard MUST be kept alive for the entire program. Dropping it flushes and closes the log file.

```rust
fn main() {
    let _guard = logging::init();  // Keep guard alive
    // ... rest of program
}  // Guard dropped here, logs flushed
```

---

## Legacy API Support

For backwards compatibility, a simple `log()` function is provided:

```rust
pub fn log(category: &str, message: &str) {
    // Add to UI buffer
    add_to_buffer(category, message);
    
    // Use tracing
    tracing::info!(category = category, legacy = true, "{}", message);
}

// Usage:
logging::log("APP", "Application starting");
```

---

## Structured Logging Helpers

### Script Events

```rust
pub fn log_script_event(script_id: &str, action: &str, duration_ms: Option<u64>, success: bool) {
    tracing::info!(
        event_type = "script_event",
        script_id = script_id,
        action = action,
        duration_ms = duration_ms,
        success = success,
        "Script {} {}",
        action,
        script_id
    );
}
```

### UI Events

```rust
pub fn log_ui_event(component: &str, action: &str, details: Option<&str>) {
    tracing::info!(
        event_type = "ui_event",
        component = component,
        action = action,
        details = details,
        "..."
    );
}
```

### Performance

```rust
pub fn log_perf(operation: &str, duration_ms: u64, threshold_ms: u64) {
    let is_slow = duration_ms > threshold_ms;
    
    if is_slow {
        tracing::warn!(
            event_type = "performance",
            operation = operation,
            duration_ms = duration_ms,
            is_slow = true,
            "Slow operation: {} took {}ms",
            operation,
            duration_ms
        );
    }
}
```

---

## Payload Truncation

Large payloads (screenshots, clipboard) are truncated:

```rust
pub fn truncate_for_log(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...({})", &s[..max_len], s.len())
    }
}

pub fn summarize_payload(json: &str) -> String {
    // Extract type from JSON
    let msg_type = extract_type(json);
    format!("{{type:{}, len:{}}}", msg_type, json.len())
}

// Usage:
log_protocol_send(fd, &summarize_payload(&json));
// Output: →stdin fd=5: {type:screenshotResult, len:125000}
```

---

## In-Memory Buffer

For UI log panel display:

```rust
static LOG_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
const MAX_LOG_LINES: usize = 50;

pub fn get_recent_logs() -> Vec<String> {
    if let Some(buffer) = LOG_BUFFER.get() {
        if let Ok(buf) = buffer.lock() {
            return buf.iter().cloned().collect();
        }
    }
    Vec::new()
}
```

---

## Token Savings Analysis

Compact AI format saves ~80% on log prefixes:

| Format | Prefix Example | Length |
|--------|----------------|--------|
| Standard | `2025-12-27T15:22:13.150640Z  INFO script_kit_gpui::logging:` | 59 chars |
| Compact | `13.150\|i\|A\|` | 11 chars |

**Savings**: 81% per line

---

## Usage Examples

### Run with AI Log Mode

```bash
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### Filter JSONL Logs

```bash
# Errors only
grep '"level":"ERROR"' ~/.scriptkit/logs/script-kit-gpui.jsonl

# Slow operations (>100ms)
grep '"duration_ms":' ~/.scriptkit/logs/script-kit-gpui.jsonl | \
  jq 'select(.fields.duration_ms > 100)'

# By correlation ID
grep '"correlation_id":"abc-123"' ~/.scriptkit/logs/script-kit-gpui.jsonl
```

### Query Compact Logs

```bash
# Errors only
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|e|'

# Focus events
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|F|'
```

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
