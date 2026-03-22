//! Tests for execution_helpers.rs — Scratch Pad CRUD, API key prompt logic,
//! and Claude config helpers.

use super::{count_occurrences as count, read_source as read};

fn execution_helpers_content() -> String {
    read("src/app_execute/execution_helpers.rs")
}

// ---------------------------------------------------------------------------
// Scratch Pad path
// ---------------------------------------------------------------------------

#[test]
fn get_scratch_pad_path_uses_kit_directory() {
    let content = execution_helpers_content();

    assert!(
        content.contains("get_kit_path().join(\"scratch-pad.md\")"),
        "Expected get_scratch_pad_path to join 'scratch-pad.md' to kit path"
    );
}

// ---------------------------------------------------------------------------
// Scratch Pad CRUD — directory creation + file loading
// ---------------------------------------------------------------------------

#[test]
fn open_scratch_pad_creates_parent_directory_if_missing() {
    let content = execution_helpers_content();

    assert!(
        content.contains("std::fs::create_dir_all(parent)"),
        "Expected open_scratch_pad to create parent directory with create_dir_all"
    );
}

#[test]
fn open_scratch_pad_handles_file_not_found_by_creating_empty_file() {
    let content = execution_helpers_content();

    assert!(
        content.contains("std::io::ErrorKind::NotFound"),
        "Expected open_scratch_pad to handle NotFound by creating an empty file"
    );
    assert!(
        content.contains("std::fs::write(&scratch_path, \"\")"),
        "Expected open_scratch_pad to write empty string for new scratch pad file"
    );
}

#[test]
fn open_scratch_pad_shows_toast_on_directory_creation_failure() {
    let content = execution_helpers_content();

    // Find the create_dir_all error handling block
    let dir_create_pos = content
        .find("create_dir_all(parent)")
        .expect("Expected create_dir_all call");
    let block = &content[dir_create_pos..dir_create_pos + 800];

    assert!(
        block.contains("show_error_toast("),
        "Expected directory creation failure to show error toast"
    );
}

#[test]
fn open_scratch_pad_shows_toast_on_read_failure() {
    let content = execution_helpers_content();

    assert!(
        content.contains("Failed to read scratch pad:"),
        "Expected open_scratch_pad to show toast on file read failure"
    );
}

// ---------------------------------------------------------------------------
// Scratch Pad auto-save
// ---------------------------------------------------------------------------

#[test]
fn scratch_pad_autosave_uses_two_second_interval() {
    let content = execution_helpers_content();

    assert!(
        content.contains("Duration::from_secs(2)"),
        "Expected scratch pad auto-save interval to be 2 seconds"
    );
}

#[test]
fn scratch_pad_autosave_stops_when_entity_is_dropped() {
    let content = execution_helpers_content();

    // The autosave loop should break when entity_weak.upgrade() returns None
    assert!(
        content.contains("entity_weak.upgrade()"),
        "Expected autosave loop to use weak reference for entity liveness check"
    );
    assert!(
        content.contains("false // Entity dropped, stop the task"),
        "Expected autosave to break when entity is dropped"
    );
}

#[test]
fn scratch_pad_submit_save_error_surfaces_toast() {
    let content = execution_helpers_content();

    // The submit callback should send errors through a channel for toast display
    assert!(
        content.contains("save_err_tx.try_send("),
        "Expected scratch pad submit callback to send save errors via channel"
    );
    assert!(
        content.contains("save_err_rx.recv().await"),
        "Expected scratch pad error listener to await save error channel"
    );
}

// ---------------------------------------------------------------------------
// API key prompt
// ---------------------------------------------------------------------------

#[test]
fn show_api_key_prompt_stores_pending_provider_name() {
    let content = execution_helpers_content();

    assert!(
        content.contains("self.pending_api_key_config = Some(provider_name.to_string())"),
        "Expected show_api_key_prompt to store pending provider name for completion handler"
    );
}

#[test]
fn show_api_key_prompt_checks_existing_secret_in_keyring() {
    let content = execution_helpers_content();

    assert!(
        content.contains("secrets::get_secret_info(&key)"),
        "Expected show_api_key_prompt to check for existing secret in keyring"
    );
    assert!(
        content.contains("exists_in_keyring"),
        "Expected show_api_key_prompt to pass exists_in_keyring to EnvPrompt"
    );
}

