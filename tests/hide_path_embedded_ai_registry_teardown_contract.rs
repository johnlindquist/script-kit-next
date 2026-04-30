//! Source-level contract pinning the Pass #21 Fix for the hide-path
//! teardown of the embedded-AI automation registry entry.
//!
//! Background: Run 9 Pass #20 (attacker probe, 33 actions × 4 categories,
//! commit `237ad3245`) filed `[?] attacker-hide-path-embedded-ai-registry-stale`
//! — after `triggerBuiltin tab-ai` then `{type:"hide"}`,
//! `listAutomationWindows` returned a stale `{id:"ai", kind:"ai",
//! visible:true, semanticSurface:"acpChat", parentWindowId:"main"}` entry
//! even though main's own entry had been correctly re-keyed to
//! `semanticSurface:"scriptList"` with `visible:false`. Root cause: the
//! four hide dispatchers called `reset_to_script_list(ctx)` +
//! `update_automation_semantic_surface("main", …)` but NEVER
//! `ensure_embedded_ai_window(false)` — so the `ai` child-entry that
//! `ensure_embedded_ai_window(true)` writes on tab-ai entry persisted
//! across hide.
//!
//! The Fix adds `ensure_embedded_ai_window(false)` to ALL FOUR hide
//! dispatcher sites, symmetric with the already-pinned
//! `close_acp_chat_to_script_list` teardown at
//! `src/app_impl/tab_ai_mode/mod.rs:3151`.
//!
//! These tests pin, for each of the 4 hide dispatcher files:
//! 1. The hide path calls `ensure_embedded_ai_window(false)` AT ALL.
//! 2. That call is AFTER `reset_to_script_list` (teardown must follow
//!    the view flip; ordering it before the flip would leak a stale
//!    entry if a future contributor splits the flip across an
//!    intermediate subview that re-asserts the `ai` registry write).
//! 3. The teardown is ADJACENT to the `update_automation_semantic_surface("main", …)`
//!    call — the Pass #20 anomaly identified that re-key as the sibling
//!    registry write that SHOULD have triggered a same-location review
//!    for the child entry. Keeping them adjacent in source makes any
//!    future refactor that moves one and forgets the other loudly wrong.
//!
//! **Refactor threat**: a contributor refactoring one of the four hide
//! dispatchers to centralize the "reset view + re-key automation" block
//! into a helper (plausible consolidation — the four dispatchers already
//! share identical 30-line bodies) could easily drop the
//! `ensure_embedded_ai_window(false)` call from the extraction. The
//! pairing is code-only, not contract-tested except by this file. A
//! moved or deleted call in any one of the four sites would silently
//! regress the receipt for every hide-from-acpChat sequence on that
//! dispatcher path and reopen the Pass #20 anomaly. This contract
//! catches that edit at test time.

const HIDE_SITES: &[(&str, &str)] = &[
    (
        "window_visibility",
        include_str!("../src/main_sections/window_visibility.rs"),
    ),
    (
        "runtime_stdin",
        include_str!("../src/main_entry/runtime_stdin.rs"),
    ),
    (
        "runtime_stdin_match_core",
        include_str!("../src/main_entry/runtime_stdin_match_core.rs"),
    ),
    (
        "app_run_setup",
        include_str!("../src/main_entry/app_run_setup.rs"),
    ),
];

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn every_hide_site_calls_ensure_embedded_ai_window_false() {
    for (name, src) in HIDE_SITES {
        assert!(
            src.contains("ensure_embedded_ai_window(false)"),
            "Hide dispatcher {name} MUST call \
             `ensure_embedded_ai_window(false)` to tear down the `ai` \
             child registry entry when the main window hides from an \
             embedded-ACP view. Without this, `listAutomationWindows` \
             post-hide leaves a stale `{{id:\"ai\", visible:true, \
             semanticSurface:\"acpChat\"}}` entry that disagrees with \
             its parent main entry on both visibility and semantic \
             surface — the Pass #20 attacker anomaly \
             `attacker-hide-path-embedded-ai-registry-stale`."
        );
    }
}

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn teardown_follows_reset_to_script_list_in_every_site() {
    for (name, src) in HIDE_SITES {
        // Find EVERY `reset_to_script_list(ctx)` and `ensure_embedded_ai_window(false)`
        // site; for each teardown, check that a preceding `reset_to_script_list(ctx)`
        // exists. (A single file may contain multiple hide arms; this test requires
        // teardown-follows-reset for every teardown call.)
        let teardown_positions: Vec<usize> = src
            .match_indices("ensure_embedded_ai_window(false)")
            .map(|(idx, _)| idx)
            .collect();
        assert!(
            !teardown_positions.is_empty(),
            "{name} has no `ensure_embedded_ai_window(false)` call (covered \
             by every_hide_site_calls_ensure_embedded_ai_window_false — \
             fix that first)."
        );
        for teardown_idx in &teardown_positions {
            let before = &src[..*teardown_idx];
            let last_reset = before.rfind("view.reset_to_script_list(ctx);");
            assert!(
                last_reset.is_some(),
                "In {name}, the `ensure_embedded_ai_window(false)` call at \
                 offset {teardown_idx} is not preceded by any \
                 `view.reset_to_script_list(ctx);`. Teardown must follow \
                 the view flip."
            );
        }
    }
}

