use super::types::{Action, ActionCategory};
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;

// =========================================================================
// Shared secondary-command contract for file search
// =========================================================================

/// A single secondary command definition that drives the file-search action
/// list, footer text, and keyboard dispatch from one source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileSearchSecondaryCommand {
    pub action_id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub shortcut: &'static str,
    pub footer_label: &'static str,
    pub icon: IconName,
    pub key: &'static str,
    pub requires_shift: bool,
    /// When true, this command is only available for files (not directories).
    pub files_only: bool,
    /// When true, this command is only available on macOS.
    pub macos_only: bool,
}

impl FileSearchSecondaryCommand {
    /// Whether this command applies to the given item type.
    pub(crate) fn supports(self, is_dir: bool) -> bool {
        if self.files_only && is_dir {
            return false;
        }
        #[cfg(not(target_os = "macos"))]
        if self.macos_only {
            return false;
        }
        true
    }

    /// Whether the given key event matches this command for the given item.
    #[allow(dead_code)] // used by binary target via include!() in main.rs
    pub(crate) fn matches(self, key: &str, has_cmd: bool, has_shift: bool, is_dir: bool) -> bool {
        if !has_cmd || has_shift != self.requires_shift || !self.supports(is_dir) {
            return false;
        }
        match self.key {
            "backspace_or_delete" => {
                key.eq_ignore_ascii_case("backspace") || key.eq_ignore_ascii_case("delete")
            }
            expected => key.eq_ignore_ascii_case(expected),
        }
    }

    /// Build an `Action` from this command definition.
    pub(crate) fn to_action(self, file_info: &FileInfo) -> Action {
        let description = if self.action_id == "move_to_trash" {
            format!(
                "Moves this {} to the Trash",
                if file_info.is_dir { "folder" } else { "file" }
            )
        } else {
            self.description.to_string()
        };

        Action::new(
            format!("file:{}", self.action_id),
            self.title,
            Some(description),
            ActionCategory::ScriptContext,
        )
        .with_shortcut(self.shortcut)
        .with_icon(self.icon)
    }
}

/// The canonical list of secondary commands for the file-search surface.
/// Order here determines action-list order and footer label order.
pub(crate) const FILE_SEARCH_SECONDARY_COMMANDS: [FileSearchSecondaryCommand; 10] = [
    FileSearchSecondaryCommand {
        action_id: "rename_path",
        title: "Rename\u{2026}",
        description: "Renames the selected file or folder",
        shortcut: "\u{2318}R",
        footer_label: "\u{2318}R Rename",
        icon: IconName::Pencil,
        key: "r",
        requires_shift: false,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "move_path",
        title: "Move\u{2026}",
        description: "Moves the selected file or folder to another folder",
        shortcut: "\u{2318}\u{21e7}M",
        footer_label: "\u{2318}\u{21e7}M Move",
        icon: IconName::FolderOpen,
        key: "m",
        requires_shift: true,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "duplicate_path",
        title: "Duplicate",
        description: "Creates a copy of the selected file or folder",
        shortcut: "\u{2318}D",
        footer_label: "\u{2318}D Duplicate",
        icon: IconName::Copy,
        key: "d",
        requires_shift: false,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "copy_filename",
        title: "Copy Filename",
        description: "Copies only the filename to the clipboard",
        shortcut: "\u{2318}C",
        footer_label: "\u{2318}C Name",
        icon: IconName::Copy,
        key: "c",
        requires_shift: false,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "open_in_editor",
        title: "Open in Editor",
        description: "Opens this item in $EDITOR",
        shortcut: "\u{2318}E",
        footer_label: "\u{2318}E Editor",
        icon: IconName::Pencil,
        key: "e",
        requires_shift: false,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "copy_path",
        title: "Copy Path",
        description: "Copies the full path to the clipboard",
        shortcut: "\u{2318}\u{21e7}C",
        footer_label: "\u{2318}\u{21e7}C Path",
        icon: IconName::Copy,
        key: "c",
        requires_shift: true,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "move_to_trash",
        title: "Move to Trash",
        description: "Moves the selected item to the Trash",
        shortcut: "\u{2318}\u{232b}",
        footer_label: "\u{2318}\u{232b} Trash",
        icon: IconName::Trash,
        key: "backspace_or_delete",
        requires_shift: false,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "open_in_terminal",
        title: "Open in Terminal",
        description: "Opens a terminal at this location",
        shortcut: "\u{2318}T",
        footer_label: "\u{2318}T Terminal",
        icon: IconName::Terminal,
        key: "t",
        requires_shift: false,
        files_only: false,
        macos_only: false,
    },
    FileSearchSecondaryCommand {
        action_id: "quick_look",
        title: "Quick Look",
        description: "Previews this item with Quick Look",
        shortcut: "\u{2318}Y",
        footer_label: "\u{2318}Y Quick Look",
        icon: IconName::File,
        key: "y",
        requires_shift: false,
        files_only: true,
        macos_only: true,
    },
    FileSearchSecondaryCommand {
        action_id: "show_info",
        title: "Show Info",
        description: "Shows file information in Finder",
        shortcut: "\u{2318}I",
        footer_label: "\u{2318}I Info",
        icon: IconName::File,
        key: "i",
        requires_shift: false,
        files_only: false,
        macos_only: true,
    },
];

