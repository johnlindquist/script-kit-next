use crate::dictation::capture::{start_capture, DictationCaptureHandle};
use crate::dictation::device::{list_input_devices, resolve_selected_input_device};
use crate::dictation::transcription::{
    captured_duration, is_parakeet_model_available, merge_captured_chunks,
    should_skip_transcription, DictationTranscriber, DictationTranscriptionConfig,
    ParakeetDictationEngine,
};
use crate::dictation::types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDeviceId, DictationDeviceInfo, DictationModelStatus, DictationSessionPhase,
    DictationTarget, DictationToggleOutcome,
};
use crate::dictation::visualizer::silent_bars;
use crate::dictation::window::DictationOverlayState;
use crate::dictation::DictationWrongTargetRefusalDraft;
use anyhow::{Context, Result};
use gpui::SharedString;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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
    /// Microphone resolved when capture started. Changing preferences while
    /// recording applies to the next session, not this live AVCaptureSession.
    active_device: Option<DictationDeviceInfo>,
}

/// Global singleton guarded by a parking_lot Mutex.
///
/// `None` means idle (no recording in progress).
static SESSION: Mutex<Option<DictationSession>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationStopReason {
    Hotkey,
    OverlaySubmit,
    OverlayAbort,
    GlobalEscape,
    NativeFooter,
    Synthetic,
}

#[derive(Debug, Clone)]
struct StopState {
    request_id: u64,
    reason: DictationStopReason,
    target: DictationTarget,
    requested_at: Instant,
}

pub enum BeginStopCapture {
    Started {
        request_id: u64,
        target: DictationTarget,
        job: Box<DictationStopJob>,
    },
    AlreadyStopping {
        request_id: u64,
    },
    NotRecording,
}

pub struct DictationStopJob {
    session: DictationSession,
}

static STOP_IN_FLIGHT: Mutex<Option<StopState>> = Mutex::new(None);
static STOP_GENERATION: AtomicU64 = AtomicU64::new(0);
static LAST_STOP_RECEIPT: Mutex<Option<serde_json::Value>> = Mutex::new(None);

/// Lazily-initialized transcriber, kept alive across sessions so the model
/// does not need to be reloaded for every dictation.  Unloaded after an idle
/// timeout via `maybe_unload_transcriber()`.
static TRANSCRIBER: Mutex<Option<DictationTranscriber>> = Mutex::new(None);

/// Monotonic counter for redacted delivery receipts exposed through automation.
static DELIVERY_RECEIPT_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Monotonic generation for passive dictation runtime state exposed to DevTools.
static DICTATION_STATE_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Last dictation delivery receipt. This stores only routing metadata, length,
/// and a one-way fingerprint; it never stores transcript text.
static LAST_DELIVERY_RECEIPT: Mutex<Option<serde_json::Value>> = Mutex::new(None);

