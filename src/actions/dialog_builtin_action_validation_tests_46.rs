// --- merged from part_01.rs ---
//! Batch 46: Dialog Built-in Action Validation Tests
//!
//! 120 tests across 30 categories validating action behaviors
//! in various built-in action window dialogs.

use crate::actions::builders::*;
use crate::actions::dialog::{
    build_grouped_items_static, coerce_action_selection, GroupedActionItem,
};
use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// =========== 1. Action::with_shortcut_opt: Some vs None ===========

#[test]
fn with_shortcut_opt_some_sets_shortcut() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘K".to_string()));
    assert_eq!(a.shortcut, Some("⌘K".to_string()));
}

#[test]
fn with_shortcut_opt_some_sets_shortcut_lower() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘K".to_string()));
    assert_eq!(a.shortcut_lower, Some("⌘k".to_string()));
}

#[test]
fn with_shortcut_opt_none_leaves_shortcut_none() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(a.shortcut.is_none());
}

#[test]
fn with_shortcut_opt_none_leaves_shortcut_lower_none() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(a.shortcut_lower.is_none());
}

// =========== 2. Action: title_lower correctly lowercased for mixed case ===========

#[test]
fn action_title_lower_from_mixed_case() {
    let a = Action::new("test", "Copy Deeplink", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "copy deeplink");
}

#[test]
fn action_title_lower_from_all_caps() {
    let a = Action::new("test", "SUBMIT", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "chat:submit");
}

#[test]
fn action_title_lower_preserves_already_lowercase() {
    let a = Action::new("test", "browse notes", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "browse notes");
}

#[test]
fn action_description_lower_from_mixed_case() {
    let a = Action::new(
        "test",
        "Test",
        Some("Open in $EDITOR".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower, Some("open in $editor".to_string()));
}

// =========== 3. ScriptInfo::with_action_verb_and_shortcut: verb and shortcut ===========

#[test]
fn with_action_verb_and_shortcut_sets_verb() {
    let s = ScriptInfo::with_action_verb_and_shortcut("Safari", "/app", false, "Launch", None);
    assert_eq!(s.action_verb, "Launch");
}

#[test]
fn with_action_verb_and_shortcut_sets_shortcut() {
    let s = ScriptInfo::with_action_verb_and_shortcut(
        "Safari",
        "/app",
        false,
        "Launch",
        Some("cmd+l".into()),
    );
    assert_eq!(s.shortcut, Some("cmd+l".to_string()));
}

#[test]
fn with_action_verb_and_shortcut_is_agent_false() {
    let s = ScriptInfo::with_action_verb_and_shortcut("Safari", "/app", false, "Launch", None);
    assert!(!s.is_agent);
}

#[test]
fn with_action_verb_and_shortcut_alias_none() {
    let s = ScriptInfo::with_action_verb_and_shortcut("Safari", "/app", false, "Launch", None);
    assert!(s.alias.is_none());
}

// =========== 4. Clipboard: unpinned text action count on macOS ===========

#[test]
fn clipboard_text_unpinned_has_pin_action() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_unpin"));
}

#[test]
fn clipboard_text_pinned_has_unpin_action() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_pin"));
}

#[test]
fn clipboard_pin_shortcut_is_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clip:clipboard_pin").unwrap();
    assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
}

#[test]
fn clipboard_unpin_shortcut_is_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clip:clipboard_unpin").unwrap();
    assert_eq!(unpin.shortcut.as_deref(), Some("⇧⌘P"));
}

// =========== 5. Clipboard: paste_keep_open shortcut ⌥↵ ===========

#[test]
fn clipboard_paste_keep_open_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_paste_keep_open")
        .unwrap();
    assert_eq!(pko.shortcut.as_deref(), Some("⌥↵"));
}

#[test]
fn clipboard_paste_keep_open_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_paste_keep_open")
        .unwrap();
    assert_eq!(pko.title, "Paste and Keep Window Open");
}

#[test]
fn clipboard_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_paste_keep_open")
        .unwrap();
    assert!(pko.description.as_ref().unwrap().contains("keep"));
}

#[test]
fn clipboard_paste_keep_open_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_paste_keep_open"));
}

// =========== 6. Clipboard: copy shortcut ⌘↵ ===========