/// Build the leading footer text from the shared secondary-command contract.
#[allow(dead_code)] // used by binary target via include!() in main.rs
pub(crate) fn build_file_search_footer_leading_text(
    file_info: &FileInfo,
    can_shift_tab_up: bool,
) -> String {
    let mut parts: Vec<&str> = Vec::new();

    if can_shift_tab_up {
        parts.push("\u{21e7}Tab Up");
    }

    for command in FILE_SEARCH_SECONDARY_COMMANDS.iter().copied() {
        if command.supports(file_info.is_dir) {
            parts.push(command.footer_label);
        }
    }

    parts.join(" \u{b7} ")
}

/// Resolve a keyboard shortcut to a secondary action ID using the shared contract.
#[allow(dead_code)] // used by binary target via include!() in main.rs
pub(crate) fn resolve_file_search_secondary_action_id(
    key: &str,
    has_cmd: bool,
    has_shift: bool,
    file_info: &FileInfo,
) -> Option<&'static str> {
    FILE_SEARCH_SECONDARY_COMMANDS
        .iter()
        .copied()
        .find(|command| command.matches(key, has_cmd, has_shift, file_info.is_dir))
        .map(|command| command.action_id)
}

fn has_missing_file_context_fields(name: &str, path: &str) -> bool {
    name.trim().is_empty() || path.trim().is_empty()
}

