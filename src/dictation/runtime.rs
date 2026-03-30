use crate::dictation::capture::{start_capture, DictationCaptureHandle};
use crate::dictation::device::{list_input_devices, resolve_selected_input_device};
use crate::dictation::transcription::{
    captured_duration, is_parakeet_model_available, merge_captured_chunks,
    should_skip_transcription, DictationTranscriber, DictationTranscriptionConfig,
    ParakeetDictationEngine,
};
use crate::dictation::types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDeviceId, DictationLevel, DictationSessionPhase, DictationToggleOutcome,
};
use crate::dictation::visualizer::bars_for_level;
use crate::dictation::window::DictationOverlayState;
use anyhow::{Context, Result};
use gpui::SharedString;
use parking_lot::Mutex;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Session state
// ---------------------------------------------------------------------------

/// Live state of a dictation recording session.
struct DictationSession {
    capture_handle: Option<DictationCaptureHandle>,
    event_rx: async_channel::Receiver<DictationCaptureEvent>,
    chunks: Vec<CapturedAudioChunk>,
    last_level: DictationLevel,
    started_at: Instant,
    /// The authoritative overlay phase — written by both the runtime (on start)
    /// and the overlay key handler (on Escape transitions).  The pump reads this
    /// on every tick so the overlay never drifts from shared state.
    overlay_phase: DictationSessionPhase,
}

/// Global singleton guarded by a parking_lot Mutex.
///
/// `None` means idle (no recording in progress).
static SESSION: Mutex<Option<DictationSession>> = Mutex::new(None);

/// Lazily-initialized transcriber, kept alive across sessions so the model
/// does not need to be reloaded for every dictation.  Unloaded after an idle
/// timeout via `maybe_unload_transcriber()`.
static TRANSCRIBER: Mutex<Option<DictationTranscriber>> = Mutex::new(None);

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns `true` when a dictation capture session is currently active.
pub fn is_dictation_recording() -> bool {
    SESSION.lock().is_some()
}

/// Update the overlay phase in the live session.
///
/// Called by the overlay key handler so the pump tick reads the correct phase
/// instead of overwriting it.  Returns `false` when no session is active.
pub fn set_overlay_phase(phase: DictationSessionPhase) -> bool {
    let mut guard = SESSION.lock();
    let Some(session) = guard.as_mut() else {
        return false;
    };
    session.overlay_phase = phase;
    true
}

/// Toggle dictation recording on/off.
///
/// - When idle: starts a new capture session and returns `Started`.
/// - When recording: drops the capture handle (flushing the tail chunk),
///   drains events until `EndOfStream`, and returns `Stopped(Some(capture))`
///   or `Stopped(None)` for an empty recording.
///
/// This is the single entrypoint shared by both `BuiltInFeature::Dictation`
/// and `HotkeyAction::Dictation`.  The caller owns transcription, overlay
/// transitions, and delivery — the runtime only captures audio.
pub fn toggle_dictation() -> Result<DictationToggleOutcome> {
    tracing::info!(category = "DICTATION", "toggle_dictation called");

    if SESSION.lock().is_some() {
        return Ok(DictationToggleOutcome::Stopped(stop_recording()?));
    }

    start_recording()?;
    Ok(DictationToggleOutcome::Started)
}

/// Abort the active dictation session without transcribing or delivering text.
///
/// This is used by the overlay's Escape confirmation flow, where the user has
/// explicitly chosen to discard the current recording.
pub fn abort_dictation() -> Result<()> {
    if SESSION.lock().is_none() {
        return Ok(());
    }

    let _ = stop_recording()?;
    tracing::info!(category = "DICTATION", "Recording aborted");
    Ok(())
}

/// Snapshot the current overlay visual state from the live session.
///
/// Drains pending events (non-blocking) so the level/chunk data stays
/// current.  Returns `None` when no recording is active.
pub fn snapshot_overlay_state() -> Option<DictationOverlayState> {
    let mut guard = SESSION.lock();
    let session = guard.as_mut()?;

    // Non-blocking drain of queued events.
    while let Ok(event) = session.event_rx.try_recv() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => session.chunks.push(chunk),
            DictationCaptureEvent::Level(level) => session.last_level = level,
            DictationCaptureEvent::EndOfStream => {}
        }
    }

    Some(DictationOverlayState {
        phase: session.overlay_phase.clone(),
        elapsed: session.started_at.elapsed(),
        bars: bars_for_level(session.last_level),
        transcript: SharedString::default(),
    })
}

