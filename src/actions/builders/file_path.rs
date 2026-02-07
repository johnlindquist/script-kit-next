use super::types::{Action, ActionCategory};
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;

/// Get actions specific to a file search result
///
/// Actions vary based on whether the item is a file or directory:
/// - Directory: `open_directory` as primary
/// - File: `open_file` as primary, plus Quick Look (macOS)
///
/// Common actions for both: reveal_in_finder, copy_path, copy_filename
pub fn get_file_context_actions(file_info: &FileInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    tracing::debug!(
        target: "script_kit::actions",
        name = %file_info.name,
        is_dir = file_info.is_dir,
        "Building file context actions"
    );

    if file_info.is_dir {
        actions.push(
            Action::new(
                "open_directory",
                format!("Open \"{}\"", file_info.name),
                Some("Open this folder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.push(
            Action::new(
                "open_file",
                format!("Open \"{}\"", file_info.name),
                Some("Open with default application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    actions.push(
        Action::new(
            "reveal_in_finder",
            "Reveal in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    #[cfg(target_os = "macos")]
    if !file_info.is_dir {
        actions.push(
            Action::new(
                "quick_look",
                "Quick Look",
                Some("Preview with Quick Look".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘Y"),
        );
    }

    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "open_with",
            "Open With...",
            Some("Choose application to open with".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘O"),
    );

    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "show_info",
            "Show Info",
            Some("Show file information in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I"),
    );

    actions.push(
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
    );

    actions.push(
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘C"),
    );

    actions
}

/// Get actions specific to a file/folder path
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action> {
    let mut actions = vec![
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
        Action::new(
            "open_in_finder",
            "Reveal in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F"),
        Action::new(
            "open_in_editor",
            "Open in Editor",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E"),
        Action::new(
            "open_in_terminal",
            "Open in Terminal",
            Some("Open terminal at this location".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T"),
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename".to_string()),
            ActionCategory::ScriptContext,
        ),
        Action::new(
            "move_to_trash",
            "Move to Trash",
            Some(format!(
                "Delete {}",
                if path_info.is_dir { "folder" } else { "file" }
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫"),
    ];

    if path_info.is_dir {
        actions.insert(
            0,
            Action::new(
                "open_directory",
                format!("Open \"{}\"", path_info.name),
                Some("Navigate into this directory".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.insert(
            0,
            Action::new(
                "select_file",
                format!("Select \"{}\"", path_info.name),
                Some("Submit this file".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    actions
}
