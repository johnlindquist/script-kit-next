use super::shared::to_deeplink_name;
use super::types::{Action, ActionCategory, ScriptInfo};
use crate::designs::icon_variations::IconName;
use itertools::Itertools;
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

    if script.shortcut.is_some() {
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
    } else {
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

    if script.alias.is_some() {
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
    } else {
        actions.push(
            Action::new(
                "add_alias",
                "Add Alias",
                Some("Set an alias trigger for this item (type alias + space to run)".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A")
            .with_icon(IconName::Settings)
            .with_section("Edit"),
        );
    }

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
        .with_icon(IconName::Copy)
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
            .with_icon(IconName::Trash)
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

#[allow(dead_code)] // Used by the binary ACP actions surface.
const ACP_SWITCH_AGENT_ACTION_PREFIX: &str = "acp_switch_agent:";

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

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_agent_switch_description(
    entry: &crate::ai::acp::AcpAgentCatalogEntry,
    is_selected: bool,
) -> String {
    let source = acp_agent_source_label(entry.source);
    let state = acp_agent_state_label(entry);

    if is_selected {
        format!("Currently selected ACP agent. {source} · {state}")
    } else {
        format!("Reconnect ACP chat using this agent. {source} · {state}")
    }
}

#[allow(dead_code)] // Used by the binary ACP actions surface.
fn acp_switch_agent_action_id(agent_id: &str) -> String {
    format!("{ACP_SWITCH_AGENT_ACTION_PREFIX}{agent_id}")
}

#[allow(dead_code)] // Used by ACP chat action dispatch in the binary target.
pub(crate) fn acp_switch_agent_id_from_action(action_id: &str) -> Option<&str> {
    action_id.strip_prefix(ACP_SWITCH_AGENT_ACTION_PREFIX)
}

/// Actions available in the ACP chat view (Cmd+K menu).
#[allow(dead_code)]
pub fn get_acp_chat_actions() -> Vec<Action> {
    vec![
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
            "Export as Markdown",
            Some("Copy the full conversation as markdown".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::FileCode)
        .with_section("Response"),
        Action::new(
            "acp_save_as_note",
            "Save as Note",
            Some("Save the conversation to Notes".to_string()),
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
            "Conversation History",
            Some("Browse and manage past conversations".to_string()),
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
            "Detach to Window",
            Some("Open in a separate floating window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowUp)
        .with_section("Window"),
        Action::new(
            "acp_reattach_panel",
            "Re-attach to Panel",
            Some("Move back to the main panel".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowDown)
        .with_section("Window"),
        Action::new(
            "acp_close",
            "Close AI Chat",
            None,
            ActionCategory::ScriptContext,
        )
        .with_shortcut("\u{2318}W")
        .with_icon(IconName::Close)
        .with_section("Window"),
    ]
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
        let title = if is_selected {
            format!("Current Agent: {}", entry.display_name)
        } else {
            format!("Use {}", entry.display_name)
        };

        Action::new(
            acp_switch_agent_action_id(entry.id.as_ref()),
            title,
            Some(acp_agent_switch_description(entry, is_selected)),
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
/// Route ID for the ACP root actions menu.
pub const ACP_ROOT_ROUTE_ID: &str = "acp:root";
/// Route ID for the agent picker sub-route.
pub const ACP_AGENT_PICKER_ROUTE_ID: &str = "acp:agent_picker";

/// Build the root-level ACP actions list. Includes a single "Change Agent"
/// entry (which triggers drill-down) plus the standard ACP chat actions.
pub(crate) fn get_acp_chat_root_actions(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
) -> Vec<Action> {
    let selected_agent =
        selected_agent_id.and_then(|id| catalog_entries.iter().find(|e| e.id.as_ref() == id));

    let mut actions = vec![Action::new(
        ACP_CHANGE_AGENT_ACTION_ID,
        "Change Agent",
        Some(
            selected_agent
                .map(|e| format!("Current: {}", e.display_name))
                .unwrap_or_else(|| "Choose the ACP agent for this chat".to_string()),
        ),
        ActionCategory::ScriptContext,
    )
    .with_icon(IconName::Terminal)
    .with_section("Agent")];

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
            Action::new(
                acp_switch_agent_action_id(entry.id.as_ref()),
                if is_selected {
                    format!("{} \u{2713}", entry.display_name)
                } else {
                    entry.display_name.to_string()
                },
                Some(acp_agent_switch_description(entry, is_selected)),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Terminal)
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
    /// Detached ACP chat window — only actions that work without the main panel.
    Detached,
}

fn acp_action_supported_in_host(host: AcpActionsDialogHost, action_id: &str) -> bool {
    match host {
        AcpActionsDialogHost::Shared => true,
        AcpActionsDialogHost::Detached => {
            matches!(
                action_id,
                "acp:change_agent"
                    | "acp_copy_last_response"
                    | "acp_retry_last"
                    | "acp_export_markdown"
                    | "acp_scroll_to_top"
                    | "acp_scroll_to_bottom"
                    | "acp_expand_all"
                    | "acp_collapse_all"
                    | "acp_new_conversation"
                    | "acp_clear_history"
                    | "acp_close"
            ) || action_id.starts_with(ACP_SWITCH_AGENT_ACTION_PREFIX)
        }
    }
}

fn filter_acp_actions_for_host(host: AcpActionsDialogHost, actions: Vec<Action>) -> Vec<Action> {
    actions
        .into_iter()
        .filter(|action| acp_action_supported_in_host(host, &action.id))
        .collect()
}

/// Build an `ActionsDialogRoute` for the ACP root menu, filtered for the given host.
pub(crate) fn get_acp_chat_root_route_for_host(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
    host: AcpActionsDialogHost,
) -> crate::actions::ActionsDialogRoute {
    let context_title = selected_agent_id
        .and_then(|id| catalog_entries.iter().find(|e| e.id.as_ref() == id))
        .map(|e| e.display_name.to_string())
        .or_else(|| Some("AI Chat".to_string()));

    crate::actions::ActionsDialogRoute {
        id: ACP_ROOT_ROUTE_ID.to_string(),
        actions: filter_acp_actions_for_host(
            host,
            get_acp_chat_root_actions(catalog_entries, selected_agent_id),
        ),
        context_title,
        search_placeholder: Some("Search ACP actions...".to_string()),
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

/// Build an `ActionsDialogRoute` for the ACP root menu (shared host).
#[allow(dead_code)]
pub(crate) fn get_acp_chat_root_route(
    catalog_entries: &[crate::ai::acp::AcpAgentCatalogEntry],
    selected_agent_id: Option<&str>,
) -> crate::actions::ActionsDialogRoute {
    get_acp_chat_root_route_for_host(
        catalog_entries,
        selected_agent_id,
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
                .is_some_and(|description| description.contains("Currently selected ACP agent")),
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
    fn test_acp_switch_agent_action_parser_returns_agent_id() {
        assert_eq!(
            acp_switch_agent_id_from_action("acp_switch_agent:codex-acp"),
            Some("codex-acp")
        );
        assert_eq!(acp_switch_agent_id_from_action("acp_retry_last"), None);
    }
}
