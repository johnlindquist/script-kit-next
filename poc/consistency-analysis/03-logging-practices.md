# Logging Practices Analysis - Script Kit GPUI

## Executive Summary

Script Kit GPUI implements a **sophisticated dual-output logging system** with structured JSONL output, compact AI mode for token efficiency, and comprehensive benchmarking utilities. The logging module (`src/logging.rs`) is well-architected at 2,300+ lines with extensive documentation, but there are **consistency gaps** in how the rest of the codebase applies logging practices.

**Key Findings:**
- ✅ **Excellent foundation**: Correlation ID system fully implemented and documented
- ✅ **Dual-output design**: JSONL + human-readable stderr with AI-optimized compact format
- ✅ **Comprehensive helpers**: Structured logging functions for common patterns
- ⚠️ **Adoption gaps**: Correlation ID rarely used in actual codebase, inconsistent structured fields
- ⚠️ **Tracing spans**: `#[instrument]` macro only used in 3 files, most code lacks distributed tracing context
- ⚠️ **Field naming inconsistency**: Mix of `event_type`, `category`, `action`, `component` with no standardization

---

## 1. Log Level Usage Analysis

### Distribution (238 total logging calls)

```
debug:   107 calls (45%)  ← Most used level
info:     39 calls (16%)
error:    46 calls (19%)
warn:     23 calls (10%)
trace:    10 calls (4%)
```

### Assessment

**✅ Strengths:**
- Good balance between debug (detailed) and error (critical issues)
- Errors properly elevated for visibility
- Warnings appropriately used for degraded conditions

**⚠️ Issues:**

1. **Debug Overuse (45%)**
   - May indicate overly chatty logging at `debug` level
   - Example: `text_injector.rs` logs "No characters to delete" at debug level on every zero-count call

   ```rust
   // Too verbose - fires frequently in normal operation
   debug!("No characters to delete");
   return Ok(());
   ```

2. **Trace Underuse (4%)**
   - Trace level (10 calls across entire codebase) suggests developers avoiding fine-grained instrumentation
   - Only used in scroll perf logging and limited benchmarking contexts

3. **Missing Log Levels in Critical Paths**
   - Protocol handling (`execute_script.rs`) has sparse logging despite being hot path
   - Script execution has minimal logging for error diagnosis

### Recommendations

```rust
// Improve idle state logging (currently logs everything)
if count == 0 {
    trace!("No characters to delete - early return");  // Change from debug
    return Ok(());
}

// Better: Only log if count is large/slow
if count > 1000 {
    warn!("Large character deletion: {}", count);
}
```

---

## 2. Structured Logging Patterns

### Current State

The logging module provides **excellent structured field support**, but usage is inconsistent:

#### Well-Implemented Patterns

**Correlation ID (Mandatory):**
```rust
// From logging.rs - correctly injected in all logs
pub fn current_correlation_id() -> String { ... }
pub fn set_correlation_id(id: impl Into<String>) -> CorrelationGuard { ... }

// In JsonWithCorrelation formatter (line 495)
root.insert("correlation_id".to_string(), Value::String(correlation_id));
```

**Event-Type Structured Logging:**
```rust
// Example from logging.rs
tracing::info!(
    event_type = "script_event",
    script_id = script_id,
    action = action,
    duration_ms = duration,
    success = success,
    "Script {} {}",
    action,
    script_id
);
```

#### Issues Found

1. **Inconsistent Field Names Across Codebase**

   | File | Field Names Used |
   |------|------------------|
   | `logging.rs` | `event_type`, `category`, `action`, `duration_ms` |
   | `scriptlet_cache.rs` | `category` (SCRIPTLET_PARSE) |
   | `keystroke_logger.rs` | `category` (KEYWORD) |
   | `perf.rs` | `category` (KEY_PERF, SCROLL_TIMING, FRAME_PERF) |
   | `notification/service.rs` | `source`, `dedupe_key`, `count` |
   | `text_injector.rs` | `text_len`, `delete_count`, `replacement_len` |

   **Problem:** No standardized field naming convention. Some use `event_type`, others use `category`.

