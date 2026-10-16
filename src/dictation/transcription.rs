use crate::dictation::types::{CapturedAudioChunk, DictationDestination, DictationSessionResult};
use crate::setup::get_kit_path;
use anyhow::{Context as _, Result};
use parking_lot::Mutex;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Version-agnostic transcription engine trait.
///
/// Implementations wrap a specific speech-to-text backend. The rest of the
/// app depends only on this trait so that swapping engines does not ripple
/// beyond `transcription.rs`.
pub trait DictationEngine: Send {
    fn transcribe(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String>;

    /// Transcribe long audio in energy-split chunks, reporting fractional
    /// progress (0.0–1.0) after each chunk completes.
    ///
    /// Finalize routes recordings longer than [`LONG_AUDIO_CHUNK_TARGET_SECS`]
    /// through this path so the HUD gets real progress ticks and no single
    /// pass exceeds the range the models were trained on.
    fn transcribe_long(
        &mut self,
        samples: &[f32],
        initial_prompt: Option<&str>,
        progress: &mut dyn FnMut(f32),
    ) -> Result<String> {
        let chunks = plan_long_audio_chunks(samples, SAMPLE_RATE_HZ);
        let total = samples.len().max(1);
        let mut parts: Vec<String> = Vec::with_capacity(chunks.len());
        for range in chunks {
            let end = range.end;
            let text = self.transcribe(&samples[range], initial_prompt)?;
            let text = text.trim();
            if !text.is_empty() {
                parts.push(text.to_owned());
            }
            progress(end as f32 / total as f32);
        }
        progress(1.0);
        Ok(parts.join(" "))
    }
}

/// Target duration of one finalize chunk.
///
/// Every bundled model handles two minutes in one pass (Parakeet uses a
/// bounded attention window, Whisper has native long-form decoding, Cohere
/// accepts up to about 400 s), and a single pass avoids mid-sentence cuts
/// that capitalise or repeat words at chunk seams. Longer recordings are
/// still split so the HUD gets progress ticks and memory stays bounded.
pub const LONG_AUDIO_CHUNK_TARGET_SECS: f32 = 120.0;

/// Energy-split search window around the chunk target.
const LONG_AUDIO_SEARCH_WINDOW_SECS: f32 = 3.0;

/// Frame length used to find the quietest cut point (30 ms).
const CUT_FRAME_SECS: f32 = 0.03;

/// A trailing chunk shorter than this is merged into the previous one.
const MIN_TAIL_CHUNK_SECS: f32 = 0.5;

/// Audio shorter than one chunk-plus-search-window is transcribed in a
/// single plain pass; anything longer takes the chunked path.
pub fn should_use_chunked_transcription(sample_count: usize, sample_rate_hz: u32) -> bool {
    let secs = sample_count as f32 / sample_rate_hz.max(1) as f32;
    secs > LONG_AUDIO_CHUNK_TARGET_SECS + LONG_AUDIO_SEARCH_WINDOW_SECS
}

/// Split long audio into contiguous chunks of roughly
/// [`LONG_AUDIO_CHUNK_TARGET_SECS`], cutting at the quietest 30 ms frame
/// within ±[`LONG_AUDIO_SEARCH_WINDOW_SECS`] of each target so cuts land in
/// pauses rather than mid-word.
///
/// The returned ranges cover every sample exactly once, in order. Audio that
/// does not need chunking yields a single range.
pub fn plan_long_audio_chunks(samples: &[f32], sample_rate_hz: u32) -> Vec<Range<usize>> {
    let rate = sample_rate_hz.max(1) as f32;
    let len = samples.len();
    let target = (LONG_AUDIO_CHUNK_TARGET_SECS * rate) as usize;
    let window = (LONG_AUDIO_SEARCH_WINDOW_SECS * rate) as usize;
    let frame = ((CUT_FRAME_SECS * rate) as usize).max(1);
    let min_tail = (MIN_TAIL_CHUNK_SECS * rate) as usize;

    let mut chunks = Vec::new();
    let mut start = 0usize;
    while len - start > target + window {
        let search_start = start + target - window;
        let search_end = (start + target + window).min(len);
        let ideal = start + target;
        let mut best_cut = ideal;
        let mut best_energy = f32::MAX;
        let mut best_distance = usize::MAX;
        let mut frame_start = search_start;
        while frame_start + frame <= search_end {
            let energy = rms(&samples[frame_start..frame_start + frame]);
            let center = frame_start + frame / 2;
            let distance = center.abs_diff(ideal);
            if energy < best_energy || (energy == best_energy && distance < best_distance) {
                best_energy = energy;
                best_distance = distance;
                best_cut = center;
            }
            frame_start += frame;
        }
        chunks.push(start..best_cut);
        start = best_cut;
    }
    if len > start {
        let tail = start..len;
        match chunks.last_mut() {
            Some(previous) if tail.len() < min_tail => previous.end = len,
            _ => chunks.push(tail),
        }
    }
    if chunks.is_empty() {
        chunks.push(0..len);
    }
    chunks
}

/// Expected input sample rate for every bundled engine.
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

/// Parakeet TDT 0.6B v3 model file (GGUF, Q8_0), relative to the models directory.
const DEFAULT_PARAKEET_MODEL_FILE: &str = "parakeet-tdt-0.6b-v3-Q8_0.gguf";

/// URL for downloading the Parakeet model file.
pub const PARAKEET_MODEL_URL: &str =
    "https://huggingface.co/handy-computer/parakeet-tdt-0.6b-v3-gguf/resolve/main/parakeet-tdt-0.6b-v3-Q8_0.gguf";

/// Expected size of the Parakeet model file in bytes.
pub const PARAKEET_MODEL_SIZE: u64 = 739_508_576;

/// Expected SHA-256 of the Parakeet model file (lowercase hex).
pub const PARAKEET_MODEL_SHA256: &str =
    "5859f77944efcd8eafa23a6350731960b2b55b2203df51f319665c807d802cc7";

/// Whisper Medium model file (GGUF, Q5_K_M), relative to the models directory.
const DEFAULT_WHISPER_MODEL_FILE: &str = "whisper-medium-Q5_K_M.gguf";

/// URL for downloading the Whisper Medium model file.
pub const WHISPER_MODEL_URL: &str =
    "https://huggingface.co/handy-computer/whisper-medium-gguf/resolve/main/whisper-medium-Q5_K_M.gguf";

/// Expected size of the Whisper Medium model file in bytes.
pub const WHISPER_MODEL_SIZE: u64 = 582_746_048;

/// Expected SHA-256 of the Whisper Medium model file (lowercase hex).
pub const WHISPER_MODEL_SHA256: &str =
    "4e2a8904a866b3aa7ef70d7640ec6abc5f0a05524cd950ea4b66ace12122bf53";

/// Default RMS silence gate (mirrors `DictationPreferences::DEFAULT_SILENCE_RMS`).
pub const DEFAULT_SILENCE_RMS: f32 = 0.01;

/// Resolve the default Parakeet model file path.
///
/// Anchors the model path to `get_kit_path()/models/` so it works
/// regardless of the process working directory.
pub fn resolve_default_model_path() -> PathBuf {
    get_kit_path()
        .join("models")
        .join(DEFAULT_PARAKEET_MODEL_FILE)
}

/// Resolve the Whisper Medium model file path.
pub fn resolve_whisper_model_path() -> PathBuf {
    get_kit_path()
        .join("models")
        .join(DEFAULT_WHISPER_MODEL_FILE)
}

/// Returns `true` when the default Parakeet model file has been downloaded.
pub fn is_parakeet_model_available() -> bool {
    resolve_default_model_path().is_file()
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
// Local speech engine (transcribe.cpp / ggml, Metal GPU with CPU fallback)
// ---------------------------------------------------------------------------

/// Which compute backend the dictation model is running on. Logged on every
/// load and surfaced in the passive dictation automation state so agents can
/// confirm GPU use without starting a capture.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationEngineReceipt {
    /// Model file name (no directory).
    pub model_file: String,
    /// `true` while the model is resident in memory.
    pub loaded: bool,
    /// Backend the model is bound to, e.g. `"Metal"` or `"CPU"`.
    pub backend: String,
    /// Device description, e.g. `"Apple M4 Max"`.
    pub device: String,
    /// `true` when the model runs on a GPU device.
    pub gpu: bool,
    /// Why the engine is on the CPU instead of the GPU, when it fell back.
    pub cpu_fallback_reason: Option<String>,
    /// Wall time of the most recent model load.
    pub load_ms: u64,
    /// Wall time of the most recent transcription pass.
    pub last_transcribe_ms: Option<u64>,
    /// Audio length of the most recent transcription pass.
    pub last_audio_ms: Option<u64>,
}

static ENGINE_RECEIPT: Mutex<Option<DictationEngineReceipt>> = Mutex::new(None);

/// Latest engine receipt, or `None` when no model has been loaded this run.
pub fn dictation_engine_receipt() -> Option<DictationEngineReceipt> {
    ENGINE_RECEIPT.lock().clone()
}

fn update_engine_receipt(update: impl FnOnce(&mut DictationEngineReceipt)) {
    if let Some(receipt) = ENGINE_RECEIPT.lock().as_mut() {
        update(receipt);
    }
}

/// Route the native library's diagnostics into `log`/`tracing` once, so ggml
/// messages land in Script Kit's logs instead of raw stderr.
fn init_native_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(transcribe_cpp::init_logging);
}