#[test]
fn clipboard_copy_shortcut_is_cmd_enter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clip:clipboard_copy").unwrap();
    assert_eq!(copy.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn clipboard_copy_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clip:clipboard_copy").unwrap();
    assert_eq!(copy.title, "Copy to Clipboard");
}

#[test]
fn clipboard_copy_desc_mentions_without_pasting() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clip:clipboard_copy").unwrap();
    assert!(copy.description.as_ref().unwrap().contains("without"));
}

#[test]
fn clipboard_copy_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((50, 50)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_copy"));
}

// =========== 7. File context: copy_filename shortcut ⌘C ===========

#[test]
fn file_context_copy_filename_shortcut() {
    let fi = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn file_context_copy_filename_title() {
    let fi = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
    assert_eq!(cf.title, "Copy Filename");
}

#[test]
fn file_context_copy_filename_present_for_dir() {
    let fi = FileInfo {
        path: "/test/docs".into(),
        name: "docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    assert!(actions.iter().any(|a| a.id == "file:copy_filename"));
}

#[test]
fn file_context_copy_filename_desc_mentions_filename() {
    let fi = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
    assert!(cf.description.as_ref().unwrap().contains("filename"));
}

// =========== 8. Path context: open_in_editor shortcut ⌘E ===========

#[test]
fn path_context_open_in_editor_shortcut() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    let oie = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
    assert_eq!(oie.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn path_context_open_in_editor_title() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    let oie = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
    assert_eq!(oie.title, "Open in Editor");
}

#[test]
fn path_context_open_in_editor_desc_mentions_editor() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    let oie = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
    assert!(oie.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn path_context_open_in_editor_present_for_dir() {
    let pi = PathInfo::new("src", "/project/src", true);
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "file:open_in_editor"));
}

// =========== 9. Path context: move_to_trash shortcut ⌘⌫ ===========

#[test]
fn path_context_move_to_trash_shortcut() {
    let pi = PathInfo::new("old.txt", "/tmp/old.txt", false);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
    assert_eq!(mt.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn path_context_move_to_trash_file_desc_says_file() {
    let pi = PathInfo::new("old.txt", "/tmp/old.txt", false);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
    assert!(mt.description.as_ref().unwrap().contains("file"));
}

#[test]
fn path_context_move_to_trash_dir_desc_says_folder() {
    let pi = PathInfo::new("old_dir", "/tmp/old_dir", true);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
    assert!(mt.description.as_ref().unwrap().contains("folder"));
}

#[test]
fn path_context_move_to_trash_title() {
    let pi = PathInfo::new("old.txt", "/tmp/old.txt", false);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
    assert_eq!(mt.title, "Move to Trash");
}

// =========== 10. Path context: file has 7 actions, dir has 8 ===========

#[test]
fn path_context_file_action_count() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    // select_file, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_dir_action_count() {
    let pi = PathInfo::new("src", "/project/src", true);
    let actions = get_path_context_actions(&pi);
    // open_directory, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_file_has_select_file() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "file:select_file"));
}

// --- merged from part_02.rs ---

#[test]
fn path_context_dir_has_open_directory() {
    let pi = PathInfo::new("src", "/project/src", true);
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "file:open_directory"));
}

// =========== 11. Script: run_script title includes verb and quoted name ===========

#[test]
fn script_run_title_default_verb() {
    let s = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Run \"my-script\"");
}

#[test]
fn script_run_title_custom_verb_launch() {
    let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Launch \"Safari\"");
}

#[test]
fn script_run_title_custom_verb_switch_to() {
    let s = ScriptInfo::with_action_verb("Doc Window", "win:1", false, "Switch to");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Switch to \"Doc Window\"");
}

#[test]
fn script_run_desc_includes_verb() {
    let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.description.as_ref().unwrap().contains("Launch"));
}

// =========== 12. Script: copy_deeplink URL format ===========

