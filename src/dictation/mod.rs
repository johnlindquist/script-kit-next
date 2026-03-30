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
pub use device::{default_input_device, list_input_devices};
pub use transcription::{
    build_session_result, captured_duration, merge_captured_chunks, DictationEngine,
    DictationTranscriber, DictationTranscriptionConfig, WhisperDictationEngine,
};
pub use types::{
    CapturedAudioChunk, DictationCaptureConfig, DictationCaptureEvent, DictationDestination,
    DictationDeviceId, DictationDeviceInfo, DictationLevel, DictationSessionPhase,
    DictationSessionResult, RawAudioChunk,
};
pub use visualizer::{bars_for_level, compute_level};
pub use runtime::toggle_dictation;
pub use window::{
    close_dictation_overlay, is_dictation_overlay_open, open_dictation_overlay,
    update_dictation_overlay, DictationOverlay, DictationOverlayState,
};

#[cfg(test)]
mod tests;
