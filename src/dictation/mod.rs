pub mod capture;
mod device;
mod types;
mod visualizer;

pub use capture::{start_capture, DictationCaptureHandle};
pub use device::{default_input_device, list_input_devices};
pub use types::{
    CapturedAudioChunk, DictationCaptureConfig, DictationCaptureEvent, DictationDeviceId,
    DictationDeviceInfo, DictationLevel, RawAudioChunk,
};
pub use visualizer::{bars_for_level, compute_level};

#[cfg(test)]
mod tests;
