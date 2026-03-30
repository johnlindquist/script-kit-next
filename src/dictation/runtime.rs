use crate::dictation::capture::{start_capture, DictationCaptureHandle};
use crate::dictation::device::{default_input_device, list_input_devices};
use crate::dictation::transcription::{
    captured_duration, DictationTranscriber, DictationTranscriptionConfig, WhisperDictationEngine,
};
use crate::dictation::types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDeviceId, DictationLevel, DictationToggleOutcome,
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
        phase: crate::dictation::DictationSessionPhase::Recording,
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
pub fn transcribe_captured_audio(
    chunks: &[CapturedAudioChunk],
) -> Result<Option<String>> {
    let mut guard = TRANSCRIBER.lock();

    if guard
        .as_ref()
        .map(DictationTranscriber::is_idle)
        .unwrap_or(true)
    {
        let config = DictationTranscriptionConfig::default();
        let engine = WhisperDictationEngine::new(&config).with_context(|| {
            format!(
                "failed to initialize Whisper engine from {}",
                config.model_path.display()
            )
        })?;
        *guard = Some(DictationTranscriber::new(config, Box::new(engine)));
    }

    guard
        .as_ref()
        .context("dictation transcriber unavailable")?
        .transcribe_chunks(chunks)
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
pub(crate) fn resolve_preferred_device() -> Result<Option<DictationDeviceId>> {
    let prefs = crate::config::load_user_preferences();
    let preferred_id = prefs.dictation.selected_device_id.clone();

    let Some(preferred) = preferred_id else {
        // No preference saved — use system default.
        let default = default_input_device()?;
        tracing::info!(
            category = "DICTATION",
            device = ?default.as_ref().map(|d| &d.name),
            "No saved mic preference, using system default"
        );
        return Ok(default.map(|d| d.id));
    };

    // Check whether the saved device is still connected.
    let devices = list_input_devices()?;
    if let Some(found) = devices.iter().find(|d| d.id.0 == preferred) {
        tracing::info!(
            category = "DICTATION",
            device_id = %found.id.0,
            device_name = %found.name,
            "Using saved microphone preference"
        );
        return Ok(Some(found.id.clone()));
    }

    // Saved device is no longer available — fall back and self-heal.
    tracing::warn!(
        category = "DICTATION",
        missing_device_id = %preferred,
        "Saved microphone device not found, falling back to system default"
    );

    if let Err(error) = crate::dictation::save_dictation_device_id(None) {
        tracing::warn!(
            category = "DICTATION",
            error = %error,
            "Failed to clear stale microphone preference"
        );
    }

    let default = default_input_device()?;
    Ok(default.map(|d| d.id))
}
