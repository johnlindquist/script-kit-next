use crate::dictation::types::{CapturedAudioChunk, DictationDestination, DictationSessionResult};
use crate::setup::get_kit_path;
use anyhow::{Context as _, Result};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use transcribe_rs::onnx::parakeet::ParakeetModel;
use transcribe_rs::onnx::Quantization;
use transcribe_rs::transcriber::{EnergyAdaptiveChunked, EnergyAdaptiveConfig, Transcriber as _};
use transcribe_rs::whisper_cpp::{
    WhisperEngine as TranscribeWhisperEngine, WhisperInferenceParams,
};
use transcribe_rs::{SpeechModel, TranscribeOptions};

/// Version-agnostic transcription engine trait.
///
/// Implementations wrap a specific speech-to-text backend (e.g. Whisper via
/// `transcribe-rs`).  The rest of the app depends only on this trait so that
/// swapping engines does not ripple beyond `transcription.rs`.
pub trait DictationEngine: Send {
    fn transcribe(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String>;

    /// Transcribe long audio in energy-split chunks, reporting fractional
    /// progress (0.0–1.0) after each chunk completes.
    ///
    /// Both bundled backends degrade on very long single passes — Parakeet's
    /// conformer encoder is O(T²) over the whole buffer and can drop trailing
    /// speech, and Whisper's internal windowing gives no progress signal — so
    /// finalize routes recordings longer than
    /// [`LONG_AUDIO_CHUNK_TARGET_SECS`] through this path instead.
    fn transcribe_long(
        &mut self,
        samples: &[f32],
        initial_prompt: Option<&str>,
        progress: &mut dyn FnMut(f32),
    ) -> Result<String> {
        let _ = progress;
        self.transcribe(samples, initial_prompt)
    }
}

/// Target duration of one finalize chunk. Chosen to sit inside Whisper's
/// native 30 s window and well inside the range Parakeet was trained on.
pub const LONG_AUDIO_CHUNK_TARGET_SECS: f32 = 24.0;

/// Energy-split search window around the chunk target.
const LONG_AUDIO_SEARCH_WINDOW_SECS: f32 = 3.0;

/// Audio shorter than one chunk-plus-search-window is transcribed in a
/// single plain pass; anything longer takes the chunked path.
pub fn should_use_chunked_transcription(sample_count: usize, sample_rate_hz: u32) -> bool {
    let secs = sample_count as f32 / sample_rate_hz.max(1) as f32;
    secs > LONG_AUDIO_CHUNK_TARGET_SECS + LONG_AUDIO_SEARCH_WINDOW_SECS
}

/// Run the shared energy-adaptive chunked transcription over any
/// `SpeechModel`, reporting progress as the fraction of audio samples whose
/// chunks have finished transcribing.
fn transcribe_long_with_model(
    model: &mut dyn SpeechModel,
    samples: &[f32],
    language: Option<String>,
    progress: &mut dyn FnMut(f32),
) -> Result<String> {
    let options = TranscribeOptions {
        language,
        ..Default::default()
    };
    let mut chunker = EnergyAdaptiveChunked::new(
        EnergyAdaptiveConfig {
            target_chunk_secs: LONG_AUDIO_CHUNK_TARGET_SECS,
            search_window_secs: LONG_AUDIO_SEARCH_WINDOW_SECS,
            padding_secs: 0.3,
            min_chunk_secs: 0.5,
            ..Default::default()
        },
        options,
    );

    // Feed in chunk-sized slices so a progress tick lands after every chunk
    // the underlying model completes.
    let slice_len = ((LONG_AUDIO_CHUNK_TARGET_SECS * SAMPLE_RATE_HZ as f32) as usize).max(1);
    let total = samples.len().max(1);
    let mut fed = 0usize;
    for slice in samples.chunks(slice_len) {
        chunker
            .feed(model, slice)
            .map_err(|e| anyhow::anyhow!("chunked transcription failed: {e}"))?;
        fed += slice.len();
        // feed() may hold up to one chunk buffered; report the fed fraction
        // scaled down so 1.0 is reserved for finish().
        progress((fed as f32 / total as f32) * 0.95);
    }
    // finish() transcribes the remainder and merges every chunk result into
    // the authoritative transcript.
    let merged = chunker
        .finish(model)
        .map_err(|e| anyhow::anyhow!("chunked transcription finalize failed: {e}"))?;
    progress(1.0);
    Ok(merged.text.trim().to_owned())
}

/// Expected input sample rate for both bundled engines.
const SAMPLE_RATE_HZ: u32 = 16_000;

#[derive(Debug, Clone, PartialEq)]
pub struct DictationTranscriptionConfig {
    pub model_path: PathBuf,
    pub initial_prompt: Option<String>,
    pub language: Option<String>,
    pub idle_unload_after: Duration,
    /// Minimum number of 16 kHz mono samples required before attempting
    /// transcription.  Shorter clips are treated as silence.
    pub minimum_samples: usize,
    /// RMS energy threshold below which audio is treated as silence.
    pub silence_rms: f32,
}

/// Default Whisper model filename (relative to the models directory).
const DEFAULT_WHISPER_MODEL: &str = "whisper-medium-q4_1.bin";

/// Default Parakeet model directory name (relative to the models directory).
const DEFAULT_PARAKEET_MODEL_DIR: &str = "parakeet-tdt-0.6b-v3-int8";

/// URL for downloading the Parakeet model archive.
pub const PARAKEET_MODEL_URL: &str = "https://blob.handy.computer/parakeet-v3-int8.tar.gz";

/// Expected size of the Parakeet model archive in bytes.
pub const PARAKEET_MODEL_ARCHIVE_SIZE: u64 = 478_517_071;

/// URL for downloading the Whisper Medium (q4_1) model file.
///
/// Served from the same blob host as the Parakeet archive and named exactly
/// like the local model file, so no extraction or renaming is needed.
pub const WHISPER_MODEL_URL: &str = "https://blob.handy.computer/whisper-medium-q4_1.bin";

/// Expected size of the Whisper Medium (q4_1) model file in bytes.
pub const WHISPER_MODEL_SIZE: u64 = 491_852_915;

/// Default RMS silence gate (mirrors `DictationPreferences::DEFAULT_SILENCE_RMS`).
pub const DEFAULT_SILENCE_RMS: f32 = 0.01;

/// Resolve the default Parakeet model directory path.
///
/// Anchors the model path to `get_kit_path()/models/` so it works
/// regardless of the process working directory.
pub fn resolve_default_model_path() -> PathBuf {
    get_kit_path()
        .join("models")
        .join(DEFAULT_PARAKEET_MODEL_DIR)
}

/// Resolve the legacy Whisper model file path.
pub fn resolve_whisper_model_path() -> PathBuf {
    get_kit_path().join("models").join(DEFAULT_WHISPER_MODEL)
}

/// Returns `true` when the default Parakeet model directory exists and
/// contains at least one file (i.e. extraction completed).
pub fn is_parakeet_model_available() -> bool {
    let path = resolve_default_model_path();
    path.is_dir()
        && std::fs::read_dir(&path)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
}

impl Default for DictationTranscriptionConfig {
    fn default() -> Self {
        Self {
            model_path: resolve_default_model_path(),
            initial_prompt: None,
            language: None,
            idle_unload_after: Duration::from_secs(300),
            // 100 ms at 16 kHz
            minimum_samples: 1_600,
            silence_rms: DEFAULT_SILENCE_RMS,
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
        self.transcribe_samples_with_progress(samples, &mut |_| {})
    }

    /// Transcribe raw 16 kHz mono samples, reporting fractional progress.
    ///
    /// Audio longer than one chunk window takes the energy-adaptive chunked
    /// path (linear-time on long recordings, full-buffer coverage, real
    /// progress ticks); short audio runs a single plain pass with a final
    /// 1.0 tick.
    pub fn transcribe_samples_with_progress(
        &self,
        samples: &[f32],
        progress: &mut dyn FnMut(f32),
    ) -> Result<Option<String>> {
        if should_skip_transcription(&self.config, samples) {
            return Ok(None);
        }

        let text = {
            let mut engine = self.engine.lock();
            if should_use_chunked_transcription(samples.len(), SAMPLE_RATE_HZ) {
                engine.transcribe_long(samples, self.config.initial_prompt.as_deref(), progress)?
            } else {
                let text = engine.transcribe(samples, self.config.initial_prompt.as_deref())?;
                progress(1.0);
                text
            }
        };

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
    language: Option<String>,
    engine: Option<TranscribeWhisperEngine>,
}

impl std::fmt::Debug for WhisperDictationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhisperDictationEngine")
            .field("model_path", &self.model_path)
            .field("language", &self.language)
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
            language: config.language.clone(),
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
        let language = self.language.clone();
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
                    language,
                    ..Default::default()
                },
            )
            .context("whisper transcription failed")?;

