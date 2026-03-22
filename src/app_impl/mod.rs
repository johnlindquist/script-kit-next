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
#[path = "ui_window.rs"]
mod ui_window;
#[path = "actions_toggle.rs"]
mod actions_toggle;
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
#[path = "execution_paths.rs"]
mod execution_paths;
#[path = "execution_scripts.rs"]
mod execution_scripts;
#[path = "lifecycle_reset.rs"]
mod lifecycle_reset;
#[path = "shortcuts_hud_grid.rs"]
mod shortcuts_hud_grid;
#[path = "registries_state.rs"]
mod registries_state;
#[path = "prompt_ai.rs"]
mod prompt_ai;
#[path = "naming_dialog.rs"]
mod naming_dialog;
#[cfg(test)]
#[path = "tests.rs"]
mod app_impl_state_sync_tests;

pub(crate) use startup::calculate_fallback_error_message;
