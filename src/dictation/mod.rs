//! Voice dictation: audio capture, transcription, and transcript delivery.
//!
//! This module compiles into both the library and the binary crate.  The
//! re-export lists below form the library's public API, so the binary build
//! (which only consumes a subset) would otherwise flag them as unused —
//! hence the scoped `unused_imports` allow.  `dead_code` is deliberately NOT
//! allowed module-wide: genuinely dead items must be deleted or carry a
//! documented item-level allow.
#![allow(unused_imports)]

pub mod capture;
mod catalog;
mod delivery;
mod device;
pub mod download;
mod history;
mod mic_check;
mod runtime;
mod setup;
pub mod takes;
mod transcription;
mod types;
mod visualizer;

pub use capture::{start_capture, DictationCaptureHandle};
pub use catalog::{
    dictation_model_catalog, dictation_model_entry, format_dictation_model_size,
    DictationEngineKind, DictationModelCatalogEntry, DictationModelId,
};
pub use delivery::{
    capture_frozen_external_destination, dictation_mutation_error_outcome,
    parse_dictation_target_label, resolve_delivery_target_request, resolve_dictation_target_label,
    validate_frozen_external_destination, with_frozen_dictation_destination,
    DictationDeliveryTargetResolution, DictationDeliveryTargetSource,
    DictationTargetLabelResolution, DictationWrongTargetReason, DictationWrongTargetRefusalDraft,
};
pub use device::{
    apply_device_selection, build_device_menu_items, default_input_device,
    device_selection_action_from_value, device_selection_value, list_input_device_menu_items,
    list_input_devices, microphone_display_label, microphone_permission_status,
    request_microphone_permission, request_microphone_permission_nonblocking,
    resolve_selected_input_device, save_dictation_device_id, save_dictation_language,
    save_dictation_model, DeviceResolution, DictationDeviceMenuItem,
    DictationDeviceSelectionAction, DICTATION_SYSTEM_DEFAULT_DEVICE_VALUE,
};
pub use history::{
    build_history_entry, delete_history_entry, delete_history_held_title,
    format_history_duration_ms, format_history_timestamp, get_history_entry,
    hydrate_dictation_resource_from_history, load_history, load_history_result,
    record_dictation_history, root_dictation_history_query_is_eligible, search_history,
    search_root_dictation_history, search_root_dictation_history_cached,
    search_root_dictation_history_direct, seed_owned_dictation_history, DictationHistoryEntry,
    DictationHistorySearchField, DictationHistorySearchHit, RootDictationHistorySearchHit,
    RootDictationHistorySectionOptions, DICTATION_HISTORY_ENTRY_VERSION,
    DICTATION_HISTORY_LEGACY_UNKNOWN_TARGET_ID,
};
pub(crate) use history::{
    discard_root_dictation_history_refresh, finish_root_dictation_history_refresh,
    invalidate_owned_root_dictation_history_freshness, owned_root_dictation_history_snapshot,
    read_root_dictation_history_snapshot, reset_owned_root_dictation_history,
    root_dictation_history_cache_is_fresh, root_dictation_history_fresh_cache_status,
    try_begin_root_dictation_history_refresh, RootDictationHistorySnapshot,
};
pub use mic_check::{
    assess_samples, laptop_lid_closed, last_microphone_check, run_microphone_check,
    MicrophoneCheck, MicrophoneLevel, MICROPHONE_CHECK_DURATION,
};
pub use runtime::{
    abort_dictation, automation_state, begin_stop_capture, can_cycle_dictation_target,
    claim_dictation_delivery, current_dictation_phase, cycle_dictation_target,
    delivery_receipt_generation, dictation_auto_stop_due, dictation_delivery_was_claimed,
    dictation_elapsed, dictation_stop_target, dictation_target_generation, finalize_progress,
    finish_stop_capture, get_active_dictation_device, get_dictation_target,
    get_dictation_target_selection, is_dictation_busy, is_dictation_recording,
    is_dictation_stopping, last_delivery_receipt, last_partial_transcript, last_stop_receipt,
    last_wrong_target_refusal, maybe_unload_transcriber, next_dictation_delivery_id,
    pending_dictation_device_label, record_delivery_receipt, record_wrong_target_refusal,
    redacted_transcript_fingerprint, resolve_final_or_partial_transcript,
    set_dictation_session_selection, set_dictation_session_target, set_dictation_target_cycle,
    set_pending_dictation_device_label, shutdown_transcriber, snapshot_live_state,
    toggle_dictation, transcribe_captured_audio, BeginStopCapture, DictationStopJob,
    DictationStopReason, DictationTranscriptResolution,
};
pub(crate) use runtime::{
    clear_dictation_recovery_work, clear_dictation_return_origin, dictation_pipeline_failure_state,
    dictation_recovery_work, dictation_return_origin, preserve_dictation_recovery_work,
    replace_dictation_recovery_work, replace_dictation_return_origin,
    retain_frozen_selection_for_delivery, toggled_post_stop_restart, DictationRecoveryWork,
};
pub use setup::{
    build_dictation_setup_state, dictation_setup_extra_rows, DictationHotkeyStatus,
    DictationMicrophonePermissionStatus, DictationMicrophoneStatus, DictationSetupExtras,
    DictationSetupRow, DictationSetupState, DICTATION_SETUP_GRANT_ACCESSIBILITY,
    DICTATION_SETUP_RETRY_LAST, DICTATION_SETUP_TEST_MIC,
};
pub use transcription::{
    build_session_result, captured_duration, dictation_engine_receipt, is_parakeet_model_available,
    merge_captured_chunks, plan_long_audio_chunks, resolve_default_model_path,
    resolve_whisper_model_path, DictationEngine, DictationEngineReceipt, DictationTranscriber,
    DictationTranscriptionConfig, LocalDictationEngine, PARAKEET_MODEL_SIZE, PARAKEET_MODEL_URL,
    WHISPER_MODEL_SIZE, WHISPER_MODEL_URL,
};
pub use types::{
    CapturedAudioChunk, CompletedDictationCapture, DayPageDictationDestinationSnapshot,
    DictationAutoSubmitPermission, DictationCaptureConfig, DictationCaptureEvent,
    DictationDeliveryFailureReason, DictationDeliveryOutcome, DictationDeliveryRequest,
    DictationDestination, DictationDeviceId, DictationDeviceInfo, DictationDeviceTransport,
    DictationFailureRecoveryCapabilities, DictationFailureState, DictationLiveState,
    DictationModelStatus, DictationMutationReceipt, DictationRecoveryAction,
    DictationRecoveryCapabilities, DictationReturnOrigin, DictationSessionPhase,
    DictationSessionResult, DictationTarget, DictationTargetDescriptor,
    DictationTargetPersistenceClass, DictationTargetSelection, DictationToggleOutcome,
    DictationTranscriptPreservationReceipt, FrozenAgentChatPolicy, FrozenDictationDestination,
    FrozenMainDictationOwner, ImmutableDictationTranscript, RawAudioChunk, ALL_DICTATION_TARGETS,
};
pub use visualizer::{animate_bars, silent_bars};
#[cfg(test)]
mod tests;
