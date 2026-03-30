use crate::dictation::types::{CapturedAudioChunk, DictationDestination, DictationSessionResult};
use anyhow::Result;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

impl Default for DictationTranscriptionConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/whisper-medium-q4_1.bin"),
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
        if samples.len() < self.config.minimum_samples || rms(samples) < 0.01 {
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
// Concrete Whisper engine
// ---------------------------------------------------------------------------

/// Configuration required to load a Whisper model from disk.
///
/// When `WhisperDictationEngine::new()` is called, the engine validates that
/// the model file exists and is readable.  Actual model loading is deferred
/// to the first `transcribe()` call in production; for now we validate early
/// so callers get a meaningful error at construction time.
#[derive(Debug)]
pub struct WhisperDictationEngine {
    model_path: PathBuf,
    /// Whether the model has been lazily loaded. In this scaffold the flag is
    /// set once the model path is validated — a real integration would store
    /// the loaded `whisper_rs::WhisperContext` here.
    loaded: bool,
}

impl WhisperDictationEngine {
    /// Create a new Whisper engine.
    ///
    /// Fails immediately if `config.model_path` does not point to an existing
    /// regular file.
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
            loaded: false,
        })
    }

    /// Unload the model, freeing memory.
    pub fn unload(&mut self) {
        self.loaded = false;
        tracing::info!(
            category = "DICTATION",
            model_path = %self.model_path.display(),
            "Whisper model unloaded"
        );
    }
}

impl DictationEngine for WhisperDictationEngine {
    fn transcribe(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String> {
        if !self.loaded {
            tracing::info!(
                category = "DICTATION",
                model_path = %self.model_path.display(),
                "Loading Whisper model"
            );
            // TODO: integrate transcribe-rs WhisperContext::new() here once
            // the whisper-rs dependency is added.  For now we mark as loaded
            // and return a placeholder so the pipeline is exercised end-to-end.
            self.loaded = true;
        }

        tracing::debug!(
            category = "DICTATION",
            samples = samples.len(),
            initial_prompt = ?initial_prompt,
            "Whisper transcription requested"
        );

        // Placeholder: real Whisper inference will replace this.
        // The pipeline (capture → merge → transcribe → deliver) is fully wired;
        // swapping in the real engine is a single-file change once the dep lands.
        Ok(String::new())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