/// Monotonic counter for wrong-target refusal receipts.
static WRONG_TARGET_REFUSAL_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Last fail-closed dictation delivery refusal. Stores only redacted routing
/// metadata and never stores transcript text or raw requested labels.
static LAST_WRONG_TARGET_REFUSAL: Mutex<Option<serde_json::Value>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationTranscriptResolution {
    pub transcript: Option<String>,
    pub used_partial_fallback: bool,
    pub final_len: usize,
    pub partial_len: Option<usize>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns `true` when a dictation capture session is currently active.
pub fn is_dictation_recording() -> bool {
    SESSION.lock().is_some()
}

pub fn is_dictation_stopping() -> bool {
    STOP_IN_FLIGHT.lock().is_some()
}

pub fn is_dictation_busy() -> bool {
    is_dictation_recording() || is_dictation_stopping()
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
    bump_dictation_state_generation();
    true
}

/// Return the active dictation session's overlay phase, or `None` when no
/// session is recording. Used by automation state receipts to surface the
/// dictation lifecycle (`recording` → `confirming` → `transcribing` …) through
/// `getAgentChatState.dictationPhase`.
pub fn current_dictation_phase() -> Option<DictationSessionPhase> {
    SESSION.lock().as_ref().map(|s| s.overlay_phase.clone())
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

pub fn begin_stop_capture(reason: DictationStopReason) -> Result<BeginStopCapture> {
    if let Some(existing) = STOP_IN_FLIGHT.lock().as_ref().cloned() {
        tracing::info!(
            category = "DICTATION",
            stop_request_id = existing.request_id,
            ?reason,
            existing_reason = ?existing.reason,
            "Dictation stop already in flight"
        );
        return Ok(BeginStopCapture::AlreadyStopping {
            request_id: existing.request_id,
        });
    }

    let session = SESSION.lock().take();
    let Some(session) = session else {
        return Ok(BeginStopCapture::NotRecording);
    };

    let request_id = STOP_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let target = session.target;
    let target_label = target.overlay_label();
    let requested_at = Instant::now();
    *STOP_IN_FLIGHT.lock() = Some(StopState {
        request_id,
        reason,
        target,
        requested_at,
    });
    *LAST_STOP_RECEIPT.lock() = Some(serde_json::json!({
        "inFlight": true,
        "requestId": request_id,
        "reason": format!("{:?}", reason),
        "target": format!("{:?}", target),
        "targetLabel": target_label,
        "collectDurationMs": null,
        "timedOut": false,
        "chunkCount": null,
        "audioDurationMs": null,
        "source": "runtime.dictation.stopCoordinator",
        "redacted": true,
    }));
    bump_dictation_state_generation();

    tracing::info!(
        category = "DICTATION",
        event = "dictation_stop_requested",
        stop_request_id = request_id,
        ?reason,
        ?target,
        target_label,
        "Dictation stop handed to coordinator"
    );

    Ok(BeginStopCapture::Started {
        request_id,
        target,
        job: Box::new(DictationStopJob { session }),
    })
}

pub fn finish_stop_capture(
    request_id: u64,
    result: &Result<Option<CompletedDictationCapture>>,
) -> bool {
    let stop_state = {
        let mut guard = STOP_IN_FLIGHT.lock();
        match guard.as_ref() {
            Some(state) if state.request_id == request_id => guard.take(),
            Some(state) => {
                tracing::warn!(
                    category = "DICTATION",
                    stop_request_id = request_id,
                    active_stop_request_id = state.request_id,
                    "Ignoring stale dictation stop completion"
                );
                None
            }
            None => None,
        }
    };

    let Some(stop_state) = stop_state else {
        return false;
    };

    let collect_duration = stop_state.requested_at.elapsed();
    let (chunk_count, audio_duration_ms, error) = match result {
        Ok(Some(capture)) => (
            Some(capture.chunks.len()),
            Some(capture.audio_duration.as_millis() as u64),
            None,
        ),
        Ok(None) => (Some(0), Some(0), None),
        Err(error) => (None, None, Some(error.to_string())),
    };
    *LAST_STOP_RECEIPT.lock() = Some(serde_json::json!({
        "inFlight": false,
        "requestId": request_id,
        "reason": format!("{:?}", stop_state.reason),
        "target": format!("{:?}", stop_state.target),
        "targetLabel": stop_state.target.overlay_label(),
        "collectDurationMs": collect_duration.as_millis() as u64,
        "timedOut": error.as_ref().is_some_and(|message| message.contains("timed out")),
        "chunkCount": chunk_count,
        "audioDurationMs": audio_duration_ms,
        "error": error,
        "source": "runtime.dictation.stopCoordinator",
        "redacted": true,
    }));
    bump_dictation_state_generation();

    tracing::info!(
        category = "DICTATION",
        event = "dictation_stop_completed",
        stop_request_id = request_id,
        reason = ?stop_state.reason,
        target = ?stop_state.target,
        collect_ms = collect_duration.as_millis() as u64,
        chunk_count = ?chunk_count,
        audio_duration_ms = ?audio_duration_ms,
        "Dictation stop coordinator completed"
    );

    true
}

pub fn last_stop_receipt() -> Option<serde_json::Value> {
    LAST_STOP_RECEIPT.lock().clone()
}

/// Return the delivery target that was captured when the current dictation
/// session started.  Returns `None` when no session is active.
pub fn get_dictation_target() -> Option<DictationTarget> {
    SESSION.lock().as_ref().map(|s| s.target)
}

/// Return the microphone that the active capture session opened with.
pub fn get_active_dictation_device() -> Option<DictationDeviceInfo> {
    SESSION
        .lock()
        .as_ref()
        .and_then(|session| session.active_device.clone())
}

/// Record a content-safe receipt for the most recent dictation delivery.
///
/// Agent-facing DevTools use this to prove target routing and delivery
/// generation without reading transcript contents from logs or UI text.
pub fn record_delivery_receipt(
    transcript: &str,
    audio_duration: std::time::Duration,
    target: DictationTarget,
    destination: crate::dictation::DictationDestination,
    delivered_internally: bool,
    history_entry_id: &str,
    insertion_range: Option<serde_json::Value>,
) -> serde_json::Value {
    let generation = DELIVERY_RECEIPT_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let receipt = serde_json::json!({
        "generation": generation,
        "target": format!("{:?}", target),
        "targetLabel": target.overlay_label(),
        "destination": format!("{:?}", destination),
        "deliveredInternally": delivered_internally,
        "historyEntryId": history_entry_id,
        "transcriptLen": transcript.len(),
        "transcriptFingerprint": redacted_transcript_fingerprint(transcript),
        "audioDurationMs": audio_duration.as_millis() as u64,
        "insertionRange": insertion_range.unwrap_or_else(|| serde_json::json!({
            "available": false,
            "reason": "destination did not expose an insertion range receipt",
        })),
        "source": "deliveryPipeline",
        "redacted": true,
    });
    *LAST_DELIVERY_RECEIPT.lock() = Some(receipt.clone());
    bump_dictation_state_generation();
    receipt
}

pub fn delivery_receipt_generation() -> u64 {
    DELIVERY_RECEIPT_GENERATION.load(Ordering::Relaxed)
}

pub fn record_wrong_target_refusal(
    draft: DictationWrongTargetRefusalDraft,
    transcript_len: Option<usize>,
) -> serde_json::Value {
    let generation = WRONG_TARGET_REFUSAL_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let requested_target_label_len = draft
        .requested_target_label
        .as_ref()
        .map(|label| label.chars().count());
    let requested_target_label_fingerprint = draft
        .requested_target_label
        .as_ref()
        .map(|label| redacted_transcript_fingerprint(label));
    let delivery_generation_after = delivery_receipt_generation();

    let receipt = serde_json::json!({
        "generation": generation,
        "reasonCode": draft.reason.as_code(),
        "requestedTarget": draft.requested_target.map(|target| format!("{:?}", target)),
        "requestedTargetLabelLen": requested_target_label_len,
        "requestedTargetLabelFingerprint": requested_target_label_fingerprint,
        "fallbackTarget": draft.fallback_target.map(|target| format!("{:?}", target)),
        "deliveryGenerationBefore": draft.delivery_generation_before,
        "deliveryGenerationAfter": delivery_generation_after,
        "noDeliveryAttempted": true,
        "transcriptLen": transcript_len,
        "source": "deliveryActor",
        "redacted": true,
    });
    *LAST_WRONG_TARGET_REFUSAL.lock() = Some(receipt.clone());
    bump_dictation_state_generation();
    receipt
}

/// Resolve final transcription text with a partial fallback for stop/finalize
/// races where the provider returns an empty final result.
pub fn resolve_final_or_partial_transcript(
    final_transcript: &str,
    partial_transcript: Option<&str>,
) -> DictationTranscriptResolution {
    if !final_transcript.trim().is_empty() {
        return DictationTranscriptResolution {
            transcript: Some(final_transcript.to_string()),
            used_partial_fallback: false,
            final_len: final_transcript.len(),
            partial_len: partial_transcript.map(str::len),
        };
    }

    if let Some(partial) = partial_transcript.filter(|value| !value.trim().is_empty()) {
        return DictationTranscriptResolution {
            transcript: Some(partial.to_string()),
            used_partial_fallback: true,
            final_len: final_transcript.len(),
            partial_len: Some(partial.len()),
        };
    }

    DictationTranscriptResolution {
        transcript: None,
        used_partial_fallback: false,
        final_len: final_transcript.len(),
        partial_len: partial_transcript.map(str::len),
    }
}

/// Return the latest content-safe dictation delivery receipt.
pub fn last_delivery_receipt() -> Option<serde_json::Value> {
    LAST_DELIVERY_RECEIPT.lock().clone()
}

pub fn last_wrong_target_refusal() -> Option<serde_json::Value> {
    LAST_WRONG_TARGET_REFUSAL.lock().clone()
}

/// Stable non-cryptographic fingerprint for correlating synthetic receipts.
///
/// This deliberately supports equality checks only; raw transcript text is not
/// recoverable from the automation state.
pub fn redacted_transcript_fingerprint(transcript: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in transcript.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
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
    bump_dictation_state_generation();
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
    bump_dictation_state_generation();

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

/// Passive, redacted automation snapshot for Script Kit DevTools.
///
/// This is safe for `getState`: it does not start capture, open System
/// Settings, request TCC permission, transcribe audio, or expose transcript
/// contents or raw microphone device ids.
pub fn automation_state() -> serde_json::Value {
    let (is_recording, phase, target, target_label, elapsed_ms, audio_levels) = {
        let guard = SESSION.lock();
        if let Some(session) = guard.as_ref() {
            (
                true,
                session.overlay_phase.as_automation_str().to_string(),
                Some(format!("{:?}", session.target)),
                Some(session.target.overlay_label().to_string()),
                Some(session.started_at.elapsed().as_millis() as u64),
                serde_json::json!({
                    "available": true,
                    "bars": session.last_bars,
                    "barCount": session.last_bars.len(),
                    "source": "runtime.session.lastBars",
                    "stopReason": null,
                }),
            )
        } else if let Some(stop_state) = STOP_IN_FLIGHT.lock().as_ref().cloned() {
            (
                false,
                "stopping".to_string(),
                Some(format!("{:?}", stop_state.target)),
                Some(stop_state.target.overlay_label().to_string()),
                Some(stop_state.requested_at.elapsed().as_millis() as u64),
                serde_json::json!({
                    "available": false,
                    "bars": [],
                    "barCount": 0,
                    "source": "runtime.session.lastBars",
                    "stopReason": "capture stop in flight",
                }),
            )
        } else {
            (
                false,
                "idle".to_string(),
                None,
                None,
                None,
                serde_json::json!({
                    "available": false,
                    "bars": [],
                    "barCount": 0,
                    "source": "runtime.session.lastBars",
                    "stopReason": "not recording",
                }),
            )
        }
    };

    let config = crate::config::load_config();
    let prefs = crate::config::load_user_preferences();
    let selected_device_id = prefs.dictation.selected_device_id.as_deref();
    let permission = crate::dictation::microphone_permission_status();
    let devices_result = if matches!(
        permission,
        crate::dictation::DictationMicrophonePermissionStatus::Granted
            | crate::dictation::DictationMicrophonePermissionStatus::Unknown
    ) {
        crate::dictation::list_input_devices().map_err(|error| error.to_string())
    } else {
        Ok(Vec::new())
    };
    let device_snapshot = microphone_device_snapshot(&devices_result, selected_device_id);
    let setup_state = crate::dictation::build_dictation_setup_state(
        dictation_model_status(),
        permission,
        devices_result,
        selected_device_id,
        config.get_dictation_hotkey().as_ref(),
        config.is_dictation_hotkey_enabled(),
    );
    let hotkey = config.get_dictation_hotkey();
    let generation = DICTATION_STATE_GENERATION.load(Ordering::Relaxed);

    serde_json::json!({
        "schemaVersion": 1,
        "source": "runtime.dictation.automationState",
        "passive": true,
        "redacted": true,
        "generation": generation,
        "recordingStateGeneration": generation,
        "isRecording": is_recording,
        "phase": phase,
        "target": target,
        "targetLabel": target_label,
        "elapsedMs": elapsed_ms,
        "audioLevels": audio_levels,
        "setup": {
            "ready": setup_state.ready,
            "model": {
                "status": dictation_model_status_label(&setup_state.model_status),
                "generation": generation,
                "source": "parakeetModelAvailability",
            },
            "microphone": {
                "permissionStatus": format!("{:?}", permission),
                "status": microphone_status_label(&setup_state.microphone_status),
                "deviceSnapshot": device_snapshot,
                "generation": generation,
                "source": "passiveAVFoundationEnumeration",
                "noPermissionPrompt": true,
            },
            "hotkey": {
                "enabled": config.is_dictation_hotkey_enabled(),
                "configured": hotkey.is_some(),
                "display": hotkey.as_ref().map(|value| value.to_display_string()),
                "status": format!("{:?}", setup_state.hotkey_status),
                "generation": generation,
                "source": "config.dictationHotkey",
            },
            "configFingerprint": crate::config::current_config_fingerprint_receipt().map(|receipt| serde_json::json!({
                "pathFingerprint": automation_fingerprint(&receipt.path),
                "len": receipt.len,
                "modifiedMs": receipt.modified_ms,
            })),
        },
        "lastDelivery": crate::dictation::last_delivery_receipt(),
        "deliveryReceiptAvailable": crate::dictation::last_delivery_receipt().is_some(),
        "wrongTargetRefusal": crate::dictation::last_wrong_target_refusal(),
        "wrongTargetRefusalAvailable": crate::dictation::last_wrong_target_refusal().is_some(),
        "stop": crate::dictation::last_stop_receipt(),
        "cleanup": {
            "captureActive": is_recording,
            "captureStopInProgress": crate::dictation::is_dictation_stopping(),
            "transcriberCached": TRANSCRIBER.lock().is_some(),
            "generation": generation,
            "source": "runtime.dictation.cleanupState",
        },
        "safety": {
            "noMicrophoneCaptureStarted": true,
            "noSystemSettingsOpened": true,
            "noTccMutation": true,
            "noTranscriptContent": true,
            "rawDeviceIdsRedacted": true,
            "deviceLabelsReturned": false,
        },
    })
}

fn bump_dictation_state_generation() -> u64 {
    DICTATION_STATE_GENERATION.fetch_add(1, Ordering::Relaxed) + 1
}

pub(crate) fn notify_dictation_device_preference_changed() -> u64 {
    bump_dictation_state_generation()
}

fn dictation_model_status() -> DictationModelStatus {
    if is_parakeet_model_available() {
        DictationModelStatus::Available
    } else {
        DictationModelStatus::NotDownloaded
    }
}

fn dictation_model_status_label(status: &DictationModelStatus) -> &'static str {
    match status {
        DictationModelStatus::Available => "available",
        DictationModelStatus::NotDownloaded => "notDownloaded",
        DictationModelStatus::Downloading { .. } => "downloading",
        DictationModelStatus::Extracting => "extracting",
        DictationModelStatus::DownloadFailed(_) => "downloadFailed",
    }
}

fn microphone_status_label(status: &crate::dictation::DictationMicrophoneStatus) -> &'static str {
    match status {
        crate::dictation::DictationMicrophoneStatus::Ready { .. } => "ready",
        crate::dictation::DictationMicrophoneStatus::SavedDeviceMissing { .. } => {
            "savedDeviceMissing"
        }
        crate::dictation::DictationMicrophoneStatus::PermissionNeeded(_) => "permissionNeeded",
        crate::dictation::DictationMicrophoneStatus::NoDevices => "noDevices",
        crate::dictation::DictationMicrophoneStatus::EnumerationFailed(_) => "enumerationFailed",
    }
}

fn automation_fingerprint(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn microphone_device_snapshot(
    devices_result: &Result<Vec<crate::dictation::DictationDeviceInfo>, String>,
    selected_device_id: Option<&str>,
) -> serde_json::Value {
    match devices_result {
        Ok(devices) => {
            let selected =
                crate::dictation::resolve_selected_input_device(devices, selected_device_id);
            serde_json::json!({
                "available": true,
                "deviceCount": devices.len(),
                "defaultDevice": devices.iter().find(|device| device.is_default).map(|device| {
                    serde_json::json!({
                        "labelFingerprint": automation_fingerprint(&device.name),
                        "idFingerprint": automation_fingerprint(&device.id.0),
                        "transport": format!("{:?}", device.transport),
                    })
                }),
                "selected": selected.as_ref().map(|selection| {
                    serde_json::json!({
                        "labelFingerprint": automation_fingerprint(&selection.device.name),
                        "idFingerprint": automation_fingerprint(&selection.device.id.0),
                        "transport": format!("{:?}", selection.device.transport),
                        "fellBack": selection.fell_back,
                    })
                }),
                "savedPreference": {
                    "configured": selected_device_id.is_some(),
                    "idFingerprint": selected_device_id.map(automation_fingerprint),
                    "resolved": selected.as_ref().is_some_and(|selection| !selection.fell_back),
                },
                "rawDeviceIdsRedacted": true,
                "deviceLabelsReturned": false,
            })
        }
        Err(error) => serde_json::json!({
            "available": false,
            "deviceCount": 0,
            "error": error,
            "rawDeviceIdsRedacted": true,
            "deviceLabelsReturned": false,
        }),
    }
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
    let active_device = resolve_preferred_device_info()?;
    let device_id = active_device.as_ref().map(|device| device.id.clone());
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
        active_device: active_device.clone(),
    };

    *SESSION.lock() = Some(session);
    bump_dictation_state_generation();

    tracing::info!(
        category = "DICTATION",
        device = ?device_id,
        device_name = ?active_device.as_ref().map(|device| device.name.as_str()),
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
    bump_dictation_state_generation();

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

impl DictationStopJob {
    pub fn collect_with_deadline(
        mut self,
        timeout: Duration,
    ) -> Result<Option<CompletedDictationCapture>> {
        let started = Instant::now();
        tracing::info!(
            category = "DICTATION",
            timeout_ms = timeout.as_millis() as u64,
            "capture_stop_job_started"
        );

        stop_capture_and_collect_with_deadline(&mut self.session, timeout)?;

        let audio_duration = captured_duration(&self.session.chunks);
        tracing::info!(
            category = "DICTATION",
            chunks = self.session.chunks.len(),
            audio_duration_ms = audio_duration.as_millis() as u64,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "capture_stop_job_finished"
        );

        if self.session.chunks.is_empty() {
            return Ok(None);
        }

        Ok(Some(CompletedDictationCapture {
            chunks: self.session.chunks,
            audio_duration,
        }))
    }
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

fn stop_capture_and_collect_with_deadline(
    session: &mut DictationSession,
    timeout: Duration,
) -> Result<()> {
    let started = Instant::now();
    let _ = session.capture_handle.take();

    loop {
        if started.elapsed() > timeout {
            tracing::warn!(
                category = "DICTATION",
                event = "capture_stop_job_timed_out",
                elapsed_ms = started.elapsed().as_millis() as u64,
                chunks = session.chunks.len(),
                "Timed out waiting for dictation capture EndOfStream; using partial capture"
            );
            return Ok(());
        }

        match session.event_rx.try_recv() {
            Ok(DictationCaptureEvent::Chunk(chunk)) => session.chunks.push(chunk),
            Ok(DictationCaptureEvent::Bars(bars)) => session.last_bars = bars,
            Ok(DictationCaptureEvent::EndOfStream) => return Ok(()),
            Err(async_channel::TryRecvError::Empty) => std::thread::sleep(Duration::from_millis(5)),
            Err(async_channel::TryRecvError::Closed) => return Ok(()),
        }
    }
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
    Ok(resolve_preferred_device_info()?.map(|device| device.id))
}

pub(crate) fn resolve_preferred_device_info() -> Result<Option<DictationDeviceInfo>> {
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
            Ok(Some(res.device))
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
            Ok(Some(res.device))
        }
        None => {
            tracing::warn!(category = "DICTATION", "No input devices available");
            Ok(None)
        }
    }
}
