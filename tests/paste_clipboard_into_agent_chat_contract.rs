//! Source-level contract test for the Run 2 Pass #32
//! `tool-paste-clipboard-into-agent_chat-composer` user story.
//!
//! Pass #30 closed `clipboard-to-agent_chat-paste`'s PRIMARY blocker
//! (simulateGpuiEvent Enter against clipboard-history) end-to-end at
//! substrate level, and Pass #31 pinned the composer's paste receiver
//! (`paste_text_from_clipboard`) invariants at source level. The
//! remaining gap was substrate: there was no automation path to
//! actually INVOKE the receiver — clipboard-history's accept pastes
//! to the OS-frontmost app (the invoking terminal during automation
//! runs), not the Agent Chat composer, and `simulateKey cmd+v` routes
//! through CGEvent which likewise targets the OS frontmost.
//!
//! Pass #32 closes that gap by adding `ExternalCommand::PasteClipboardIntoAgentChat`
//! — a stdin command that invokes `paste_text_from_clipboard` DIRECTLY
//! on the active `AppView::AgentChatView` entity, bypassing the OS
//! frontmost heuristic entirely. The command is the substrate that
//! lets `clipboard-to-agent_chat-paste`'s character-for-character round-trip
//! acceptance clause be live-verified against the composer.
//!
//! This contract test pins the variant definition + routing + three
//! dispatcher arms + composer-visibility promotion so a mechanical
//! refactor of the stdin command machinery can't silently regress the
//! substrate gain behind the now-closed `clipboard-to-agent_chat-paste`
//! story.

const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_TAIL: &str =
    include_str!("../src/main_entry/runtime_stdin_match_tail.rs");
const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");

#[test]
fn paste_clipboard_into_agent_chat_variant_is_defined_with_request_id_only() {
    // The command takes no args beyond the standard request_id — the
    // source of truth for the pasted text is the current system
    // clipboard, the target is always the active AgentChatView.
    // Adding surface-targeting args (e.g. target: "agentChatDetached") would
    // belong to a separate follow-up story and must NOT silently drift
    // this variant's shape.
    assert!(
        STDIN_COMMANDS.contains("PasteClipboardIntoAgentChat {\n        #[serde(default, rename = \"requestId\")]\n        request_id: Option<ExternalCommandRequestId>,\n    },"),
        "src/stdin_commands/mod.rs must define `PasteClipboardIntoAgentChat \
         {{ request_id }}` with ONLY the standard `requestId` field \
         (no target/host/text args — the system clipboard is the \
         text source and AgentChatView is the implicit target). Adding \
         args here without updating this test would let a refactor \
         silently widen the command's surface."
    );
}

#[test]
fn paste_clipboard_into_agent_chat_is_wired_into_request_id_and_command_type() {
    // Both `request_id()` and `command_type()` are the structured-log
    // correlation surface. A variant that is parsed but missing from
    // these two helpers would log as an unknown command — the agentic
    // testing harness keys on the exact `command = "pasteClipboardIntoAgentChat"`
    // string in `stdin_agent_chat_command_received` / `_finished` events.
    assert!(
        STDIN_COMMANDS.contains("| Self::PasteClipboardIntoAgentChat { request_id, .. } => {"),
        "src/stdin_commands/mod.rs `ExternalCommand::request_id()` must \
         include `| Self::PasteClipboardIntoAgentChat {{ request_id, .. }}` in \
         the chained match arm so the request id is included in the \
         structured-tracing log context. Missing this makes correlation \
         across `received` / `finished` events impossible."
    );
    assert!(
        STDIN_COMMANDS.contains(
            "Self::PasteClipboardIntoAgentChat { .. } => \"pasteClipboardIntoAgentChat\","
        ),
        "src/stdin_commands/mod.rs `ExternalCommand::command_type()` must \
         map `Self::PasteClipboardIntoAgentChat {{ .. }}` to the exact literal \
         string `\"pasteClipboardIntoAgentChat\"`. The agentic-testing harness \
         keys on this exact string in `stdin_agent_chat_command_received` + \
         `stdin_agent_chat_command_finished` events — renaming it invalidates \
         every test fixture that inspects stdin tracing."
    );
}

