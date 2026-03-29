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
// Internal helpers
// ---------------------------------------------------------------------------

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
