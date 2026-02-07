// =============================================================================
// BACKWARD COMPATIBILITY - Legacy log() function wrappers
// =============================================================================

/// Legacy log function - wraps tracing::info! for backward compatibility.
///
/// Prefer using tracing macros directly for structured fields:
/// ```rust
/// tracing::info!(category = "UI", duration_ms = 42, "Button clicked");
/// ```
pub fn log(category: &str, message: &str) {
    // Add to legacy buffer for UI display
    add_to_buffer(category, message);

    let correlation_id = current_correlation_id();
    let level = legacy_level_for_category(category);

    // Write to capture file if capture is enabled
    if is_capture_enabled() {
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let json_line = format!(
            r#"{{"timestamp":"{}","level":"{}","category":"{}","correlation_id":"{}","message":"{}"}}"#,
            timestamp,
            level.as_json_label(),
            category,
            correlation_id,
            message
        );
        write_to_capture(&json_line);
    }

    // Preserve intended severity for legacy category-only callsites.
    match level {
        LegacyLogLevel::Error => tracing::error!(
            category = category,
            correlation_id = %correlation_id,
            legacy = true,
            "{}",
            message
        ),
        LegacyLogLevel::Warn => tracing::warn!(
            category = category,
            correlation_id = %correlation_id,
            legacy = true,
            "{}",
            message
        ),
        LegacyLogLevel::Info => tracing::info!(
            category = category,
            correlation_id = %correlation_id,
            legacy = true,
            "{}",
            message
        ),
        LegacyLogLevel::Debug => tracing::debug!(
            category = category,
            correlation_id = %correlation_id,
            legacy = true,
            "{}",
            message
        ),
        LegacyLogLevel::Trace => tracing::trace!(
            category = category,
            correlation_id = %correlation_id,
            legacy = true,
            "{}",
            message
        ),
    }
}
/// Add a log entry to the in-memory buffer for UI display
fn add_to_buffer(category: &str, message: &str) {
    if let Some(buffer) = LOG_BUFFER.get() {
        if let Ok(mut buf) = buffer.lock() {
            if buf.len() >= MAX_LOG_LINES {
                buf.pop_front();
            }
            buf.push_back(format!("[{}] {}", category, message));
        }
    }
}
/// Get recent log lines for UI display
pub fn get_recent_logs() -> Vec<String> {
    if let Some(buffer) = LOG_BUFFER.get() {
        if let Ok(buf) = buffer.lock() {
            return buf.iter().cloned().collect();
        }
    }
    Vec::new()
}
/// Get the last N log lines
pub fn get_last_logs(n: usize) -> Vec<String> {
    if let Some(buffer) = LOG_BUFFER.get() {
        if let Ok(buf) = buffer.lock() {
            return buf.iter().rev().take(n).cloned().collect();
        }
    }
    Vec::new()
}
/// Debug-only logging - compiled out in release builds
/// Use for verbose performance/scroll/cache logging
#[cfg(debug_assertions)]
pub fn log_debug(category: &str, message: &str) {
    add_to_buffer(category, message);
    tracing::debug!(category = category, legacy = true, "{}", message);
}
#[cfg(not(debug_assertions))]
pub fn log_debug(_category: &str, _message: &str) {
    // No-op in release builds
}
// =============================================================================
// STRUCTURED LOGGING HELPERS
// These provide typed, structured logging for common operations
// =============================================================================

