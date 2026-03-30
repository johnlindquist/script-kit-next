//! AI surfaces and shared contracts.
//!
//! This module contains both:
//! - the legacy AI window/chat stack (`model`, `storage`, `providers`, `window`)
//! - the current Tab AI harness/context stack (`harness`, `tab_context`, `message_parts`, `context_contract`, `current_app_automation_memory`)
//!
//! The primary Tab-triggered AI experience is **not** the old inline chat UI.
//! Pressing Tab routes to a warm harness terminal in `AppView::QuickTerminalView`
//! and injects `TabAiContextBlob` into the running CLI harness via
//! `build_tab_ai_harness_submission()` and PTY-backed text injection.
//!
//! # Architecture
//!
//! ```text
//! src/ai/
//! ├── harness/     - Harness config + `<scriptKitContext>` / `<scriptKitHints>` formatting
//! ├── tab_context.rs - Tab AI context, receipts, memory lookup, compatibility types
//! ├── message_parts.rs - MCP/file context-part composition
//! ├── context_contract.rs - Shared context contract enforcement
//! ├── current_app_automation_memory.rs - Bundle-scoped prior-automation memory
//! ├── model.rs / storage.rs / providers.rs - Legacy AI window data + BYOK providers
//! └── window/      - Legacy AI window UI and interactions
//! ```
//!
//! # Primary Tab AI contract
//!
//! - Entry path: `open_tab_ai_chat()` → `begin_tab_ai_harness_entry()` → `open_tab_ai_harness_terminal_from_request()`
//! - Surface: `AppView::QuickTerminalView` rendered by `TermPrompt`
//! - Submission modes: `TabAiHarnessSubmissionMode::PasteOnly` and `TabAiHarnessSubmissionMode::Submit`
//! - Capture profile: `CaptureContextOptions::tab_ai_submit()` for the landed PTY path
//! - Legacy chat/window code still exists, but it is not the default Tab AI surface.

// Re-exports intentionally cover the module's API surface.
#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) mod acp;
pub(crate) mod config;
pub(crate) mod context_contract;
#[cfg(test)]
mod context_contract_integration_tests;
pub(crate) mod context_mentions;
pub(crate) mod current_app_automation_memory;
pub(crate) mod harness;
pub mod message_parts;
pub(crate) mod model;
pub(crate) mod preflight_audit;
pub(crate) mod presets;
pub(crate) mod providers;
#[cfg(test)]
mod public_contract_tests;
pub(crate) mod script_generation;
pub(crate) mod sdk_handlers;
pub(crate) mod session;
pub(crate) mod storage;
pub(crate) mod tab_context;
pub(crate) mod window;

