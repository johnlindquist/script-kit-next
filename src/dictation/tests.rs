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

#[test]
fn whisper_engine_missing_model_error_names_attempted_path() {
    use crate::dictation::transcription::{DictationTranscriptionConfig, WhisperDictationEngine};

    let explicit_path = std::path::PathBuf::from("/nonexistent/dir/model.bin");
    let config = DictationTranscriptionConfig {
        model_path: explicit_path.clone(),
        ..Default::default()
    };
    let err_msg = WhisperDictationEngine::new(&config)
        .unwrap_err()
        .to_string();
    assert!(
        err_msg.contains(explicit_path.to_str().unwrap()),
        "error must name the attempted model path, got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Model path resolution tests
// ---------------------------------------------------------------------------

#[test]
fn default_model_path_is_absolute() {
    let config = DictationTranscriptionConfig::default();
    assert!(
        config.model_path.is_absolute(),
        "default model path must be absolute, got: {}",
        config.model_path.display()
    );
}

#[test]
fn default_model_path_lives_under_kit_path() {
    use crate::setup::get_kit_path;
    let config = DictationTranscriptionConfig::default();
    let kit_path = get_kit_path();
    assert!(
        config.model_path.starts_with(&kit_path),
        "default model path must be under kit_path ({}), got: {}",
        kit_path.display(),
        config.model_path.display()
    );
}

#[test]
fn resolve_default_model_path_matches_config_default() {
    use crate::dictation::transcription::resolve_default_model_path;
    let config = DictationTranscriptionConfig::default();
    assert_eq!(
        config.model_path,
        resolve_default_model_path(),
        "DictationTranscriptionConfig::default().model_path must equal resolve_default_model_path()"
    );
}

#[test]
fn resolve_default_model_path_ends_with_expected_filename() {
    use crate::dictation::transcription::resolve_default_model_path;
    let path = resolve_default_model_path();
    assert!(
        path.ends_with("models/whisper-medium-q4_1.bin"),
        "resolved path must end with models/whisper-medium-q4_1.bin, got: {}",
        path.display()
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
        runtime_src.contains("resolve_selected_input_device"),
        "runtime must delegate selection to resolve_selected_input_device"
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
        builtin_src.contains("crate::dictation::list_input_device_menu_items("),
        "SelectMicrophone must load rows from the shared menu-item helper"
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
    let helpers_src =
        std::fs::read_to_string("src/render_prompts/arg/helpers.rs").expect("read arg helpers");
    let config_src = std::fs::read_to_string("src/config/types.rs").expect("read config types");

    assert!(
        helpers_src.contains("fn is_valid_builtin_mic_selection(&self, value: &str) -> bool"),
        "arg helpers must validate built-in microphone selections"
    );
    assert!(
        helpers_src.contains("value == BUILTIN_MIC_DEFAULT_VALUE"),
        "submit handling must accept the synthetic system-default value"
    );
    assert!(
        helpers_src.contains("crate::dictation::apply_device_selection(&action)"),
        "submit handling must persist choices through the shared apply_device_selection helper"
    );
    assert!(
        config_src.contains("pub selected_device_id: Option<String>"),
        "user preferences must persist dictation.selected_device_id"
    );
}

#[test]
fn builtin_microphone_prompt_labels_current_and_preselects() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("format!(\"{} (current)\", item.title)"),
        "prompt must label the selected microphone as current"
    );
    assert!(
        builtin_src.contains("self.arg_selected_index = start_index;"),
        "prompt must preselect the saved/current microphone"
    );
}

#[test]
fn builtin_microphone_prompt_delegates_to_shared_menu_items() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("crate::dictation::list_input_device_menu_items("),
        "SelectMicrophone must delegate to the shared menu-item builder"
    );
    assert!(
        builtin_src.contains("item.is_selected"),
        "SelectMicrophone must use the shared is_selected flag for stale-device fallback"
    );
}

// ---------------------------------------------------------------------------
// Delivery contract tests: prompt-first, frontmost-app-second
// ---------------------------------------------------------------------------

