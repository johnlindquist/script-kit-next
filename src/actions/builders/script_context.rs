use super::shared::to_deeplink_name;
use super::types::{Action, ActionCategory, ScriptInfo};
use std::collections::HashSet;

/// Get actions specific to the focused script.
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action> {
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
            format!("{} \"{}\"", script.action_verb, script.name),
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
                "Update Keyboard Shortcut",
                Some("Change the keyboard shortcut for this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K")
            .with_section("Edit"),
        );
        destructive_actions.push(
            Action::new(
                "remove_shortcut",
                "Remove Keyboard Shortcut",
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
                "Update Alias",
                Some("Change the alias trigger for this item".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A")
            .with_section("Edit"),
        );
        destructive_actions.push(
            Action::new(
                "remove_alias",
                "Remove Alias",
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
                "View Logs",
                Some("Show script execution logs".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘L")
            .with_section("Edit"),
        );

        actions.push(
            Action::new(
                "reveal_in_finder",
                "Reveal in Finder",
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
                "Reveal in Finder",
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
                "Reveal in Finder",
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
            "Copy Deeplink",
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
                "Reset Ranking",
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