/// Get actions specific to a file search result.
///
/// Actions vary based on whether the item is a file or directory:
/// - Directory: `file:open_directory` as primary
/// - File: `file:open_file` as primary, plus Quick Look and Attach to AI (macOS)
///
/// Secondary actions are driven by `FILE_SEARCH_SECONDARY_COMMANDS` — the
/// single source of truth shared with the footer and key handler.
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
            .with_shortcut("\u{21b5}")
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
            .with_shortcut("\u{21b5}")
            .with_icon(IconName::File),
        );
    }

    // Reveal in Finder — kept explicit (not a secondary keyboard command)
    actions.push(
        Action::new(
            "file:reveal_in_finder",
            "Reveal in Finder",
            Some("Shows this item in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{21e7}F")
        .with_icon(IconName::FolderOpen),
    );

    // Secondary commands from the shared contract
    for command in FILE_SEARCH_SECONDARY_COMMANDS.iter().copied() {
        if !command.supports(file_info.is_dir) {
            continue;
        }
        actions.push(command.to_action(file_info));

        // Attach to AI Chat — inserted after open_in_terminal for files only
        if command.action_id == "open_in_terminal" && !file_info.is_dir {
            actions.push(
                Action::new(
                    "file:attach_to_ai",
                    "Attach to AI Chat",
                    Some("Attaches this file to the AI chat window".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("\u{2303}\u{2318}A")
                .with_icon(IconName::MessageCircle),
            );
        }
    }

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

#[cfg(test)]
mod secondary_command_contract_tests {
    use super::*;
    use crate::file_search::{FileInfo, FileType};

    fn make_file(name: &str, path: &str) -> FileInfo {
        FileInfo {
            path: path.to_string(),
            name: name.to_string(),
            file_type: FileType::File,
            is_dir: false,
        }
    }

    fn make_dir(name: &str, path: &str) -> FileInfo {
        FileInfo {
            path: path.to_string(),
            name: name.to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        }
    }

    // --- Shared contract: stable IDs ---

    #[test]
    fn secondary_command_ids_are_stable() {
        let ids: Vec<&str> = FILE_SEARCH_SECONDARY_COMMANDS
            .iter()
            .map(|c| c.action_id)
            .collect();
        assert_eq!(
            ids,
            vec![
                "rename_path",
                "move_path",
                "duplicate_path",
                "copy_filename",
                "open_in_editor",
                "copy_path",
                "move_to_trash",
                "open_in_terminal",
                "quick_look",
                "show_info",
            ]
        );
    }

    #[test]
    fn secondary_command_shortcuts_are_stable() {
        let shortcuts: Vec<&str> = FILE_SEARCH_SECONDARY_COMMANDS
            .iter()
            .map(|c| c.shortcut)
            .collect();
        assert_eq!(
            shortcuts,
            vec![
                "\u{2318}R",
                "\u{2318}\u{21e7}M",
                "\u{2318}D",
                "\u{2318}C",
                "\u{2318}E",
                "\u{2318}\u{21e7}C",
                "\u{2318}\u{232b}",
                "\u{2318}T",
                "\u{2318}Y",
                "\u{2318}I",
            ]
        );
    }

    // --- Directory: exclude Quick Look, include the rest ---

    #[test]
    fn directory_excludes_quick_look() {
        let dir = make_dir("photos", "/tmp/photos");
        let actions = get_file_context_actions(&dir);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            !ids.contains(&"file:quick_look"),
            "Directories must not have Quick Look"
        );
    }

    #[test]
    fn directory_includes_all_non_file_only_secondary_commands() {
        let dir = make_dir("photos", "/tmp/photos");
        let actions = get_file_context_actions(&dir);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

        for cmd in FILE_SEARCH_SECONDARY_COMMANDS.iter() {
            if cmd.files_only || cmd.macos_only {
                continue;
            }
            let expected_id = format!("file:{}", cmd.action_id);
            assert!(
                ids.contains(&expected_id.as_str()),
                "Directory should have {expected_id}"
            );
        }
    }

    // --- File on macOS: Quick Look and Attach to AI ---

    #[cfg(target_os = "macos")]
    #[test]
    fn file_on_macos_includes_quick_look() {
        let file = make_file("photo.png", "/tmp/photo.png");
        let actions = get_file_context_actions(&file);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            ids.contains(&"file:quick_look"),
            "Files on macOS should have Quick Look"
        );
    }

    #[test]
    fn file_includes_attach_to_ai() {
        let file = make_file("data.csv", "/tmp/data.csv");
        let actions = get_file_context_actions(&file);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            ids.contains(&"file:attach_to_ai"),
            "Files should have Attach to AI Chat"
        );
    }

    #[test]
    fn directory_excludes_attach_to_ai() {
        let dir = make_dir("photos", "/tmp/photos");
        let actions = get_file_context_actions(&dir);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            !ids.contains(&"file:attach_to_ai"),
            "Directories must not have Attach to AI Chat"
        );
    }

    // --- Footer builder ---

    #[test]
    fn footer_for_directory_excludes_quick_look() {
        let dir = make_dir("photos", "/tmp/photos");
        let footer = build_file_search_footer_leading_text(&dir, false);
        assert!(
            !footer.contains("Quick Look"),
            "Directory footer must not mention Quick Look"
        );
    }

    #[test]
    fn footer_for_directory_includes_core_labels() {
        let dir = make_dir("photos", "/tmp/photos");
        let footer = build_file_search_footer_leading_text(&dir, false);
        assert!(footer.contains("Rename"), "Footer should mention Rename");
        assert!(footer.contains("Move"), "Footer should mention Move");
        assert!(footer.contains("Name"), "Footer should mention Name");
        assert!(footer.contains("Editor"), "Footer should mention Editor");
        assert!(footer.contains("Path"), "Footer should mention Path");
        assert!(footer.contains("Trash"), "Footer should mention Trash");
        assert!(
            footer.contains("Terminal"),
            "Footer should mention Terminal"
        );
    }

    #[test]
    fn footer_with_shift_tab_prefix() {
        let dir = make_dir("photos", "/tmp/photos");
        let footer = build_file_search_footer_leading_text(&dir, true);
        assert!(
            footer.starts_with("\u{21e7}Tab Up"),
            "Footer should start with shift-tab when can_shift_tab_up"
        );
    }

    #[test]
    fn footer_without_shift_tab_prefix() {
        let dir = make_dir("photos", "/tmp/photos");
        let footer = build_file_search_footer_leading_text(&dir, false);
        assert!(
            !footer.contains("Tab Up"),
            "Footer should not include shift-tab when can_shift_tab_up is false"
        );
    }

    // --- Keyboard resolver ---

    #[test]
    fn resolver_cmd_c_resolves_to_copy_filename() {
        let dir = make_dir("photos", "/tmp/photos");
        assert_eq!(
            resolve_file_search_secondary_action_id("c", true, false, &dir),
            Some("copy_filename")
        );
    }

    #[test]
    fn resolver_cmd_shift_c_resolves_to_copy_path() {
        let file = make_file("data.csv", "/tmp/data.csv");
        assert_eq!(
            resolve_file_search_secondary_action_id("c", true, true, &file),
            Some("copy_path")
        );
    }

    #[test]
    fn resolver_cmd_y_on_directory_returns_none() {
        let dir = make_dir("photos", "/tmp/photos");
        assert_eq!(
            resolve_file_search_secondary_action_id("y", true, false, &dir),
            None,
            "Quick Look must not resolve for directories"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn resolver_cmd_y_on_file_returns_quick_look() {
        let file = make_file("photo.png", "/tmp/photo.png");
        assert_eq!(
            resolve_file_search_secondary_action_id("y", true, false, &file),
            Some("quick_look")
        );
    }

    #[test]
    fn resolver_cmd_r_resolves_to_rename() {
        let file = make_file("data.csv", "/tmp/data.csv");
        assert_eq!(
            resolve_file_search_secondary_action_id("r", true, false, &file),
            Some("rename_path")
        );
    }

    #[test]
    fn resolver_cmd_shift_m_resolves_to_move() {
        let dir = make_dir("photos", "/tmp/photos");
        assert_eq!(
            resolve_file_search_secondary_action_id("m", true, true, &dir),
            Some("move_path")
        );
    }

    #[test]
    fn resolver_cmd_backspace_resolves_to_trash() {
        let file = make_file("temp.log", "/tmp/temp.log");
        assert_eq!(
            resolve_file_search_secondary_action_id("backspace", true, false, &file),
            Some("move_to_trash")
        );
    }

    #[test]
    fn resolver_cmd_delete_resolves_to_trash() {
        let file = make_file("temp.log", "/tmp/temp.log");
        assert_eq!(
            resolve_file_search_secondary_action_id("delete", true, false, &file),
            Some("move_to_trash")
        );
    }

    #[test]
    fn resolver_without_cmd_returns_none() {
        let file = make_file("data.csv", "/tmp/data.csv");
        assert_eq!(
            resolve_file_search_secondary_action_id("c", false, false, &file),
            None,
            "Without Cmd held, nothing should match"
        );
    }

    #[test]
    fn resolver_cmd_e_resolves_to_open_in_editor() {
        let dir = make_dir("src", "/tmp/src");
        assert_eq!(
            resolve_file_search_secondary_action_id("e", true, false, &dir),
            Some("open_in_editor")
        );
    }

    #[test]
    fn resolver_cmd_t_resolves_to_open_in_terminal() {
        let dir = make_dir("src", "/tmp/src");
        assert_eq!(
            resolve_file_search_secondary_action_id("t", true, false, &dir),
            Some("open_in_terminal")
        );
    }

    // --- Action generation uses shared contract ---

    #[test]
    fn get_file_context_actions_secondary_ids_match_contract() {
        let file = make_file("data.csv", "/tmp/data.csv");
        let actions = get_file_context_actions(&file);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

        for cmd in FILE_SEARCH_SECONDARY_COMMANDS.iter() {
            if !cmd.supports(file.is_dir) {
                continue;
            }
            let expected_id = format!("file:{}", cmd.action_id);
            assert!(
                ids.contains(&expected_id.as_str()),
                "File actions should include {expected_id}"
            );
        }
    }

    #[test]
    fn get_file_context_actions_secondary_shortcuts_match_contract() {
        let file = make_file("data.csv", "/tmp/data.csv");
        let actions = get_file_context_actions(&file);

        for cmd in FILE_SEARCH_SECONDARY_COMMANDS.iter() {
            if !cmd.supports(file.is_dir) {
                continue;
            }
            let expected_id = format!("file:{}", cmd.action_id);
            let action = actions
                .iter()
                .find(|a| a.id == expected_id)
                .unwrap_or_else(|| panic!("missing action {expected_id}"));
            assert_eq!(
                action.shortcut.as_deref(),
                Some(cmd.shortcut),
                "Shortcut mismatch for {expected_id}"
            );
        }
    }

    #[test]
    fn trash_description_varies_by_item_type() {
        let file = make_file("temp.log", "/tmp/temp.log");
        let dir = make_dir("build", "/tmp/build");

        let file_actions = get_file_context_actions(&file);
        let dir_actions = get_file_context_actions(&dir);

        let file_trash = file_actions
            .iter()
            .find(|a| a.id == "file:move_to_trash")
            .expect("missing trash for file");
        let dir_trash = dir_actions
            .iter()
            .find(|a| a.id == "file:move_to_trash")
            .expect("missing trash for dir");

        assert!(file_trash
            .description
            .as_ref()
            .is_some_and(|d| d.contains("file")));
        assert!(dir_trash
            .description
            .as_ref()
            .is_some_and(|d| d.contains("folder")));
    }
}