/// Prove that `handle_dictation_transcript` checks `try_set_prompt_input`
/// BEFORE calling `paste_text`, so an active prompt always wins.
#[test]
fn delivery_checks_prompt_input_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let prompt_pos = src
        .find("try_set_prompt_input")
        .expect("handle_dictation_transcript must call try_set_prompt_input");
    let paste_pos = src
        .find("paste_text")
        .expect("handle_dictation_transcript must call paste_text");

    assert!(
        prompt_pos < paste_pos,
        "try_set_prompt_input (byte {prompt_pos}) must appear before paste_text (byte {paste_pos}) \
         — active prompt takes priority over frontmost app paste"
    );
}

/// Prove that when `try_set_prompt_input` returns true, the destination is
/// `ActivePrompt` and no paste fallback occurs.
#[test]
fn delivery_active_prompt_sets_correct_destination() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    // The delivery block must assign ActivePrompt when try_set_prompt_input succeeds.
    assert!(
        src.contains("DictationDestination::ActivePrompt"),
        "delivery must use DictationDestination::ActivePrompt when prompt accepts input"
    );

    // Verify ActivePrompt is set in the if-true branch (before paste_text).
    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];
    let active_prompt_pos = handler_src
        .find("DictationDestination::ActivePrompt")
        .expect("ActivePrompt must be in handler");
    let paste_pos = handler_src
        .find("paste_text")
        .expect("paste_text must be in handler");
    assert!(
        active_prompt_pos < paste_pos,
        "ActivePrompt destination must be assigned BEFORE the paste fallback branch"
    );
}

/// Prove that when `try_set_prompt_input` returns false, delivery falls
/// through to `paste_text` and sets `FrontmostApp` on success.
#[test]
fn delivery_frontmost_app_sets_correct_destination() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    // paste_text must be in an else branch after try_set_prompt_input
    assert!(
        handler_src.contains("paste_text"),
        "handler must call paste_text as fallback"
    );
    assert!(
        handler_src.contains("DictationDestination::FrontmostApp"),
        "handler must set FrontmostApp destination on successful paste"
    );
}

/// Prove that paste failures are surfaced (logged + toast) and not silently
/// dropped.  The overlay is closed before pasting (for focus handoff), so
/// failures cannot use the overlay — they show a toast instead.
#[test]
fn delivery_paste_failure_surfaces_error() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    // The Err branch of paste_text must log the failure.
    let paste_pos = handler_src
        .find("paste_text")
        .expect("handler must call paste_text");
    let after_paste = &handler_src[paste_pos..];

    assert!(
        after_paste.contains("Failed to paste dictation transcript"),
        "paste failure must log an error describing what failed"
    );
    assert!(
        after_paste.contains("show_error_toast"),
        "paste failure must show an error toast to the user"
    );
}

/// Prove that the frontmost-app paste path hides the main window and closes
/// the dictation overlay BEFORE pasting, so macOS returns keyboard focus to
/// the target app before the CGEvent Cmd+V fires.
///
/// The close + hide logic now lives inside `yield_focus_for_dictation_paste`,
/// which the frontmost branch calls before `paste_text`.
#[test]
fn delivery_frontmost_app_hides_window_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    // The frontmost branch calls the helper which does close + hide.
    let yield_pos = frontmost_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost branch must call yield_focus_for_dictation_paste");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost branch must call paste_text");

    assert!(
        yield_pos < paste_pos,
        "yield_focus_for_dictation_paste (byte {yield_pos}) must appear before paste_text (byte {paste_pos})"
    );

    // The helper itself must close the overlay and hide the main window.
    let helper_start = src
        .find("fn yield_focus_for_dictation_paste")
        .expect("helper must exist");
    let helper_src = &src[helper_start..helper_start + 1200.min(src.len() - helper_start)];
    assert!(
        helper_src.contains("close_dictation_overlay"),
        "helper must close dictation overlay"
    );
    assert!(
        helper_src.contains("defer_hide_main_window"),
        "helper must call defer_hide_main_window"
    );
}

/// Prove that transcription errors also surface as Failed phase.
#[test]
fn delivery_transcription_error_surfaces_as_failed_phase() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    // The Err(error) arm must show Failed overlay.
    assert!(
        handler_src.contains("Err(error)"),
        "handler must match on Err(error) for transcription failures"
    );
    assert!(
        handler_src.contains("Transcription failed"),
        "transcription error must be logged"
    );
}

