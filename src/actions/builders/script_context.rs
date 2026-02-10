use super::shared::to_deeplink_name;
use super::types::{Action, ActionCategory, ScriptInfo};
use std::collections::HashSet;

fn has_invalid_script_context_input(script: &ScriptInfo) -> bool {
    script.name.trim().is_empty() || script.action_verb.trim().is_empty()
}

fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let mut normalized = first.to_uppercase().collect::<String>();
                    normalized.push_str(&chars.as_str().to_lowercase());
                    normalized
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn favorite_action_copy(is_favorite: bool) -> (&'static str, &'static str) {
    if is_favorite {
        (
            "Remove from Favorites",
            "Remove this item from your favorites list",
        )
    } else {
        ("Add to Favorites", "Save this item to your favorites list")
    }
}

/// Get actions specific to the focused script.
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action> {
    if has_invalid_script_context_input(script) {
        tracing::warn!(
            target: "script_kit::actions",
            builder = "script_context",
            name = %script.name,
            action_verb = %script.action_verb,
            "Invalid script context input; returning empty actions"
        );
        return vec![];
    }

    let mut actions = Vec::new();
    let mut destructive_actions = Vec::new();

    tracing::debug!(
        target: "script_kit::actions",
        name = %script.name,
        is_script = script.is_script,
        is_scriptlet = script.is_scriptlet,
        is_agent = script.is_agent,
        has_shortcut = script.shortcut.is_some(),
        has_alias = script.alias.is_some(),
        is_suggested = script.is_suggested,
        "Building script context actions"
    );

    actions.push(
        Action::new(
            "run_script",
            title_case_words(&script.action_verb),
            Some(format!("{} this item", script.action_verb)),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_section("Actions"),
    );

    if script.shortcut.is_some() {
        actions.push(
            Action::new(
                "update_shortcut",
                "Edit Keyboard Shortcut",
                Some("Change the keyboard shortcut for this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K")
            .with_section("Edit"),
        );
        destructive_actions.push(
            Action::new(
                "remove_shortcut",
                "Delete Keyboard Shortcut",
                Some("Remove the keyboard shortcut from this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥K")
            .with_section("Destructive"),
        );
    } else {
        actions.push(
            Action::new(
                "add_shortcut",
                "Add Keyboard Shortcut",
                Some("Set a keyboard shortcut for this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K")
            .with_section("Edit"),
        );
    }

    if script.alias.is_some() {
        actions.push(
            Action::new(
                "update_alias",
                "Edit Alias",
                Some("Change the alias trigger for this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A")
            .with_section("Edit"),
        );
        destructive_actions.push(
            Action::new(
                "remove_alias",
                "Delete Alias",
                Some("Remove the alias trigger from this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥A")
            .with_section("Destructive"),
        );
    } else {
        actions.push(
            Action::new(
                "add_alias",
                "Add Alias",
                Some("Set an alias trigger for this item (type alias + space to run)".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A")
            .with_section("Edit"),
        );
    }

    if (script.is_script || script.is_scriptlet || script.is_agent)
        && !script.path.trim().is_empty()
    {
        let (title, description) =
            favorite_action_copy(crate::favorites::is_favorite(&script.path));
        actions.push(
            Action::new(
                "toggle_favorite",
                title,
                Some(description.to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Edit"),
        );
    }

    if script.is_script {
        actions.push(
            Action::new(
                "edit_script",
                "Edit Script",
                Some("Open in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E")
            .with_section("Edit"),
        );

        actions.push(
            Action::new(
                "view_logs",
                "Show Logs",
                Some("Show script execution logs".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘L")
            .with_section("Edit"),
        );

        actions.push(
            Action::new(
                "reveal_in_finder",
                "Open in Finder",
                Some("Reveal script file in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F")
            .with_section("Share"),
        );

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy script path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C")
            .with_section("Share"),
        );

        actions.push(
            Action::new(
                "copy_content",
                "Copy Content",
                Some("Copy entire file content to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥C")
            .with_section("Share"),
        );
    }

    if script.is_scriptlet {
        actions.push(
            Action::new(
                "edit_scriptlet",
                "Edit Scriptlet",
                Some("Open the markdown file in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E")
            .with_section("Edit"),
        );

        actions.push(
            Action::new(
                "reveal_scriptlet_in_finder",
                "Open in Finder",
                Some("Reveal scriptlet bundle in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F")
            .with_section("Share"),
        );

        actions.push(
            Action::new(
                "copy_scriptlet_path",
                "Copy Path",
                Some("Copy scriptlet bundle path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C")
            .with_section("Share"),
        );

        actions.push(
            Action::new(
                "copy_content",
                "Copy Content",
                Some("Copy entire file content to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥C")
            .with_section("Share"),
        );
    }

    if script.is_agent {
        actions.push(
            Action::new(
                "edit_script",
                "Edit Agent",
                Some("Open the agent file in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E")
            .with_section("Edit"),
        );

        actions.push(
            Action::new(
                "reveal_in_finder",
                "Open in Finder",
                Some("Reveal agent file in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F")
            .with_section("Share"),
        );

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy agent path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C")
            .with_section("Share"),
        );

        actions.push(
            Action::new(
                "copy_content",
                "Copy Content",
                Some("Copy entire file content to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥C")
            .with_section("Share"),
        );
    }

    let deeplink_name = to_deeplink_name(&script.name);
    actions.push(
        Action::new(
            "copy_deeplink",
            "Copy Deep Link",
            Some(format!(
                "Copy scriptkit://run/{} URL to clipboard",
                deeplink_name
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧D")
        .with_section("Share"),
    );

    if script.is_suggested {
        destructive_actions.push(
            Action::new(
                "reset_ranking",
                "Delete Ranking Entry",
                Some("Remove this item from Suggested section".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌃⌘R")
            .with_section("Destructive"),
        );
    }

    actions.extend(destructive_actions);
    let mut seen_ids = HashSet::new();
    let mut duplicate_ids = Vec::new();
    let deduped_actions: Vec<Action> = actions
        .into_iter()
        .filter(|action| {
            if seen_ids.insert(action.id.clone()) {
                true
            } else {
                duplicate_ids.push(action.id.clone());
                false
            }
        })
        .collect();

    if !duplicate_ids.is_empty() {
        tracing::warn!(
            target: "script_kit::actions",
            name = %script.name,
            duplicate_ids = ?duplicate_ids,
            "Deduplicated overlapping script context action IDs"
        );
    }

    tracing::debug!(
        target: "script_kit::actions",
        action_count = deduped_actions.len(),
        action_ids = ?deduped_actions.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
        "Created script context actions"
    );

    deduped_actions
}

/// Predefined global actions.
/// Note: Settings and Quit are available from the main menu, not shown in actions dialog.
pub fn get_global_actions() -> Vec<Action> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_action_title(actions: &[Action], id: &str) -> String {
        actions
            .iter()
            .find(|action| action.id == id)
            .map(|action| action.title.clone())
            .expect("action id should exist in script context actions")
    }

    fn has_action(actions: &[Action], id: &str) -> bool {
        actions.iter().any(|action| action.id == id)
    }

    #[test]
    fn test_get_script_context_actions_returns_empty_when_name_is_blank() {
        let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
        script.name = "   ".to_string();

        let actions = get_script_context_actions(&script);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_script_context_actions_returns_empty_when_action_verb_is_blank() {
        let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
        script.action_verb = "   ".to_string();

        let actions = get_script_context_actions(&script);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_script_context_actions_run_label_uses_title_case_verb() {
        let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
        script.action_verb = "switch to".to_string();

        let actions = get_script_context_actions(&script);

        assert_eq!(find_action_title(&actions, "run_script"), "Switch To");
    }

    #[test]
    fn test_favorite_action_copy_returns_add_copy_when_not_favorite() {
        let (title, description) = favorite_action_copy(false);

        assert_eq!(title, "Add to Favorites");
        assert_eq!(description, "Save this item to your favorites list");
    }

    #[test]
    fn test_favorite_action_copy_returns_remove_copy_when_favorite() {
        let (title, description) = favorite_action_copy(true);

        assert_eq!(title, "Remove from Favorites");
        assert_eq!(description, "Remove this item from your favorites list");
    }

    #[test]
    fn test_get_script_context_actions_includes_toggle_favorite_for_script_items() {
        let script = ScriptInfo::new("Valid", "/tmp/script-context-favorites-test.ts");

        let actions = get_script_context_actions(&script);

        assert!(has_action(&actions, "toggle_favorite"));
    }

    #[test]
    fn test_get_script_context_actions_skips_toggle_favorite_for_builtin_items() {
        let script = ScriptInfo::builtin("Clipboard History");

        let actions = get_script_context_actions(&script);

        assert!(!has_action(&actions, "toggle_favorite"));
    }

    #[test]
    fn test_get_script_context_actions_labels_use_consistent_verb_style() {
        let mut script = ScriptInfo::new("Valid", "/tmp/valid.ts");
        script.shortcut = Some("cmd-shift-k".to_string());
        script.alias = Some("v".to_string());
        script.is_suggested = true;

        let actions = get_script_context_actions(&script);

        assert_eq!(find_action_title(&actions, "run_script"), "Run");
        assert_eq!(
            find_action_title(&actions, "update_shortcut"),
            "Edit Keyboard Shortcut"
        );
        assert_eq!(
            find_action_title(&actions, "remove_shortcut"),
            "Delete Keyboard Shortcut"
        );
        assert_eq!(find_action_title(&actions, "update_alias"), "Edit Alias");
        assert_eq!(find_action_title(&actions, "remove_alias"), "Delete Alias");
        assert_eq!(find_action_title(&actions, "view_logs"), "Show Logs");
        assert_eq!(
            find_action_title(&actions, "reveal_in_finder"),
            "Open in Finder"
        );
        assert_eq!(
            find_action_title(&actions, "copy_deeplink"),
            "Copy Deep Link"
        );
        assert_eq!(
            find_action_title(&actions, "reset_ranking"),
            "Delete Ranking Entry"
        );

        for action in &actions {
            assert!(
                !action.title.ends_with("..."),
                "label should not end with ellipsis: {}",
                action.title
            );
            assert!(
                action.title.chars().count() < 30,
                "label should stay concise: {}",
                action.title
            );
        }
    }
}