/// Log a script execution event with structured fields
pub fn log_script_event(script_id: &str, action: &str, duration_ms: Option<u64>, success: bool) {
    add_to_buffer(
        "SCRIPT",
        &format!("{} {} (success={})", action, script_id, success),
    );

    match duration_ms {
        Some(duration) => {
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
        }
        None => {
            tracing::info!(
                event_type = "script_event",
                script_id = script_id,
                action = action,
                success = success,
                "Script {} {}",
                action,
                script_id
            );
        }
    }
}
/// Log a UI event with structured fields
pub fn log_ui_event(component: &str, action: &str, details: Option<&str>) {
    let msg = match details {
        Some(d) => format!("{} {} - {}", component, action, d),
        None => format!("{} {}", component, action),
    };
    add_to_buffer("UI", &msg);

    tracing::info!(
        event_type = "ui_event",
        component = component,
        action = action,
        details = details,
        "{}",
        msg
    );
}
/// Log a keyboard event with structured fields
pub fn log_key_event(key: &str, modifiers: &str, action: &str) {
    add_to_buffer("KEY", &format!("{} {} ({})", action, key, modifiers));

    tracing::debug!(
        event_type = "key_event",
        key = key,
        modifiers = modifiers,
        action = action,
        "Key {} {}",
        action,
        key
    );
}
/// Log a performance metric with structured fields
pub fn log_perf(operation: &str, duration_ms: u64, threshold_ms: u64) {
    let is_slow = duration_ms > threshold_ms;
    let level_marker = if is_slow { "SLOW" } else { "OK" };

    add_to_buffer(
        "PERF",
        &format!("{} {}ms [{}]", operation, duration_ms, level_marker),
    );

    if is_slow {
        tracing::warn!(
            event_type = "performance",
            operation = operation,
            duration_ms = duration_ms,
            threshold_ms = threshold_ms,
            is_slow = true,
            "Slow operation: {} took {}ms (threshold: {}ms)",
            operation,
            duration_ms,
            threshold_ms
        );
    } else {
        tracing::debug!(
            event_type = "performance",
            operation = operation,
            duration_ms = duration_ms,
            threshold_ms = threshold_ms,
            is_slow = false,
            "Operation {} completed in {}ms",
            operation,
            duration_ms
        );
    }
}
/// Log an error with structured fields and context
pub fn log_error(category: &str, error: &str, context: Option<&str>) {
    let msg = match context {
        Some(ctx) => format!("{}: {} (context: {})", category, error, ctx),
        None => format!("{}: {}", category, error),
    };
    add_to_buffer("ERROR", &msg);

    tracing::error!(
        event_type = "error",
        category = category,
        error_message = error,
        context = context,
        "{}",
        msg
    );
}
// =============================================================================
// PAYLOAD TRUNCATION HELPERS
// Purpose: Avoid logging sensitive/large data like base64 screenshots, clipboard
// =============================================================================

/// Maximum length for logged message payloads
const MAX_PAYLOAD_LOG_LEN: usize = 200;
/// Truncate a string for logging, adding "..." suffix if truncated.
/// This function is UTF-8 safe - it will never panic on multi-byte characters.
/// If max_len falls in the middle of a multi-byte character, it backs up to the
/// nearest valid character boundary.
pub fn truncate_for_log(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_owned();
    }
    // Find a valid UTF-8 char boundary at or before max_len
    let mut end = max_len.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...({})", &s[..end], s.len())
}
/// Summarize a JSON payload for logging (type + length, truncated preview)
/// Used for protocol messages to avoid logging full screenshots/clipboard data
pub fn summarize_payload(json: &str) -> String {
    // Try to extract message type from JSON
    let msg_type = json.find("\"type\":\"").and_then(|pos| {
        let start = pos + 8; // length of "\"type\":\""
        json[start..].find('"').map(|end| &json[start..start + end])
    });

    match msg_type {
        Some(t) => format!("{{type:{}, len:{}}}", t, json.len()),
        None => format!("{{len:{}}}", json.len()),
    }
}
/// Log a protocol message being sent to script (truncated for performance/privacy)
pub fn log_protocol_send(fd: i32, json: &str) {
    // In debug/verbose mode, show truncated preview
    // In normal mode, just show type + length
    #[cfg(debug_assertions)]
    {
        let summary = summarize_payload(json);
        add_to_buffer("EXEC", &format!("→stdin fd={}: {}", fd, summary));
        tracing::debug!(
            event_type = "protocol_send",
            fd = fd,
            payload_len = json.len(),
            summary = %summary,
            "Sending to script stdin"
        );
    }

    #[cfg(not(debug_assertions))]
    {
        // Minimal logging in release - just type + length
        let summary = summarize_payload(json);
        tracing::info!(
            event_type = "protocol_send",
            fd = fd,
            payload_len = json.len(),
            "→script: {}",
            summary
        );
    }
}
/// Log a protocol message received from script (truncated for performance/privacy)
pub fn log_protocol_recv(msg_type: &str, json_len: usize) {
    #[cfg(debug_assertions)]
    {
        add_to_buffer(
            "EXEC",
            &format!("←stdout: type={} len={}", msg_type, json_len),
        );
        tracing::debug!(
            event_type = "protocol_recv",
            message_type = msg_type,
            payload_len = json_len,
            "Received from script"
        );
    }

    #[cfg(not(debug_assertions))]
    {
        tracing::info!(
            event_type = "protocol_recv",
            message_type = msg_type,
            payload_len = json_len,
            "←script: type={} len={}",
            msg_type,
            json_len
        );
    }
}
// =============================================================================
// MOUSE HOVER LOGGING
// Category: MOUSE_HOVER
// Purpose: Log mouse enter/leave events on list items for debugging hover/focus behavior
// =============================================================================

