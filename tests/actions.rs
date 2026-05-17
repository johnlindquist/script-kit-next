// Integration-level behavior tests for action handler feedback contracts.
// These verify the structural invariants of the action system across files.

use std::fs;

/// Verify the feedback contract constants exist in helpers.rs:
/// success feedback uses HUD, errors use Toast.
#[test]
fn feedback_contract_constants_defined() {
    let content =
        fs::read_to_string("src/app_actions/helpers.rs").expect("Failed to read helpers.rs");

    // HUD constants for success paths
    assert!(
        content.contains("HUD_SHORT_MS"),
        "HUD_SHORT_MS must be defined for success feedback"
    );
    assert!(
        content.contains("HUD_MEDIUM_MS"),
        "HUD_MEDIUM_MS must be defined for success feedback"
    );

    // Toast constants for error paths
    assert!(
        content.contains("TOAST_ERROR_MS"),
        "TOAST_ERROR_MS must be defined for error feedback"
    );
    assert!(
        content.contains("TOAST_CRITICAL_MS"),
        "TOAST_CRITICAL_MS must be defined for critical errors"
    );
}

/// Verify that the action handler files have the expected dispatch structure.
#[test]
fn action_handler_dispatch_structure_exists() {
    // Collect all handler source from modular handler directory
    let handler_dir = std::path::Path::new("src/app_actions/handle_action");
    let mut content = String::new();
    for entry in fs::read_dir(handler_dir).expect("Failed to read handle_action directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let chunk = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
            content.push_str(&chunk);
            content.push('\n');
        }
    }

    // Must handle copy actions
    assert!(
        content.contains("\"copy_path\""),
        "Action handler must dispatch copy_path"
    );
    assert!(
        content.contains("\"copy_deeplink\""),
        "Action handler must dispatch copy_deeplink"
    );
    assert!(
        content.contains("\"copy_content\""),
        "Action handler must dispatch copy_content"
    );

    // Must handle file search actions
    assert!(
        content.contains("\"open_file\""),
        "Action handler must dispatch open_file"
    );
    assert!(
        content.contains("\"quick_look\""),
        "Action handler must dispatch quick_look"
    );

    // Must have toast_manager usage for errors (may be in chunk files)
    assert!(
        content.contains("toast_manager.push("),
        "Action handler must use toast_manager for error feedback"
    );
}

#[test]
fn chat_transcript_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat action builder");

    assert!(
        content.contains("enum ChatTranscriptActionPlan")
            && content.contains("NoTranscriptActions")
            && content.contains("CopyLastResponse")
            && content.contains("ClearConversation")
            && content.contains("CopyLastResponseAndClearConversation"),
        "chat transcript copy/clear actions should be driven by named transcript action plan states"
    );
    assert!(
        content.contains("fn append_transcript_actions")
            && content.contains("ChatTranscriptActionPlan::from_info(info)"),
        "chat context actions should append transcript actions from the named plan"
    );
}

#[test]
fn chat_model_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat action builder");

    assert!(
        content.contains("enum ChatModelPickerRowPlan")
            && content.contains("CurrentModel")
            && content.contains("AvailableModel"),
        "chat model picker checkmark copy should be driven by named row plan states"
    );
    assert!(
        content.contains("enum ChatChangeModelActionPlan")
            && content.contains("CurrentModelSelected")
            && content.contains("NoCurrentModelSelected"),
        "chat Change Model description should be driven by named action plan states"
    );
    assert!(
        content.contains("ChatModelPickerRowPlan::from_model(info, model)")
            && content.contains("ChatChangeModelActionPlan::from_info(info).description(info)"),
        "chat model rows and root Change Model copy should derive from named plans"
    );
}

#[test]
fn clipboard_entry_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/clipboard.rs")
        .expect("Failed to read clipboard action builder");

    assert!(
        content.contains("enum ClipboardEntryActionPlan")
            && content.contains("TextPinned")
            && content.contains("TextUnpinned")
            && content.contains("ImagePinned")
            && content.contains("ImageUnpinned")
            && content.contains("OtherPinned")
            && content.contains("OtherUnpinned"),
        "clipboard text/image and pin/unpin actions should be driven by named entry action plan states"
    );
    assert!(
        content.contains("ClipboardEntryActionPlan::from_entry(entry)")
            && content.contains("fn pin_action")
            && content.contains("fn is_image")
            && content.contains("fn is_text"),
        "clipboard context actions should derive pin and content-specific rows from the named plan"
    );
}