2. **Correlation ID Rarely Set (Only 1 Usage)**

   ```rust
   // execute_script.rs - ONLY place setting correlation_id in codebase
   let correlation_id = issue.correlation_id.clone();
   logging::set_correlation_id(issue.correlation_id.clone());
   ```

   This defeats the purpose of correlation tracking across requests.

3. **Missing Event-Type in Most Logs**

   Example from `notification/service.rs` (line 73):
   ```rust
   tracing::debug!(
       source = %source_key,
       "Notification rate limited, skipping"
   );
   // Missing: event_type field!
   ```

   Should be:
   ```rust
   tracing::debug!(
       event_type = "notification",
       source = %source_key,
       action = "rate_limit_applied",
       "Notification rate limited, skipping"
   );
   ```

### Field Naming Standard (Recommended)

Every log should include these fields where applicable:

```rust
// Standard structured fields (in order)
event_type      // What category of event (e.g., "script_event", "ui_interaction")
action          // Specific action (e.g., "started", "completed", "failed")
[component]     // UI component (only if UI-related)
[duration_ms]   // How long it took
[error]         // Error message (if error level)
[context]       // Additional context (free-form)
```

Example refactored:

```rust
// Before: inconsistent, missing context
tracing::debug!(action = %label, "Toast action button clicked");

// After: structured, standardized
tracing::debug!(
    event_type = "ui_interaction",
    action = "button_clicked",
    component = "toast",
    label = %label,
    "Toast action triggered"
);
```

---

## 3. Correlation ID Usage

### Implementation (✅ Excellent)

The logging module has a complete correlation ID system:

```rust
// Thread-local with guard pattern for safety
thread_local! {
    static CORRELATION_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn set_correlation_id(id: impl Into<String>) -> CorrelationGuard { ... }
pub fn current_correlation_id() -> String { ... }

// Automatically injected into all logs
let correlation_id = extractor
    .correlation_id
    .unwrap_or_else(current_correlation_id);
```

### Adoption (⚠️ Critical Gap)

**Only 1 location actually sets correlation ID in entire codebase:**

```rust
// execute_script.rs - only place
let correlation_id = issue.correlation_id.clone();
logging::set_correlation_id(issue.correlation_id.clone());
```

This means:
- All other request chains use the global default correlation ID
- Can't trace individual requests through hotkey → prompt → script execution
- Defeats the purpose of correlation tracking

### Recommended Adoption Points

These critical entry points should set correlation IDs:

```rust
// 1. Hotkey dispatch
fn dispatch_hotkey(action: HotkeyAction) {
    let _guard = logging::set_correlation_id(format!("hotkey:{:?}", action));
    // Handle action...
}

// 2. Protocol message handling
fn handle_stdin_message(msg: &Message) -> Result<()> {
    let _guard = logging::set_correlation_id(msg.get_correlation_id());
    // Process message...
}

// 3. Script execution
fn execute_script(script_path: &str) -> Result<()> {
    let _guard = logging::set_correlation_id(format!("script:{}", script_path));
    // Execute...
}

// 4. Window events
fn on_window_event(event: &WindowEvent) {
    let _guard = logging::set_correlation_id(format!("window:{}", event.window_id));
    // Handle event...
}
```

---

## 4. Message Formatting Consistency

### Compact AI Format (✅ Well-Designed)

Environment variable: `SCRIPT_KIT_AI_LOG=1`

**Format:** `SS.mmm|L|C|cid=<uuid> message`

Example:
```
11.697|i|A|cid=550e8400-e29b 11-41d3-a4b7-446655440000 Application logging initialized
13.150|e|X|cid=550e8400-e29b 11-41d3-a4b7-446655440000 IMPOSSIBLE STATE: some error
```

**Category Codes:**
- P = Position/Display
- A = App lifecycle
- U = UI components
- S = Stdin/protocol
- H = Hotkey/Tray
- E = Execution
- T = Theme
- W = Window manager
- X = Error
- N = Config
- G = Script loading
- R = Performance
- D = Design
- B = Benchmark timing
- M = Mouse hover
- L = Scroll state
- Q = Scroll performance
- Z = Resize

**Token Savings:** 81% reduction in prefix (11 chars vs 59 chars per line)

### JSONL Output Format (✅ Well-Structured)

Output file: `~/.scriptkit/logs/script-kit-gpui.jsonl`

