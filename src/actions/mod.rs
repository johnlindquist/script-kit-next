//! Actions Dialog Module
//!
//! The source of every action: what can be done to a script, a file, a clipboard entry, a
//! prompt's value. ⌘K stages the focused thing in the main window and lists these actions in
//! place under the main input (`components::staged_actions`); nothing opens.
//!
//! ## Module Structure
//! - `types`: Core types (Action, ActionCategory, ScriptInfo)
//! - `builders`: Factory functions for creating action lists
//! - `constants`: Popup dimensions and styling constants
//! - `dialog`: ActionsDialog struct and implementation
//! - `in_place`: what the main window has staged, for surfaces that don't own it

mod builders;
mod command_bar;
pub(crate) mod constants;
mod dialog;
mod in_place;
mod types;

// Re-export only the public API that is actually used externally:
// - ScriptInfo: used by main.rs for action context
// - ActionsDialog: the main dialog component

#[allow(unused_imports)]
// The binary's day page switcher uses this; the library has no caller.
pub(crate) use builders::get_canonical_note_search_actions;
#[allow(unused_imports)] // used by binary target via include!() in main.rs
pub(crate) use builders::resolve_file_search_secondary_action_id;
pub(crate) use builders::AgentChatActionsDialogHost;
#[allow(unused_imports)]
pub(crate) use builders::{
    agent_chat_fork_edit_entry_from_action, agent_chat_receipt_history_request_id_from_action,
    agent_chat_switch_model_id_from_action, agent_chat_switch_profile_id_from_action,
    agent_chat_switch_thread_id_from_action, get_agent_chat_actions, get_agent_chat_history_route,
    get_agent_chat_model_picker_route, get_agent_chat_receipt_history_route,
    get_agent_chat_root_route_for_host, get_global_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, AGENT_CHAT_HISTORY_SELECT_ACTION_PREFIX,
    AGENT_CHAT_RECEIPT_HISTORY_COPY_ACTION_PREFIX, AGENT_CHAT_RECEIPT_HISTORY_ROUTE_ID,
    AGENT_CHAT_SHOW_RECEIPT_HISTORY_ACTION_ID,
};
#[allow(unused_imports)]
// Used by actions builder tests; the legacy AI window consumer is gone.
pub use builders::{
    get_ai_command_bar_actions, get_day_note_switcher_actions, get_new_chat_actions,
    get_note_switcher_actions, get_notes_command_bar_actions,
};
#[allow(unused_imports)]
// Used by actions builder tests; the legacy AI window consumer is gone.
pub use builders::{
    to_deeplink_name, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo, FileSearchDirectoryInfo,
    FileSearchSortMode, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
pub use command_bar::{CommandBar, CommandBarConfig};
#[allow(unused_imports)] // Used by the binary target through include!()-ed app_impl code.
pub(crate) use dialog::matching_filtered_action_id_for_keystroke;
#[allow(unused_imports)] // Used by binary target through include!()-ed app_impl code.
pub(crate) use dialog::ActionsHostContextSnapshot;
#[allow(unused_imports)] // The binary's app implementation owns activation callback creation.
pub(crate) use dialog::ActivationCallback;
pub(crate) use dialog::AgentChatActionsDialogContext;
pub(crate) use dialog::GroupedActionItem;
#[allow(unused_imports)] // Used by binary target through include!()-ed render/app_impl code.
pub(crate) use dialog::{
    displayed_action_keybinding_specs, matching_action_id_for_canonical_shortcut,
    MainListDisplayedActionShortcut,
};
#[allow(unused_imports)]
// Used by the binary target through include!()-ed prompt_handler code.
pub(crate) use dialog::{is_destructive_action, matching_action_id_for_keystroke};
pub use dialog::{
    ActionsDialog, ActionsDialogActivation, ActionsDialogEscapeOutcome, ActionsDialogRoute,
};
pub use types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};

pub(crate) use in_place::set_actions_dialog_search_text;
#[allow(unused_imports)] // The binary's ScriptListApp stages and lets go.
pub(crate) use in_place::{clear_in_place, stage_in_place, LetGo, RouteKey};
pub use in_place::{
    close_actions_window, is_actions_window_open, route_key_to_detached_actions_window,
};

// Design-contract resolvers: shared between the dialog renderer and the token
// exporter (src/design_contract) so the two can never drift.
#[allow(unused_imports)]
pub(crate) use dialog::{
    resolved_actions_dialog_row_chrome, resolved_actions_dialog_row_metrics,
    resolved_actions_dialog_search_chrome, resolved_actions_dialog_section_chrome,
    resolved_actions_popup_height, ResolvedActionsDialogRowChrome,
    ResolvedActionsDialogSearchChrome, ResolvedActionsDialogSectionChrome,
};

pub mod prelude;

#[cfg(test)]
// Unit-test module wiring lives in src/actions/tests.rs and includes
// the canonical builtin dialog validation suite from
// src/actions/tests/dialog_builtin_validation/mod.rs.
include!("tests.rs");