#[test]
fn clipboard_pin_handler_uses_named_action_states() {
    let content = fs::read_to_string("src/app_actions/handle_action/clipboard.rs")
        .expect("Failed to read clipboard action handler");

    assert!(
        content.contains("enum ClipboardPinHandlerAction")
            && content.contains("Pin")
            && content.contains("Unpin"),
        "clipboard pin/unpin handler should be driven by named action states"
    );
    assert!(
        content.contains("ClipboardPinHandlerAction::from_action_id(action_id)")
            && content.contains("pin_action.apply(&entry.id)"),
        "clipboard pin/unpin handler should derive the storage operation from the named action state"
    );
}

#[test]
fn clipboard_paste_destination_uses_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/clipboard.rs")
        .expect("Failed to read clipboard action builder");

    assert!(
        content.contains("enum ClipboardPasteDestinationPlan")
            && content.contains("FrontmostApp")
            && content.contains("ActiveAppFallback"),
        "clipboard paste title should be driven by named destination plan states"
    );
    assert!(
        content.contains("ClipboardPasteDestinationPlan::from_entry(entry)")
            && content.contains("paste_destination_plan.title(entry)"),
        "clipboard paste action title should derive from the named destination plan"
    );
}

#[test]
fn emoji_entry_actions_use_named_plan_states() {
    let content =
        fs::read_to_string("src/actions/builders/emoji.rs").expect("Failed to read emoji builder");

    assert!(
        content.contains("enum EmojiEntryActionPlan")
            && content.contains("PinnedWithCategory")
            && content.contains("PinnedWithoutCategory")
            && content.contains("UnpinnedWithCategory")
            && content.contains("UnpinnedWithoutCategory"),
        "emoji pin/unpin and section-copy actions should be driven by named entry action plan states"
    );
    assert!(
        content.contains("EmojiEntryActionPlan::from_info(emoji)")
            && content.contains("fn pin_action")
            && content.contains("fn has_category"),
        "emoji context actions should derive pin and category rows from the named plan"
    );
}

#[test]
fn emoji_pin_handler_uses_named_action_states() {
    let content = fs::read_to_string("src/app_actions/handle_action/emoji.rs")
        .expect("Failed to read emoji action handler");

    assert!(
        content.contains("enum EmojiPinHandlerAction")
            && content.contains("Pin")
            && content.contains("Unpin"),
        "emoji pin/unpin handler should be driven by named action states"
    );
    assert!(
        content.contains("EmojiPinHandlerAction::from_action_id(action_id)")
            && content.contains("pin_action.apply(&mut self.pinned_emojis, &emoji.value)")
            && content.contains("pin_action.success_hud(&emoji.value)"),
        "emoji pin/unpin handler should derive mutation and HUD copy from the named action state"
    );
}

#[test]
fn emoji_paste_destination_uses_named_plan_states() {
    let content =
        fs::read_to_string("src/actions/builders/emoji.rs").expect("Failed to read emoji builder");

    assert!(
        content.contains("enum EmojiPasteDestinationPlan")
            && content.contains("FrontmostApp")
            && content.contains("ActiveAppFallback"),
        "emoji paste title should be driven by named destination plan states"
    );
    assert!(
        content.contains("EmojiPasteDestinationPlan::from_info(emoji)")
            && content.contains("paste_destination_plan.title(emoji)"),
        "emoji paste action title should derive from the named destination plan"
    );
}

#[test]
fn notes_command_bar_actions_use_named_plan_states() {
    let content =
        fs::read_to_string("src/actions/builders/notes.rs").expect("Failed to read notes builder");

    assert!(
        content.contains("enum NotesCommandBarActionPlan")
            && content.contains("EmptyActiveAutoSized")
            && content.contains("EmptyActiveNeedsAutoSizing")
            && content.contains("SelectedActiveAutoSized")
            && content.contains("SelectedActiveNeedsAutoSizing")
            && content.contains("EmptyTrashAutoSized")
            && content.contains("EmptyTrashNeedsAutoSizing")
            && content.contains("SelectedTrashAutoSized")
            && content.contains("SelectedTrashNeedsAutoSizing"),
        "notes command-bar selection/trash/auto-size actions should be driven by named plan states"
    );
    assert!(
        content.contains("NotesCommandBarActionPlan::from_info(info)")
            && content.contains("fn has_active_note_actions")
            && content.contains("fn has_trash_note_actions")
            && content.contains("fn needs_auto_sizing_action"),
        "notes command-bar actions should derive visible action groups from the named plan"
    );
}