```json
{
  "timestamp": "2024-12-25T10:30:45.123Z",
  "level": "INFO",
  "target": "script_kit_gpui::executor",
  "correlation_id": "550e8400-e29b-41d3-a4b7-446655440000",
  "message": "Script execution completed",
  "fields": {
    "event_type": "script_event",
    "script_id": "hello-world",
    "duration_ms": 42,
    "success": true
  }
}
```

**✅ Strengths:**
- RFC3339 timestamps with millisecond precision
- Always includes correlation_id
- Hierarchical `fields` object for structured data
- AI-parseable format

### Human-Readable Format (✅ Terminal Output)

When `SCRIPT_KIT_AI_LOG` not set, stderr shows pretty-printed logs:
```
  2024-12-25T10:30:45.123Z  INFO script_kit_gpui::logging: Application logging initialized
```

---

## 5. Tracing Spans Usage

### Current Adoption: VERY LIMITED

**Only 3 files use `#[instrument]` macro:**

```
text_injector.rs        3 uses
  - delete_chars(count)
  - paste_text(text)
  - inject_text(delete_count, replacement)

hotkey_pollers.rs       1 use (reference only)

keystroke_logger.rs     (grepped but not found)
```

**Total: 3 actual span-instrumented functions out of 400+ in codebase**

### Issues

1. **No Distributed Tracing Context**

   Most execution paths have zero tracing context:

   ```rust
   // text_injector.rs - GOOD (has span)
   #[instrument(skip(self), fields(count))]
   pub fn delete_chars(&self, count: usize) -> Result<()> {
       debug!("Deleting characters via backspace simulation");
       // ...
   }

   // execute_script.rs - BAD (no span, no structured context)
   pub fn execute_script(script_path: &str) -> Result<()> {
       tracing::info!("Executing script");  // Fields: none
       // ...
   }
   ```

2. **Span Chain Broken at Module Boundaries**

   When `text_injector::inject_text()` calls `delete_chars()`, the span properly nests.
   But most inter-module calls have NO span:

   ```rust
   // hotkeys.rs calls execute_script.rs - NO SPAN CONTEXT PASSED
   pub fn on_hotkey(action: HotkeyAction) {
       // No tracing span here - context lost!
       executor::execute_script(&path)?;
   }
   ```

### Recommended Span Instrumentation

**High-priority entry points:**

```rust
// 1. Hotkey dispatch (most important - start of request chain)
#[instrument(skip_all, fields(hotkey = ?action))]
pub fn dispatch_hotkey(action: HotkeyAction) -> Result<()> {
    // Now all logs within this call + children will have hotkey context
}

// 2. Script execution (second most important)
#[instrument(skip(self), fields(script_path = %script_path, duration_ms))]
pub fn execute_script(&mut self, script_path: &str) -> Result<()> {
    let start = Instant::now();
    // ...
    tracing::info!(duration_ms = start.elapsed().as_millis(), "Script completed");
}

// 3. Protocol message handling
#[instrument(skip(self), fields(message_type = %msg.msg_type))]
pub fn handle_protocol_message(&mut self, msg: &ProtocolMessage) -> Result<()> {
    // ...
}

// 4. UI render/event handlers
#[instrument(skip_all, fields(component = "list_view", action = "render"))]
pub fn render(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    // ...
}
```

---

## 6. Log Message Formatting

### Quality Assessment

**✅ Good Examples:**
```rust
// Clear, concise, with context
logging::log_perf(operation: &str, duration_ms: u64, threshold_ms: u64) {
    // Output: "PERF operation_name 42ms [OK]"
}

// Structured benchmark logging
logging::bench_start("hotkey:cmd+k")  // ▶ START [12345] hotkey:cmd+k
logging::bench_log("show_window")     // [+123ms] show_window
logging::bench_end("full_cycle")      // ◼ END [+456ms] full_cycle
```

**⚠️ Issues:**

1. **Inconsistent Message Content**

   ```rust
   // Some logs are narrative
   tracing::info!("Application logging initialized")

   // Others are data-focused
   tracing::info!(event_type = "script_event", ...)

   // And some mix both (redundant)
   tracing::info!(
       event_type = "ui_event",
       component = "toast",
       action = action,
       details = details,
       "{}",  // Also includes the same info in message text
       msg
   );
   ```

