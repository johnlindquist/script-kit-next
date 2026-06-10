//! Source-level contract pinning the Run 7 Pass #21 Fix (commit
//! `69d5846f0`): `resolve_actions_popup_parent_automation_id` in
//! `src/actions/window.rs` MUST read the registry's existing
//! `semantic_surface` for the synthesized `"main"` entry BEFORE
//! re-upserting it, and MUST NOT hardcode `Some("scriptList"...)` in
//! the upsert payload.
//!
//! **The defended behavior**: when a non-main-menu host (clipboard
//! history with `clipboardHistory` surface, file-search with
//! `fileSearch`, embedded Agent Chat with `agentChatChat`, etc.) opens the shared
//! actions dialog, the actions-popup synthesis path needs a main-window
//! automation record to parent the popup against. If that record is
//! missing, the synthesis path upserts one. Because
//! [[src/windows/automation_registry.rs#upsert_automation_window]] is a
//! full-replace (`state.windows.insert(id, info)`), the upsert must
//! read the *existing* `semantic_surface` from the registry and pass it
//! through — falling back to `"scriptList"` only when no prior record
//! exists. Previously the site hardcoded `"scriptList"`, which wiped
//! `clipboardHistory` / `fileSearch` / `agentChatChat` every time the user
//! pressed Cmd+K on a non-main-menu host. Filed originally as
//! `[?] actions-cmdk-clipboard-main-surface-flip` in Run 7 Pass #17;
//! Fix landed Run 7 Pass #21 (commit `69d5846f0`).
//!
//! `removed-docs metadata` explicitly identified a
//! source-level Pin pass against this invariant as "a natural follow-up
//! because it's a quiet preserve-existing-value contract that any
//! future 'simplification' of the synthesis path could regress without
//! any compile-time signal." This file is that Pin.
//!
//! **Refactor threat** (per looper/rules/discipline.md §"Pin verb
//! semantics"): a contributor consolidating the 3 `upsert_automation_window`
//! call sites in `src/actions/window.rs` (there are multiple: the
//! synthesis path here, plus `register_attached_popup` for the actions
//! popup itself, plus the resize-time re-publish at
//! `set_automation_bounds` callers) into a shared helper could easily
//! drop the `preserved_semantic_surface` read — the helper's signature
//! would naturally take `semantic_surface: Option<String>` as a
//! parameter, and a well-meaning "cleanup" that "the caller already
//! knows what surface it wants" would produce either `Some("scriptList")`
//! (regression) or `None` (also regression, because `AutomationWindowInfo`
//! would drop to default surface). Second plausible refactor: a
//! contributor moving the `preserved_semantic_surface` logic out of
//! this function into a reusable `synthesize_main_with_preserved_surface()`
//! helper could leave the `upsert_automation_window` call here with a
//! literal default, accidentally inverting the preserve/default
//! precedence. Third: a "cleanup" that keeps the read/upsert pattern
//! but deletes the 8-line comment block explaining *why* (anchored at
//! `actions-cmdk-clipboard-main-surface-flip` + "Run 7 Pass #17" + "Pass
//! #20") loses the load-bearing "why", making the next contributor
//! wonder why the code is so convoluted and "simplify" it back to a
//! hardcoded literal.
//!
//! The four assertions pinned below:
//!   1. The function `resolve_actions_popup_parent_automation_id`
//!      exists at source level (structural anchor).
//!   2. Inside that function, `list_automation_windows()` appears
//!      followed by `.and_then(|w| w.semantic_surface)` — the
//!      registry-read-before-upsert signature.
//!   3. The function body MUST NOT contain any hardcoded
//!      `semantic_surface: Some("scriptList".to_string())` or
//!      `semantic_surface: Some("scriptList".into())` literal in the
//!      upsert payload — the regression signature of the pre-fix code.
//!   4. The 4-phrase Run 7 Pass #21 anchor-comment block appears
//!      verbatim between the `fn resolve_actions_popup_parent_automation_id`
//!      head and the `upsert_automation_window` call in the same
//!      function body.

const ACTIONS_WINDOW: &str = include_str!("../src/actions/window.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/actions/dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &after_start[..end_index]
}