        Ok(result.text.trim().to_owned())
    }

    fn transcribe_long(
        &mut self,
        samples: &[f32],
        _initial_prompt: Option<&str>,
        progress: &mut dyn FnMut(f32),
    ) -> Result<String> {
        let language = self.language.clone();
        let engine = self.load_if_needed()?;
        transcribe_long_with_model(engine, samples, language, progress)
    }
}

// ---------------------------------------------------------------------------
// Concrete Parakeet engine (ONNX via transcribe-rs)
// ---------------------------------------------------------------------------

/// Local Parakeet speech-to-text engine backed by ONNX Runtime via
/// `transcribe-rs`.
///
/// The underlying `ParakeetModel` is loaded lazily on the first
/// `transcribe()` call.  Call `unload()` to free model memory.
pub struct ParakeetDictationEngine {
    model_dir: PathBuf,
    model: Option<ParakeetModel>,
}

impl std::fmt::Debug for ParakeetDictationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParakeetDictationEngine")
            .field("model_dir", &self.model_dir)
            .field("model_loaded", &self.model.is_some())
            .finish()
    }
}

impl ParakeetDictationEngine {
    /// Create a new Parakeet engine handle.
    ///
    /// Validates that `model_dir` points to an existing directory but defers
    /// the expensive model load to the first `transcribe()` call.
    pub fn new(model_dir: &Path) -> anyhow::Result<Self> {
        if !model_dir.exists() {
            anyhow::bail!("Parakeet model not found at {}", model_dir.display());
        }
        if !model_dir.is_dir() {
            anyhow::bail!(
                "Parakeet model path is not a directory: {}",
                model_dir.display()
            );
        }
        Ok(Self {
            model_dir: model_dir.to_path_buf(),
            model: None,
        })
    }

