//! Source-level contract test for the `portal-dictation-roundtrip` user story.
//!
//! The story wants a live end-to-end proof: dictate into the chat, feed a
//! synthetic transcript via `pushDictationResult`, and watch the text land in
//! the chat's input without auto-submit while `dictationStatus` returns to
//! `idle`. The stdin hook covers transcript injection, but a full live run
//! still needs stable `dictationStatus` exposure. Until then, this test pins
//! the production delivery invariants that the story's acceptance criteria
//! depend on.
//!
//! Invariants pinned (routing lives in `mutate_internal_dictation_request`,
//! ordering in `handle_dictation_transcript`, both in
//! `src/app_execute/builtin_execution.rs`):
//!
//! 1. `DictationTarget::TabAiHarness` seeds an open chat's input with
//!    `ChatEntry::seeded`, so the transcript waits for the user's Enter; the
//!    legacy `AiChatComposer` target is canonicalized to it and never delivers
//!    on its own.
//!
//! 2. With no chat open, `TabAiHarness` opens the one chat with
//!    `ChatEntry::sending`, so the dictation becomes the first turn without
//!    inheriting the selected launcher row as context.
//!
//! 3. `record_dictation_history(&transcript, audio_duration, target)` runs
//!    BEFORE the delivery mutation, so a delivery failure never silently drops
//!    the captured audio.
//!
//! 4. `DictationSessionPhase::Idle` remains a valid terminal/inactive state
//!    the session returns to — approximates the story's "dictationStatus
//!    returns to idle" assertion at the type level.

const DELIVERY_SOURCE: &str = include_str!("../src/app_execute/builtin_execution.rs");
const DICTATION_TYPES_SOURCE: &str = include_str!("../src/dictation/types.rs");

fn function_slice(signature: &str) -> &'static str {
    let start = DELIVERY_SOURCE
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} must exist in builtin_execution.rs"));
    let tail = &DELIVERY_SOURCE[start..];
    let end = tail[signature.len()..]
        .find("\n    fn ")
        .map(|offset| signature.len() + offset)
        .unwrap_or(tail.len());
    &tail[..end]
}

fn handler_slice() -> &'static str {
    function_slice("fn handle_dictation_transcript(")
}

/// The delivery match arm for `target`, ending where the next arm begins.
fn delivery_arm(target: &str) -> &'static str {
    let routing = function_slice("fn mutate_internal_dictation_request(");
    let pattern = format!("crate::dictation::DictationTarget::{target} =>");
    let start = routing
        .find(&pattern)
        .unwrap_or_else(|| panic!("delivery must keep an explicit {target} arm"));
    let arm = &routing[start + pattern.len()..];
    let end = arm
        .find("crate::dictation::DictationTarget::")
        .unwrap_or(arm.len());
    &routing[start..start + pattern.len() + end]
}

#[test]
fn chat_delivery_seeds_an_open_chat_without_auto_submit() {
    let legacy = delivery_arm("AiChatComposer");
    assert!(
        legacy.contains("Err(\"Legacy AI Chat destination was not canonicalized\""),
        "the legacy AiChatComposer target must be canonicalized to TabAiHarness, \
         never delivered through its own composer path"
    );
    assert!(
        DICTATION_TYPES_SOURCE
            .contains("DictationTarget::AiChatComposer => DictationTarget::TabAiHarness"),
        "AiChatComposer must canonicalize to the TabAiHarness chat destination"
    );

    let harness = delivery_arm("TabAiHarness");
    let seeded = harness
        .find("if self.one_window.chat().is_some() {\n")
        .and_then(|open| {
            harness[open..]
                .find("ChatEntry::seeded(words)")
                .map(|offset| open + offset)
        });
    let sending = harness.find("ChatEntry::sending(words)");
    assert!(
        matches!((seeded, sending), (Some(seeded), Some(sending)) if seeded < sending),
        "dictation into an open chat must seed its input (the user still presses \
         Enter); only a new chat takes the transcript as its first turn"
    );
}

#[test]
fn tab_ai_harness_delivery_opens_agent_chat_with_transcript_entry_intent() {
    let harness = delivery_arm("TabAiHarness");
    assert!(
        harness.contains("ChatEntry::sending(words)")
            && harness.contains("self.one_window_open_chat(entry, cx)")
            && !harness.contains(".with("),
        "TabAiHarness delivery must use the explicit Send entry verb so Agent Chat \
         auto-submits the dictated prompt exactly once without \
         inheriting the selected launcher row as context"
    );
}

#[test]
fn day_page_today_delivery_appends_capture_without_opening_ui() {
    let day_page = delivery_arm("DayPageToday");
    let append = function_slice("fn append_dictation_to_captured_day(");
    assert!(
        day_page.contains("append_dictation_to_captured_day(")
            && append.contains("append_to_captured_day(")
            && append.contains("DayEntry::Capture"),
        "DayPageToday delivery must append a timestamped capture through the \
         brain substrate (lock-safe, self-indexing) rather than opening any UI"
    );
    assert!(
        !day_page.contains("one_window_open") && !day_page.contains("open_note"),
        "DayPageToday delivery must not open a view"
    );
}

#[test]
fn quick_ai_delivery_submit_flag_comes_from_config() {
    let quick_ai = delivery_arm("QuickAiQuestion");
    assert!(
        quick_ai.contains("self.one_window_open_chat("),
        "QuickAiQuestion delivery must open the one chat under the main input"
    );
    assert!(
        quick_ai.contains("quick_ai_answers()"),
        "QuickAiQuestion delivery must derive its submit flag from \
         dictation.quickAi via quick_ai_answers() — hardcoding submit would \
         break the 'composer' config mode"
    );
}

#[test]
fn history_is_recorded_before_delivery_routing() {
    let handler = handler_slice();
    // Match the call site by name — rustfmt reflows the argument list, and
    // the invariant is the ordering, not the call's line layout.
    let history_pos = handler
        .find("record_dictation_history(")
        .expect("record_dictation_history call must be present in the handler");
    let dispatch_pos = handler
        .find("self.mutate_internal_dictation_request(&request")
        .expect("delivery mutation must be present in the handler");
    assert!(
        history_pos < dispatch_pos,
        "record_dictation_history must run BEFORE the delivery mutation so a \
         delivery failure never silently drops the captured audio — found \
         history at {history_pos} and dispatch at {dispatch_pos}"
    );
}

#[test]
fn dictation_session_phase_idle_is_a_valid_inactive_state() {
    assert!(
        DICTATION_TYPES_SOURCE.contains("pub enum DictationSessionPhase {")
            && DICTATION_TYPES_SOURCE.contains("    Idle,"),
        "DictationSessionPhase::Idle must remain a variant of the session \
         phase enum — that is the type-level anchor for the story's \
         'dictationStatus returns to idle' assertion. If the phase enum \
         ever loses Idle, the story's acceptance becomes structurally \
         unverifiable."
    );
    assert!(
        DICTATION_TYPES_SOURCE.contains("Finished,"),
        "DictationSessionPhase::Finished must remain — a session moves \
         from Recording → Transcribing → Finished, then the cleanup \
         schedule returns the runtime to Idle."
    );
}
