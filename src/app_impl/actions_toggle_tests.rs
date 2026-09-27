#[cfg(test)]
mod terminal_command_shortcut_tests {
    use super::*;
    use crate::actions::{AnchorPosition, SearchPosition, SectionStyle};
    use crate::designs::icon_variations::IconName;
    use std::fs;

    #[test]
    fn test_terminal_actions_for_dialog_shows_cmd_shift_k_for_clear_terminal() {
        let clear_action = terminal_actions_for_dialog()
            .into_iter()
            .find(|action| action.id == TERM_PROMPT_CLEAR_ACTION_ID)
            .expect("clear action should exist in terminal actions");

        assert_eq!(
            clear_action.shortcut.as_deref(),
            Some(TERM_PROMPT_CLEAR_SHORTCUT)
        );
    }

    #[test]
    fn test_terminal_actions_for_dialog_adds_cmd_k_toggle_shortcut() {
        let toggle_actions = terminal_actions_for_dialog()
            .into_iter()
            .find(|action| action.id == TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID)
            .expect("toggle actions entry should exist in terminal actions");

        assert_eq!(
            toggle_actions.shortcut.as_deref(),
            Some(TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT)
        );
    }

    #[test]
    fn test_terminal_actions_for_dialog_groups_sections_and_icons() {
        let actions = terminal_actions_for_dialog();

        let copy_action = actions
            .iter()
            .find(|action| action.id == "copy")
            .expect("copy action should exist");
        assert_eq!(copy_action.section.as_deref(), Some("Clipboard"));
        assert_eq!(copy_action.icon, Some(IconName::Copy));

        let find_action = actions
            .iter()
            .find(|action| action.id == "find")
            .expect("find action should exist");
        assert_eq!(find_action.section.as_deref(), Some("Search"));
        assert_eq!(find_action.icon, Some(IconName::MagnifyingGlass));

        let scroll_to_top_action = actions
            .iter()
            .find(|action| action.id == "scroll_to_top")
            .expect("scroll_to_top action should exist");
        assert_eq!(scroll_to_top_action.section.as_deref(), Some("Navigation"));
        assert_eq!(scroll_to_top_action.icon, Some(IconName::ArrowUp));

        let scroll_to_bottom_action = actions
            .iter()
            .find(|action| action.id == TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID)
            .expect("scroll_to_bottom action should exist");
        assert_eq!(
            scroll_to_bottom_action.section.as_deref(),
            Some("Navigation")
        );
        assert_eq!(scroll_to_bottom_action.icon, Some(IconName::ArrowDown));

        let clear_action = actions
            .iter()
            .find(|action| action.id == TERM_PROMPT_CLEAR_ACTION_ID)
            .expect("clear action should exist");
        assert_eq!(clear_action.section.as_deref(), Some("Session"));
        assert_eq!(clear_action.icon, Some(IconName::Trash));

        let reset_action = actions
            .iter()
            .find(|action| action.id == "reset")
            .expect("reset action should exist");
        assert_eq!(reset_action.section.as_deref(), Some("Session"));
        assert_eq!(reset_action.icon, Some(IconName::Refresh));
    }

    #[test]
    fn test_terminal_actions_dialog_config_enables_visual_features() {
        let config = terminal_actions_dialog_config();

        assert_eq!(config.search_position, SearchPosition::Top);
        assert_eq!(config.section_style, SectionStyle::Headers);
        assert_eq!(config.anchor, AnchorPosition::Top);
        assert!(config.show_icons);
        assert!(
            !config.show_footer,
            "Terminal actions should stay footerless because shortcuts are rendered inline"
        );
    }

    #[test]
    fn test_toggle_terminal_commands_sets_terminal_context_title() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");

        assert!(
            source.contains("d.set_context_title(Some(\"Terminal\".to_string()));"),
            "toggle_terminal_commands should set terminal context title"
        );
    }
}

#[cfg(test)]
mod root_file_action_tests {
    use super::*;
    use crate::file_search::{FileResult, FileType};

