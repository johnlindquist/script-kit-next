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
fn destructive_confirm_routes_use_shared_parent_confirm_helpers() {
    let scripts = read("src/app_actions/handle_action/scripts.rs");
    let files = read("src/app_actions/handle_action/files.rs");
    let clipboard = read("src/app_actions/handle_action/clipboard.rs");
    let notes = read("src/notes/window/notes.rs");

    assert!(
        scripts.contains("crate::confirm::open_parent_confirm_dialog_for_entity(")
            && scripts.contains("remove_script_confirmed")
            && scripts.contains("quit_script_kit_confirm_options()"),
        "script removal and quit confirmations must use the entity-owned shared parent confirm helper"
    );
    assert!(
        files.contains("crate::confirm::ParentConfirmOptions::destructive(")
            && files.contains("crate::confirm::confirm_with_parent_dialog(")
            && files.contains("\"Async action cancelled: move_to_trash\""),
        "file move-to-trash confirmation must use the shared parent confirm dialog and handle cancel"
    );
    assert!(
        clipboard.contains("clipboard_delete_multiple")
            && clipboard.contains("clipboard_delete_all")
            && clipboard.contains("crate::confirm::ParentConfirmOptions::destructive(")
            && clipboard.contains("crate::confirm::confirm_with_parent_dialog("),
        "clipboard bulk delete confirmations must use the shared parent confirm dialog"
    );
    assert!(
        notes.contains("crate::confirm::open_parent_confirm_dialog_for_automation_parent(")
            && notes.contains("\"notes\"")
            && notes.contains("notes_delete_cancelled")
            && notes.contains("restore_primary_focus_after_dialog"),
        "notes delete confirmation must use the parent-id-aware shared confirm helper and restore focus on cancel"
    );
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

#[test]
fn sdk_confirm_api_stays_single_word_and_avoids_modal_namespace() {
    let sdk = read("scripts/kit-sdk.ts");
    let goal = read(".goals/modal-consistency-design-system.md");
    let inventory = read(".goals/receipts/modal-inventory.md");

    assert!(
        sdk.contains("export interface ConfirmConfig")
            && sdk.contains("(globalThis as any).confirm = async function confirm")
            && sdk.contains("type: 'confirm'"),
        "SDK confirm exposure must keep the typed global confirm() implementation"
    );
    assert!(
        !sdk.contains("modal.confirm") && !sdk.contains("globalThis.modal"),
        "SDK confirm exposure must not introduce a modal.confirm namespace"
    );
    assert!(
        goal.contains("API as `confirm`") && !goal.contains("modal.confirm"),
        "goal contract must preserve the single-word SDK confirm API"
    );
    assert!(
        inventory.contains("SDK `confirm`"),
        "inventory must track the script-facing SDK confirm route"
    );
}

#[test]
fn destructive_confirm_safety_scenario_uses_dry_run_confirm_fixture() {
    let scenario = read("scripts/agentic/scenario.ts");

    assert!(
        scenario.contains("runDestructiveConfirmModalSafetyStressScenario")
            && scenario.contains("dryRunOnlyRequired")
            && scenario.contains("type: \"openConfirmPrompt\"")
            && scenario.contains("agentic-destructive-confirm-open"),
        "destructive confirm safety stress must drive a dry-run confirm fixture instead of staying a static missing-receipt stub"
    );
    assert!(
        scenario.contains("destructiveCommandExecuted: false")
            && scenario.contains("systemCommandRequested: false")
            && scenario.contains("trashMutationRequested: false"),
        "dry-run destructive confirm proof must explicitly guard against real destructive commands"
    );
}

#[test]
fn confirm_prompt_simulate_key_routes_confirm_and_cancel() {
    let source = read("src/app_impl/simulate_key_dispatch.rs");

    assert!(
        source.contains("AppView::ConfirmPrompt { .. }")
            && source.contains("SimulateKey: Escape - cancel ConfirmPrompt")
            && source.contains("view.resolve_confirm_prompt(false, window, ctx)")
            && source.contains("SimulateKey: Enter - confirm ConfirmPrompt")
            && source.contains("view.resolve_confirm_prompt(confirmed, window, ctx)")
            && source.contains("view.toggle_confirm_prompt_focus(ctx)"),
        "stdin simulateKey must route ConfirmPrompt Tab, Enter, and Escape through the shared confirm resolver"
    );
}

#[test]
fn sdk_confirm_host_route_uses_shared_confirm_prompt_surface() {
    let source = read("src/prompt_handler/mod.rs");
    let show_confirm = source
        .split("PromptMessage::ShowConfirm {\n                id,")
        .nth(1)
        .and_then(|section| section.split("PromptMessage::ShowChat").next())
        .expect("ShowConfirm handler block should be present");

    assert!(
        show_confirm.contains("self.prepare_window_for_prompt(\"UI\", \"confirm\", \"\")")
            && show_confirm.contains("self.open_confirm_prompt(")
            && show_confirm.contains("async_channel::bounded::<bool>(1)")
            && show_confirm.contains("Message::Submit"),
        "SDK confirm() host route must open the shared in-window ConfirmPrompt and submit the bool result back to the script"
    );
    assert!(
        !show_confirm.contains("confirm_with_parent_dialog"),
        "SDK confirm() host route must not bypass the shared ConfirmPrompt surface via the native parent popup"
    );
}