#[test]
fn handle_api_key_completion_clears_pending_state() {
    let content = execution_helpers_content();

    let completion_fn_start = content
        .find("fn handle_api_key_completion(")
        .expect("Expected handle_api_key_completion to exist");
    let block = &content[completion_fn_start..completion_fn_start + 300];

    assert!(
        block.contains("self.pending_api_key_config = None"),
        "Expected handle_api_key_completion to clear pending_api_key_config"
    );
}

#[test]
fn handle_api_key_completion_rebuilds_provider_registry_on_success() {
    let content = execution_helpers_content();

    let completion_fn_start = content
        .find("fn handle_api_key_completion(")
        .expect("Expected handle_api_key_completion to exist");
    let block = &content[completion_fn_start..completion_fn_start + 1200];

    assert!(
        block.contains("self.rebuild_provider_registry_async(cx)"),
        "Expected handle_api_key_completion to rebuild provider registry on success"
    );
}

#[test]
fn handle_api_key_completion_uses_deferred_resize() {
    let content = execution_helpers_content();

    let completion_fn_start = content
        .find("fn handle_api_key_completion(")
        .expect("Expected handle_api_key_completion to exist");
    let block = &content[completion_fn_start..completion_fn_start + 1200];

    assert!(
        block.contains("window.defer(cx,"),
        "Expected handle_api_key_completion to use deferred resize (called from render)"
    );
}

// ---------------------------------------------------------------------------
// Claude config
// ---------------------------------------------------------------------------

#[test]
fn enable_claude_code_in_config_recovers_from_validation_failure() {
    let content = execution_helpers_content();

    assert!(
        content.contains("ConfigWriteError::ValidationFailed"),
        "Expected enable_claude_code_in_config to handle validation failure"
    );
    assert!(
        content.contains("editor::recover_from_backup("),
        "Expected enable_claude_code_in_config to attempt backup recovery on validation failure"
    );
}

#[test]
fn enable_claude_code_in_config_checks_claude_cli_availability() {
    let content = execution_helpers_content();

    assert!(
        content.contains("claude_available"),
        "Expected enable_claude_code_in_config to check if Claude CLI is installed"
    );
    assert!(
        content.contains("\"--version\""),
        "Expected enable_claude_code_in_config to verify Claude CLI with --version"
    );
}

#[test]
fn enable_claude_code_in_config_shows_install_instructions_when_cli_missing() {
    let content = execution_helpers_content();

    assert!(
        content.contains("npm install -g @anthropic-ai/claude-code"),
        "Expected install instructions to be shown when Claude CLI is not found"
    );
}

// ---------------------------------------------------------------------------
// AI command dispatch — all user-facing branches use deferred helper
// ---------------------------------------------------------------------------

fn builtin_execution_content() -> String {
    read("src/app_execute/builtin_execution.rs")
}

#[test]
fn ai_open_and_new_conversation_use_deferred_helper() {
    let content = builtin_execution_content();

    let ai_command_section_start = content
        .find("AiCommandType::OpenAi | AiCommandType::NewConversation")
        .expect("Expected OpenAi/NewConversation match arm");
    let block = &content[ai_command_section_start..ai_command_section_start + 800];

    assert!(
        block.contains("open_ai_window_after_main_hide("),
        "Expected OpenAi/NewConversation to use deferred AI window helper"
    );
    assert!(
        block.contains("DeferredAiWindowAction::OpenOnly"),
        "Expected OpenAi/NewConversation to use OpenOnly deferred action"
    );
}

#[test]
fn ai_clear_conversation_uses_deferred_helper() {
    let content = builtin_execution_content();

    let clear_section_start = content
        .find("AiCommandType::ClearConversation")
        .expect("Expected ClearConversation match arm");
    let block = &content[clear_section_start..clear_section_start + 1200];

    assert!(
        block.contains("open_ai_window_after_main_hide("),
        "Expected ClearConversation to use deferred AI window helper after clearing"
    );
    assert!(
        block.contains("close_ai_window(cx)"),
        "Expected ClearConversation to close AI window before deferred reopen"
    );
}

