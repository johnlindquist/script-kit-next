//! Source-level contract tests for the repo-local Codex Stop hook.
//!
//! Codex Stop continuations can keep the same `turn_id` across multiple
//! generated continuation prompts. The marketing image run depends on each
//! Stop event being able to block again until a pause marker appears.

const STOP_HOOK: &str = include_str!("../.codex/hooks/stop-continue-agentic-testing.ts");

#[test]
fn marketing_stop_run_is_supported() {
    assert!(
        STOP_HOOK.contains("\"marketing-infographics\""),
        "the Stop hook must keep the marketing infographic run kind wired"
    );
    assert!(
        STOP_HOOK.contains("buildMarketingPrompt("),
        "the Stop hook must build a dedicated marketing continuation prompt"
    );
    assert!(
        STOP_HOOK.contains("MARKETING_STYLE_DIRECTIONS"),
        "marketing continuations must rotate through varied visual directions"
    );
}

#[test]
fn repeated_turn_id_does_not_stop_continuation_chain() {
    assert!(
        !STOP_HOOK.contains("state.lastTurnId && payload.turn_id")
            && !STOP_HOOK.contains("state.lastTurnId === payload.turn_id"),
        "Stop continuations can reuse the active turn_id. A repeated-turn guard \
         makes the chain stop after the first generated image instead of \
         continuing until the pause marker is set."
    );
    assert!(
        STOP_HOOK.contains("state.lastTurnId = payload.turn_id"),
        "the hook may still record the last turn id for diagnostics"
    );
}

#[test]
fn pause_marker_is_the_manual_stop_contract() {
    assert!(
        STOP_HOOK.contains("PAUSE_MARKER = \"[stop-hook:pause]\""),
        "the manual stop contract must remain explicit and grepable"
    );
    assert!(
        STOP_HOOK.contains("shouldPauseFromMessage(payload)")
            && STOP_HOOK.contains("existsSync(pausePath)"),
        "the hook must stop from either assistant pause text or the pause file"
    );
    assert!(
        STOP_HOOK.contains(".some((line) => line.trim() === PAUSE_MARKER)")
            && !STOP_HOOK.contains("last_assistant_message?.includes(PAUSE_MARKER)"),
        "mentions of the marker in prose or backticks must not accidentally \
         pause the run; the marker must be alone on a line"
    );
    assert!(
        STOP_HOOK.contains("if (!DRY_RUN) {\n    writeFileSync(pausePath"),
        "dry-run hook verification must not create a pause file"
    );
}

#[test]
fn selected_stop_hook_events_are_logged() {
    assert!(
        STOP_HOOK.contains("defaultEventLogPath(")
            && STOP_HOOK.contains(".events.jsonl")
            && STOP_HOOK.contains("logHookEvent("),
        "the hook must maintain a per-run event ledger for Stop-hook evidence"
    );
    assert!(
        STOP_HOOK.contains("reason: \"block_with_continuation_prompt\"")
            && STOP_HOOK.contains("reason: \"pause_file_exists\"")
            && STOP_HOOK.contains("reason: \"assistant_pause_marker\""),
        "the event ledger must distinguish continue, pause-file no-op, and \
         assistant-marker pause outcomes"
    );
}