#[test]
fn paste_text_from_clipboard_is_reachable_from_crate_scope() {
    // The stdin handler calls `chat.paste_text_from_clipboard(cx)` from
    // the main crate, so the method must be at least `pub(crate)`.
    // Pass #31's contract test pinned the method SIGNATURE via substring
    // match (which still holds for `pub(crate) fn paste_text_from_clipboard(...)`)
    // but did NOT pin visibility — this assertion fills that gap.
    assert!(
        AGENT_CHAT_VIEW.contains(
            "pub(crate) fn paste_text_from_clipboard(&mut self, cx: &mut Context<Self>) -> bool"
        ),
        "src/ai/agent_chat/ui/view.rs `paste_text_from_clipboard` must be declared \
         `pub(crate) fn paste_text_from_clipboard(&mut self, cx: &mut \
         Context<Self>) -> bool` so the stdin `PasteClipboardIntoAgentChat` \
         handler in `main_entry/*` can invoke it directly. Regressing \
         to private `fn` would break the substrate without a cargo \
         check failure — the Pass #31 signature contract uses \
         substring match and would still pass."
    );
}

#[test]
fn all_three_dispatchers_handle_paste_clipboard_into_agent_chat() {
    // The triple-embedded stdin dispatcher pattern is a known rough
    // edge in this codebase (see memories 6330/6331). A variant added
    // to only ONE of the three would work from ONLY one entry point
    // — silently dropping the command from the other two. Pin all
    // three arms.
    for (name, source) in [
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
        ("src/main_entry/runtime_stdin.rs", RUNTIME_STDIN),
        (
            "src/main_entry/runtime_stdin_match_tail.rs",
            RUNTIME_STDIN_MATCH_TAIL,
        ),
    ] {
        assert!(
            source.contains("ExternalCommand::PasteClipboardIntoAgentChat { ref request_id } =>"),
            "{} must contain an `ExternalCommand::PasteClipboardIntoAgentChat \
             {{ ref request_id }} =>` arm. The triple-embedded stdin \
             dispatcher pattern means ALL three files must match — \
             otherwise the command is silently dropped from whichever \
             code path the missing file owns.",
            name
        );
        assert!(
            source.contains("chat.paste_text_from_clipboard(cx)"),
            "{} must call `chat.paste_text_from_clipboard(cx)` from \
             inside the `AgentChatView` arm of the handler. Routing the \
             stdin command through any other helper would bypass the \
             Pass #31-pinned receiver invariants.",
            name
        );
        assert!(
            source.contains("_ => Err(\"Agent Chat view is not active\".to_string()),"),
            "{} must return `Err(\"Agent Chat view is not active\")` when \
             the current view is not `AppView::AgentChatView`. The \
             agentic-testing harness keys on this exact error string to \
             distinguish \"command not routed\" from \"command ran but \
             clipboard was empty\" (the latter returns \"clipboard is \
             empty or text fetch failed\").",
            name
        );
        assert!(
            source.contains("\"clipboard is empty or text fetch failed\""),
            "{} must return the distinct error `\"clipboard is empty or \
             text fetch failed\"` when `paste_text_from_clipboard` \
             returns false. Conflating the two error shapes would mask \
             a broken clipboard (arboard init failure, non-text \
             content) as a \"wrong view\" error.",
            name
        );
        assert!(
            source.contains("command = \"pasteClipboardIntoAgentChat\""),
            "{} must emit `command = \"pasteClipboardIntoAgentChat\"` in the \
             structured-tracing `stdin_agent_chat_command_received` + \
             `stdin_agent_chat_command_finished` events so agentic-testing \
             harnesses can correlate the receive/finish edges by \
             command type.",
            name
        );
    }
}

#[test]
fn paste_handler_bypasses_os_frontmost_heuristic() {
    // The documented reason this command exists is to bypass the OS
    // Cmd+V heuristic that routes pastes to the frontmost app. Pin
    // the doc comment that anchors that invariant — if someone swaps
    // the implementation to drive CGEvent cmd+v (which would re-enter
    // the frontmost trap), this comment would stop being true and
    // should be updated, prompting review of the implementation.
    assert!(
        STDIN_COMMANDS.contains("bypassing the OS Cmd+V heuristic"),
        "src/stdin_commands/mod.rs doc comment on \
         `PasteClipboardIntoAgentChat` must preserve the `bypassing the OS \
         Cmd+V heuristic` explanation. This comment documents the \
         load-bearing reason the command exists — replacing it with a \
         generic paraphrase loses the design-intent anchor for future \
         refactors."
    );
    assert!(
        STDIN_COMMANDS.contains("routes pastes to the frontmost app"),
        "src/stdin_commands/mod.rs doc comment must preserve the \
         `routes pastes to the frontmost app` explanation so a future \
         reader understands why `simulateKey cmd+v` is NOT the right \
         substrate for Agent Chat-targeted paste tests."
    );
}
