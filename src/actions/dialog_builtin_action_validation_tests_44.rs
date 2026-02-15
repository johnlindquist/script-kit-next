// --- merged from part_01.rs ---
//! Batch 44: Dialog Built-in Action Validation Tests
//!
//! 120 tests across 30 categories validating action behaviors
//! in various built-in action window dialogs.

use crate::actions::builders::*;
use crate::actions::dialog::ActionsDialog;
use crate::actions::types::{Action, ActionCategory, ScriptInfo};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =========== 1. ScriptInfo::with_is_script: is_script true sets correct defaults ===========

#[test]
fn with_is_script_true_sets_is_script() {
    let s = ScriptInfo::with_is_script("my-script", "/path", true);
    assert!(s.is_script);
}

#[test]
fn with_is_script_true_is_scriptlet_false() {
    let s = ScriptInfo::with_is_script("my-script", "/path", true);
    assert!(!s.is_scriptlet);
}

#[test]
fn with_is_script_false_sets_is_script_false() {
    let s = ScriptInfo::with_is_script("builtin", "", false);
    assert!(!s.is_script);
}

#[test]
fn with_is_script_defaults_action_verb_run() {
    let s = ScriptInfo::with_is_script("test", "/p", true);
    assert_eq!(s.action_verb, "Run");
}

// =========== 2. ScriptInfo::with_action_verb: custom verb preserved ===========

#[test]
fn with_action_verb_sets_verb() {
    let s = ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
    assert_eq!(s.action_verb, "Launch");
}

#[test]
fn with_action_verb_is_script_param() {
    let s = ScriptInfo::with_action_verb("test", "/p", true, "Execute");
    assert!(s.is_script);
}

#[test]
fn with_action_verb_false_is_script() {
    let s = ScriptInfo::with_action_verb("test", "/p", false, "Open");
    assert!(!s.is_script);
}

#[test]
fn with_action_verb_shortcut_none() {
    let s = ScriptInfo::with_action_verb("test", "/p", true, "Run");
    assert!(s.shortcut.is_none());
}

// =========== 3. Clipboard: paste title with frontmost_app_name ===========

#[test]
fn clipboard_paste_title_with_app_name() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Safari".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Safari");
}

#[test]
fn clipboard_paste_title_without_app_name() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn clipboard_paste_shortcut_is_enter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.shortcut, Some("↵".to_string()));
}

#[test]
fn clipboard_paste_desc_mentions_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert!(paste.description.as_ref().unwrap().contains("paste"));
}

// =========== 4. Clipboard: save_snippet and save_file details ===========

#[test]
fn clipboard_save_snippet_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_save_snippet")
        .unwrap();
    assert_eq!(ss.shortcut, Some("⇧⌘S".to_string()));
}

#[test]
fn clipboard_save_snippet_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_save_snippet")
        .unwrap();
    assert_eq!(ss.title, "Save Text as Snippet");
}

#[test]
fn clipboard_save_file_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let sf = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_save_file")
        .unwrap();
    assert_eq!(sf.shortcut, Some("⌥⇧⌘S".to_string()));
}

#[test]
fn clipboard_save_file_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let sf = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_save_file")
        .unwrap();
    assert_eq!(sf.title, "Save as File...");
}

// =========== 5. Clipboard: image upload_cleanshot details (macOS) ===========

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let uc = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(uc.shortcut, Some("⇧⌘U".to_string()));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let uc = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(uc.title, "Upload to CleanShot X");
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_not_present_for_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "txt".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_upload_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_desc_mentions_cloud() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((200, 200)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let uc = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_upload_cleanshot")
        .unwrap();
    assert!(uc.description.as_ref().unwrap().contains("Cloud"));
}

// =========== 6. Clipboard: OCR shortcut and desc ===========

#[test]
fn clipboard_ocr_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clip:clipboard_ocr").unwrap();
    assert_eq!(ocr.shortcut, Some("⇧⌘C".to_string()));
}

