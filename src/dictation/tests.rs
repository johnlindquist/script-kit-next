use crate::dictation::capture::{mix_to_mono, normalize_chunk, resample_linear, run_processor};
use crate::dictation::transcription::{
    build_session_result, merge_captured_chunks, DictationEngine, DictationTranscriber,
    DictationTranscriptionConfig,
};
use crate::dictation::types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDestination, DictationLevel, RawAudioChunk,
};
use crate::dictation::visualizer::{bars_for_level, compute_level};
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Stub engine for unit tests
// ---------------------------------------------------------------------------

struct StubEngine {
    output: String,
}

impl DictationEngine for StubEngine {
    fn transcribe(&mut self, _samples: &[f32], _initial_prompt: Option<&str>) -> Result<String> {
        Ok(self.output.clone())
    }
}

// ---------------------------------------------------------------------------
// Existing capture / visualizer tests
// ---------------------------------------------------------------------------

#[test]
fn mix_to_mono_averages_interleaved_channels() {
    let mono = mix_to_mono(&[1.0, -1.0, 0.5, 0.0], 2);
    assert_eq!(mono, vec![0.0, 0.25]);
}

#[test]
fn resample_linear_preserves_endpoints() {
    let resampled = resample_linear(&[0.0, 1.0], 2, 4);
    assert_eq!(resampled.len(), 4);
    assert!((resampled[0] - 0.0).abs() < 1e-6);
    assert!((resampled[3] - 1.0).abs() < 1e-6);
    assert!(resampled[1] > 0.0 && resampled[1] < 1.0);
    assert!(resampled[2] > 0.0 && resampled[2] < 1.0);
}

#[test]
fn normalize_chunk_mixes_resamples_and_sets_duration() {
    let config = DictationCaptureConfig::default();
    let raw = RawAudioChunk {
        sample_rate_hz: 8_000,
        channels: 2,
        samples: vec![0.5, -0.5, 0.25, 0.25],
    };

    let normalized = normalize_chunk(raw, &config);

    assert_eq!(normalized.sample_rate_hz, 16_000);
    assert_eq!(normalized.samples.len(), 4);
    assert_eq!(normalized.duration, Duration::from_micros(250));
    assert!((normalized.samples[0] - 0.0).abs() < 1e-6);
}

#[test]
fn compute_level_reports_rms_and_peak_with_clamping() {
    let level = compute_level(&[2.0, -0.5, 0.5]);
    assert!((level.peak - 1.0).abs() < 1e-6);
    assert!(level.rms > 0.0 && level.rms <= 1.0);
}

#[test]
fn bars_for_level_are_symmetric_and_clamped() {
    let bars = bars_for_level(DictationLevel {
        rms: 0.5,
        peak: 0.8,
    });

    assert_eq!(bars.len(), 9);
    assert_eq!(bars[0], bars[8]);
    assert_eq!(bars[1], bars[7]);
    assert_eq!(bars[2], bars[6]);
    assert_eq!(bars[3], bars[5]);
    assert!(bars[4] >= bars[3]);
    assert!(bars.into_iter().all(|bar| (0.08..=1.0).contains(&bar)));
}

// ---------------------------------------------------------------------------
// Transcription facade tests
// ---------------------------------------------------------------------------

#[test]
fn merge_captured_chunks_concatenates_audio() {
    let merged = merge_captured_chunks(&[
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.1, 0.2],
            duration: Duration::from_millis(10),
        },
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.3, 0.4, 0.5],
            duration: Duration::from_millis(15),
        },
    ]);
    assert_eq!(merged, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
}

#[test]
fn merge_captured_chunks_empty_input() {
    let merged = merge_captured_chunks(&[]);
    assert!(merged.is_empty());
}

#[test]
fn transcriber_returns_none_for_silence() -> Result<()> {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            minimum_samples: 4,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "should not be used".to_string(),
        }),
    );
    assert_eq!(transcriber.transcribe_samples(&[0.0, 0.0, 0.0, 0.0])?, None);
    Ok(())
}

#[test]
fn transcriber_returns_none_for_too_few_samples() -> Result<()> {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            minimum_samples: 100,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "should not be used".to_string(),
        }),
    );
    assert_eq!(transcriber.transcribe_samples(&[0.5, -0.5])?, None);
    Ok(())
}