#[test]
fn note_switcher_actions_use_named_plan_states() {
    let content =
        fs::read_to_string("src/actions/builders/notes.rs").expect("Failed to read notes builder");

    assert!(
        content.contains("enum NoteSwitcherRowPlan")
            && content.contains("PinnedCurrent")
            && content.contains("PinnedOther")
            && content.contains("Current")
            && content.contains("Recent"),
        "note switcher pinned/current priority should be driven by named row plan states"
    );
    assert!(
        content.contains("enum NoteSwitcherDescriptionPlan")
            && content.contains("PreviewWithRelativeTime")
            && content.contains("PreviewOnly")
            && content.contains("RelativeTimeOnly")
            && content.contains("CharacterCount"),
        "note switcher preview/time/count text should be driven by named description plan states"
    );
    assert!(
        content.contains("NoteSwitcherRowPlan::from_note(note)")
            && content.contains("NoteSwitcherDescriptionPlan::from_note(note)")
            && content.contains("fn truncated_preview"),
        "note switcher actions should derive row text and description text from named plans"
    );
}

#[test]
fn scriptlet_context_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/scriptlet.rs")
        .expect("Failed to read scriptlet builder");

    assert!(
        content.contains("enum ScriptletContextActionPlan")
            && content.contains("NoShortcutNoAlias")
            && content.contains("ShortcutOnly")
            && content.contains("AliasOnly")
            && content.contains("ShortcutAndAlias"),
        "scriptlet shortcut and alias action rows should be driven by named context plan states"
    );
    assert!(
        content.contains("ScriptletContextActionPlan::from_script(script)")
            && content.contains("fn has_shortcut")
            && content.contains("fn has_alias"),
        "scriptlet context actions should derive add/update/remove shortcut and alias rows from the named plan"
    );
}

#[test]
fn scriptlet_defined_action_shortcuts_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/scriptlet.rs")
        .expect("Failed to read scriptlet builder");

    assert!(
        content.contains("enum ScriptletDefinedActionShortcutPlan")
            && content.contains("NoShortcut")
            && content.contains("Shortcut(&'a str)"),
        "scriptlet-defined H3 action shortcuts should be driven by named shortcut plan states"
    );
    assert!(
        content.contains("ScriptletDefinedActionShortcutPlan::from_shortcut(sa.shortcut.as_deref())")
            && content.contains("shortcut_plan.apply_to_action(Action::new"),
        "scriptlet-defined action rows should apply shortcut copy through the named plan"
    );
}

#[test]
fn scriptlet_ranking_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/scriptlet.rs")
        .expect("Failed to read scriptlet builder");

    assert!(
        content.contains("enum ScriptletRankingActionPlan")
            && content.contains("NoRankingAction")
            && content.contains("ResetSuggestedRanking"),
        "scriptlet suggested ranking row should be driven by named ranking plan states"
    );
    assert!(
        content.contains("ScriptletRankingActionPlan::from_script(script)")
            && content.contains("fn reset_action")
            && content.contains("ranking_plan.reset_action()"),
        "scriptlet ranking row should derive availability and copy from the named plan"
    );
}

#[test]
fn file_path_actions_use_named_item_plan_states() {
    let content = fs::read_to_string("src/actions/builders/file_path.rs")
        .expect("Failed to read file path builder");

    assert!(
        content.contains("enum FileItemActionPlan")
            && content.contains("File")
            && content.contains("Directory"),
        "file/path primary, trash, and attach actions should be driven by named item plan states"
    );
    assert!(
        content.contains("FileItemActionPlan::from_is_dir(file_info.is_dir)")
            && content.contains("FileItemActionPlan::from_is_dir(path_info.is_dir)")
            && content.contains("fn file_context_primary_action")
            && content.contains("fn path_context_primary_action")
            && content.contains("fn supports_attach_to_ai")
            && content.contains("fn item_noun"),
        "file/path actions should derive file-vs-folder copy and availability from the named plan"
    );
}