/// Prove delivery logic stays in builtin_execution, not in dictation/runtime.
#[test]
fn delivery_logic_not_in_runtime() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        !runtime_src.contains("try_set_prompt_input"),
        "runtime must not call try_set_prompt_input — delivery is the caller's job"
    );
    assert!(
        !runtime_src.contains("paste_text"),
        "runtime must not call paste_text — delivery is the caller's job"
    );
    assert!(
        !runtime_src.contains("TextInjector"),
        "runtime must not reference TextInjector — delivery is the caller's job"
    );
    assert!(
        !runtime_src.contains("handle_dictation_transcript"),
        "runtime must not define or call handle_dictation_transcript"
    );
}

/// Prove that `try_set_prompt_input` covers the main prompt types that
/// should accept dictated text.
#[test]
fn try_set_prompt_input_covers_key_prompt_views() {
    let src = std::fs::read_to_string("src/app_impl/ui_window.rs").expect("read ui_window.rs");

    let fn_start = src
        .find("fn try_set_prompt_input")
        .expect("try_set_prompt_input must exist");
    let fn_src = &src[fn_start..fn_start + 3000.min(src.len() - fn_start)];

    for view in &[
        "AppView::ArgPrompt",
        "AppView::MiniPrompt",
        "AppView::MicroPrompt",
        "AppView::PathPrompt",
        "AppView::SelectPrompt",
    ] {
        assert!(
            fn_src.contains(view),
            "try_set_prompt_input must handle {view}"
        );
    }

    // Must return false for unhandled views (the _ => false arm).
    assert!(
        fn_src.contains("_ => false"),
        "try_set_prompt_input must return false for unhandled views"
    );
}

/// Prove that the silent-audio path (Ok(None)) does not show an error and
/// closes the overlay quickly without surfacing a failure.
#[test]
fn delivery_silent_audio_closes_without_error() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    // The Ok(None) arm must schedule close without showing Failed.
    assert!(
        handler_src.contains("Ok(None)"),
        "handler must match Ok(None) for silent audio"
    );

    // Ok(None) must not produce a Failed phase.
    let none_pos = handler_src
        .find("Ok(None)")
        .expect("Ok(None) arm must exist");
    // Find the next match arm boundary (either Ok(Some(..)) or Err(..))
    let next_arm = handler_src[none_pos..]
        .find("Err(error)")
        .unwrap_or(handler_src.len() - none_pos);
    let none_arm = &handler_src[none_pos..none_pos + next_arm];

    assert!(
        !none_arm.contains("DictationSessionPhase::Failed"),
        "silent audio must not surface as Failed — it should close quietly"
    );
}

// ---------------------------------------------------------------------------
// Stale mic preference self-heal regression test
// ---------------------------------------------------------------------------

#[test]
fn runtime_clears_stale_mic_preference_on_missing_device() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    // The fallback branch must attempt to clear the stale preference.
    assert!(
        runtime_src.contains("save_dictation_device_id(None)"),
        "resolve_preferred_device must clear stale preference when saved device is missing"
    );

    // If clearing fails, runtime must log a warning and continue.
    assert!(
        runtime_src.contains("Failed to clear stale microphone preference"),
        "runtime must warn when clearing stale preference fails"
    );

    // The runtime must delegate to resolve_selected_input_device, which
    // handles the system-default fallback internally.
    assert!(
        runtime_src.contains("resolve_selected_input_device"),
        "resolve_preferred_device must delegate to resolve_selected_input_device for fallback"
    );
}

// ---------------------------------------------------------------------------
// Hotkey routing: dictation hotkey uses builtin toggle, not a duplicate path
// ---------------------------------------------------------------------------

