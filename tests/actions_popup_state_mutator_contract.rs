//! Source-level contract for shared actions popup state mutators.
//!
//! The raw `show_actions_popup` flag, `actions_dialog`, and `actions_closed_at`
//! debounce timestamp should be mutated through named helpers on
//! `ScriptListApp` for shared actions popup paths. This keeps the footer Cmd+K
//! debounce coupled to popup open/close state instead of relying on repeated
//! paired field writes.

const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
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
fn builtin_actions_open_close_paths_use_state_mutators() {
    assert!(
        BUILTIN_ACTIONS.contains("self.one_window_stage_in_view(")
            && BUILTIN_ACTIONS.contains("self.close_actions_popup("),
        "built-in actions toggles must open and close through the shared staging and close paths"
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
        if name.ends_with("_tests.rs") || name.contains("/tests/") {
            continue;
        }
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