const FN_HEAD: &str = "fn resolve_actions_popup_parent_automation_id(";
const REGISTRY_READ: &str = "list_automation_windows()";
const SEMANTIC_SURFACE_EXTRACT: &str = ".and_then(|w| w.semantic_surface)";
const UPSERT_CALL: &str = "crate::windows::upsert_automation_window(";
const PRESERVED_FIELD_ASSIGN: &str = "semantic_surface: Some(preserved_semantic_surface)";

// The four phrases from the Run 7 Pass #21 anchor-comment that explain
// *why* the read-before-upsert pattern exists. Checked as individual
// substrings so whitespace / rewrapping changes don't break the pin;
// checked in order below to catch a reshuffled rewrite that would lose
// the "why".
const PASS_21_COMMENT_PHRASES: &[&str] = &[
    "Preserve the existing main window's semantic_surface if the registry",
    "hardcoded `semantic_surface: \"scriptList\"` and so REWROTE main's",
    "actions-cmdk-clipboard-main-surface-flip",
    "Run 7 Pass #17",
];

#[test]
fn resolve_actions_popup_parent_automation_id_function_exists() {
    let count = ACTIONS_WINDOW.matches(FN_HEAD).count();
    assert_eq!(
        count, 1,
        "`src/actions/window.rs` must contain the function head \
         `{FN_HEAD}` exactly once (found {count}). This function is \
         the actions-popup synthesis path that the Run 7 Pass #21 Fix \
         landed the `preserved_semantic_surface` read inside. Without \
         this function, the Fix has no home and the invariant cannot \
         be pinned."
    );
}

#[test]
fn resolve_actions_popup_parent_reads_registry_before_upsert() {
    let fn_start = ACTIONS_WINDOW
        .find(FN_HEAD)
        .expect("function head missing (covered by resolve_actions_popup_parent_automation_id_function_exists)");
    let tail = &ACTIONS_WINDOW[fn_start..];

    // The tail contains the function body, which ends at the next
    // top-level `fn ` definition. Slice to that boundary so we don't
    // pick up calls from sibling functions below.
    let body_end = tail[FN_HEAD.len()..]
        .find("\nfn ")
        .map(|o| o + FN_HEAD.len())
        .unwrap_or(tail.len());
    let body = &tail[..body_end];

    let read_idx = body.find(REGISTRY_READ).unwrap_or_else(|| {
        panic!(
            "`resolve_actions_popup_parent_automation_id` body must \
             call `{REGISTRY_READ}` to read the existing main window's \
             `semantic_surface` from the registry BEFORE re-upserting \
             it. The Run 7 Pass #21 Fix (commit `69d5846f0`) introduced \
             this read to close the \
             `actions-cmdk-clipboard-main-surface-flip` anomaly. A \
             contributor removing the read would regress the fix."
        )
    });
    let extract_idx = body.find(SEMANTIC_SURFACE_EXTRACT).unwrap_or_else(|| {
        panic!(
            "`resolve_actions_popup_parent_automation_id` body must \
             extract the existing semantic_surface via \
             `{SEMANTIC_SURFACE_EXTRACT}` on the `list_automation_windows()` \
             result. Without this extraction, the upsert would use a \
             default and wipe any non-scriptList surface (clipboardHistory, \
             fileSearch, agentChatChat) that was already set."
        )
    });
    let upsert_idx = body.find(UPSERT_CALL).unwrap_or_else(|| {
        panic!(
            "`resolve_actions_popup_parent_automation_id` body must \
             call `{UPSERT_CALL}` to register the synthesized main \
             window. Without the upsert, the actions popup has no \
             parent to attach to."
        )
    });
    let preserved_idx = body.find(PRESERVED_FIELD_ASSIGN).unwrap_or_else(|| {
        panic!(
            "`resolve_actions_popup_parent_automation_id` body must \
             pass `{PRESERVED_FIELD_ASSIGN}` as the `semantic_surface` \
             field of the upserted `AutomationWindowInfo`. Without \
             this, the upsert defaults to some other value and the \
             `preserved_semantic_surface` read is dead code."
        )
    });

    assert!(
        read_idx < extract_idx,
        "In `resolve_actions_popup_parent_automation_id`, the \
         `{REGISTRY_READ}` call (offset {read_idx}) must precede the \
         `{SEMANTIC_SURFACE_EXTRACT}` extraction (offset {extract_idx}). \
         The extract reads from the registry result."
    );
    assert!(
        extract_idx < upsert_idx,
        "In `resolve_actions_popup_parent_automation_id`, the \
         `{SEMANTIC_SURFACE_EXTRACT}` extraction (offset {extract_idx}) \
         must precede the `{UPSERT_CALL}` call (offset {upsert_idx}). \
         The upsert consumes the extracted value; reversing this order \
         means the upsert ran before the read — the exact pre-Run 7 \
         Pass #21 regression shape."
    );
    assert!(
        upsert_idx < preserved_idx || preserved_idx < upsert_idx + 800,
        "The `{PRESERVED_FIELD_ASSIGN}` field (offset {preserved_idx}) \
         must appear inside the upsert call body (near offset \
         {upsert_idx}, within 800 bytes). If it lives outside the \
         upsert call, the `preserved_semantic_surface` variable is \
         computed but not threaded into the upsert — the fix is dead."
    );
}

