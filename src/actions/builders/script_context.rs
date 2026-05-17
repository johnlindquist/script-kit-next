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
    vec![
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
    ]
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
const ACP_SWITCH_AGENT_ACTION_PREFIX: &str = "acp_switch_agent:";
#[allow(dead_code)] // Used by the binary ACP actions surface.
const ACP_SWITCH_MODEL_ACTION_PREFIX: &str = "acp_switch_model:";

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_agent_source_label(source: crate::ai::acp::AcpAgentSource) -> &'static str {
    match source {
        crate::ai::acp::AcpAgentSource::LegacyClaudeCode => "Legacy",
        crate::ai::acp::AcpAgentSource::ScriptKitCatalog => "Catalog",
        crate::ai::acp::AcpAgentSource::BuiltIn => "Built-in",
    }
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_agent_state_label(entry: &crate::ai::acp::AcpAgentCatalogEntry) -> &'static str {
    match (entry.install_state, entry.auth_state, entry.config_state) {
        (crate::ai::acp::AcpAgentInstallState::Unsupported, _, _) => "Unsupported",
        (crate::ai::acp::AcpAgentInstallState::NeedsInstall, _, _) => "Needs install",
        (_, crate::ai::acp::AcpAgentAuthState::NeedsAuthentication, _) => "Needs auth",
        (
            _,
            _,
            crate::ai::acp::AcpAgentConfigState::Missing
            | crate::ai::acp::AcpAgentConfigState::Invalid,
        ) => "Needs config",
        _ => "Ready",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AcpAgentSelectionActionPlan {
    CurrentAgent,
    AvailableAgent,
}

impl AcpAgentSelectionActionPlan {
    fn from_is_selected(is_selected: bool) -> Self {
        if is_selected {
            Self::CurrentAgent
        } else {
            Self::AvailableAgent
        }
    }

    fn action_title(self, display_name: &str) -> String {
        match self {
            Self::CurrentAgent => format!("Current Agent: {display_name}"),
            Self::AvailableAgent => format!("Use {display_name}"),
        }
    }

    fn picker_title(self, display_name: &str) -> String {
        match self {
            Self::CurrentAgent => format!("{display_name} \u{2713}"),
            Self::AvailableAgent => display_name.to_string(),
        }
    }

    fn description(self, entry: &crate::ai::acp::AcpAgentCatalogEntry) -> String {
        let source = acp_agent_source_label(entry.source);
        let state = acp_agent_state_label(entry);

        match self {
            Self::CurrentAgent => format!("Currently selected agent. {source} · {state}"),
            Self::AvailableAgent => {
                format!("Reconnect Agent Chat using this agent. {source} · {state}")
            }
        }
    }
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_agent_switch_description(
    entry: &crate::ai::acp::AcpAgentCatalogEntry,
    is_selected: bool,
) -> String {
    AcpAgentSelectionActionPlan::from_is_selected(is_selected).description(entry)
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_switch_agent_action_id(agent_id: &str) -> String {
    format!("{ACP_SWITCH_AGENT_ACTION_PREFIX}{agent_id}")
}

#[allow(dead_code)] // Used by ACP chat action dispatch in the binary target.
pub(crate) fn acp_switch_agent_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(ACP_SWITCH_AGENT_ACTION_PREFIX)
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_switch_model_action_id(model_id: &str) -> String {
    format!("{ACP_SWITCH_MODEL_ACTION_PREFIX}{model_id}")
}

#[allow(dead_code)] // Used by ACP chat action dispatch in the binary target.
pub(crate) fn acp_switch_model_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(ACP_SWITCH_MODEL_ACTION_PREFIX)
}

const ACP_SWITCH_PROFILE_ACTION_PREFIX: &str = "acp_switch_profile:";

fn acp_switch_profile_action_id(profile_name: &str) -> String {
    format!("{ACP_SWITCH_PROFILE_ACTION_PREFIX}{profile_name}")
}

#[allow(dead_code)] // Used by ACP chat action dispatch in the binary target.
pub(crate) fn acp_switch_profile_name_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(ACP_SWITCH_PROFILE_ACTION_PREFIX)
}

fn acp_profile_actions(ai_preferences: &crate::config::AiPreferences) -> Vec<Action> {
    ai_preferences
        .profiles
        .iter()
        .filter(|profile| !profile.name.trim().is_empty())
        .map(|profile| {
            Action::new(
                acp_switch_profile_action_id(&profile.name),
                format!("Switch profile: {}", profile.name),
                Some("Apply this agent profile".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Settings)
            .with_section("Profile")
        })
        .collect()
}

/// Actions available in the ACP chat view (Cmd+K menu).
#[allow(dead_code)]
pub fn get_acp_chat_actions() -> Vec<Action> {
    let ai_preferences = crate::config::load_user_preferences().ai;
    let mut actions = vec![
        // ── Response ─────────────────────────────────────────
        Action::new(
            "acp_copy_last_response",
            "Copy Last Response",
            Some("Copy the most recent assistant response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{21e7}\u{2318}C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        Action::new(
            "acp_paste_to_frontmost",
            "Paste Response to App",
            Some("Paste into the frontmost application".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowRight)
        .with_section("Response"),
        Action::new(
            "acp_retry_last",
            "Retry Last Message",
            Some("Resend the last user message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowRight)
        .with_section("Response"),
        Action::new(
            "acp_export_markdown",
            "Copy Conversation as Markdown",
            Some("Copy the full conversation as markdown".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Response"),
        Action::new(
            "acp_save_as_note",
            "Save as Note",
            Some("Create or update a note from the current ACP content".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{21e7}\u{2318}S")
        .with_icon(IconName::File)
        .with_section("Response"),
        // ── Code ─────────────────────────────────────────────
        Action::new(
            "acp_copy_all_code",
            "Copy All Code Blocks",
            Some("Copy all code blocks to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Code)
        .with_section("Code"),
        Action::new(
            "acp_save_as_script",
            "Save as Script",
            Some("Save last code block as a Script Kit script".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Code"),
        Action::new(
            "acp_run_last_code",
            "Run Last Code Block",
            Some("Execute the last code block and show output".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::BoltFilled)
        .with_section("Code"),
        Action::new(
            "acp_open_in_editor",
            "Open in Editor",
            Some("Open ~/.scriptkit in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Code)
        .with_section("Code"),
        // ── Navigate ─────────────────────────────────────────
        Action::new(
            "acp_scroll_to_top",
            "Scroll to Top",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowUp)
        .with_section("Navigate"),
        Action::new(
            "acp_scroll_to_bottom",
            "Scroll to Latest",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowDown)
        .with_section("Navigate"),
        Action::new(
            "acp_show_history",
            "Agent Chat History",
            Some("Browse and manage past Agent Chat conversations".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}P")
        .with_icon(IconName::MagnifyingGlass)
        .with_section("Navigate"),
        // ── View ─────────────────────────────────────────────
        Action::new(
            "acp_expand_all",
            "Expand All Blocks",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ChevronDown)
        .with_section("View"),
        Action::new(
            "acp_collapse_all",
            "Collapse All Blocks",
            None,
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ChevronRight)
        .with_section("View"),
        // ── Session ──────────────────────────────────────────
        Action::new(
            "acp_new_conversation",
            "New Conversation",
            Some("Clear messages, keep session".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}N")
        .with_icon(IconName::Plus)
        .with_section("Session"),
        Action::new(
            "acp_clear_conversation",
            "Clear & Restart",
            Some("Close and reopen a fresh session".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Trash)
        .with_section("Session"),
        Action::new(
            "acp_clear_history",
            "Clear History",
            Some("Delete all saved conversations".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Trash)
        .with_section("Session"),
        // ── Window ───────────────────────────────────────────
        Action::new(
            "acp_detach_window",
            "Keep Open in Window",
            Some("Keep this chat open in a separate window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowUp)
        .with_section("Window"),
        Action::new(
            "acp_reattach_panel",
            "Return to Panel",
            Some("Move this chat back to the main panel".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowDown)
        .with_section("Window"),
        Action::new(
            "acp_close",
            "Close Agent Chat",
            None,
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}W")
        .with_icon(IconName::Close)
        .with_section("Window"),
    ];

    actions.extend(acp_profile_actions(&ai_preferences));
    actions
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
pub(crate) fn get_acp_chat_actions_with_agents(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
) -> Vec<Action> {
    let mut actions = get_acp_chat_actions();

    if catalog_entries.is_empty() {
        return actions;
    }

    let selected_agent_id = selected_agent_id.filter(|id| !id.is_empty());
    actions.extend(catalog_entries.iter().map(|entry| {
        let is_selected = selected_agent_id == Some(entry.id.as_ref());
        let selection_plan = AcpAgentSelectionActionPlan::from_is_selected(is_selected);

        Action::new(
            acp_switch_agent_action_id(entry.id.as_ref()),
            selection_plan.action_title(&entry.display_name),
            Some(selection_plan.description(entry)),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Terminal)
        .with_section("Agent")
    }));

    actions
}

// ── ACP route builders ───────────────────────────────────────────────────────

/// Action ID for the root-level "Change Agent" entry that pushes the agent picker.
pub const ACP_CHANGE_AGENT_ACTION_ID: &str = "acp:change_agent";
/// Action ID for the root-level "Change Model" entry that pushes the model picker.
pub const ACP_CHANGE_MODEL_ACTION_ID: &str = "acp:change_model";

/// Stable root labels and descriptions for ACP Actions Menu parity across hosts.
const ACP_CHANGE_AGENT_LABEL: &str = "Change Agent";
const ACP_CHANGE_AGENT_DESCRIPTION: &str = "Pick the agent for this chat";
const ACP_CHANGE_MODEL_LABEL: &str = "Change Model";
const ACP_CHANGE_MODEL_DESCRIPTION: &str = "Pick the model for this chat";
/// Route ID for the ACP root actions menu.
pub const ACP_ROOT_ROUTE_ID: &str = "acp:root";
/// Route ID for the agent picker sub-route.
pub const ACP_AGENT_PICKER_ROUTE_ID: &str = "acp:agent_picker";
/// Route ID for the model picker sub-route.
pub const ACP_MODEL_PICKER_ROUTE_ID: &str = "acp:model_picker";

fn acp_model_display_name(entry: &crate::ai::acp::config::AcpModelEntry) -> String {
    entry
        .display_name
        .clone()
        .unwrap_or_else(|| entry.id.clone())
}

fn acp_model_switch_description(
    entry: &crate::ai::acp::config::AcpModelEntry,
    is_selected: bool,
) -> String {
    let display_name = acp_model_display_name(entry);
    if is_selected {
        format!("Currently selected model: {display_name}")
    } else {
        format!("Switch Agent Chat to {display_name}")
    }
}

/// Build the root-level ACP actions list. Includes a single "Change Agent"
/// entry (which triggers drill-down) plus the standard ACP chat actions.
pub(crate) fn get_acp_chat_root_actions(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
    available_models: &[crate::ai::acp::config::AcpModelEntry],
    selected_model_id: Option<&str>,
) -> Vec<Action> {
    let selected_agent =
        selected_agent_id.and_then(|id| catalog_entries.iter().find(|e| e.id.as_ref() == id));
    let selected_model =
        selected_model_id.and_then(|id| available_models.iter().find(|entry| entry.id == id));

    let mut actions = vec![Action::new(
        ACP_CHANGE_AGENT_ACTION_ID,
        ACP_CHANGE_AGENT_LABEL,
        Some(
            selected_agent
                .map(|e| format!("Current: {}", e.display_name))
                .unwrap_or_else(|| ACP_CHANGE_AGENT_DESCRIPTION.to_string()),
        ),
        ActionCategory::ScriptContext,
    )
    .with_icon(IconName::Terminal)
    .with_section("Agent")];

    if !available_models.is_empty() {
        actions.push(
            Action::new(
                ACP_CHANGE_MODEL_ACTION_ID,
                ACP_CHANGE_MODEL_LABEL,
                Some(
                    selected_model
                        .map(|entry| format!("Current: {}", acp_model_display_name(entry)))
                        .unwrap_or_else(|| ACP_CHANGE_MODEL_DESCRIPTION.to_string()),
                ),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Settings)
            .with_section("Agent"),
        );
    }

    actions.extend(get_acp_chat_actions());
    actions
}

/// Build the second-level agent picker actions. Preserves the existing
/// `acp_switch_agent:<id>` action IDs so `handle_acp_chat_action` keeps working.
pub(crate) fn get_acp_agent_picker_actions(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
) -> Vec<Action> {
    let selected_agent_id = selected_agent_id.filter(|id| !id.is_empty());
    catalog_entries
        .iter()
        .map(|entry| {
            let is_selected = selected_agent_id == Some(entry.id.as_ref());
            let selection_plan = AcpAgentSelectionActionPlan::from_is_selected(is_selected);
            Action::new(
                acp_switch_agent_action_id(entry.id.as_ref()),
                selection_plan.picker_title(&entry.display_name),
                Some(selection_plan.description(entry)),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Terminal)
        })
        .collect()
}

/// Build the second-level ACP model picker actions.
pub(crate) fn get_acp_model_picker_actions(
    available_models: &[crate::ai::acp::config::AcpModelEntry],
    selected_model_id: Option<&str>,
) -> Vec<Action> {
    let selected_model_id = selected_model_id.filter(|id| !id.is_empty());
    available_models
        .iter()
        .map(|entry| {
            let is_selected = selected_model_id == Some(entry.id.as_str());
            let display_name = acp_model_display_name(entry);
            Action::new(
                acp_switch_model_action_id(&entry.id),
                if is_selected {
                    format!("{display_name} \u{2713}")
                } else {
                    display_name
                },
                Some(acp_model_switch_description(entry, is_selected)),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Settings)
        })
        .collect()
}

// ── Host-aware ACP action filtering ─────────────────────────────────────────

/// Distinguishes whether the ACP actions dialog is hosted in the shared main
/// panel or in the detached ACP chat window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpActionsDialogHost {
    /// Shared ACP surface in the main Script Kit panel — all actions available.
    Shared,
    /// Notes-hosted ACP surface — subset that works inside the Notes window.
    /// `acp_close` returns to the Notes editor rather than closing a window.
    /// `acp_save_as_note` is excluded because the user is already in Notes.
    Notes,
    /// Detached ACP chat window — only actions that work without the main panel.
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcpHostActionPlan {
    IncludeWithShortcut,
    IncludeWithoutShortcut,
    Exclude,
}

impl AcpHostActionPlan {
    fn is_included(self) -> bool {
        !matches!(self, Self::Exclude)
    }

    fn keeps_shortcut(self) -> bool {
        matches!(self, Self::IncludeWithShortcut)
    }
}

fn acp_host_action_plan(host: AcpActionsDialogHost, action_id: &str) -> AcpHostActionPlan {
    match host {
        AcpActionsDialogHost::Shared => {
            if action_id == "acp_close" {
                AcpHostActionPlan::IncludeWithoutShortcut
            } else {
                AcpHostActionPlan::IncludeWithShortcut
            }
        }
        AcpActionsDialogHost::Notes => {
            // Notes-hosted: same as Detached but without `acp_save_as_note`
            // (already in Notes), keeping `acp_close` (returns to editor),
            // and opening `acp_show_history` as a Notes-anchored popup.
            if matches!(
                action_id,
                "acp:change_agent"
                    | "acp:change_model"
                    | "acp_copy_last_response"
                    | "acp_retry_last"
                    | "acp_export_markdown"
                    | "acp_show_history"
                    | "acp_scroll_to_top"
                    | "acp_scroll_to_bottom"
                    | "acp_expand_all"
                    | "acp_collapse_all"
                    | "acp_new_conversation"
                    | "acp_clear_history"
                    | "acp_close"
            ) || action_id.starts_with(ACP_SWITCH_AGENT_ACTION_PREFIX)
                || action_id.starts_with(ACP_SWITCH_MODEL_ACTION_PREFIX)
            {
                if action_id == "acp_close" {
                    AcpHostActionPlan::IncludeWithoutShortcut
                } else {
                    AcpHostActionPlan::IncludeWithShortcut
                }
            } else {
                AcpHostActionPlan::Exclude
            }
        }
        AcpActionsDialogHost::Detached => {
            if matches!(
                action_id,
                "acp:change_agent"
                    | "acp:change_model"
                    | "acp_copy_last_response"
                    | "acp_retry_last"
                    | "acp_export_markdown"
                    | "acp_save_as_note"
                    | "acp_scroll_to_top"
                    | "acp_scroll_to_bottom"
                    | "acp_expand_all"
                    | "acp_collapse_all"
                    | "acp_new_conversation"
                    | "acp_clear_history"
                    | "acp_close"
            ) || action_id.starts_with(ACP_SWITCH_AGENT_ACTION_PREFIX)
                || action_id.starts_with(ACP_SWITCH_MODEL_ACTION_PREFIX)
            {
                AcpHostActionPlan::IncludeWithShortcut
            } else {
                AcpHostActionPlan::Exclude
            }
        }
    }
}

fn filter_acp_actions_for_host(host: AcpActionsDialogHost, actions: Vec<Action>) -> Vec<Action> {
    let host_label = match host {
        AcpActionsDialogHost::Shared => "shared",
        AcpActionsDialogHost::Notes => "notes",
        AcpActionsDialogHost::Detached => "detached",
    };
    actions
        .into_iter()
        .filter_map(|mut action| {
            let plan = acp_host_action_plan(host, &action.id);
            if !plan.is_included() {
                tracing::warn!(
                    event = "acp_actions_menu_filtered",
                    host = host_label,
                    action_id = %action.id,
                    reason = "unsupported_in_host",
                    "Filtered unsupported ACP Actions Menu item"
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

/// Build an `ActionsDialogRoute` for the ACP root menu, filtered for the given host.
pub(crate) fn get_acp_chat_root_route_for_host(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
    available_models: &[crate::ai::acp::config::AcpModelEntry],
    selected_model_id: Option<&str>,
    host: AcpActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    let host_label = match host {
        AcpActionsDialogHost::Shared => "shared",
        AcpActionsDialogHost::Notes => "notes",
        AcpActionsDialogHost::Detached => "detached",
    };
    let context_title = selected_agent_id
        .and_then(|id| catalog_entries.iter().find(|e| e.id.as_ref() == id))
        .map(|e| e.display_name.to_string())
        .or_else(|| Some("Agent Chat".to_string()));

    let actions = filter_acp_actions_for_host(
        host,
        get_acp_chat_root_actions(
            catalog_entries,
            selected_agent_id,
            available_models,
            selected_model_id,
        ),
    );

    let agent_count = catalog_entries.len();
    let model_count = available_models.len();
    tracing::info!(
        event = "acp_actions_menu_built",
        host = host_label,
        agent_count,
        model_count,
        action_count = actions.len(),
        "Built ACP Actions Menu"
    );

    crate::actions::ActionsDialogRoute {
        id: ACP_ROOT_ROUTE_ID.to_string(),
        actions,
        context_title,
        search_placeholder: Some("Search Agent Chat actions...".to_string()),
        initial_selected_action_id: Some(ACP_CHANGE_AGENT_ACTION_ID.to_string()),
    }
}

/// Build an `ActionsDialogRoute` for the agent picker sub-route, filtered for the given host.
pub(crate) fn get_acp_agent_picker_route_for_host(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
    host: AcpActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: ACP_AGENT_PICKER_ROUTE_ID.to_string(),
        actions: filter_acp_actions_for_host(
            host,
            get_acp_agent_picker_actions(catalog_entries, selected_agent_id),
        ),
        context_title: Some("Change Agent".to_string()),
        search_placeholder: Some("Search agents...".to_string()),
        initial_selected_action_id: selected_agent_id.map(acp_switch_agent_action_id),
    }
}

/// Build an `ActionsDialogRoute` for the ACP model picker sub-route, filtered for the given host.
pub(crate) fn get_acp_model_picker_route_for_host(
    available_models: &[crate::ai::acp::config::AcpModelEntry],
    selected_model_id: Option<&str>,
    host: AcpActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: ACP_MODEL_PICKER_ROUTE_ID.to_string(),
        actions: filter_acp_actions_for_host(
            host,
            get_acp_model_picker_actions(available_models, selected_model_id),
        ),
        context_title: Some("Change Model".to_string()),
        search_placeholder: Some("Search models...".to_string()),
        initial_selected_action_id: selected_model_id.map(acp_switch_model_action_id),
    }
}

/// Build an `ActionsDialogRoute` for the ACP root menu (shared host).
#[allow(dead_code)]
pub(crate) fn get_acp_chat_root_route(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
    available_models: &[crate::ai::acp::config::AcpModelEntry],
    selected_model_id: Option<&str>,
) -> crate::actions::ActionsDialogRoute {
    get_acp_chat_root_route_for_host(
        catalog_entries,
        selected_agent_id,
        available_models,
        selected_model_id,
        AcpActionsDialogHost::Shared,
    )
}

/// Build an `ActionsDialogRoute` for the agent picker sub-route (shared host).
#[allow(dead_code)]
pub(crate) fn get_acp_agent_picker_route(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
) -> crate::actions::ActionsDialogRoute {
    get_acp_agent_picker_route_for_host(
        catalog_entries,
        selected_agent_id,
        AcpActionsDialogHost::Shared,
    )
}

/// Build an `ActionsDialogRoute` for the ACP model picker sub-route (shared host).
#[allow(dead_code)]
pub(crate) fn get_acp_model_picker_route(
    available_models: &[crate::ai::acp::config::AcpModelEntry],
    selected_model_id: Option<&str>,
) -> crate::actions::ActionsDialogRoute {
    get_acp_model_picker_route_for_host(
        available_models,
        selected_model_id,
        AcpActionsDialogHost::Shared,
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

    fn sample_acp_agent(
        id: &str,
        display_name: &str,
        source: crate::ai::acp::AcpAgentSource,
        install_state: crate::ai::acp::AcpAgentInstallState,
        auth_state: crate::ai::acp::AcpAgentAuthState,
        config_state: crate::ai::acp::AcpAgentConfigState,
    ) -> crate::ai::acp::AcpAgentCatalogEntry {
        crate::ai::acp::AcpAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: display_name.to_string().into(),
            source,
            install_state,
            auth_state,
            config_state,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        }
    }

    fn sample_acp_model(id: &str, display_name: &str) -> crate::ai::acp::config::AcpModelEntry {
        crate::ai::acp::config::AcpModelEntry {
            id: id.to_string(),
            display_name: Some(display_name.to_string()),
            context_window: None,
        }
    }

    #[test]
    fn test_acp_close_shortcut_is_only_advertised_for_detached_host() {
        let shared =
            get_acp_chat_root_route_for_host(&[], None, &[], None, AcpActionsDialogHost::Shared);
        let notes =
            get_acp_chat_root_route_for_host(&[], None, &[], None, AcpActionsDialogHost::Notes);
        let detached =
            get_acp_chat_root_route_for_host(&[], None, &[], None, AcpActionsDialogHost::Detached);

        let shared_close = shared
            .actions
            .iter()
            .find(|action| action.id == "acp_close")
            .expect("shared acp_close action should exist");
        let notes_close = notes
            .actions
            .iter()
            .find(|action| action.id == "acp_close")
            .expect("notes acp_close action should exist");
        let detached_close = detached
            .actions
            .iter()
            .find(|action| action.id == "acp_close")
            .expect("detached acp_close action should exist");

        assert!(shared_close.shortcut.is_none());
        assert!(notes_close.shortcut.is_none());
        assert_eq!(detached_close.shortcut.as_deref(), Some("⌘W"));
    }

    #[test]
    fn test_acp_host_action_plan_matrix() {
        for host in [AcpActionsDialogHost::Shared, AcpActionsDialogHost::Notes] {
            assert_eq!(
                acp_host_action_plan(host, "acp_close"),
                AcpHostActionPlan::IncludeWithoutShortcut,
                "{host:?} should keep close available without advertising Cmd-W"
            );
        }

        assert_eq!(
            acp_host_action_plan(AcpActionsDialogHost::Detached, "acp_close"),
            AcpHostActionPlan::IncludeWithShortcut
        );
        assert_eq!(
            acp_host_action_plan(AcpActionsDialogHost::Notes, "acp_save_as_note"),
            AcpHostActionPlan::Exclude
        );
        assert_eq!(
            acp_host_action_plan(AcpActionsDialogHost::Detached, "acp_show_history"),
            AcpHostActionPlan::Exclude
        );

        for host in [
            AcpActionsDialogHost::Shared,
            AcpActionsDialogHost::Notes,
            AcpActionsDialogHost::Detached,
        ] {
            assert_eq!(
                acp_host_action_plan(host, "acp_switch_agent:codex"),
                AcpHostActionPlan::IncludeWithShortcut
            );
            assert_eq!(
                acp_host_action_plan(host, "acp_switch_model:gpt"),
                AcpHostActionPlan::IncludeWithShortcut
            );
        }
    }

    #[test]
    fn test_acp_chat_actions_with_agents_adds_agent_section_entries() {
        let actions = get_acp_chat_actions_with_agents(
            &[
                sample_acp_agent(
                    "codex-acp",
                    "Codex (ACP)",
                    crate::ai::acp::AcpAgentSource::BuiltIn,
                    crate::ai::acp::AcpAgentInstallState::Ready,
                    crate::ai::acp::AcpAgentAuthState::Unknown,
                    crate::ai::acp::AcpAgentConfigState::Valid,
                ),
                sample_acp_agent(
                    "opencode",
                    "OpenCode",
                    crate::ai::acp::AcpAgentSource::BuiltIn,
                    crate::ai::acp::AcpAgentInstallState::NeedsInstall,
                    crate::ai::acp::AcpAgentAuthState::Unknown,
                    crate::ai::acp::AcpAgentConfigState::Valid,
                ),
            ],
            Some("codex-acp"),
        );

        let current = actions
            .iter()
            .find(|action| action.id == "acp_switch_agent:codex-acp")
            .expect("current agent action should exist");
        assert_eq!(current.title, "Current Agent: Codex (ACP)");
        assert_eq!(current.section.as_deref(), Some("Agent"));
        assert!(
            current
                .description
                .as_deref()
                .is_some_and(|description| description.contains("Currently selected agent")),
            "current agent description should explain selection state"
        );

        let alternate = actions
            .iter()
            .find(|action| action.id == "acp_switch_agent:opencode")
            .expect("alternate agent action should exist");
        assert_eq!(alternate.title, "Use OpenCode");
        assert!(
            alternate
                .description
                .as_deref()
                .is_some_and(|description| description.contains("Needs install")),
            "alternate agent description should include readiness state"
        );
    }

    #[test]
    fn test_acp_chat_root_actions_add_change_model_when_models_exist() {
        let actions = get_acp_chat_root_actions(
            &[sample_acp_agent(
                "codex-acp",
                "Codex (ACP)",
                crate::ai::acp::AcpAgentSource::BuiltIn,
                crate::ai::acp::AcpAgentInstallState::Ready,
                crate::ai::acp::AcpAgentAuthState::Unknown,
                crate::ai::acp::AcpAgentConfigState::Valid,
            )],
            Some("codex-acp"),
            &[
                sample_acp_model("claude-sonnet-4-6", "Sonnet 4.6"),
                sample_acp_model("claude-opus-4-6", "Opus 4.6"),
            ],
            Some("claude-sonnet-4-6"),
        );

        let change_model = actions
            .iter()
            .find(|action| action.id == ACP_CHANGE_MODEL_ACTION_ID)
            .expect("change model action should exist");
        assert_eq!(change_model.section.as_deref(), Some("Agent"));
        assert_eq!(
            change_model.description.as_deref(),
            Some("Current: Sonnet 4.6")
        );
    }

    #[test]
    fn test_acp_model_picker_actions_mark_selected_model() {
        let actions = get_acp_model_picker_actions(
            &[
                sample_acp_model("claude-sonnet-4-6", "Sonnet 4.6"),
                sample_acp_model("claude-opus-4-6", "Opus 4.6"),
            ],
            Some("claude-opus-4-6"),
        );

        let current = actions
            .iter()
            .find(|action| action.id == "acp_switch_model:claude-opus-4-6")
            .expect("current model action should exist");
        assert_eq!(current.title, "Opus 4.6 ✓");

        let alternate = actions
            .iter()
            .find(|action| action.id == "acp_switch_model:claude-sonnet-4-6")
            .expect("alternate model action should exist");
        assert_eq!(alternate.title, "Sonnet 4.6");
    }

    #[test]
    fn test_acp_switch_agent_action_parser_returns_agent_id() {
        assert_eq!(
            acp_switch_agent_id_from_action("acp_switch_agent:codex-acp"),
            Some("codex-acp")
        );
        assert_eq!(acp_switch_agent_id_from_action("acp_retry_last"), None);
    }

    #[test]
    fn test_acp_switch_model_action_parser_returns_model_id() {
        assert_eq!(
            acp_switch_model_id_from_action("acp_switch_model:claude-sonnet-4-6"),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(acp_switch_model_id_from_action("acp_retry_last"), None);
    }
}
