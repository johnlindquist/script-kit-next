//! AI surfaces and shared contracts.
//!
//! # Architecture
//!
//! ```text
//! src/ai/
//! ├── harness/     - Harness config + flat context-block / artifact-authoring guidance formatting
//! ├── tab_context.rs - Tab AI context, receipts, memory lookup, compatibility types
//! ├── message_parts.rs - MCP/file context-part composition
//! ├── context_contract.rs - Shared context contract enforcement
//! ├── current_app_automation_memory.rs - Bundle-scoped prior-automation memory
//! ├── model.rs / storage.rs / providers.rs - Deprecated legacy AI window data + BYOK providers
//! └── window/      - Deprecated legacy AI window UI and interactions
//! ```
//!
//! # Primary ACP Chat contract
//!
//! - User-facing AI chat surface: ACP Chat
//! - Entry points should route to `open_tab_ai_acp_with_entry_intent(...)` when they need the canonical chat UI
//! - Compatibility-named `tab_ai_*` helpers and harness/context types still back ACP Chat plumbing
//! - The legacy `window/` module remains only for deprecated compatibility flows and should not be used for new entry points

// Re-exports intentionally cover the module's API surface.
#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) mod acp;
pub(crate) mod config;
pub(crate) mod context_contract;
#[cfg(test)]
mod context_contract_integration_tests;
pub(crate) mod context_mentions;
pub(crate) mod context_picker_row;
pub(crate) mod current_app_automation_memory;
pub(crate) mod explicit_target_handoff;
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
    should_include_artifact_authoring_guidance, tab_ai_harness_config_path,
    tab_ai_screenshot_prefix, tab_ai_surface_preference_for_prompt, validate_tab_ai_harness_config,
    HarnessBackendKind, HarnessConfig, TabAiCaptureKind, TabAiHarnessSessionState,
    TabAiHarnessSubmissionMode, TabAiHarnessWarmState, TabAiScreenshotFile, TabAiSurfacePreference,
    BUN_BUILD_VERIFICATION_MARKER, BUN_EXECUTE_VERIFICATION_MARKER, SCRIPT_AUTHORING_SKILL_MARKER,
    TAB_AI_HARNESS_CONFIG_SCHEMA_VERSION, TAB_AI_HARNESS_CONTEXT_SCHEMA_VERSION,
    TAB_AI_SCREENSHOT_MAX_KEEP,
};
pub use self::harness::{
    plan_tab_ai_quick_submit, TabAiQuickSubmitKind, TabAiQuickSubmitPlan, TabAiQuickSubmitSource,
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
    tab_ai_apply_back_footer_label, tab_ai_execution_audit_path, tab_ai_experience_pack_name,
    tab_ai_experience_pack_subtitle, tab_ai_intent_uses_implicit_target, tab_ai_memory_index_path,
    truncate_tab_ai_text, write_tab_ai_memory_entry, write_tab_ai_memory_entry_to_path,
    TabAiApplyBackHint, TabAiApplyBackRoute, TabAiClipboardContext, TabAiClipboardHistoryEntry,
    TabAiContextBlob, TabAiDegradationReason, TabAiExecutionReceipt, TabAiExecutionRecord,
    TabAiExecutionStatus, TabAiExperienceIntent, TabAiExperiencePack, TabAiExperienceSpec,
    TabAiFieldStatus, TabAiInvocationReceipt, TabAiMemoryEntry, TabAiMemoryResolution,
    TabAiMemoryResolutionOutcome, TabAiMemoryResolutionReason, TabAiMemorySuggestion,
    TabAiSourceType, TabAiSuggestedIntentSpec, TabAiTargetAudit, TabAiTargetContext,
    TabAiUiSnapshot, TAB_AI_CONTEXT_SCHEMA_VERSION, TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION,
    TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION, TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION,
    TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION, TAB_AI_TARGET_AUDIT_SCHEMA_VERSION,
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
pub use self::context_mentions::{
    mention_range_at_cursor, parse_inline_context_mentions, InlineContextMention,
};
pub(crate) use self::explicit_target_handoff::request_explicit_acp_handoff_from_secondary_window;
pub use self::window::context_picker::types::ContextPickerTrigger;
pub use self::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, ContextPickerItemSnapshot, ContextPickerSection,
    ContextPickerSnapshot, ContextPickerState, SlashCommandPayload,
};
pub use self::window::context_picker::{
    build_picker_items, build_slash_picker_items, score_builtin, score_builtin_with_trigger,
};
pub use self::window::context_preflight::{
    estimate_tokens_from_text, preflight_state_from_receipt, status_from_decision,
    ContextPreflightSnapshot, ContextPreflightState, ContextPreflightStatus,
};

// ---------------------------------------------------------------------------
// Pending explicit ACP target — cross-window handoff slot
// ---------------------------------------------------------------------------

use parking_lot::Mutex;
use std::sync::OnceLock;

/// Pending explicit ACP target from a secondary window (e.g. detached actions
/// popup). The main window checks this after the secondary surface closes and
/// hands the target to `open_tab_ai_acp_with_explicit_target`.
static PENDING_EXPLICIT_ACP_TARGET: OnceLock<Mutex<Option<TabAiTargetContext>>> = OnceLock::new();

/// Build a canonical ACP handoff target for an action-menu selection.
pub(crate) fn build_action_target_for_ai(
    action: &crate::actions::Action,
    host_label: &str,
) -> TabAiTargetContext {
    TabAiTargetContext {
        source: "ActionsDialog".to_string(),
        kind: "action".to_string(),
        semantic_id: crate::protocol::generate_semantic_id("action", 0, &action.id),
        label: action.title.clone(),
        metadata: Some(serde_json::json!({
            "actionId": action.id,
            "title": action.title,
            "description": action.description,
            "category": format!("{:?}", action.category),
            "shortcut": action.shortcut,
            "host": host_label,
        })),
    }
}

/// Enqueue a canonical target for ACP handoff from a secondary window.
///
/// The main window picks this up via `take_pending_explicit_acp_target` the
/// next time it processes a close-actions-popup event.
pub fn enqueue_explicit_acp_target(target: TabAiTargetContext) {
    let storage = PENDING_EXPLICIT_ACP_TARGET.get_or_init(|| Mutex::new(None));
    *storage.lock() = Some(target);
}

/// Take a pending ACP handoff target, if any, clearing the slot.
pub fn take_pending_explicit_acp_target() -> Option<TabAiTargetContext> {
    PENDING_EXPLICIT_ACP_TARGET
        .get()
        .and_then(|storage| storage.lock().take())
}

/// Build the canonical chip label used by explicit ACP target handoffs.
///
/// Notes, actions, and any future secondary surfaces should use this instead
/// of formatting bespoke labels locally. The main-window ACP entry also
/// delegates here so all chip labels share a single source of truth.
pub(crate) fn format_explicit_target_chip_label(target: &TabAiTargetContext) -> String {
    let prefix = match target.kind.as_str() {
        "file" => "File",
        "directory" => "Folder",
        "search_query" => "Search",
        "input" => "Input",
        "clipboard_entry" => "Clipboard",
        "script" | "scriptlet" | "builtin" => "Command",
        "window" => "Window",
        "app" => "App",
        "process" => "Process",
        "menu_command" => "Menu Command",
        "action" => "Action",
        "note" => "Note",
        "agent" => "Agent",
        "fallback" => "Suggestion",
        _ => "Selection",
    };
    format!("{}: {}", prefix, target.label)
}