// @lat: [[lat.md/acp-chat#Detached window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn teardown_is_adjacent_to_semantic_surface_rekey_in_every_site() {
    for (name, src) in HIDE_SITES {
        // Pair each `update_automation_semantic_surface("main", Some("scriptList".to_string()))`
        // (either single-line or the multi-line, hide-arm-specific form)
        // with its following `ensure_embedded_ai_window(false)` and
        // require the gap to be under ~800 bytes. We deliberately
        // discriminate on the `Some("scriptList".to_string())` literal
        // in the 2nd argument so the TriggerBuiltin post-dispatch rekey
        // helper is NOT matched — only hide-path rekeys carry the
        // hardcoded scriptList literal and only they need the paired
        // teardown.
        let mut rekey_positions: Vec<usize> = Vec::new();
        let inline =
            "update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()))";
        rekey_positions.extend(src.match_indices(inline).map(|(idx, _)| idx));
        // Multi-line form: scan for every `update_automation_semantic_surface(`
        // and keep it only if `Some("scriptList"` appears within the next
        // 200 bytes (covers the 3-line `fn(\n    "main",\n    Some("scriptList"...))`
        // shape used across all four hide dispatchers).
        for (idx, _) in src.match_indices("update_automation_semantic_surface(") {
            let window_end = (idx + 200).min(src.len());
            let window = &src[idx..window_end];
            if window.contains("Some(\"scriptList\"") && !rekey_positions.contains(&idx) {
                rekey_positions.push(idx);
            }
        }
        rekey_positions.sort();
        assert!(
            !rekey_positions.is_empty(),
            "{name} does not contain any \
             `update_automation_semantic_surface(\"main\", ...)` call; \
             the hide path source shape has drifted from the contract \
             this test defends."
        );
        for rekey_idx in &rekey_positions {
            let after = &src[*rekey_idx..];
            let teardown_rel = after.find("ensure_embedded_ai_window(false)");
            assert!(
                teardown_rel.is_some(),
                "In {name}, the `update_automation_semantic_surface(\"main\", ...)` \
                 at offset {rekey_idx} has NO following \
                 `ensure_embedded_ai_window(false)` call. They must be \
                 sibling registry writes within the same hide arm."
            );
            let gap = teardown_rel.unwrap();
            assert!(
                gap < 800,
                "In {name}, the gap between \
                 `update_automation_semantic_surface(\"main\", ...)` \
                 (offset {rekey_idx}) and the following \
                 `ensure_embedded_ai_window(false)` is {gap} bytes — \
                 must stay under 800 so the two sibling registry writes \
                 remain lexically co-located. A refactor that pushes \
                 them apart breaks this invariant."
            );
            let between = &after[..gap];
            assert!(
                !between.contains("pub fn ") && !between.contains("\nfn "),
                "In {name}, a function boundary appears between the \
                 `update_automation_semantic_surface(\"main\", ...)` \
                 re-key and the `ensure_embedded_ai_window(false)` \
                 teardown. They must live in the same function body. \
                 Intervening text:\n{between}"
            );
        }
    }
}
