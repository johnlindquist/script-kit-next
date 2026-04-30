//! Source-level contract pinning the 6-site symmetry of
//! `ensure_embedded_ai_window` calls inside `src/app_impl/tab_ai_mode/mod.rs`.
//!
//! Background: the embedded-AI automation registry entry is maintained
//! by exactly one helper, `crate::windows::ensure_embedded_ai_window`.
//! Inside `tab_ai_mode.rs` it is called from exactly six sites:
//!   - 4 × `ensure_embedded_ai_window(true)` — one after each
//!     `self.current_view = AppView::AcpChatView { ... }` assignment
//!     (reuse path, setup-error path, not-ready path, full-launch path).
//!   - 2 × `ensure_embedded_ai_window(false)` — one inside
//!     `close_acp_chat_to_script_list`, and one inside the normal
//!     embedded ACP close helper used by Escape/Cmd+W/native close.
//!
//! The 4 hide-flow exits (in `src/main_sections/window_visibility.rs`
//! and the three stdin `ExternalCommand::Hide` arms) are already pinned
//! by `tests/hide_path_embedded_ai_registry_teardown_contract.rs`. This
//! file defends the OTHER half of the lock-step: the 4 entry upserts
//! and the 2 close-flow teardowns that all live in `tab_ai_mode.rs`.
//!
//! **Refactor threat**: the four `self.current_view = AppView::AcpChatView { ... }`
//! entry sites are labelled in `lat.md/acp-chat.md` as "reuse path,
//! setup-error path, not-ready path, full-launch path" — a plausible
//! consolidation. A contributor extracting a shared `enter_acp_chat_view`
//! helper could easily drop the paired `ensure_embedded_ai_window(true)`
//! call during extraction, silently regressing the embedded AI registry
//! entry on that entry path so `listAutomationWindows` no longer reports
//! the `ai` child window after tab-ai launch. Same threat on the exit
//! side: moving either `false` call out of its ACP close helper would
//! likely lose the pairing with its main-surface re-key, repeating the
//! Pass #20 bug class on a close-flow path.
//!
//! This contract catches both edits at test time:
//!   1. exact count of `(true)` calls == 4
//!   2. exact count of `(false)` calls == 2
//!   3. every `(true)` call is preceded within 200 bytes by
//!      `self.current_view = AppView::AcpChatView {`
//!   4. every `(true)` call is followed by the main-window semantic re-key
//!   5. the `(false)` calls live inside both ACP close bodies.

const SRC: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn exactly_four_ensure_embedded_ai_window_true_calls() {
    let count = SRC
        .matches("crate::windows::ensure_embedded_ai_window(true)")
        .count();
    assert_eq!(
        count, 4,
        "`src/app_impl/tab_ai_mode/mod.rs` MUST contain exactly 4 \
         `ensure_embedded_ai_window(true)` call sites (one after each \
         of the four `self.current_view = AppView::AcpChatView {{ ... }}` \
         entry paths — reuse path, setup-error path, not-ready path, \
         full-launch path). Found {count}. A change in this count \
         means either (a) a new entry path was added without the \
         paired upsert — `listAutomationWindows` will not report the \
         `ai` child window for that path; or (b) an entry path was \
         removed/merged without cleaning up the call — dead code that \
         may upsert a stale entry before any view flip. Either way, \
         the refactor must be reviewed against the 6-site invariant \
         documented at `lat.md/acp-chat.md#Embedded AI subview`."
    );
}

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn exactly_two_ensure_embedded_ai_window_false_calls() {
    let count = SRC
        .matches("crate::windows::ensure_embedded_ai_window(false)")
        .count();
    assert_eq!(
        count, 2,
        "`src/app_impl/tab_ai_mode/mod.rs` MUST contain exactly 2 \
         `ensure_embedded_ai_window(false)` calls: one forced ScriptList \
         detach/close path and one normal embedded ACP return-origin \
         close path. Found {count}."
    );
}

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn every_true_call_is_preceded_by_acpchatview_assignment() {
    let true_positions: Vec<usize> = SRC
        .match_indices("crate::windows::ensure_embedded_ai_window(true)")
        .map(|(idx, _)| idx)
        .collect();
    assert_eq!(
        true_positions.len(),
        4,
        "Guard test expects 4 `(true)` call sites (see \
         exactly_four_ensure_embedded_ai_window_true_calls)."
    );
    for call_idx in &true_positions {
        let window_start = call_idx.saturating_sub(200);
        let window = &SRC[window_start..*call_idx];
        assert!(
            window.contains("self.current_view = AppView::AcpChatView {"),
            "`ensure_embedded_ai_window(true)` call at offset {call_idx} \
             is not preceded within 200 bytes by \
             `self.current_view = AppView::AcpChatView {{`. The upsert \
             must follow the view flip so the registry entry shape \
             matches the active subview. Preceding context:\n{window}"
        );
    }
}

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn every_true_call_rekeys_main_window_to_active_surface() {
    let true_positions: Vec<usize> = SRC
        .match_indices("crate::windows::ensure_embedded_ai_window(true)")
        .map(|(idx, _)| idx)
        .collect();
    assert_eq!(
        true_positions.len(),
        4,
        "Guard test expects 4 `(true)` call sites."
    );
    for call_idx in &true_positions {
        let after = &SRC[*call_idx..SRC.len().min(*call_idx + 260)];
        assert!(
            after.contains("crate::windows::update_automation_semantic_surface(")
                && after.contains("crate::semantic_surface_for_main_view(&self.current_view)"),
            "`ensure_embedded_ai_window(true)` call at offset {call_idx} \
             must immediately re-key main's semantic surface from scriptList \
             to acpChat. Following context:\n{after}"
        );
    }
}

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn false_calls_live_inside_both_acp_close_bodies() {
    for header in [
        "fn close_tab_ai_harness_terminal_impl",
        "pub(crate) fn close_acp_chat_to_script_list",
    ] {
        let header_idx = SRC.find(header).expect("ACP close function must exist");
        let after_header = &SRC[header_idx..];
        let false_rel = after_header
            .find("crate::windows::ensure_embedded_ai_window(false)")
            .expect("each ACP close function must tear down the embedded AI registry entry");
        let false_idx = header_idx + false_rel;
        let between = &SRC[header_idx..false_idx];
        let intruder_patterns = ["\n    pub fn ", "\n    pub(crate) fn ", "\n    fn "];
        for pat in &intruder_patterns {
            assert!(
                !between[header.len()..].contains(pat),
                "A function boundary (`{pat}`) appears between the \
                 `{header}` header and the `ensure_embedded_ai_window(false)` \
                 call — the teardown has moved out of that close body. \
                 Between (truncated):\n{snippet}",
                snippet = &between[..between.len().min(400)]
            );
        }
    }
}