#[test]
fn file_search_secondary_descriptions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/file_path.rs")
        .expect("Failed to read file path builder");

    assert!(
        content.contains("enum FileSearchSecondaryDescriptionPlan")
            && content.contains("StaticDescription")
            && content.contains("TrashItem"),
        "file-search secondary command descriptions should be driven by named description plan states"
    );
    assert!(
        content.contains("FileSearchSecondaryDescriptionPlan::from_action_id(self.action_id)")
            && content.contains("description_plan.description(self.description, action_plan)"),
        "file-search secondary actions should derive static/trash descriptions from the named plan"
    );
}

#[test]
fn file_search_handler_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/app_actions/handle_action/files.rs")
        .expect("Failed to read file action handler");

    assert!(
        content.contains("enum FileSearchHandlerAction")
            && content.contains("Open")
            && content.contains("QuickLook")
            && content.contains("OpenWith")
            && content.contains("ShowInfo")
            && content.contains("AttachToAi"),
        "safe file-search handler actions should be driven by named action states"
    );
    assert!(
        content.contains("FileSearchHandlerAction::from_action_id(action_id)")
            && content.contains("file_action.success_hud(action_id)")
            && content.contains("file_action.error_prefix(action_id)")
            && content.contains("file_action.hides_main_after_success()"),
        "file-search handler should derive HUD, error, and hide behavior from the named action state"
    );
}

#[test]
fn file_search_sort_handler_uses_named_action_states() {
    let content = fs::read_to_string("src/app_actions/handle_action/files.rs")
        .expect("Failed to read file action handler");

    assert!(
        content.contains("enum FileSearchSortHandlerAction")
            && content.contains("NameAsc")
            && content.contains("NameDesc")
            && content.contains("ModifiedDesc")
            && content.contains("ModifiedAsc"),
        "file-search sort handler should be driven by named sort action states"
    );
    assert!(
        content.contains("FileSearchSortHandlerAction::from_action_id(action_id)")
            && content.contains("let mode = sort_action.mode()")
            && content.contains("sort_action.success_hud()"),
        "file-search sort handler should derive sort mode and HUD copy from the named state"
    );
}

#[test]
fn file_search_sort_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/file_path.rs")
        .expect("Failed to read file path builder");

    assert!(
        content.contains("enum FileSearchSortActionPlan")
            && content.contains("ActiveSort")
            && content.contains("AvailableSort"),
        "file-search sort checkmark and current-sort text should be driven by named sort action plan states"
    );
    assert!(
        content.contains("FileSearchSortActionPlan::from_active(name_asc_active)")
            && content.contains("fn title")
            && content.contains("fn description"),
        "file-search sort actions should derive title and description copy from the named sort plan"
    );
}

#[test]
fn script_context_preference_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum ScriptContextPreferenceActionPlan")
            && content.contains("AgentNoPreferenceActions")
            && content.contains("NoShortcutNoAlias")
            && content.contains("ShortcutOnly")
            && content.contains("AliasOnly")
            && content.contains("ShortcutAndAlias"),
        "script context shortcut and alias rows should be driven by named preference plan states"
    );
    assert!(
        content.contains("preference_action_plan(script)")
            && content.contains("fn append_shortcut_preference_actions")
            && content.contains("fn append_alias_preference_actions"),
        "script context preference rows should be appended from the named plan"
    );
}

#[test]
fn script_context_share_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum ScriptContextShareActionPlan")
            && content.contains("PortableShareLink")
            && content.contains("DirectRunDeepLink")
            && content.contains("struct ScriptContextShareActionCopy"),
        "script context Share vs Copy Deep Link copy should be driven by named share plan states"
    );
    assert!(
        content.contains("share_action_plan(script)")
            && content.contains("fn share_action_copy")
            && content.contains("let share_copy = share_action_copy(script)"),
        "script context copy_deeplink row should derive title and description from the named share plan"
    );
}

#[test]
fn script_context_favorite_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum FavoriteActionPlan")
            && content.contains("AddToFavorites")
            && content.contains("RemoveFromFavorites"),
        "script context favorite action copy should be driven by named favorite plan states"
    );
    assert!(
        content.contains("FavoriteActionPlan::from_is_favorite(is_favorite).copy()")
            && content.contains("fn favorite_action_copy"),
        "favorite action copy should derive title and description from the named favorite plan"
    );
}

