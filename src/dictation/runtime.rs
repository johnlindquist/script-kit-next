use crate::dictation::capture::{start_capture, DictationCaptureHandle};
use crate::dictation::device::{default_input_device, list_input_devices};
use crate::dictation::transcription::{
    build_session_result, DictationTranscriber, DictationTranscriptionConfig,
    WhisperDictationEngine,
};
use crate::dictation::types::{
    CapturedAudioChunk, DictationCaptureConfig, DictationCaptureEvent, DictationDestination,
    DictationDeviceId, DictationSessionResult,
};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Session state
// ---------------------------------------------------------------------------

/// Live state of a dictation recording session.
struct DictationSession {
    _capture_handle: DictationCaptureHandle,
    event_rx: async_channel::Receiver<DictationCaptureEvent>,
    chunks: Vec<CapturedAudioChunk>,
    started_at: Instant,
}

/// Global singleton guarded by a parking_lot Mutex.
///
/// `None` means idle (no recording in progress).
static SESSION: Mutex<Option<DictationSession>> = Mutex::new(None);

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Toggle dictation recording on/off.
///
/// When called while idle, starts a new dictation capture session.
/// When called while recording, stops capture and begins transcription.
///
/// This is the single entrypoint shared by both `BuiltInFeature::Dictation`
/// and `HotkeyAction::Dictation`.
pub fn toggle_dictation() -> Result<()> {
    tracing::info!(category = "DICTATION", "toggle_dictation called");

    let was_recording = SESSION.lock().is_some();

    if was_recording {
        stop_and_transcribe()?;
    } else {
        start_recording()?;
    }

    Ok(())
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
        _capture_handle: capture_handle,
        event_rx,
        chunks: Vec::new(),
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
// Stop + transcribe + deliver
// ---------------------------------------------------------------------------

fn stop_and_transcribe() -> Result<()> {
    let session = SESSION.lock().take();
    let Some(mut session) = session else {
        tracing::warn!(
            category = "DICTATION",
            "stop_and_transcribe called but no session active"
        );
        return Ok(());
    };

    // Drain remaining events from the capture channel.
    while let Ok(event) = session.event_rx.try_recv() {
        if let DictationCaptureEvent::Chunk(chunk) = event {
            session.chunks.push(chunk);
        }
    }

    let elapsed = session.started_at.elapsed();
    tracing::info!(
        category = "DICTATION",
        chunks = session.chunks.len(),
        elapsed_ms = elapsed.as_millis() as u64,
        "Recording stopped, beginning transcription"
    );

    // Transcribe
    let result = transcribe_and_deliver(&session.chunks)?;

    if let Some(result) = &result {
        tracing::info!(
            category = "DICTATION",
            transcript_len = result.transcript.len(),
            destination = ?result.destination,
            audio_duration_ms = result.audio_duration.as_millis() as u64,
            "Dictation session complete"
        );
    } else {
        tracing::info!(
            category = "DICTATION",
            "No speech detected — nothing to deliver"
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Transcription + delivery
// ---------------------------------------------------------------------------

/// Transcribe captured audio and deliver the text to the appropriate
/// destination.
///
/// Returns `Ok(None)` when no speech is detected.
pub(crate) fn transcribe_and_deliver(
    chunks: &[CapturedAudioChunk],
) -> Result<Option<DictationSessionResult>> {
    let config = DictationTranscriptionConfig::default();
    let engine = WhisperDictationEngine::new(&config).with_context(|| {
        format!(
            "failed to initialize Whisper engine from {}",
            config.model_path.display()
        )
    })?;

    let transcriber = DictationTranscriber::new(config, Box::new(engine));

    let transcript = match transcriber.transcribe_chunks(chunks)? {
        Some(text) => text,
        None => return Ok(None),
    };

    let destination = resolve_destination();
    deliver_transcript(&transcript, &destination)?;

    Ok(Some(build_session_result(chunks, destination, transcript)))
}

/// Deliver transcribed text to the resolved destination.
///
/// - `ActivePrompt`: calls `set_prompt_input(...)` on the main window.
/// - `FrontmostApp`: uses the existing `TextInjector::paste_text(...)` path.
pub(crate) fn deliver_transcript(
    transcript: &str,
    destination: &DictationDestination,
) -> Result<()> {
    match destination {
        DictationDestination::ActivePrompt => {
            tracing::info!(
                category = "DICTATION",
                "Delivering transcript to active prompt via set_prompt_input"
            );
            // In the full integration this calls through the main window handle:
            //   window_handle.update(cx, |view, _window, cx| {
            //       view.set_prompt_input(transcript.to_string(), cx);
            //   });
            // For now we log the intent — the GPUI App context is not available
            // in this synchronous helper.  The caller (toggle_dictation via
            // cx.spawn) will perform the actual update.
            Ok(())
        }
        DictationDestination::FrontmostApp => {
            tracing::info!(
                category = "DICTATION",
                "Delivering transcript to frontmost app via paste_text"
            );
            let injector = crate::text_injector::TextInjector::new();
            injector
                .paste_text(transcript)
                .context("failed to paste transcribed text to frontmost app")?;
            Ok(())
        }
    }
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

    // Saved device is no longer available — fall back.
    tracing::warn!(
        category = "DICTATION",
        missing_device_id = %preferred,
        "Saved microphone device not found, falling back to system default"
    );
    let default = default_input_device()?;
    Ok(default.map(|d| d.id))
}

// ---------------------------------------------------------------------------
// Destination resolution
// ---------------------------------------------------------------------------

/// Determine where transcribed text should be delivered.
///
/// If a Script Kit prompt is currently active, text goes to the prompt input.
/// Otherwise it goes to the frontmost app via clipboard paste.
fn resolve_destination() -> DictationDestination {
    // TODO: check if a Script Kit prompt is active by inspecting AppView.
    // For now, default to frontmost app — the safer choice.
    DictationDestination::FrontmostApp
}