#[test]
fn dictation_hotkey_routes_through_builtin_toggle_flow() {
    // Verify both entry-point files have a dictation hotkey listener that
    // routes through execute_by_command_id_or_path("builtin-dictation"),
    // ensuring one toggle path instead of a duplicate dictation implementation.
    for (label, path) in [
        (
            "runtime_tray_hotkeys.rs",
            "src/main_entry/runtime_tray_hotkeys.rs",
        ),
        ("app_run_setup.rs", "src/main_entry/app_run_setup.rs"),
    ] {
        let src = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {label}"));

        assert!(
            src.contains("dictation_hotkey_channel"),
            "{label} must consume the dictation hotkey channel"
        );
        assert!(
            src.contains("builtin-dictation"),
            "{label} must route dictation hotkey through builtin-dictation command"
        );
        // Must NOT contain a second toggle_dictation call — only the builtin path owns that.
        let hotkey_section_start = src
            .find("Dictation hotkey listener")
            .unwrap_or_else(|| panic!("{label} must have a Dictation hotkey listener section"));
        let hotkey_section_end = src[hotkey_section_start..]
            .find(".detach()")
            .unwrap_or(src.len() - hotkey_section_start);
        let hotkey_section = &src[hotkey_section_start..hotkey_section_start + hotkey_section_end];

        assert!(
            !hotkey_section.contains("toggle_dictation()"),
            "{label} dictation hotkey listener must NOT call toggle_dictation() directly — \
             it must route through the builtin execution path"
        );
        assert!(
            !hotkey_section.contains("open_dictation_overlay"),
            "{label} dictation hotkey listener must NOT call open_dictation_overlay directly — \
             it must route through the builtin execution path"
        );
    }
}

#[test]
fn dictation_hotkey_channel_not_dead_code() {
    // The hotkey channel must be consumed (receiver side) in at least one entry point.
    let runtime = std::fs::read_to_string("src/main_entry/runtime_tray_hotkeys.rs")
        .expect("read runtime_tray_hotkeys.rs");
    let setup =
        std::fs::read_to_string("src/main_entry/app_run_setup.rs").expect("read app_run_setup.rs");

    let has_receiver = runtime.contains("dictation_hotkey_channel().1.recv()")
        || setup.contains("dictation_hotkey_channel().1.recv()");
    assert!(
        has_receiver,
        "dictation hotkey channel receiver must be consumed in at least one entry point"
    );
}

// ---------------------------------------------------------------------------
// Finished-label formatting tests
// ---------------------------------------------------------------------------

#[test]
fn finished_label_formats_short_and_long_transcripts() {
    let short: gpui::SharedString = "hello world".into();
    assert_eq!(
        super::window::finished_label(&short).to_string(),
        "Done · hello world"
    );

    let long: gpui::SharedString = "abcdefghijklmnopqrstuvwxyz0123456789".into();
    assert_eq!(
        super::window::finished_label(&long).to_string(),
        "Done · abcdefghijklmnopqrstuvwxyz01…"
    );
}

#[test]
fn finished_label_returns_done_for_empty_transcript() {
    let empty: gpui::SharedString = "".into();
    assert_eq!(
        super::window::finished_label(&empty).to_string(),
        "Done"
    );

    let whitespace: gpui::SharedString = "   ".into();
    assert_eq!(
        super::window::finished_label(&whitespace).to_string(),
        "Done"
    );
}

#[test]
fn frontmost_app_delivery_shows_done_state_before_close_and_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    // Both delivery paths (prompt + frontmost-app) must show Finished.
    let finished_count = handler_src
        .match_indices("DictationSessionPhase::Finished")
        .count();
    assert!(
        finished_count >= 2,
        "handle_dictation_transcript must use Finished for both prompt and frontmost-app delivery paths"
    );

    // Scope to the frontmost-app else branch: starts at the 75ms timer
    // (unique to that branch) and extends through paste_text.
    let done_pause_pos = handler_src
        .find("timer(std::time::Duration::from_millis(75))")
        .expect("frontmost-app delivery must wait briefly so the done state is visible");

    // Search forward from the done-pause for yield-focus (which closes the
    // overlay internally) and paste, in order.
    let after_pause = &handler_src[done_pause_pos..];
    let yield_offset = after_pause
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app delivery must yield focus (close overlay) after the done-state pause");
    let paste_offset = after_pause
        .find("paste_text")
        .expect("frontmost-app delivery must paste after the done-state pause");

    assert!(
        yield_offset < paste_offset,
        "frontmost-app delivery must yield focus before pasting (yield at {yield_offset}, paste at {paste_offset})"
    );
}

