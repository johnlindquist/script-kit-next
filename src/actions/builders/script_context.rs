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

    actions.push(
        Action::new(
            "run_script",
            primary_copy.title,
            Some(primary_copy.description),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::PlayFilled)
        .with_section("Actions"),
    );

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
pub fn get_global_actions() -> Vec<Action> {
    let config = crate::config::load_config();
    let mut actions = vec![
        Action::new(
            "reload_scripts",
            "Reload Scripts",
            Some("Re-scan ~/.scriptkit and rebuild the script index".into()),
            ActionCategory::GlobalOps,
        ),
        Action::new(
            "settings",
            "Open Settings",
            Some("Open ~/.scriptkit/config.ts in your editor".into()),
            ActionCategory::GlobalOps,
        ),
        Action::new(
            "view_logs",
            "Show Logs",
            Some("Toggle the in-launcher log panel".into()),
            ActionCategory::GlobalOps,
        ),
    ];
    actions.extend(get_prompt_export_actions(&config));
    actions.extend(get_prompt_target_actions(&config));
    actions
}

fn get_prompt_export_actions(config: &crate::config::Config) -> Vec<Action> {
    crate::ai::agent_prompt_handoff::builtin_prompt_actions()
        .into_iter()
        .map(|action| {
            let action_id = action.action_id();
            let icon = match action {
                crate::ai::agent_prompt_handoff::AgentPromptActionId::CopyPrompt => IconName::Copy,
                _ => IconName::File,
            };
            let shortcut = config
                .get_command_shortcut(action_id)
                .map(|shortcut| shortcut.to_shortcut_string());
            Action::new(
                action_id,
                action.title(),
                Some(action.description().to_string()),
                ActionCategory::GlobalOps,
            )
            .with_shortcut_opt(shortcut)
            .with_icon(icon)
            .with_section("Export")
        })
        .collect()
}

fn get_prompt_target_actions(config: &crate::config::Config) -> Vec<Action> {
    crate::ai::agent_prompt_handoff::all_prompt_targets(config)
        .into_iter()
        .map(|target| {
            let action_id = target.action_id();
            let description = match &target {
                crate::ai::agent_prompt_handoff::AgentPromptHandoffAdapterId::CmuxCodex => {
                    Some("Open cmux with Codex using the current prompt".to_string())
                }
                crate::ai::agent_prompt_handoff::AgentPromptHandoffAdapterId::Command(target) => {
                    target
                        .description
                        .clone()
                        .or_else(|| Some(format!("Send the current prompt to {}", target.command)))
                }
            };
            let shortcut = config
                .get_command_shortcut(&action_id)
                .map(|shortcut| shortcut.to_shortcut_string());
            Action::new(
                action_id,
                format!("Send Prompt to {}", target.title()),
                description,
                ActionCategory::GlobalOps,
            )
            .with_shortcut_opt(shortcut)
            .with_icon(IconName::ArrowRight)
            .with_section("Handoff")
        })
        .collect()
}

#[allow(dead_code)] // Used by the binary Agent Chat actions surface.
const AGENT_CHAT_SWITCH_MODEL_ACTION_PREFIX: &str = "agent_chat_switch_model:";
const AGENT_CHAT_SWITCH_PROFILE_ACTION_PREFIX: &str = "agent_chat_switch_profile:";

#[allow(dead_code)] // Used by the binary Agent Chat actions surface.
fn agent_chat_switch_model_action_id(model_id: &str) -> String {
    format!("{AGENT_CHAT_SWITCH_MODEL_ACTION_PREFIX}{model_id}")
}

#[allow(dead_code)] // Used by Agent Chat chat action dispatch in the binary target.
pub(crate) fn agent_chat_switch_model_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(AGENT_CHAT_SWITCH_MODEL_ACTION_PREFIX)
}

#[allow(dead_code)] // Used by Agent Chat action dispatch in the binary target.
fn agent_chat_switch_profile_action_id(profile_id: &str) -> String {
    format!("{AGENT_CHAT_SWITCH_PROFILE_ACTION_PREFIX}{profile_id}")
}

#[allow(dead_code)] // Used by Agent Chat action dispatch in the binary target.
pub(crate) fn agent_chat_switch_profile_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(AGENT_CHAT_SWITCH_PROFILE_ACTION_PREFIX)
}

/// Action ID for reviewing session "Allow always" permission grants.
pub(crate) const AGENT_CHAT_REVIEW_APPROVALS_ACTION_ID: &str = "agent_chat_review_approvals";

/// Action ID for starting a fresh thread while the current one keeps
/// streaming in the background.
pub(crate) const AGENT_CHAT_NEW_THREAD_ACTION_ID: &str = "agent_chat_new_thread";

const AGENT_CHAT_SWITCH_THREAD_ACTION_PREFIX: &str = "agent_chat_switch_thread:";

fn agent_chat_switch_thread_action_id(ui_thread_id: &str) -> String {
    format!("{AGENT_CHAT_SWITCH_THREAD_ACTION_PREFIX}{ui_thread_id}")
}

#[allow(dead_code)] // Used by Agent Chat action dispatch in the binary target.
pub(crate) fn agent_chat_switch_thread_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(AGENT_CHAT_SWITCH_THREAD_ACTION_PREFIX)
}

/// Action ID for the "Rewind & Edit Message" drill-down trigger.
pub(crate) const AGENT_CHAT_REWIND_ACTION_ID: &str = "agent_chat_rewind_edit";

/// Route ID for the rewind checkpoint picker sub-route.
pub(crate) const AGENT_CHAT_FORK_PICKER_ROUTE_ID: &str = "agent_chat:fork_picker";

const AGENT_CHAT_FORK_EDIT_ACTION_PREFIX: &str = "agent_chat_fork_edit:";

fn agent_chat_fork_edit_action_id(entry_id: &str) -> String {
    format!("{AGENT_CHAT_FORK_EDIT_ACTION_PREFIX}{entry_id}")
}

#[allow(dead_code)] // Used by Agent Chat action dispatch in the binary target.
pub(crate) fn agent_chat_fork_edit_entry_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(AGENT_CHAT_FORK_EDIT_ACTION_PREFIX)
}

/// Truncate a user message to a single action-row title line.
fn agent_chat_message_action_title(text: &str) -> String {
    const MAX_CHARS: usize = 64;
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return "(empty message)".to_string();
    }
    if line.chars().count() <= MAX_CHARS {
        return line.to_string();
    }
    let truncated: String = line.chars().take(MAX_CHARS).collect();
    format!("{}\u{2026}", truncated.trim_end())
}