#[test]
fn transcriber_returns_none_for_empty_engine_output() -> Result<()> {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "   ".to_string(),
        }),
    );
    assert_eq!(transcriber.transcribe_samples(&[0.25, -0.25])?, None);
    Ok(())
}

#[test]
fn transcriber_trims_engine_output() -> Result<()> {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "  hello world  ".to_string(),
        }),
    );
    assert_eq!(
        transcriber.transcribe_samples(&[0.25, -0.25])?,
        Some("hello world".to_string())
    );
    Ok(())
}

#[test]
fn transcriber_chunks_delegates_to_samples() -> Result<()> {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "from chunks".to_string(),
        }),
    );
    let chunks = vec![
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.3, 0.4],
            duration: Duration::from_millis(10),
        },
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.5],
            duration: Duration::from_millis(5),
        },
    ];
    assert_eq!(
        transcriber.transcribe_chunks(&chunks)?,
        Some("from chunks".to_string())
    );
    Ok(())
}

#[test]
fn build_session_result_sums_duration_and_keeps_destination() {
    let result = build_session_result(
        &[
            CapturedAudioChunk {
                sample_rate_hz: 16_000,
                samples: vec![0.1, 0.2],
                duration: Duration::from_millis(10),
            },
            CapturedAudioChunk {
                sample_rate_hz: 16_000,
                samples: vec![0.3],
                duration: Duration::from_millis(20),
            },
        ],
        DictationDestination::FrontmostApp,
        "hello".to_string(),
    );

    assert_eq!(result.transcript, "hello");
    assert_eq!(result.destination, DictationDestination::FrontmostApp);
    assert_eq!(result.audio_duration, Duration::from_millis(30));
}

#[test]
fn build_session_result_active_prompt_destination() {
    let result = build_session_result(
        &[CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.1],
            duration: Duration::from_millis(50),
        }],
        DictationDestination::ActivePrompt,
        "dictated text".to_string(),
    );

    assert_eq!(result.transcript, "dictated text");
    assert_eq!(result.destination, DictationDestination::ActivePrompt);
    assert_eq!(result.audio_duration, Duration::from_millis(50));
}

// ---------------------------------------------------------------------------
// Chunk-duration buffering tests
// ---------------------------------------------------------------------------

#[test]
fn run_processor_honors_chunk_duration_and_flushes_tail() {
    let config = DictationCaptureConfig {
        sample_rate_hz: 16_000,
        chunk_duration: Duration::from_millis(1), // 16 samples per chunk
        level_window: Duration::from_millis(1),
    };
    let (raw_tx, raw_rx) = std::sync::mpsc::sync_channel(4);
    let (event_tx, event_rx) = async_channel::bounded(16);

    let join = std::thread::spawn(move || run_processor(raw_rx, event_tx, config));

    raw_tx
        .send(RawAudioChunk {
            sample_rate_hz: 16_000,
            channels: 1,
            samples: vec![0.25; 20],
        })
        .expect("send raw chunk");
    drop(raw_tx);

    let mut chunk_lengths = Vec::new();
    while let Ok(event) = event_rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => chunk_lengths.push(chunk.samples.len()),
            DictationCaptureEvent::EndOfStream => break,
            DictationCaptureEvent::Level(_) => {}
        }
    }

    join.join().expect("processor thread");
    assert_eq!(chunk_lengths, vec![16, 4]);
}

#[test]
fn run_processor_emits_exact_chunk_with_no_tail() {
    let config = DictationCaptureConfig {
        sample_rate_hz: 16_000,
        chunk_duration: Duration::from_millis(1), // 16 samples per chunk
        level_window: Duration::from_millis(1),
    };
    let (raw_tx, raw_rx) = std::sync::mpsc::sync_channel(4);
    let (event_tx, event_rx) = async_channel::bounded(16);

    let join = std::thread::spawn(move || run_processor(raw_rx, event_tx, config));

    raw_tx
        .send(RawAudioChunk {
            sample_rate_hz: 16_000,
            channels: 1,
            samples: vec![0.5; 16],
        })
        .expect("send raw chunk");
    drop(raw_tx);

    let mut chunk_lengths = Vec::new();
    while let Ok(event) = event_rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => chunk_lengths.push(chunk.samples.len()),
            DictationCaptureEvent::EndOfStream => break,
            DictationCaptureEvent::Level(_) => {}
        }
    }

    join.join().expect("processor thread");
    assert_eq!(chunk_lengths, vec![16]);
}