/// Transcribe previously captured audio chunks.
///
/// Lazily initialises the Whisper engine on first use and caches it for
/// subsequent calls.  Returns `Ok(None)` when the audio is too short or
/// silent.
pub fn transcribe_captured_audio(chunks: &[CapturedAudioChunk]) -> Result<Option<String>> {
    let config = DictationTranscriptionConfig::default();
    let audio_duration = captured_duration(chunks);
    let samples = merge_captured_chunks(chunks);
    let rms = crate::dictation::transcription::rms(&samples);

    tracing::info!(
        category = "DICTATION",
        chunk_count = chunks.len(),
        audio_duration_ms = audio_duration.as_millis() as u64,
        sample_count = samples.len(),
        rms,
        minimum_samples = config.minimum_samples,
        model_path = %config.model_path.display(),
        model_exists = config.model_path.exists(),
        model_is_file = config.model_path.is_file(),
        "Starting dictation transcription"
    );

    if samples.len() < config.minimum_samples {
        tracing::info!(
            category = "DICTATION",
            sample_count = samples.len(),
            minimum_samples = config.minimum_samples,
            "Skipping dictation transcription: audio too short"
        );
        return Ok(None);
    }

    if rms < 0.01 {
        tracing::info!(
            category = "DICTATION",
            rms,
            threshold = 0.01_f32,
            "Skipping dictation transcription: audio too silent"
        );
        return Ok(None);
    }

    let mut guard = TRANSCRIBER.lock();

    let should_rebuild = guard
        .as_ref()
        .map(DictationTranscriber::is_idle)
        .unwrap_or(true);

    tracing::debug!(
        category = "DICTATION",
        has_cached_transcriber = guard.is_some(),
        should_rebuild,
        "Resolving dictation transcriber"
    );

    if should_rebuild {
        if !is_parakeet_model_available() {
            anyhow::bail!(
                "Parakeet model not downloaded. Use the dictation settings to download it."
            );
        }
        tracing::info!(
            category = "DICTATION",
            model_path = %config.model_path.display(),
            "Initializing Parakeet ONNX dictation engine"
        );
        let engine: Box<dyn crate::dictation::transcription::DictationEngine> = Box::new(
            ParakeetDictationEngine::new(&config.model_path).with_context(|| {
                format!(
                    "failed to initialize Parakeet engine from {}",
                    config.model_path.display()
                )
            })?,
        );
        *guard = Some(DictationTranscriber::new(config.clone(), engine));
    }

    let result = guard
        .as_ref()
        .context("dictation transcriber unavailable")?
        .transcribe_samples(&samples);

    match &result {
        Ok(Some(transcript)) => tracing::info!(
            category = "DICTATION",
            transcript_len = transcript.len(),
            "Dictation transcription succeeded"
        ),
        Ok(None) => tracing::info!(
            category = "DICTATION",
            "Dictation transcription completed without text"
        ),
        Err(error) => tracing::error!(
            category = "DICTATION",
            error = %error,
            "Dictation transcription failed"
        ),
    }

    result
}

/// Unload the cached transcriber if it has been idle for longer than its
/// configured timeout.  Intended to be called on a timer from the app layer.
pub fn maybe_unload_transcriber() {
    let mut guard = TRANSCRIBER.lock();
    if guard
        .as_ref()
        .map(DictationTranscriber::is_idle)
        .unwrap_or(false)
    {
        *guard = None;
    }
}

#[cfg(test)]
pub(crate) fn reset_cached_transcriber_for_tests() {
    *TRANSCRIBER.lock() = None;
}

// ---------------------------------------------------------------------------
// Start recording
// ---------------------------------------------------------------------------