#[test]
fn resolve_actions_popup_parent_forbids_hardcoded_scriptlist_upsert() {
    let fn_start = ACTIONS_WINDOW
        .find(FN_HEAD)
        .expect("function head missing (covered by resolve_actions_popup_parent_automation_id_function_exists)");
    let tail = &ACTIONS_WINDOW[fn_start..];
    let body_end = tail[FN_HEAD.len()..]
        .find("\nfn ")
        .map(|o| o + FN_HEAD.len())
        .unwrap_or(tail.len());
    let body = &tail[..body_end];

    // The regression signatures: a contributor "simplifying" the
    // upsert back to a hardcoded literal. Check for both common
    // `.to_string()` and `.into()` variants. Also forbid the bare
    // `Some("scriptList")` form in case a future Rust edition allows
    // &str -> String coercion in that position.
    let forbidden_literals: &[&str] = &[
        "semantic_surface: Some(\"scriptList\".to_string())",
        "semantic_surface: Some(\"scriptList\".into())",
        "semantic_surface: Some(String::from(\"scriptList\"))",
    ];
    for forbidden in forbidden_literals {
        assert!(
            !body.contains(forbidden),
            "`resolve_actions_popup_parent_automation_id` body contains \
             the forbidden literal `{forbidden}` in its \
             `upsert_automation_window` call. This is the pre-Run 7 \
             Pass #21 regression shape: hardcoding `scriptList` \
             wipes any existing non-scriptList surface \
             (clipboardHistory, fileSearch, agentChatChat) every time the \
             actions popup opens on a non-main-menu host. The Fix \
             replaced this literal with \
             `semantic_surface: Some(preserved_semantic_surface)` \
             backed by a `list_automation_windows().and_then(|w| \
             w.semantic_surface).unwrap_or_else(|| \
             \"scriptList\".to_string())` read. Restoring the \
             hardcoded literal reverts the Fix."
        );
    }
}

#[test]
fn resolve_actions_popup_parent_carries_pass_21_anchor_comment() {
    let fn_start = ACTIONS_WINDOW
        .find(FN_HEAD)
        .expect("function head missing (covered by resolve_actions_popup_parent_automation_id_function_exists)");
    let tail = &ACTIONS_WINDOW[fn_start..];
    let body_end = tail[FN_HEAD.len()..]
        .find("\nfn ")
        .map(|o| o + FN_HEAD.len())
        .unwrap_or(tail.len());
    let body = &tail[..body_end];

    let upsert_idx = body
        .find(UPSERT_CALL)
        .expect("upsert call missing (covered by resolve_actions_popup_parent_reads_registry_before_upsert)");
    let before_upsert = &body[..upsert_idx];

    // Each Pass #21 comment phrase must appear in order in the slice
    // BEFORE the upsert call — this is the load-bearing "why" that
    // explains to a future contributor why the registry-read-before-
    // upsert pattern exists. A contributor who deletes or reshuffles
    // the comment while keeping the code loses the "why" and makes the
    // next "simplification" more likely.
    let mut cursor = 0usize;
    for phrase in PASS_21_COMMENT_PHRASES {
        let found = before_upsert[cursor..].find(phrase).unwrap_or_else(|| {
            panic!(
                "The Run 7 Pass #21 anchor-comment phrase {phrase:?} \
                 is missing (or out of order) before the \
                 `upsert_automation_window` call in \
                 `resolve_actions_popup_parent_automation_id`. The \
                 four-phrase comment block cites the original anomaly \
                 (`actions-cmdk-clipboard-main-surface-flip`) and \
                 explains why the `upsert_automation_window` full-replace \
                 semantics force a read-before-upsert pattern. A \
                 contributor who deletes or reshuffles this comment \
                 loses the 'why' — the pattern looks convoluted and \
                 invites a 'simplification' that reverts to a hardcoded \
                 literal. All four phrases must appear in order before \
                 the upsert call."
            )
        });
        cursor += found + phrase.len();
    }
}

