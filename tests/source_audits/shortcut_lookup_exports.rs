// Source-audit regression: ensure src/actions/mod.rs re-exports the
// window-specific shortcut-lookup helpers so AI and Notes windows can
// resolve keystrokes via `crate::actions::*`.

use super::read_source as read;

#[test]
fn test_actions_mod_re_exports_ai_window_shortcut_lookup() {
    let source = read("src/actions/mod.rs");
    assert!(
        source.contains("ai_window_action_id_for_keystroke"),
        "src/actions/mod.rs must re-export ai_window_action_id_for_keystroke \
         so AI window shortcut dispatch compiles"
    );
}

#[test]
fn test_actions_mod_re_exports_notes_window_shortcut_lookup() {
    let source = read("src/actions/mod.rs");
    assert!(
        source.contains("notes_window_action_id_for_keystroke"),
        "src/actions/mod.rs must re-export notes_window_action_id_for_keystroke \
         so Notes window shortcut dispatch compiles"
    );
}