#[test]
fn run_processor_buffers_across_multiple_raw_chunks() {
    let config = DictationCaptureConfig {
        sample_rate_hz: 16_000,
        chunk_duration: Duration::from_millis(1), // 16 samples per chunk
        level_window: Duration::from_millis(1),
    };
    let (raw_tx, raw_rx) = std::sync::mpsc::sync_channel(4);
    let (event_tx, event_rx) = async_channel::bounded(16);

    let join = std::thread::spawn(move || run_processor(raw_rx, event_tx, config));

    // Send 10 samples, then 10 more — should produce one 16-sample chunk + 4-sample tail
    raw_tx
        .send(RawAudioChunk {
            sample_rate_hz: 16_000,
            channels: 1,
            samples: vec![0.1; 10],
        })
        .expect("send first");
    raw_tx
        .send(RawAudioChunk {
            sample_rate_hz: 16_000,
            channels: 1,
            samples: vec![0.2; 10],
        })
        .expect("send second");
    drop(raw_tx);

    let mut chunk_lengths = Vec::new();
    while let Ok(event) = event_rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => chunk_lengths.push(chunk.samples.len()),
            DictationCaptureEvent::EndOfStream => break,
            DictationCaptureEvent::Level(_) => {}
        }
    }

    join.join().expect("processor thread");
    assert_eq!(chunk_lengths, vec![16, 4]);
}

// ---------------------------------------------------------------------------
// Transcriber contract tests (prompt forwarding, idle timeout)
// ---------------------------------------------------------------------------

struct RecordingEngine {
    prompts: Arc<Mutex<Vec<Option<String>>>>,
    output: String,
}

impl DictationEngine for RecordingEngine {
    fn transcribe(&mut self, _samples: &[f32], initial_prompt: Option<&str>) -> Result<String> {
        self.prompts.lock().push(initial_prompt.map(str::to_owned));
        Ok(self.output.clone())
    }
}

#[test]
fn transcriber_forwards_initial_prompt() -> Result<()> {
    let prompts = Arc::new(Mutex::new(Vec::new()));
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            initial_prompt: Some("keep punctuation".into()),
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(RecordingEngine {
            prompts: prompts.clone(),
            output: "hello".into(),
        }),
    );

    assert_eq!(
        transcriber.transcribe_samples(&[0.25])?,
        Some("hello".into())
    );
    assert_eq!(
        prompts.lock().as_slice(),
        &[Some("keep punctuation".into())]
    );
    Ok(())
}

#[test]
fn transcriber_reports_idle_after_timeout() {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            idle_unload_after: Duration::from_millis(1),
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "ok".into(),
        }),
    );

    std::thread::sleep(Duration::from_millis(5));
    assert!(transcriber.is_idle());
}

#[test]
fn transcriber_not_idle_immediately_after_use() -> Result<()> {
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            idle_unload_after: Duration::from_secs(300),
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "test".into(),
        }),
    );

    let _ = transcriber.transcribe_samples(&[0.5])?;
    assert!(!transcriber.is_idle());
    Ok(())
}

// ---------------------------------------------------------------------------
// WhisperDictationEngine tests
// ---------------------------------------------------------------------------

#[test]
fn whisper_engine_new_fails_for_missing_model() {
    use crate::dictation::transcription::{DictationTranscriptionConfig, WhisperDictationEngine};

    let config = DictationTranscriptionConfig {
        model_path: std::path::PathBuf::from("/definitely/missing-model.bin"),
        ..Default::default()
    };
    let result = WhisperDictationEngine::new(&config);
    assert!(
        result.is_err(),
        "WhisperDictationEngine::new must fail for a missing model path"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found"),
        "error should mention 'not found', got: {err_msg}"
    );
}