// ---------------------------------------------------------------------------
// Ordering regressions: frontmost-app delivery sequence
// ---------------------------------------------------------------------------

/// Extract the frontmost-app else branch from handle_dictation_transcript.
fn frontmost_dictation_delivery_branch(handler_src: &str) -> &str {
    let prompt_if = handler_src
        .find("if self.try_set_prompt_input")
        .expect("handler must branch on prompt delivery");
    let else_offset = handler_src[prompt_if..]
        .find("} else {")
        .expect("handler must have a frontmost-app else branch");
    &handler_src[prompt_if + else_offset..]
}

#[test]
fn delivery_frontmost_app_shows_finished_before_close_and_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    let finished_pos = frontmost_src
        .find("DictationSessionPhase::Finished")
        .expect("frontmost-app branch must render Finished");
    let yield_pos = frontmost_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app branch must yield focus (close overlay + hide)");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must paste transcript");

    assert!(
        finished_pos < yield_pos,
        "Finished phase (byte {finished_pos}) must appear before yield_focus_for_dictation_paste (byte {yield_pos})"
    );
    assert!(
        yield_pos < paste_pos,
        "yield_focus_for_dictation_paste (byte {yield_pos}) must appear before paste_text (byte {paste_pos})"
    );
}

#[test]
fn delivery_frontmost_app_waits_before_closing_and_pasting() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    let done_timer_pos = frontmost_src
        .find("from_millis(75)")
        .expect("frontmost-app branch must wait briefly on Finished state");
    let yield_pos = frontmost_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app branch must yield focus (close overlay + hide)");
    let focus_timer_pos = frontmost_src
        .find("from_millis(100)")
        .expect("frontmost-app branch must wait for focus to settle");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must paste transcript");

    assert!(
        done_timer_pos < yield_pos,
        "Finished-state timer (byte {done_timer_pos}) must appear before yield_focus_for_dictation_paste (byte {yield_pos})"
    );
    assert!(
        yield_pos < focus_timer_pos,
        "yield_focus_for_dictation_paste (byte {yield_pos}) must appear before focus-settle timer (byte {focus_timer_pos})"
    );
    assert!(
        focus_timer_pos < paste_pos,
        "focus-settle timer (byte {focus_timer_pos}) must appear before paste_text (byte {paste_pos})"
    );
}

#[test]
fn delivery_frontmost_app_aborts_paste_on_focus_yield_failure() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    // The focus-yield result must be checked (not discarded with `let _ =`).
    assert!(
        frontmost_src.contains("yield_focus_result"),
        "frontmost-app branch must name the focus-yield result for error checking"
    );

    // On failure, must show error toast and schedule cleanup before returning.
    let err_check_pos = frontmost_src
        .find("if let Err(error) = yield_focus_result")
        .expect("frontmost-app branch must check yield_focus_result for errors");
    let show_error_pos = frontmost_src
        .find("show_error_toast")
        .expect("frontmost-app branch must show error toast on focus-yield failure");
    let cleanup_in_err = frontmost_src
        .find("schedule_dictation_transcriber_cleanup");

    assert!(
        err_check_pos < show_error_pos.min(cleanup_in_err.unwrap_or(usize::MAX)),
        "error check must come before toast and cleanup"
    );
}

// ---------------------------------------------------------------------------
// Overlay contract: behavioral tests for helpers & constants
// ---------------------------------------------------------------------------

#[test]
fn overlay_elapsed_formatter_uses_mm_ss() {
    assert_eq!(
        super::window::format_elapsed(Duration::from_secs(0)).to_string(),
        "0:00"
    );
    assert_eq!(
        super::window::format_elapsed(Duration::from_secs(65)).to_string(),
        "1:05"
    );
    assert_eq!(
        super::window::format_elapsed(Duration::from_secs(600)).to_string(),
        "10:00"
    );
}