struct LoadedSpeechModel {
    session: transcribe_cpp::Session,
    languages: Vec<String>,
    supports_prompt: bool,
    gpu: bool,
}

/// Local speech-to-text engine for GGUF models (Parakeet, Whisper) backed by
/// transcribe.cpp.
///
/// The model loads lazily on the first `transcribe()` call, on the GPU
/// (Metal) when available. If the GPU load or a GPU pass fails, the engine
/// reloads on the CPU and stays there for the rest of its life. Call
/// `unload()` (or drop the engine) to free model memory.
pub struct LocalDictationEngine {
    model_path: PathBuf,
    language: Option<String>,
    loaded: Option<LoadedSpeechModel>,
    cpu_fallback_reason: Option<String>,
}

impl std::fmt::Debug for LocalDictationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalDictationEngine")
            .field("model_path", &self.model_path)
            .field("language", &self.language)
            .field("model_loaded", &self.loaded.is_some())
            .field("cpu_fallback_reason", &self.cpu_fallback_reason)
            .finish()
    }
}

impl LocalDictationEngine {
    /// Create a new engine handle.
    ///
    /// Validates that `config.model_path` points to an existing regular file
    /// but defers the expensive model load to the first `transcribe()` call.
    pub fn new(config: &DictationTranscriptionConfig) -> Result<Self> {
        let path = &config.model_path;
        if !path.exists() {
            anyhow::bail!("Dictation model not found at {}", path.display());
        }
        if !path.is_file() {
            anyhow::bail!(
                "Dictation model path is not a regular file: {}",
                path.display()
            );
        }
        Ok(Self {
            model_path: path.clone(),
            language: config.language.clone(),
            loaded: None,
            cpu_fallback_reason: None,
        })
    }