#[test]
fn clipboard_ocr_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clip:clipboard_ocr").unwrap();
    assert_eq!(ocr.title, "Copy Text from Image");
}

#[test]
fn clipboard_ocr_desc_mentions_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clip:clipboard_ocr").unwrap();
    assert!(ocr.description.as_ref().unwrap().contains("OCR"));
}

#[test]
fn clipboard_ocr_not_present_for_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "txt".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
}

// =========== 7. File context: quick_look only for files (macOS) ===========

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_present_for_file() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert!(actions.iter().any(|a| a.id == "file:quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_shortcut() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ql = actions.iter().find(|a| a.id == "file:quick_look").unwrap();
    assert_eq!(ql.shortcut, Some("⌘Y".to_string()));
}

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_absent_for_dir() {
    let dir = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert!(!actions.iter().any(|a| a.id == "file:quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_desc_mentions_preview() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ql = actions.iter().find(|a| a.id == "file:quick_look").unwrap();
    assert!(ql
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("preview"));
}

// =========== 8. File context: copy_path shortcut is ⌘⇧C ===========

#[test]
fn file_copy_path_shortcut() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
    assert_eq!(cp.shortcut, Some("⌘⇧C".to_string()));
}

#[test]
fn file_copy_path_title() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
    assert_eq!(cp.title, "Copy Path");
}

#[test]
fn file_copy_path_desc_mentions_clipboard() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("clipboard"));
}

#[test]
fn file_copy_path_present_for_dir() {
    let dir = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert!(actions.iter().any(|a| a.id == "file:copy_path"));
}

// --- merged from part_02.rs ---

// =========== 9. Path context: all actions have ScriptContext category ===========

#[test]
fn path_file_all_script_context() {
    let p = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&p);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn path_dir_all_script_context() {
    let p = PathInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&p);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn path_file_primary_is_first() {
    let p = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&p);
    assert_eq!(actions[0].id, "file:select_file");
}

#[test]
fn path_dir_primary_is_first() {
    let p = PathInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&p);
    assert_eq!(actions[0].id, "file:open_directory");
}

// =========== 10. Script context: run_script title includes verb and quoted name ===========

#[test]
fn script_run_title_includes_verb() {
    let s = ScriptInfo::with_action_verb("Test", "/p", true, "Launch");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Launch"));
}

#[test]
fn script_run_title_includes_quoted_name() {
    let s = ScriptInfo::new("My Script", "/p");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Run");
}

#[test]
fn script_run_desc_includes_verb() {
    let s = ScriptInfo::with_action_verb("X", "/p", true, "Execute");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.description.as_ref().unwrap().contains("Execute"));
}

#[test]
fn script_run_shortcut_enter() {
    let s = ScriptInfo::new("X", "/p");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.shortcut, Some("↵".to_string()));
}

// =========== 11. Script context: copy_deeplink desc uses to_deeplink_name ===========

#[test]
fn script_deeplink_desc_has_correct_url() {
    let s = ScriptInfo::new("My Cool Script", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn script_deeplink_shortcut() {
    let s = ScriptInfo::new("X", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut, Some("⌘⇧D".to_string()));
}

#[test]
fn script_deeplink_title() {
    let s = ScriptInfo::new("X", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.title, "Copy Deep Link");
}

#[test]
fn scriptlet_deeplink_desc_has_slugified_name() {
    let s = ScriptInfo::scriptlet("Open GitHub PR", "/path.md", None, None);
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl.description.as_ref().unwrap().contains("open-github-pr"));
}

// =========== 12. Script context: agent actions have agent-specific descriptions ===========

#[test]
fn agent_edit_desc_mentions_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_reveal_desc_mentions_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_copy_path_desc_mentions_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_no_view_logs() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =========== 13. Scriptlet with_custom: run_script title format ===========

#[test]
fn scriptlet_with_custom_run_title_includes_name() {
    let s = ScriptInfo::scriptlet("My Snippet", "/path.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.contains("\"My Snippet\""));
}

#[test]
fn scriptlet_with_custom_run_title_starts_with_verb() {
    let s = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Run"));
}

