//! Source-level contract: the actions dialog carries the host context it was
//! opened for (parent automation id, semantic surface, subject, selected row)
//! as an immutable snapshot, so DevTools receipts never infer the parent from
//! focus.

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
fn actions_open_paths_capture_host_context_before_staging() {
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
        "            self.one_window_stage_in_view(host, dialog, title, window, cx);",
    );
    assert!(
        generic_open.contains("build_actions_host_context_snapshot(host"),
        "toggle_actions must build a host context snapshot before staging the dialog"
    );
    assert!(
        generic_open.contains("d.set_host_context(host_context);"),
        "toggle_actions must install the snapshot on the dialog before staging it"
    );
}