    fn model_file_name(&self) -> String {
        self.model_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    fn load_with_backend(&self, backend: transcribe_cpp::Backend) -> Result<LoadedSpeechModel> {
        init_native_logging();
        let model = transcribe_cpp::Model::load_with(
            &self.model_path,
            &transcribe_cpp::ModelOptions {
                backend,
                ..Default::default()
            },
        )
        .with_context(|| {
            format!(
                "failed to load dictation model from {} ({backend:?})",
                self.model_path.display()
            )
        })?;
        let languages = model
            .capabilities()
            .map(|caps| caps.languages)
            .unwrap_or_default();
        let supports_prompt = model.supports(transcribe_cpp::Feature::ContextPrompt);
        let gpu = model
            .device()
            .map(|device| is_gpu_device_kind(&device.kind))
            .unwrap_or(false);
        let session = model
            .session()
            .context("failed to create dictation model session")?;
        Ok(LoadedSpeechModel {
            session,
            languages,
            supports_prompt,
            gpu,
        })
    }

    /// Lazily load the model: GPU first, CPU on failure.
    fn load_if_needed(&mut self) -> Result<&mut LoadedSpeechModel> {
        if self.loaded.is_none() {
            let started = Instant::now();
            let loaded = if self.cpu_fallback_reason.is_some() {
                self.load_with_backend(transcribe_cpp::Backend::Cpu)?
            } else {
                match self.load_with_backend(transcribe_cpp::Backend::Auto) {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        tracing::warn!(
                            category = "DICTATION",
                            error = %format!("{error:#}"),
                            "GPU model load failed; falling back to CPU"
                        );
                        self.cpu_fallback_reason = Some(format!("GPU load failed: {error:#}"));
                        self.load_with_backend(transcribe_cpp::Backend::Cpu)?
                    }
                }
            };
            let load_ms = started.elapsed().as_millis() as u64;
            let model = loaded.session.model();
            let backend = model.backend();
            let device = model
                .device()
                .map(|device| device.description)
                .unwrap_or_default();
            tracing::info!(
                category = "DICTATION",
                model_path = %self.model_path.display(),
                backend = %backend,
                device = %device,
                gpu = loaded.gpu,
                load_ms,
                cpu_fallback_reason = ?self.cpu_fallback_reason,
                "Dictation model loaded"
            );
            *ENGINE_RECEIPT.lock() = Some(DictationEngineReceipt {
                model_file: self.model_file_name(),
                loaded: true,
                backend,
                device,
                gpu: loaded.gpu,
                cpu_fallback_reason: self.cpu_fallback_reason.clone(),
                load_ms,
                last_transcribe_ms: None,
                last_audio_ms: None,
            });
            self.loaded = Some(loaded);
        }
        self.loaded
            .as_mut()
            .context("dictation model missing after load")
    }

