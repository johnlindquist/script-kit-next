//! Source-level contract test for the Run 2 Pass #29
//! `tool-trigger-action-agentchatdetached-host` user story.
//!
//! Pass #28 proved the `detached-agent_chat-roundtrip` story 3/3, but observed
//! a carry-forward gap: once a detached Agent Chat window exists, automation
//! had no way to close or otherwise route an action to it via
//! `ExternalCommand::TriggerAction { host: "agentChatDetached", action_id:
//! "agent_chat_close", .. }` — the host parser in
//! `src/main_entry/app_run_setup.rs` only accepted `"agentChatChat"` /
//! `"agent_chatHistory"`, and even if the parse had succeeded the
//! `execute_action_for_actions_host` router delegated AgentChat / detached
//! work to `handle_action` whose `agent_chat_close` arm closes the TabAI
//! harness (`src/app_actions/handle_action/mod.rs:1509`), NOT the
//! detached chat window.
//!
//! Pass #29 closes the tool gap by:
//!
//! 1. Adding `ActionsDialogHost::AgentChatDetached` to the enum in
//!    `src/main_sections/app_view_state.rs` (and covering it in every
//!    exhaustive match on the enum).
//! 2. Accepting `Some("agentChatDetached") => Some(ActionsDialogHost::AgentChatDetached)`
//!    in the TriggerAction host parser in
//!    `src/main_entry/app_run_setup.rs`.
//! 3. Routing `ActionsDialogHost::AgentChatDetached` in
//!    `src/app_impl/actions_dialog.rs::execute_action_for_actions_host`
//!    through the new public helper
//!    `crate::ai::agent_chat::ui::chat_window::dispatch_action_to_detached`, which
//!    hands the action id to the detached window's own
//!    `dispatch_detached_action_checked` — the same dispatcher reached
//!    when the user clicks the action inside the detached popup.
//!
//! This contract test pins all three edits so a mechanical refactor of
//! the enum, the parser, or the router cannot silently drop the
//! AgentChatDetached path and regress the detached-cleanup story.

const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const CHAT_WINDOW: &str = include_str!("../src/ai/agent_chat/ui/chat_window.rs");

#[test]
fn actions_dialog_host_enum_defines_agent_chat_detached_variant() {
    // The enum variant itself is the anchor — every other arm in the
    // codebase can be re-derived from it by rustc's non-exhaustive
    // match check, but the variant's presence must be an explicit
    // contract because it's the only place `"agentChatDetached"` maps to a
    // runtime routing decision.
    assert!(
        APP_VIEW_STATE.contains("AgentChatDetached,"),
        "src/main_sections/app_view_state.rs must define \
         `ActionsDialogHost::AgentChatDetached` — the enum variant that \
         carries `triggerAction host=agentChatDetached` through the main view \
         and into the detached window's own dispatcher. Removing this \
         variant would silently fall back to the `_ => handle_action` \
         arm which closes the TabAI harness instead of the detached \
         window."
    );
    assert!(
        APP_VIEW_STATE.contains("Actions in the detached Agent Chat chat window"),
        "src/main_sections/app_view_state.rs must keep the doc comment \
         explaining what `AgentChatDetached` is for — the comment is the \
         single place that records *why* this variant routes differently \
         from `AgentChat` (detached window owns its own focus + dispatcher)."
    );
}

#[test]
fn trigger_action_host_parser_accepts_agent_chat_detached() {
    // The parser lives in the single TriggerAction dispatcher site at
    // src/main_entry/app_run_setup.rs:~2616. Pin the exact host-string
    // form that automation sends so a rename (e.g. "agent_chat-detached" or
    // "agentChatDetachedChat") would require an intentional co-edit of this
    // test and the agentic-testing client docs.
    assert!(
        APP_RUN_SETUP.contains("Some(\"agentChatDetached\") => {")
            || APP_RUN_SETUP.contains("Some(\"agentChatDetached\") =>"),
        "src/main_entry/app_run_setup.rs TriggerAction host parser \
         must match `Some(\"agentChatDetached\")` and resolve to \
         `ActionsDialogHost::AgentChatDetached`. Without this arm, the \
         `Some(other) => ... falling back to current view host` branch \
         will silently re-route AgentChatDetached requests to whatever the \
         main view currently advertises, and the agentic harness will \
         never reach the detached close path."
    );
    assert!(
        APP_RUN_SETUP.contains("Some(ActionsDialogHost::AgentChatDetached)"),
        "src/main_entry/app_run_setup.rs must resolve the \
         `Some(\"agentChatDetached\")` host string to \
         `Some(ActionsDialogHost::AgentChatDetached)` (not to a fallback \
         variant). Any routing to a different enum value would send \
         the action through the main view's handler rather than the \
         detached window's dispatcher."
    );
}

