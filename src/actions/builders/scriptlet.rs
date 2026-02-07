use super::shared::{format_shortcut_hint, to_deeplink_name};
use super::types::{Action, ActionCategory, ScriptInfo};
use crate::scriptlets::Scriptlet;

/// Convert scriptlet-defined actions (from H3 headers) to Action structs for the UI.
pub fn get_scriptlet_defined_actions(scriptlet: &Scriptlet) -> Vec<Action> {
    let actions: Vec<Action> = scriptlet
        .actions
        .iter()
        .map(|sa| {
            let mut action = Action::new(
                sa.action_id(),
                &sa.name,
                sa.description.clone(),
                ActionCategory::ScriptContext,
            );

            if let Some(ref shortcut) = sa.shortcut {
                action = action.with_shortcut(format_shortcut_hint(shortcut));
            }

            action = action.with_section("Actions");
            action.has_action = true;
            action.value = Some(sa.command.clone());

            tracing::debug!(
                target: "script_kit::actions",
                action_id = %action.id,
                has_action = action.has_action,
                has_shortcut = action.shortcut.is_some(),
                "Created scriptlet-defined action (has_action=true)"
            );

            action
        })
        .collect();

    if !actions.is_empty() {
        tracing::debug!(
            target: "script_kit::actions",
            scriptlet_name = %scriptlet.name,
            custom_action_count = actions.len(),
            "Built scriptlet-defined actions from H3 headers"
        );
    }

    actions
}

/// Get actions for a scriptlet, including both custom (H3-defined) and built-in actions.
pub fn get_scriptlet_context_actions_with_custom(
    script: &ScriptInfo,
    scriptlet: Option<&Scriptlet>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let mut destructive_actions = Vec::new();

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

    if let Some(scriptlet) = scriptlet {
        actions.extend(get_scriptlet_defined_actions(scriptlet));
    }

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
    actions
}