fn start_recording() -> Result<()> {
    let device_id = resolve_preferred_device()?;
    let capture_config = DictationCaptureConfig::default();

    let (event_rx, capture_handle) = start_capture(capture_config, device_id.as_ref())
        .context("failed to start audio capture")?;

    let session = DictationSession {
        capture_handle: Some(capture_handle),
        event_rx,
        chunks: Vec::new(),
        last_level: DictationLevel {
            rms: 0.0,
            peak: 0.0,
        },
        started_at: Instant::now(),
        overlay_phase: DictationSessionPhase::Recording,
    };

    *SESSION.lock() = Some(session);

    tracing::info!(
        category = "DICTATION",
        device = ?device_id,
        "Recording started"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Stop recording
// ---------------------------------------------------------------------------

/// Stop the current recording and return the completed capture.
///
/// **Important**: the capture handle is dropped *before* draining the event
/// channel.  This ensures the AVFoundation session stops, the processor
/// thread flushes the tail chunk, and `EndOfStream` is sent before we begin
/// collecting.  Without this ordering, `try_recv()` would miss the tail.
fn stop_recording() -> Result<Option<CompletedDictationCapture>> {
    let session = SESSION.lock().take();
    let Some(mut session) = session else {
        tracing::warn!(
            category = "DICTATION",
            "stop_recording called but no session active"
        );
        return Ok(None);
    };

    stop_capture_and_collect(&mut session)?;

    let audio_duration = captured_duration(&session.chunks);
    tracing::info!(
        category = "DICTATION",
        chunks = session.chunks.len(),
        audio_duration_ms = audio_duration.as_millis() as u64,
        "Recording stopped"
    );

    if session.chunks.is_empty() {
        return Ok(None);
    }

    Ok(Some(CompletedDictationCapture {
        chunks: session.chunks,
        audio_duration,
    }))
}

/// Drop the capture handle first so the processor thread can flush its tail
/// chunk and send `EndOfStream`, then blocking-drain the channel.
fn stop_capture_and_collect(session: &mut DictationSession) -> Result<()> {
    // Drop capture handle — triggers AVCaptureSession stop, processor thread
    // flush, and EndOfStream emission.
    let _ = session.capture_handle.take();

    // Blocking drain until EndOfStream.
    while let Ok(event) = session.event_rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => session.chunks.push(chunk),
            DictationCaptureEvent::Level(level) => session.last_level = level,
            DictationCaptureEvent::EndOfStream => return Ok(()),
        }
    }

    anyhow::bail!("dictation capture stream closed before EndOfStream")
}

// ---------------------------------------------------------------------------
// Mic preference resolution
// ---------------------------------------------------------------------------

/// Resolve the user's preferred microphone device.
///
/// Reads `ScriptKitUserPreferences.dictation.selected_device_id` and, if the
/// device is still present, returns its ID.  Falls back to the system default
/// when the preference is unset or the device has disappeared.
///
/// Delegates the pure selection logic to [`resolve_selected_input_device`] so
/// the behavior is deterministic and testable without I/O.
pub(crate) fn resolve_preferred_device() -> Result<Option<DictationDeviceId>> {
    let prefs = crate::config::load_user_preferences();
    let preferred_id = prefs.dictation.selected_device_id.clone();

    let devices = list_input_devices()?;
    let selected = resolve_selected_input_device(&devices, preferred_id.as_deref());

    match (preferred_id.as_deref(), &selected) {
        (Some(saved_id), Some(device)) if device.id.0 == saved_id => {
            tracing::info!(
                category = "DICTATION",
                device_id = %device.id.0,
                device_name = %device.name,
                "Using saved microphone preference"
            );
        }
        (Some(saved_id), Some(device)) => {
            tracing::warn!(
                category = "DICTATION",
                missing_device_id = %saved_id,
                fallback_device_id = %device.id.0,
                fallback_device_name = %device.name,
                "Saved microphone device not found, falling back to system default"
            );
            if let Err(error) = crate::dictation::save_dictation_device_id(None) {
                tracing::warn!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to clear stale microphone preference"
                );
            }
        }
        (Some(saved_id), None) => {
            tracing::warn!(
                category = "DICTATION",
                missing_device_id = %saved_id,
                "Saved microphone device not found and no default input device is available"
            );
            if let Err(error) = crate::dictation::save_dictation_device_id(None) {
                tracing::warn!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to clear stale microphone preference"
                );
            }
        }
        (None, Some(device)) => {
            tracing::info!(
                category = "DICTATION",
                device_id = %device.id.0,
                device_name = %device.name,
                "No saved mic preference, using system default"
            );
        }
        (None, None) => {
            tracing::warn!(
                category = "DICTATION",
                "No saved mic preference and no default input device is available"
            );
        }
    }

    Ok(selected.map(|d| d.id))
}