#[test]
fn actions_dialog_stores_immutable_host_context_snapshot() {
    assert!(
        ACTIONS_DIALOG.contains("pub(crate) struct ActionsHostContextSnapshot"),
        "ActionsDialog must expose a typed host context snapshot so parent identity is not inferred from popup focus"
    );
    assert!(
        ACTIONS_DIALOG.contains("#[serde(rename_all = \"camelCase\")]"),
        "ActionsHostContextSnapshot must serialize camelCase fields for DevTools receipts"
    );
    assert!(
        ACTIONS_DIALOG.contains("host_context: Option<ActionsHostContextSnapshot>"),
        "ActionsDialog must store the immutable host context captured at open time"
    );
    assert!(
        ACTIONS_DIALOG.contains(
            "pub(crate) fn set_host_context(&mut self, context: ActionsHostContextSnapshot)"
        ),
        "ActionsDialog must have an explicit setter for the open-time host context snapshot"
    );
}

#[test]
fn actions_dialog_automation_state_emits_host_context_fields() {
    let automation_state = source_between(
        ACTIONS_DIALOG,
        "pub(crate) fn automation_state(&self, surface: &str, cx: &gpui::App)",
        "    /// Set actions from SDK",
    );

    for required in [
        "\"hostContext\"",
        "\"contextStableKey\"",
        "\"contextSource\"",
        "\"selectedSemanticId\"",
    ] {
        assert!(
            automation_state.contains(required),
            "ActionsDialog automation_state must emit {required} from the immutable host context snapshot"
        );
    }
}

#[test]
fn script_list_actions_open_paths_capture_host_context_before_popup_spawn() {
    let helper = source_between(
        ACTIONS_TOGGLE,
        "pub(crate) fn build_actions_host_context_snapshot(",
        "        crate::actions::ActionsHostContextSnapshot",
    );
    assert!(helper.contains("parent_automation_id"));
    assert!(helper.contains("parent_semantic_surface"));
    assert!(helper.contains("parent_subject_id"));
    assert!(helper.contains("selected_semantic_id"));

    let generic_open = source_between(
        ACTIONS_TOGGLE,
        "            // Create the dialog entity HERE in main app",
        "            // Store the dialog entity for keyboard routing",
    );
    assert!(
        generic_open.contains("build_actions_host_context_snapshot(host"),
        "toggle_actions must build a host context snapshot before storing/spawning the dialog"
    );
    assert!(
        generic_open.contains("d.set_host_context(host_context);"),
        "toggle_actions must install the snapshot on the dialog before popup registration"
    );

    let root_file_open = source_between(
        ACTIONS_TOGGLE,
        "pub(crate) fn toggle_root_file_actions(",
        "        self.actions_dialog = Some(dialog.clone());",
    );
    assert!(
        root_file_open.contains("build_actions_host_context_snapshot(host"),
        "root file actions must capture the same host context snapshot before popup registration"
    );

    let root_unified_open = source_between(
        ACTIONS_TOGGLE,
        "pub(crate) fn toggle_root_unified_result_actions(",
        "        self.actions_dialog = Some(dialog.clone());",
    );
    assert!(
        root_unified_open.contains("build_actions_host_context_snapshot(host"),
        "root unified actions must capture the same host context snapshot before popup registration"
    );
}