    /// Lazily load the Parakeet ONNX model.
    fn load_if_needed(&mut self) -> Result<&mut ParakeetModel> {
        if self.model.is_none() {
            tracing::info!(
                category = "DICTATION",
                model_dir = %self.model_dir.display(),
                "Loading Parakeet ONNX model (INT8)"
            );
            let loaded = ParakeetModel::load(&self.model_dir, &Quantization::Int8)
                .map_err(|e| anyhow::anyhow!("failed to load Parakeet model: {e}"))?;
            self.model = Some(loaded);
        }
        self.model
            .as_mut()
            .context("parakeet model missing after load")
    }

    /// Unload the model, freeing memory.
    pub fn unload(&mut self) {
        self.model = None;
        tracing::info!(
            category = "DICTATION",
            model_dir = %self.model_dir.display(),
            "Parakeet model unloaded"
        );
    }
}

impl DictationEngine for ParakeetDictationEngine {
    fn transcribe(&mut self, samples: &[f32], _initial_prompt: Option<&str>) -> Result<String> {
        let model = self.load_if_needed()?;

        tracing::debug!(
            category = "DICTATION",
            samples = samples.len(),
            "Parakeet transcription requested"
        );

        let options = TranscribeOptions::default();
        let result = model
            .transcribe(samples, &options)
            .map_err(|e| anyhow::anyhow!("parakeet transcription failed: {e}"))?;

        Ok(result.text.trim().to_owned())
    }

    fn transcribe_long(
        &mut self,
        samples: &[f32],
        _initial_prompt: Option<&str>,
        progress: &mut dyn FnMut(f32),
    ) -> Result<String> {
        let model = self.load_if_needed()?;
        transcribe_long_with_model(model, samples, None, progress)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

pub(crate) fn should_skip_transcription(
    config: &DictationTranscriptionConfig,
    samples: &[f32],
) -> bool {
    samples.len() < config.minimum_samples || rms(samples) < config.silence_rms
}

pub(crate) fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
