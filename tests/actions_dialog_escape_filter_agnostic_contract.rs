//! Source-level structural contract: `ActionsDialog::handle_escape`
//! MUST stay filter-text-agnostic.
//!
//! # Why this matters (UX)
//!
//! The shared `ActionsDialog` at `src/actions/dialog.rs` hosts every
//! Cmd+K actions menu in the app — main-menu, clipboard history,
//! emoji picker, file search, app launcher, Agent Chat chat, notes, and
//! any future host registered in `actions_host_for_view` at
//! `src/app_impl/actions_dialog.rs`. A single user-visible invariant
//! spans all of them: pressing Escape closes the dialog in one
//! keystroke, regardless of whether the user has typed filter text.
//!
//! This is the Raycast / Spotlight UX contract. Native macOS alerts
//! and `NSSearchField` do the opposite — Escape first clears the
//! field, then a second Escape closes. Script Kit deliberately
//! chose the Raycast convention so the dialog is a "modal-lite"
//! popup, not a two-step dismissal.
//!
//! A "natural" refactor a future contributor might ship:
//!
//! ```ignore
//! pub fn handle_escape(&mut self, cx: &mut Context<Self>) -> ActionsDialogEscapeOutcome {
//!     if !self.search_text.is_empty() {
//!         self.search_text.clear();
//!         self.refilter_actions(cx);
//!         return ActionsDialogEscapeOutcome::FilterCleared; // new variant
//!     }
//!     // ...existing pop-route-or-close logic...
//! }
//! ```
//!
//! would silently regress EVERY Cmd+K host: users who type a filter
//! and then press Escape expecting the dialog to close would instead
//! see the dialog stay open with an empty filter and have to press
//! Escape a second time. That's a very user-visible regression and
//! one that per-host live probes (Run 7 Pass #10/#11, Run 8 Pass #2/#3)
//! would NOT catch — they exercised escape-from-open on a dialog with
//! empty filter text.
//!
//! # Structural anchors pinned in `handle_escape`'s body
//!
//!   (1) `self.pop_route(cx)` — the back-stack decision call. Removing
//!       this would collapse the `PoppedRoute` path and break route
//!       drill-downs (e.g., Agent Chat agent switch sub-menu at
//!       `src/actions/builders/agent_chat.rs`).
//!   (2) The body MUST NOT contain `self.search_text.clear()` — any
//!       self-clear of the filter inside the escape handler is the
//!       regression this contract exists to block.
//!   (3) The body MUST NOT contain `self.search_text.is_empty()` —
//!       gating escape behavior on filter content is the same class
//!       of regression expressed as a conditional instead of a wipe.
//!   (4) `ActionsDialogEscapeOutcome` MUST have exactly the two
//!       documented variants — `PoppedRoute` and `CloseDialog`. A new
//!       `FilterCleared` / `SearchTextCleared` / similar variant is
//!       the shape the regression would take at the enum level even
//!       if the body-level anchors were preserved by accident.
//!
//! Behavioural tests do NOT cover this: the existing dialog-hosted
//! live probes exercise an empty-filter escape, and the few unit
//! tests in `src/actions/dialog.rs` / `tests/agent_chat_switch_actions.rs`
//! pin the two existing outcomes without asserting the filter-text
//! input is ignored. This file is the structural pin that makes the
//! regression impossible to ship silently.

const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");

fn handle_escape_body() -> &'static str {
    let header = "pub fn handle_escape(";
    let header_pos = DIALOG_SOURCE.find(header).unwrap_or_else(|| {
        panic!(
            "src/actions/dialog.rs: `{header}` not found — the method may have been renamed. \
             Update tests/actions_dialog_escape_filter_agnostic_contract.rs anchors to match."
        )
    });

    let open_rel = DIALOG_SOURCE[header_pos..]
        .find('{')
        .expect("no `{` after `pub fn handle_escape(` header");
    let open_abs = header_pos + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in DIALOG_SOURCE.as_bytes()[open_abs..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_abs = Some(open_abs + offset);
                    break;
                }
            }
            _ => {}
        }
    }

    let close_abs = close_abs.expect("no matching `}` for handle_escape body");
    &DIALOG_SOURCE[open_abs..=close_abs]
}

