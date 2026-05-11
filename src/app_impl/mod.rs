#[path = "startup.rs"]
mod startup;
#[path = "theme_focus.rs"]
mod theme_focus;
#[path = "refresh_scriptlets.rs"]
mod refresh_scriptlets;
#[path = "filtering_cache.rs"]
mod filtering_cache;
#[path = "selection_fallback.rs"]
mod selection_fallback;
#[path = "filter_input_core.rs"]
mod filter_input_core;
#[path = "filter_input_change.rs"]
mod filter_input_change;
#[path = "filter_input_updates.rs"]
mod filter_input_updates;
#[path = "root_file_search.rs"]
mod root_file_search;
#[path = "root_unified_result_actions.rs"]
pub(crate) mod root_unified_result_actions;
#[path = "menu_syntax_trigger_popup.rs"]
mod menu_syntax_trigger_popup;
#[path = "menu_syntax_trigger_popup_window.rs"]
mod menu_syntax_trigger_popup_window;
#[path = "menu_syntax_main_hint.rs"]
mod menu_syntax_main_hint;
#[path = "menu_syntax_actions.rs"]
pub(crate) mod menu_syntax_actions;
#[path = "menu_syntax_ai.rs"]
pub(crate) mod menu_syntax_ai;
#[path = "menu_syntax_ai_apply.rs"]
pub(crate) mod menu_syntax_ai_apply;
#[path = "ui_window.rs"]
mod ui_window;
#[path = "actions_toggle.rs"]
mod actions_toggle;
#[path = "about_route.rs"]
mod about_route;
#[path = "chat_actions.rs"]
mod chat_actions;
#[path = "webcam_actions.rs"]
mod webcam_actions;
#[path = "actions_dialog.rs"]
mod actions_dialog;
#[path = "shortcut_recorder.rs"]
mod shortcut_recorder;
#[path = "alias_input.rs"]
mod alias_input;
#[path = "quick_terminal_warm.rs"]
mod quick_terminal_warm;
#[path = "execution_paths.rs"]
mod execution_paths;
#[path = "path_action.rs"]
pub(crate) mod path_action;
#[path = "routes.rs"]
pub(crate) mod routes;
#[path = "execution_scripts.rs"]
mod execution_scripts;
#[path = "lifecycle_reset.rs"]
mod lifecycle_reset;
#[path = "shortcuts_hud_grid.rs"]
mod shortcuts_hud_grid;
#[path = "registries_state.rs"]
mod registries_state;
#[path = "automation_surface.rs"]
mod automation_surface;
#[path = "prompt_ai.rs"]
mod prompt_ai;
#[path = "naming_dialog.rs"]
mod naming_dialog;
mod tab_ai_mode;
#[path = "attachment_portal.rs"]
mod attachment_portal;
#[path = "trigger_builtin_dispatch.rs"]
mod trigger_builtin_dispatch;
#[path = "acp_surface_transitions.rs"]
mod acp_surface_transitions;
#[allow(dead_code)]
#[path = "window_orchestrator_bridge.rs"]
mod window_orchestrator_bridge;
#[cfg(test)]
#[path = "tests.rs"]
mod app_impl_state_sync_tests;

pub(crate) use startup::calculate_fallback_error_message;