#[test]
fn ai_clear_conversation_shows_hud_on_success() {
    let content = builtin_execution_content();

    let clear_section_start = content
        .find("AiCommandType::ClearConversation")
        .expect("Expected ClearConversation match arm");
    let block = &content[clear_section_start..clear_section_start + 1200];

    assert!(
        block.contains("Cleared AI conversations"),
        "Expected ClearConversation success to show HUD with confirmation message"
    );
}

#[test]
fn ai_clear_conversation_shows_toast_when_clear_fails() {
    let content = builtin_execution_content();

    let clear_section_start = content
        .find("AiCommandType::ClearConversation")
        .expect("Expected ClearConversation match arm");
    let block = &content[clear_section_start..clear_section_start + 1500];

    assert!(
        block.contains("Failed to clear AI conversations"),
        "Expected ClearConversation failure to show descriptive toast"
    );
}

#[test]
fn no_direct_ai_open_window_in_user_facing_branches() {
    let content = builtin_execution_content();

    // The only allowed `ai::open_ai_window` usage is inside internal helpers
    // (spawn_send_screen_to_ai_after_hide etc.), not in user-facing match arms.
    // Count direct calls outside the helper functions.
    let ai_chat_section = content
        .find("BuiltInFeature::AiChat")
        .expect("Expected AiChat arm");
    let ai_chat_block = &content[ai_chat_section..ai_chat_section + 600];
    assert!(
        !ai_chat_block.contains("ai::open_ai_window("),
        "AiChat should not directly call ai::open_ai_window — use deferred helper"
    );
}

// ---------------------------------------------------------------------------
// Settings command handlers — Toast/HUD feedback on errors
// ---------------------------------------------------------------------------

#[test]
fn reset_window_positions_shows_hud_confirmation() {
    let content = builtin_execution_content();

    let reset_section_start = content
        .find("SettingsCommandType::ResetWindowPositions")
        .expect("Expected ResetWindowPositions match arm");
    let block = &content[reset_section_start..reset_section_start + 800];

    assert!(
        block.contains("show_hud("),
        "Expected ResetWindowPositions to show HUD feedback"
    );
    assert!(
        block.contains("Window positions reset"),
        "Expected ResetWindowPositions HUD to confirm reset"
    );
}

#[test]
fn reset_window_positions_suppresses_save_before_reset() {
    let content = builtin_execution_content();

    let reset_section_start = content
        .find("SettingsCommandType::ResetWindowPositions")
        .expect("Expected ResetWindowPositions match arm");
    let block = &content[reset_section_start..reset_section_start + 500];

    assert!(
        block.contains("suppress_save()"),
        "Expected ResetWindowPositions to suppress position saving before reset"
    );
    assert!(
        block.contains("reset_all_positions()"),
        "Expected ResetWindowPositions to call reset_all_positions"
    );
}

#[test]
fn settings_api_key_prompts_use_show_api_key_prompt() {
    let content = builtin_execution_content();

    let settings_section_start = content
        .find("SettingsCommandType::ConfigureOpenAiApiKey")
        .expect("Expected ConfigureOpenAiApiKey match arm");
    let block = &content[settings_section_start..settings_section_start + 200];

    assert!(
        block.contains("self.show_api_key_prompt("),
        "Expected ConfigureOpenAiApiKey to use show_api_key_prompt helper"
    );
    assert!(
        block.contains("SCRIPT_KIT_OPENAI_API_KEY"),
        "Expected ConfigureOpenAiApiKey to pass correct key name"
    );
}

#[test]
fn settings_api_key_prompts_cover_all_providers() {
    let content = builtin_execution_content();

    // All three API key configuration commands should exist
    assert!(
        content.contains("SettingsCommandType::ConfigureVercelApiKey"),
        "Expected ConfigureVercelApiKey settings command"
    );
    assert!(
        content.contains("SettingsCommandType::ConfigureOpenAiApiKey"),
        "Expected ConfigureOpenAiApiKey settings command"
    );
    assert!(
        content.contains("SettingsCommandType::ConfigureAnthropicApiKey"),
        "Expected ConfigureAnthropicApiKey settings command"
    );

    // Each should use show_api_key_prompt
    let prompt_count = count(&content, "self.show_api_key_prompt(");
    assert!(
        prompt_count >= 3,
        "Expected at least 3 API key prompt usages (found {prompt_count})"
    );
}
