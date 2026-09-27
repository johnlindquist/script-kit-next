use super::shared::to_deeplink_name;
use super::types::{Action, ActionCategory, ScriptInfo};
use crate::actions::builders::file_path::open_in_quick_terminal_action;
use crate::designs::icon_variations::IconName;
use itertools::Itertools;
use std::collections::HashSet;
use std::path::Path;

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
        .join(" ")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptContextKind {
    App,
    Agent,
    Scriptlet,
    Script,
    Skill,
    BuiltIn,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrimaryActionPlan {
    PreserveCatalogActionText,
    VerbForSubject { subject: &'static str },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PrimaryActionCopy {
    title: String,
    description: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptContextPreferenceActionPlan {
    AgentNoPreferenceActions,
    NoShortcutNoAlias,
    ShortcutOnly,
    AliasOnly,
    ShortcutAndAlias,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptContextShareActionPlan {
    PortableShareLink,
    DirectRunDeepLink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FavoriteActionPlan {
    AddToFavorites,
    RemoveFromFavorites,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RankingActionPlan {
    NoRankingAction,
    ResetSuggestedRanking,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScriptContextShareActionCopy {
    title: &'static str,
    description: String,
}

impl FavoriteActionPlan {
    fn from_is_favorite(is_favorite: bool) -> Self {
        if is_favorite {
            Self::RemoveFromFavorites
        } else {
            Self::AddToFavorites
        }
    }

    fn copy(self) -> (&'static str, &'static str) {
        match self {
            Self::AddToFavorites => ("Add to Favorites", "Save this item to your favorites list"),
            Self::RemoveFromFavorites => (
                "Remove from Favorites",
                "Remove this item from your favorites list",
            ),
        }
    }
}

fn favorite_action_copy(is_favorite: bool) -> (&'static str, &'static str) {
    FavoriteActionPlan::from_is_favorite(is_favorite).copy()
}

impl RankingActionPlan {
    fn from_is_suggested(is_suggested: bool) -> Self {
        if is_suggested {
            Self::ResetSuggestedRanking
        } else {
            Self::NoRankingAction
        }
    }

    fn reset_action(self) -> Option<Action> {
        match self {
            Self::NoRankingAction => None,
            Self::ResetSuggestedRanking => Some(
                Action::new(
                    "reset_ranking",
                    "Delete Ranking Entry",
                    Some("Remove this item from Suggested section".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌃⌘R")
                .with_icon(IconName::Trash)
                .with_section("Destructive"),
            ),
        }
    }
}

fn script_context_kind(script: &ScriptInfo) -> ScriptContextKind {
    if script.is_app {
        ScriptContextKind::App
    } else if script.is_agent {
        ScriptContextKind::Agent
    } else if script.is_scriptlet {
        ScriptContextKind::Scriptlet
    } else if script.is_script {
        ScriptContextKind::Script
    } else if script.path.starts_with("skill:") {
        ScriptContextKind::Skill
    } else if script.path.starts_with("builtin:") {
        ScriptContextKind::BuiltIn
    } else {
        ScriptContextKind::Generic
    }
}

fn share_action_plan(script: &ScriptInfo) -> ScriptContextShareActionPlan {
    match script_context_kind(script) {
        ScriptContextKind::Script
        | ScriptContextKind::Scriptlet
        | ScriptContextKind::Agent
        | ScriptContextKind::Skill => ScriptContextShareActionPlan::PortableShareLink,
        ScriptContextKind::App | ScriptContextKind::BuiltIn | ScriptContextKind::Generic => {
            ScriptContextShareActionPlan::DirectRunDeepLink
        }
    }
}

fn preference_action_plan(script: &ScriptInfo) -> ScriptContextPreferenceActionPlan {
    if script.is_agent {
        return ScriptContextPreferenceActionPlan::AgentNoPreferenceActions;
    }

    match (script.shortcut.is_some(), script.alias.is_some()) {
        (false, false) => ScriptContextPreferenceActionPlan::NoShortcutNoAlias,
        (true, false) => ScriptContextPreferenceActionPlan::ShortcutOnly,
        (false, true) => ScriptContextPreferenceActionPlan::AliasOnly,
        (true, true) => ScriptContextPreferenceActionPlan::ShortcutAndAlias,
    }
}

fn share_action_copy(script: &ScriptInfo) -> ScriptContextShareActionCopy {
    match share_action_plan(script) {
        ScriptContextShareActionPlan::PortableShareLink => ScriptContextShareActionCopy {
            title: "Share",
            description: "Copy a portable Script Kit share link to clipboard".to_string(),
        },
        ScriptContextShareActionPlan::DirectRunDeepLink => {
            let deeplink_name = to_deeplink_name(&script.name);
            ScriptContextShareActionCopy {
                title: "Copy Deep Link",
                description: format!("Copy scriptkit://run/{} URL to clipboard", deeplink_name),
            }
        }
    }
}

fn primary_action_plan(kind: ScriptContextKind) -> PrimaryActionPlan {
    match kind {
        ScriptContextKind::BuiltIn => PrimaryActionPlan::PreserveCatalogActionText,
        ScriptContextKind::Script => PrimaryActionPlan::VerbForSubject { subject: "script" },
        ScriptContextKind::App => PrimaryActionPlan::VerbForSubject {
            subject: "application",
        },
        ScriptContextKind::Scriptlet => PrimaryActionPlan::VerbForSubject {
            subject: "scriptlet",
        },
        ScriptContextKind::Agent => PrimaryActionPlan::VerbForSubject { subject: "agent" },
        ScriptContextKind::Skill => PrimaryActionPlan::VerbForSubject { subject: "skill" },
        ScriptContextKind::Generic => PrimaryActionPlan::VerbForSubject { subject: "item" },
    }
}

fn append_shortcut_preference_actions(
    plan: ScriptContextPreferenceActionPlan,
    actions: &mut Vec<Action>,
    destructive_actions: &mut Vec<Action>,
) {
    match plan {
        ScriptContextPreferenceActionPlan::AgentNoPreferenceActions => {}
        ScriptContextPreferenceActionPlan::ShortcutOnly
        | ScriptContextPreferenceActionPlan::ShortcutAndAlias => {
            actions.push(
                Action::new(
                    "update_shortcut",
                    "Edit Keyboard Shortcut",
                    Some("Change the keyboard shortcut for this item".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘⇧K")
                .with_icon(IconName::Settings)
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
                .with_icon(IconName::Trash)
                .with_section("Destructive"),
            );
        }
        ScriptContextPreferenceActionPlan::NoShortcutNoAlias
        | ScriptContextPreferenceActionPlan::AliasOnly => {
            actions.push(
                Action::new(
                    "add_shortcut",
                    "Add Keyboard Shortcut",
                    Some("Set a keyboard shortcut for this item".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘⇧K")
                .with_icon(IconName::Settings)
                .with_section("Edit"),
            );
        }
    }
}

fn append_alias_preference_actions(
    plan: ScriptContextPreferenceActionPlan,
    actions: &mut Vec<Action>,
    destructive_actions: &mut Vec<Action>,
) {
    match plan {
        ScriptContextPreferenceActionPlan::AgentNoPreferenceActions => {}
        ScriptContextPreferenceActionPlan::AliasOnly
        | ScriptContextPreferenceActionPlan::ShortcutAndAlias => {
            actions.push(
                Action::new(
                    "update_alias",
                    "Edit Alias",
                    Some("Change the alias trigger for this item".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘⇧A")
                .with_icon(IconName::Settings)
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
                .with_icon(IconName::Trash)
                .with_section("Destructive"),
            );
        }
        ScriptContextPreferenceActionPlan::NoShortcutNoAlias
        | ScriptContextPreferenceActionPlan::ShortcutOnly => {
            actions.push(
                Action::new(
                    "add_alias",
                    "Add Alias",
                    Some(
                        "Set an alias trigger for this item (type alias + space to run)"
                            .to_string(),
                    ),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘⇧A")
                .with_icon(IconName::Settings)
                .with_section("Edit"),
            );
        }
    }
}

fn primary_action_copy(script: &ScriptInfo) -> PrimaryActionCopy {
    if let Some(primary) = script
        .command_descriptor
        .as_ref()
        .and_then(|descriptor| descriptor.primary_action())
    {
        return PrimaryActionCopy {
            title: primary.title.clone(),
            description: primary.title.clone(),
        };
    }

    match primary_action_plan(script_context_kind(script)) {
        PrimaryActionPlan::PreserveCatalogActionText => PrimaryActionCopy {
            title: script.action_verb.clone(),
            description: script.action_verb.clone(),
        },
        PrimaryActionPlan::VerbForSubject { subject } => PrimaryActionCopy {
            title: title_case_words(&script.action_verb),
            description: format!("{} this {}", script.action_verb, subject),
        },
    }
}

fn is_intent_plan(script: &ScriptInfo) -> bool {
    script
        .command_descriptor
        .as_ref()
        .is_some_and(|descriptor| {
            descriptor.identity.source() == sk_protocol::command_contract::CommandSource::Intent
        })
}

/// A plan read from a sentence can run, be kept as a script, or be explained; script
/// preferences like aliases and favorites don't apply to it.
fn intent_plan_actions(script: &ScriptInfo) -> Vec<Action> {
    let primary_copy = primary_action_copy(script);
    let mut run = Action::new(
        "run_script",
        primary_copy.title,
        Some(primary_copy.description),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("↵")
    .with_icon(IconName::PlayFilled)
    .with_section("Actions");
    if let Some(descriptor) = &script.command_descriptor {
        run = run.with_command_contract(descriptor);
    }
    vec![
        run,
        Action::new(
            "save_intent_as_script",
            "Save as Script",
            Some("Keep this as a script you can run again or give a shortcut".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘S")
        .with_icon(IconName::FileCode)
        .with_section("Actions"),
        Action::new(
            "toggle_info",
            "Show Info",
            Some("Toggle detailed info about this item".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I")
        .with_icon(IconName::File)
        .with_section("Actions"),
    ]
}

/// Get actions specific to the focused script.
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action> {
    if is_intent_plan(script) {
        return intent_plan_actions(script);
    }
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
    let primary_copy = primary_action_copy(script);
    let preference_plan = preference_action_plan(script);
    let ranking_plan = RankingActionPlan::from_is_suggested(script.is_suggested);

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

    let mut primary_action = Action::new(
        "run_script",
        primary_copy.title,
        Some(primary_copy.description),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("↵")
    .with_icon(IconName::PlayFilled)
    .with_section("Actions");
    if let Some(descriptor) = &script.command_descriptor {
        primary_action = primary_action.with_command_contract(descriptor);
    }
    actions.push(primary_action);

    actions.push(
        Action::new(
            "toggle_info",
            "Show Info",
            Some("Toggle detailed info about this item".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I")
        .with_icon(IconName::File)
        .with_section("Actions"),
    );

    append_shortcut_preference_actions(preference_plan, &mut actions, &mut destructive_actions);
    append_alias_preference_actions(preference_plan, &mut actions, &mut destructive_actions);

    if (script.is_script || script.is_scriptlet || script.is_agent || script.is_app)
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
            .with_icon(IconName::Star)
            .with_section("Edit"),
        );
    }

    if script.is_app {
        // Finder actions
        actions.push(
            Action::new(
                "reveal_in_finder",
                "Show in Finder",
                Some("Reveal application in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F")
            .with_icon(IconName::FolderOpen)
            .with_section("Finder"),
        );

        actions.push(open_in_quick_terminal_action(Path::new(&script.path)));

        actions.push(
            Action::new(
                "show_info_in_finder",
                "Show Info in Finder",
                Some("Open Finder info window for this application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::File)
            .with_section("Finder"),
        );

        actions.push(
            Action::new(
                "show_package_contents",
                "Show Package Contents",
                Some("Open the application bundle Contents folder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::FolderOpen)
            .with_section("Finder"),
        );

        // Copy actions
        actions.push(
            Action::new(
                "copy_name",
                "Copy Name",
                Some("Copy application name to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘.")
            .with_icon(IconName::Copy)
            .with_section("Copy"),
        );

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy application path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘.")
            .with_icon(IconName::Copy)
            .with_section("Copy"),
        );

        if script.bundle_id.is_some() {
            actions.push(
                Action::new(
                    "copy_bundle_id",
                    "Copy Bundle Identifier",
                    Some("Copy bundle identifier to clipboard".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⇧⌘C")
                .with_icon(IconName::Copy)
                .with_section("Copy"),
            );
        }

        // Process actions
        actions.push(
            Action::new(
                "quit_app",
                "Quit Application",
                Some("Gracefully quit this application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧Q")
            .with_icon(IconName::Close)
            .with_section("Process"),
        );

        actions.push(
            Action::new(
                "restart_app",
                "Restart Application",
                Some("Quit and relaunch this application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Refresh)
            .with_section("Process"),
        );

        destructive_actions.push(
            Action::new(
                "force_quit_app",
                "Force Quit Application",
                Some("Force quit this application immediately".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Close)
            .with_section("Destructive"),
        );
    }

    // Built-in command rows (identified by their `builtin:<id>` path) get a
    // Copy Command ID action so the id is reachable without opening config.
    if !script.is_script
        && !script.is_scriptlet
        && !script.is_app
        && !script.is_agent
        && script.path.starts_with("builtin:")
    {
        actions.push(
            Action::new(
                "copy_command_id",
                "Copy Command ID",
                Some("Copy this built-in command's id to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘.")
            .with_icon(IconName::Copy)
            .with_section("Copy"),
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
            .with_icon(IconName::Pencil)
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
            .with_icon(IconName::File)
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
            .with_icon(IconName::FolderOpen)
            .with_section("Share"),
        );

        actions.push(open_in_quick_terminal_action(Path::new(&script.path)));

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy script path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C")
            .with_icon(IconName::Copy)
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
            .with_icon(IconName::Copy)
            .with_section("Share"),
        );

        destructive_actions.push(
            Action::new(
                "delete_script",
                "Delete Script?",
                Some("Move the selected script to Trash".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Trash)
            .with_section("Destructive"),
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
            .with_icon(IconName::Pencil)
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
            .with_icon(IconName::FolderOpen)
            .with_section("Share"),
        );

        actions.push(open_in_quick_terminal_action(Path::new(&script.path)));

        actions.push(
            Action::new(
                "copy_scriptlet_path",
                "Copy Path",
                Some("Copy scriptlet bundle path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C")
            .with_icon(IconName::Copy)
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
            .with_icon(IconName::Copy)
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
            .with_icon(IconName::Pencil)
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
            .with_icon(IconName::FolderOpen)
            .with_section("Share"),
        );

        actions.push(open_in_quick_terminal_action(Path::new(&script.path)));

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy agent path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C")
            .with_icon(IconName::Copy)
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
            .with_icon(IconName::Copy)
            .with_section("Share"),
        );
    }

    let share_copy = share_action_copy(script);
    actions.push(
        Action::new(
            "copy_deeplink",
            share_copy.title,
            Some(share_copy.description),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧D")
        .with_icon(IconName::Copy)
        .with_section("Share"),
    );

    if let Some(reset_action) = ranking_plan.reset_action() {
        destructive_actions.push(reset_action);
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

/// Predefined global actions surfaced in the Cmd+K dialog when the focused
/// row offers no script-context entries (e.g. on the main script list).
///
/// The actions menu is contextual to the focused item, so this list must stay
/// free of rows that act on some other surface's state. In particular the
/// prompt export/handoff rows ("Export Prompt…", "Send Prompt to…") operate on
/// the Agent Chat composer's built prompt and are owned by
/// `get_agent_chat_actions`; adding them here leaks guaranteed-failure rows
/// into every host (main list, file search, built-ins, …).
pub fn get_global_actions() -> Vec<Action> {
    vec![
        Action::new(
            "reload_scripts",
            "Reload Scripts",
            Some("Re-scan ~/.scriptkit and rebuild the script index".into()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘R"),
        Action::new(
            "settings",
            "Edit Config File",
            Some("Open ~/.scriptkit/config.ts in your editor".into()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘,"),
        Action::new(
            "view_logs",
            "Show Logs",
            Some("Toggle the in-launcher log panel".into()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘L"),
        Action::new(
            "open_help",
            "Help & User Guide",
            Some("Open the Script Kit guide (~/.scriptkit/GUIDE.md)".into()),
            ActionCategory::GlobalOps,
        )
        .with_section("Discover"),
        Action::new(
            "ask_ai_settings",
            "Change Settings with AI",
            Some("Ask Agent Chat to update ~/.scriptkit/config.ts for you".into()),
            ActionCategory::GlobalOps,
        )
        .with_section("Discover"),
        Action::new(
            "open_settings_menu",
            "Open Settings Menu",
            Some("Theme, dictation, microphone, permissions, window options".into()),
            ActionCategory::GlobalOps,
        )
        .with_section("Discover"),
        Action::new(
            "setup_dictation",
            "Set Up Dictation",
            Some("Check microphone, model, and hotkey readiness".into()),
            ActionCategory::GlobalOps,
        )
        .with_section("Discover"),
        Action::new(
            "check_permissions",
            "Check Permissions",
            Some("Review the macOS permissions Script Kit needs".into()),
            ActionCategory::GlobalOps,
        )
        .with_section("Discover"),
        Action::new(
            "sdk_reference",
            "Browse SDK Reference",
            Some("Search the Script Kit SDK API documentation".into()),
            ActionCategory::GlobalOps,
        )
        .with_section("Discover"),
    ]
}

#[cfg(test)]
#[path = "script_context/tests.rs"]
mod tests;
