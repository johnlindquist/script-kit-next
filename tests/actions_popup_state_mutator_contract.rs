//! Source-level contract for shared actions popup state mutators.
//!
//! The raw `show_actions_popup` flag, `actions_dialog`, and `actions_closed_at`
//! debounce timestamp should be mutated through named helpers on
//! `ScriptListApp` for shared actions popup paths. This keeps the footer Cmd+K
//! debounce coupled to popup open/close state instead of relying on repeated
//! paired field writes.

const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const BUILTIN_ACTIONS: &str = include_str!("../src/render_builtins/actions.rs");

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

fn rust_sources_under(dir: &std::path::Path) -> Vec<(String, String)> {
    fn walk(path: &std::path::Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", path.display());
        }) {
            let entry = entry.expect("directory entry must be readable");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                let rel = path
                    .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                    .expect("source path must live under manifest dir")
                    .display()
                    .to_string();
                let source = std::fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("failed to read {rel}: {err}"));
                out.push((rel, source));
            }
        }
    }

    let mut out = Vec::new();
    walk(dir, &mut out);
    out
}

#[test]
fn actions_popup_state_mutators_own_open_and_close_field_writes() {
    let opening_body = source_between(
        ACTIONS_DIALOG,
        "pub(crate) fn mark_actions_popup_opening(&mut self)",
        "/// Clear shared actions popup state without recording a recent-close debounce.",
    );
    assert!(
        opening_body.contains("self.show_actions_popup = true;")
            && opening_body.contains("self.actions_closed_at = None; // Clear debounce on open"),
        "mark_actions_popup_opening must own popup-open state and debounce reset"
    );

    let clear_body = source_between(
        ACTIONS_DIALOG,
        "pub(crate) fn clear_actions_popup_state(&mut self)",
        "/// Mark the shared actions popup as closed.",
    );
    assert!(
        clear_body.contains("self.show_actions_popup = false;")
            && clear_body.contains("self.actions_dialog = None;"),
        "clear_actions_popup_state must own non-debounced popup cleanup"
    );

    let closed_body = source_between(
        ACTIONS_DIALOG,
        "pub(crate) fn mark_actions_popup_closed(&mut self)",
        "/// Close the actions popup and restore focus based on host type.",
    );
    assert!(
        closed_body.contains("self.clear_actions_popup_state();")
            && closed_body.contains(
                "self.actions_closed_at = Some(std::time::Instant::now()); // Record debounce on close",
            ),
        "mark_actions_popup_closed must own popup-close state and recent-close timestamp"
    );
}

#[test]
fn canonical_actions_toggle_open_uses_state_mutator() {
    let begin_body = source_between(
        ACTIONS_TOGGLE,
        "fn begin_actions_popup_window_open(&mut self",
        "fn actions_dialog_host_for_current_view",
    );
    assert!(
        begin_body.contains("self.mark_actions_popup_opening();"),
        "begin_actions_popup_window_open must open through mark_actions_popup_opening"
    );
    assert!(
        !begin_body.contains("self.actions_closed_at = None"),
        "begin_actions_popup_window_open must not reset debounce with a raw field write"
    );
}

#[test]
fn shared_close_paths_use_state_mutator() {
    let close_body = source_between(
        ACTIONS_DIALOG,
        "pub(crate) fn close_actions_popup(",
        "pub(crate) fn close_actions_popup_for_current_view(",
    );
    assert!(
        close_body.contains("self.mark_actions_popup_closed();"),
        "close_actions_popup must close through mark_actions_popup_closed"
    );
    assert!(
        !close_body.contains("self.actions_closed_at = Some"),
        "close_actions_popup must not record debounce with a raw field write"
    );

    let fallback_body = source_between(
        ACTIONS_DIALOG,
        "pub(crate) fn close_actions_popup_for_current_view(",
        "}\n\n#[cfg(test)]",
    );
    assert!(
        fallback_body.contains("self.mark_actions_popup_closed();"),
        "close_actions_popup_for_current_view fallback must close through mark_actions_popup_closed"
    );
}

#[test]
fn native_actions_close_focus_loss_path_uses_close_state_mutator() {
    let callback_body = source_between(
        ACTIONS_TOGGLE,
        "pub(crate) fn make_actions_window_on_close_callback(",
        "pub(crate) fn spawn_open_actions_window(",
    );
    assert!(
        callback_body.contains("app.mark_actions_popup_closed();"),
        "native actions on_close must close through mark_actions_popup_closed"
    );
    assert!(
        callback_body.contains("app.hide_main_window_preserving_state_for_focus_loss(cx);"),
        "MainList focus-loss branch should preserve the host ScriptList state"
    );
    assert!(
        !callback_body.contains("app.show_actions_popup = false")
            && !callback_body.contains("app.actions_dialog = None")
            && !callback_body.contains("app.actions_closed_at = Some"),
        "native actions on_close must not bypass shared close state mutators"
    );
}

#[test]
fn builtin_actions_open_close_paths_use_state_mutators() {
    assert!(
        BUILTIN_ACTIONS.contains("self.mark_actions_popup_opening();")
            && BUILTIN_ACTIONS.contains("self.mark_actions_popup_closed();"),
        "built-in actions toggles must use shared state mutators"
    );
    assert!(
        !BUILTIN_ACTIONS.contains("self.actions_closed_at = None")
            && !BUILTIN_ACTIONS.contains("self.actions_closed_at = Some"),
        "built-in actions toggles must not mutate debounce timestamp directly"
    );
}

#[test]
fn production_popup_state_field_writes_stay_inside_mutators() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, source) in rust_sources_under(&src_dir) {
        let outside_mutators = if name == "src/app_impl/actions_dialog.rs" {
            source_between(
                &source,
                "/// Close the actions popup and restore focus",
                "#[cfg(test)]",
            )
        } else {
            &source
        };
        for forbidden in [
            "show_actions_popup = true",
            "show_actions_popup = false",
            "actions_closed_at = None",
            "actions_closed_at = Some",
            "actions_dialog = None",
        ] {
            assert!(
                !outside_mutators.contains(forbidden),
                "{name} must not write `{forbidden}` outside the named actions popup state mutators"
            );
        }
    }
}
