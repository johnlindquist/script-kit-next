use super::*;

#[test]
fn pending_menu_syntax_ai_proposals_carry_origin_and_guard_accept() {
    let ai_source = read_source("src/app_impl/menu_syntax_ai.rs");
    assert!(
        ai_source.contains("pub(crate) struct PendingMenuSyntaxAiProposal"),
        "pending proposal wrapper must exist so runtime proposals carry their origin"
    );
    assert!(
        ai_source.contains("pub raw_input: String"),
        "pending proposal origin must store the exact raw input that produced it"
    );
    assert!(
        ai_source.contains("self.origin.raw_input == current_input"),
        "freshness check must be exact raw-input equality"
    );

    let apply_source = read_source("src/app_impl/menu_syntax_ai_apply.rs");
    assert!(
        apply_source.contains("pub(crate) fn apply_pending_proposal("),
        "runtime proposal acceptance must route through the pending wrapper"
    );
    assert!(
        apply_source
            .contains("ProposalApplyAction::Accept if !pending.is_current_for(current_input)"),
        "stale accepts must dismiss instead of applying to changed input"
    );
}

#[test]
fn runtime_generation_and_render_paths_use_pending_origin() {
    let tab_source = read_source("src/app_impl/agent_handoff/mod.rs");
    assert!(
        tab_source.contains("PendingMenuSyntaxAiProposal::new("),
        "Cmd+Enter generation must store proposals with their raw-input origin"
    );
    assert!(
        tab_source.contains("let origin_matches = pending.is_current_for(&current_input);"),
        "accept/dismiss logging must record whether the proposal origin still matches"
    );
    assert!(
        tab_source.contains("apply_pending_proposal(&current_input, &pending, action)"),
        "accept must call the stale-guarded pending applier"
    );

    let hint_source = read_source("src/app_impl/menu_syntax_main_hint.rs");
    assert!(
        hint_source.contains(".filter(|pending| pending.is_current_for(raw_filter_text))"),
        "main hint snapshots must not expose stale pending proposals"
    );
    assert!(
        hint_source.contains(".map(|pending| &pending.proposal)"),
        "hint context should receive only the renderable proposal after the origin guard"
    );
}
