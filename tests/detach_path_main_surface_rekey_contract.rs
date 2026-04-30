//! Source-level contract pinning the detach-path main-surface re-key invariant.
//!
//! Pass #49 of the AFK audit (Run 2) live-verified Pass #29's detached-popup
//! close dispatch but surfaced a drift: after `triggerAction acp_detach_window
//! host=main`, `listAutomationWindows[0].semanticSurface` stayed `"acpChat"`
//! even though `getState` reported the view was back on ScriptList. The
//! automation registry's surface tag was NOT re-keyed at detach time — only
//! on the subsequent `hide` trigger.
//!
//! Pass #50 fixes this by adding an `update_automation_semantic_surface("main",
//! Some("scriptList".to_string()))` call to `close_acp_chat_to_script_list` in
//! `src/app_impl/tab_ai_mode/mod.rs`, right after the view flip and before the
//! `acp_chat_restored_to_script_list` tracing emit. The call mirrors the
//! existing hide-path re-key in `src/main_sections/window_visibility.rs:397`,
//! which calls the same helper after `reset_to_script_list` for the same
//! reason.
//!
//! A future refactor that extracted the view-flip into a helper but forgot
//! the re-key would silently reintroduce the drift. This contract test pins
//! the re-key call to `close_acp_chat_to_script_list`'s body so the invariant
//! can't drift silently.

const TAB_AI_MODE_RS: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");

fn close_acp_chat_to_script_list_body(src: &str) -> &str {
    let start_marker = "pub(crate) fn close_acp_chat_to_script_list(";
    let start = src.find(start_marker).unwrap_or_else(|| {
        panic!(
            "src/app_impl/tab_ai_mode/mod.rs must define `pub(crate) fn close_acp_chat_to_script_list`. \
             Did you rename or remove it? The function is the canonical entry point for the \
             ACP-chat-detach-to-script-list transition; renaming it without updating this contract \
             breaks the surface-rekey pin."
        )
    });
    // Find the next `pub fn`, `fn`, or `pub(crate) fn` at the same indentation
    // after the function header. Conservative: any top-level-ish function break.
    let search_from = start + start_marker.len();
    let next_item_offset = ["\n    pub(crate) fn ", "\n    pub fn ", "\n    fn "]
        .iter()
        .filter_map(|m| src[search_from..].find(m))
        .min()
        .unwrap_or(src.len() - search_from);
    &src[start..search_from + next_item_offset]
}

#[test]
fn close_acp_chat_to_script_list_rekeys_main_surface_to_scriptlist() {
    let body = close_acp_chat_to_script_list_body(TAB_AI_MODE_RS);
    assert!(
        body.contains(
            "update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()))"
        ),
        "src/app_impl/tab_ai_mode/mod.rs `close_acp_chat_to_script_list` must call \
         `update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()))` \
         after the view flip to `AppView::ScriptList`. Without this call, \
         `listAutomationWindows[0].semanticSurface` stays `\"acpChat\"` after \
         detach until the next `hide` or subview flip re-keys it, causing a \
         false state report to any automation consumer that queries the \
         registry immediately after detach. The hide path in \
         `src/main_sections/window_visibility.rs:397` uses the same helper \
         for the same reason; `close_acp_chat_to_script_list` is the detach-path \
         sibling. Body searched (first 400 chars): {:?}",
        &body.chars().take(400).collect::<String>()
    );
}

#[test]
fn rekey_call_appears_before_acp_chat_restored_tracing_event() {
    // The re-key must happen in lockstep with the view flip, BEFORE the
    // `acp_chat_restored_to_script_list` tracing event fires, so any
    // downstream observer of that event (test harness, telemetry consumer)
    // sees a consistent registry snapshot when it queries back.
    let body = close_acp_chat_to_script_list_body(TAB_AI_MODE_RS);

    let rekey_pos = body
        .find("update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()))")
        .unwrap_or_else(|| {
            panic!(
                "Re-key call missing from `close_acp_chat_to_script_list` body; \
                 the companion test should have failed first. If you're seeing \
                 this panic, the prior test was skipped — investigate."
            )
        });

    let trace_event_pos = body
        .find("event = \"acp_chat_restored_to_script_list\"")
        .unwrap_or_else(|| {
            panic!(
                "`close_acp_chat_to_script_list` must emit the \
                 `acp_chat_restored_to_script_list` tracing event; did you \
                 rename or remove it?"
            )
        });

    assert!(
        rekey_pos < trace_event_pos,
        "`update_automation_semantic_surface(\"main\", ...)` must be called \
         BEFORE the `acp_chat_restored_to_script_list` tracing event fires — \
         found re-key at body byte {}, trace event at body byte {}. If the \
         re-key fires after the trace event, a race-sensitive observer that \
         snapshots the registry on receiving the event can still see the \
         stale `\"acpChat\"` tag.",
        rekey_pos,
        trace_event_pos
    );
}

#[test]
fn rekey_call_appears_after_current_view_is_set_to_scriptlist() {
    // Sanity check: the re-key must follow the view flip, not precede it,
    // so the registry tag matches the view state the moment the function
    // returns.
    let body = close_acp_chat_to_script_list_body(TAB_AI_MODE_RS);

    let view_flip_pos = body
        .find("self.current_view = AppView::ScriptList;")
        .unwrap_or_else(|| {
            panic!(
                "`close_acp_chat_to_script_list` must flip `self.current_view` \
                 to `AppView::ScriptList`; did you change the assignment form?"
            )
        });

    let rekey_pos = body
        .find("update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()))")
        .expect("Re-key missing; first test should have caught this");

    assert!(
        view_flip_pos < rekey_pos,
        "`update_automation_semantic_surface(\"main\", ...)` must be called \
         AFTER `self.current_view = AppView::ScriptList;` so the registry \
         tag matches the new view state. Found view flip at body byte {}, \
         re-key at body byte {}. If the re-key fires before the view flip, \
         a concurrent automation query could observe the tag-view pair in \
         an inconsistent intermediate state.",
        view_flip_pos,
        rekey_pos
    );
}
