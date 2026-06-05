//! Source-level contract pinning the 300ms `actions_closed_at` debounce
//! at `src/app_impl/actions_dialog.rs::was_actions_recently_closed` —
//! the single guard that stops Cmd+K from double-firing when an
//! activation-triggered close races with the toggle handler.
//!
//! The shape:
//!
//! ```ignore
//! /// Check if the actions popup was closed very recently (within 300ms).
//! ///
//! /// This guards against a race where clicking the footer ⌘K button causes
//! /// the actions window's activation observer to close the dialog (deferred)
//! /// before the click handler fires `toggle_actions`. Without this debounce
//! /// the toggle would see the dialog as closed and immediately reopen it.
//! pub(crate) fn was_actions_recently_closed(&self) -> bool {
//!     const ACTIONS_CLOSE_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(300);
//!     self.actions_closed_at
//!         .map(|t| t.elapsed() < ACTIONS_CLOSE_DEBOUNCE)
//!         .unwrap_or(false)
//! }
//! ```
//!
//! Why this Pin exists. The debounce is called from
//! `src/app_impl/actions_toggle.rs::toggle_actions` (line 479) and
//! `src/app_impl/actions_toggle.rs` (line 738 — the secondary entry)
//! BEFORE the `if self.show_actions_popup || is_actions_window_open()`
//! branch decides OPEN vs CLOSE. Every Cmd+K on every host flows
//! through this check. The Run 9 Pass #32 attacker probe produced
//! receipts confirming this discipline holds under rapid-fire
//! (4 cycles in 216ms: 3 of 4 cmd+k events logged
//! `actions_toggle_suppressed_recent_close`), but no source-level
//! contract protects the five load-bearing details:
//!
//!   1. **Signature stability** — callers (the two actions_toggle
//!      sites + anything else that reads `recently_closed`) rely on
//!      `(&self) -> bool` with no ambient context (no `cx`, no
//!      `window`). If a future "clean up" changes the signature to
//!      `(&self, cx: &App) -> bool` or inverts the return ("not
//!      recently closed"), the two callers either fail to compile or
//!      silently invert.
//!
//!   2. **300ms window literal** — `std::time::Duration::from_millis(300)`
//!      is the keyframe. Tightening to 50ms re-opens the mouseDown
//!      race. Loosening to 3s blocks legitimate reopens after a user
//!      intentionally closes then re-opens the dialog. Either drift
//!      is a user-visible regression, but the value looks arbitrary
//!      to a reader without the rationale.
//!
//!   3. **`<` (strict less-than) comparator** — `elapsed() <
//!      ACTIONS_CLOSE_DEBOUNCE`. A "cleanup" that changes to `<=`
//!      is almost certainly harmless (1-nanosecond edge case), but
//!      a flip to `>` (or swapping operands to
//!      `ACTIONS_CLOSE_DEBOUNCE < elapsed()`) inverts the guard —
//!      the Cmd+K button would be suppressed WHENEVER the debounce
//!      window has EXPIRED, which is the opposite of intended.
//!      Short of property testing with wall-clock, this source-level
//!      comparator check is the cheapest way to catch the inversion.
//!
//!   4. **`self.actions_closed_at` field read** — the function
//!      reads the `Option<std::time::Instant>` field at
//!      `src/main_sections/app_state.rs:160`. A refactor that
//!      replaces the field with a typed wrapper (e.g.
//!      `ActionsCloseDebouncer` struct with its own `.is_active()`)
//!      could leave the function body calling `self.actions_closed_at`
//!      on a removed field, OR could silently swap the name
//!      (`actions_closed_ts`, `actions_close_instant`) and a
//!      hand-edit of callers would miss the rename at compile time
//!      only if the call site guards are retained — but the
//!      source-level pin catches the name drift immediately.
//!
//!   5. **Anchor-comment phrases** — the phrases
//!      `300ms`, `footer ⌘K button`, and `activation observer` carry
//!      the "why". The classic silent-cleanup shape deletes the
//!      comment first (looks like obvious doc noise), then a
//!      subsequent contributor reads the bare
//!      `std::time::Duration::from_millis(300)` literal without
//!      context and "simplifies" it to 100ms or 1000ms because
//!      either feels more principled. Pinning the three phrases
//!      anchors the reasoning to the code.
//!
//! Adjacent defended invariants already pinned elsewhere:
//!   - `src/ai/acp/tests.rs::acp_history_toggle_uses_recent_close_debounce`
//!     (line 711) pins the SIBLING ACP-history debounce (same
//!     mechanism, different host) — the existence of that pin for
//!     the sibling makes the actions-dialog pin even more urgent,
//!     because the ACP-history variant could drift in isolation
//!     without this one catching the drift on the primary site.
//!
//! **Refactor threat** (must be named per looper/rules/discipline.md):
//! "A contributor extracts `was_actions_recently_closed` + its
//! `actions_closed_at` field + the 5 close-path setters
//! (`src/app_impl/actions_dialog.rs:653, 730`,
//! `src/app_impl/actions_toggle.rs:174`,
//! `src/app_execute/builtin_execution.rs:2362, 2488`) and the
//! 2 open-path clearers (`src/app_impl/actions_toggle.rs:256`,
//! `src/render_builtins/actions.rs:82`) into a shared
//! `RecentCloseDebouncer` helper reused by the actions dialog AND
//! the ACP-history debounce (sibling at
//! `src/ai/acp/view.rs::handle_cmd_p_acp_history_toggle`). The
//! consolidation naturally parameterizes the 300ms window; a
//! "clean" default of 100ms or 500ms lands silently and nobody
//! notices in review because the raw-literal debounce still
//! functions — just with a different window." Deleting the anchor
//! comment during that extraction is the next move because its
//! specific "footer ⌘K button" framing doesn't generalize across
//! both users of the helper. This Pin defends against BOTH halves
//! (window drift + comment drift).