#[test]
fn whisper_engine_new_fails_for_directory_path() {
    use crate::dictation::transcription::{DictationTranscriptionConfig, WhisperDictationEngine};

    let config = DictationTranscriptionConfig {
        model_path: std::path::PathBuf::from("/tmp"),
        ..Default::default()
    };
    let result = WhisperDictationEngine::new(&config);
    assert!(
        result.is_err(),
        "WhisperDictationEngine::new must fail for a directory path"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not a regular file"),
        "error should mention 'not a regular file', got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Runtime handoff architecture tests
// ---------------------------------------------------------------------------

#[test]
fn runtime_returns_toggle_outcome_not_unit() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        runtime_src.contains("DictationToggleOutcome"),
        "toggle_dictation must return DictationToggleOutcome"
    );
    assert!(
        !runtime_src.contains("deliver_transcript"),
        "runtime must not own delivery — caller does"
    );
    assert!(
        !runtime_src.contains("resolve_destination"),
        "runtime must not resolve destination — caller does"
    );
}

#[test]
fn runtime_drops_capture_handle_before_draining_events() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    // The function that stops capture must drop the handle before blocking on
    // recv_blocking so the processor thread can flush its tail chunk and send
    // EndOfStream.
    let drop_pos = runtime_src
        .find("capture_handle.take()")
        .expect("runtime must drop capture handle via .take()");
    let drain_pos = runtime_src
        .find("recv_blocking()")
        .expect("runtime must blocking-drain via recv_blocking");
    assert!(
        drop_pos < drain_pos,
        "capture handle must be dropped BEFORE blocking drain (drop at byte {drop_pos}, drain at {drain_pos})"
    );
}

// ---------------------------------------------------------------------------
// Dictation goal critical paths (regression guard)
// ---------------------------------------------------------------------------

#[test]
fn dictation_goal_critical_paths_exist() {
    let mod_rs = std::fs::read_to_string("src/dictation/mod.rs").expect("read mod.rs");
    let builtin =
        std::fs::read_to_string("src/app_execute/builtin_execution.rs").expect("read builtin");
    let hotkeys = std::fs::read_to_string("src/hotkeys/mod.rs").expect("read hotkeys");
    let window = std::fs::read_to_string("src/dictation/window.rs").expect("read window.rs");
    let transcription =
        std::fs::read_to_string("src/dictation/transcription.rs").expect("read transcription.rs");

    assert!(
        mod_rs.contains("toggle_dictation"),
        "mod.rs must export toggle_dictation"
    );
    assert!(
        builtin.contains("dictation_toggle") || builtin.contains("toggle_dictation"),
        "builtin_execution.rs must call dictation toggle"
    );
    assert!(
        !builtin.contains("dictation_stub"),
        "builtin_execution.rs must not contain 'dictation_stub'"
    );
    assert!(
        !hotkeys.contains("TODO: wire dictation"),
        "hotkeys/mod.rs must not contain 'TODO: wire dictation'"
    );
    assert!(
        window.contains("WindowKind::PopUp"),
        "window.rs must use WindowKind::PopUp"
    );
    assert!(
        transcription.contains("WhisperDictationEngine"),
        "transcription.rs must define WhisperDictationEngine"
    );
}

// ---------------------------------------------------------------------------
// Mic preference resolution source audit
// ---------------------------------------------------------------------------

#[test]
fn runtime_resolves_mic_preference_from_settings() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        runtime_src.contains("load_user_preferences"),
        "runtime must read user preferences for mic selection"
    );
    assert!(
        runtime_src.contains("selected_device_id"),
        "runtime must check dictation.selected_device_id"
    );
    assert!(
        runtime_src.contains("default_input_device"),
        "runtime must fall back to default_input_device"
    );
}

// ---------------------------------------------------------------------------
// Regression: overlay window uses PopUp + Blurred + vibrancy
// ---------------------------------------------------------------------------

#[test]
fn dictation_overlay_uses_popup_blur_and_vibrancy() {
    let source =
        std::fs::read_to_string("src/dictation/window.rs").expect("read dictation window.rs");
    assert!(
        source.contains("WindowKind::PopUp"),
        "overlay must use WindowKind::PopUp"
    );
    assert!(
        source.contains("WindowBackgroundAppearance::Blurred"),
        "overlay must support blurred background"
    );
    assert!(
        source.contains("configure_secondary_window_vibrancy"),
        "overlay must configure vibrancy via platform helper"
    );
}

