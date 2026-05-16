//! Source-level contract pinning embedded ACP entry/exit symmetry.
//!
//! Background: the embedded-AI automation registry entry is maintained
//! by exactly one helper, `crate::windows::ensure_embedded_ai_window`.
//! ACP entry now routes through exactly one app-level owner:
//!   - 1 × `ensure_embedded_ai_window(true)` inside
//!     `enter_embedded_acp_chat_surface`, after
//!     `self.current_view = AppView::AcpChatView { ... }` and before
//!     the main-window re-key.
//!   - 3 × `enter_embedded_acp_chat_surface(...)` callers — setup,
//!     reuse, and full-launch.
//!   - 2 × `ensure_embedded_ai_window(false)` — one inside
//!     `close_acp_chat_to_script_list`, and one inside the normal
//!     embedded ACP close helper used by Escape/Cmd+W/native close.
//!
//! The 4 hide-flow exits (in `src/main_sections/window_visibility.rs`
//! and the three stdin `ExternalCommand::Hide` arms) are already pinned
//! by `tests/hide_path_embedded_ai_registry_teardown_contract.rs`. This
//! file defends the OTHER half of the lock-step: the shared entry upsert
//! and the 2 close-flow teardowns.
//!
//! **Refactor threat**: the `self.current_view = AppView::AcpChatView { ... }`
//! entry sites are labelled in `removed-docs` as setup helper,
//! reuse path, and full-launch path — a plausible
//! consolidation. A contributor extracting a shared `enter_acp_chat_view`
//! helper could easily drop the paired `ensure_embedded_ai_window(true)`
//! call during extraction, silently regressing the embedded AI registry
//! entry on that entry path so `listAutomationWindows` no longer reports
//! the `ai` child window after tab-ai launch. Same threat on the exit
//! side: moving either `false` call out of its ACP close helper would
//! likely lose the pairing with its main-surface re-key, repeating the
//! Pass #20 bug class on a close-flow path.
//!
//! This contract catches these edits at test time:
//!   1. exact count of production `(true)` calls == 1
//!   2. exact count of `(false)` calls == 2
//!   3. the production `(true)` call lives inside the shared entry owner
//!   4. the shared entry owner preserves view/upsert/re-key/surface/focus order
//!   5. the 3 ACP entry paths call the shared owner
//!   6. the `(false)` calls live inside both ACP close bodies.

const SRC: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const ACP_SETUP_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_setup.rs");
const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const ACP_SURFACE_TRANSITIONS_SOURCE: &str =
    include_str!("../src/app_impl/acp_surface_transitions.rs");

fn production_sources() -> [(&'static str, &'static str); 3] {
    [
        ("tab_ai_mode/mod.rs", SRC),
        ("tab_ai_mode/acp_setup.rs", ACP_SETUP_SOURCE),
        ("tab_ai_mode/acp_launch.rs", ACP_LAUNCH_SOURCE),
    ]
}

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_start = &source[start..];
    let open = after_start
        .find('{')
        .unwrap_or_else(|| panic!("missing function body for: {signature}"));
    let mut depth = 0usize;
    for (offset, ch) in after_start[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after_start[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn exactly_one_ensure_embedded_ai_window_true_call() {
    let count: usize = production_sources()
        .iter()
        .map(|(_, source)| {
            source
                .matches("crate::windows::ensure_embedded_ai_window(true)")
                .count()
        })
        .sum::<usize>()
        + ACP_SURFACE_TRANSITIONS_SOURCE
            .matches("crate::windows::ensure_embedded_ai_window(true)")
            .count();
    assert_eq!(
        count, 1,
        "embedded ACP entry MUST contain exactly 1 production \
         `ensure_embedded_ai_window(true)` call, inside \
         `enter_embedded_acp_chat_surface`. Found {count}."
    );
}

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
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

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn shared_entry_owner_preserves_transition_order() {
    let body = function_body(
        ACP_SURFACE_TRANSITIONS_SOURCE,
        "pub(crate) fn enter_embedded_acp_chat_surface",
    );
    let ordered_patterns = [
        "self.current_view = AppView::AcpChatView { entity };",
        "crate::windows::ensure_embedded_ai_window(true);",
        "self.rekey_main_automation_surface_from_current_view();",
        "self.transition_acp_surface(AcpSurfaceEvent::EmbeddedOpened);",
        "self.focused_input = FocusedInput::None;",
        "self.clear_actions_popup_state();",
        "self.pending_focus = Some(FocusTarget::ChatPrompt);",
    ];
    let mut last = 0usize;
    for pattern in ordered_patterns {
        let idx = body[last..]
            .find(pattern)
            .unwrap_or_else(|| panic!("missing ordered ACP entry step: {pattern}\n{body}"));
        last += idx + pattern.len();
    }
}

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn entry_paths_delegate_to_shared_owner() {
    let count: usize = production_sources()
        .iter()
        .map(|(_, source)| {
            source
                .matches("self.enter_embedded_acp_chat_surface(")
                .count()
        })
        .sum();
    assert_eq!(
        count, 3,
        "setup, reuse, and full-launch ACP entry paths must delegate to \
         `enter_embedded_acp_chat_surface`. Found {count} callers."
    );
    for (name, source) in production_sources() {
        assert!(
            !source.contains("crate::windows::ensure_embedded_ai_window(true)"),
            "{name} must not split embedded ACP view assignment from the shared entry owner"
        );
    }
    for (source, signature) in [
        (ACP_SETUP_SOURCE, "fn show_embedded_acp_setup_view"),
        (
            ACP_LAUNCH_SOURCE,
            "pub(super) fn open_tab_ai_acp_view_from_request_impl",
        ),
        (SRC, "fn try_reuse_embedded_acp_view"),
    ] {
        let body = function_body(source, signature);
        assert!(
            !body.contains("self.current_view = AppView::AcpChatView {"),
            "{signature} must not assign AcpChatView directly; use enter_embedded_acp_chat_surface"
        );
    }
}

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
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
