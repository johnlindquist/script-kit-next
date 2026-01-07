# Logging & Observability - Expert Bundle

## Overview

Script Kit uses structured JSONL logging with a compact AI mode, correlation IDs for tracing, and performance timing for debugging.

## Logging Architecture

### Dual Output System (src/logging.rs)

```rust
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

pub fn init_logging() -> Result<()> {
    let log_path = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/logs/script-kit-gpui.jsonl");
    
    // Ensure directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive("script_kit=debug".parse()?))
        .with(fmt::layer()
            .json()
            .with_writer(file)
            .with_target(true)
            .with_thread_ids(true));
    
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

### Compact AI Mode (SCRIPT_KIT_AI_LOG=1)

```rust
// Format: SS.mmm|L|C|message
// L: i=INFO, w=WARN, e=ERROR, d=DEBUG, t=TRACE
// C: Category code

pub fn log_compact(category: &str, level: Level, message: &str) {
    let elapsed = START_TIME.elapsed();
    let secs = elapsed.as_secs() % 60;
    let millis = elapsed.subsec_millis();
    
    let level_char = match level {
        Level::INFO => 'i',
        Level::WARN => 'w',
        Level::ERROR => 'e',
        Level::DEBUG => 'd',
        Level::TRACE => 't',
    };
    
    let cat_char = category_to_char(category);
    
    eprintln!("{:02}.{:03}|{}|{}|{}", secs, millis, level_char, cat_char, message);
}

fn category_to_char(category: &str) -> char {
    match category {
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
        "DESIGN" => 'D',
        "SCRIPT" => 'G',
        "CONFIG" => 'N',
        "RESIZE" => 'Z',
        _ => '?',
    }
}
```

### Log Function

```rust
static AI_LOG_MODE: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("SCRIPT_KIT_AI_LOG").map(|v| v == "1").unwrap_or(false)
});

pub fn log(category: &str, message: &str) {
    if *AI_LOG_MODE {
        log_compact(category, Level::INFO, message);
    } else {
        info!(category = category, "{}", message);
    }
}

pub fn log_warn(category: &str, message: &str) {
    if *AI_LOG_MODE {
        log_compact(category, Level::WARN, message);
    } else {
        warn!(category = category, "{}", message);
    }
}

pub fn log_error(category: &str, message: &str) {
    if *AI_LOG_MODE {
        log_compact(category, Level::ERROR, message);
    } else {
        error!(category = category, "{}", message);
    }
}
```

## Structured Logging

### Typed Fields

```rust
// Good - typed fields for querying
info!(
    script_path = %path,
    duration_ms = duration.as_millis() as u64,
    exit_code = code,
    "Script execution complete"
);

