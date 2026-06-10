use std::fs;

/// The dev style tool sliders must never re-layout the control tree while a drag
/// is in flight. Three pieces cooperate: the vendored slider freezes its bounds for
/// the drag lifetime, the tool splits live Change events (input-text-only sync, no
/// whole-view notify) from Release commits (full sync), and external syncs never
/// reconcile a slider that owns an active drag.
#[test]
fn slider_drag_uses_bounds_snapshot_in_vendored_slider() {
    let source = fs::read_to_string("vendor/gpui-component/crates/ui/src/slider.rs")
        .expect("read vendored slider");

    assert!(
        source.contains("drag_bounds: Bounds<Pixels>"),
        "SliderState must keep a bounds snapshot for the drag lifetime"
    );
    assert!(
        source.contains("self.drag_bounds = self.bounds;"),
        "drag bounds must be snapshotted on the drag start transition"
    );
    assert!(
        source.contains("let bounds = self.drag_bounds;"),
        "drag value math must use the snapshot, not live (re-layout-prone) bounds"
    );
}

/// The four knob families and the live/commit apply pair each family's slider
/// subscription must route through.
const KNOB_FAMILIES: [(&str, &str, &str); 4] = [
    (
        "main",
        "this.apply_knob_value_live(",
        "this.apply_knob_value(",
    ),
    (
        "actions popup",
        "this.apply_actions_popup_knob_value_live(",
        "this.apply_actions_popup_knob_value(",
    ),
    (
        "agent chat",
        "this.apply_agent_chat_knob_value_live(",
        "this.apply_agent_chat_knob_value(",
    ),
    (
        "confirm modal",
        "this.apply_confirm_modal_knob_value_live(",
        "this.apply_confirm_modal_knob_value(",
    ),
];

#[test]
fn dev_style_tool_splits_live_change_from_release_commit() {
    let source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style tool render");
    let new_body = function_body(&source, "pub(crate) fn new(");

    for (family, live_call, commit_call) in KNOB_FAMILIES {
        let live_index = new_body.find(live_call).unwrap_or_else(|| {
            panic!("{family} knob family must route Change through the live path ({live_call})")
        });
        assert!(
            new_body[..live_index]
                .trim_end()
                .ends_with("SliderEvent::Change(value) => {"),
            "{family} live apply must be invoked from the SliderEvent::Change arm"
        );
        let after_live = &new_body[live_index + live_call.len()..];
        let commit_index = after_live.find(commit_call).unwrap_or_else(|| {
            panic!("{family} knob family must commit on Release ({commit_call})")
        });
        assert!(
            after_live[..commit_index]
                .trim_end()
                .ends_with("SliderEvent::Release(value) => {"),
            "{family} commit apply must be invoked from the SliderEvent::Release arm"
        );
    }
    assert!(
        source.contains("fn apply_knob_value_live(")
            && source.contains("fn apply_actions_popup_knob_value_live(")
            && source.contains("fn apply_agent_chat_knob_value_live(")
            && source.contains("fn apply_confirm_modal_knob_value_live("),
        "each knob family must have a live apply variant"
    );
    assert!(
        source.contains("fn refresh_main_window_throttled("),
        "live drag ticks must throttle main window refreshes"
    );
    assert!(
        source.contains("fn sync_live_input_text("),
        "live path must sync the paired input entity only"
    );
    assert!(
        !function_body(&source, "fn sync_live_input_text(").contains("cx.notify()"),
        "live input sync must not notify the whole tool view mid-drag"
    );
}

#[test]
fn dev_style_tool_never_reconciles_a_dragging_slider() {
    let source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style tool render");

    for sync_fn in [
        "fn sync_control_to_value(",
        "fn sync_actions_popup_control_to_value(",
        "fn sync_agent_chat_control_to_value(",
        "fn sync_confirm_modal_control_to_value(",
    ] {
        assert!(
            function_body(&source, sync_fn)
                .contains("if !slider.is_dragging() && (slider.value().end() - value).abs()"),
            "{sync_fn} must skip slider reconciliation while a drag owns the slider"
        );
    }
}

#[test]
fn dev_style_tool_value_readouts_are_fixed_width() {
    let source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style tool render");

    assert!(
        source.contains("const VALUE_READOUT_COL_W: f32"),
        "value readout column width must be a shared constant"
    );
    for control_fn in [
        "fn render_control(",
        "fn render_actions_popup_control(",
        "fn render_agent_chat_control(",
        "fn render_confirm_modal_control(",
    ] {
        assert!(
            function_body(&source, control_fn).contains(".w(px(VALUE_READOUT_COL_W))"),
            "{control_fn} must render the value readout in the fixed-width column"
        );
    }
}

#[test]
fn dev_style_tool_exposes_cross_group_knob_filter_and_anatomy_hints() {
    let source =
        fs::read_to_string("src/dev_style_tool/render.rs").expect("read dev style tool render");

    assert!(
        source.contains("input:dev-style-tool-knob-filter")
            && source.contains("button:dev-style-tool-knob-filter-clear")
            && source.contains("status:dev-style-tool-filter-empty"),
        "knob filter input, clear button, and empty state must keep stable semantic ids"
    );
    assert!(
        source.contains("fn knob_matches_filter("),
        "knob filtering must match on label and id"
    );
    for group_fn in [
        "fn render_group(",
        "fn render_actions_popup_group(",
        "fn render_agent_chat_group(",
        "fn render_confirm_modal_group(",
    ] {
        assert!(
            function_body(&source, group_fn).contains("group.description()"),
            "{group_fn} must show the anatomy description under the header"
        );
    }
    for control_fn in [
        "fn render_control(",
        "fn render_actions_popup_control(",
        "fn render_agent_chat_control(",
        "fn render_confirm_modal_control(",
    ] {
        assert!(
            function_body(&source, control_fn).contains("rgba(chrome.accent_badge_border_rgba)"),
            "{control_fn} must tint the card border when a knob is overridden"
        );
    }
}

fn function_body(source: &str, signature: &str) -> String {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} not found"));
    let rest = &source[start..];
    let open = rest.find('{').expect("function body open brace");
    let mut depth = 0usize;
    for (index, ch) in rest[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return rest[open..open + index + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body for {signature}");
}
