pub mod capture;
mod device;
mod transcription;
mod types;
mod visualizer;

pub use capture::{start_capture, DictationCaptureHandle};
pub use device::{default_input_device, list_input_devices};
pub use transcription::{
    build_session_result, captured_duration, merge_captured_chunks, DictationEngine,
    DictationTranscriber, DictationTranscriptionConfig,
};
pub use types::{
    CapturedAudioChunk, DictationCaptureConfig, DictationCaptureEvent, DictationDestination,
    DictationDeviceId, DictationDeviceInfo, DictationLevel, DictationSessionPhase,
    DictationSessionResult, RawAudioChunk,
};
pub use visualizer::{bars_for_level, compute_level};

#[cfg(test)]
mod tests;
