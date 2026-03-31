use super::types::{Action, ActionCategory};
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;

fn has_missing_file_context_fields(name: &str, path: &str) -> bool {
    name.trim().is_empty() || path.trim().is_empty()
}

/// Get actions specific to a file search result
///
/// Actions vary based on whether the item is a file or directory:
/// - Directory: `file:open_directory` as primary
/// - File: `file:open_file` as primary, plus Quick Look and Attach to AI (macOS)
///
/// Core file-manager verbs for both: reveal_in_finder, open_in_editor,
/// open_in_terminal, copy_path, copy_filename, move_to_trash
pub fn get_file_context_actions(file_info: &FileInfo) -> Vec<Action> {
    if has_missing_file_context_fields(&file_info.name, &file_info.path) {
        tracing::warn!(
            target: "script_kit::actions",
            name = %file_info.name,
            path = %file_info.path,
            is_dir = file_info.is_dir,
            "Invalid file context info: missing path or name; returning no actions"
        );
        return Vec::new();
    }

    let mut actions = Vec::new();

    tracing::debug!(
        target: "script_kit::actions",
        name = %file_info.name,
        is_dir = file_info.is_dir,
        "Building file context actions"
    );

    // Primary action: open file or directory
    if file_info.is_dir {
        actions.push(
            Action::new(
                "file:open_directory",
                format!("Open \"{}\"", file_info.name),
                Some("Opens this folder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_icon(IconName::FolderOpen),
        );
    } else {
        actions.push(
            Action::new(
                "file:open_file",
                format!("Open \"{}\"", file_info.name),
                Some("Opens with the default app".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_icon(IconName::File),
        );
    }

    actions.push(
        Action::new(
            "file:reveal_in_finder",
            "Reveal in Finder",
            Some("Shows this item in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F")
        .with_icon(IconName::FolderOpen),
    );

    actions.push(
        Action::new(
            "file:rename_path",
            "Rename\u{2026}",
            Some("Renames the selected file or folder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}R")
        .with_icon(IconName::Pencil),
    );

    actions.push(
        Action::new(
            "file:move_path",
            "Move\u{2026}",
            Some("Moves the selected file or folder to another folder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{21e7}M")
        .with_icon(IconName::FolderOpen),
    );

    actions.push(
        Action::new(
            "file:open_in_editor",
            "Open in Editor",
            Some("Opens this item in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E")
        .with_icon(IconName::Pencil),
    );

    actions.push(
        Action::new(
            "file:open_in_terminal",
            "Open in Terminal",
            Some("Opens a terminal at this location".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T")
        .with_icon(IconName::Terminal),
    );

    if !file_info.is_dir {
        actions.push(
            Action::new(
                "file:attach_to_ai",
                "Attach to AI Chat",
                Some("Attaches this file to the AI chat window".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌃⌘A")
            .with_icon(IconName::MessageCircle),
        );
    }

    #[cfg(target_os = "macos")]
    if !file_info.is_dir {
        actions.push(
            Action::new(
                "file:quick_look",
                "Quick Look",
                Some("Previews this item with Quick Look".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘Y")
            .with_icon(IconName::File),
        );
    }

    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "file:show_info",
            "Show Info",
            Some("Shows file information in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I")
        .with_icon(IconName::File),
    );

    actions.push(
        Action::new(
            "file:copy_path",
            "Copy Path",
            Some("Copies the full path to the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C")
        .with_icon(IconName::Copy),
    );

    actions.push(
        Action::new(
            "file:copy_filename",
            "Copy Filename",
            Some("Copies only the filename to the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘C")
        .with_icon(IconName::Copy),
    );

    actions.push(
        Action::new(
            "file:move_to_trash",
            "Move to Trash",
            Some(format!(
                "Moves this {} to the Trash",
                if file_info.is_dir { "folder" } else { "file" }
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫")
        .with_icon(IconName::Trash),
    );

    actions
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use super::*;
    use crate::file_search::{FileInfo, FileType};

    #[test]
    fn test_get_file_context_actions_includes_show_info_on_macos() {
        let file_info = FileInfo {
            path: "/tmp/example.txt".to_string(),
            name: "example.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };

        let actions = get_file_context_actions(&file_info);
        let show_info = actions
            .iter()
            .find(|action| action.id == "file:show_info")
            .expect("missing show_info action");

        assert_eq!(show_info.title, "Show Info");
        assert_eq!(show_info.shortcut.as_deref(), Some("⌘I"));
    }

    #[test]
    fn test_get_file_context_actions_does_not_expose_open_with() {
        let file_info = FileInfo {
            path: "/tmp/example.txt".to_string(),
            name: "example.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };

        let actions = get_file_context_actions(&file_info);
        assert!(
            actions.iter().all(|action| action.id != "file:open_with"),
            "file:open_with should not be exposed from file-search actions"
        );
    }
}

/// Get actions specific to a file/folder path
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action> {
    if has_missing_file_context_fields(&path_info.name, &path_info.path) {
        tracing::warn!(
            target: "script_kit::actions",
            name = %path_info.name,
            path = %path_info.path,
            is_dir = path_info.is_dir,
            "Invalid path context info: missing path or name; returning no actions"
        );
        return Vec::new();
    }

    let mut actions = vec![
        Action::new(
            "file:copy_path",
            "Copy Path",
            Some("Copies the full path to the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C")
        .with_icon(IconName::Copy),
        Action::new(
            "file:open_in_finder",
            "Reveal in Finder",
            Some("Shows this item in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F")
        .with_icon(IconName::FolderOpen),
        Action::new(
            "file:open_in_editor",
            "Open in Editor",
            Some("Opens this item in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E")
        .with_icon(IconName::Pencil),
        Action::new(
            "file:open_in_terminal",
            "Open in Terminal",
            Some("Opens a terminal at this location".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T")
        .with_icon(IconName::Terminal),
        Action::new(
            "file:copy_filename",
            "Copy Filename",
            Some("Copies only the filename".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Copy),
        Action::new(
            "file:move_to_trash",
            "Move to Trash",
            Some(format!(
                "Moves this {} to the Trash",
                if path_info.is_dir { "folder" } else { "file" }
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫")
        .with_icon(IconName::Trash),
    ];

    if path_info.is_dir {
        actions.insert(
            0,
            Action::new(
                "file:open_directory",
                format!("Open \"{}\"", path_info.name),
                Some("Opens this directory".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_icon(IconName::FolderOpen),
        );
    } else {
        actions.insert(
            0,
            Action::new(
                "file:select_file",
                format!("Select \"{}\"", path_info.name),
                Some("Selects this file".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_icon(IconName::File),
        );
    }

    actions
}

#[cfg(test)]
mod namespace_tests {
    use super::*;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};

    fn sample_file_info(is_dir: bool) -> FileInfo {
        FileInfo {
            path: "/tmp/example".to_string(),
            name: "example".to_string(),
            file_type: if is_dir {
                FileType::Directory
            } else {
                FileType::File
            },
            is_dir,
        }
    }

    fn sample_path_info(is_dir: bool) -> PathInfo {
        PathInfo {
            path: "/tmp/example".to_string(),
            name: "example".to_string(),
            is_dir,
        }
    }

    #[test]
    fn test_get_file_context_actions_prefixes_ids_with_file_namespace() {
        let file_actions = get_file_context_actions(&sample_file_info(false));
        assert!(file_actions
            .iter()
            .all(|action| action.id.starts_with("file:")));

        let directory_actions = get_file_context_actions(&sample_file_info(true));
        assert!(directory_actions
            .iter()
            .all(|action| action.id.starts_with("file:")));
    }

    #[test]
    fn test_get_path_context_actions_prefixes_ids_with_file_namespace() {
        let file_actions = get_path_context_actions(&sample_path_info(false));
        assert!(file_actions
            .iter()
            .all(|action| action.id.starts_with("file:")));

        let directory_actions = get_path_context_actions(&sample_path_info(true));
        assert!(directory_actions
            .iter()
            .all(|action| action.id.starts_with("file:")));
    }

    #[test]
    fn test_get_file_context_actions_returns_empty_when_required_fields_missing() {
        let mut file_info = sample_file_info(false);
        file_info.path = "   ".to_string();

        let actions = get_file_context_actions(&file_info);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_path_context_actions_returns_empty_when_required_fields_missing() {
        let mut path_info = sample_path_info(true);
        path_info.name = "   ".to_string();

        let actions = get_path_context_actions(&path_info);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_file_context_actions_includes_attach_to_ai_for_files() {
        let actions = get_file_context_actions(&sample_file_info(false));
        let attach = actions
            .iter()
            .find(|action| action.id == "file:attach_to_ai")
            .expect("missing file attach_to_ai action");

        assert_eq!(attach.title, "Attach to AI Chat");
        assert_eq!(
            attach.description.as_deref(),
            Some("Attaches this file to the AI chat window")
        );
        assert_eq!(attach.shortcut.as_deref(), Some("⌃⌘A"));
        assert_eq!(attach.icon, Some(IconName::MessageCircle));
    }

    #[test]
    fn test_get_file_context_actions_excludes_attach_to_ai_for_directories() {
        let actions = get_file_context_actions(&sample_file_info(true));

        assert!(
            actions
                .iter()
                .all(|action| action.id != "file:attach_to_ai"),
            "directories should not include attach_to_ai action"
        );
    }

    #[test]
    fn test_get_file_context_actions_includes_core_file_manager_verbs() {
        let file_info = FileInfo {
            path: "/tmp/example.txt".to_string(),
            name: "example.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };

        let actions = get_file_context_actions(&file_info);
        let ids: Vec<&str> = actions.iter().map(|action| action.id.as_str()).collect();

        assert!(ids.contains(&"file:open_file"), "missing file:open_file");
        assert!(
            ids.contains(&"file:reveal_in_finder"),
            "missing file:reveal_in_finder"
        );
        assert!(
            ids.contains(&"file:open_in_editor"),
            "missing file:open_in_editor"
        );
        assert!(
            ids.contains(&"file:open_in_terminal"),
            "missing file:open_in_terminal"
        );
        assert!(ids.contains(&"file:copy_path"), "missing file:copy_path");
        assert!(
            ids.contains(&"file:copy_filename"),
            "missing file:copy_filename"
        );
        assert!(
            ids.contains(&"file:move_to_trash"),
            "missing file:move_to_trash"
        );
    }

    #[test]
    fn test_get_file_context_actions_directory_uses_correct_primary_and_trash_label() {
        let dir_info = FileInfo {
            path: "/tmp/my_folder".to_string(),
            name: "my_folder".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };

        let actions = get_file_context_actions(&dir_info);
        let ids: Vec<&str> = actions.iter().map(|action| action.id.as_str()).collect();

        assert!(
            ids.contains(&"file:open_directory"),
            "directory should have file:open_directory"
        );
        assert!(
            !ids.contains(&"file:open_file"),
            "directory should not have file:open_file"
        );
        assert!(
            !ids.contains(&"file:attach_to_ai"),
            "directory should not have file:attach_to_ai"
        );

        let trash = actions
            .iter()
            .find(|action| action.id == "file:move_to_trash")
            .expect("missing move_to_trash for directory");
        assert_eq!(
            trash.description.as_deref(),
            Some("Moves this folder to the Trash")
        );
    }

    #[test]
    fn test_file_and_path_copy_actions_share_copy_icon() {
        let file_actions = get_file_context_actions(&sample_file_info(false));
        let path_actions = get_path_context_actions(&sample_path_info(false));

        let file_copy_path = file_actions
            .iter()
            .find(|action| action.id == "file:copy_path")
            .expect("missing file copy_path action");
        let file_copy_filename = file_actions
            .iter()
            .find(|action| action.id == "file:copy_filename")
            .expect("missing file copy_filename action");
        let path_copy_path = path_actions
            .iter()
            .find(|action| action.id == "file:copy_path")
            .expect("missing path copy_path action");
        let path_copy_filename = path_actions
            .iter()
            .find(|action| action.id == "file:copy_filename")
            .expect("missing path copy_filename action");

        assert_eq!(file_copy_path.icon, Some(IconName::Copy));
        assert_eq!(file_copy_filename.icon, Some(IconName::Copy));
        assert_eq!(path_copy_path.icon, Some(IconName::Copy));
        assert_eq!(path_copy_filename.icon, Some(IconName::Copy));
    }
}
