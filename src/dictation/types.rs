use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DictationDeviceId(pub String);

/// Transport type for an audio input device.
///
/// Used to rank devices when the user has no explicit preference: built-in
/// microphones are preferred over USB/Bluetooth/virtual devices as a safe
/// first-launch default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DictationDeviceTransport {
    BuiltIn,
    Usb,
    Bluetooth,
    Virtual,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationDeviceInfo {
    pub id: DictationDeviceId,
    pub name: String,
    pub is_default: bool,
    pub transport: DictationDeviceTransport,
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

// --- Capture completion types ---

/// Audio data returned when dictation recording is stopped.
///
/// Contains the collected audio chunks and their total duration.  The caller
/// is responsible for transcription and delivery — the runtime only captures.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletedDictationCapture {
    pub chunks: Vec<CapturedAudioChunk>,
    pub audio_duration: Duration,
}

/// Outcome of a `toggle_dictation()` call.
#[derive(Debug, Clone, PartialEq)]
pub enum DictationToggleOutcome {
    /// A new recording session was started.
    Started,
    /// An active recording was stopped.  `Some(capture)` when audio was
    /// collected, `None` for an empty recording.
    Stopped(Option<CompletedDictationCapture>),
}

// --- Session / transcription types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationDestination {
    ActivePrompt,
    FrontmostApp,
    NotesEditor,
    AiChatComposer,
    TabAiHarness,
}

/// The Script Kit surface that was active when dictation was invoked.
///
/// Determined at dictation start time and stored in the session so the
/// transcript delivery path knows where to route without re-inspecting
/// the UI (which may have changed while the user was speaking).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationTarget {
    /// A prompt in the main window that accepts text input (arg, path,
    /// select, env, template, form, file search, mini, micro).
    MainWindowPrompt,
    /// The notes window editor.
    NotesEditor,
    /// The AI chat window composer.
    AiChatComposer,
    /// The Tab AI harness terminal (`QuickTerminalView`).
    TabAiHarness,
    /// No internal Script Kit surface was active — deliver to the
    /// frontmost external app via simulated paste.
    ExternalApp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationSessionPhase {
    Idle,
    Recording,
    /// Escape pressed during recording — overlay shows Abort/Resume affordances.
    /// Escape again resumes recording; Enter or clicking Abort cancels the session.
    Confirming,
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

// --- Model availability ---

/// Whether the dictation engine's model is ready to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationModelStatus {
    /// Model files are present and ready.
    Available,
    /// Model is not downloaded yet.
    NotDownloaded,
    /// Model is currently being downloaded.
    Downloading {
        percentage: u8,
        /// Bytes downloaded so far (0 when unknown).
        downloaded_bytes: u64,
        /// Total expected bytes (0 when unknown).
        total_bytes: u64,
        /// Transfer speed in bytes/sec (0 when not yet measured).
        speed_bytes_per_sec: u64,
        /// Estimated seconds remaining, or `None` when not enough data exists yet.
        eta_seconds: Option<u64>,
    },
    /// Model is being extracted from the archive.
    Extracting,
    /// Download or extraction failed.
    DownloadFailed(String),
}