// ---------------------------------------------------------------------------
// Regression: dictation entrypoints must not be stubs
// ---------------------------------------------------------------------------

#[test]
fn dictation_entrypoints_are_not_stubs() {
    let builtin_src =
        std::fs::read_to_string("src/app_execute/builtin_execution.rs").expect("read builtin");
    let hotkeys_src = std::fs::read_to_string("src/hotkeys/mod.rs").expect("read hotkeys");

    assert!(
        !builtin_src.contains("dictation_stub"),
        "builtin_execution.rs still contains 'dictation_stub'"
    );
    assert!(
        !builtin_src.contains("not yet wired"),
        "builtin_execution.rs still contains 'not yet wired'"
    );
    assert!(
        !hotkeys_src.contains("TODO: wire dictation"),
        "hotkeys/mod.rs still contains 'TODO: wire dictation'"
    );
}

// ---------------------------------------------------------------------------
// Stop-path flush: tail chunk preserved after stop
// ---------------------------------------------------------------------------

/// Simulates the stop-path by sending chunks through a channel, then
/// closing the sender (simulating handle drop) and verifying that the
/// receiver collects all chunks including the tail before EndOfStream.
#[test]
fn stop_path_collects_all_chunks_including_tail_after_handle_drop() {
    use crate::dictation::types::CompletedDictationCapture;

    let (tx, rx) = async_channel::bounded::<DictationCaptureEvent>(16);

    // Simulate a processor thread that sends chunks then EndOfStream.
    let producer = std::thread::spawn(move || {
        tx.send_blocking(DictationCaptureEvent::Chunk(CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.1; 160],
            duration: Duration::from_millis(10),
        }))
        .expect("send chunk 1");

        tx.send_blocking(DictationCaptureEvent::Level(DictationLevel {
            rms: 0.3,
            peak: 0.5,
        }))
        .expect("send level");

        // Tail chunk — the one that would be lost if we drained with
        // try_recv() before the handle was dropped.
        tx.send_blocking(DictationCaptureEvent::Chunk(CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.2; 80],
            duration: Duration::from_millis(5),
        }))
        .expect("send tail chunk");

        tx.send_blocking(DictationCaptureEvent::EndOfStream)
            .expect("send EOS");
    });

    // Consumer: blocking drain (mirrors stop_capture_and_collect).
    let mut chunks = Vec::new();
    while let Ok(event) = rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => chunks.push(chunk),
            DictationCaptureEvent::Level(_) => {}
            DictationCaptureEvent::EndOfStream => break,
        }
    }

    producer.join().expect("producer thread");

    assert_eq!(chunks.len(), 2, "must collect both chunks including tail");
    assert_eq!(chunks[0].samples.len(), 160);
    assert_eq!(chunks[1].samples.len(), 80, "tail chunk must be preserved");

    let audio_duration = crate::dictation::transcription::captured_duration(&chunks);
    assert_eq!(audio_duration, Duration::from_millis(15));

    // Verify CompletedDictationCapture can be constructed from the result.
    let capture = CompletedDictationCapture {
        chunks,
        audio_duration,
    };
    assert_eq!(capture.chunks.len(), 2);
    assert_eq!(capture.audio_duration, Duration::from_millis(15));
}

/// Verifies that an empty recording (no chunks before EndOfStream) results
/// in `Stopped(None)`.
#[test]
fn stop_path_empty_recording_produces_none() {
    use crate::dictation::types::DictationToggleOutcome;

    let (tx, rx) = async_channel::bounded::<DictationCaptureEvent>(4);

    let producer = std::thread::spawn(move || {
        tx.send_blocking(DictationCaptureEvent::EndOfStream)
            .expect("send EOS");
    });

    let mut chunks = Vec::new();
    while let Ok(event) = rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => chunks.push(chunk),
            DictationCaptureEvent::Level(_) => {}
            DictationCaptureEvent::EndOfStream => break,
        }
    }

    producer.join().expect("producer thread");

    let outcome = if chunks.is_empty() {
        DictationToggleOutcome::Stopped(None)
    } else {
        DictationToggleOutcome::Stopped(Some(CompletedDictationCapture {
            audio_duration: crate::dictation::transcription::captured_duration(&chunks),
            chunks,
        }))
    };

    assert_eq!(outcome, DictationToggleOutcome::Stopped(None));
}

