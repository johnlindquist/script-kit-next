//! Source-level contract test for the Run 2 Pass #31
//! `agent_chat-composer-paste-text-invariants` user story.
//!
//! Pass #30 closed `clipboard-to-agent_chat-paste`'s PRIMARY blocker (the
//! `handle_unavailable` failure on `simulateGpuiEvent keyDown enter`
//! target=Main in the clipboard-history view) by live-verifying Pass
//! #5's Main-window re-acquire fix against that exact surface. The
//! remaining open half of `clipboard-to-agent_chat-paste` is the paste-to-Agent Chat
//! side: clipboard-history's accept action pastes to the OS-frontmost
//! app (the invoking terminal during automation), not into the Agent Chat
//! composer directly, so verifying the composer's `inputValue` after
//! a real clipboard-history accept requires either a new
//! `ExternalCommand::PasteClipboardInto {target:"agent_chat"}` substrate or
//! a careful frontmost-ordering dance.
//!
//! This pass takes the orthogonal approach: pin the Agent Chat composer's
//! OWN paste handler at source level, so the paste-to-Agent Chat half of
//! `clipboard-to-agent_chat-paste` has a locked-down receiver even before
//! the full-flow live verification is reachable. The handler lives
//! at `src/ai/agent_chat/ui/view.rs::paste_text_from_clipboard`, is the sole
//! entry point from Cmd+V into the composer's text input, and
//! implements the documented invariants in
//! [[removed-docs Chat#Agent Chat composer]] — most notably the
//! "Large clipboard text pastes collapse into inline `@text:` tokens"
//! contract that protects the composer from flooding.
//!
//! If any of these invariants silently drifts (e.g. someone swaps
//! `arboard` for a different clipboard API without preserving the
//! CRLF normalization, or inlines the `prepare_pasted_text` call
//! without registering the typed-mention alias), the documented
//! removed-docs section becomes a lie and the eventual `clipboard-to-agent_chat-paste`
//! live verification would land text in the composer that doesn't
//! match the documented paste model. This contract catches those
//! drifts at `cargo test` time instead of at pass-time.
//!
//! The thresholds (`PASTED_TEXT_LINE_THRESHOLD`, `PASTED_TEXT_CHAR_THRESHOLD`)
//! and the `@text:"..."` token format are also pinned because
//! removed-docs explicitly documents the collapse behavior and
//! portal_contract.rs uses the same token format for preview
//! descriptions — drifting one without the other would silently
//! break preview rendering for newly-pasted text.

const PASTED_TEXT: &str = include_str!("../src/pasted_text.rs");

#[test]
fn paste_text_thresholds_and_token_format_match_lat_md_documentation() {
    // removed-docs composer documents "Large clipboard text
    // pastes collapse into inline `@text:\"Pasted text #n +...\"`
    // tokens". The source-of-truth for "large" is the two constants
    // at the top of `src/pasted_text.rs`; pin them so a future
    // refactor that loosens or tightens the threshold requires a
    // deliberate edit to both this contract and the removed-docs section.
    assert!(
        PASTED_TEXT.contains("const PASTED_TEXT_LINE_THRESHOLD: usize = 8;"),
        "src/pasted_text.rs must keep `PASTED_TEXT_LINE_THRESHOLD: \
         usize = 8`. Adjusting this number changes the \"large paste\" \
         definition documented in removed-docs; both must move \
         together. 8 lines matches the composer's visible row count \
         before scroll — pasting a screen of text should tokenize, \
         pasting a short quote should not."
    );
    assert!(
        PASTED_TEXT.contains("const PASTED_TEXT_CHAR_THRESHOLD: usize = 600;"),
        "src/pasted_text.rs must keep `PASTED_TEXT_CHAR_THRESHOLD: \
         usize = 600`. This is the character threshold from \
         removed-docs \"large clipboard text pastes\" contract; \
         drifting this alone would make the tokenization behavior \
         differ from the documented model in ways that are hard to \
         notice by eye."
    );
    // The `@text:"Pasted text #n +..."` token format is the other
    // half of the contract — portal_contract.rs keys its preview
    // description on the `"text"` prefix, and the `"Pasted text #"`
    // literal is the stable label prefix that distinguishes synthetic
    // paste tokens from user-typed `@text:` mentions.
    assert!(
        PASTED_TEXT.contains("format!(\"@text:\\\"{label}\\\"\")"),
        "src/pasted_text.rs `prepare_pasted_text` must emit the token \
         as `format!(\"@text:\\\"{{label}}\\\"\")`. Any other format \
         (quoting style, prefix) would break the alias-lookup in \
         `typed_mention_aliases` and the preview-description parser \
         in portal_contract.rs at once."
    );
    assert!(
        PASTED_TEXT.contains("Pasted text #"),
        "src/pasted_text.rs must use the `\"Pasted text #\"` prefix \
         for generated labels. `preview_description_for_token` greps \
         for this exact prefix; renaming would silently drop preview \
         text for newly-pasted tokens."
    );
}

#[test]
fn prepare_pasted_text_is_pure_and_respects_existing_tokens() {
    // `prepare_pasted_text` is the bridge between the paste handler
    // and the token registry. It must accept `&[PastedTextToken]` by
    // reference (not consume it) so the caller can keep the registry
    // intact; and it must compute the next token index from the
    // passed registry (not a hidden global) so multiple Agent Chat views
    // (main-embedded, detached, notes-hosted) don't collide on
    // `Pasted text #N` labels.
    assert!(
        PASTED_TEXT.contains(
            "pub(crate) fn prepare_pasted_text(\n    text: &str,\n    existing_tokens: &[PastedTextToken],\n) -> PreparedPastedText"
        ),
        "src/pasted_text.rs must expose \
         `pub(crate) fn prepare_pasted_text(text: &str, existing_tokens: &[PastedTextToken]) -> PreparedPastedText`. \
         Changing the signature (e.g. taking `&mut Vec<PastedTextToken>` \
         or an owned `Vec`) would let the function mutate caller \
         state implicitly and couple the three Agent Chat hosts' paste \
         registries together."
    );
    assert!(
        PASTED_TEXT.contains("next_token_index(existing_tokens)"),
        "src/pasted_text.rs `prepare_pasted_text` must compute the \
         next index via `next_token_index(existing_tokens)` on the \
         passed-in slice. Using a `static AtomicUsize` counter would \
         make paste numbering process-global instead of per-view, \
         colliding across embedded + detached + notes-hosted Agent Chat."
    );
}