/// Log mouse enter event on a list item
pub fn log_mouse_enter(item_index: usize, item_id: Option<&str>) {
    let id_info = item_id.unwrap_or("none");
    add_to_buffer(
        "MOUSE_HOVER",
        &format!("ENTER item_index={} id={}", item_index, id_info),
    );

    tracing::debug!(
        event_type = "mouse_hover",
        action = "enter",
        item_index = item_index,
        item_id = id_info,
        "Mouse enter item {}",
        item_index
    );
}
/// Log mouse leave event on a list item
pub fn log_mouse_leave(item_index: usize, item_id: Option<&str>) {
    let id_info = item_id.unwrap_or("none");
    add_to_buffer(
        "MOUSE_HOVER",
        &format!("LEAVE item_index={} id={}", item_index, id_info),
    );

    tracing::debug!(
        event_type = "mouse_hover",
        action = "leave",
        item_index = item_index,
        item_id = id_info,
        "Mouse leave item {}",
        item_index
    );
}
/// Log mouse hover state change (for tracking focus/highlight transitions)
pub fn log_mouse_hover_state(item_index: usize, is_hovered: bool, is_focused: bool) {
    add_to_buffer(
        "MOUSE_HOVER",
        &format!(
            "STATE item_index={} hovered={} focused={}",
            item_index, is_hovered, is_focused
        ),
    );

    tracing::debug!(
        event_type = "mouse_hover",
        action = "state_change",
        item_index = item_index,
        is_hovered = is_hovered,
        is_focused = is_focused,
        "Hover state: item {} hovered={} focused={}",
        item_index,
        is_hovered,
        is_focused
    );
}
// =============================================================================
// SCROLL STATE LOGGING
// Category: SCROLL_STATE
// Purpose: Log scroll position changes and scroll_to_item calls for debugging jitter
// =============================================================================

/// Log scroll position change
pub fn log_scroll_position(scroll_top: f32, visible_start: usize, visible_end: usize) {
    add_to_buffer(
        "SCROLL_STATE",
        &format!(
            "POSITION scroll_top={:.2} visible_range={}..{}",
            scroll_top, visible_start, visible_end
        ),
    );

    tracing::debug!(
        event_type = "scroll_state",
        action = "position",
        scroll_top = scroll_top,
        visible_start = visible_start,
        visible_end = visible_end,
        "Scroll position: {:.2} (visible {}..{})",
        scroll_top,
        visible_start,
        visible_end
    );
}
/// Log scroll_to_item call
pub fn log_scroll_to_item(target_index: usize, reason: &str) {
    add_to_buffer(
        "SCROLL_STATE",
        &format!("SCROLL_TO_ITEM target={} reason={}", target_index, reason),
    );

    tracing::debug!(
        event_type = "scroll_state",
        action = "scroll_to_item",
        target_index = target_index,
        reason = reason,
        "Scroll to item {} (reason: {})",
        target_index,
        reason
    );
}
/// Log scroll bounds/viewport info
pub fn log_scroll_bounds(viewport_height: f32, content_height: f32, item_count: usize) {
    add_to_buffer(
        "SCROLL_STATE",
        &format!(
            "BOUNDS viewport={:.2} content={:.2} items={}",
            viewport_height, content_height, item_count
        ),
    );

    tracing::debug!(
        event_type = "scroll_state",
        action = "bounds",
        viewport_height = viewport_height,
        content_height = content_height,
        item_count = item_count,
        "Scroll bounds: viewport={:.2} content={:.2} items={}",
        viewport_height,
        content_height,
        item_count
    );
}
/// Log scroll adjustment (when scroll position is programmatically corrected)
pub fn log_scroll_adjustment(from: f32, to: f32, reason: &str) {
    add_to_buffer(
        "SCROLL_STATE",
        &format!("ADJUSTMENT from={:.2} to={:.2} reason={}", from, to, reason),
    );

    tracing::debug!(
        event_type = "scroll_state",
        action = "adjustment",
        from = from,
        to = to,
        reason = reason,
        "Scroll adjustment: {:.2} -> {:.2} ({})",
        from,
        to,
        reason
    );
}