#[test]
fn script_context_ranking_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum RankingActionPlan")
            && content.contains("NoRankingAction")
            && content.contains("ResetSuggestedRanking"),
        "suggested ranking destructive row should be driven by named ranking plan states"
    );
    assert!(
        content.contains("RankingActionPlan::from_is_suggested(script.is_suggested)")
            && content.contains("fn reset_action")
            && content.contains("ranking_plan.reset_action()"),
        "script context ranking row should derive availability and copy from the named plan"
    );
}

#[test]
fn acp_agent_selection_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum AcpAgentSelectionActionPlan")
            && content.contains("CurrentAgent")
            && content.contains("AvailableAgent"),
        "ACP agent current/use labels and descriptions should be driven by named selection plan states"
    );
    assert!(
        content.contains("AcpAgentSelectionActionPlan::from_is_selected(is_selected)")
            && content.contains("fn action_title")
            && content.contains("fn picker_title")
            && content.contains("fn description"),
        "ACP agent action and picker rows should derive visible copy from the named selection plan"
    );
}

#[test]
fn acp_model_selection_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum AcpModelSelectionActionPlan")
            && content.contains("CurrentModel")
            && content.contains("AvailableModel"),
        "ACP model current/switch labels and descriptions should be driven by named selection plan states"
    );
    assert!(
        content.contains("AcpModelSelectionActionPlan::from_is_selected(is_selected)")
            && content.contains("fn picker_title")
            && content.contains("fn description"),
        "ACP model picker rows should derive visible copy from the named selection plan"
    );
}

#[test]
fn acp_root_change_actions_use_named_plan_states() {
    let content = fs::read_to_string("src/actions/builders/script_context.rs")
        .expect("Failed to read script context builder");

    assert!(
        content.contains("enum AcpRootPickerActionPlan")
            && content.contains("CurrentSelection")
            && content.contains("NoCurrentSelection"),
        "ACP root Change Agent/Model descriptions should be driven by named picker plan states"
    );
    assert!(
        content.contains("AcpRootPickerActionPlan::from_selected_display_name")
            && content.contains("agent_picker_plan.description(ACP_CHANGE_AGENT_DESCRIPTION)")
            && content.contains("model_picker_plan.description(ACP_CHANGE_MODEL_DESCRIPTION)"),
        "ACP root Change Agent/Model actions should derive current/fallback copy from the named picker plan"
    );
}

/// Verify SDK actions use tracing, not legacy logging.
#[test]
fn sdk_actions_uses_modern_logging() {
    let content = fs::read_to_string("src/app_actions/sdk_actions.rs")
        .expect("Failed to read sdk_actions.rs");

    assert!(
        !content.contains("logging::log("),
        "sdk_actions.rs must not use legacy logging::log — use tracing:: instead"
    );
    assert!(
        content.contains("tracing::info!(") || content.contains("tracing::warn!("),
        "sdk_actions.rs must use tracing for observability"
    );
}

#[test]
fn sdk_action_lookup_uses_named_plan_states() {
    let content = fs::read_to_string("src/app_actions/sdk_actions.rs")
        .expect("Failed to read sdk_actions.rs");

    assert!(
        content.contains("enum SdkActionLookupPlan")
            && content.contains("NoActionsDefined")
            && content.contains("ActionFound")
            && content.contains("ActionMissing"),
        "SDK action lookup should be driven by named action-list states"
    );
    assert!(
        content.contains("SdkActionLookupPlan::from_actions(self.sdk_actions.as_deref(), action_name)")
            && content.contains("fn is_found"),
        "SDK action dispatch and shortcut triggering should derive found/missing behavior from the named lookup plan"
    );
}

// ---------------------------------------------------------------------------
// Structural coverage tests for action handler consistency
// ---------------------------------------------------------------------------

