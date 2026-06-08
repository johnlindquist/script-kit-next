use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn confirm_modal_shell_is_registered_as_shared_component() {
    let components_mod = read("src/components/mod.rs");
    let shell = read("src/components/confirm_modal_shell.rs");

    assert!(
        components_mod.contains("pub(crate) mod confirm_modal_shell;"),
        "shared confirm modal shell must be registered in src/components/mod.rs"
    );
    assert!(
        components_mod.contains("confirm_modal_shell")
            && components_mod.contains("ConfirmModalShellConfig")
            && components_mod.contains("CONFIRM_MODAL_SHELL_ID"),
        "shared shell exports must be available to modal routes"
    );
    assert!(
        shell.contains("CONFIRM_MODAL_SHELL_ID")
            && shell.contains("\"modal-shell:confirm\"")
            && shell.contains("confirm_modal_header")
            && shell.contains("confirm_modal_shell("),
        "shared shell must own the common confirm modal marker, header, and panel builder"
    );
}

#[test]
fn shortcut_recorder_and_confirm_popup_use_the_same_shell() {
    let shortcut = read("src/components/shortcut_recorder/render.rs");
    let confirm = read("src/confirm/window.rs");

    for (label, source, content_id) in [
        (
            "shortcut recorder",
            shortcut.as_str(),
            "shortcut-modal-content",
        ),
        ("confirm popup", confirm.as_str(), "confirm-modal-content"),
    ] {
        assert!(
            source.contains("confirm_modal_shell(")
                && source.contains("ConfirmModalShellConfig")
                && source.contains(content_id),
            "{label} must render through the shared confirm modal shell while preserving route content id {content_id}"
        );
        assert!(
            source.contains("confirm_modal_header("),
            "{label} must use the shared confirm modal header"
        );
    }
}

#[test]
fn confirm_modal_inventory_tracks_routes_and_excludes_non_modals() {
    let inventory = read(".goals/receipts/modal-inventory.md");

    for route in [
        "Add Shortcut / shortcut recorder",
        "Quit Script Kit",
        "Remove/delete script",
        "Move file to trash",
        "Clipboard bulk delete / clear unpinned",
        "SDK `confirm`",
        "`openConfirmPrompt` stdin fixture",
        "`showShortcutRecorder` stdin fixture",
        "Notes delete confirmation",
    ] {
        assert!(
            inventory.contains(route),
            "modal inventory must track confirm route: {route}"
        );
    }

    for excluded in [
        "Actions Menu",
        "Trigger popup",
        "Hover/dropdown menus",
        "Browse panels",
        "Editor choice popup",
    ] {
        assert!(
            inventory.contains(excluded),
            "modal inventory must explicitly exclude non-modal surface: {excluded}"
        );
    }
}

#[test]
fn actions_and_trigger_popups_are_not_classified_as_confirm_modals() {
    let inventory = read(".goals/receipts/modal-inventory.md");

    assert!(
        inventory.contains("Actions Menu")
            && inventory.contains("not a confirm/deny modal")
            && inventory.contains("Trigger popup"),
        "actions menu and trigger popup must stay out of the confirm modal migration"
    );
}