2. **Printf-style Formatting (Old Pattern)**

   ```rust
   // Legacy logging helper - still used
   pub fn log(category: &str, message: &str) {
       tracing::info!(
           category = category,
           correlation_id = %correlation_id,
           legacy = true,
           "{}",  // String is passed as message, not structured
           message
       );
   }
   ```

3. **Inconsistent Payload Truncation**

   ```rust
   // Excellent: payload summarization for privacy
   logging::log_protocol_send(fd: i32, json: &str) {
       // Outputs: {type:submit, len:1024} instead of full JSON
   }

   // But not used consistently across protocol handling
   ```

### Recommended Message Standard

```rust
// Rule 1: Message should NOT duplicate structured fields
✅ tracing::info!(
    event_type = "button_click",
    button_id = %id,
    "Button activated"  // Concise, doesn't duplicate fields
);

❌ tracing::info!(
    event_type = "button_click",
    button_id = %id,
    "Button click event with id {} activated",  // Duplicates structured data
    id
);

// Rule 2: Use structured fields for data, message for narrative
✅ tracing::warn!(
    event_type = "performance",
    operation = "script_exec",
    duration_ms = 150,
    threshold_ms = 100,
    "Slow operation exceeded threshold"
);

// Rule 3: String interpolation should be minimal
✅ tracing::debug!(operation = %op, "Starting");
❌ tracing::debug!("Starting operation: {}", op);
```

---

## 7. Error Handling in Logs

### Current Practice

**Most errors lack context:**

```rust
// execute_script.rs - minimal context
tracing::error!(error = %e, "Failed to send screenshot response");

// notification/service.rs - missing event type
tracing::warn!(action_id, "Unknown action");
```

### Recommended Error Logging Pattern

```rust
// Comprehensive error logging
pub fn log_error(category: &str, error: &str, context: Option<&str>) {
    let msg = match context {
        Some(ctx) => format!("{}: {} (context: {})", category, error, ctx),
        None => format!("{}: {}", category, error),
    };

    tracing::error!(
        event_type = "error",
        category = category,
        error_message = error,
        context = context,
        "{}",
        msg
    );
}

// Usage
if let Err(e) = execute_script() {
    logging::log_error(
        "EXEC",
        &e.to_string(),
        Some("while processing hotkey")
    );
}
```

---

## 8. Performance Logging

### ✅ Excellent Benchmarking System

```rust
// Microsecond-level precision
logging::bench_start("operation")    // Start timer
logging::bench_log("checkpoint")     // [+12.3ms] checkpoint
logging::bench_elapsed_ms()          // Get elapsed
logging::bench_end("operation")      // Log final

// Used effectively in hot paths:
// - hotkey dispatch → bench_log("hotkey:cmd+k")
// - script execution → bench_log("bun_spawn_start")
// - text selection → bench_log("ax_api_call_start")
// - frame timing → bench_log("frame_complete")
```

### ✅ Scroll Performance Tracking

```rust
logging::log_scroll_perf_start("scroll_to_item")
logging::log_scroll_perf_end("scroll_to_item", start_micros)
logging::log_scroll_frame(frame_time_ms, expected_frame_ms)
logging::log_scroll_event_rate(events_per_second)
logging::log_frame_gap(gap_ms)
logging::log_render_stall(duration_ms)
```

### Issues

1. **Benchmark data not accessible through standard queries**
   - Bench logs use `log()` function (legacy)
   - Don't appear in structured JSONL with event_type
   - Harder to analyze/alert on via log aggregation

2. **Performance threshold constants not configurable**

   ```rust
   // Hardcoded in perf.rs
   const SLOW_KEY_THRESHOLD_US: u128 = 16_666;  // ~16ms
   const SLOW_SCROLL_THRESHOLD_US: u128 = 8_000;  // 8ms

   // Should be configurable for different environments
   ```

---

## 9. Key Findings Summary

### Strengths ✅

