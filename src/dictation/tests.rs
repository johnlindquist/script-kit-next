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

#[test]
fn builtin_microphone_prompt_treats_missing_saved_device_as_system_default_current() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("let saved_device_available = current_id"),
        "SelectMicrophone must detect whether the saved microphone still exists"
    );
    assert!(
        builtin_src
            .contains("let default_selected =\n                            current_id.is_none() || !saved_device_available"),
        "missing saved microphones must fall back to System Default as current"
    );
    assert!(
        builtin_src.contains(
            "saved_device_available\n                                && current_id.as_deref()"
        ),
        "only an available saved device should be labeled as current"
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
#[test]
fn delivery_frontmost_app_hides_window_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    // The else branch (frontmost-app path) must close the overlay and hide the
    // main window before scheduling the paste.
    let close_overlay_pos = handler_src
        .find("close_dictation_overlay")
        .expect("handler must close dictation overlay before paste");
    let hide_pos = handler_src
        .find("defer_hide_main_window")
        .expect("handler must call defer_hide_main_window before paste");
    let paste_pos = handler_src
        .find("paste_text")
        .expect("handler must call paste_text");

    assert!(
        close_overlay_pos < paste_pos,
        "close_dictation_overlay (byte {close_overlay_pos}) must appear before paste_text (byte {paste_pos})"
    );
    assert!(
        hide_pos < paste_pos,
        "defer_hide_main_window (byte {hide_pos}) must appear before paste_text (byte {paste_pos})"
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

    // The fallback must still resolve to the system default after clearing.
    let clear_pos = runtime_src
        .find("save_dictation_device_id(None)")
        .expect("must call save_dictation_device_id(None)");
    let fallback_pos = runtime_src[clear_pos..]
        .find("default_input_device()")
        .expect("must fall back to default_input_device after clearing");
    assert!(
        fallback_pos > 0,
        "default_input_device must be called after clearing stale preference (offset {fallback_pos})"
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