const SOURCE: &str = include_str!("../src/app_impl/actions_dialog.rs");
const FN_SIGNATURE: &str = "pub(crate) fn was_actions_recently_closed(&self) -> bool";
const DEBOUNCE_LITERAL: &str = "std::time::Duration::from_millis(300)";
const COMPARATOR_EXPR: &str = "t.elapsed() < ACTIONS_CLOSE_DEBOUNCE";
const FIELD_READ: &str = "self.actions_closed_at";
const ANCHOR_300MS: &str = "300ms";
const ANCHOR_FOOTER: &str = "footer ⌘K button";
const ANCHOR_ACTIVATION: &str = "activation observer";

/// Extract the body of `was_actions_recently_closed` — everything between the
/// `{` opening the function and the matching `}`. Panics on malformed source.
fn extract_function_body(source: &str) -> &str {
    let fn_start = source
        .find(FN_SIGNATURE)
        .expect("was_actions_recently_closed function not found at source level");
    let body_open = source[fn_start..]
        .find('{')
        .expect("opening brace of was_actions_recently_closed not found")
        + fn_start;
    let body_bytes = source.as_bytes();
    let mut depth: i32 = 0;
    let mut i = body_open;
    while i < body_bytes.len() {
        match body_bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_open + 1..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    panic!("no matching closing brace found for was_actions_recently_closed body");
}

/// Extract the ~20 lines ABOVE the function signature — the doc-comment block
/// that carries the rationale. Returns up to 800 bytes before the signature.
fn extract_doc_comment(source: &str) -> &str {
    let fn_start = source
        .find(FN_SIGNATURE)
        .expect("was_actions_recently_closed function not found at source level");
    let lookback = fn_start.saturating_sub(800);
    &source[lookback..fn_start]
}

#[test]
fn was_actions_recently_closed_exists_with_exact_signature() {
    let hits: Vec<_> = SOURCE.match_indices(FN_SIGNATURE).collect();
    assert_eq!(
        hits.len(),
        1,
        "Expected exactly one `{FN_SIGNATURE}` definition in \
         src/app_impl/actions_dialog.rs (found {}). If the signature \
         changed — for example to take a `cx: &App` parameter, to \
         return `Option<Duration>` instead of `bool`, or to invert \
         the sense (`was_actions_NOT_recently_closed`) — the two \
         callers at src/app_impl/actions_toggle.rs:479 and :738 \
         either fail to compile (signature break) OR silently invert \
         their branch (sense break). The `bool` return + no-ambient-\
         context signature is load-bearing for both call sites.",
        hits.len()
    );
}

#[test]
fn was_actions_recently_closed_pins_300ms_debounce_window() {
    let body = extract_function_body(SOURCE);
    assert!(
        body.contains(DEBOUNCE_LITERAL),
        "`was_actions_recently_closed` body MUST contain the literal \
         `{DEBOUNCE_LITERAL}` — the 300ms window is the keyframe of \
         the footer-click-vs-activation-observer race guard. A \
         refactor that parameterizes the window (e.g. a generic \
         `RecentCloseDebouncer<const MS: u64>`) and threads a \
         different default (100ms, 500ms, 1s) silently changes the \
         observable Cmd+K behavior. Tightening to <200ms reopens \
         the mouseDown-vs-click race; loosening to >500ms blocks \
         legitimate fast reopens. Body follows:\n{body}"
    );
}

#[test]
fn was_actions_recently_closed_uses_strict_less_than_comparator() {
    let body = extract_function_body(SOURCE);
    assert!(
        body.contains(COMPARATOR_EXPR),
        "`was_actions_recently_closed` body MUST contain the exact \
         expression `{COMPARATOR_EXPR}` — `<` strictly, with \
         `t.elapsed()` on the LEFT and `ACTIONS_CLOSE_DEBOUNCE` on \
         the RIGHT. A flip to `>` or to `ACTIONS_CLOSE_DEBOUNCE < \
         t.elapsed()` inverts the guard semantics: the debounce \
         would fire OUTSIDE the 300ms window instead of INSIDE it, \
         suppressing all intended reopens. The `<=` variant would \
         be harmless in practice (1-nanosecond edge) but a clippy \
         autofix that flips operator associativity could mask a \
         semantic change, so pin the exact expression. Body \
         follows:\n{body}"
    );
    assert!(
        body.contains(FIELD_READ),
        "`was_actions_recently_closed` body MUST read \
         `{FIELD_READ}` — the `Option<std::time::Instant>` field \
         at src/main_sections/app_state.rs:160 is the debounce \
         state. A refactor that renames the field (e.g. \
         `actions_closed_ts`) or wraps it in a `Debouncer` struct \
         without updating this read reintroduces the race. Body \
         follows:\n{body}"
    );
}

#[test]
fn was_actions_recently_closed_anchor_comment_carries_rationale() {
    let doc = extract_doc_comment(SOURCE);
    for phrase in [ANCHOR_300MS, ANCHOR_FOOTER, ANCHOR_ACTIVATION] {
        assert!(
            doc.contains(phrase),
            "The doc-comment block above `was_actions_recently_closed` \
             MUST contain the phrase `{phrase}` — one of three \
             load-bearing rationale anchors. Together the three \
             phrases answer: (a) what window is this? (`300ms`), \
             (b) which click path does it defend? (`footer ⌘K \
             button`), (c) what observer is racing? (`activation \
             observer`). The classic silent-cleanup refactor shape \
             deletes the comment first — it looks like obvious \
             doc noise — then a subsequent contributor reads the \
             bare `Duration::from_millis(300)` literal with no \
             context and 'simplifies' it. Pinning the three phrases \
             anchors the reasoning to the code. Last 800 bytes \
             above the signature follow:\n{doc}"
        );
    }
}
