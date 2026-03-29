use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DictationDeviceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationDeviceInfo {
    pub id: DictationDeviceId,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationCaptureConfig {
    pub sample_rate_hz: u32,
    pub chunk_duration: Duration,
    pub level_window: Duration,
}

impl Default for DictationCaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 16_000,
            chunk_duration: Duration::from_millis(40),
            level_window: Duration::from_millis(60),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawAudioChunk {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapturedAudioChunk {
    pub sample_rate_hz: u32,
    pub samples: Vec<f32>,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DictationLevel {
    pub rms: f32,
    pub peak: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DictationCaptureEvent {
    Chunk(CapturedAudioChunk),
    Level(DictationLevel),
    EndOfStream,
}

// --- Session / transcription types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationDestination {
    ActivePrompt,
    FrontmostApp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationSessionPhase {
    Idle,
    Recording,
    Transcribing,
    Delivering,
    Finished,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationSessionResult {
    pub transcript: String,
    pub destination: DictationDestination,
    pub audio_duration: Duration,
}
