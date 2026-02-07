// =============================================================================
// SCROLL PERFORMANCE LOGGING
// Category: SCROLL_PERF
// Purpose: Log timing information for scroll operations to detect jitter sources
// =============================================================================

/// Log scroll operation timing - returns start timestamp
pub fn log_scroll_perf_start(operation: &str) -> u128 {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);

    #[cfg(debug_assertions)]
    {
        add_to_buffer("SCROLL_PERF", &format!("START {} at={}", operation, start));
        tracing::trace!(
            event_type = "scroll_perf",
            action = "start",
            operation = operation,
            start_micros = start,
            "Scroll perf start: {}",
            operation
        );
    }

    start
}
/// Log scroll operation completion with duration
pub fn log_scroll_perf_end(operation: &str, start_micros: u128) {
    let end = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    let duration = end.saturating_sub(start_micros);

    #[cfg(debug_assertions)]
    {
        add_to_buffer(
            "SCROLL_PERF",
            &format!("END {} duration_us={}", operation, duration),
        );
        tracing::trace!(
            event_type = "scroll_perf",
            action = "end",
            operation = operation,
            duration_us = duration,
            "Scroll perf end: {} ({}us)",
            operation,
            duration
        );
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (operation, duration); // Silence unused warnings
    }
}
/// Log scroll frame timing (for detecting dropped frames)
pub fn log_scroll_frame(frame_time_ms: f32, expected_frame_ms: f32) {
    let is_slow = frame_time_ms > expected_frame_ms * 1.5;

    #[cfg(debug_assertions)]
    {
        let marker = if is_slow { " [SLOW]" } else { "" };
        add_to_buffer(
            "SCROLL_PERF",
            &format!(
                "FRAME time={:.2}ms expected={:.2}ms{}",
                frame_time_ms, expected_frame_ms, marker
            ),
        );

        if is_slow {
            tracing::warn!(
                event_type = "scroll_perf",
                action = "frame",
                frame_time_ms = frame_time_ms,
                expected_frame_ms = expected_frame_ms,
                is_slow = true,
                "Slow frame: {:.2}ms (expected {:.2}ms)",
                frame_time_ms,
                expected_frame_ms
            );
        } else {
            tracing::trace!(
                event_type = "scroll_perf",
                action = "frame",
                frame_time_ms = frame_time_ms,
                expected_frame_ms = expected_frame_ms,
                is_slow = false,
                "Frame: {:.2}ms",
                frame_time_ms
            );
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (frame_time_ms, expected_frame_ms, is_slow);
    }
}
/// Log scroll event rate (for detecting rapid scroll input)
pub fn log_scroll_event_rate(events_per_second: f32) {
    let is_rapid = events_per_second > 60.0;

    #[cfg(debug_assertions)]
    {
        let marker = if is_rapid { " [RAPID]" } else { "" };
        add_to_buffer(
            "SCROLL_PERF",
            &format!("EVENT_RATE eps={:.1}{}", events_per_second, marker),
        );

        if is_rapid {
            tracing::debug!(
                event_type = "scroll_perf",
                action = "event_rate",
                events_per_second = events_per_second,
                is_rapid = true,
                "Rapid scroll events: {:.1}/s",
                events_per_second
            );
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (events_per_second, is_rapid);
    }
}
// =============================================================================
// KEY EVENT & SCROLL QUEUE METRICS
// Category: SCROLL_PERF
// Purpose: Track input rates, frame gaps, queue depth, and render stalls
// =============================================================================

/// Log keyboard event rate (events per second) for detecting fast key repeat
pub fn log_key_event_rate(events_per_sec: f64) {
    let is_fast = events_per_sec > 30.0;
    let is_very_fast = events_per_sec > 60.0;

    #[cfg(debug_assertions)]
    {
        let marker = if is_very_fast {
            " [VERY_FAST]"
        } else if is_fast {
            " [FAST]"
        } else {
            ""
        };
        add_to_buffer(
            "SCROLL_PERF",
            &format!("KEY_EVENT_RATE eps={:.1}{}", events_per_sec, marker),
        );

        tracing::debug!(
            event_type = "scroll_perf",
            action = "key_event_rate",
            events_per_sec = events_per_sec,
            is_fast = is_fast,
            is_very_fast = is_very_fast,
            "Key event rate: {:.1}/s",
            events_per_sec
        );
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (events_per_sec, is_fast, is_very_fast);
    }
}
/// Log frame timing gap (when frames take longer than expected)
pub fn log_frame_gap(gap_ms: u64) {
    let is_significant = gap_ms > 16;
    let is_severe = gap_ms > 100;

    #[cfg(debug_assertions)]
    {
        let marker = if is_severe {
            " [SEVERE]"
        } else if is_significant {
            " [SLOW]"
        } else {
            ""
        };
        add_to_buffer(
            "SCROLL_PERF",
            &format!("FRAME_GAP gap_ms={}{}", gap_ms, marker),
        );

        if is_severe {
            tracing::warn!(
                event_type = "scroll_perf",
                action = "frame_gap",
                gap_ms = gap_ms,
                is_severe = true,
                "Severe frame gap: {}ms",
                gap_ms
            );
        } else if is_significant {
            tracing::debug!(
                event_type = "scroll_perf",
                action = "frame_gap",
                gap_ms = gap_ms,
                is_significant = true,
                "Frame gap: {}ms",
                gap_ms
            );
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (gap_ms, is_significant, is_severe);
    }
}
/// Log scroll queue depth (number of pending scroll operations)
pub fn log_scroll_queue_depth(depth: usize) {
    let is_backed_up = depth > 5;
    let is_critical = depth > 20;

    #[cfg(debug_assertions)]
    {
        let marker = if is_critical {
            " [CRITICAL]"
        } else if is_backed_up {
            " [BACKED_UP]"
        } else {
            ""
        };
        add_to_buffer(
            "SCROLL_PERF",
            &format!("QUEUE_DEPTH depth={}{}", depth, marker),
        );

        if is_critical {
            tracing::warn!(
                event_type = "scroll_perf",
                action = "queue_depth",
                depth = depth,
                is_critical = true,
                "Critical queue depth: {}",
                depth
            );
        } else if is_backed_up {
            tracing::debug!(
                event_type = "scroll_perf",
                action = "queue_depth",
                depth = depth,
                is_backed_up = true,
                "Queue backed up: {}",
                depth
            );
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (depth, is_backed_up, is_critical);
    }
}
/// Log render stall (when render blocks for too long)
pub fn log_render_stall(duration_ms: u64) {
    let is_stall = duration_ms > 16;
    let is_hang = duration_ms > 100;

    let marker = if is_hang {
        " [HANG]"
    } else if is_stall {
        " [STALL]"
    } else {
        ""
    };
    add_to_buffer(
        "SCROLL_PERF",
        &format!("RENDER_STALL duration_ms={}{}", duration_ms, marker),
    );

    if is_hang {
        tracing::error!(
            event_type = "scroll_perf",
            action = "render_stall",
            duration_ms = duration_ms,
            is_hang = true,
            "Render hang: {}ms",
            duration_ms
        );
    } else if is_stall {
        tracing::warn!(
            event_type = "scroll_perf",
            action = "render_stall",
            duration_ms = duration_ms,
            is_stall = true,
            "Render stall: {}ms",
            duration_ms
        );
    }
}
/// Log scroll operation batch (when multiple scroll events are coalesced)
pub fn log_scroll_batch(batch_size: usize, coalesced_from: usize) {
    if coalesced_from > batch_size {
        #[cfg(debug_assertions)]
        {
            add_to_buffer(
                "SCROLL_PERF",
                &format!(
                    "BATCH_COALESCE processed={} from={}",
                    batch_size, coalesced_from
                ),
            );

            tracing::debug!(
                event_type = "scroll_perf",
                action = "batch_coalesce",
                batch_size = batch_size,
                coalesced_from = coalesced_from,
                "Coalesced {} scroll events to {}",
                coalesced_from,
                batch_size
            );
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = (batch_size, coalesced_from);
        }
    }
}
/// Log key repeat timing for debugging fast scroll issues
pub fn log_key_repeat_timing(key: &str, interval_ms: u64, repeat_count: u32) {
    let is_fast = interval_ms < 50;

    #[cfg(debug_assertions)]
    {
        let marker = if is_fast { " [FAST_REPEAT]" } else { "" };
        add_to_buffer(
            "SCROLL_PERF",
            &format!(
                "KEY_REPEAT key={} interval_ms={} count={}{}",
                key, interval_ms, repeat_count, marker
            ),
        );

        tracing::debug!(
            event_type = "scroll_perf",
            action = "key_repeat",
            key = key,
            interval_ms = interval_ms,
            repeat_count = repeat_count,
            is_fast = is_fast,
            "Key repeat: {} interval={}ms count={}",
            key,
            interval_ms,
            repeat_count
        );
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = (key, interval_ms, repeat_count, is_fast);
    }
}
// =============================================================================
// CONVENIENCE MACROS (re-exported)
// =============================================================================

// Re-export tracing for use by other modules
// Example usage:
//   use crate::logging;
//   logging::info!(event_type = "action", "Something happened");
//
// Or import tracing directly:
//   use tracing::{info, error, warn, debug, trace};
pub use tracing;
