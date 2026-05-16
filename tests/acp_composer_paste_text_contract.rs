//! Source-level contract test for the Run 2 Pass #31
//! `acp-composer-paste-text-invariants` user story.
//!
//! Pass #30 closed `clipboard-to-acp-paste`'s PRIMARY blocker (the
//! `handle_unavailable` failure on `simulateGpuiEvent keyDown enter`
//! target=Main in the clipboard-history view) by live-verifying Pass
//! #5's Main-window re-acquire fix against that exact surface. The
//! remaining open half of `clipboard-to-acp-paste` is the paste-to-ACP
//! side: clipboard-history's accept action pastes to the OS-frontmost
//! app (the invoking terminal during automation), not into the ACP
//! composer directly, so verifying the composer's `inputValue` after
//! a real clipboard-history accept requires either a new
//! `ExternalCommand::PasteClipboardInto {target:"acp"}` substrate or
//! a careful frontmost-ordering dance.
//!
//! This pass takes the orthogonal approach: pin the ACP composer's
//! OWN paste handler at source level, so the paste-to-ACP half of
//! `clipboard-to-acp-paste` has a locked-down receiver even before
//! the full-flow live verification is reachable. The handler lives
//! at `src/ai/acp/view.rs::paste_text_from_clipboard`, is the sole
//! entry point from Cmd+V into the composer's text input, and
//! implements the documented invariants in
//! [[removed-docs Chat#ACP composer]] — most notably the
//! "Large clipboard text pastes collapse into inline `@text:` tokens"
//! contract that protects the composer from flooding.
//!
//! If any of these invariants silently drifts (e.g. someone swaps
//! `arboard` for a different clipboard API without preserving the
//! CRLF normalization, or inlines the `prepare_pasted_text` call
//! without registering the typed-mention alias), the documented
//! removed-docs section becomes a lie and the eventual `clipboard-to-acp-paste`
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