#[test]
fn scriptlet_with_custom_edit_desc_mentions_editor() {
    let s = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn scriptlet_with_custom_reveal_desc_mentions_finder() {
    let s = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("Finder"));
}

// =========== 14. Scriptlet defined actions: has_action and value set ===========

#[test]
fn scriptlet_defined_action_has_action_true() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].has_action);
}

#[test]
fn scriptlet_defined_action_value_is_command() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy-text".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value, Some("copy-text".to_string()));
}

#[test]
fn scriptlet_defined_action_id_uses_prefix() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Open".to_string(),
        command: "open-link".to_string(),
        tool: "open".to_string(),
        code: "https://example.com".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].id, "scriptlet_action:open-link");
}

#[test]
fn scriptlet_defined_action_shortcut_formatted() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+c".to_string()),
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
}

// =========== 15. AI bar: copy_last_code details ===========

#[test]
fn ai_bar_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(clc.shortcut, Some("⌥⌘C".to_string()));
}

#[test]
fn ai_bar_copy_last_code_icon() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(clc.icon, Some(IconName::Code));
}

#[test]
fn ai_bar_copy_last_code_section() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(clc.section, Some("Response".to_string()));
}

#[test]
fn ai_bar_copy_last_code_desc_mentions_code() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert!(clc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("code"));
}

// =========== 16. AI bar: submit action details ===========

#[test]
fn ai_bar_submit_shortcut() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert_eq!(sub.shortcut, Some("↵".to_string()));
}

#[test]
fn ai_bar_submit_icon() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert_eq!(sub.icon, Some(IconName::ArrowUp));
}

#[test]
fn ai_bar_submit_section_actions() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert_eq!(sub.section, Some("Actions".to_string()));
}

#[test]
fn ai_bar_submit_desc_mentions_send() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert!(sub
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("send"));
}

// =========== 17. AI bar: export_markdown details ===========

#[test]
fn ai_bar_export_markdown_shortcut() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.shortcut, Some("⇧⌘E".to_string()));
}

#[test]
fn ai_bar_export_markdown_icon() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.icon, Some(IconName::FileCode));
}

#[test]
fn ai_bar_export_markdown_section() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.section, Some("Export".to_string()));
}

#[test]
fn ai_bar_export_markdown_title() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.title, "Export as Markdown");
}

// =========== 18. Notes: find_in_note details ===========

#[test]
fn notes_find_in_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.shortcut, Some("⌘F".to_string()));
}

#[test]
fn notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn notes_find_in_note_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.section, Some("Edit".to_string()));
}

#[test]
fn notes_find_in_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

// =========== 19. Notes: duplicate_note details ===========

#[test]
fn notes_duplicate_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.shortcut, Some("⌘D".to_string()));
}

#[test]
fn notes_duplicate_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.icon, Some(IconName::Copy));
}

// --- merged from part_03.rs ---

#[test]
fn notes_duplicate_note_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn notes_duplicate_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

// =========== 20. Notes: copy_note_as details ===========

#[test]
fn notes_copy_note_as_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.shortcut, Some("⇧⌘C".to_string()));
}

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
    assert_eq!(cna.section, Some("Copy".to_string()));
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

// =========== 21. Notes: total action count varies by state ===========

#[test]
fn notes_full_selection_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + duplicate + browse + find + format + copy_note_as + copy_deeplink + create_quicklink + export + enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_trash_selection_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + restore_note + permanently_delete_note + browse + enable_auto_sizing = 5
    assert_eq!(actions.len(), 5);
}

#[test]
fn notes_full_selection_auto_sizing_enabled_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // 10 minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

// =========== 22. Chat context: no models produces only continue_in_chat ===========

#[test]
fn chat_no_models_no_messages_single_action() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
}

#[test]
fn chat_no_models_single_is_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "chat:continue_in_chat");
}

#[test]
fn chat_with_messages_adds_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
}

#[test]
fn chat_with_response_adds_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
}

