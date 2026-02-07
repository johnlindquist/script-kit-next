
#[cfg(test)]
mod tests {
    use super::{is_destructive_action, should_render_section_separator};
    use crate::actions::types::{Action, ActionCategory};

    #[test]
    fn destructive_detection_matches_known_ids() {
        let remove_action = Action::new(
            "remove_alias",
            "Remove Alias",
            Some("Remove alias".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(is_destructive_action(&remove_action));

        let trash_action = Action::new(
            "move_to_trash",
            "Move to Trash",
            Some("Move item to Trash".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(is_destructive_action(&trash_action));
    }

    #[test]
    fn destructive_detection_matches_title_prefix_fallback() {
        let delete_action = Action::new(
            "custom_action",
            "Delete Export Cache",
            Some("Delete cached export".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(is_destructive_action(&delete_action));

        let safe_action = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy path".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(!is_destructive_action(&safe_action));
    }

    #[test]
    fn section_separator_only_shows_on_section_boundary() {
        let actions = vec![
            Action::new(
                "run_script",
                "Run Script",
                Some("Run".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Actions"),
            Action::new(
                "edit_script",
                "Edit Script",
                Some("Edit".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Edit"),
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Share"),
            Action::new(
                "copy_deeplink",
                "Copy Deeplink",
                Some("Copy".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Share"),
        ];
        let filtered_actions = vec![0, 1, 2, 3];

        assert!(!should_render_section_separator(
            &actions,
            &filtered_actions,
            0
        ));
        assert!(should_render_section_separator(
            &actions,
            &filtered_actions,
            1
        ));
        assert!(should_render_section_separator(
            &actions,
            &filtered_actions,
            2
        ));
        assert!(!should_render_section_separator(
            &actions,
            &filtered_actions,
            3
        ));
    }
}