// Bad - string interpolation loses structure
info!("Script {} completed in {}ms with code {}", path, duration, code);
```

### JSONL Output

```json
{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","target":"script_kit::executor","message":"Script execution complete","fields":{"script_path":"hello.ts","duration_ms":142,"exit_code":0}}
```

## Correlation IDs

### Generating IDs

```rust
use uuid::Uuid;

pub struct CorrelationId(String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### Span Integration

```rust
use tracing::{instrument, Span};

#[instrument(skip(cx), fields(correlation_id = %correlation_id))]
pub fn execute_script(path: &str, correlation_id: &CorrelationId, cx: &mut Context<Self>) {
    info!("Starting script execution");
    
    // All nested logs inherit correlation_id
    let result = spawn_script(path);
    
    match result {
        Ok(_) => info!("Script spawned successfully"),
        Err(e) => error!(error = ?e, "Script spawn failed"),
    }
}
```

### Querying by Correlation ID

```bash
# Find all logs for a specific execution
grep '"correlation_id":"abc-123"' ~/.scriptkit/logs/script-kit-gpui.jsonl
```

## Performance Timing

### Duration Logging

```rust
use std::time::Instant;

pub fn timed<F, R>(operation: &str, f: F) -> R 
where F: FnOnce() -> R {
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    
    if duration.as_millis() > 100 {
        warn!(
            operation = operation,
            duration_ms = duration.as_millis() as u64,
            "[PERF_SLOW] Operation exceeded threshold"
        );
    } else {
        debug!(
            operation = operation,
            duration_ms = duration.as_millis() as u64,
            "Operation complete"
        );
    }
    
    result
}
```

### Performance Spans

```rust
#[instrument(fields(duration_ms))]
pub fn filter_scripts(&mut self, query: &str) {
    let start = Instant::now();
    
    self.filtered_scripts = self.all_scripts
        .iter()
        .filter(|s| s.matches(query))
        .cloned()
        .collect();
    
    Span::current().record("duration_ms", start.elapsed().as_millis() as u64);
}
```

## Log Categories

### Category Reference

| Code | Category | Use Case |
|------|----------|----------|
| P | POSITION | Window positioning |
| A | APP | Application lifecycle |
| U | UI | UI rendering |
| S | STDIN | Stdin protocol |
| H | HOTKEY | Hotkey events |
| V | VISIBILITY | Show/hide |
| E | EXEC | Script execution |
| K | KEY | Keyboard events |
| F | FOCUS | Focus changes |
| T | THEME | Theme loading |
| C | CACHE | Cache operations |
| R | PERF | Performance |
| W | WINDOW_MGR | Window management |
| X | ERROR | Errors |
| M | MOUSE_HOVER | Mouse hover |
| L | SCROLL_STATE | Scroll state |
| Q | SCROLL_PERF | Scroll performance |
| D | DESIGN | Design tokens |
| G | SCRIPT | Script loading |
| N | CONFIG | Configuration |
| Z | RESIZE | Window resize |

## Log Filtering

### By Category

```bash
# Filter by category in compact mode
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|E|'

# Multiple categories
grep -E '\|E\||\|H\||\|K\|' log.txt
```

### By Level

```bash
# Errors only
grep '"level":"ERROR"' ~/.scriptkit/logs/script-kit-gpui.jsonl

# Warnings and errors
grep -E '"level":"(ERROR|WARN)"' ~/.scriptkit/logs/script-kit-gpui.jsonl
```

### By Duration

```bash
# Slow operations (>100ms)
cat log.jsonl | jq 'select(.fields.duration_ms > 100)'
```

## Debug Panel

### In-App Log Viewing

```rust
pub struct DebugPanel {
    logs: VecDeque<LogEntry>,
    max_entries: usize,
    filter: Option<String>,
    show_level: Level,
}

impl DebugPanel {
    pub fn add_log(&mut self, entry: LogEntry) {
        self.logs.push_back(entry);
        if self.logs.len() > self.max_entries {
            self.logs.pop_front();
        }
    }

    pub fn filtered_logs(&self) -> impl Iterator<Item = &LogEntry> {
        self.logs.iter().filter(|e| {
            e.level >= self.show_level &&
            self.filter.as_ref()
                .map(|f| e.message.contains(f))
                .unwrap_or(true)
        })
    }
}
```

### Keyboard Shortcut

```rust
// Cmd+L toggles debug panel
fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
    if event.modifiers.command && event.key == "l" {
        self.toggle_debug_panel(cx);
    }
}
```

## Best Practices

### 1. Use Appropriate Levels

```rust
// ERROR: Operation failed, user impact
error!(error = ?e, "Failed to save note");

// WARN: Unexpected but handled
warn!("Config file not found, using defaults");

// INFO: Key events, state changes
info!(script = %path, "Script started");

// DEBUG: Development details
debug!(choice_count = choices.len(), "Filtered choices");

// TRACE: Very verbose
trace!(event = ?event, "Processing key event");
```

### 2. Include Context

```rust
// Good - includes all relevant context
error!(
    script_path = %path,
    error = ?e,
    exit_code = status.code(),
    stderr = %stderr,
    "Script execution failed"
);

// Bad - missing context
error!("Script failed: {}", e);
```

### 3. Log at Boundaries

```rust
// Log at entry and exit of significant operations
pub fn execute_script(&mut self, path: &str) {
    info!(path = %path, "Starting script execution");
    
    // ... execution logic ...
    
    match result {
        Ok(_) => info!(path = %path, "Script completed successfully"),
        Err(e) => error!(path = %path, error = ?e, "Script execution failed"),
    }
}
```

### 4. Avoid Log Spam

```rust
// Bad - logs on every key press
fn on_key_down(&mut self, key: &str) {
    debug!("Key pressed: {}", key);  // Spammy!
}

// Good - log only significant events
fn on_key_down(&mut self, key: &str) {
    if key == "enter" {
        info!("Submit triggered");
    }
}
```

## Querying Logs

```bash
# Recent errors
tail -100 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep ERROR

# Performance issues
cat log.jsonl | jq 'select(.fields.duration_ms > 100) | {msg: .message, ms: .fields.duration_ms}'

# Script execution timeline
grep '"script"' log.jsonl | jq '{time: .timestamp, msg: .message}'
```

## Summary

1. **JSONL format** for structured logs
2. **Compact AI mode** saves tokens (SCRIPT_KIT_AI_LOG=1)
3. **Correlation IDs** for request tracing
4. **Duration tracking** for performance
5. **Category codes** for filtering
6. **Typed fields** for queryability