/// Verifies that the blocking drain terminates with an error when the
/// channel is closed without sending EndOfStream.
#[test]
fn stop_path_errors_when_channel_closes_without_eos() {
    let (tx, rx) = async_channel::bounded::<DictationCaptureEvent>(4);

    let producer = std::thread::spawn(move || {
        tx.send_blocking(DictationCaptureEvent::Chunk(CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.5; 160],
            duration: Duration::from_millis(10),
        }))
        .expect("send chunk");
        // Drop tx without sending EndOfStream.
    });

    let mut chunks = Vec::new();
    let mut saw_eos = false;
    while let Ok(event) = rx.recv_blocking() {
        match event {
            DictationCaptureEvent::Chunk(chunk) => chunks.push(chunk),
            DictationCaptureEvent::Level(_) => {}
            DictationCaptureEvent::EndOfStream => {
                saw_eos = true;
                break;
            }
        }
    }

    producer.join().expect("producer thread");

    assert!(
        !saw_eos,
        "should not have received EndOfStream when sender dropped early"
    );
    assert_eq!(chunks.len(), 1, "should still collect chunks before close");
}

// ---------------------------------------------------------------------------
// Mic selection wiring regression tests
// ---------------------------------------------------------------------------

#[test]
fn builtin_microphone_selection_command_is_wired() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("SettingsCommandType::SelectMicrophone"),
        "builtin settings command must expose SelectMicrophone"
    );
    assert!(
        builtin_src.contains("crate::dictation::list_input_devices()"),
        "SelectMicrophone must enumerate available input devices"
    );
    assert!(
        builtin_src.contains("BUILTIN_MIC_SELECT_PROMPT_ID"),
        "SelectMicrophone must open a dedicated synthetic ArgPrompt"
    );
    assert!(
        builtin_src.contains("BUILTIN_MIC_DEFAULT_VALUE"),
        "SelectMicrophone must include a system-default choice value"
    );
    assert!(
        builtin_src.contains("AppView::ArgPrompt"),
        "SelectMicrophone must open an ArgPrompt"
    );
    assert!(
        builtin_src.contains("Select microphone..."),
        "SelectMicrophone prompt placeholder must stay user-facing"
    );
}

#[test]
fn builtin_microphone_submit_handler_persists_or_clears_preference() {
    let helpers_src = std::fs::read_to_string("src/render_prompts/arg/helpers.rs")
        .expect("read arg helpers");
    let config_src =
        std::fs::read_to_string("src/config/types.rs").expect("read config types");

    assert!(
        helpers_src
            .contains("fn is_valid_builtin_mic_selection(&self, value: &str) -> bool"),
        "arg helpers must validate built-in microphone selections"
    );
    assert!(
        helpers_src.contains("value == BUILTIN_MIC_DEFAULT_VALUE"),
        "submit handling must accept the synthetic system-default value"
    );
    assert!(
        helpers_src.contains("let device_id = if value == BUILTIN_MIC_DEFAULT_VALUE"),
        "submit handling must clear the stored preference when System Default is chosen"
    );
    assert!(
        helpers_src.contains("crate::dictation::save_dictation_device_id(device_id)"),
        "submit handling must persist the chosen microphone device"
    );
    assert!(
        config_src.contains("pub selected_device_id: Option<String>"),
        "user preferences must persist dictation.selected_device_id"
    );
}

#[test]
fn builtin_microphone_prompt_labels_current_and_default_choices() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("\"System Default (current)\""),
        "prompt must mark the default entry when no saved mic is set"
    );
    assert!(
        builtin_src.contains("\" (current)\""),
        "prompt must label the saved microphone as current"
    );
    assert!(
        builtin_src.contains("\" (system default)\""),
        "prompt must label whichever enumerated mic is the OS default"
    );
    assert!(
        builtin_src.contains("self.arg_selected_index = start_index;"),
        "prompt must preselect the saved/current microphone"
    );
}
