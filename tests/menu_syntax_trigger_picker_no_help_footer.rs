//! Integration test for Run 12: advanced-query trigger popup must not
//! emit a generic "Open Menu Syntax help" footer row.
//!
//! Runs as a separate integration test binary so it links the rlib
//! without re-triggering the proc-macro recursion that currently
//! breaks `cargo test --lib` in some local environments.

use script_kit_gpui::menu_syntax::trigger_picker::TriggerPickerAction;
use script_kit_gpui::menu_syntax::{build_trigger_picker_snapshot, TriggerPickerContext};

fn empty_ctx() -> TriggerPickerContext {
    TriggerPickerContext::default()
}

#[test]
fn advanced_query_popup_has_no_help_footer_by_default() {
    let snap = build_trigger_picker_snapshot(":", &empty_ctx()).expect("snapshot");
    assert!(
        snap.rows
            .iter()
            .all(|r| r.id != "footer:help" && r.action != TriggerPickerAction::OpenHelp),
        "advanced-query popup must not emit a generic help footer",
    );
}

#[test]
fn colon_has_partial_popup_has_no_help_footer() {
    let snap = build_trigger_picker_snapshot(":has:", &empty_ctx()).expect("snapshot");
    assert!(
        snap.rows.iter().all(|r| r.id != "footer:help"),
        ":has: popup must not emit a help footer",
    );
}
