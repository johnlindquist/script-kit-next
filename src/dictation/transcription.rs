use crate::dictation::types::{CapturedAudioChunk, DictationDestination, DictationSessionResult};
use crate::setup::get_kit_path;
use anyhow::{Context as _, Result};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use transcribe_rs::whisper_cpp::{
    WhisperEngine as TranscribeWhisperEngine, WhisperInferenceParams,
};

/// Version-agnostic transcription engine trait.
///
/// Implementations wrap a specific speech-to-text backend (e.g. Whisper via
/// `transcribe-rs`).  The rest of the app depends only on this trait so that
/// swapping engines does not ripple beyond `transcription.rs`.
pub trait DictationEngine: Send {
    fn transcribe(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationTranscriptionConfig {
    pub model_path: PathBuf,
    pub initial_prompt: Option<String>,
    pub idle_unload_after: Duration,
    /// Minimum number of 16 kHz mono samples required before attempting
    /// transcription.  Shorter clips are treated as silence.
    pub minimum_samples: usize,
}

/// Default Whisper model filename (relative to the models directory).
const DEFAULT_WHISPER_MODEL: &str = "whisper-medium-q4_1.bin";

/// Resolve the default Whisper model path.
///
/// Anchors the model path to `get_kit_path()/models/` so it works
/// regardless of the process working directory.  The returned path is
/// always absolute (assuming `get_kit_path()` returns an absolute path,
/// which it does for every documented configuration).
pub fn resolve_default_model_path() -> PathBuf {
    get_kit_path().join("models").join(DEFAULT_WHISPER_MODEL)
}

impl Default for DictationTranscriptionConfig {
    fn default() -> Self {
        Self {
            model_path: resolve_default_model_path(),
            initial_prompt: None,
            idle_unload_after: Duration::from_secs(300),
            // 100 ms at 16 kHz
            minimum_samples: 1_600,
        }
    }
}

pub struct DictationTranscriber {
    config: DictationTranscriptionConfig,
    engine: Mutex<Box<dyn DictationEngine>>,
    last_used_at: Mutex<Instant>,
}

impl DictationTranscriber {
    pub fn new(config: DictationTranscriptionConfig, engine: Box<dyn DictationEngine>) -> Self {
        Self {
            config,
            engine: Mutex::new(engine),
            last_used_at: Mutex::new(Instant::now()),
        }
    }

    pub fn model_path(&self) -> &Path {
        &self.config.model_path
    }

    /// Merge captured chunks and transcribe.  Returns `Ok(None)` when the
    /// audio is too short or silent.
    pub fn transcribe_chunks(&self, chunks: &[CapturedAudioChunk]) -> Result<Option<String>> {
        let samples = merge_captured_chunks(chunks);
        self.transcribe_samples(&samples)
    }

    /// Transcribe raw 16 kHz mono samples.  Returns `Ok(None)` when the audio
    /// is below the minimum sample count or energy threshold.
    pub fn transcribe_samples(&self, samples: &[f32]) -> Result<Option<String>> {
        if should_skip_transcription(&self.config, samples) {
            return Ok(None);
        }

        let text = self
            .engine
            .lock()
            .transcribe(samples, self.config.initial_prompt.as_deref())?;

        *self.last_used_at.lock() = Instant::now();

        let trimmed = text.trim().to_owned();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    }

    /// Returns `true` when the engine has not been used for longer than the
    /// configured idle timeout.
    pub fn is_idle(&self) -> bool {
        self.last_used_at.lock().elapsed() >= self.config.idle_unload_after
    }
}

// ---------------------------------------------------------------------------
// Chunk helpers
// ---------------------------------------------------------------------------

/// Concatenate captured chunks into a single contiguous sample buffer.
pub fn merge_captured_chunks(chunks: &[CapturedAudioChunk]) -> Vec<f32> {
    let total_samples: usize = chunks.iter().map(|c| c.samples.len()).sum();
    let mut merged = Vec::with_capacity(total_samples);
    for chunk in chunks {
        merged.extend_from_slice(&chunk.samples);
    }
    merged
}

/// Total audio duration across captured chunks.
pub fn captured_duration(chunks: &[CapturedAudioChunk]) -> Duration {
    chunks
        .iter()
        .fold(Duration::ZERO, |acc, chunk| acc + chunk.duration)
}

/// Build a session result from completed transcription.
pub fn build_session_result(
    chunks: &[CapturedAudioChunk],
    destination: DictationDestination,
    transcript: String,
) -> DictationSessionResult {
    DictationSessionResult {
        transcript,
        destination,
        audio_duration: captured_duration(chunks),
    }
}

// ---------------------------------------------------------------------------
// Concrete Whisper engine (transcribe-rs / whisper.cpp backend)
// ---------------------------------------------------------------------------

/// Local Whisper speech-to-text engine backed by `transcribe-rs`.
///
/// The underlying `TranscribeWhisperEngine` is loaded lazily on the first
/// `transcribe()` call and cached for subsequent invocations.  Call
/// `unload()` to free the model memory.
pub struct WhisperDictationEngine {
    model_path: PathBuf,
    engine: Option<TranscribeWhisperEngine>,
}

impl std::fmt::Debug for WhisperDictationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhisperDictationEngine")
            .field("model_path", &self.model_path)
            .field("engine_loaded", &self.engine.is_some())
            .finish()
    }
}

impl WhisperDictationEngine {
    /// Create a new Whisper engine handle.
    ///
    /// Validates that `config.model_path` points to an existing regular file
    /// but defers the expensive model load to the first `transcribe()` call.
    pub fn new(config: &DictationTranscriptionConfig) -> anyhow::Result<Self> {
        let path = &config.model_path;
        if !path.exists() {
            anyhow::bail!("Whisper model not found at {}", path.display());
        }
        if !path.is_file() {
            anyhow::bail!(
                "Whisper model path is not a regular file: {}",
                path.display()
            );
        }
        Ok(Self {
            model_path: path.clone(),
            engine: None,
        })
    }

    /// Lazily load the Whisper model, returning a mutable reference to the
    /// live engine.
    fn load_if_needed(&mut self) -> Result<&mut TranscribeWhisperEngine> {
        if self.engine.is_none() {
            tracing::info!(
                category = "DICTATION",
                model_path = %self.model_path.display(),
                "Loading Whisper model"
            );
            self.engine = Some(
                TranscribeWhisperEngine::load(&self.model_path).with_context(|| {
                    format!(
                        "failed to load Whisper model from {}",
                        self.model_path.display()
                    )
                })?,
            );
        }
        self.engine
            .as_mut()
            .context("whisper engine missing after load")
    }

    /// Unload the model, freeing memory.
    pub fn unload(&mut self) {
        self.engine = None;
        tracing::info!(
            category = "DICTATION",
            model_path = %self.model_path.display(),
            "Whisper model unloaded"
        );
    }
}

impl DictationEngine for WhisperDictationEngine {
    fn transcribe(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String> {
        let engine = self.load_if_needed()?;

        tracing::debug!(
            category = "DICTATION",
            samples = samples.len(),
            initial_prompt = ?initial_prompt,
            "Whisper transcription requested"
        );

        let result = engine
            .transcribe_with(
                samples,
                &WhisperInferenceParams {
                    initial_prompt: initial_prompt.map(str::to_owned),
                    ..Default::default()
                },
            )
            .context("whisper transcription failed")?;

        Ok(result.text.trim().to_owned())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

pub(crate) fn should_skip_transcription(
    config: &DictationTranscriptionConfig,
    samples: &[f32],
) -> bool {
    samples.len() < config.minimum_samples || rms(samples) < 0.01
}

pub(crate) fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
