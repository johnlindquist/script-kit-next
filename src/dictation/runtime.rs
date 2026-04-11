use crate::dictation::capture::{start_capture, DictationCaptureHandle};
use crate::dictation::device::{list_input_devices, resolve_selected_input_device};
use crate::dictation::transcription::{
    captured_duration, is_parakeet_model_available, merge_captured_chunks,
    should_skip_transcription, DictationTranscriber, DictationTranscriptionConfig,
    ParakeetDictationEngine,
};
use crate::dictation::types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDeviceId, DictationSessionPhase, DictationTarget, DictationToggleOutcome,
};
use crate::dictation::visualizer::silent_bars;
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
    last_bars: [f32; 9],
    started_at: Instant,
    /// The authoritative overlay phase — written by both the runtime (on start)
    /// and the overlay key handler (on Escape transitions).  The pump reads this
    /// on every tick so the overlay never drifts from shared state.
    overlay_phase: DictationSessionPhase,
    /// The Script Kit surface that was active when dictation started.
    /// Captured at start time so the delivery path knows where to route
    /// the transcript even if the UI changes while the user is speaking.
    target: DictationTarget,
    /// Ordered list of destinations the overlay badge cycles through.
    target_cycle: Vec<DictationTarget>,
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

/// Returns the elapsed duration of the active dictation session, or `None`
/// when no session is active.  This reads `started_at` directly from the
/// live session so the caller gets an authoritative wall-clock elapsed time
/// rather than a pump-tick-stale snapshot.
pub fn dictation_elapsed() -> Option<std::time::Duration> {
    SESSION.lock().as_ref().map(|s| s.started_at.elapsed())
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
/// - When idle: starts a new capture session targeting `target` and
///   returns `Started`.
/// - When recording: drops the capture handle (flushing the tail chunk),
///   drains events until `EndOfStream`, and returns `Stopped(Some(capture))`
///   or `Stopped(None)` for an empty recording.
///
/// This is the single entrypoint shared by both `BuiltInFeature::Dictation`
/// and `HotkeyAction::Dictation`.  The caller owns transcription, overlay
/// transitions, and delivery — the runtime only captures audio.
pub fn toggle_dictation(target: DictationTarget) -> Result<DictationToggleOutcome> {
    tracing::info!(category = "DICTATION", ?target, "toggle_dictation called");

    if SESSION.lock().is_some() {
        return Ok(DictationToggleOutcome::Stopped(stop_recording()?));
    }

    start_recording(target)?;
    Ok(DictationToggleOutcome::Started)
}

/// Return the delivery target that was captured when the current dictation
/// session started.  Returns `None` when no session is active.
pub fn get_dictation_target() -> Option<DictationTarget> {
    SESSION.lock().as_ref().map(|s| s.target)
}

/// Replace the active session's destination cycle. The current target is
/// preserved and inserted if omitted so overlay state and delivery stay aligned.
pub fn set_dictation_target_cycle(targets: Vec<DictationTarget>) -> bool {
    let mut guard = SESSION.lock();
    let Some(session) = guard.as_mut() else {
        return false;
    };

    let mut cycle = vec![session.target];
    for target in targets {
        if !cycle.contains(&target) {
            cycle.push(target);
        }
    }

    session.target_cycle = cycle;
    true
}

/// Returns true when the active session has more than one destination in its
/// cycle, meaning the overlay badge should look interactive.
pub fn can_cycle_dictation_target() -> bool {
    SESSION
        .lock()
        .as_ref()
        .map(|session| session.target_cycle.len() > 1)
        .unwrap_or(false)
}

/// Advance the active session to the next configured destination.
pub fn cycle_dictation_target() -> Option<DictationTarget> {
    let mut guard = SESSION.lock();
    let session = guard.as_mut()?;

    if session.target_cycle.len() < 2 {
        return Some(session.target);
    }

    let current_ix = session
        .target_cycle
        .iter()
        .position(|target| *target == session.target)
        .unwrap_or(0);
    let next_ix = (current_ix + 1) % session.target_cycle.len();
    let next_target = session.target_cycle[next_ix];
    session.target = next_target;

    tracing::info!(
        category = "DICTATION",
        ?next_target,
        target_label = next_target.overlay_label(),
        "Dictation target cycled"
    );

    Some(next_target)
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
            DictationCaptureEvent::Bars(bars) => session.last_bars = bars,
            DictationCaptureEvent::EndOfStream => {}
        }
    }

    Some(DictationOverlayState {
        phase: session.overlay_phase.clone(),
        elapsed: session.started_at.elapsed(),
        bars: session.last_bars,
        transcript: SharedString::default(),
        target: session.target,
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

fn start_recording(target: DictationTarget) -> Result<()> {
    let device_id = resolve_preferred_device()?;
    let capture_config = DictationCaptureConfig::default();

    let (event_rx, capture_handle) = start_capture(capture_config, device_id.as_ref())
        .context("failed to start audio capture")?;

    let session = DictationSession {
        capture_handle: Some(capture_handle),
        event_rx,
        chunks: Vec::new(),
        last_bars: silent_bars(),
        started_at: Instant::now(),
        overlay_phase: DictationSessionPhase::Recording,
        target,
        target_cycle: vec![target],
    };

    *SESSION.lock() = Some(session);

    tracing::info!(
        category = "DICTATION",
        device = ?device_id,
        ?target,
        target_label = target.overlay_label(),
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
            DictationCaptureEvent::Bars(bars) => session.last_bars = bars,
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
/// Reads the config-backed `dictation.selected_device_id` preference and, if the
/// device is still present, returns its ID.  Falls back using ranked heuristics
/// (system default → built-in → USB → first non-virtual → any) when the
/// preference is unset or the device has disappeared.
///
/// Delegates the pure selection logic to [`resolve_selected_input_device`] so
/// the behavior is deterministic and testable without I/O.
pub(crate) fn resolve_preferred_device() -> Result<Option<DictationDeviceId>> {
    let prefs = crate::config::load_user_preferences();
    let preferred_id = prefs.dictation.selected_device_id.clone();

    let devices = list_input_devices()?;
    let resolution = resolve_selected_input_device(&devices, preferred_id.as_deref());

    match resolution {
        Some(res) if !res.fell_back => {
            tracing::info!(
                category = "DICTATION",
                device_id = %res.device.id.0,
                device_name = %res.device.name,
                transport = ?res.device.transport,
                saved = preferred_id.is_some(),
                "Using microphone"
            );
            Ok(Some(res.device.id))
        }
        Some(res) => {
            // Saved device disappeared — clear the stale preference.
            tracing::warn!(
                category = "DICTATION",
                missing_device_id = ?preferred_id,
                fallback_device_id = %res.device.id.0,
                fallback_device_name = %res.device.name,
                fallback_transport = ?res.device.transport,
                "Saved microphone not found, falling back"
            );
            if let Err(error) = crate::dictation::save_dictation_device_id(None) {
                tracing::warn!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to clear stale microphone preference"
                );
            }
            Ok(Some(res.device.id))
        }
        None => {
            tracing::warn!(category = "DICTATION", "No input devices available");
            Ok(None)
        }
    }
}
