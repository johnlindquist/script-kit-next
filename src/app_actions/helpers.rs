// Actions handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: handle_action, trigger_action_by_name

pub(crate) const HUD_SHORT_MS: u64 = 1500;
pub(crate) const HUD_MEDIUM_MS: u64 = 2000;
pub(crate) const HUD_LONG_MS: u64 = 3000;

fn select_clipboard_entry_meta<'a>(
    entries: &'a [clipboard_history::ClipboardEntryMeta],
    filter: &str,
    selected_index: usize,
) -> Option<&'a clipboard_history::ClipboardEntryMeta> {
    if filter.is_empty() {
        if entries.is_empty() {
            return None;
        }
        let clamped_index = selected_index.min(entries.len().saturating_sub(1));
        return entries.get(clamped_index);
    }

    let filter_lower = filter.to_lowercase();
    let filtered_entries: Vec<_> = entries
        .iter()
        .filter(|entry| entry.text_preview.to_lowercase().contains(&filter_lower))
        .collect();

    if filtered_entries.is_empty() {
        return None;
    }

    let clamped_index = selected_index.min(filtered_entries.len().saturating_sub(1));
    filtered_entries.get(clamped_index).copied()
}

fn clipboard_pin_action_success_hud(action_id: &str) -> Option<&'static str> {
    match action_id {
        "clipboard_pin" => Some("Pinned"),
        "clipboard_unpin" => Some("Unpinned"),
        _ => None,
    }
}

fn file_search_action_success_hud(action_id: &str) -> Option<&'static str> {
    match action_id {
        "open_file" | "open_directory" => Some("Opened"),
        "quick_look" => Some("Quick Look opened"),
        "open_with" => Some("Open With opened"),
        "show_info" => Some("Info opened"),
        _ => None,
    }
}

fn file_search_action_error_hud_prefix(action_id: &str) -> Option<&'static str> {
    match action_id {
        "open_file" | "open_directory" => Some("Open failed"),
        "quick_look" => Some("Quick Look failed"),
        "open_with" => Some("Open With failed"),
        "show_info" => Some("Show Info failed"),
        _ => None,
    }
}

fn should_transition_to_script_list_after_action(current_view: &AppView) -> bool {
    matches!(current_view, AppView::ScriptList | AppView::ActionsDialog)
}

fn selection_required_message_for_action(action_id: &str) -> &'static str {
    match action_id {
        "copy_path" => "Select an item to copy its path.",
        "copy_deeplink" => "Select an item to copy its deeplink.",
        "configure_shortcut" | "add_shortcut" | "update_shortcut" => {
            "Select an item to configure its shortcut."
        }
        "remove_shortcut" => "Select an item to remove its shortcut.",
        "add_alias" | "update_alias" => "Select an item to add or update its alias.",
        "remove_alias" => "Select an item to remove its alias.",
        "edit_scriptlet" => "Select a scriptlet to edit.",
        "reveal_scriptlet_in_finder" => "Select a scriptlet to reveal in Finder.",
        "copy_scriptlet_path" => "Select a scriptlet to copy its path.",
        "copy_content" => "Select a script, agent, or scriptlet to copy its content.",
        "reset_ranking" => "Select an item to reset its ranking.",
        action if action.starts_with("scriptlet_action:") => {
            "Select a scriptlet to run this action."
        }
        _ => "Select an item to continue.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptRemovalTarget {
    path: std::path::PathBuf,
    name: String,
    item_kind: &'static str,
}

fn extract_scriptlet_source_path(
    file_path_with_anchor: Option<&String>,
) -> Option<std::path::PathBuf> {
    file_path_with_anchor
        .and_then(|path| path.split('#').next())
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from)
}

fn script_removal_target_from_result(
    result: &crate::scripts::SearchResult,
) -> Option<ScriptRemovalTarget> {
    match result {
        crate::scripts::SearchResult::Script(m) => Some(ScriptRemovalTarget {
            path: m.script.path.clone(),
            name: m.script.name.clone(),
            item_kind: "script",
        }),
        crate::scripts::SearchResult::Scriptlet(m) => {
            let path = extract_scriptlet_source_path(m.scriptlet.file_path.as_ref())?;
            Some(ScriptRemovalTarget {
                path,
                name: m.scriptlet.name.clone(),
                item_kind: "scriptlet",
            })
        }
        crate::scripts::SearchResult::Agent(m) => Some(ScriptRemovalTarget {
            path: m.agent.path.clone(),
            name: m.agent.name.clone(),
            item_kind: "agent",
        }),
        _ => None,
    }
}