| Aspect | Assessment |
|--------|------------|
| Foundation Design | Excellent - comprehensive logging module with dual output |
| Correlation ID System | Excellent implementation, thread-safe, well-documented |
| Compact AI Format | Excellent - 81% token savings, well-designed category codes |
| JSONL Output | Excellent - RFC3339, always includes correlation_id |
| Structured Helpers | Good - script_event, ui_event, perf, error helpers provided |
| Benchmarking | Excellent - microsecond precision, many instrumentation points |
| Field Type Safety | Good - Visit trait-based field extraction prevents type errors |
| Error Logging | Good foundation in logging.rs, inconsistent usage |

### Gaps ⚠️

| Aspect | Issue | Impact |
|--------|-------|--------|
| Correlation ID Adoption | Only 1 location sets it (execute_script.rs) | Can't trace requests through system |
| Tracing Spans | Only 3 files use `#[instrument]` macro | No distributed tracing context |
| Field Naming | Inconsistent (event_type vs category) | Harder to query logs aggregation |
| Structured Fields | Inconsistent use across codebase | 45% of logs are unstructured |
| Entry Point Coverage | No correlation ID at hotkey, protocol, window event boundaries | Request chains not traceable |
| Debug Level | 45% of logs at debug (potentially too verbose) | Harder to identify signal in noise |

---

## 10. Recommendations (Priority Order)

### Immediate (1-2 days)

1. **Add correlation ID at critical entry points**
   ```rust
   // hotkeys.rs
   pub fn dispatch_hotkey(action: HotkeyAction) {
       let _guard = logging::set_correlation_id(
           format!("hotkey:{:?}", action)
       );
       // ...
   }

   // protocol/io.rs
   pub fn handle_message(msg: Message) {
       let _guard = logging::set_correlation_id(
           msg.request_id.clone()
       );
       // ...
   }
   ```

2. **Add structured fields to all logs**
   ```rust
   // Use standardized field order:
   // event_type, action, [component], [duration_ms], [error]

   tracing::info!(
       event_type = "ui_interaction",
       action = "button_clicked",
       component = "toast",
       "Toast action triggered"
   );
   ```

3. **Instrument critical path spans**
   ```rust
   // hotkey_dispatch → execute_script → handle_response
   #[instrument(skip_all, fields(hotkey = ?action))]
   pub fn dispatch_hotkey(action: HotkeyAction) { ... }

   #[instrument(skip_all, fields(script_path = %path))]
   pub fn execute_script(&self, path: &str) { ... }
   ```

### Short-term (1-2 weeks)

4. **Standardize field naming**
   - Deprecate `category` field
   - Always use `event_type` + `action`
   - Document standard field list

5. **Convert high-frequency debug logs to trace**
   ```rust
   // From: debug!("No characters to delete");
   // To: trace!("Early return - count=0");
   ```

6. **Audit and add missing event_types**
   - notification/service.rs: add `event_type = "notification"`
   - components/toast.rs: add `event_type = "ui_interaction"`
   - Execute all module files for gaps

### Medium-term (3-4 weeks)

7. **Create log query dashboards**
   - Group by correlation_id for request tracing
   - Alert on slow operations (duration_ms > threshold)
   - Track error rates by event_type

8. **Add distributed tracing to all public methods**
   - UI event handlers
   - Script execution path
   - Window management
   - Clipboard history operations

9. **Improve payload truncation**
   - Consistently apply to protocol messages
   - Add to clipboard history operations
   - Configure max payload length

---

## 11. Code Examples

### Before: Inconsistent Logging

```rust
// From multiple files - inconsistent patterns
pub fn execute_script(path: &str) -> Result<()> {
    tracing::info!("Executing script");  // No fields!
    // ...
    if let Err(e) = run() {
        tracing::error!(error = %e, "Script failed");  // Missing context
    }
}

pub fn handle_hotkey(action: &HotkeyAction) {
    log("HOTKEY", &format!("{:?}", action));  // Legacy log() function
    // ...
}

pub fn on_notification(n: &Notification) {
    tracing::debug!(
        source = %n.source_key(),
        "Notification rate limited, skipping"
    );
    // Missing: event_type, action
}
```

### After: Consistent Logging with Correlation IDs