#[test]
fn overlay_waveform_height_matches_compact_contract() {
    assert!(
        (super::window::waveform_bar_height(0.0) - 4.0).abs() < 0.001,
        "silent bar height must be 4.0"
    );
    assert!(
        (super::window::waveform_bar_height(1.0) - 20.0).abs() < 0.001,
        "peak bar height must be 20.0"
    );
    let mid = super::window::waveform_bar_height(0.5);
    assert!(
        mid > 4.0 && mid < 20.0,
        "mid-level bar height must stay inside compact overlay bounds, got {mid}"
    );
}

#[test]
fn overlay_waveform_opacity_stays_clamped() {
    assert!(
        (super::window::waveform_bar_opacity(0.0) - 0.3).abs() < 0.001,
        "silent bar opacity must be 0.3"
    );
    assert!(
        (super::window::waveform_bar_opacity(1.0) - 1.0).abs() < 0.001,
        "peak bar opacity must be 1.0"
    );
    let mid = super::window::waveform_bar_opacity(0.5);
    assert!(
        (0.3..=1.0).contains(&mid),
        "bar opacity must remain between 0.3 and 1.0, got {mid}"
    );
}

#[test]
fn overlay_dot_and_window_constants_match_target_contract() {
    use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_SELECTED};

    assert_eq!(super::window::OVERLAY_WIDTH_PX, 220.0);
    assert_eq!(super::window::OVERLAY_HEIGHT_PX, 36.0);
    assert_eq!(super::window::OVERLAY_RADIUS_PX, 18.0);
    assert_eq!(super::window::STATUS_TEXT_SIZE_PX, 11.5);
    assert_eq!(super::window::WAVEFORM_BAR_COUNT, 9);
    assert_eq!(super::window::TRANSCRIBING_DOT_COUNT, 3);
    assert_eq!(
        super::window::transcribing_dot_opacities(),
        [OPACITY_SELECTED, OPACITY_ACTIVE, OPACITY_SELECTED]
    );
}

#[test]
fn overlay_has_sound_detects_active_audio() {
    let silent = [0.0_f32; super::window::WAVEFORM_BAR_COUNT];
    assert!(
        !super::window::has_sound(&silent),
        "all-zero bars must be silent"
    );

    let mut loud = [0.0_f32; super::window::WAVEFORM_BAR_COUNT];
    loud[4] = 0.5;
    assert!(
        super::window::has_sound(&loud),
        "bar above threshold must count as sound"
    );
}

#[test]
fn dictation_overlay_derives_colors_from_theme_tokens() {
    let source =
        std::fs::read_to_string("src/dictation/window.rs").expect("read dictation window.rs");

    assert!(
        source.contains("theme.colors.background.main.with_opacity"),
        "overlay surface must derive from theme background"
    );
    assert!(
        source.contains("theme.colors.ui.border.with_opacity"),
        "overlay border must derive from theme border token"
    );
    assert!(
        source.contains("theme.colors.ui.success"),
        "active waveform / transcribing state must use theme success color"
    );
    assert!(
        source.contains("theme.colors.ui.error"),
        "inactive waveform must use theme error color"
    );
    assert!(
        source.contains("theme.colors.text.primary.with_opacity"),
        "finished text must derive from theme primary text color"
    );
    assert!(
        source.contains("theme.colors.text.muted.with_opacity"),
        "timer and muted text must derive from theme muted text color"
    );
}

#[test]
fn dictation_overlay_close_propagates_window_update_failure() {
    let source =
        std::fs::read_to_string("src/dictation/window.rs").expect("read dictation window.rs");

    let close_start = source
        .find("pub fn close_dictation_overlay")
        .expect("close_dictation_overlay must exist");
    let close_src = &source[close_start..close_start + 900.min(source.len() - close_start)];

    assert!(
        close_src.contains(".update("),
        "close_dictation_overlay must call handle.update"
    );
    assert!(
        !close_src.contains("let _ = handle.update"),
        "close_dictation_overlay must not discard window close failures"
    );
    assert!(
        close_src.contains("failed to close dictation overlay window"),
        "close_dictation_overlay must propagate update failure with context"
    );
}