fn move_path_to_trash(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let escaped_path = crate::utils::escape_applescript_string(&path.to_string_lossy());
        let script = format!(
            r#"tell application "Finder"
    delete POSIX file "{}"
end tell"#,
            escaped_path
        );

        let status = std::process::Command::new("osascript")
            .args(["-e", &script])
            .status()
            .map_err(|err| format!("failed to launch osascript: {err}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("osascript exited with status {}", status))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .map_err(|err| format!("failed to remove directory '{}': {err}", path.display()))
        } else {
            std::fs::remove_file(path)
                .map_err(|err| format!("failed to remove file '{}': {err}", path.display()))
        }
    }
}

#[cfg(test)]
mod app_actions_tests {
    use super::{
        clipboard_pin_action_success_hud, extract_scriptlet_source_path,
        file_search_action_error_hud_prefix, file_search_action_success_hud,
        script_removal_target_from_result, select_clipboard_entry_meta,
        selection_required_message_for_action, should_transition_to_script_list_after_action,
        ScriptRemovalTarget,
    };
    use crate::clipboard_history::{ClipboardEntryMeta, ContentType};
    use crate::scripts;
    use crate::AppView;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn entry(id: &str, preview: &str) -> ClipboardEntryMeta {
        ClipboardEntryMeta {
            id: id.to_string(),
            content_type: ContentType::Text,
            timestamp: 0,
            pinned: false,
            text_preview: preview.to_string(),
            image_width: None,
            image_height: None,
            byte_size: 0,
            ocr_text: None,
        }
    }