/// Every known action ID that appears in the modular handle_action handlers
/// must have a match arm. This test collects all quoted action
/// ID strings from the dispatch match and verifies they form a known set.
#[test]
fn all_action_ids_have_handler_arms() {
    // Collect all handler source from modular handler directory
    let handler_dir = std::path::Path::new("src/app_actions/handle_action");
    let mut all_content = String::new();
    for entry in fs::read_dir(handler_dir).expect("Failed to read handle_action directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let chunk = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
            all_content.push_str(&chunk);
            all_content.push('\n');
        }
    }

    // These are the action IDs that must have handler arms in the dispatch.
    // Each entry represents a user-facing action the system can trigger.
    let required_action_ids = [
        // Clipboard actions
        "clipboard_pin",
        "clipboard_unpin",
        "clipboard_share",
        "clipboard_paste",
        "clipboard_attach_to_ai",
        "clipboard_copy",
        "clipboard_paste_keep_open",
        "clipboard_quick_look",
        "clipboard_open_with",
        "clipboard_annotate_cleanshot",
        "clipboard_upload_cleanshot",
        "clipboard_ocr",
        "clipboard_delete",
        "clipboard_delete_multiple",
        "clipboard_delete_all",
        "clipboard_save_file",
        "clipboard_save_snippet",
        // Script management actions
        "create_script",
        "run_script",
        "view_logs",
        "reveal_in_finder",
        "copy_path",
        "copy_deeplink",
        "copy_content",
        "copy_filename",
        "edit_script",
        "remove_script",
        "delete_script",
        "reload_scripts",
        "settings",
        "quit",
        // Shortcut / alias actions
        "configure_shortcut",
        "add_shortcut",
        "update_shortcut",
        "remove_shortcut",
        "add_alias",
        "update_alias",
        "remove_alias",
        // File search actions
        "open_file",
        "quick_look",
        "open_with",
        "show_info",
        "attach_to_ai",
        // Scriptlet actions
        "edit_scriptlet",
        "reveal_scriptlet_in_finder",
        "copy_scriptlet_path",
        "reset_ranking",
        // Control signals
        "__cancel__",
    ];

    for action_id in &required_action_ids {
        let pattern = format!("\"{}\"", action_id);
        assert!(
            all_content.contains(&pattern),
            "Action ID '{}' must have a handler arm in handle_action dispatch",
            action_id
        );
    }
}

/// No handler files should use the legacy `logging::log()` pattern.
/// All logging must use `tracing::` macros.
#[test]
fn no_legacy_logging_in_handler_files() {
    let handler_dir = std::path::Path::new("src/app_actions/handle_action");
    let files_to_check: Vec<std::path::PathBuf> = {
        let mut files = Vec::new();
        for entry in fs::read_dir(handler_dir).expect("Failed to read handle_action directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        files
    };

    let mut violations = Vec::new();
    for file_path in &files_to_check {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));
        let count = content.matches("logging::log(").count();
        if count > 0 {
            violations.push(format!(
                "{}: {} logging::log() calls",
                file_path.display(),
                count
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Legacy logging::log() calls found in handler files (must use tracing::):\n  {}",
        violations.join("\n  ")
    );
}

/// Every variant of `BuiltInFeature` must have a corresponding match arm
/// in `execute_builtin_with_query` in `builtin_execution.rs`.
#[test]
fn all_builtin_feature_variants_have_execution_arms() {
    let enum_source =
        fs::read_to_string("src/builtins/mod.rs").expect("Failed to read builtins/mod.rs");
    let exec_source = fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("Failed to read builtin_execution.rs");

    // Extract variant names from the enum definition.
    // We look for lines between `pub enum BuiltInFeature {` and the closing `}`.
    let enum_start = enum_source
        .find("pub enum BuiltInFeature {")
        .expect("BuiltInFeature enum not found");
    let enum_body_start = enum_source[enum_start..]
        .find('{')
        .expect("Opening brace not found")
        + enum_start
        + 1;

    // Find matching closing brace (handle nested braces)
    let mut depth = 1;
    let mut enum_body_end = enum_body_start;
    for (i, ch) in enum_source[enum_body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    enum_body_end = enum_body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }

    let enum_body = &enum_source[enum_body_start..enum_body_end];

    // Extract variant names (identifier before `(` or `,` or end-of-line)
    let mut variants = Vec::new();
    for line in enum_body.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("///") {
            continue;
        }
        // Extract the variant name (first identifier on the line)
        let variant_name: String = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !variant_name.is_empty() {
            variants.push(variant_name);
        }
    }

    assert!(
        !variants.is_empty(),
        "Failed to extract any variants from BuiltInFeature enum"
    );

    // Verify each variant appears in execute_builtin_with_query as a match arm
    let mut missing = Vec::new();
    for variant in &variants {
        // Look for `BuiltInFeature::VariantName` in the execution source
        let pattern = format!("BuiltInFeature::{}", variant);
        if !exec_source.contains(&pattern) {
            missing.push(variant.as_str());
        }
    }

    assert!(
        missing.is_empty(),
        "BuiltInFeature variants missing from execute_builtin_with_query:\n  {}",
        missing.join(", ")
    );
}
