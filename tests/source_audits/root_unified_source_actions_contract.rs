use super::read_source;

#[test]
fn root_unified_source_actions_contract() {
    let actions_dialog = read_source("src/app_impl/actions_dialog.rs");
    let actions_toggle = read_source("src/app_impl/actions_toggle.rs");
    let root_actions = read_source("src/app_impl/root_unified_result_actions.rs");
    let app_state = read_source("src/main_sections/app_state.rs");
    let preflight = read_source("src/prompt_handler/mod.rs");

    let main_list = actions_dialog
        .find("ActionsDialogHost::MainList => {")
        .expect("MainList host branch should exist");
    let root_owner = actions_dialog[main_list..]
        .find("root_unified_action_owner_for_result")
        .expect("MainList should inspect the focused root result before generic fallback");
    let generic = actions_dialog[main_list..]
        .find("self.has_actions()")
        .expect("MainList should retain generic script action fallback");
    assert!(
        root_owner < generic,
        "root result action owner must run before generic has_actions/toggle_actions fallback"
    );

    assert!(
        actions_toggle.contains("toggle_root_unified_result_actions")
            && actions_toggle.contains("pending_root_unified_actions_subject")
            && root_actions.contains("RootUnifiedActionSubject::File"),
        "root files should be represented in the shared root result subject model"
    );

    for source in [
        "File(",
        "Note {",
        "Clipboard(",
        "BrowserTab(",
        "BrowserHistory(",
        "AcpHistory(",
        "Dictation {",
        "App(",
        "Window(",
        "BuiltIn(",
        "Skill(",
        "ScriptIssue(",
    ] {
        assert!(
            root_actions.contains(source),
            "missing root action subject coverage for {source}"
        );
    }

    for id in [
        "ROOT_FILE_OPEN_ACTION_ID",
        "root_note_open",
        "root_clipboard_paste",
        "root_browser_tab_switch",
        "root_browser_history_open",
        "root_acp_history_resume",
        "root_dictation_paste",
        "root_app_launch",
        "root_window_switch",
        "root_command_run",
        "root_skill_open",
        "root_script_issue_inspect",
    ] {
        assert!(
            root_actions.contains(id),
            "missing stable root action id {id}"
        );
    }

    assert!(
        root_actions.contains("pub(crate) enum RootUnifiedResultAction")
            && root_actions.contains("from_action_id")
            && root_actions.contains("unknown_root_unified_result_action"),
        "root actions must use a typed parser and no-op unknown ids"
    );
    assert!(
        actions_dialog.contains("root_unified_result_action_missing_subject")
            && actions_dialog.contains("return;"),
        "known root ids without a captured subject must not fall through to handle_action"
    );
    assert!(
        actions_dialog.contains("clear_actions_context_for_host")
            && actions_dialog.contains("pending_root_unified_actions_subject = None"),
        "MainList close/reset must clear pending root action subjects"
    );
    assert!(
        root_actions.contains("ExistingScriptActions")
            && actions_toggle.contains("ExistingScriptActions"),
        "script/scriptlet rows should delegate to existing script action ownership"
    );
    assert!(
        preflight.contains("\"visibleActions\"")
            && preflight.contains("\"contextStableKey\"")
            && !preflight.contains("\"rawClipboard\"")
            && !preflight.contains("\"noteBody\"")
            && !preflight.contains("\"transcript\""),
        "actionsDialog receipt should be content-light IDs/labels/sections only"
    );
    assert!(
        app_state.contains("pending_root_unified_actions_subject"),
        "app state should snapshot the subject while the popup is open"
    );
}
