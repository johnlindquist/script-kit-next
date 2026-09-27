//! The shared AI recovery card must never paint a button.
//!
//! # Why this is a source audit
//!
//! This is the fifth rung of the enforcement ladder, and it is here because
//! the four above it cannot reach the invariant:
//!
//! - The **compiler** cannot forbid calling `Button::new` inside one function.
//! - A **lint** (`disallowed-methods`) is crate-wide; `Button::new` is correct
//!   almost everywhere else, so banning it globally is wrong.
//! - A **behavior test** cannot see the rendered element tree. GPUI gives no
//!   way to assert "this element subtree contains no button".
//! - A **runtime probe** could in principle see it, but the Agent Chat probe
//!   family is currently dead (its fixture was removed in `401936c41`), and a
//!   probe cannot cover the layouts that only appear on a rare failure.
//!
//! # What it protects
//!
//! Every AI surface used to render recovery actions as loose buttons in the
//! middle of the window. This app has exactly two sanctioned homes for a
//! button: a modal, for a decision you must make, or the floating footer /
//! actions menu for everything else. A card in the middle of a conversation is
//! neither.
//!
//! The failure mode this guards against is quiet: someone adds "just one
//! button" back to the card, it looks fine, every test stays green, and the
//! surfaces drift apart again. If this test fails, the question to ask is not
//! "how do I make it pass" but "where should this action actually live" —
//! `src/ai/reliability/placement.rs` answers that.

use super::function_body;
use super::read_source as read;

const CARD: &str = "render_ai_recovery_card";

#[test]
fn the_recovery_card_paints_no_buttons() {
    let source = read("src/components/ai_recovery.rs");
    let body = function_body(&source, CARD)
        .unwrap_or_else(|| panic!("{CARD} must exist in src/components/ai_recovery.rs"));

    for forbidden in ["Button::new", "ButtonVariant::", "on_click("] {
        assert!(
            !body.contains(forbidden),
            "{CARD} contains `{forbidden}`. The recovery card is a message; its \
             actions belong in the footer or the actions menu. Route the action \
             through plan_recovery_presentation instead of painting it here."
        );
    }
}

#[test]
fn the_card_no_longer_takes_action_handlers() {
    // A handler parameter is the seam a button grows back through. Removing it
    // makes the invariant structural rather than stylistic: you cannot wire a
    // click in a function that never receives a click handler.
    let source = read("src/components/ai_recovery.rs");
    let body = function_body(&source, CARD)
        .unwrap_or_else(|| panic!("{CARD} must exist in src/components/ai_recovery.rs"));

    assert!(
        !body.contains("handlers"),
        "{CARD} still references `handlers`. It renders a message only, so it \
         must not be able to reach an action handler at all."
    );
}