    fn run_once(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String> {
        let requested_language = self.language.clone();
        let loaded = self.load_if_needed()?;
        let options = transcribe_cpp::RunOptions {
            language: supported_language(requested_language.as_deref(), &loaded.languages),
            prompt: if loaded.supports_prompt {
                initial_prompt.map(str::to_owned)
            } else {
                None
            },
            ..Default::default()
        };
        let transcript = loaded
            .session
            .run(samples, &options)
            .context("dictation transcription failed")?;
        Ok(transcript.text.trim().to_owned())
    }

    /// Unload the model, freeing memory.
    pub fn unload(&mut self) {
        if self.loaded.take().is_some() {
            update_engine_receipt(|receipt| receipt.loaded = false);
            tracing::info!(
                category = "DICTATION",
                model_path = %self.model_path.display(),
                "Dictation model unloaded"
            );
        }
    }
}

impl Drop for LocalDictationEngine {
    fn drop(&mut self) {
        self.unload();
    }
}

impl DictationEngine for LocalDictationEngine {
    fn transcribe(&mut self, samples: &[f32], initial_prompt: Option<&str>) -> Result<String> {
        let started = Instant::now();
        let text = match self.run_once(samples, initial_prompt) {
            Ok(text) => text,
            Err(error) if self.loaded.as_ref().is_some_and(|loaded| loaded.gpu) => {
                tracing::warn!(
                    category = "DICTATION",
                    error = %format!("{error:#}"),
                    "GPU transcription failed; reloading on CPU and retrying once"
                );
                self.unload();
                self.cpu_fallback_reason = Some(format!("GPU transcription failed: {error:#}"));
                self.run_once(samples, initial_prompt)?
            }
            Err(error) => return Err(error),
        };
        let elapsed_ms = started.elapsed().as_millis() as u64;
        let audio_ms = (samples.len() as u64 * 1000) / u64::from(SAMPLE_RATE_HZ);
        update_engine_receipt(|receipt| {
            receipt.last_transcribe_ms = Some(elapsed_ms);
            receipt.last_audio_ms = Some(audio_ms);
        });
        tracing::debug!(
            category = "DICTATION",
            samples = samples.len(),
            audio_ms,
            elapsed_ms,
            "Dictation transcription pass complete"
        );
        Ok(text)
    }
}

/// `true` for device kinds that run on a GPU rather than the host CPU.
pub(crate) fn is_gpu_device_kind(kind: &str) -> bool {
    !matches!(kind, "cpu" | "accel" | "unknown" | "")
}

/// Pass the language hint only when the model lists it; models without a
/// language list (or that don't support the requested code) auto-detect.
pub(crate) fn supported_language(requested: Option<&str>, supported: &[String]) -> Option<String> {
    let requested = requested?.trim();
    if requested.is_empty() || requested.eq_ignore_ascii_case("auto") {
        return None;
    }
    supported
        .iter()
        .find(|code| code.eq_ignore_ascii_case(requested))
        .cloned()
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