    fn read(path: &str) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"))
    }

    fn count_occurrences(haystack: &str, needle: &str) -> usize {
        haystack.match_indices(needle).count()
    }

    #[test]
    fn test_select_clipboard_entry_meta_filters_and_clamps() {
        let entries = vec![entry("1", "Alpha"), entry("2", "Beta"), entry("3", "Gamma")];

        let filtered = select_clipboard_entry_meta(&entries, "et", 0).unwrap();
        assert_eq!(filtered.id, "2");

        let clamped = select_clipboard_entry_meta(&entries, "", 99).unwrap();
        assert_eq!(clamped.id, "3");
    }

    #[test]
    fn test_clipboard_pin_action_success_hud_messages() {
        assert_eq!(
            clipboard_pin_action_success_hud("clipboard_pin"),
            Some("Pinned")
        );
        assert_eq!(
            clipboard_pin_action_success_hud("clipboard_unpin"),
            Some("Unpinned")
        );
        assert_eq!(clipboard_pin_action_success_hud("clipboard_share"), None);
    }

    #[test]
    fn test_file_search_action_success_hud_messages() {
        assert_eq!(file_search_action_success_hud("open_file"), Some("Opened"));
        assert_eq!(
            file_search_action_success_hud("open_directory"),
            Some("Opened")
        );
        assert_eq!(
            file_search_action_success_hud("quick_look"),
            Some("Quick Look opened")
        );
        assert_eq!(
            file_search_action_success_hud("open_with"),
            Some("Open With opened")
        );
        assert_eq!(
            file_search_action_success_hud("show_info"),
            Some("Info opened")
        );
        assert_eq!(file_search_action_success_hud("copy_filename"), None);
    }

    #[test]
    fn test_file_search_action_error_hud_prefixes() {
        assert_eq!(
            file_search_action_error_hud_prefix("open_file"),
            Some("Open failed")
        );
        assert_eq!(
            file_search_action_error_hud_prefix("open_directory"),
            Some("Open failed")
        );
        assert_eq!(
            file_search_action_error_hud_prefix("quick_look"),
            Some("Quick Look failed")
        );
        assert_eq!(
            file_search_action_error_hud_prefix("open_with"),
            Some("Open With failed")
        );
        assert_eq!(
            file_search_action_error_hud_prefix("show_info"),
            Some("Show Info failed")
        );
        assert_eq!(file_search_action_error_hud_prefix("copy_filename"), None);
    }

    #[test]
    fn test_selection_required_message_for_action_returns_action_specific_guidance() {
        assert_eq!(
            selection_required_message_for_action("copy_path"),
            "Select an item to copy its path."
        );
        assert_eq!(
            selection_required_message_for_action("remove_shortcut"),
            "Select an item to remove its shortcut."
        );
        assert_eq!(
            selection_required_message_for_action("scriptlet_action:test"),
            "Select a scriptlet to run this action."
        );
    }

    #[test]
    fn test_selection_required_message_for_action_returns_safe_default() {
        assert_eq!(
            selection_required_message_for_action("unknown-action"),
            "Select an item to continue."
        );
    }

    #[test]
    fn test_should_transition_to_script_list_after_action_is_context_aware() {
        assert!(should_transition_to_script_list_after_action(
            &AppView::ScriptList
        ));
        assert!(should_transition_to_script_list_after_action(
            &AppView::ActionsDialog
        ));
        assert!(!should_transition_to_script_list_after_action(
            &AppView::ClipboardHistoryView {
                filter: String::new(),
                selected_index: 0,
            }
        ));
        assert!(!should_transition_to_script_list_after_action(
            &AppView::FileSearchView {
                query: String::new(),
                selected_index: 0,
            }
        ));
    }

    #[test]
    fn test_extract_scriptlet_source_path_removes_anchor() {
        let path_with_anchor = Some("/tmp/snippets/tools.md#open-github".to_string());
        let extracted = extract_scriptlet_source_path(path_with_anchor.as_ref());
        assert_eq!(extracted, Some(PathBuf::from("/tmp/snippets/tools.md")));
    }

    #[test]
    fn test_script_removal_target_from_result_for_script_and_scriptlet() {
        let script_result = scripts::SearchResult::Script(scripts::ScriptMatch {
            script: Arc::new(scripts::Script {
                name: "Deploy".to_string(),
                path: PathBuf::from("/tmp/deploy.ts"),
                ..Default::default()
            }),
            score: 0,
            filename: "deploy.ts".to_string(),
            match_indices: scripts::MatchIndices::default(),
        });

        let script_target = script_removal_target_from_result(&script_result);
        assert_eq!(
            script_target,
            Some(ScriptRemovalTarget {
                path: PathBuf::from("/tmp/deploy.ts"),
                name: "Deploy".to_string(),
                item_kind: "script",
            })
        );

        let scriptlet_result = scripts::SearchResult::Scriptlet(scripts::ScriptletMatch {
            scriptlet: Arc::new(scripts::Scriptlet {
                name: "Open GitHub".to_string(),
                description: Some("Open project page".to_string()),
                code: "https://github.com".to_string(),
                tool: "open".to_string(),
                shortcut: None,
                keyword: None,
                group: Some("Tools".to_string()),
                file_path: Some("/tmp/snippets/tools.md#open-github".to_string()),
                command: Some("open-github".to_string()),
                alias: None,
            }),
            score: 0,
            display_file_path: Some("tools.md#open-github".to_string()),
            match_indices: scripts::MatchIndices::default(),
        });

        let scriptlet_target = script_removal_target_from_result(&scriptlet_result);
        assert_eq!(
            scriptlet_target,
            Some(ScriptRemovalTarget {
                path: PathBuf::from("/tmp/snippets/tools.md"),
                name: "Open GitHub".to_string(),
                item_kind: "scriptlet",
            })
        );
    }

    #[test]
    fn test_edit_actions_show_error_feedback_when_editor_launch_fails() {
        let content = read("src/app_actions/handle_action.rs");

        assert!(
            content.contains("fn launch_editor_with_feedback_async"),
            "Expected async editor launch helper to exist"
        );

        let usage_count =
            count_occurrences(&content, "self.launch_editor_with_feedback_async(&path)");
        assert!(
            usage_count >= 2,
            "Expected edit_script and edit_scriptlet to use async editor launch feedback (found {usage_count} usages)"
        );

        assert!(
            content.contains("this.show_hud(message, Some(HUD_LONG_MS), cx);"),
            "Expected async editor launch failure to surface a HUD error message"
        );
    }

    #[test]
    fn test_reveal_actions_show_success_hud_after_async_completion() {
        let content = read("src/app_actions/handle_action.rs");

        assert!(
            content.contains("fn reveal_in_finder_with_feedback_async"),
            "Expected async reveal helper to exist"
        );

        let usage_count =
            count_occurrences(&content, "self.reveal_in_finder_with_feedback_async(&path)")
                + count_occurrences(&content, "self.reveal_in_finder_with_feedback_async(path)");
        assert!(
            usage_count >= 2,
            "Expected reveal actions to use async reveal feedback helper (found {usage_count} usages)"
        );

        assert!(
            content.contains("let Ok(reveal_result) = reveal_result_rx.recv().await else {"),
            "Expected reveal actions to await reveal completion before showing HUD"
        );

        assert!(
            content.contains("this.show_hud(\"Opened in Finder\".to_string(), Some(HUD_SHORT_MS), cx);"),
            "Expected reveal success HUD to be emitted from async completion callback"
        );
    }
}