#[test]
fn script_copy_deeplink_url_contains_slugified_name() {
    let s = ScriptInfo::new("My Cool Script", "/path/script.ts");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn script_copy_deeplink_shortcut() {
    let s = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
    assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
}

#[test]
fn script_copy_deeplink_title() {
    let s = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
    assert_eq!(dl.title, "Copy Deeplink");
}

#[test]
fn builtin_copy_deeplink_url_contains_slugified_name() {
    let s = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// =========== 13. Script: reset_ranking has no shortcut ===========

#[test]
fn script_reset_ranking_no_shortcut() {
    let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/p".into()));
    let actions = get_script_context_actions(&s);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert!(rr.shortcut.is_none());
}

#[test]
fn script_reset_ranking_title() {
    let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/p".into()));
    let actions = get_script_context_actions(&s);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert_eq!(rr.title, "Reset Ranking");
}

#[test]
fn script_reset_ranking_desc_mentions_suggested() {
    let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/p".into()));
    let actions = get_script_context_actions(&s);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert!(rr.description.as_ref().unwrap().contains("Suggested"));
}

#[test]
fn script_reset_ranking_absent_when_not_suggested() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
}

// =========== 14. Script: add_shortcut vs update_shortcut descriptions ===========

#[test]
fn script_add_shortcut_desc_mentions_set() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Set"));
}

#[test]
fn script_update_shortcut_desc_mentions_change() {
    let s = ScriptInfo::with_shortcut("test", "/p", Some("cmd+t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "update_shortcut").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Change"));
}

#[test]
fn script_remove_shortcut_desc_mentions_remove() {
    let s = ScriptInfo::with_shortcut("test", "/p", Some("cmd+t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "remove_shortcut").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Remove"));
}

#[test]
fn script_add_shortcut_shortcut_is_cmd_shift_k() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⇧K"));
}

// =========== 15. Script: add_alias vs update_alias descriptions ===========

#[test]
fn script_add_alias_desc_mentions_alias() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert!(a.description.as_ref().unwrap().contains("alias"));
}

#[test]
fn script_update_alias_desc_mentions_change() {
    let s = ScriptInfo::with_shortcut_and_alias("test", "/p", None, Some("t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "update_alias").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Change"));
}

#[test]
fn script_remove_alias_shortcut_is_cmd_opt_a() {
    let s = ScriptInfo::with_shortcut_and_alias("test", "/p", None, Some("t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "remove_alias").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥A"));
}

#[test]
fn script_add_alias_shortcut_is_cmd_shift_a() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⇧A"));
}

// =========== 16. AI bar: paste_image details ===========

#[test]
fn ai_bar_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "chat:paste_image").unwrap();
    assert_eq!(pi.shortcut.as_deref(), Some("⌘V"));
}

#[test]
fn ai_bar_paste_image_icon() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "chat:paste_image").unwrap();
    assert_eq!(pi.icon, Some(IconName::File));
}

#[test]
fn ai_bar_paste_image_section() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "chat:paste_image").unwrap();
    assert_eq!(pi.section.as_deref(), Some("Attachments"));
}

#[test]
fn ai_bar_paste_image_desc_mentions_clipboard() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "chat:paste_image").unwrap();
    assert!(pi.description.as_ref().unwrap().contains("clipboard"));
}

// =========== 17. AI bar: toggle_shortcuts_help details ===========

#[test]
fn ai_bar_toggle_shortcuts_help_shortcut() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "chat:toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn ai_bar_toggle_shortcuts_help_icon() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "chat:toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.icon, Some(IconName::Star));
}

#[test]
fn ai_bar_toggle_shortcuts_help_section() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "chat:toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.section.as_deref(), Some("Help"));
}

#[test]
fn ai_bar_toggle_shortcuts_help_title() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "chat:toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.title, "Keyboard Shortcuts");
}

// =========== 18. AI bar: change_model details ===========

#[test]
fn ai_bar_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert!(cm.shortcut.is_none());
}

#[test]
fn ai_bar_change_model_icon() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert_eq!(cm.icon, Some(IconName::Settings));
}

#[test]
fn ai_bar_change_model_section() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert_eq!(cm.section.as_deref(), Some("Settings"));
}

#[test]
fn ai_bar_change_model_desc_mentions_model() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert!(cm.description.as_ref().unwrap().contains("model"));
}

// =========== 19. AI bar: unique action IDs ===========

#[test]
fn ai_bar_all_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total);
}

#[test]
fn ai_bar_no_empty_ids() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn ai_bar_all_titles_non_empty() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.title.is_empty());
    }
}

#[test]
fn ai_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(a.section.is_some(), "Action {} should have a section", a.id);
    }
}

// =========== 20. Notes: browse_notes always present ===========

#[test]
fn notes_browse_notes_always_present_no_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

#[test]
fn notes_browse_notes_always_present_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