#[test]
fn delivery_frontmost_app_uses_error_propagating_focus_helper() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let helper_start = src
        .find("fn yield_focus_for_dictation_paste")
        .expect("builtin_execution.rs must define yield_focus_for_dictation_paste");
    let helper_src = &src[helper_start..helper_start + 1200.min(src.len() - helper_start)];

    assert!(
        helper_src.contains("close_dictation_overlay(cx)")
            && helper_src.contains("failed to close dictation overlay before paste"),
        "focus helper must propagate close_dictation_overlay failure with context"
    );
    assert!(
        helper_src.contains("platform::defer_hide_main_window(cx)"),
        "focus helper must still hide the main window before frontmost-app paste"
    );

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];
    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    assert!(
        frontmost_src.contains("this.yield_focus_for_dictation_paste(cx)"),
        "frontmost-app delivery must call yield_focus_for_dictation_paste"
    );

    // Scope to the async block that does close+paste (up to paste_text) to
    // avoid matching `let _ =` in unrelated arms like Stopped(None).
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost branch must paste");
    let pre_paste = &frontmost_src[..paste_pos];
    assert!(
        !pre_paste.contains("let _ = crate::dictation::close_dictation_overlay(cx)"),
        "frontmost-app delivery must not discard close_dictation_overlay errors before paste"
    );
}

// ---------------------------------------------------------------------------
// Strong source-level ordering proof for dictation focus handoff
// ---------------------------------------------------------------------------

/// Extract the body of `handle_dictation_transcript` from the source text.
fn dictation_handler_source(src: &str) -> &str {
    let start = src
        .find("fn handle_dictation_transcript")
        .expect("handle_dictation_transcript must exist");
    &src[start..]
}

/// Extract the body of `yield_focus_for_dictation_paste` from the source text.
/// Uses `schedule_dictation_overlay_close` as the end boundary.
fn dictation_yield_focus_helper_source(src: &str) -> &str {
    let start = src
        .find("fn yield_focus_for_dictation_paste(")
        .expect("yield_focus_for_dictation_paste must exist");
    let tail = &src[start..];
    let end = tail
        .find("fn schedule_dictation_overlay_close(")
        .expect(
            "yield_focus_for_dictation_paste must be followed by schedule_dictation_overlay_close",
        );
    &tail[..end]
}

/// Extract the frontmost-app else branch from `handle_dictation_transcript`.
/// Uses `Ok(None)` as the end boundary.
fn dictation_frontmost_paste_source(src: &str) -> &str {
    let handler_src = dictation_handler_source(src);
    let else_pos = handler_src
        .find("} else {")
        .expect("handle_dictation_transcript must have a frontmost-app else branch");
    let tail = &handler_src[else_pos..];
    let end = tail
        .find("Ok(None)")
        .expect("frontmost-app else branch must end before the Ok(None) arm");
    &tail[..end]
}

#[test]
fn delivery_focus_helper_closes_overlay_before_hiding_main_window() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let helper_src = dictation_yield_focus_helper_source(&src);

    let close_pos = helper_src
        .find("close_dictation_overlay")
        .expect("focus helper must close dictation overlay");
    let hide_pos = helper_src
        .find("defer_hide_main_window")
        .expect("focus helper must defer main-window hide");

    assert!(
        close_pos < hide_pos,
        "close_dictation_overlay (byte {close_pos}) must appear before \
         defer_hide_main_window (byte {hide_pos}) inside yield_focus_for_dictation_paste"
    );
}

#[test]
fn delivery_frontmost_app_calls_focus_helper_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let frontmost_src = dictation_frontmost_paste_source(&src);

    let yield_pos = frontmost_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app branch must call yield_focus_for_dictation_paste");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must call paste_text");

    assert!(
        yield_pos < paste_pos,
        "yield_focus_for_dictation_paste (byte {yield_pos}) must appear before \
         paste_text (byte {paste_pos}) in the frontmost-app branch"
    );
}