#[test]
fn handle_escape_delegates_to_pop_route_for_back_navigation() {
    let body = handle_escape_body();
    assert!(
        body.contains("self.pop_route(cx)"),
        "handle_escape body must call `self.pop_route(cx)` to decide \
         between PoppedRoute and CloseDialog. \
         Dropping it would collapse back-navigation and close the \
         dialog on every Escape, breaking drill-down routes \
         (Agent Chat agent switch, etc.). Body was:\n{body}"
    );
}

#[test]
fn handle_escape_does_not_clear_search_text() {
    let body = handle_escape_body();
    assert!(
        !body.contains("self.search_text.clear()"),
        "handle_escape MUST NOT call `self.search_text.clear()` — \
         clearing the filter inside the escape handler means users \
         must press Escape twice to dismiss the dialog after typing, \
         which regresses every Cmd+K host from the shared Raycast UX \
         to a macOS-NSSearchField two-step dismissal. Body was:\n{body}"
    );
}

#[test]
fn handle_escape_does_not_gate_on_filter_text_emptiness() {
    let body = handle_escape_body();
    assert!(
        !body.contains("self.search_text.is_empty()"),
        "handle_escape MUST NOT branch on `self.search_text.is_empty()` — \
         any conditional that diverges escape behavior based on filter \
         content is the same regression as calling `search_text.clear()` \
         inline (expressed as a guard instead of a wipe). \
         Escape is filter-text-agnostic by contract. Body was:\n{body}"
    );
}

#[test]
fn actions_dialog_escape_outcome_has_exactly_two_variants() {
    let marker = "pub enum ActionsDialogEscapeOutcome {";
    let start = DIALOG_SOURCE.find(marker).expect(
        "pub enum ActionsDialogEscapeOutcome not found in src/actions/dialog.rs — \
         the enum may have been renamed. Update the contract anchors.",
    );
    let after_brace = start + marker.len();
    let end_rel = DIALOG_SOURCE[after_brace..]
        .find('}')
        .expect("no closing `}` for ActionsDialogEscapeOutcome enum");
    let enum_body = &DIALOG_SOURCE[after_brace..after_brace + end_rel];

    assert!(
        enum_body.contains("PoppedRoute"),
        "ActionsDialogEscapeOutcome must declare `PoppedRoute`. Body was:\n{enum_body}"
    );
    assert!(
        enum_body.contains("CloseDialog"),
        "ActionsDialogEscapeOutcome must declare `CloseDialog`. Body was:\n{enum_body}"
    );

    // Count identifier-like lines (strip `//` comments, doc-comments, whitespace).
    // A variant line ends with `,` and is not a comment.
    let variant_count = enum_body
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with('*')
                && trimmed.ends_with(',')
        })
        .count();

    assert_eq!(
        variant_count, 2,
        "ActionsDialogEscapeOutcome must have EXACTLY 2 variants \
         (`PoppedRoute`, `CloseDialog`). A new variant like `FilterCleared` \
         / `SearchTextCleared` / `FilterPopped` is the shape the \
         regression this contract blocks would take at the enum level. \
         Adding a third outcome requires updating this test AND the \
         documentation in `src/actions/dialog.rs` above the enum. \
         Found {variant_count} variants. Body was:\n{enum_body}"
    );
}

#[test]
fn handle_escape_emits_structured_tracing_event() {
    let body = handle_escape_body();
    assert!(
        body.contains("actions_dialog_escape"),
        "handle_escape must emit the `actions_dialog_escape` tracing \
         event so audit receipts can correlate escape → outcome \
         across all Cmd+K hosts. Dropping it would break the log-scrape \
         path that Run 7 Pass #12 (attacker-rapid-toggle) and the \
         actions-cmdk-reopen-idempotent resolution relied on to prove \
         the toggle-closed outcome. Body was:\n{body}"
    );
}