#[test]
fn notes_browse_notes_shortcut_cmd_p() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
}

#[test]
fn notes_browse_notes_icon_folder_open() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.icon, Some(IconName::FolderOpen));
}

// =========== 21. Notes: export details ===========

#[test]
fn notes_export_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ex = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(ex.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn notes_export_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ex = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(ex.icon, Some(IconName::ArrowRight));
}

#[test]
fn notes_export_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ex = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(ex.section.as_deref(), Some("Export"));
}

#[test]
fn notes_export_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =========== 22. Notes: copy_note_as icon Copy ===========

#[test]
fn notes_copy_note_as_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.icon, Some(IconName::Copy));
}

#[test]
fn notes_copy_note_as_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.section.as_deref(), Some("Copy"));
}

// --- merged from part_03.rs ---

#[test]
fn notes_copy_note_as_absent_no_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
}

#[test]
fn notes_copy_note_as_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
}

// =========== 23. Chat: copy_response conditional on has_response ===========

#[test]
fn chat_copy_response_present_when_has_response() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "chat:copy_response"));
}

#[test]
fn chat_copy_response_absent_when_no_response() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "chat:copy_response"));
}

#[test]
fn chat_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn chat_copy_response_title() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
    assert_eq!(cr.title, "Copy Last Response");
}

// =========== 24. Chat: clear_conversation conditional on has_messages ===========

#[test]
fn chat_clear_conversation_present_when_has_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "chat:clear_conversation"));
}

#[test]
fn chat_clear_conversation_absent_when_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "chat:clear_conversation"));
}

#[test]
fn chat_clear_conversation_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cc = actions
        .iter()
        .find(|a| a.id == "chat:clear_conversation")
        .unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn chat_clear_conversation_title() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cc = actions
        .iter()
        .find(|a| a.id == "chat:clear_conversation")
        .unwrap();
    assert_eq!(cc.title, "Clear Conversation");
}

// =========== 25. New chat: empty inputs produce empty actions ===========

#[test]
fn new_chat_all_empty_produces_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_only_last_used_count() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
}

#[test]
fn new_chat_only_presets_count() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
}

#[test]
fn new_chat_only_models_count() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
}

// =========== 26. New chat: model ID format uses index ===========

#[test]
fn new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn new_chat_last_used_id_format() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code");
}

#[test]
fn new_chat_combined_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "c".into(),
        display_name: "Claude".into(),
        provider: "a".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "g".into(),
        display_name: "GPT-4".into(),
        provider: "o".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

// =========== 27. Note switcher: empty notes produces "no notes yet" ===========

#[test]
fn note_switcher_empty_has_no_notes_message() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn note_switcher_no_notes_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn note_switcher_no_notes_icon() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn note_switcher_no_notes_desc_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

// =========== 28. Note switcher: char count display when no preview ===========

#[test]
fn note_switcher_no_preview_shows_char_count_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn note_switcher_no_preview_shows_char_count_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_no_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5m ago"));
}

#[test]
fn note_switcher_with_preview_and_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("Hello world · 2d ago")
    );
}

// =========== 29. coerce_action_selection: mixed headers and items ===========

#[test]
fn coerce_selection_first_header_then_items() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_item_between_headers() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H2".into()),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 2), Some(3));
}

#[test]
fn coerce_selection_trailing_header_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_multiple_headers_between_items() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::SectionHeader("H2".into()),
        GroupedActionItem::Item(1),
    ];
    // Index 1 is header, search down → finds Item(1) at index 3
    assert_eq!(coerce_action_selection(&rows, 1), Some(3));
}

// =========== 30. build_grouped_items_static: action count matches filtered ===========

#[test]
fn build_grouped_items_item_count_matches_filtered() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext),
        Action::new("b", "B", None, ActionCategory::ScriptContext),
        Action::new("c", "C", None, ActionCategory::ScriptContext),
    ];
    let filtered = vec![0usize, 1, 2];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let item_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::Item(_)))
        .count();
    assert_eq!(item_count, 3);
}

#[test]
fn build_grouped_items_headers_from_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0usize, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 2);
}

#[test]
fn build_grouped_items_no_headers_with_none_style() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0usize, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 0);
}

#[test]
fn build_grouped_items_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Same"),
    ];
    let filtered = vec![0usize, 1, 2];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 1);
}