// Re-export commonly used types
pub use self::config::{DetectedKeys, ModelInfo, ProviderConfig};
pub use self::current_app_automation_memory::{
    current_app_automation_memory_index_path, read_current_app_automation_memory_index,
    resolve_current_app_automation_from_memory, upsert_current_app_automation_memory_from_receipt,
    CurrentAppAutomationMemoryDecision, CurrentAppAutomationMemoryIndexEntry,
};
pub use self::harness::{
    build_tab_ai_harness_context_block, build_tab_ai_harness_submission,
    cleanup_old_tab_ai_screenshot_files_in_dir, read_tab_ai_harness_config,
    tab_ai_harness_config_path, tab_ai_screenshot_prefix, validate_tab_ai_harness_config,
    HarnessBackendKind, HarnessConfig, TabAiCaptureKind, TabAiHarnessSessionState,
    TabAiHarnessSubmissionMode, TabAiHarnessWarmState, TabAiScreenshotFile,
    TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION, TAB_AI_HARNESS_CONTEXT_SCHEMA_VERSION,
    TAB_AI_SCREENSHOT_MAX_KEEP,
};
pub use self::message_parts::{
    file_path_parts, merge_context_parts, prepare_user_message_with_receipt,
    resolve_context_part_to_prompt_block, resolve_context_parts_to_prompt_prefix,
    resolve_context_parts_with_receipt, AiContextPart, ContextAssemblyReceipt,
    ContextPartPreparationOutcome, ContextPartPreparationOutcomeKind, ContextResolutionFailure,
    ContextResolutionReceipt, PreparedMessageDecision, PreparedMessageReceipt,
    AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
};
pub use self::model::{Chat, ChatId, ChatSource, Message, MessageRole};
pub use self::preflight_audit::{
    build_actionable_preflight_error, log_preflight_audit, ActionableContextFailure,
    AiPreflightAudit, AI_PREFLIGHT_AUDIT_SCHEMA_VERSION,
};
pub use self::providers::{AiProvider, ProviderMessage, ProviderRegistry};
pub use self::script_generation::{
    extract_current_app_recipe_from_script, generate_script_from_prompt,
    generate_script_from_prompt_with_receipt, generated_script_receipt_path,
    GeneratedScriptContractAudit, GeneratedScriptMetadataStyle, GeneratedScriptOutput,
    GeneratedScriptReceipt, AI_GENERATED_SCRIPT_RECEIPT_SCHEMA_VERSION,
    AI_SCRIPT_GENERATION_SYSTEM_PROMPT,
};
pub use self::sdk_handlers::try_handle_ai_message;
pub use self::storage::{
    clear_all_chats, create_chat, delete_chat, get_all_chats, get_chat, get_chat_messages,
    get_deleted_chats, get_last_message_preparation_audit, init_ai_db, insert_mock_data,
    save_message, save_message_preparation_audit, search_chats, update_chat_title,
};
pub use self::tab_context::{
    append_tab_ai_execution_receipt, append_tab_ai_execution_receipt_to_path,
    build_tab_ai_apply_back_hint_from_source, build_tab_ai_execution_receipt,
    build_tab_ai_experience_intents, build_tab_ai_experience_spec, build_tab_ai_suggested_intents,
    build_tab_ai_user_prompt, cleanup_tab_ai_temp_script, detect_tab_ai_source_type_from_prompt,
    read_tab_ai_memory_index, read_tab_ai_memory_index_from_path,
    recent_tab_ai_automations_for_bundle, recent_tab_ai_automations_for_bundle_from_path,
    resolve_tab_ai_memory_suggestions, resolve_tab_ai_memory_suggestions_from_path,
    resolve_tab_ai_memory_suggestions_with_outcome,
    resolve_tab_ai_memory_suggestions_with_outcome_from_path,
    resolve_tab_ai_prior_automations_for_entry,
    resolve_tab_ai_prior_automations_for_entry_from_path, should_offer_save,
    tab_ai_execution_audit_path, tab_ai_experience_pack_name, tab_ai_experience_pack_subtitle,
    tab_ai_intent_uses_implicit_target, tab_ai_memory_index_path, truncate_tab_ai_text,
    write_tab_ai_memory_entry, write_tab_ai_memory_entry_to_path, TabAiApplyBackHint,
    TabAiApplyBackRoute, TabAiClipboardContext, TabAiClipboardHistoryEntry, TabAiContextBlob,
    TabAiDegradationReason, TabAiExecutionReceipt, TabAiExecutionRecord, TabAiExecutionStatus,
    TabAiExperienceIntent, TabAiExperiencePack, TabAiExperienceSpec, TabAiFieldStatus,
    TabAiInvocationReceipt, TabAiMemoryEntry, TabAiMemoryResolution, TabAiMemoryResolutionOutcome,
    TabAiMemoryResolutionReason, TabAiMemorySuggestion, TabAiSourceType, TabAiSuggestedIntentSpec,
    TabAiTargetAudit, TabAiTargetContext, TabAiUiSnapshot, TAB_AI_CONTEXT_SCHEMA_VERSION,
    TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION, TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION,
    TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION, TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
    TAB_AI_TARGET_AUDIT_SCHEMA_VERSION,
};
pub use self::window::{
    add_ai_attachment, apply_ai_preset, close_ai_window, get_ai_window_state, is_ai_window,
    is_ai_window_open, is_ai_window_ready, open_ai_window, open_ai_window_with_chat,
    open_mini_ai_window, open_mini_ai_window_from, reload_ai_presets, set_ai_input,
    set_ai_input_with_image, set_ai_pending_chat, set_ai_search, show_ai_command_bar,
    simulate_ai_key, start_ai_chat, AiMiniDebugSnapshot, PendingChatMessage,
};

// Re-export context-composer types for integration tests
pub use self::context_contract::{
    context_attachment_specs, ContextAttachmentKind, ContextAttachmentSpec,
};
pub use self::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, ContextPickerItemSnapshot, ContextPickerSection,
    ContextPickerSnapshot, ContextPickerState,
};
pub use self::window::context_picker::{build_picker_items, score_builtin};
pub use self::window::context_preflight::{
    estimate_tokens_from_text, preflight_state_from_receipt, status_from_decision,
    ContextPreflightSnapshot, ContextPreflightState, ContextPreflightStatus,
};