/// Actions available in the Agent Chat chat view (Cmd+K menu).
#[allow(dead_code)]
pub fn get_agent_chat_actions() -> Vec<Action> {
    let config = crate::config::load_config();
    let mut actions = get_prompt_export_actions(&config)
        .into_iter()
        .chain(get_prompt_target_actions(&config))
        .map(|mut action| {
            action.category = ActionCategory::ScriptContext;
            action
        })
        .collect::<Vec<_>>();
    actions.extend(vec![
        // ── Response ─────────────────────────────────────────
        Action::new(
            "agent_chat_copy_last_response",
            "Copy Last Response",
            Some("Copy the most recent assistant response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{21e7}\u{2318}C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        Action::new(
            "agent_chat_paste_to_frontmost",
            "Paste Response to App",
            Some("Paste into the frontmost application".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowRight)
        .with_section("Response"),
        Action::new(
            "agent_chat_retry_last",
            "Retry Last Message",
            Some("Resend the last user message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowRight)
        .with_section("Response"),
        Action::new(
            "agent_chat_export_markdown",
            "Copy Conversation as Markdown",
            Some("Copy the full conversation as markdown".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Response"),
        Action::new(
            "agent_chat_save_as_note",
            "Save as Note",
            Some("Create or update a note from the current Agent Chat content".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{21e7}\u{2318}S")
        .with_icon(IconName::File)
        .with_section("Response"),
        // ── Code ─────────────────────────────────────────────
        Action::new(
            "agent_chat_copy_all_code",
            "Copy All Code Blocks",
            Some("Copy all code blocks to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Code)
        .with_section("Code"),
        Action::new(
            "agent_chat_save_as_script",
            "Save as Script",
            Some("Save last code block as a Script Kit script".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Code"),
        Action::new(
            "agent_chat_run_last_code",
            "Run Last Code Block",
            Some("Execute the last code block and show output".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::BoltFilled)
        .with_section("Code"),
        Action::new(
            "agent_chat_open_in_editor",
            "Open in Editor",
            Some("Open ~/.scriptkit in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Code)
        .with_section("Code"),
        // ── Navigate ─────────────────────────────────────────
        Action::new(
            "agent_chat_scroll_to_top",
            "Scroll to Top",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowUp)
        .with_section("Navigate"),
        Action::new(
            "agent_chat_scroll_to_bottom",
            "Scroll to Latest",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowDown)
        .with_section("Navigate"),
        Action::new(
            "agent_chat_show_history",
            "Agent Chat History",
            Some("Browse and manage past Agent Chat conversations".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}P")
        .with_icon(IconName::MagnifyingGlass)
        .with_section("Navigate"),
        Action::new(
            AGENT_CHAT_SHOW_RECEIPT_HISTORY_ACTION_ID,
            "Receipt History",
            Some("Inspect recent semantic automation receipts for this session".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Proof"),
        // ── View ─────────────────────────────────────────────
        Action::new(
            "agent_chat_expand_all",
            "Expand All Blocks",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ChevronDown)
        .with_section("View"),
        Action::new(
            "agent_chat_collapse_all",
            "Collapse All Blocks",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ChevronRight)
        .with_section("View"),
        // ── Session ──────────────────────────────────────────
        Action::new(
            "agent_chat_new_conversation",
            "New Conversation",
            Some("Clear messages, keep session".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}L")
        .with_icon(IconName::Plus)
        .with_section("Session"),
        Action::new(
            "agent_chat_clear_conversation",
            "Clear & Restart",
            Some("Close and reopen a fresh session".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Trash)
        .with_section("Session"),
        Action::new(
            "agent_chat_clear_history",
            "Clear History",
            Some("Delete all saved conversations".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Trash)
        .with_section("Session"),
        // ── Window ───────────────────────────────────────────
        Action::new(
            "agent_chat_detach_window",
            "Keep Open in Window",
            Some("Keep this chat open in a separate window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowUp)
        .with_section("Window"),
        Action::new(
            "agent_chat_reattach_panel",
            "Return to Panel",
            Some("Move this chat back to the main panel".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowDown)
        .with_section("Window"),
        Action::new(
            "agent_chat_close",
            "Close Agent Chat",
            None,
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}W")
        .with_icon(IconName::Close)
        .with_section("Window"),
    ]);
    actions
}

pub(crate) fn get_focused_text_agent_chat_actions(expanded: bool) -> Vec<Action> {
    let expand_id = if expanded {
        "focused-text-action-collapse"
    } else {
        "focused-text-action-expand"
    };
    let expand_label = if expanded { "Collapse" } else { "Chat" };
    let expand_description = if expanded {
        "Return to the compact focused-text editor"
    } else {
        "Expand into the full Agent Chat conversation"
    };

    vec![
        Action::new(
            "focused-text-action-replace",
            "Replace Selected Text",
            Some("Replace the captured focused field with the latest response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}\u{21b5}")
        .with_icon(IconName::ArrowRight)
        .with_section("Focused Text"),
        Action::new(
            "focused-text-action-append",
            "Append to Selected Text",
            Some("Append the latest response to the captured focused field".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Plus)
        .with_section("Focused Text"),
        Action::new(
            "focused-text-action-copy",
            "Copy Response",
            Some("Copy the latest response without changing the focused field".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Copy)
        .with_section("Focused Text"),
        Action::new(
            expand_id,
            expand_label,
            Some(expand_description.to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::MessageCircle)
        .with_section("Focused Text"),
        Action::new(
            "focused-text-action-retry",
            "Retry",
            Some("Retry the last focused-text turn".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Refresh)
        .with_section("Focused Text"),
        Action::new(
            "focused-text-action-stop",
            "Stop",
            Some("Stop the current focused-text response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("Esc")
        .with_icon(IconName::Close)
        .with_section("Focused Text"),
    ]
}

pub(crate) fn get_focused_text_agent_chat_root_route(
    expanded: bool,
) -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_ROOT_ROUTE_ID.to_string(),
        actions: get_focused_text_agent_chat_actions(expanded),
        context_title: Some("Focused Text".to_string()),
        search_placeholder: Some("Search focused text actions...".to_string()),
        initial_selected_action_id: Some("focused-text-action-replace".to_string()),
    }
}

// ── Agent Chat route builders ───────────────────────────────────────────────────────
/// Action ID for the root-level "Change Model" entry that pushes the model picker.
pub const AGENT_CHAT_CHANGE_MODEL_ACTION_ID: &str = "agent_chat:change_model";
/// Action ID for the root-level "Profile picker" entry that pushes Agent Chat profiles.
pub const AGENT_CHAT_CHANGE_PROFILE_ACTION_ID: &str = "agent_chat:change_profile";

/// Stable root labels and descriptions for Agent Chat Actions Menu parity across hosts.
const AGENT_CHAT_CHANGE_MODEL_LABEL: &str = "Change Model";
const AGENT_CHAT_CHANGE_MODEL_DESCRIPTION: &str = "Pick the model for this chat";
const AGENT_CHAT_CHANGE_PROFILE_LABEL: &str = "Profile picker";
const AGENT_CHAT_CHANGE_PROFILE_DESCRIPTION: &str =
    "Pick the Agent Chat profile. Switching profiles starts a new chat";
/// Route ID for the Agent Chat root actions menu.
pub const AGENT_CHAT_ROOT_ROUTE_ID: &str = "agent_chat:root";
/// Route ID for the model picker sub-route.
pub const AGENT_CHAT_MODEL_PICKER_ROUTE_ID: &str = "agent_chat:model_picker";
/// Route ID for the Agent Chat profile picker sub-route.
pub const AGENT_CHAT_PROFILE_PICKER_ROUTE_ID: &str = "agent_chat:profile_picker";
/// Route ID for the Agent Chat history sub-route.
pub const AGENT_CHAT_HISTORY_ROUTE_ID: &str = "agent_chat:history";
/// Prefix for Agent Chat history row actions.
pub const AGENT_CHAT_HISTORY_SELECT_ACTION_PREFIX: &str = "agent_chat_history:select:";
/// Action ID for the root-level receipt-history entry.
pub const AGENT_CHAT_SHOW_RECEIPT_HISTORY_ACTION_ID: &str = "agent_chat_show_receipt_history";
/// Route ID for the Agent Chat receipt-history sub-route.
pub const AGENT_CHAT_RECEIPT_HISTORY_ROUTE_ID: &str = "agent_chat:receipt_history";
/// Prefix for receipt-history copy row actions.
pub const AGENT_CHAT_RECEIPT_HISTORY_COPY_ACTION_PREFIX: &str = "agent_chat_receipt_history:copy:";
/// Maximum receipt rows displayed in the compact ActionsDialog route.
pub const AGENT_CHAT_RECEIPT_HISTORY_ROUTE_LIMIT: usize = 20;

fn agent_chat_model_display_name(
    entry: &crate::ai::agent_chat::ui::config::AgentChatModelEntry,
) -> String {
    entry
        .display_name
        .clone()
        .unwrap_or_else(|| entry.id.clone())
}

pub(crate) fn agent_chat_history_select_action_id(session_id: &str) -> String {
    format!("{AGENT_CHAT_HISTORY_SELECT_ACTION_PREFIX}{session_id}")
}

pub(crate) fn agent_chat_receipt_history_copy_action_id(request_id: &str) -> String {
    format!("{AGENT_CHAT_RECEIPT_HISTORY_COPY_ACTION_PREFIX}{request_id}")
}

pub(crate) fn agent_chat_receipt_history_request_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(AGENT_CHAT_RECEIPT_HISTORY_COPY_ACTION_PREFIX)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentChatModelSelectionActionPlan {
    CurrentModel,
    AvailableModel,
}

impl AgentChatModelSelectionActionPlan {
    fn from_is_selected(is_selected: bool) -> Self {
        if is_selected {
            Self::CurrentModel
        } else {
            Self::AvailableModel
        }
    }

    fn picker_title(self, display_name: &str) -> String {
        match self {
            Self::CurrentModel => format!("{display_name} \u{2713}"),
            Self::AvailableModel => display_name.to_string(),
        }
    }

    fn description(self, display_name: &str) -> String {
        match self {
            Self::CurrentModel => format!("Currently selected model: {display_name}"),
            Self::AvailableModel => format!("Switch Agent Chat to {display_name}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AgentChatRootPickerActionPlan {
    CurrentSelection(String),
    NoCurrentSelection,
}

impl AgentChatRootPickerActionPlan {
    fn from_selected_display_name(selected_display_name: Option<String>) -> Self {
        selected_display_name
            .map(Self::CurrentSelection)
            .unwrap_or(Self::NoCurrentSelection)
    }

    fn description(&self, fallback_description: &'static str) -> String {
        match self {
            Self::CurrentSelection(display_name) => format!("Current: {display_name}"),
            Self::NoCurrentSelection => fallback_description.to_string(),
        }
    }
}

fn agent_chat_model_switch_description(
    entry: &crate::ai::agent_chat::ui::config::AgentChatModelEntry,
    is_selected: bool,
) -> String {
    let display_name = agent_chat_model_display_name(entry);
    AgentChatModelSelectionActionPlan::from_is_selected(is_selected).description(&display_name)
}

fn agent_chat_profile_backend_label(_backend: crate::config::AgentChatBackend) -> &'static str {
    "Pi"
}

fn agent_chat_profile_source_label(
    source: crate::ai::agent_chat::profiles::AgentChatProfileSource,
) -> &'static str {
    match source {
        crate::ai::agent_chat::profiles::AgentChatProfileSource::BuiltIn => "Built-in",
        crate::ai::agent_chat::profiles::AgentChatProfileSource::User => "Custom",
        crate::ai::agent_chat::profiles::AgentChatProfileSource::Plugin => "Plugin",
    }
}

fn agent_chat_profile_picker_entries(
) -> Vec<crate::ai::agent_chat::profiles::AgentChatProfilePickerEntry> {
    let ai_preferences = crate::config::load_user_preferences().ai;
    let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
    crate::ai::agent_chat::profiles::agent_chat_profile_picker_entries(&ai_preferences, &ctx)
}

fn selected_agent_chat_profile_picker_id() -> String {
    let ai_preferences = crate::config::load_user_preferences().ai;
    let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
    crate::ai::agent_chat::profiles::selected_agent_chat_profile_picker_id(&ai_preferences, &ctx)
}

fn agent_chat_profile_picker_actions(
    entries: &[crate::ai::agent_chat::profiles::AgentChatProfilePickerEntry],
    selected_profile_id: &str,
) -> Vec<Action> {
    entries
        .iter()
        .map(|entry| {
            let is_selected = entry.id == selected_profile_id;
            let title = if is_selected {
                format!("{} \u{2713}", entry.name)
            } else {
                entry.name.clone()
            };
            let backend = agent_chat_profile_backend_label(entry.backend);
            let source = agent_chat_profile_source_label(entry.source);
            let description = if is_selected {
                format!("Currently selected. {source} · {backend}")
            } else {
                format!(
                    "Switch to this profile in a new chat. {source} · {backend} · Starts a new chat when a conversation is already active"
                )
            };

            Action::new(
                agent_chat_switch_profile_action_id(&entry.id),
                title,
                Some(description),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Settings)
        })
        .collect()
}

/// Build the root-level Agent Chat actions list. Includes the profile and model
/// picker entries (which trigger drill-down) plus the standard Agent Chat chat
/// actions. (The legacy "Change Agent" picker was removed — all sessions use
/// the Pi backend, with provider/model chosen via the Shift+Tab picker.)
pub(crate) fn get_agent_chat_root_actions(
    available_models: &[crate::ai::agent_chat::ui::config::AgentChatModelEntry],
    selected_model_id: Option<&str>,
    standing_approval_count: usize,
    thread_summaries: &[crate::ai::agent_chat::ui::AgentChatThreadSummary],
    fork_points: &[crate::ai::agent_chat::ui::AgentChatForkPoint],
) -> Vec<Action> {
    let profile_entries = agent_chat_profile_picker_entries();
    let selected_profile_id = selected_agent_chat_profile_picker_id();
    let selected_profile_name = profile_entries
        .iter()
        .find(|entry| entry.id == selected_profile_id)
        .map(|entry| entry.name.clone());
    let selected_model =
        selected_model_id.and_then(|id| available_models.iter().find(|entry| entry.id == id));
    let model_picker_plan = AgentChatRootPickerActionPlan::from_selected_display_name(
        selected_model.map(agent_chat_model_display_name),
    );
    let profile_picker_plan =
        AgentChatRootPickerActionPlan::from_selected_display_name(selected_profile_name);

    let mut actions = vec![Action::new(
        AGENT_CHAT_CHANGE_PROFILE_ACTION_ID,
        AGENT_CHAT_CHANGE_PROFILE_LABEL,
        Some(profile_picker_plan.description(AGENT_CHAT_CHANGE_PROFILE_DESCRIPTION)),
        ActionCategory::ScriptContext,
    )
    .with_icon(IconName::Settings)
    .with_section("Agent")];

    if !available_models.is_empty() {
        actions.push(
            Action::new(
                AGENT_CHAT_CHANGE_MODEL_ACTION_ID,
                AGENT_CHAT_CHANGE_MODEL_LABEL,
                Some(model_picker_plan.description(AGENT_CHAT_CHANGE_MODEL_DESCRIPTION)),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Settings)
            .with_section("Agent"),
        );
    }

    if standing_approval_count > 0 {
        actions.push(
            Action::new(
                AGENT_CHAT_REVIEW_APPROVALS_ACTION_ID,
                format!("Review Auto-Approvals ({standing_approval_count})"),
                Some("List the permissions this session was granted with Allow Always".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Check)
            .with_section("Agent"),
        );
    }

    actions.push(
        Action::new(
            AGENT_CHAT_NEW_THREAD_ACTION_ID,
            "New Thread",
            Some(
                "Start a fresh thread; the current conversation keeps running in the background"
                    .to_string(),
            ),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}N")
        .with_icon(IconName::Plus)
        .with_section("Threads"),
    );

    for summary in thread_summaries {
        let mut title = format!("Switch to: {}", summary.title);
        if summary.unread > 0 {
            title.push_str(&format!(" ({} new)", summary.unread));
        }
        let description = if summary.is_streaming {
            "Streaming in the background — switch to view live output"
        } else {
            "Resume this background thread"
        };
        actions.push(
            Action::new(
                agent_chat_switch_thread_action_id(&summary.ui_thread_id),
                title,
                Some(description.to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::MessageCircle)
            .with_section("Threads"),
        );
    }

    if !fork_points.is_empty() {
        actions.push(
            Action::new(
                AGENT_CHAT_REWIND_ACTION_ID,
                "Rewind & Edit Message",
                Some(
                    "Pick an earlier message: the conversation rewinds to that point and the text returns to the composer for editing"
                        .to_string(),
                ),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Pencil)
            .with_section("Session"),
        );
    }

    actions.extend(get_agent_chat_actions());
    actions
}

/// Build the second-level Agent Chat profile picker actions.
pub(crate) fn get_agent_chat_profile_picker_actions() -> Vec<Action> {
    let entries = agent_chat_profile_picker_entries();
    let selected_profile_id = selected_agent_chat_profile_picker_id();
    agent_chat_profile_picker_actions(&entries, &selected_profile_id)
}

/// Build the second-level Agent Chat model picker actions.
pub(crate) fn get_agent_chat_model_picker_actions(
    available_models: &[crate::ai::agent_chat::ui::config::AgentChatModelEntry],
    selected_model_id: Option<&str>,
) -> Vec<Action> {
    let selected_model_id = selected_model_id.filter(|id| !id.is_empty());
    available_models
        .iter()
        .map(|entry| {
            let is_selected = selected_model_id == Some(entry.id.as_str());
            let selection_plan = AgentChatModelSelectionActionPlan::from_is_selected(is_selected);
            let display_name = agent_chat_model_display_name(entry);
            Action::new(
                agent_chat_switch_model_action_id(&entry.id),
                selection_plan.picker_title(&display_name),
                Some(agent_chat_model_switch_description(entry, is_selected)),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Settings)
        })
        .collect()
}

// ── Host-aware Agent Chat action filtering ─────────────────────────────────────────

/// Distinguishes whether the Agent Chat actions dialog is hosted in the shared main
/// panel or in the detached Agent Chat chat window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatActionsDialogHost {
    /// Shared Agent Chat surface in the main Script Kit panel — all actions available.
    Shared,
    /// Notes-hosted Agent Chat surface — subset that works inside the Notes window.
    /// `agent_chat_close` returns to the Notes editor rather than closing a window.
    /// `agent_chat_save_as_note` saves the embedded transcript as a new canonical note.
    Notes,
    /// Detached Agent Chat chat window — only actions that work without the main panel.
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentChatHostActionPlan {
    IncludeWithShortcut,
    IncludeWithoutShortcut,
    Exclude,
}

impl AgentChatHostActionPlan {
    fn is_included(self) -> bool {
        !matches!(self, Self::Exclude)
    }

    fn keeps_shortcut(self) -> bool {
        matches!(self, Self::IncludeWithShortcut)
    }
}

fn agent_chat_host_action_plan(
    host: AgentChatActionsDialogHost,
    action_id: &str,
) -> AgentChatHostActionPlan {
    match host {
        AgentChatActionsDialogHost::Shared => {
            if action_id == "agent_chat_close" {
                AgentChatHostActionPlan::IncludeWithoutShortcut
            } else {
                AgentChatHostActionPlan::IncludeWithShortcut
            }
        }
        AgentChatActionsDialogHost::Notes => {
            // Notes-hosted: same as Detached, keeping `agent_chat_close`
            // (returns to editor), and routing `agent_chat_show_history`
            // through the shared ActionsDialog.
            if matches!(
                action_id,
                "agent_chat:change_profile"
                    | "agent_chat:change_model"
                    | "agent_chat_copy_last_response"
                    | "agent_chat_retry_last"
                    | "agent_chat_export_markdown"
                    | "agent_chat_save_as_note"
                    | "agent_chat_show_history"
                    | AGENT_CHAT_SHOW_RECEIPT_HISTORY_ACTION_ID
                    | "agent_chat_scroll_to_top"
                    | "agent_chat_scroll_to_bottom"
                    | "agent_chat_expand_all"
                    | "agent_chat_collapse_all"
                    | "agent_chat_new_conversation"
                    | "agent_chat_clear_history"
                    | "agent_chat_close"
                    | AGENT_CHAT_NEW_THREAD_ACTION_ID
                    | AGENT_CHAT_REWIND_ACTION_ID
            ) || action_id.starts_with(AGENT_CHAT_SWITCH_PROFILE_ACTION_PREFIX)
                || action_id.starts_with(AGENT_CHAT_SWITCH_MODEL_ACTION_PREFIX)
                || action_id.starts_with(AGENT_CHAT_SWITCH_THREAD_ACTION_PREFIX)
                || action_id.starts_with(AGENT_CHAT_FORK_EDIT_ACTION_PREFIX)
            {
                if action_id == "agent_chat_close" {
                    AgentChatHostActionPlan::IncludeWithoutShortcut
                } else {
                    AgentChatHostActionPlan::IncludeWithShortcut
                }
            } else {
                AgentChatHostActionPlan::Exclude
            }
        }
        AgentChatActionsDialogHost::Detached => {
            if matches!(
                action_id,
                "agent_chat:change_model"
                    | "agent_chat_copy_last_response"
                    | "agent_chat_retry_last"
                    | "agent_chat_export_markdown"
                    | "agent_chat_save_as_note"
                    | "agent_chat_show_history"
                    | AGENT_CHAT_SHOW_RECEIPT_HISTORY_ACTION_ID
                    | "agent_chat_scroll_to_top"
                    | "agent_chat_scroll_to_bottom"
                    | "agent_chat_expand_all"
                    | "agent_chat_collapse_all"
                    | "agent_chat_new_conversation"
                    | "agent_chat_clear_history"
                    | "agent_chat_close"
                    | AGENT_CHAT_NEW_THREAD_ACTION_ID
                    | AGENT_CHAT_REWIND_ACTION_ID
            ) || action_id.starts_with(AGENT_CHAT_SWITCH_MODEL_ACTION_PREFIX)
                || action_id.starts_with(AGENT_CHAT_SWITCH_THREAD_ACTION_PREFIX)
                || action_id.starts_with(AGENT_CHAT_FORK_EDIT_ACTION_PREFIX)
            {
                AgentChatHostActionPlan::IncludeWithShortcut
            } else {
                AgentChatHostActionPlan::Exclude
            }
        }
    }
}

fn filter_agent_chat_actions_for_host(
    host: AgentChatActionsDialogHost,
    actions: Vec<Action>,
) -> Vec<Action> {
    let host_label = match host {
        AgentChatActionsDialogHost::Shared => "shared",
        AgentChatActionsDialogHost::Notes => "notes",
        AgentChatActionsDialogHost::Detached => "detached",
    };
    actions
        .into_iter()
        .filter_map(|mut action| {
            let plan = agent_chat_host_action_plan(host, &action.id);
            if !plan.is_included() {
                tracing::warn!(
                    event = "agent_chat_actions_menu_filtered",
                    host = host_label,
                    action_id = %action.id,
                    reason = "unsupported_in_host",
                    "Filtered unsupported Agent Chat Actions Menu item"
                );
                return None;
            }
            if !plan.keeps_shortcut() {
                action.shortcut = None;
                action.shortcut_tokens = None;
                action.shortcut_lower = None;
            }
            Some(action)
        })
        .collect()
}

/// Build an `ActionsDialogRoute` for the Agent Chat root menu, filtered for the given host.
pub(crate) fn get_agent_chat_root_route_for_host(
    available_models: &[crate::ai::agent_chat::ui::config::AgentChatModelEntry],
    selected_model_id: Option<&str>,
    standing_approval_count: usize,
    thread_summaries: &[crate::ai::agent_chat::ui::AgentChatThreadSummary],
    fork_points: &[crate::ai::agent_chat::ui::AgentChatForkPoint],
    host: AgentChatActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    let host_label = match host {
        AgentChatActionsDialogHost::Shared => "shared",
        AgentChatActionsDialogHost::Notes => "notes",
        AgentChatActionsDialogHost::Detached => "detached",
    };

    let actions = filter_agent_chat_actions_for_host(
        host,
        get_agent_chat_root_actions(
            available_models,
            selected_model_id,
            standing_approval_count,
            thread_summaries,
            fork_points,
        ),
    );

    let model_count = available_models.len();
    tracing::info!(
        event = "agent_chat_actions_menu_built",
        host = host_label,
        model_count,
        action_count = actions.len(),
        "Built Agent Chat Actions Menu"
    );

    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_ROOT_ROUTE_ID.to_string(),
        actions,
        context_title: Some("Agent Chat".to_string()),
        search_placeholder: Some("Search Agent Chat actions...".to_string()),
        initial_selected_action_id: Some(AGENT_CHAT_CHANGE_PROFILE_ACTION_ID.to_string()),
    }
}

/// Build an `ActionsDialogRoute` for the Agent Chat model picker sub-route, filtered for the given host.
pub(crate) fn get_agent_chat_model_picker_route_for_host(
    available_models: &[crate::ai::agent_chat::ui::config::AgentChatModelEntry],
    selected_model_id: Option<&str>,
    host: AgentChatActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_MODEL_PICKER_ROUTE_ID.to_string(),
        actions: filter_agent_chat_actions_for_host(
            host,
            get_agent_chat_model_picker_actions(available_models, selected_model_id),
        ),
        context_title: Some("Change Model".to_string()),
        search_placeholder: Some("Search models...".to_string()),
        initial_selected_action_id: selected_model_id.map(agent_chat_switch_model_action_id),
    }
}

/// Build the second-level rewind checkpoint picker actions (latest first,
/// since the most recent message is the most likely edit target).
pub(crate) fn get_agent_chat_fork_picker_actions(
    fork_points: &[crate::ai::agent_chat::ui::AgentChatForkPoint],
) -> Vec<Action> {
    fork_points
        .iter()
        .rev()
        .map(|point| {
            Action::new(
                agent_chat_fork_edit_action_id(&point.entry_id),
                agent_chat_message_action_title(&point.text),
                Some("Rewind here and edit this message; later replies are discarded".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Pencil)
            .with_section("Messages")
        })
        .collect()
}

/// Build an `ActionsDialogRoute` for the rewind checkpoint picker.
pub(crate) fn get_agent_chat_fork_picker_route_for_host(
    fork_points: &[crate::ai::agent_chat::ui::AgentChatForkPoint],
    host: AgentChatActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_FORK_PICKER_ROUTE_ID.to_string(),
        actions: filter_agent_chat_actions_for_host(
            host,
            get_agent_chat_fork_picker_actions(fork_points),
        ),
        context_title: Some("Rewind & Edit".to_string()),
        search_placeholder: Some("Search messages...".to_string()),
        initial_selected_action_id: fork_points
            .last()
            .map(|point| agent_chat_fork_edit_action_id(&point.entry_id)),
    }
}

/// Build an `ActionsDialogRoute` for recent Agent Chat conversations.
pub(crate) fn get_agent_chat_history_route() -> crate::actions::ActionsDialogRoute {
    let actions = crate::ai::agent_chat::ui::history::load_history()
        .into_iter()
        .take(100)
        .map(|entry| {
            Action::new(
                agent_chat_history_select_action_id(&entry.session_id),
                entry.title_display().to_string(),
                Some(entry.preview_display().to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::MessageCircle)
            .with_section("History")
        })
        .collect();

    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_HISTORY_ROUTE_ID.to_string(),
        actions,
        context_title: Some("Agent Chat History".to_string()),
        search_placeholder: Some("Search conversation history...".to_string()),
        initial_selected_action_id: None,
    }
}

/// Build an `ActionsDialogRoute` for recent semantic automation receipts.
pub(crate) fn get_agent_chat_receipt_history_route() -> crate::actions::ActionsDialogRoute {
    let mut actions = crate::agentic_protocol_bus::load_recent_protocol_response_summaries(
        AGENT_CHAT_RECEIPT_HISTORY_ROUTE_LIMIT,
    )
    .into_iter()
    .map(|summary| {
        let description = format!(
            "{} · session {} · {}",
            summary.preview,
            summary.session,
            summary
                .automation_id
                .as_deref()
                .unwrap_or("no automation target")
        );
        Action::new(
            agent_chat_receipt_history_copy_action_id(&summary.request_id),
            format!("{} · {}", summary.response_type, summary.request_id),
            Some(description),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Receipts")
    })
    .collect::<Vec<_>>();

    if actions.is_empty() {
        actions.push(
            Action::new(
                "agent_chat_receipt_history_empty",
                "No receipt history",
                Some("No protocol receipts have been recorded for this app session.".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::FileCode)
            .with_section("Receipts"),
        );
    }

    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_RECEIPT_HISTORY_ROUTE_ID.to_string(),
        actions,
        context_title: Some("Receipt History".to_string()),
        search_placeholder: Some("Search receipts...".to_string()),
        initial_selected_action_id: None,
    }
}

/// Build an `ActionsDialogRoute` for the Agent Chat profile picker sub-route.
pub(crate) fn get_agent_chat_profile_picker_route_for_host(
    host: AgentChatActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: AGENT_CHAT_PROFILE_PICKER_ROUTE_ID.to_string(),
        actions: filter_agent_chat_actions_for_host(host, get_agent_chat_profile_picker_actions()),
        context_title: Some("Profile picker".to_string()),
        search_placeholder: Some("Search profiles...".to_string()),
        initial_selected_action_id: Some(agent_chat_switch_profile_action_id(
            &selected_agent_chat_profile_picker_id(),
        )),
    }
}

/// Build an `ActionsDialogRoute` for the Agent Chat profile picker sub-route (shared host).
#[allow(dead_code)]
pub(crate) fn get_agent_chat_profile_picker_route() -> crate::actions::ActionsDialogRoute {
    get_agent_chat_profile_picker_route_for_host(AgentChatActionsDialogHost::Shared)
}

/// Build an `ActionsDialogRoute` for the Agent Chat root menu (shared host).
#[allow(dead_code)]
pub(crate) fn get_agent_chat_root_route(
    available_models: &[crate::ai::agent_chat::ui::config::AgentChatModelEntry],
    selected_model_id: Option<&str>,
) -> crate::actions::ActionsDialogRoute {
    get_agent_chat_root_route_for_host(
        available_models,
        selected_model_id,
        0,
        &[],
        &[],
        AgentChatActionsDialogHost::Shared,
    )
}

/// Build an `ActionsDialogRoute` for the Agent Chat model picker sub-route (shared host).
#[allow(dead_code)]
pub(crate) fn get_agent_chat_model_picker_route(
    available_models: &[crate::ai::agent_chat::ui::config::AgentChatModelEntry],
    selected_model_id: Option<&str>,
) -> crate::actions::ActionsDialogRoute {
    get_agent_chat_model_picker_route_for_host(
        available_models,
        selected_model_id,
        AgentChatActionsDialogHost::Shared,
    )
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

    fn find_action_description(actions: &[Action], id: &str) -> Option<String> {
        actions
            .iter()
            .find(|action| action.id == id)
            .and_then(|action| action.description.clone())
    }

    fn has_action(actions: &[Action], id: &str) -> bool {
        actions.iter().any(|action| action.id == id)
    }

    fn assert_all_actions_have_icons(context: &str, actions: &[Action]) {
        for action in actions {
            assert!(
                action.icon.is_some(),
                "context '{context}' action '{}' should include an icon",
                action.id
            );
        }
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
    fn test_get_script_context_actions_preserves_builtin_action_text() {
        let script = ScriptInfo::with_action_verb(
            "Agent Chat",
            "builtin:builtin/ai-chat",
            false,
            "Open Agent Chat",
        );

        let actions = get_script_context_actions(&script);

        assert_eq!(find_action_title(&actions, "run_script"), "Open Agent Chat");
        assert_eq!(
            find_action_description(&actions, "run_script").as_deref(),
            Some("Open Agent Chat")
        );
    }

    #[test]
    fn test_primary_action_plan_classifies_context_kind_matrix() {
        let app = ScriptInfo::app("App", "/Applications/App.app", None, None, None);
        let agent = ScriptInfo::agent("Agent", "/tmp/agent.md", None, None);
        let scriptlet = ScriptInfo::scriptlet("Scriptlet", "/tmp/scriptlets.md", None, None);
        let script = ScriptInfo::new("Script", "/tmp/script.ts");
        let skill = ScriptInfo::with_action_verb("Skill", "skill:scriptkit:demo", false, "open");
        let builtin = ScriptInfo::with_action_verb(
            "Agent Chat",
            "builtin:builtin/ai-chat",
            false,
            "Open Agent Chat",
        );
        let generic = ScriptInfo::with_action_verb("Thing", "virtual:thing", false, "open");
        let typed_builtin_path =
            ScriptInfo::with_action_verb("Typed", "builtin:builtin/new-script", true, "run");

        assert_eq!(script_context_kind(&app), ScriptContextKind::App);
        assert_eq!(script_context_kind(&agent), ScriptContextKind::Agent);
        assert_eq!(
            script_context_kind(&scriptlet),
            ScriptContextKind::Scriptlet
        );
        assert_eq!(script_context_kind(&script), ScriptContextKind::Script);
        assert_eq!(script_context_kind(&skill), ScriptContextKind::Skill);
        assert_eq!(script_context_kind(&builtin), ScriptContextKind::BuiltIn);
        assert_eq!(script_context_kind(&generic), ScriptContextKind::Generic);
        assert_eq!(
            script_context_kind(&typed_builtin_path),
            ScriptContextKind::Script
        );
    }

    #[test]
    fn test_primary_action_copy_matrix() {
        let mut script = ScriptInfo::new("Script", "/tmp/script.ts");
        script.action_verb = "run".to_string();
        let mut app = ScriptInfo::app("App", "/Applications/App.app", None, None, None);
        app.action_verb = "launch".to_string();
        let mut scriptlet = ScriptInfo::scriptlet("Scriptlet", "/tmp/scriptlets.md", None, None);
        scriptlet.action_verb = "run".to_string();
        let mut agent = ScriptInfo::agent("Agent", "/tmp/agent.md", None, None);
        agent.action_verb = "open".to_string();
        let skill = ScriptInfo::with_action_verb("Skill", "skill:scriptkit:demo", false, "open");
        let generic = ScriptInfo::with_action_verb("Thing", "virtual:thing", false, "open");
        let builtin = ScriptInfo::with_action_verb(
            "Agent Chat",
            "builtin:builtin/ai-chat",
            false,
            "Open Agent Chat",
        );

        for (context, script, expected) in [
            (
                "script",
                script,
                PrimaryActionCopy {
                    title: "Run".to_string(),
                    description: "run this script".to_string(),
                },
            ),
            (
                "app",
                app,
                PrimaryActionCopy {
                    title: "Launch".to_string(),
                    description: "launch this application".to_string(),
                },
            ),
            (
                "scriptlet",
                scriptlet,
                PrimaryActionCopy {
                    title: "Run".to_string(),
                    description: "run this scriptlet".to_string(),
                },
            ),
            (
                "agent",
                agent,
                PrimaryActionCopy {
                    title: "Open".to_string(),
                    description: "open this agent".to_string(),
                },
            ),
            (
                "skill",
                skill,
                PrimaryActionCopy {
                    title: "Open".to_string(),
                    description: "open this skill".to_string(),
                },
            ),
            (
                "generic",
                generic,
                PrimaryActionCopy {
                    title: "Open".to_string(),
                    description: "open this item".to_string(),
                },
            ),
            (
                "builtin",
                builtin,
                PrimaryActionCopy {
                    title: "Open Agent Chat".to_string(),
                    description: "Open Agent Chat".to_string(),
                },
            ),
        ] {
            assert_eq!(primary_action_copy(&script), expected, "{context}");
        }
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
        assert_eq!(find_action_title(&actions, "copy_deeplink"), "Share");
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

    #[test]
    fn test_get_script_context_actions_assigns_icons_for_all_contexts() {
        let script = ScriptInfo::new("Script", "/tmp/script-context-icon-test.ts");
        let builtin = ScriptInfo::builtin("Clipboard History");
        let scriptlet =
            ScriptInfo::scriptlet("Scriptlet", "/tmp/script-context-icon-test.md", None, None);
        let agent = ScriptInfo::agent(
            "Agent",
            "/tmp/script-context-icon-test.agent.md",
            None,
            None,
        );

        let script_actions = get_script_context_actions(&script);
        assert!(
            !script_actions.is_empty(),
            "script actions should not be empty"
        );
        assert_all_actions_have_icons("script", &script_actions);

        let builtin_actions = get_script_context_actions(&builtin);
        assert!(
            !builtin_actions.is_empty(),
            "builtin actions should not be empty"
        );
        assert_all_actions_have_icons("builtin", &builtin_actions);

        let scriptlet_actions = get_script_context_actions(&scriptlet);
        assert!(
            !scriptlet_actions.is_empty(),
            "scriptlet actions should not be empty"
        );
        assert_all_actions_have_icons("scriptlet", &scriptlet_actions);

        let agent_actions = get_script_context_actions(&agent);
        assert!(
            !agent_actions.is_empty(),
            "agent actions should not be empty"
        );
        assert_all_actions_have_icons("agent", &agent_actions);
    }

    #[test]
    fn test_script_context_actions_include_toggle_info_with_cmd_i() {
        let script = ScriptInfo::new("TestScript", "/tmp/info-test.ts");
        let actions = get_script_context_actions(&script);

        let info_action = actions
            .iter()
            .find(|a| a.id == "toggle_info")
            .expect("script context actions must include toggle_info");

        assert_eq!(info_action.title, "Show Info");
        assert_eq!(
            info_action.shortcut.as_deref(),
            Some("⌘I"),
            "toggle_info action must have ⌘I shortcut for discoverability"
        );
        assert_eq!(
            info_action.section.as_deref(),
            Some("Actions"),
            "toggle_info must appear in the Actions section"
        );
        assert!(
            info_action.icon.is_some(),
            "toggle_info must have an icon for visual consistency"
        );
    }

    #[test]
    fn test_get_script_context_actions_includes_app_actions_when_is_app() {
        let script = ScriptInfo::app(
            "Google Chrome",
            "/Applications/Google Chrome.app",
            Some("com.google.Chrome".to_string()),
            None,
            None,
        );
        let actions = get_script_context_actions(&script);

        // App-specific actions
        assert!(has_action(&actions, "reveal_in_finder"));
        assert!(has_action(&actions, "show_info_in_finder"));
        assert!(has_action(&actions, "show_package_contents"));
        assert!(has_action(&actions, "copy_name"));
        assert!(has_action(&actions, "copy_path"));
        assert!(has_action(&actions, "copy_bundle_id"));
        assert!(has_action(&actions, "quit_app"));
        assert!(has_action(&actions, "force_quit_app"));
        assert!(has_action(&actions, "restart_app"));

        // Common actions still present
        assert!(has_action(&actions, "run_script"));
        assert!(has_action(&actions, "toggle_info"));
        assert!(has_action(&actions, "copy_deeplink"));
    }

    #[test]
    fn test_get_script_context_actions_omits_copy_bundle_id_when_none() {
        let script = ScriptInfo::app("MyApp", "/Applications/MyApp.app", None, None, None);
        let actions = get_script_context_actions(&script);

        assert!(!has_action(&actions, "copy_bundle_id"));
        // Other app actions still present
        assert!(has_action(&actions, "reveal_in_finder"));
        assert!(has_action(&actions, "quit_app"));
    }

    #[test]
    fn test_get_script_context_actions_app_does_not_include_script_only_actions() {
        let script = ScriptInfo::app(
            "Safari",
            "/Applications/Safari.app",
            Some("com.apple.Safari".to_string()),
            None,
            None,
        );
        let actions = get_script_context_actions(&script);

        assert!(!has_action(&actions, "edit_script"));
        assert!(!has_action(&actions, "view_logs"));
        assert!(!has_action(&actions, "copy_content"));
        assert!(!has_action(&actions, "delete_script"));
        assert!(!has_action(&actions, "edit_scriptlet"));
    }

    #[test]
    fn test_get_script_context_actions_includes_favorites_for_apps() {
        let script = ScriptInfo::app(
            "Safari",
            "/Applications/Safari.app",
            Some("com.apple.Safari".to_string()),
            None,
            None,
        );
        let actions = get_script_context_actions(&script);

        assert!(has_action(&actions, "toggle_favorite"));
    }

    #[test]
    fn test_get_script_context_actions_app_actions_all_have_icons() {
        let script = ScriptInfo::app(
            "Chrome",
            "/Applications/Google Chrome.app",
            Some("com.google.Chrome".to_string()),
            None,
            None,
        );
        let actions = get_script_context_actions(&script);
        assert_all_actions_have_icons("app", &actions);
    }

    #[test]
    fn test_toggle_info_appears_for_all_script_types() {
        let script = ScriptInfo::new("Script", "/tmp/all-types-info.ts");
        let builtin = ScriptInfo::builtin("Clipboard History");
        let scriptlet = ScriptInfo::scriptlet("Scriptlet", "/tmp/all-types-info.md", None, None);
        let agent = ScriptInfo::agent("Agent", "/tmp/all-types-info.agent.md", None, None);

        for (label, actions) in [
            ("script", get_script_context_actions(&script)),
            ("builtin", get_script_context_actions(&builtin)),
            ("scriptlet", get_script_context_actions(&scriptlet)),
            ("agent", get_script_context_actions(&agent)),
        ] {
            assert!(
                actions.iter().any(|a| a.id == "toggle_info"),
                "toggle_info must be present in {label} context actions"
            );
        }
    }

    fn sample_agent_chat_model(
        id: &str,
        display_name: &str,
    ) -> crate::ai::agent_chat::ui::config::AgentChatModelEntry {
        crate::ai::agent_chat::ui::config::AgentChatModelEntry {
            id: id.to_string(),
            display_name: Some(display_name.to_string()),
            context_window: None,
        }
    }

    #[test]
    fn test_agent_chat_close_shortcut_is_only_advertised_for_detached_host() {
        let shared = get_agent_chat_root_route_for_host(
            &[],
            None,
            0,
            &[],
            &[],
            AgentChatActionsDialogHost::Shared,
        );
        let notes = get_agent_chat_root_route_for_host(
            &[],
            None,
            0,
            &[],
            &[],
            AgentChatActionsDialogHost::Notes,
        );
        let detached = get_agent_chat_root_route_for_host(
            &[],
            None,
            0,
            &[],
            &[],
            AgentChatActionsDialogHost::Detached,
        );

        let shared_close = shared
            .actions
            .iter()
            .find(|action| action.id == "agent_chat_close")
            .expect("shared agent_chat_close action should exist");
        let notes_close = notes
            .actions
            .iter()
            .find(|action| action.id == "agent_chat_close")
            .expect("notes agent_chat_close action should exist");
        let detached_close = detached
            .actions
            .iter()
            .find(|action| action.id == "agent_chat_close")
            .expect("detached agent_chat_close action should exist");

        assert!(shared_close.shortcut.is_none());
        assert!(notes_close.shortcut.is_none());
        assert_eq!(detached_close.shortcut.as_deref(), Some("⌘W"));
    }

    #[test]
    fn test_agent_chat_host_action_plan_matrix() {
        for host in [
            AgentChatActionsDialogHost::Shared,
            AgentChatActionsDialogHost::Notes,
        ] {
            assert_eq!(
                agent_chat_host_action_plan(host, "agent_chat_close"),
                AgentChatHostActionPlan::IncludeWithoutShortcut,
                "{host:?} should keep close available without advertising Cmd-W"
            );
        }

        assert_eq!(
            agent_chat_host_action_plan(AgentChatActionsDialogHost::Detached, "agent_chat_close"),
            AgentChatHostActionPlan::IncludeWithShortcut
        );
        assert_eq!(
            agent_chat_host_action_plan(
                AgentChatActionsDialogHost::Notes,
                "agent_chat_save_as_note"
            ),
            AgentChatHostActionPlan::IncludeWithShortcut
        );
        assert_eq!(
            agent_chat_host_action_plan(
                AgentChatActionsDialogHost::Detached,
                "agent_chat_show_history"
            ),
            AgentChatHostActionPlan::IncludeWithShortcut
        );

        for host in [
            AgentChatActionsDialogHost::Shared,
            AgentChatActionsDialogHost::Notes,
            AgentChatActionsDialogHost::Detached,
        ] {
            assert_eq!(
                agent_chat_host_action_plan(host, "agent_chat_switch_model:gpt"),
                AgentChatHostActionPlan::IncludeWithShortcut
            );
        }
    }

    #[test]
    fn test_agent_chat_root_actions_add_change_model_when_models_exist() {
        let actions = get_agent_chat_root_actions(
            &[
                sample_agent_chat_model("claude-sonnet-4-6", "Sonnet 4.6"),
                sample_agent_chat_model("claude-opus-4-6", "Opus 4.6"),
            ],
            Some("claude-sonnet-4-6"),
            0,
            &[],
            &[],
        );

        let change_model = actions
            .iter()
            .find(|action| action.id == AGENT_CHAT_CHANGE_MODEL_ACTION_ID)
            .expect("change model action should exist");
        assert_eq!(change_model.section.as_deref(), Some("Agent"));
        assert_eq!(
            change_model.description.as_deref(),
            Some("Current: Sonnet 4.6")
        );
    }

    #[test]
    fn test_agent_chat_root_actions_surface_review_approvals_only_when_grants_exist() {
        let without_grants = get_agent_chat_root_actions(&[], None, 0, &[], &[]);
        assert!(
            !without_grants
                .iter()
                .any(|action| action.id == AGENT_CHAT_REVIEW_APPROVALS_ACTION_ID),
            "no review action when the session has no standing grants"
        );

        let with_grants = get_agent_chat_root_actions(&[], None, 2, &[], &[]);
        let review = with_grants
            .iter()
            .find(|action| action.id == AGENT_CHAT_REVIEW_APPROVALS_ACTION_ID)
            .expect("review action should exist when standing grants exist");
        assert_eq!(review.title, "Review Auto-Approvals (2)");
        assert_eq!(review.section.as_deref(), Some("Agent"));
    }

    #[test]
    fn test_agent_chat_root_actions_surface_thread_switcher() {
        let summaries = vec![crate::ai::agent_chat::ui::AgentChatThreadSummary {
            ui_thread_id: "thread-abc".to_string(),
            title: "Refactor the parser".to_string(),
            unread: 3,
            is_streaming: true,
        }];
        let actions = get_agent_chat_root_actions(&[], None, 0, &summaries, &[]);

        let new_thread = actions
            .iter()
            .find(|action| action.id == AGENT_CHAT_NEW_THREAD_ACTION_ID)
            .expect("New Thread action should always exist");
        assert_eq!(new_thread.section.as_deref(), Some("Threads"));
        assert_eq!(new_thread.shortcut.as_deref(), Some("⌘N"));

        let switch = actions
            .iter()
            .find(|action| action.id == "agent_chat_switch_thread:thread-abc")
            .expect("switch action should exist per retained thread");
        assert_eq!(switch.title, "Switch to: Refactor the parser (3 new)");
        assert_eq!(switch.section.as_deref(), Some("Threads"));

        assert_eq!(
            agent_chat_switch_thread_id_from_action("agent_chat_switch_thread:thread-abc"),
            Some("thread-abc")
        );
        assert_eq!(
            agent_chat_switch_thread_id_from_action("agent_chat_new_thread"),
            None
        );
    }

    #[test]
    fn test_agent_chat_root_actions_surface_rewind_only_with_fork_points() {
        let points = vec![crate::ai::agent_chat::ui::AgentChatForkPoint {
            entry_id: "entry-7".to_string(),
            text: "fix the parser bug".to_string(),
        }];

        let without = get_agent_chat_root_actions(&[], None, 0, &[], &[]);
        assert!(
            !without
                .iter()
                .any(|action| action.id == AGENT_CHAT_REWIND_ACTION_ID),
            "no rewind action without checkpoints"
        );

        let with = get_agent_chat_root_actions(&[], None, 0, &[], &points);
        let rewind = with
            .iter()
            .find(|action| action.id == AGENT_CHAT_REWIND_ACTION_ID)
            .expect("rewind action should exist when fork points exist");
        assert_eq!(rewind.section.as_deref(), Some("Session"));

        let picker = get_agent_chat_fork_picker_actions(&points);
        assert_eq!(picker.len(), 1);
        assert_eq!(picker[0].id, "agent_chat_fork_edit:entry-7");
        assert_eq!(picker[0].title, "fix the parser bug");

        assert_eq!(
            agent_chat_fork_edit_entry_from_action("agent_chat_fork_edit:entry-7"),
            Some("entry-7")
        );
        assert_eq!(
            agent_chat_fork_edit_entry_from_action("agent_chat_rewind_edit"),
            None
        );
    }

    #[test]
    fn test_agent_chat_fork_picker_lists_latest_first() {
        let points = vec![
            crate::ai::agent_chat::ui::AgentChatForkPoint {
                entry_id: "e0".to_string(),
                text: "older".to_string(),
            },
            crate::ai::agent_chat::ui::AgentChatForkPoint {
                entry_id: "e1".to_string(),
                text: "newest".to_string(),
            },
        ];
        let route =
            get_agent_chat_fork_picker_route_for_host(&points, AgentChatActionsDialogHost::Shared);
        assert_eq!(route.id, AGENT_CHAT_FORK_PICKER_ROUTE_ID);
        assert_eq!(route.actions[0].title, "newest");
        assert_eq!(
            route.initial_selected_action_id.as_deref(),
            Some("agent_chat_fork_edit:e1"),
            "latest message preselected"
        );
    }

    #[test]
    fn detached_agent_chat_history_routes_through_actions_dialog() {
        let detached = get_agent_chat_root_route_for_host(
            &[],
            None,
            0,
            &[],
            &[],
            AgentChatActionsDialogHost::Detached,
        );
        assert!(
            detached
                .actions
                .iter()
                .any(|action| action.id == "agent_chat_show_history"),
            "detached Agent Chat must expose history in Cmd+K instead of relying on a PromptPopup-only shortcut"
        );

        assert!(
            agent_chat_history_select_action_id("session-123")
                .starts_with(AGENT_CHAT_HISTORY_SELECT_ACTION_PREFIX),
            "history rows must dispatch through stable session-id action ids"
        );
    }

    #[test]
    fn test_agent_chat_model_picker_actions_mark_selected_model() {
        let actions = get_agent_chat_model_picker_actions(
            &[
                sample_agent_chat_model("claude-sonnet-4-6", "Sonnet 4.6"),
                sample_agent_chat_model("claude-opus-4-6", "Opus 4.6"),
            ],
            Some("claude-opus-4-6"),
        );

        let current = actions
            .iter()
            .find(|action| action.id == "agent_chat_switch_model:claude-opus-4-6")
            .expect("current model action should exist");
        assert_eq!(current.title, "Opus 4.6 ✓");

        let alternate = actions
            .iter()
            .find(|action| action.id == "agent_chat_switch_model:claude-sonnet-4-6")
            .expect("alternate model action should exist");
        assert_eq!(alternate.title, "Sonnet 4.6");
    }

    #[test]
    fn agent_chat_actions_include_cmux_codex_prompt_handoff() {
        let actions = get_agent_chat_actions();
        let action = actions
            .iter()
            .find(|action| action.id == crate::ai::agent_prompt_handoff::CMUX_CODEX_ACTION_ID)
            .expect("Agent Chat actions should expose cmux Codex prompt handoff");

        assert_eq!(action.title, "Send Prompt to cmux Codex");
        assert_eq!(action.section.as_deref(), Some("Handoff"));
        assert!(action.shortcut.is_none());
    }

    #[test]
    fn test_agent_chat_switch_model_action_parser_returns_model_id() {
        assert_eq!(
            agent_chat_switch_model_id_from_action("agent_chat_switch_model:claude-sonnet-4-6"),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(
            agent_chat_switch_model_id_from_action("agent_chat_retry_last"),
            None
        );
    }
}