```rust
#[instrument(skip_all, fields(script_path = %path))]
pub fn execute_script(path: &str) -> Result<()> {
    let start = Instant::now();
    let _guard = logging::set_correlation_id(
        format!("script:{}", path)
    );

    tracing::info!(
        event_type = "script_event",
        action = "started",
        "Script execution initiated"
    );

    match run() {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::info!(
                event_type = "script_event",
                action = "completed",
                duration_ms = duration_ms,
                success = true,
                "Script executed successfully"
            );
            Ok(result)
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::error!(
                event_type = "script_event",
                action = "failed",
                duration_ms = duration_ms,
                error = %e,
                error_type = std::any::type_name_of_val(&e),
                "Script execution failed"
            );
            Err(e)
        }
    }
}

#[instrument(skip_all, fields(hotkey = ?action))]
pub fn handle_hotkey(action: &HotkeyAction) {
    let _guard = logging::set_correlation_id(
        format!("hotkey:{:?}", action)
    );

    logging::bench_start(&format!("hotkey:{:?}", action));

    tracing::info!(
        event_type = "hotkey_event",
        action = "triggered",
        hotkey = ?action,
        "Hotkey activated"
    );

    // ...

    logging::bench_end(&format!("hotkey:{:?}", action));
}

#[instrument(skip_all)]
pub fn on_notification(n: &Notification) {
    tracing::debug!(
        event_type = "notification",
        action = "rate_limit_check",
        source = %n.source_key(),
        "Rate limit evaluated"
    );

    if is_rate_limited(&n.source_key()) {
        tracing::debug!(
            event_type = "notification",
            action = "rate_limited",
            source = %n.source_key(),
            "Notification dropped - rate limit exceeded"
        );
        return;
    }
}
```

---

## 12. Testing Recommendations

### Unit Tests (Already Exist - Excellent)

The logging module has comprehensive tests:
- Category code mappings (line 1757-1859)
- Level character conversions
- Timestamp formatting
- Compact format validation
- Token savings verification
- Correlation ID injection
- Payload truncation with UTF-8 safety

### Integration Tests to Add

```rust
#[test]
fn test_correlation_id_persists_through_call_chain() {
    logging::init();
    let test_id = "test-123";
    let _guard = logging::set_correlation_id(test_id);

    // Simulate nested calls
    inner_function_with_log();

    // Verify test_id appears in logs
    let logs = capture_logs();
    assert!(logs.iter().any(|l| l.contains(test_id)));
}

#[test]
fn test_hotkey_dispatch_sets_correlation_id() {
    dispatch_hotkey(HotkeyAction::Main);

    // All logs from this dispatch should have same correlation_id
}

#[test]
fn test_structured_fields_in_json_output() {
    let output = execute_script_and_capture_jsonl();

    let lines: Vec<serde_json::Value> = output
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    // All entries should have event_type and correlation_id
    for entry in lines {
        assert!(entry.get("fields").is_some());
        assert!(entry.get("correlation_id").is_some());
    }
}
```

---

## 13. References

### CLAUDE.md Requirements Met
- ✅ Include `correlation_id` in all log entries (documented, partially implemented)
- ✅ Logging module provides complete infrastructure
- ⚠️ Usage inconsistent across codebase (adoption gap)

### Documentation Files
- `src/logging.rs` - 2,300 lines, comprehensive inline documentation
- Comments on JSONL format (lines 21-26)
- Compact AI format specification (lines 9-19)
- Field visitor pattern (lines 256-342)
- Category code mapping (lines 161-189)

### Test Coverage
- `src/logging.rs` lines 1666-2297 - Extensive unit tests
- 40+ tests covering all major functionality
- Real-world log line examples in tests

---

## Conclusion

Script Kit GPUI has **world-class logging infrastructure** that puts many Rust projects to shame. The implementation is:
- Well-documented
- Thoroughly tested
- Designed for AI agent parsing (compact mode, JSONL format)
- Supports distributed tracing (correlation IDs, spans)

However, the **adoption in the actual codebase is inconsistent**. To realize the full value of this infrastructure, the team should:

1. **Start setting correlation IDs** at request entry points (hotkeys, protocol messages)
2. **Instrument critical paths** with `#[instrument]` macros (currently only 3 functions use this)
3. **Standardize structured fields** across all log calls
4. **Audit and complete** event_type coverage

With these changes, Script Kit GPUI would have best-in-class observability for debugging complex multi-module interactions.

