use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("current dir"))
}

fn read(path: impl AsRef<Path>) -> String {
    let path = repo_root().join(path);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

#[test]
fn notes_recent_switcher_has_dedicated_taller_popup_contract() {
    let constants = read("src/actions/constants.rs");
    let types = read("src/actions/types/action_model.rs");
    let command_bar = read("src/actions/command_bar.rs");
    let actions_window = read("src/actions/window.rs");
    let actions_dialog = read("src/actions/dialog.rs");
    let notes_init = read("src/notes/window/init.rs");

    assert!(
        constants.contains("NOTES_RECENT_POPUP_MAX_HEIGHT")
            && constants.contains("NOTES_RECENT_POPUP_MAX_HEIGHT: f32 = 560.0"),
        "Notes recent switcher needs a named taller popup max height so Cmd+P can reveal more Recent rows"
    );
    assert!(
        types.contains("pub max_height: f32") && types.contains("max_height: POPUP_MAX_HEIGHT"),
        "ActionsDialogConfig must carry a per-popup max_height defaulting to POPUP_MAX_HEIGHT"
    );
    assert!(
        command_bar.contains("pub fn notes_recent_style() -> Self")
            && command_bar.contains("max_height: NOTES_RECENT_POPUP_MAX_HEIGHT"),
        "CommandBarConfig must expose a Notes Recent style with the taller popup cap"
    );
    assert!(
        notes_init.contains("CommandBarConfig::notes_recent_style()"),
        "Notes Cmd+P note_switcher must use notes_recent_style(), leaving Cmd+K on notes_style()"
    );
    assert!(
        actions_window.contains("max_height: f32")
            && actions_window.contains("dialog.config.max_height")
            && actions_window
                .contains(".min(max_height - search_box_height - header_height - footer_height)"),
        "actions window open/resize height computation must honor dialog.config.max_height"
    );
    assert!(
        actions_dialog.contains("actions_dialog_scrollbar_viewport_height(")
            && actions_dialog.contains("max_height: f32")
            && actions_dialog
                .contains("(max_height - search_height - header_height - footer_height).max(0.0)")
            && actions_dialog.contains(
                ".min(self.config.max_height - search_box_height - header_height - footer_height)"
            ),
        "dialog render and scrollbar viewport must honor the same per-popup max_height"
    );
}