#[test]
fn execute_action_for_actions_host_routes_agent_chat_detached_to_chat_window_helper() {
    // The router in `execute_action_for_actions_host` is the seam: it
    // is where "the main view received a TriggerAction for the detached
    // window" becomes "the detached window's dispatcher runs".
    // Losing this arm would make AgentChatDetached fall through the `_ =>
    // self.handle_action(action_id, window, cx)` branch, which as of
    // Pass #29 still handles `agent_chat_close` by closing the TabAI harness.
    assert!(
        ACTIONS_DIALOG.contains("ActionsDialogHost::AgentChatDetached =>"),
        "src/app_impl/actions_dialog.rs `execute_action_for_actions_host` \
         must include an explicit arm for `ActionsDialogHost::AgentChatDetached`. \
         The wildcard `_ => self.handle_action(...)` path would close \
         the TabAI harness instead of the detached window."
    );
    assert!(
        ACTIONS_DIALOG
            .contains("crate::ai::agent_chat::ui::chat_window::dispatch_action_to_detached("),
        "src/app_impl/actions_dialog.rs must call \
         `crate::ai::agent_chat::ui::chat_window::dispatch_action_to_detached` \
         inside the AgentChatDetached arm — this is the one helper that \
         routes the action id through the detached window's own \
         `dispatch_detached_action_checked`. Replacing it with a \
         reimplementation would duplicate the detached-window action \
         allowlist and drift from the in-window popup path."
    );
}

#[test]
fn chat_window_exposes_dispatch_action_to_detached_helper() {
    // Pin the public helper's signature so it stays callable from
    // `actions_dialog.rs`. If the helper is renamed without updating
    // `actions_dialog.rs`, the compile break is obvious — this test
    // makes the breakage obvious as a story regression, not just a
    // naming lint.
    assert!(
        CHAT_WINDOW.contains("pub fn dispatch_action_to_detached(action_id: &str, cx: &mut App)"),
        "src/ai/agent_chat/ui/chat_window.rs must expose \
         `pub fn dispatch_action_to_detached(action_id: &str, cx: &mut App) -> bool` \
         — the single entry point that routes automation TriggerAction \
         requests into the detached window's own \
         `dispatch_detached_action_checked`. Narrowing its visibility \
         (e.g. `pub(crate)`) is fine as long as `actions_dialog.rs` can \
         still reach it; renaming requires a deliberate edit to both \
         sites and this contract."
    );
    assert!(
        CHAT_WINDOW.contains("dispatch_detached_action_checked(&view_weak, action_id, cx)"),
        "src/ai/agent_chat/ui/chat_window.rs `dispatch_action_to_detached` must \
         delegate to `dispatch_detached_action_checked` — the same \
         dispatcher the detached window's popup uses. Rolling an \
         inline match would bypass the checked-upgrade guard and \
         silently no-op when the view entity is dead."
    );
}

#[test]
fn agent_chat_close_arm_still_closes_chat_window_in_detached_dispatcher() {
    // The detached dispatcher's `"agent_chat_close"` arm at the end of
    // `dispatch_detached_action` is the actual close action. If that
    // arm is renamed or its body changed to something other than
    // `close_chat_window(cx)`, the TriggerAction path will still
    // reach the dispatcher but the window will not close — a silent
    // story regression. Pin both the literal action id and the
    // `close_chat_window` call inside the same function so a refactor
    // that drops one without the other fails loudly.
    let close_arm_idx = CHAT_WINDOW
        .find("\"agent_chat_close\" => {\n            close_chat_window(cx);")
        .unwrap_or_else(|| {
            panic!(
                "src/ai/agent_chat/ui/chat_window.rs `dispatch_detached_action` \
                 must keep the `\"agent_chat_close\" => {{ close_chat_window(cx); \
                 ... }}` arm intact. Any change here must be mirrored \
                 in `dispatch_action_to_detached` wiring above, since \
                 both TriggerAction-driven and popup-driven closes \
                 share this single arm."
            )
        });
    // Also assert the tracing event name stays stable — the log is the
    // primary off-live-verification receipt and audit story proofs
    // reference it by name.
    let log_idx = CHAT_WINDOW[close_arm_idx..]
        .find("event = \"detached_action_close\"")
        .map(|i| i + close_arm_idx);
    assert!(
        log_idx.is_some(),
        "src/ai/agent_chat/ui/chat_window.rs `\"agent_chat_close\"` arm must emit \
         `tracing::info!(event = \"detached_action_close\", ...)` so \
         `audits/afk/log.md` receipts can reference the event by name. \
         Renaming the event is a tooling-visible change — update this \
         contract deliberately."
    );
}
