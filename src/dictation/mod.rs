#![allow(dead_code)]
#![allow(unused_imports)]

pub mod capture;
mod device;
mod runtime;
mod transcription;
mod types;
mod visualizer;
mod window;

pub use capture::{start_capture, DictationCaptureHandle};
pub use device::{
    apply_device_selection, build_device_menu_items, default_input_device,
    list_input_device_menu_items, list_input_devices, resolve_selected_input_device,
    save_dictation_device_id, DictationDeviceMenuItem, DictationDeviceSelectionAction,
};
pub use runtime::{
    maybe_unload_transcriber, snapshot_overlay_state, toggle_dictation, transcribe_captured_audio,
};
pub use transcription::{
    build_session_result, captured_duration, merge_captured_chunks, resolve_default_model_path,
    DictationEngine, DictationTranscriber, DictationTranscriptionConfig, WhisperDictationEngine,
};
pub use types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDestination, DictationDeviceId, DictationDeviceInfo, DictationLevel,
    DictationSessionPhase, DictationSessionResult, DictationToggleOutcome, RawAudioChunk,
};
pub use visualizer::{bars_for_level, compute_level};
pub use window::{
    close_dictation_overlay, is_dictation_overlay_open, open_dictation_overlay,
    update_dictation_overlay, DictationOverlay, DictationOverlayState,
};

#[cfg(test)]
mod tests;