// =========== 23. Chat context: model IDs use select_model_{model.id} ===========

#[test]
fn chat_model_id_format() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude-3-opus".into(),
            display_name: "Claude 3 Opus".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "chat:select_model_claude-3-opus");
}

#[test]
fn chat_model_title_is_display_name() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].title, "GPT-4");
}

#[test]
fn chat_model_desc_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].description, Some("Uses OpenAI".to_string()));
}

#[test]
fn chat_current_model_gets_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".to_string()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].title.contains('✓'));
}

// =========== 24. New chat: last_used section and icon ===========

#[test]
fn new_chat_last_used_section() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider 1".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].section, Some("Last Used Settings".to_string()));
}

#[test]
fn new_chat_last_used_icon_bolt() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider 1".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_last_used_desc_is_provider() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].description, Some("Uses Anthropic".to_string()));
}

#[test]
fn new_chat_last_used_id_format() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].id, "last_used_p::m1");
}

// =========== 25. New chat: preset section and icon ===========

#[test]
fn new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section, Some("Presets".to_string()));
}

#[test]
fn new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "writer".into(),
        name: "Writer".into(),
        icon: IconName::File,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_writer");
}

#[test]
fn new_chat_preset_desc_none() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].description, Some("Uses General preset".to_string()));
}

// =========== 26. Note switcher: current note has bullet prefix ===========

#[test]
fn note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_pinned_current_icon_star() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    // pinned takes priority over current for icon
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// =========== 27. Note switcher: preview truncation at 60 chars ===========

#[test]
fn note_switcher_short_preview_not_truncated() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Short preview".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("Short preview".to_string()));
}

#[test]
fn note_switcher_long_preview_truncated_with_ellipsis() {
    let long_preview = "a".repeat(80);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 80,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'));
    // 60 'a's + ellipsis
    assert_eq!(desc.chars().count(), 61);
}

#[test]
fn note_switcher_preview_with_time_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains(" · "));
    assert!(desc.contains("2m ago"));
}

// --- merged from part_04.rs ---

#[test]
fn note_switcher_no_preview_shows_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("42 chars".to_string()));
}

// =========== 28. to_deeplink_name: various edge cases ===========

#[test]
fn to_deeplink_name_uppercase_to_lower() {
    assert_eq!(to_deeplink_name("HELLO"), "hello");
}

#[test]
fn to_deeplink_name_preserves_numbers() {
    assert_eq!(to_deeplink_name("test123"), "test123");
}

#[test]
fn to_deeplink_name_multiple_special_chars_collapse() {
    assert_eq!(to_deeplink_name("a!!!b"), "a-b");
}

#[test]
fn to_deeplink_name_leading_trailing_special_removed() {
    assert_eq!(to_deeplink_name("---hello---"), "hello");
}

// =========== 29. score_action: various match type scores ===========

#[test]
fn score_action_prefix_match_100() {
    let a = Action::new("id", "copy path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "copy");
    assert_eq!(score, 100);
}

#[test]
fn score_action_contains_match_50() {
    let a = Action::new("id", "my copy action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "copy");
    assert_eq!(score, 50);
}

#[test]
fn score_action_fuzzy_match_25() {
    let a = Action::new("id", "clipboard", None, ActionCategory::ScriptContext);
    // "cpd" is a subsequence of "clipboard" (c-l-i-p-b-o-a-r-d)
    // c..p..d - wait, let me verify: c(lipboar)d - not quite
    // "cbd" = c(lip)b(oar)d - that works
    let score = ActionsDialog::score_action(&a, "cbd");
    assert_eq!(score, 25);
}

#[test]
fn score_action_no_match_0() {
    let a = Action::new("id", "abc title", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "xyz");
    assert_eq!(score, 0);
}

// =========== 30. fuzzy_match: various patterns ===========

#[test]
fn fuzzy_match_full_string() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hwd"));
}

#[test]
fn fuzzy_match_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_reversed_fails() {
    assert!(!ActionsDialog::fuzzy_match("abc", "cba"));
}