    fn root_file(file_type: FileType) -> FileResult {
        FileResult {
            path: "/Users/example/Desktop/fix spelling.png".to_string(),
            name: "fix spelling.png".to_string(),
            size: 0,
            modified: 0,
            file_type,
        }
    }

    #[test]
    fn root_file_actions_for_regular_file_adds_browse_parent_folder() {
        let actions = root_file_actions_for(&root_file(FileType::Image));
        let titles = actions
            .iter()
            .map(|action| action.title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            titles,
            vec![
                "Open File",
                "Browse Parent Folder",
                "Reveal in Finder",
                "Copy Path",
                "Copy Name",
                "Quick Look"
            ]
        );
        assert_eq!(
            actions[1].id,
            crate::action_helpers::ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID
        );
        assert_eq!(actions.len(), 6);
    }

    #[test]
    fn root_file_actions_for_regular_file_displays_parent_folder_with_tilde_home() {
        let home = dirs::home_dir()
            .and_then(|path| path.to_str().map(|value| value.to_string()))
            .expect("home path should be valid UTF-8");
        let file = FileResult {
            path: format!("{home}/dev/script-kit-gpui/README.md"),
            name: "README.md".to_string(),
            size: 0,
            modified: 0,
            file_type: FileType::Document,
        };

        let actions = root_file_actions_for(&file);
        let browse_parent = actions
            .iter()
            .find(|action| {
                action.id == crate::action_helpers::ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID
            })
            .expect("Browse Parent Folder action");

        assert_eq!(browse_parent.title, "Browse Parent Folder");
        assert_eq!(
            browse_parent.description.as_deref(),
            Some("Opens ~/dev/script-kit-gpui/ in File Search")
        );
        assert!(!browse_parent
            .description
            .as_deref()
            .unwrap_or_default()
            .contains(&home));
    }

    #[test]
    fn root_file_actions_for_directory_adds_search_inside_folder() {
        let actions = root_file_actions_for(&root_file(FileType::Directory));
        let titles = actions
            .iter()
            .map(|action| action.title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(actions[0].title, "Open Folder");
        assert_eq!(
            actions[0].id,
            crate::action_helpers::ROOT_FILE_OPEN_ACTION_ID
        );
        assert_eq!(
            titles,
            vec![
                "Open Folder",
                "Search Inside Folder",
                "Reveal in Finder",
                "Copy Path",
                "Copy Name",
                "Quick Look"
            ]
        );
        assert_eq!(
            actions[1].id,
            crate::action_helpers::ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID
        );
        assert!(!actions
            .iter()
            .any(|action| action.id
                == crate::action_helpers::ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID));
    }

    #[test]
    fn root_file_action_ids_are_stable() {
        let actions = root_file_actions_for(&root_file(FileType::Image));
        let ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                crate::action_helpers::ROOT_FILE_OPEN_ACTION_ID,
                crate::action_helpers::ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID,
                crate::action_helpers::ROOT_FILE_REVEAL_IN_FINDER_ACTION_ID,
                crate::action_helpers::ROOT_FILE_COPY_PATH_ACTION_ID,
                crate::action_helpers::ROOT_FILE_COPY_NAME_ACTION_ID,
                crate::action_helpers::ROOT_FILE_QUICK_LOOK_ACTION_ID,
            ]
        );
    }

    #[test]
    fn root_file_actions_do_not_include_deferred_file_search_actions() {
        let actions = root_file_actions_for(&root_file(FileType::Image));
        let ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();

        for deferred in [
            "open_with",
            "show_info",
            "attach_to_ai",
            "copy_filename",
            "move_to_trash",
            "duplicate_file",
            "copy_file",
            "file:open_with",
            "file:show_info",
            "file:attach_to_ai",
            "file:copy_filename",
            "file:move_to_trash",
            "file:duplicate_path",
        ] {
            assert!(
                !ids.contains(&deferred),
                "root file action palette should not include deferred action {deferred}"
            );
        }
    }
}
