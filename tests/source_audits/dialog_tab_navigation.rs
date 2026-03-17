//! Regression test: the shared gpui-component Dialog must bind Tab/Shift-Tab
//! focus navigation so that buttons are keyboard-accessible even when the
//! parent view captures Tab for other purposes (e.g. Notes editor indentation).

#[cfg(test)]
mod tests {
    use std::fs;

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn dialog_binds_tab_navigation_actions() {
        let source = fs::read_to_string("vendor/gpui-component/crates/ui/src/dialog.rs")
            .expect("Failed to read vendor/gpui-component/crates/ui/src/dialog.rs");
        let normalized = normalize_ws(&source);

        assert!(
            normalized.contains("actions!(dialog, [FocusNext, FocusPrev]);"),
            "Dialog should define explicit focus navigation actions"
        );
        assert!(
            normalized.contains("KeyBinding::new(\"tab\", FocusNext, Some(CONTEXT))")
                && normalized.contains("KeyBinding::new(\"shift-tab\", FocusPrev, Some(CONTEXT))"),
            "Dialog should bind Tab and Shift-Tab in the Dialog key context"
        );
        assert!(
            normalized.contains(
                ".on_action(|_: &FocusNext, window, cx| { window.focus_next_in_dialog(cx); })"
            ) && normalized.contains(
                ".on_action(|_: &FocusPrev, window, cx| { window.focus_prev_in_dialog(cx); })"
            ),
            "Dialog should move focus between tab stops from inside the dialog context"
        );
    }
}
