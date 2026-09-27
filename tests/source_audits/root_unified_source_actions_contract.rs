use super::read_source;

#[test]
fn root_unified_source_actions_contract() {
    let actions_dialog = read_source("src/app_impl/actions_dialog.rs");
    let staging = read_source("src/one_window/staging.rs");
    let root_actions = read_source("src/app_impl/root_unified_result_actions.rs");
    let app_state = read_source("src/main_sections/app_state.rs");
    let preflight = read_source("src/prompt_handler/mod.rs");

    let actions_for = staging
        .find("fn one_window_actions_for(")
        .expect("⌘K on the main list should build the staged subject's actions");
    let root_owner = staging[actions_for..]
        .find("root_unified_action_owner_for_result")
        .expect("MainList should inspect the focused root result before generic fallback");
    let generic = staging[actions_for..]
        .find("crate::actions::get_script_context_actions(")
        .expect("MainList should retain generic script action fallback");
    assert!(
        root_owner < generic,
        "root result action owner must run before the generic script actions fallback"
    );

    assert!(
        staging.contains("self.pending_root_unified_actions_subject = Some(*owned);")
            && root_actions.contains("RootUnifiedActionSubject::File"),
        "root files should be represented in the shared root result subject model"
    );

    for source in [
        "File(",
        "Note {",
        "Clipboard(",
        "BrowserTab(",
        "BrowserHistory(",
        "AgentChatHistory(",
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
        "root_agent_chat_history_resume",
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
            && staging[actions_for..].contains("get_scriptlet_context_actions_with_custom("),
        "script/scriptlet rows should delegate to existing script action ownership"
    );
    assert!(
        root_actions.contains("SearchResult::BuiltIn(_)")
            && root_actions.contains("SearchResult::App(_)")
            && root_actions.contains("| SearchResult::BuiltIn(_)")
            && root_actions.contains("| SearchResult::App(_)"),
        "config-backed built-in and app rows should delegate to generic MainList actions"
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
        "app state should snapshot the subject while it is staged"
    );
}
