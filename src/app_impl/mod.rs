#[path = "app_impl/startup.rs"]
mod startup;
#[path = "app_impl/theme_focus.rs"]
mod theme_focus;
#[path = "app_impl/refresh_scriptlets.rs"]
mod refresh_scriptlets;
#[path = "app_impl/filtering_cache.rs"]
mod filtering_cache;
#[path = "app_impl/selection_fallback.rs"]
mod selection_fallback;
#[path = "app_impl/filter_input_core.rs"]
mod filter_input_core;
#[path = "app_impl/filter_input_change.rs"]
mod filter_input_change;
#[path = "app_impl/filter_input_updates.rs"]
mod filter_input_updates;
#[path = "app_impl/ui_window.rs"]
mod ui_window;
#[path = "app_impl/actions_toggle.rs"]
mod actions_toggle;
#[path = "app_impl/chat_actions.rs"]
mod chat_actions;
#[path = "app_impl/webcam_actions.rs"]
mod webcam_actions;
#[path = "app_impl/actions_dialog.rs"]
mod actions_dialog;
#[path = "app_impl/shortcut_recorder.rs"]
mod shortcut_recorder;
#[path = "app_impl/alias_input.rs"]
mod alias_input;
#[path = "app_impl/execution_paths.rs"]
mod execution_paths;
#[path = "app_impl/execution_scripts.rs"]
mod execution_scripts;
#[path = "app_impl/lifecycle_reset.rs"]
mod lifecycle_reset;
#[path = "app_impl/shortcuts_hud_grid.rs"]
mod shortcuts_hud_grid;
#[path = "app_impl/registries_state.rs"]
mod registries_state;
#[path = "app_impl/prompt_ai.rs"]
mod prompt_ai;
#[cfg(test)]
#[path = "app_impl/tests.rs"]
mod app_impl_state_sync_tests;

pub(super) use startup::calculate_fallback_error_message;
