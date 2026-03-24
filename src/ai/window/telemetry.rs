use super::types::AiWindowMode;

/// Emit a structured lifecycle event for AI window open/close/mode transitions.
///
/// All lifecycle events share one schema so agents and log parsers can query
/// the full open→mode→close arc with a single filter.
pub(super) fn log_ai_lifecycle(
    event: &'static str,
    window_mode: AiWindowMode,
    source: &'static str,
    status: &'static str,
) {
    tracing::info!(
        target: "ai",
        category = "AI",
        event,
        window_mode = ?window_mode,
        source,
        status,
        "ai_lifecycle"
    );
}

/// Emit a structured UI interaction event (button clicks, overlay toggles, etc.).
pub(super) fn log_ai_ui(event: &'static str, window_mode: AiWindowMode, source: &'static str) {
    tracing::info!(
        target: "ai",
        category = "AI_UI",
        event,
        window_mode = ?window_mode,
        source,
        "ai_ui"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_ai_lifecycle_does_not_panic_for_both_modes() {
        log_ai_lifecycle("test_event", AiWindowMode::Full, "test", "ok");
        log_ai_lifecycle("test_event", AiWindowMode::Mini, "test", "ok");
    }

    #[test]
    fn log_ai_ui_does_not_panic_for_both_modes() {
        log_ai_ui("test_event", AiWindowMode::Full, "test");
        log_ai_ui("test_event", AiWindowMode::Mini, "test");
    }
}