#[test]
fn delivery_focus_yield_failure_surfaces_and_returns_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let frontmost_src = dictation_frontmost_paste_source(&src);

    assert!(
        frontmost_src.contains("failed to update app state before paste"),
        "frontmost-app path must wrap update failures with a dictation-specific context"
    );
    assert!(
        frontmost_src.contains("Dictation paste failed before paste step"),
        "frontmost-app path must show a pre-paste failure toast"
    );

    let err_pos = frontmost_src
        .find("if let Err(error) = yield_focus_result")
        .expect("frontmost-app path must check yield_focus_result");
    let return_pos = err_pos
        + frontmost_src[err_pos..]
            .find("return;")
            .expect("pre-paste failure path must return before paste");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app path must eventually call paste_text");

    assert!(
        err_pos < paste_pos,
        "yield_focus_result error handling must run before paste_text"
    );
    assert!(
        return_pos < paste_pos,
        "pre-paste failure path must return before paste_text"
    );
}

// ---------------------------------------------------------------------------
// Device selection helpers
// ---------------------------------------------------------------------------

use crate::dictation::device::{
    build_device_menu_items, resolve_selected_input_device, DictationDeviceMenuItem,
    DictationDeviceSelectionAction,
};
use crate::dictation::types::{DictationDeviceId, DictationDeviceInfo};

fn device(id: &str, name: &str, is_default: bool) -> DictationDeviceInfo {
    DictationDeviceInfo {
        id: DictationDeviceId(id.to_string()),
        name: name.to_string(),
        is_default,
    }
}

#[test]
fn resolve_selected_input_device_prefers_saved_device() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let selected = resolve_selected_input_device(&devices, Some("usb")).unwrap();
    assert_eq!(selected.id.0, "usb");
    assert_eq!(selected.name, "USB Microphone");
}

#[test]
fn resolve_selected_input_device_falls_back_to_default_for_stale_saved_id() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let selected = resolve_selected_input_device(&devices, Some("missing")).unwrap();
    assert_eq!(selected.id.0, "builtin");
    assert!(selected.is_default);
}

#[test]
fn resolve_selected_input_device_returns_default_when_no_preference() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let selected = resolve_selected_input_device(&devices, None).unwrap();
    assert_eq!(selected.id.0, "builtin");
}

#[test]
fn resolve_selected_input_device_returns_none_for_empty_list() {
    let selected = resolve_selected_input_device(&[], Some("anything"));
    assert!(selected.is_none());
}

#[test]
fn build_device_menu_items_marks_system_default_when_saved_device_is_missing() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let items = build_device_menu_items(&devices, Some("missing"));
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].title, "System Default");
    assert!(items[0].is_selected);
    assert!(!items[1].is_selected);
    assert!(!items[2].is_selected);
}

#[test]
fn build_device_menu_items_marks_saved_device_when_present() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let items = build_device_menu_items(&devices, Some("usb"));
    assert_eq!(items.len(), 3);
    assert!(!items[0].is_selected, "System Default should not be selected");
    assert!(!items[1].is_selected, "builtin should not be selected");
    assert!(items[2].is_selected, "USB Mic should be selected");
}

#[test]
fn build_device_menu_items_marks_system_default_when_no_preference() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let items = build_device_menu_items(&devices, None);
    assert!(items[0].is_selected);
    assert!(!items[1].is_selected);
    assert!(!items[2].is_selected);
}

#[test]
fn build_device_menu_items_default_row_subtitle_includes_device_name() {
    let devices = vec![device("builtin", "MacBook Pro Microphone", true)];
    let items = build_device_menu_items(&devices, None);
    assert!(
        items[0].subtitle.contains("MacBook Pro Microphone"),
        "subtitle should name the default device"
    );
}

#[test]
fn build_device_menu_items_labels_default_device_with_dot_suffix() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let items = build_device_menu_items(&devices, None);
    assert!(
        items[1].title.contains("\u{00b7} default"),
        "default device row should have '\u{00b7} default' suffix"
    );
    assert_eq!(items[2].title, "USB Microphone");
}

#[test]
fn build_device_menu_items_actions_match_device_ids() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let items = build_device_menu_items(&devices, None);
    assert_eq!(
        items[0].action,
        DictationDeviceSelectionAction::UseSystemDefault
    );
    assert_eq!(
        items[1].action,
        DictationDeviceSelectionAction::UseDevice(DictationDeviceId("builtin".to_string()))
    );
    assert_eq!(
        items[2].action,
        DictationDeviceSelectionAction::UseDevice(DictationDeviceId("usb".to_string()))
    );
}
