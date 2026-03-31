#![allow(dead_code)]
#![allow(unused_imports)]

pub mod capture;
mod device;
pub mod download;
mod runtime;
mod transcription;
mod types;
mod visualizer;
mod window;

pub use capture::{start_capture, DictationCaptureHandle};
pub use device::{
    apply_device_selection, build_device_menu_items, default_input_device,
    list_input_device_menu_items, list_input_devices, resolve_selected_input_device,
    save_dictation_device_id, DeviceResolution, DictationDeviceMenuItem,
    DictationDeviceSelectionAction,
};
pub use runtime::{
    abort_dictation, get_dictation_target, is_dictation_recording, maybe_unload_transcriber,
    set_overlay_phase, snapshot_overlay_state, toggle_dictation, transcribe_captured_audio,
};
pub use transcription::{
    build_session_result, captured_duration, is_parakeet_model_available, merge_captured_chunks,
    resolve_default_model_path, resolve_whisper_model_path, DictationEngine, DictationTranscriber,
    DictationTranscriptionConfig, ParakeetDictationEngine, WhisperDictationEngine,
    PARAKEET_MODEL_ARCHIVE_SIZE, PARAKEET_MODEL_URL,
};
pub use types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDestination, DictationDeviceId, DictationDeviceInfo, DictationDeviceTransport,
    DictationLevel, DictationModelStatus, DictationSessionPhase, DictationSessionResult,
    DictationTarget, DictationToggleOutcome, RawAudioChunk,
};
pub use visualizer::{animate_bars, bars_for_level, compute_level};
pub use window::{
    begin_overlay_session, close_dictation_overlay, is_dictation_overlay_open,
    open_dictation_overlay, overlay_generation, set_overlay_abort_callback,
    update_dictation_overlay, DictationOverlay, DictationOverlayState,
};
pub(crate) use window::overlay_phase_copy;

#[cfg(test)]
mod tests;