const VIEW: &str = include_str!("../src/ai/acp/view.rs");
const PASTED_TEXT: &str = include_str!("../src/pasted_text.rs");

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn paste_text_from_clipboard_uses_arboard_and_normalizes_line_endings() {
    // arboard is the single clipboard API the composer uses; swapping
    // it silently would break the paste-to-composer path in ways that
    // only show up at runtime on a specific platform. Pin the
    // `arboard::Clipboard::new()` + `get_text()` shape here.
    assert!(
        VIEW.contains("fn paste_text_from_clipboard(&mut self, cx: &mut Context<Self>) -> bool"),
        "src/ai/acp/view.rs must keep the \
         `paste_text_from_clipboard(&mut self, cx: &mut Context<Self>) -> bool` \
         signature. This is the sole Cmd+V entry point from the \
         composer into the input text; renaming or changing the \
         signature breaks the clipboard→composer contract that \
         `clipboard-to-acp-paste` eventually relies on."
    );
    assert!(
        VIEW.contains("arboard::Clipboard::new()"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must read \
         the clipboard via `arboard::Clipboard::new()`. Swapping to \
         a different clipboard library would bypass the CRLF \
         normalization below and leak platform-specific line-ending \
         behavior into the composer."
    );
    assert!(
        VIEW.contains("clipboard.get_text()"),
        "src/ai/acp/view.rs must call `clipboard.get_text()` — the \
         text-only fast path. Using `get()` or similar would broaden \
         the handler to image/URL types that have their own dedicated \
         handlers (`paste_image_from_clipboard`)."
    );
    assert!(
        VIEW.contains(".replace(\"\\r\\n\", \"\\n\").replace('\\r', \"\\n\")"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must normalize \
         clipboard line endings with \
         `.replace(\"\\r\\n\", \"\\n\").replace('\\r', \"\\n\")` in that \
         order. Skipping this step would let Windows-originated \
         clipboard content (CRLF) drift vs. typed text (LF), producing \
         invisible whitespace diffs in the composer's input model."
    );
}

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn paste_text_from_clipboard_short_circuits_on_empty_text() {
    // An empty normalized string must return false without mutating
    // the input. A regression here would cause an empty paste to
    // insert an empty insertion_text AND notify the composer, which
    // can trigger cursor-move and selection-clear side effects.
    assert!(
        VIEW.contains("if normalized.is_empty() {\n            return false;\n        }"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must \
         short-circuit with `if normalized.is_empty() {{ return false; \
         }}` before calling `prepare_pasted_text`. Without this guard, \
         an empty clipboard paste would push an empty token through \
         `prepare_pasted_text`'s threshold check and potentially \
         register a zero-length typed-mention alias."
    );
}

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn paste_text_from_clipboard_routes_through_prepare_and_inserts_at_cursor() {
    // The `prepare_pasted_text` call is the single seam where small
    // pastes stay inline and large pastes collapse into `@text:"..."`
    // tokens. Inlining the logic at the call site would duplicate
    // the threshold constants (600 chars / 8 lines) and open the
    // door to them drifting apart from the unit tests in
    // `src/pasted_text.rs`.
    assert!(
        VIEW.contains(
            "crate::pasted_text::prepare_pasted_text(&normalized, &self.pasted_text_tokens)"
        ),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must call \
         `crate::pasted_text::prepare_pasted_text(&normalized, \
         &self.pasted_text_tokens)`. Passing anything other than \
         `&self.pasted_text_tokens` would break the next-token-index \
         computation and produce duplicate `@text:\"Pasted text #n\"` \
         labels across multiple pastes in the same session."
    );
    assert!(
        VIEW.contains("thread.input.insert_str(&insertion_text)"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must insert \
         via `thread.input.insert_str(&insertion_text)` — the \
         composer's text-input primitive that respects the current \
         cursor. Replacing with `set_text(...)` would wipe existing \
         typed content; replacing with a direct field write would \
         skip the undo-stack entry."
    );
    assert!(
        VIEW.contains("self.live_thread().update(cx, move |thread, cx|"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must insert \
         inside `self.live_thread().update(cx, move |thread, cx| \
         ...)`. Running the insert outside the thread's update scope \
         would miss the `cx.notify()` that triggers the composer \
         re-render."
    );
}

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn paste_text_from_clipboard_registers_token_as_text_block_context_part() {
    // When `prepare_pasted_text` returns a token (large paste),
    // `paste_text_from_clipboard` must build an `AiContextPart::TextBlock`
    // with the documented `clipboard://pasted-text/N` source and the
    // `text/plain` mime type. `portal_contract.rs` keys its preview
    // description on the token prefix `"text"` (see
    // `pasted_text::preview_description_for_token`) — drifting the
    // `TextBlock` shape here would silently break preview rendering
    // for newly-pasted text.
    assert!(
        VIEW.contains("AiContextPart::TextBlock {"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must build \
         an `AiContextPart::TextBlock {{ ... }}` for tokenized pastes. \
         Using a different AiContextPart variant would break the \
         preview-description lookup in portal_contract.rs."
    );
    assert!(
        VIEW.contains("clipboard://pasted-text/"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must build \
         the TextBlock's `source` as \
         `format!(\"clipboard://pasted-text/{{}}\", \
         self.pasted_text_tokens.len() + 1)` — this URI is how the \
         portal contract disambiguates paste N from paste N+1 in \
         preview descriptions."
    );
    assert!(
        VIEW.contains("mime_type: Some(\"text/plain\".to_string())"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must set \
         TextBlock `mime_type: Some(\"text/plain\".to_string())`. \
         Dropping this or changing to `text/html` would break the \
         preview-description path that keys on `text/plain` and would \
         cause the renderer to attempt image/HTML decoding on a \
         plain-text payload."
    );
    assert!(
        VIEW.contains("self.pasted_text_tokens.push(token.clone())"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must push the \
         fresh token onto `self.pasted_text_tokens` so subsequent \
         pastes see it in `existing_tokens` and pick a non-colliding \
         label. Forgetting this would produce `Pasted text #1` over \
         and over for every paste in a session."
    );
    assert!(
        VIEW.contains("self.typed_mention_aliases.insert(token.token, part)"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must register \
         the token in `self.typed_mention_aliases` via \
         `insert(token.token, part)`. Without this, the token's pill \
         renders correctly but `expand_typed_tokens_for_submit` can't \
         resolve the alias back to the TextBlock on submit — the \
         agent would see the literal `@text:\"Pasted text #1\"` \
         string instead of the pasted content."
    );
    assert!(
        VIEW.contains("self.sync_inline_mentions(cx)"),
        "src/ai/acp/view.rs `paste_text_from_clipboard` must call \
         `self.sync_inline_mentions(cx)` after inserting (and after \
         the token-registration branch). This is the function that \
         re-parses inline mentions and re-renders pills; skipping it \
         would leave a newly-pasted `@text:` token rendered as raw \
         text until the next keystroke."
    );
}

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
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

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn prepare_pasted_text_is_pure_and_respects_existing_tokens() {
    // `prepare_pasted_text` is the bridge between the paste handler
    // and the token registry. It must accept `&[PastedTextToken]` by
    // reference (not consume it) so the caller can keep the registry
    // intact; and it must compute the next token index from the
    // passed registry (not a hidden global) so multiple ACP views
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
         state implicitly and couple the three ACP hosts' paste \
         registries together."
    );
    assert!(
        PASTED_TEXT.contains("next_token_index(existing_tokens)"),
        "src/pasted_text.rs `prepare_pasted_text` must compute the \
         next index via `next_token_index(existing_tokens)` on the \
         passed-in slice. Using a `static AtomicUsize` counter would \
         make paste numbering process-global instead of per-view, \
         colliding across embedded + detached + notes-hosted ACP."
    );
}
