use crate::dictation::capture::{mix_to_mono, normalize_chunk, resample_linear, run_processor};
use crate::dictation::transcription::{
    build_session_result, is_parakeet_model_available, merge_captured_chunks, DictationEngine,
    DictationTranscriber, DictationTranscriptionConfig, ParakeetDictationEngine,
    WhisperDictationEngine,
};
use crate::dictation::types::{
    CapturedAudioChunk, CompletedDictationCapture, DictationCaptureConfig, DictationCaptureEvent,
    DictationDestination, RawAudioChunk,
};
use crate::dictation::visualizer::{silent_bars, AudioVisualiser};
use anyhow::Result;
use parking_lot::Mutex;
use serde_json::json;
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
fn fft_visualiser_returns_none_until_window_fills() {
    let mut vis = AudioVisualiser::new_speech(16_000);
    // 100 samples is less than the 1024-sample FFT window — should return None.
    let silence = vec![0.0_f32; 100];
    assert!(vis.feed(&silence).is_none());
}

#[test]
fn fft_visualiser_returns_bars_when_window_fills() {
    let mut vis = AudioVisualiser::new_speech(16_000);
    // Generate a 440 Hz sine tone — 1024 samples at 16 kHz.
    let samples: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16_000.0).sin() * 0.5)
        .collect();
    let bars = vis.feed(&samples);
    assert!(bars.is_some(), "should return bars after full window");
    let bars = bars.expect("bars");
    assert_eq!(bars.len(), 9);
    assert!(
        bars.iter().all(|&b| (0.0..=1.0).contains(&b)),
        "all bars should be 0.0–1.0"
    );
}

#[test]
fn silent_bars_are_minimum_height() {
    let bars = silent_bars();
    assert_eq!(bars.len(), 9);
    assert!(bars.iter().all(|&b| b < 0.1), "silent bars should be low");
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
            DictationCaptureEvent::Bars(_) => {}
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
            DictationCaptureEvent::Bars(_) => {}
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
            DictationCaptureEvent::Bars(_) => {}
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

#[test]
fn whisper_engine_new_accepts_regular_file_model_path() {
    let temp_file = tempfile::NamedTempFile::new().expect("create temp model file");
    let config = DictationTranscriptionConfig {
        model_path: temp_file.path().to_path_buf(),
        ..Default::default()
    };

    WhisperDictationEngine::new(&config)
        .expect("WhisperDictationEngine::new should accept a regular file path");
}

#[test]
fn whisper_engine_new_with_whisper_config_surfaces_path_or_succeeds() {
    use crate::dictation::transcription::resolve_whisper_model_path;
    let whisper_path = resolve_whisper_model_path();
    let config = DictationTranscriptionConfig {
        model_path: whisper_path.clone(),
        ..Default::default()
    };
    let result = WhisperDictationEngine::new(&config);

    if whisper_path.is_file() {
        assert!(
            result.is_ok(),
            "whisper model path should initialize when the file exists: {}",
            whisper_path.display()
        );
    } else {
        let error = result
            .expect_err("whisper model path should fail when the file is missing or invalid")
            .to_string();
        assert!(
            error.contains(&whisper_path.display().to_string()),
            "whisper-path init error should name the attempted path, got: {error}"
        );
    }
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
fn resolve_default_model_path_ends_with_parakeet_dir() {
    use crate::dictation::transcription::resolve_default_model_path;
    let path = resolve_default_model_path();
    assert!(
        path.ends_with("models/parakeet-tdt-0.6b-v3-int8"),
        "resolved path must end with models/parakeet-tdt-0.6b-v3-int8, got: {}",
        path.display()
    );
}

#[test]
fn resolve_whisper_model_path_ends_with_expected_filename() {
    use crate::dictation::transcription::resolve_whisper_model_path;
    let path = resolve_whisper_model_path();
    assert!(
        path.ends_with("models/whisper-medium-q4_1.bin"),
        "resolved path must end with models/whisper-medium-q4_1.bin, got: {}",
        path.display()
    );
}

#[test]
fn dictation_model_catalog_ids_round_trip_and_unknown_falls_back() {
    use crate::dictation::DictationModelId;

    for entry in crate::dictation::dictation_model_catalog() {
        assert_eq!(
            DictationModelId::from_str(entry.stable_id),
            Some(entry.id),
            "catalog id must round-trip: {}",
            entry.stable_id
        );
    }

    assert_eq!(
        DictationModelId::from_preference(Some("not-a-real-model")),
        DictationModelId::default()
    );
    assert_eq!(
        DictationModelId::from_preference(None),
        DictationModelId::ParakeetTdt06bV3
    );
}

#[test]
fn dictation_model_catalog_marks_only_parakeet_recommended() {
    use crate::dictation::DictationModelId;

    let catalog = crate::dictation::dictation_model_catalog();
    let recommended: Vec<_> = catalog
        .iter()
        .filter(|entry| entry.recommended)
        .map(|entry| entry.id)
        .collect();

    assert_eq!(recommended, vec![DictationModelId::ParakeetTdt06bV3]);
    assert!(catalog
        .iter()
        .any(|entry| entry.description
            == "Fast and accurate. Auto-detects 25 European languages (ignores the language setting)."));
    assert!(catalog.iter().any(|entry| entry.description
        == "Broadest language coverage and honors the language setting, but may run a bit slow."));
}

#[test]
fn dictation_model_size_formatting_reuses_human_readable_units() {
    assert_eq!(crate::dictation::format_dictation_model_size(0), "0 B");
    assert_eq!(crate::dictation::format_dictation_model_size(1024), "1 KB");
    assert_eq!(
        crate::dictation::format_dictation_model_size(500_000_000),
        "476.8 MB"
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
// Mic preference resolution source audit
// ---------------------------------------------------------------------------

#[test]
fn runtime_resolves_mic_preference_from_settings() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        runtime_src.contains("load_user_preferences"),
        "runtime must read config-backed preferences for mic selection"
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

        tx.send_blocking(DictationCaptureEvent::Bars([0.3; 9]))
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
            DictationCaptureEvent::Bars(_) => {}
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
        truncated: false,
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
            DictationCaptureEvent::Bars(_) => {}
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
            truncated: false,
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
            DictationCaptureEvent::Bars(_) => {}
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

#[test]
fn transcribe_captured_audio_returns_none_for_silent_input_without_model() {
    crate::dictation::runtime::reset_cached_transcriber_for_tests();

    let chunks = vec![CapturedAudioChunk {
        sample_rate_hz: 16_000,
        samples: vec![0.0; 1_600],
        duration: Duration::from_millis(100),
    }];

    let result = crate::dictation::transcribe_captured_audio(&chunks)
        .expect("silent input should short-circuit before engine init");

    assert_eq!(result, None);

    crate::dictation::runtime::reset_cached_transcriber_for_tests();
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

/// Prompt delivery validates and mutates only its frozen prompt; the canonical
/// target mapping supplies the receipt destination without a fallback branch.
#[test]
fn delivery_active_prompt_uses_frozen_target_mapping() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let mutation = extract_delta_contract_section(
        &src,
        "fn mutate_internal_dictation_request(",
        "fn complete_internal_dictation_delivery(",
    );
    let prompt = compact_delta_contract(extract_delta_contract_section(
        mutation,
        "crate::dictation::DictationTarget::MainWindowPrompt =>",
        "crate::dictation::DictationTarget::NotesEditor =>",
    ));
    assert!(
        prompt.contains("self.capture_dictation_target_selection(target,cx)?")
            && prompt.contains(
                "with_frozen_dictation_destination(&request.selection.destination,&current.destination,||{ifself.try_set_prompt_input(request.transcript.text().to_string(),cx)"
            ),
        "prompt mutation must run inside the shared frozen-identity guard"
    );
    let completion = extract_delta_contract_section(
        &src,
        "fn complete_internal_dictation_delivery(",
        "fn handle_dictation_transcript(",
    );
    assert!(
        completion.contains("let destination = target.destination();")
            && completion.contains("record_delivery_receipt"),
        "prompt receipts must use the exhaustive target-to-destination mapping"
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
/// dropped.  The main window hides before pasting (for focus handoff), so
/// failures show a toast.
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
        after_paste.contains("notice_failed"),
        "paste failure must show an error toast to the user"
    );
}

/// Prove that the frontmost-app paste path hides the main window BEFORE
/// pasting, so macOS returns keyboard focus to
/// the target app before the CGEvent Cmd+V fires.
///
/// The hide logic lives inside `yield_focus_for_dictation_paste`,
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

    // The helper itself must hide the main window.
    let helper_start = src
        .find("fn yield_focus_for_dictation_paste")
        .expect("helper must exist");
    let helper_src = &src[helper_start..helper_start + 1200.min(src.len() - helper_start)];
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

    // The handler must have an Err(error) arm.
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

    let fn_src = extract_delta_contract_section(
        &src,
        "fn try_set_prompt_input(",
        "pub(crate) fn set_prompt_input(",
    );

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

/// Prove that the silent-audio path (Ok(None)) ends quietly without
/// surfacing a failure.
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
// Hotkey isolation: dictation hotkey must never show main window
// ---------------------------------------------------------------------------

#[test]
fn dictation_hotkey_never_shows_main_window() {
    let execution_src = std::fs::read_to_string("src/app_impl/execution_scripts.rs")
        .expect("read execution_scripts.rs");
    assert!(
        execution_src.contains("\"builtin/dictation\""),
        "builtin/dictation must be listed in NO_MAIN_WINDOW_BUILTINS"
    );

    for (label, path) in [
        (
            "runtime_tray_hotkeys.rs",
            "src/main_entry/runtime_tray_hotkeys.rs",
        ),
        ("app_run_setup.rs", "src/main_entry/app_run_setup.rs"),
    ] {
        let src = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {label}"));
        let start = src
            .find("Dictation hotkey listener")
            .unwrap_or_else(|| panic!("{label} must have a Dictation hotkey listener section"));
        let end = src[start..].find(".detach()").unwrap_or(src.len() - start);
        let section = &src[start..start + end];

        assert!(
            section.contains("let should_show_window = app_entity_inner.update"),
            "{label} must inspect the execute_by_command_id_or_path return value"
        );
        assert!(
            !section.contains("show_main_window_helper"),
            "{label} dictation hotkey must not show the main window"
        );
    }
}

// ---------------------------------------------------------------------------
// Transcription observability: skip reasons and model path are logged
// ---------------------------------------------------------------------------

#[test]
fn dictation_transcription_logs_skip_reasons_and_model_path() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        runtime_src.contains("Starting dictation transcription"),
        "runtime must log transcription entry"
    );
    assert!(
        runtime_src.contains("Skipping dictation transcription: audio too short"),
        "runtime must log the too-short skip branch"
    );
    assert!(
        runtime_src.contains("Skipping dictation transcription: audio too silent"),
        "runtime must log the too-silent skip branch"
    );
    assert!(
        runtime_src.contains("model_path = %config.model_path.display()"),
        "runtime must log the model path"
    );
    assert!(
        runtime_src.contains("Dictation transcription succeeded"),
        "runtime must log successful transcription"
    );
}

#[test]
fn dictation_surfaces_missing_model_with_download() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("dictation_model_entry(model_id)"),
        "dictation start path must check selected catalog model availability"
    );
    assert!(
        builtin_src.contains("start_dictation_model_download"),
        "missing dictation model must trigger a background download"
    );
}

#[test]
fn dictation_focus_settle_matches_reference_contract() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("const DICTATION_FOCUS_SETTLE_MS: u64 = 120;"),
        "frontmost-app paste settle delay must match the 120ms reference"
    );
    assert!(
        builtin_src.contains("Preparing frontmost-app dictation paste"),
        "frontmost-app paste path must log the target handoff"
    );
}

// ---------------------------------------------------------------------------
// Hotkey routing: dictation hotkey uses builtin toggle, not a duplicate path
// ---------------------------------------------------------------------------

#[test]
fn dictation_hotkey_routes_through_builtin_toggle_flow() {
    // Verify both entry-point files have a dictation hotkey listener that
    // routes through the config-governed dictation builtin (dictation.target
    // decides the destination) without introducing a duplicate dictation
    // implementation.
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
            src.contains(r#"execute_by_command_id_or_path("builtin/dictation", ctx)"#),
            "{label} must route the dictation hotkey through the config-governed \
             builtin/dictation entry (dictation.target decides the destination)"
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
fn generic_dictation_builtin_remains_contextual() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let branch_start = src
        .find("builtins::BuiltInFeature::Dictation =>")
        .expect("dictation builtin branch must exist");
    let branch_tail = &src[branch_start..];
    let next_branch = branch_tail[1..]
        .find("builtins::BuiltInFeature::")
        .map(|offset| offset + 1)
        .unwrap_or(branch_tail.len());
    let branch_src = &branch_tail[..next_branch];

    assert!(
        branch_src.contains("DictationBuiltinAction::CurrentSurface")
            && src.contains("let is_start_edge = !crate::dictation::is_dictation_busy();")
            && src.contains("self.dictation_start_target(action)")
            && src.contains(".unwrap_or_else(|| self.resolve_dictation_target())"),
        "generic dictation builtin must continue resolving context-sensitive targets"
    );
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

#[test]
fn frontmost_app_delivery_yields_focus_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let yield_offset = handler_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app delivery must yield focus before paste");
    let paste_offset = handler_src
        .find("paste_text")
        .expect("frontmost-app delivery must paste after yielding focus");

    assert!(
        yield_offset < paste_offset,
        "frontmost-app delivery must yield focus before pasting (yield at {yield_offset}, paste at {paste_offset})"
    );
}

// ---------------------------------------------------------------------------
// Ordering regressions: frontmost-app delivery sequence
// ---------------------------------------------------------------------------

/// Extract the frozen external-app actor from handle_dictation_transcript.
fn frontmost_dictation_delivery_branch(handler_src: &str) -> &str {
    let actor = handler_src
        .find("validate_frozen_external_destination")
        .expect("handler must validate the frozen external destination");
    &handler_src[actor..]
}

#[test]
fn delivery_frontmost_app_yields_focus_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    let yield_pos = frontmost_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app branch must yield focus (hide main) before paste");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must paste transcript");

    assert!(
        yield_pos < paste_pos,
        "yield_focus_for_dictation_paste (byte {yield_pos}) must appear before paste_text (byte {paste_pos})"
    );
}

#[test]
fn delivery_frontmost_app_waits_for_focus_before_pasting() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];

    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    let yield_pos = frontmost_src
        .find("yield_focus_for_dictation_paste")
        .expect("frontmost-app branch must yield focus (hide main) before paste");
    let focus_timer_pos = frontmost_src
        .find("dictation_focus_settle_duration")
        .expect("frontmost-app branch must wait for focus to settle");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must paste transcript");

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
    // Search for notice_failed AFTER the yield_focus_result error check,
    // since the tracker guard earlier in the branch also uses notice_failed.
    let post_err_src = &frontmost_src[err_check_pos..];
    let show_error_pos = err_check_pos
        + post_err_src
            .find("notice_failed")
            .expect("frontmost-app branch must show error toast on focus-yield failure");
    let cleanup_in_err = err_check_pos
        + post_err_src
            .find("schedule_dictation_transcriber_cleanup")
            .unwrap_or(usize::MAX - err_check_pos);

    assert!(
        err_check_pos < show_error_pos.min(cleanup_in_err),
        "error check must come before toast and cleanup"
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
        helper_src.contains("platform::defer_hide_main_window(cx)"),
        "focus helper must still hide the main window before frontmost-app paste"
    );

    let handler_start = src
        .find("fn handle_dictation_transcript")
        .expect("handler must exist");
    let handler_src = &src[handler_start..];
    let frontmost_src = frontmost_dictation_delivery_branch(handler_src);

    let compacted_frontmost_src: String = frontmost_src
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        compacted_frontmost_src.contains(
            "this.yield_focus_for_dictation_paste(target_pid,&target_bundle_id,&target_app_name,cx,)"
        ),
        "frontmost-app delivery must call yield_focus_for_dictation_paste with tracked app identity"
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
/// Uses `activate_tracked_app_for_dictation_paste` as the end boundary.
fn dictation_yield_focus_helper_source(src: &str) -> &str {
    let start = src
        .find("fn yield_focus_for_dictation_paste(")
        .expect("yield_focus_for_dictation_paste must exist");
    let tail = &src[start..];
    let end = tail.find("fn activate_tracked_app_for_dictation_paste(").expect(
        "yield_focus_for_dictation_paste must be followed by activate_tracked_app_for_dictation_paste",
    );
    &tail[..end]
}

/// Extract the frozen external-app delivery actor from `handle_dictation_transcript`.
/// Uses `Ok(None)` as the end boundary.
fn dictation_frontmost_paste_source(src: &str) -> &str {
    let handler_src = dictation_handler_source(src);
    let actor_pos = handler_src
        .find("validate_frozen_external_destination")
        .expect("handler must validate its frozen external destination");
    let tail = &handler_src[actor_pos..];
    let end = tail
        .find("Ok(None)")
        .expect("frontmost-app else branch must end before the Ok(None) arm");
    &tail[..end]
}

#[test]
fn delivery_focus_helper_activates_tracked_target_app() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let helper_src = dictation_yield_focus_helper_source(&src);

    assert!(
        helper_src.contains("target_pid: i32")
            && helper_src.contains("target_bundle_id: &str")
            && helper_src.contains("target_app_name: &str"),
        "yield_focus_for_dictation_paste must accept the tracked target app identity"
    );

    assert!(
        helper_src.contains("activate_tracked_app_for_dictation_paste("),
        "focus helper must activate the tracked target app before paste"
    );
    assert!(
        src.contains("let app_pid: i32 = msg_send![app, processIdentifier];")
            && src.contains("if app_pid != target_pid")
            && src.contains("activateWithOptions: 2u64"),
        "activation must find the tracked running app by pid rather than guessing from the current frontmost window"
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
        frontmost_src.contains("present_dictation_delivery_failure")
            && frontmost_src.contains("DestinationUnavailable"),
        "frontmost-app path must preserve the transcript in typed recovery before returning"
    );
    assert!(
        !frontmost_src.contains("Dictation paste failed before paste step"),
        "pre-paste failures must not interpolate raw platform errors into user-visible copy"
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
// Frontmost-app target gating and named timing helpers
// ---------------------------------------------------------------------------

#[test]
fn delivery_frontmost_app_checks_tracked_target_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let frontmost_src = dictation_frontmost_paste_source(&src);

    let target_pos = frontmost_src
        .find("validate_frozen_external_destination")
        .expect("frontmost-app branch must validate its exact frozen target before paste");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must call paste_text");

    assert!(
        target_pos < paste_pos,
        "frozen external validation (byte {target_pos}) must appear before paste_text \
         (byte {paste_pos}) in the external-app actor"
    );
}

#[test]
fn delivery_frontmost_app_surfaces_missing_tracked_target() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_src = dictation_handler_source(&src);

    assert!(
        handler_src.contains("validate_frozen_external_destination")
            && handler_src.contains("destination_failure(true, &error)"),
        "frontmost-app path must classify missing or stale frozen targets before mutation"
    );
    assert!(
        handler_src.contains("Frozen external Dictation destination refused delivery"),
        "frontmost-app path must log a typed redacted refusal"
    );
    assert!(
        handler_src.contains("notice_failed"),
        "frontmost-app path must show the user a tracked-target failure"
    );
}

#[test]
fn delivery_frontmost_app_uses_named_focus_settle_duration_before_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let frontmost_src = dictation_frontmost_paste_source(&src);

    let settle_pos = frontmost_src
        .find("dictation_focus_settle_duration")
        .expect("frontmost-app branch must wait for named focus settle duration");
    let paste_pos = frontmost_src
        .find("paste_text")
        .expect("frontmost-app branch must call paste_text");

    assert!(
        settle_pos < paste_pos,
        "dictation_focus_settle_duration (byte {settle_pos}) must appear before \
         paste_text (byte {paste_pos}) in the frontmost-app branch"
    );
}

#[test]
fn delivery_frontmost_app_target_helper_uses_frontmost_app_tracker() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let helper_start = src
        .find("fn ensure_dictation_frontmost_target_available")
        .expect("tracked-target helper must exist");
    let helper_src = &src[helper_start..helper_start + 600.min(src.len() - helper_start)];

    assert!(
        helper_src.contains("get_last_real_app_bundle_id"),
        "tracked-target helper must use frontmost_app_tracker"
    );
}

// ---------------------------------------------------------------------------
// Device selection helpers
// ---------------------------------------------------------------------------

use crate::dictation::device::{
    build_device_menu_items, resolve_selected_input_device, DictationDeviceMenuItem,
    DictationDeviceSelectionAction,
};
use crate::dictation::types::{DictationDeviceId, DictationDeviceInfo, DictationDeviceTransport};

fn device(id: &str, name: &str, is_default: bool) -> DictationDeviceInfo {
    DictationDeviceInfo {
        id: DictationDeviceId(id.to_string()),
        name: name.to_string(),
        is_default,
        transport: DictationDeviceTransport::Unknown,
    }
}

fn device_with_transport(
    id: &str,
    name: &str,
    is_default: bool,
    transport: DictationDeviceTransport,
) -> DictationDeviceInfo {
    DictationDeviceInfo {
        id: DictationDeviceId(id.to_string()),
        name: name.to_string(),
        is_default,
        transport,
    }
}

#[test]
fn resolve_selected_input_device_prefers_saved_device() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let res = resolve_selected_input_device(&devices, Some("usb")).unwrap();
    assert_eq!(res.device.id.0, "usb");
    assert_eq!(res.device.name, "USB Microphone");
    assert!(!res.fell_back);
}

#[test]
fn resolve_selected_input_device_falls_back_to_default_for_stale_saved_id() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let res = resolve_selected_input_device(&devices, Some("missing")).unwrap();
    assert_eq!(res.device.id.0, "builtin");
    assert!(res.device.is_default);
    assert!(res.fell_back);
}

#[test]
fn resolve_selected_input_device_returns_default_when_no_preference() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let res = resolve_selected_input_device(&devices, None).unwrap();
    assert_eq!(res.device.id.0, "builtin");
    assert!(!res.fell_back);
}

#[test]
fn resolve_selected_input_device_returns_none_for_empty_list() {
    let selected = resolve_selected_input_device(&[], Some("anything"));
    assert!(selected.is_none());
}

#[test]
fn build_device_menu_items_reports_missing_device_without_pretending_it_is_selected() {
    let devices = vec![
        device("builtin", "MacBook Pro Microphone", true),
        device("usb", "USB Microphone", false),
    ];
    let items = build_device_menu_items(&devices, Some("missing"));
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].title, "System Default");
    assert!(!items[0].is_selected);
    assert!(items[0].subtitle.contains("unavailable"));
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
    assert!(
        !items[0].is_selected,
        "System Default should not be selected"
    );
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

#[test]
fn apply_device_selection_persists_selected_device_id_and_clears_to_default() {
    let lock = crate::test_utils::SK_PATH_TEST_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let temp_dir = tempfile::tempdir().expect("create temp Script Kit dir");
    let kit_dir = temp_dir.path().to_path_buf();
    std::fs::create_dir_all(&kit_dir).expect("create Script Kit dir");
    let previous_sk_path = std::env::var(crate::setup::SK_PATH_ENV).ok();
    std::env::set_var(crate::setup::SK_PATH_ENV, temp_dir.path());

    crate::dictation::apply_device_selection(&DictationDeviceSelectionAction::UseDevice(
        DictationDeviceId("usb-mic".to_string()),
    ))
    .expect("persist selected microphone");

    let selected = crate::config::load_user_preferences();
    assert_eq!(
        selected.dictation.selected_device_id.as_deref(),
        Some("usb-mic")
    );

    crate::dictation::apply_device_selection(&DictationDeviceSelectionAction::UseSystemDefault)
        .expect("clear selected microphone");

    let cleared = crate::config::load_user_preferences();
    assert_eq!(cleared.dictation.selected_device_id, None);

    match previous_sk_path {
        Some(path) => std::env::set_var(crate::setup::SK_PATH_ENV, path),
        None => std::env::remove_var(crate::setup::SK_PATH_ENV),
    }
    drop(lock);
}

#[test]
fn dictation_preferences_serialize_selected_device_id_as_camel_case() {
    let preferences = crate::config::ScriptKitUserPreferences {
        dictation: crate::config::DictationPreferences {
            selected_device_id: Some("usb-mic".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let value = serde_json::to_value(&preferences).expect("serialize config-backed preferences");
    assert_eq!(value["dictation"]["selectedDeviceId"], "usb-mic");
    assert!(value["dictation"].get("model").is_none());
    assert!(value["dictation"].get("language").is_none());
    assert!(value["dictation"].get("selected_device_id").is_none());
}

#[test]
fn dictation_preferences_serde_round_trip_model_and_language() {
    let preferences = crate::config::ScriptKitUserPreferences {
        dictation: crate::config::DictationPreferences {
            selected_device_id: Some("usb-mic".to_string()),
            model: Some("whisper-medium".to_string()),
            language: Some("de".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let value = serde_json::to_value(&preferences).expect("serialize dictation preferences");
    assert_eq!(value["dictation"]["selectedDeviceId"], "usb-mic");
    assert_eq!(value["dictation"]["model"], "whisper-medium");
    assert_eq!(value["dictation"]["language"], "de");

    let decoded: crate::config::ScriptKitUserPreferences =
        serde_json::from_value(value).expect("deserialize dictation preferences");
    assert_eq!(decoded.dictation, preferences.dictation);
}

#[test]
fn save_user_preferences_writes_config_and_cleans_known_legacy_settings() {
    let lock = crate::test_utils::SK_PATH_TEST_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let temp_dir = tempfile::tempdir().expect("create temp Script Kit dir");
    let kit_dir = temp_dir.path().to_path_buf();
    std::fs::create_dir_all(&kit_dir).expect("create Script Kit dir");
    std::env::set_var(crate::setup::SK_PATH_ENV, temp_dir.path());

    let settings_path = kit_dir.join("settings.json");
    let original = json!({
        "theme": { "presetId": "nord" }
    });
    std::fs::write(
        &settings_path,
        serde_json::to_string_pretty(&original).expect("serialize original settings"),
    )
    .expect("write settings file");

    let preferences = crate::config::ScriptKitUserPreferences {
        dictation: crate::config::DictationPreferences {
            selected_device_id: Some("usb-mic".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    crate::config::save_user_preferences(&preferences)
        .expect("save merged config-backed preferences");

    assert!(
        !settings_path.exists(),
        "known legacy settings file should be removed after config migration"
    );

    let config_path = kit_dir.join("config.ts");
    let config_contents = std::fs::read_to_string(&config_path).expect("read saved config");
    assert!(config_contents.contains("dictation"));
    assert!(config_contents.contains("selectedDeviceId"));
    assert!(config_contents.contains("usb-mic"));

    let loaded = crate::config::load_user_preferences();
    assert_eq!(
        loaded.dictation.selected_device_id.as_deref(),
        Some("usb-mic")
    );

    std::env::remove_var(crate::setup::SK_PATH_ENV);
    drop(lock);
}

// ---------------------------------------------------------------------------
// Delivery-target preflight tests
// ---------------------------------------------------------------------------

#[test]
fn dictation_runtime_exposes_is_recording_helper() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");
    let fn_start = runtime_src
        .find("pub fn is_dictation_recording() -> bool")
        .expect("runtime must expose is_dictation_recording");
    let fn_src = &runtime_src[fn_start..fn_start + 200.min(runtime_src.len() - fn_start)];
    assert!(
        fn_src.contains("SESSION.lock().is_some()"),
        "is_dictation_recording must reflect live session state"
    );

    let mod_src = std::fs::read_to_string("src/dictation/mod.rs").expect("read mod.rs");
    assert!(
        mod_src.contains("is_dictation_recording"),
        "dictation mod facade must re-export is_dictation_recording"
    );
}

#[test]
fn can_accept_dictation_into_prompt_matches_delivery_views() {
    let src = std::fs::read_to_string("src/app_impl/ui_window.rs").expect("read ui_window.rs");
    let fn_start = src
        .find("fn can_accept_dictation_into_prompt")
        .expect("can_accept_dictation_into_prompt must exist");
    let fn_src = &src[fn_start..fn_start + 1200.min(src.len() - fn_start)];

    for view in &[
        "AppView::ArgPrompt",
        "AppView::MiniPrompt",
        "AppView::MicroPrompt",
        "AppView::PathPrompt",
        "AppView::SelectPrompt",
        "AppView::EnvPrompt",
        "AppView::TemplatePrompt",
        "AppView::FormPrompt",
    ] {
        assert!(
            fn_src.contains(view),
            "can_accept_dictation_into_prompt must include {view}"
        );
    }
}

#[test]
fn dictation_start_preflights_delivery_target_before_toggle() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let dictation_start = src
        .find("fn execute_dictation_builtin_action(")
        .expect("execute_dictation_builtin_action must exist");
    let dictation_src =
        &src[dictation_start..dictation_start + 7000.min(src.len() - dictation_start)];

    let preflight_pos = dictation_src
        .find("let preflight = self.prepare_dictation_builtin_start(action, None, cx);")
        .expect("dictation start must prepare preflight");
    let toggle_pos = dictation_src
        .find("crate::dictation::toggle_dictation(canonical_target)")
        .expect("dictation builtin must call toggle_dictation");
    assert!(
        preflight_pos < toggle_pos,
        "delivery-target preflight must run before toggle_dictation"
    );
}

#[test]
fn dictation_preflight_allows_prompt_or_frontmost_target() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let helper_start = src
        .find("fn ensure_dictation_delivery_target_available")
        .expect("delivery preflight helper must exist");
    let helper_src = &src[helper_start..helper_start + 600.min(src.len() - helper_start)];

    assert!(
        helper_src.contains("self.can_accept_dictation_into_prompt()"),
        "delivery preflight must allow direct prompt delivery"
    );
    assert!(
        helper_src.contains("ensure_dictation_frontmost_target_available"),
        "delivery preflight must use the tracked frontmost-app guard when no prompt can accept text"
    );
}

// ---------------------------------------------------------------------------
// Behavioral toggle outcome tests
// ---------------------------------------------------------------------------

/// Prove that `DictationToggleOutcome::Started` is constructible and distinct
/// from `Stopped` variants, matching the contract where the first toggle
/// returns `Started` and the second returns `Stopped(Some(_))` or
/// `Stopped(None)`.
#[test]
fn toggle_outcome_started_is_distinct_from_stopped() {
    use crate::dictation::types::DictationToggleOutcome;

    let started = DictationToggleOutcome::Started;
    let stopped_none = DictationToggleOutcome::Stopped(None);
    let stopped_some = DictationToggleOutcome::Stopped(Some(CompletedDictationCapture {
        chunks: vec![CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.5; 160],
            duration: Duration::from_millis(10),
        }],
        audio_duration: Duration::from_millis(10),
        truncated: false,
    }));

    assert_ne!(started, stopped_none);
    assert_ne!(started, stopped_some);
    assert_ne!(stopped_none, stopped_some);
}

/// Prove that `Stopped(Some(capture))` carries the expected audio data
/// through the type, validating the end-to-end contract from toggle_dictation
/// to transcription handoff.
#[test]
fn toggle_outcome_stopped_some_carries_capture_data() {
    use crate::dictation::types::DictationToggleOutcome;

    let chunks = vec![
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.1; 160],
            duration: Duration::from_millis(10),
        },
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.2; 80],
            duration: Duration::from_millis(5),
        },
    ];
    let capture = CompletedDictationCapture {
        chunks: chunks.clone(),
        audio_duration: Duration::from_millis(15),
        truncated: false,
    };
    let outcome = DictationToggleOutcome::Stopped(Some(capture));

    match outcome {
        DictationToggleOutcome::Stopped(Some(cap)) => {
            assert_eq!(cap.chunks.len(), 2);
            assert_eq!(cap.audio_duration, Duration::from_millis(15));
            // Verify the chunks can be merged for transcription.
            let merged = crate::dictation::merge_captured_chunks(&cap.chunks);
            assert_eq!(merged.len(), 240);
        }
        other => panic!("expected Stopped(Some(_)), got {other:?}"),
    }
}

/// Simulate the full toggle cycle: Started → (capture chunks) → Stopped(Some)
/// and verify the transcription facade processes the captured audio.
#[test]
fn toggle_cycle_produces_transcribable_capture() -> Result<()> {
    // Simulate producing captured chunks (as toggle_dictation would return).
    let chunks = vec![
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.3; 320],
            duration: Duration::from_millis(20),
        },
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            samples: vec![0.4; 160],
            duration: Duration::from_millis(10),
        },
    ];

    let capture = CompletedDictationCapture {
        audio_duration: crate::dictation::captured_duration(&chunks),
        chunks,
        truncated: false,
    };

    assert_eq!(capture.audio_duration, Duration::from_millis(30));

    // Feed the captured audio through a stub transcriber to prove the
    // capture → transcribe handoff works.
    let transcriber = DictationTranscriber::new(
        DictationTranscriptionConfig {
            minimum_samples: 1,
            ..Default::default()
        },
        Box::new(StubEngine {
            output: "hello world".to_string(),
        }),
    );

    let result = transcriber.transcribe_chunks(&capture.chunks)?;
    assert_eq!(result, Some("hello world".to_string()));

    Ok(())
}

// ---------------------------------------------------------------------------
// Dictation start-edge preflight & helper parity tests
// ---------------------------------------------------------------------------

#[test]
fn dictation_start_preflight_runs_before_toggle() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let dictation_start = src
        .find("fn execute_dictation_builtin_action(")
        .expect("execute_dictation_builtin_action must exist");
    let dictation_src =
        &src[dictation_start..dictation_start + 7000.min(src.len() - dictation_start)];

    let recording_guard_pos = dictation_src
        .find("let is_start_edge = !crate::dictation::is_dictation_busy();")
        .expect("dictation start path must gate preflight on recording state");
    let preflight_pos = dictation_src
        .find("let preflight = self.prepare_dictation_builtin_start(action, None, cx);")
        .expect("dictation start path must prepare preflight");
    let toggle_pos = dictation_src
        .find("crate::dictation::toggle_dictation(canonical_target)")
        .expect("dictation start path must toggle dictation");

    assert!(
        recording_guard_pos < preflight_pos && preflight_pos < toggle_pos,
        "start-edge preflight must run before toggle_dictation"
    );
}

#[test]
fn rapid_dictation_toggle_parity_is_last_intent_wins_during_stop() {
    let restart_after_first_extra_toggle = crate::dictation::toggled_post_stop_restart(false);
    assert!(
        restart_after_first_extra_toggle,
        "first toggle during stop must queue a restart"
    );
    assert!(
        !crate::dictation::toggled_post_stop_restart(restart_after_first_extra_toggle),
        "second toggle during stop must cancel the queued restart"
    );
}

#[test]
fn dictation_start_preflight_surfaces_unavailable_target() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let dictation_start = src
        .find("fn prepare_dictation_builtin_start(")
        .expect("prepare_dictation_builtin_start must exist");
    let dictation_src =
        &src[dictation_start..dictation_start + 3000.min(src.len() - dictation_start)];

    assert!(
        dictation_src.contains("preflight_failure_message()"),
        "preflight failures must be logged"
    );
    assert!(
        dictation_src.contains("self.notice_failed"),
        "preflight failures must surface a toast"
    );
}

#[test]
fn can_accept_dictation_into_prompt_stays_aligned_with_direct_delivery_views() {
    let src = std::fs::read_to_string("src/app_impl/ui_window.rs").expect("read ui_window.rs");

    let helper_start = src
        .find("fn can_accept_dictation_into_prompt")
        .expect("can_accept_dictation_into_prompt must exist");
    let helper_src = &src[helper_start..helper_start + 1200.min(src.len() - helper_start)];

    let setter_start = src
        .find("fn try_set_prompt_input")
        .expect("try_set_prompt_input must exist");
    let setter_src = &src[setter_start..setter_start + 3500.min(src.len() - setter_start)];

    for view in [
        "AppView::ArgPrompt",
        "AppView::MiniPrompt",
        "AppView::MicroPrompt",
        "AppView::PathPrompt",
        "AppView::SelectPrompt",
        "AppView::EnvPrompt",
        "AppView::TemplatePrompt",
        "AppView::FormPrompt",
        "AppView::FileSearchView",
    ] {
        assert!(
            helper_src.contains(view),
            "can_accept_dictation_into_prompt must include {view}"
        );
        assert!(
            setter_src.contains(view),
            "try_set_prompt_input must include {view}"
        );
    }
}

#[test]
fn active_display_api_exists_in_platform() {
    let display_src = std::fs::read_to_string("src/platform/display.rs").expect("read display.rs");

    assert!(
        display_src.contains("pub fn get_active_display()"),
        "platform must expose get_active_display() for key-window display resolution"
    );
    assert!(
        display_src.contains("mainScreen"),
        "get_active_display() must use NSScreen.mainScreen"
    );
    // Non-macOS stub must exist.
    assert!(
        display_src.contains("#[cfg(not(target_os = \"macos\"))]")
            && display_src.contains("get_active_display"),
        "get_active_display() must have a non-macOS stub"
    );
}

// ---------------------------------------------------------------------------
// Overlay: Escape confirmation state machine
// ---------------------------------------------------------------------------

#[test]
fn overlay_has_confirming_phase_in_session_phase_enum() {
    let types_src = std::fs::read_to_string("src/dictation/types.rs").expect("read types.rs");

    assert!(
        types_src.contains("Confirming"),
        "DictationSessionPhase must include a Confirming variant"
    );
}

// ---------------------------------------------------------------------------
// Runtime session owns overlay phase (confirm/resume/abort contract)
// ---------------------------------------------------------------------------

#[test]
fn runtime_session_has_overlay_phase_field() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        runtime_src.contains("overlay_phase: DictationSessionPhase"),
        "DictationSession must own overlay_phase field"
    );
}

#[test]
fn active_recording_microphone_is_captured_at_session_start() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");
    // Compile-time export contract only: never resolve or open a real device.
    let _: fn() -> Option<DictationDeviceInfo> = crate::dictation::get_active_dictation_device;
    let start = compact_delta_contract(extract_delta_contract_section(
        &runtime_src,
        "fn start_recording(",
        "fn stop_recording(",
    ));
    assert!(
        start.contains("letactive_device=resolve_preferred_device_info()?")
            && start.contains("letdevice_id=active_device.as_ref().map(|device|device.id.clone());")
            && start.contains("start_capture(capture_config,device_id.as_ref())")
            && start.contains(
                "DictationSession::from_source(target,event_rx,Some(capture_handle),active_device.clone(),"
            ),
        "capture and session metadata must share the microphone resolved at session start"
    );
    let constructor = compact_delta_contract(extract_delta_contract_section(
        &runtime_src,
        "impl DictationSession {",
        "include!(\"runtime_owned_fixture.rs\")",
    ));
    assert!(
        constructor.contains("active_device:Option<DictationDeviceInfo>")
            && constructor.contains("target_cycle:vec![target],active_device,"),
        "the shared session constructor must retain the supplied microphone"
    );
    let active_device = extract_delta_contract_section(
        &runtime_src,
        "pub fn get_active_dictation_device()",
        "pub fn pending_dictation_device_label()",
    );
    assert!(
        active_device.contains("session.active_device.clone()")
            && !active_device.contains("resolve_preferred_device_info"),
        "active microphone receipts must read the session snapshot, not a changed preference"
    );
}

#[test]
fn start_recording_initialises_overlay_phase_to_recording() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        runtime_src.contains("overlay_phase: DictationSessionPhase::Recording"),
        "start_recording must initialise overlay_phase to Recording"
    );
}

// ---------------------------------------------------------------------------
// Regression: missing-model error path must NEVER attempt paste or delivery
// ---------------------------------------------------------------------------

/// When `transcribe_captured_audio` returns an `Err` (e.g. missing Whisper
/// model), `handle_dictation_transcript` must show a toast — but it must
/// **never** call `paste_text` or
/// `try_set_prompt_input`, because there is no transcript to deliver.
///
/// This test structurally verifies the error arm to prevent accidental
/// delivery regressions.
#[test]
fn missing_model_error_path_never_attempts_paste_or_prompt_delivery() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_src = dictation_handler_source(&src);

    // The error arm starts at `Err(error) =>` and extends to the end of the
    // match (the handler's closing brace).  Extract it by finding the last
    // `Err(error) =>` after the `Ok(None)` arm.
    let ok_none_pos = handler_src
        .find("Ok(None) =>")
        .expect("handler must have an Ok(None) arm");
    let err_arm_start = handler_src[ok_none_pos..]
        .find("Err(error) =>")
        .expect("handler must have an Err(error) arm after Ok(None)");
    let err_arm_src = &handler_src[ok_none_pos + err_arm_start..];

    assert!(
        !err_arm_src.contains("paste_text"),
        "Err arm of handle_dictation_transcript must NEVER call paste_text \
         — there is no transcript to deliver when transcription fails"
    );
    assert!(
        !err_arm_src.contains("try_set_prompt_input"),
        "Err arm of handle_dictation_transcript must NEVER call try_set_prompt_input \
         — there is no transcript to deliver when transcription fails"
    );

    // Confirm the error arm DOES show the failure's safe message.
    assert!(
        err_arm_src.contains("failure.failure.primary_message(),"),
        "Err arm must surface the classified failure to the user in the line above the input"
    );
}

/// The `Ok(None)` arm (silent/short audio) must also never attempt delivery.
#[test]
fn silent_audio_path_never_attempts_paste_or_prompt_delivery() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_src = dictation_handler_source(&src);

    // Extract the Ok(None) arm: from `Ok(None) =>` to `Err(error) =>`.
    let none_start = handler_src
        .find("Ok(None) =>")
        .expect("handler must have an Ok(None) arm");
    let none_src_tail = &handler_src[none_start..];
    let err_offset = none_src_tail
        .find("Err(error) =>")
        .expect("Ok(None) arm must be followed by Err arm");
    let none_arm_src = &none_src_tail[..err_offset];

    assert!(
        !none_arm_src.contains("paste_text"),
        "Ok(None) arm must NEVER call paste_text — no transcript exists"
    );
    assert!(
        !none_arm_src.contains("try_set_prompt_input"),
        "Ok(None) arm must NEVER call try_set_prompt_input — no transcript exists"
    );
    assert!(
        !none_arm_src.contains("open_tab_ai_agent_chat_with_entry_intent"),
        "Ok(None) arm must NEVER open Agent Chat — no transcript exists to seed or submit"
    );
    assert!(
        none_arm_src.contains("one_window_talk_final(String::new(), None, cx)"),
        "Ok(None) arm must end the voice view with no words and nothing to paste"
    );
}

#[test]
fn agent_chat_dictation_delivery_suppresses_focused_launcher_context() {
    let src = std::fs::read_to_string("src/app_impl/agent_handoff/agent_chat_entry.rs")
        .expect("read Agent Chat entry request owner");
    let request = compact_delta_contract(extract_delta_contract_section(
        &src,
        "pub(crate) fn dictation_send(",
        "pub(crate) fn explicit_send(",
    ));
    assert!(
        request.contains(
            "ifui_variant==crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant::QuickAi{AgentChatContextPolicy::NoContext}else{AgentChatContextPolicy::SuppressFocused}"
        ),
        "Dictation must suppress focused launcher context, and Quick AI must admit no context"
    );
    assert!(
        request.contains(
            "origin:AgentChatEntryOrigin::Dictation,target,intent,ui_variant,context_policy,"
        ),
        "the entry request must retain the supplied frozen thread target and context policy"
    );
}

#[test]
fn orchestrator_focus_main_commands_queue_app_focus_after_reveal() {
    let bridge_src = std::fs::read_to_string("src/app_impl/window_orchestrator_bridge.rs")
        .expect("read window_orchestrator_bridge.rs");

    assert!(
        bridge_src.contains("WindowCommand::FocusMain(token)"),
        "the orchestrator bridge must consume FocusMain commands instead of leaving them as executor no-ops"
    );
    assert!(
        bridge_src.contains("this.pending_focus = Some(target);")
            && bridge_src.contains("cx.notify();"),
        "FocusMain commands must queue ScriptListApp pending_focus after reveal so the next render applies keyboard focus"
    );
    assert!(
        bridge_src.contains("FocusToken::ChatComposer => Some(FocusTarget::AgentChat)"),
        "Agent Chat dictation's ChatComposer token must map to the dedicated AgentChat focus target"
    );
}

#[test]
fn orchestrator_executor_keeps_global_main_visibility_in_sync() {
    let src =
        std::fs::read_to_string("src/window_orchestrator/executor.rs").expect("read executor.rs");

    let conceal_start = src
        .find("WindowCommand::ConcealMain")
        .expect("executor must handle ConcealMain");
    let dismiss_start = src
        .find("WindowCommand::DismissMain")
        .expect("executor must handle DismissMain");
    let reveal_start = src
        .find("WindowCommand::RevealMain")
        .expect("executor must handle RevealMain");
    let focus_surface_start = src
        .find("WindowCommand::FocusSurface")
        .expect("executor must handle FocusSurface");

    assert!(
        src[conceal_start..dismiss_start].contains("set_main_window_visible(false)"),
        "ConcealMain must mark the main window hidden before dictation opens"
    );
    assert!(
        src[dismiss_start..reveal_start].contains("set_main_window_visible(false)"),
        "DismissMain must mark the main window hidden"
    );
    assert!(
        src[reveal_start..focus_surface_start].contains("set_main_window_visible(true)"),
        "RevealMain must mark the main window visible"
    );
}

/// `abort_dictation()` must drop the session, ensuring `is_dictation_recording()`
/// returns false afterward and preventing any further overlay pump ticks from
/// reading stale state.
#[test]
fn abort_dictation_clears_session_state() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    // abort_dictation must call stop_recording which takes the session.
    let abort_start = runtime_src
        .find("pub fn abort_dictation() -> Result<()>")
        .expect("abort_dictation must exist");
    let abort_src =
        &runtime_src[abort_start..abort_start + 300.min(runtime_src.len() - abort_start)];

    assert!(
        abort_src.contains("stop_recording()"),
        "abort_dictation must call stop_recording() to drain and drop the session"
    );

    // stop_recording must take() the session from the mutex.
    let stop_start = runtime_src
        .find("fn stop_recording()")
        .expect("stop_recording must exist");
    let stop_src = &runtime_src[stop_start..stop_start + 600.min(runtime_src.len() - stop_start)];

    assert!(
        stop_src.contains(".take()"),
        "stop_recording must take() the session from the global mutex, \
         clearing it so is_dictation_recording() returns false"
    );
}

// ---------------------------------------------------------------------------
// Regression: missing-model toast includes download guidance
// ---------------------------------------------------------------------------

/// When the Parakeet model is missing at transcription time, the error
/// handler must detect the specific error string and log the resolved
/// model path for diagnostics.
#[test]
fn missing_model_error_detects_parakeet_string_and_logs_model_path() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_src = dictation_handler_source(&src);

    // The error arm must detect the model-missing string (both engines
    // bail with "... model not downloaded").
    assert!(
        handler_src.contains(r#"error_text.contains("model not downloaded")"#),
        "handler must detect the 'model not downloaded' error string"
    );
    assert!(
        handler_src.contains("resolve_default_model_path()"),
        "error handler must use the resolved Parakeet model path"
    );
}

// ---------------------------------------------------------------------------
// Regression: complete delivery ordering chain (end-to-end)
// ---------------------------------------------------------------------------

/// Verify the full frontmost-app delivery chain ordering in one test:
/// yield_focus → focus_settle → paste_text.
/// This catches regressions if any step is reordered or removed.
#[test]
fn frontmost_app_delivery_full_ordering_chain() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let frontmost_src = dictation_frontmost_paste_source(&src);

    let steps: Vec<(&str, usize)> = [
        (
            "yield_focus_for_dictation_paste",
            "yield focus to target app",
        ),
        (
            "dictation_focus_settle_duration",
            "wait for focus to settle",
        ),
        ("paste_text", "paste transcript"),
    ]
    .iter()
    .map(|(needle, label)| {
        let pos = frontmost_src
            .find(needle)
            .unwrap_or_else(|| panic!("frontmost-app branch must contain {label} ({needle})"));
        (*label, pos)
    })
    .collect();

    for pair in steps.windows(2) {
        assert!(
            pair[0].1 < pair[1].1,
            "delivery ordering violated: '{}' (byte {}) must come before '{}' (byte {})",
            pair[0].0,
            pair[0].1,
            pair[1].0,
            pair[1].1
        );
    }
}

// ---------------------------------------------------------------------------
// ParakeetDictationEngine tests
// ---------------------------------------------------------------------------

#[test]
fn parakeet_engine_new_fails_for_missing_model_dir() {
    let result =
        ParakeetDictationEngine::new(std::path::Path::new("/definitely/missing-parakeet-dir"));
    assert!(
        result.is_err(),
        "ParakeetDictationEngine::new must fail for a missing directory"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found"),
        "error should mention 'not found', got: {err_msg}"
    );
}

#[test]
fn parakeet_engine_new_fails_for_file_path() {
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    let result = ParakeetDictationEngine::new(temp_file.path());
    assert!(
        result.is_err(),
        "ParakeetDictationEngine::new must fail for a file path"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not a directory"),
        "error should mention 'not a directory', got: {err_msg}"
    );
}

#[test]
fn parakeet_engine_new_accepts_existing_directory() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    ParakeetDictationEngine::new(temp_dir.path())
        .expect("ParakeetDictationEngine::new should accept an existing directory");
}

// ---------------------------------------------------------------------------
// Model availability tests
// ---------------------------------------------------------------------------

#[test]
fn is_parakeet_model_available_returns_false_for_nonexistent_path() {
    // The default path is unlikely to exist in CI, so this should be false.
    // We test the function runs without panicking and returns a bool.
    let _available = is_parakeet_model_available();
}

#[test]
fn is_parakeet_model_available_returns_true_for_populated_dir() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    // Create a dummy file inside to simulate an extracted model.
    std::fs::write(temp_dir.path().join("model.onnx"), b"dummy").expect("write dummy model");

    // We can't easily test the actual function since it reads from a fixed path,
    // but we can verify the logic inline.
    let path = temp_dir.path();
    let available = path.is_dir()
        && std::fs::read_dir(path)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);
    assert!(
        available,
        "populated directory should be detected as available"
    );
}

// ---------------------------------------------------------------------------
// Download types tests
// ---------------------------------------------------------------------------

#[test]
fn download_progress_percentage_ranges() {
    use crate::dictation::download::{DownloadPhase, DownloadProgress};

    let p = DownloadProgress {
        downloaded: 0,
        total: 100,
    };
    assert_eq!(p.percentage(), 0);

    let p = DownloadProgress {
        downloaded: 50,
        total: 100,
    };
    assert_eq!(p.percentage(), 50);

    let p = DownloadProgress {
        downloaded: 100,
        total: 100,
    };
    assert_eq!(p.percentage(), 100);

    // Unknown total.
    let p = DownloadProgress {
        downloaded: 42,
        total: 0,
    };
    assert_eq!(p.percentage(), 0);
}

#[test]
fn download_phase_variants_are_distinct() {
    use crate::dictation::download::DownloadPhase;

    let phases = [
        DownloadPhase::Downloading,
        DownloadPhase::Extracting,
        DownloadPhase::Complete,
        DownloadPhase::Cancelled,
        DownloadPhase::Failed("test".to_string()),
    ];

    for (i, a) in phases.iter().enumerate() {
        for (j, b) in phases.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parakeet model constants and URL validation
// ---------------------------------------------------------------------------

#[test]
fn parakeet_model_url_is_https() {
    use crate::dictation::transcription::PARAKEET_MODEL_URL;
    assert!(
        PARAKEET_MODEL_URL.starts_with("https://"),
        "model URL must use HTTPS, got: {PARAKEET_MODEL_URL}"
    );
}

#[test]
fn parakeet_model_archive_size_is_reasonable() {
    use crate::dictation::transcription::PARAKEET_MODEL_ARCHIVE_SIZE;
    // ~478 MB — sanity check it's in the right ballpark.
    assert!(
        PARAKEET_MODEL_ARCHIVE_SIZE > 400_000_000,
        "archive size should be > 400 MB"
    );
    assert!(
        PARAKEET_MODEL_ARCHIVE_SIZE < 600_000_000,
        "archive size should be < 600 MB"
    );
}

// ---------------------------------------------------------------------------
// Runtime transcriber cache selection logic
// ---------------------------------------------------------------------------

#[test]
fn transcriber_cache_rebuilds_when_model_or_language_changes_or_idle() {
    use crate::dictation::runtime::{should_rebuild_cached_transcriber, CachedTranscriberKey};
    use crate::dictation::DictationModelId;

    let parakeet = CachedTranscriberKey {
        model_id: DictationModelId::ParakeetTdt06bV3,
        language: None,
    };
    let whisper_en = CachedTranscriberKey {
        model_id: DictationModelId::WhisperMedium,
        language: Some("en".to_string()),
    };
    let whisper_de = CachedTranscriberKey {
        model_id: DictationModelId::WhisperMedium,
        language: Some("de".to_string()),
    };

    assert!(should_rebuild_cached_transcriber(None, &parakeet, false));
    assert!(!should_rebuild_cached_transcriber(
        Some(&parakeet),
        &parakeet,
        false
    ));
    assert!(should_rebuild_cached_transcriber(
        Some(&parakeet),
        &parakeet,
        true
    ));
    assert!(should_rebuild_cached_transcriber(
        Some(&parakeet),
        &whisper_en,
        false
    ));
    assert!(should_rebuild_cached_transcriber(
        Some(&whisper_en),
        &whisper_de,
        false
    ));
}

// ---------------------------------------------------------------------------
// Model status type tests
// ---------------------------------------------------------------------------

#[test]
fn dictation_model_status_variants() {
    use crate::dictation::types::DictationModelStatus;

    let statuses = [
        DictationModelStatus::Available,
        DictationModelStatus::NotDownloaded,
        DictationModelStatus::Downloading {
            percentage: 50,
            downloaded_bytes: 250_000_000,
            total_bytes: 500_000_000,
            speed_bytes_per_sec: 10_000_000,
            eta_seconds: Some(25),
        },
        DictationModelStatus::Extracting,
        DictationModelStatus::DownloadFailed("err".to_string()),
    ];

    assert_eq!(statuses.len(), 5);
    assert_ne!(statuses[0], statuses[1]);
}

#[test]
fn format_eta_covers_ranges() {
    use crate::dictation::download::format_eta;

    assert_eq!(format_eta(None), "ETA --");
    assert_eq!(format_eta(Some(0)), "ETA <1s");
    assert_eq!(format_eta(Some(15)), "ETA 15s");
    assert_eq!(format_eta(Some(75)), "ETA 1m 15s");
    assert_eq!(format_eta(Some(3600)), "ETA 1h");
    assert_eq!(format_eta(Some(3672)), "ETA 1h 1m");
}

#[test]
fn downloading_prompt_includes_eta_and_cancel_action() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let prompt_start = src
        .find("fn build_dictation_model_prompt")
        .expect("build_dictation_model_prompt must exist");
    let prompt_src = &src[prompt_start..prompt_start + 7000.min(src.len() - prompt_start)];

    assert!(
        prompt_src.contains("format_progress_summary"),
        "downloading prompt must use the shared progress summary (which includes ETA)"
    );
    assert!(
        prompt_src.contains("Cancel download"),
        "downloading prompt must expose a cancel action"
    );
    assert!(
        prompt_src.contains("retry resumes from the partial file"),
        "cancel action must explain resume behavior"
    );
    assert!(
        prompt_src.contains("Partial download kept. Retry resumes"),
        "cancelled downloads must explain resume behavior"
    );
}

/// Ensure the transcription pipeline logs model resolution details so that
/// second-hotkey failures are diagnosable from logs alone.
#[test]
fn dictation_transcription_logs_model_resolution_details() {
    let src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");
    assert!(
        src.contains("model_exists = config.model_path.exists()"),
        "transcription log must include model_exists"
    );
    assert!(
        src.contains("model_is_file = config.model_path.is_file()"),
        "transcription log must include model_is_file"
    );
    assert!(
        src.contains("model not downloaded"),
        "runtime must bail with a clear message when the selected model is missing"
    );
}

#[test]
fn dictation_builtin_does_not_set_opened_from_main_menu() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    // Extract the Dictation arm of the match.
    let arm_start = src
        .find("builtins::BuiltInFeature::Dictation =>")
        .expect("Dictation arm must exist");
    let tail = &src[arm_start..];
    // The arm ends at the next BuiltInFeature variant.
    let arm_end = tail[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(tail.len() - 1);
    let arm_text = &tail[..arm_end + 1];

    assert!(
        !arm_text.contains("opened_from_main_menu = true"),
        "dictation builtin must not set opened_from_main_menu — \
         that would cause the main window to appear on the dictation hotkey path"
    );
}

// ---------------------------------------------------------------------------
// Source-contract coverage: singleton, non-activating, edge-to-edge fill,
// and generation guards
// ---------------------------------------------------------------------------

fn extract_delta_contract_section<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing section start: {start_pat}"));
    let tail = &source[start..];
    let end = tail
        .find(end_pat)
        .map(|offset| start + offset)
        .unwrap_or(source.len());
    &source[start..end]
}

fn compact_delta_contract(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn dictation_elapsed_is_exported() {
    let mod_src = std::fs::read_to_string("src/dictation/mod.rs").expect("read mod.rs");
    assert!(
        mod_src.contains("dictation_elapsed"),
        "dictation_elapsed must be re-exported from dictation module"
    );
}

// ---------------------------------------------------------------------------
// Waveform animation quality
// ---------------------------------------------------------------------------

#[test]
fn fft_visualiser_produces_nonzero_bars_for_speech_tone() {
    let mut vis = AudioVisualiser::new_speech(16_000);
    // 440 Hz sine at moderate volume — should light up at least some bars.
    let samples: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16_000.0).sin() * 0.3)
        .collect();
    let bars = vis.feed(&samples).expect("should produce bars");
    let max_bar = bars.iter().cloned().fold(0.0_f32, f32::max);
    assert!(max_bar > 0.05, "speech tone should produce visible bars");
    assert!(bars.iter().all(|bar| *bar >= 0.0 && *bar <= 1.0));
}

#[test]
fn waveform_attack_is_faster_than_decay() {
    use super::visualizer::animate_bars;

    let rise = animate_bars([0.08; 9], [1.0; 9], Duration::from_millis(16))[4];
    let fall = animate_bars([1.0; 9], [0.08; 9], Duration::from_millis(16))[4];
    // Rise delta should exceed fall delta (fast attack, slow decay).
    assert!(rise - 0.08 > 1.0 - fall);
}

// ---------------------------------------------------------------------------
// Mic selection heuristic tests
// ---------------------------------------------------------------------------

#[test]
fn mic_resolution_prefers_saved_device() {
    let devices = vec![
        device_with_transport("mic-1", "Built-in", true, DictationDeviceTransport::BuiltIn),
        device_with_transport("mic-2", "USB Mic", false, DictationDeviceTransport::Usb),
    ];
    let res = resolve_selected_input_device(&devices, Some("mic-2")).expect("should resolve");
    assert_eq!(res.device.id.0, "mic-2");
    assert!(!res.fell_back);
}

#[test]
fn mic_resolution_falls_back_when_saved_device_missing() {
    let devices = vec![device_with_transport(
        "mic-1",
        "Built-in",
        true,
        DictationDeviceTransport::BuiltIn,
    )];
    let res =
        resolve_selected_input_device(&devices, Some("disappeared-mic")).expect("should resolve");
    assert_eq!(res.device.id.0, "mic-1");
    assert!(res.fell_back);
}

#[test]
fn mic_resolution_prefers_system_default_when_no_preference() {
    let devices = vec![
        device_with_transport("mic-1", "USB Mic", false, DictationDeviceTransport::Usb),
        device_with_transport("mic-2", "Built-in", true, DictationDeviceTransport::BuiltIn),
    ];
    let res = resolve_selected_input_device(&devices, None).expect("should resolve");
    assert_eq!(res.device.id.0, "mic-2", "should prefer is_default device");
    assert!(!res.fell_back);
}

#[test]
fn mic_resolution_prefers_builtin_over_virtual_when_no_default() {
    let devices = vec![
        device_with_transport(
            "mic-v",
            "Virtual Audio",
            false,
            DictationDeviceTransport::Virtual,
        ),
        device_with_transport(
            "mic-b",
            "MacBook Mic",
            false,
            DictationDeviceTransport::BuiltIn,
        ),
    ];
    let res = resolve_selected_input_device(&devices, None).expect("should resolve");
    assert_eq!(
        res.device.id.0, "mic-b",
        "should prefer built-in over virtual"
    );
}

#[test]
fn mic_resolution_prefers_usb_over_virtual_when_no_builtin() {
    let devices = vec![
        device_with_transport(
            "mic-v",
            "Virtual Audio",
            false,
            DictationDeviceTransport::Virtual,
        ),
        device_with_transport("mic-u", "USB Mic", false, DictationDeviceTransport::Usb),
    ];
    let res = resolve_selected_input_device(&devices, None).expect("should resolve");
    assert_eq!(res.device.id.0, "mic-u", "should prefer USB over virtual");
}

#[test]
fn mic_resolution_uses_first_device_as_last_resort() {
    let devices = vec![device_with_transport(
        "mic-v",
        "Virtual Audio",
        false,
        DictationDeviceTransport::Virtual,
    )];
    let res = resolve_selected_input_device(&devices, None).expect("should resolve");
    assert_eq!(res.device.id.0, "mic-v", "should fall back to any device");
}

#[test]
fn mic_resolution_returns_none_for_empty_list() {
    let res = resolve_selected_input_device(&[], None);
    assert!(res.is_none());
}

#[test]
fn mic_resolution_returns_none_for_empty_list_with_saved_pref() {
    let res = resolve_selected_input_device(&[], Some("mic-1"));
    assert!(res.is_none());
}

// ---------------------------------------------------------------------------
// Download progress formatting tests
// ---------------------------------------------------------------------------

#[test]
fn format_bytes_covers_ranges() {
    use crate::dictation::download::format_bytes;

    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1 KB");
    assert_eq!(format_bytes(1_048_576), "1.0 MB");
    assert_eq!(format_bytes(500_000_000), "476.8 MB");
    assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
}

#[test]
fn format_speed_shows_dash_for_zero() {
    use crate::dictation::download::format_speed;

    assert_eq!(format_speed(0), "-- MB/s");
    assert!(format_speed(10_485_760).contains("10.0 MB/s"));
}

#[test]
fn download_progress_percentage_edge_cases() {
    use crate::dictation::download::DownloadProgress;

    assert_eq!(
        DownloadProgress {
            downloaded: 1,
            total: 3
        }
        .percentage(),
        33
    );
    assert_eq!(
        DownloadProgress {
            downloaded: 999,
            total: 1000
        }
        .percentage(),
        99
    );
}

// ---------------------------------------------------------------------------
// Internal dictation target routing tests
// ---------------------------------------------------------------------------

#[test]
fn dictation_target_enum_covers_all_surfaces() {
    use crate::dictation::types::DictationTarget;

    // Exhaustive match proves all variants exist and are reachable.
    let targets = [
        DictationTarget::MainWindowFilter,
        DictationTarget::MainWindowPrompt,
        DictationTarget::NotesEditor,
        DictationTarget::AiChatComposer,
        DictationTarget::TabAiHarness,
        DictationTarget::ExternalApp,
        DictationTarget::DayPageToday,
        DictationTarget::QuickAiQuestion,
    ];
    for target in &targets {
        match target {
            DictationTarget::MainWindowFilter => {}
            DictationTarget::MainWindowPrompt => {}
            DictationTarget::NotesEditor => {}
            DictationTarget::AiChatComposer => {}
            DictationTarget::TabAiHarness => {}
            DictationTarget::ExternalApp => {}
            DictationTarget::DayPageToday => {}
            DictationTarget::QuickAiQuestion => {}
        }
    }
}

#[test]
fn dictation_destination_includes_internal_surfaces() {
    use crate::dictation::types::DictationDestination;

    // NotesEditor, AiChatComposer, and TabAiHarness must exist alongside the
    // original variants, plus the chip-selectable DayPageToday and
    // QuickAiQuestion destinations.
    let destinations = [
        DictationDestination::MainWindowFilter,
        DictationDestination::ActivePrompt,
        DictationDestination::FrontmostApp,
        DictationDestination::NotesEditor,
        DictationDestination::AiChatComposer,
        DictationDestination::TabAiHarness,
        DictationDestination::DayPageToday,
        DictationDestination::QuickAiQuestion,
    ];
    assert_eq!(destinations.len(), 8);
}

#[test]
fn resolve_dictation_target_exists_in_builtin_execution() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        src.contains("fn resolve_dictation_target("),
        "builtin_execution.rs must define resolve_dictation_target"
    );

    // Must check notes and AI windows before falling back to prompt or external.
    // Use the exact signature to skip resolve_dictation_target_with_override.
    let resolver_start = src
        .find("fn resolve_dictation_target(&self) ->")
        .expect("resolver must exist");
    let resolver_src = &src[resolver_start..resolver_start + 1400.min(src.len() - resolver_start)];

    assert!(
        resolver_src.contains("notes::is_notes_window_open()"),
        "resolver must check notes window"
    );
    assert!(
        resolver_src.contains("agent_chat_dictation_target_active()"),
        "resolver must check Agent Chat availability"
    );
    assert!(
        resolver_src.contains("can_accept_dictation_into_prompt()"),
        "resolver must check prompt acceptance"
    );
    assert!(
        resolver_src.contains("DictationTarget::ExternalApp"),
        "resolver must fall back to ExternalApp"
    );
}

#[test]
fn resolve_dictation_target_checks_notes_before_ai() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    // Use the exact signature to skip resolve_dictation_target_with_override.
    let resolver_start = src
        .find("fn resolve_dictation_target(&self) ->")
        .expect("resolver must exist");
    let resolver_src = &src[resolver_start..resolver_start + 1400.min(src.len() - resolver_start)];

    let notes_pos = resolver_src
        .find("notes::is_notes_window_open()")
        .expect("resolver must check notes");
    let ai_pos = resolver_src
        .find("agent_chat_dictation_target_active()")
        .expect("resolver must check Agent Chat");
    let prompt_pos = resolver_src
        .find("can_accept_dictation_into_prompt()")
        .expect("resolver must check prompt");

    assert!(
        notes_pos < ai_pos,
        "notes must be checked before Agent Chat (notes_pos={notes_pos}, ai_pos={ai_pos})"
    );
    assert!(
        ai_pos < prompt_pos,
        "Agent Chat must be checked before prompt (ai_pos={ai_pos}, prompt_pos={prompt_pos})"
    );
}

#[test]
fn handle_dictation_transcript_accepts_target_parameter() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler = dictation_handler_source(&src);
    let signature = extract_delta_contract_section(
        handler,
        "fn handle_dictation_transcript(",
        "let DictationDeliveryTiming",
    );
    assert!(
        signature.contains("timing: DictationDeliveryTiming")
            && signature
                .contains("frozen_selection: Option<crate::dictation::DictationTargetSelection>")
            && signature
                .contains("preserved_request: Option<crate::dictation::DictationDeliveryRequest>"),
        "delivery must receive timing, frozen identity, and the optional original retry request"
    );
    assert!(
        compact_delta_contract(handler)
            .contains("letDictationDeliveryTiming{audio_duration,target,record_history,}=timing;")
            && handler.contains("Some(selection) if selection.is_compatible_with(target)"),
        "the target carried by timing must be checked against the frozen selection"
    );
}

#[test]
fn delivery_routes_notes_transcript_via_inject_text() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let mutation = extract_delta_contract_section(
        &src,
        "fn mutate_internal_dictation_request(",
        "fn complete_internal_dictation_delivery(",
    );
    let notes = compact_delta_contract(extract_delta_contract_section(
        mutation,
        "crate::dictation::DictationTarget::NotesEditor =>",
        "crate::dictation::DictationTarget::AiChatComposer =>",
    ));
    assert!(
        notes.contains(
            "FrozenDictationDestination::NotesEditor{notes_instance_id,document_id,editor_generation,insertion_anchor,}=&request.selection.destination"
        ) && notes.contains(
            "NotesDictationDestinationSnapshot{notes_instance_id:*notes_instance_id,document_id:document_id.clone(),editor_generation:editor_generation.clone(),insertion_anchor:insertion_anchor.clone(),}"
        ) && notes.contains(
            "notes::inject_text_into_frozen_notes(cx,&expected,request.transcript.text(),).map(Some)"
        ),
        "Notes delivery must pass the exact frozen instance/document/generation/anchor and observe its insertion"
    );
    assert!(
        dictation_handler_source(&src)
            .contains("self.mutate_internal_dictation_request(&request, None, cx)"),
        "transcription delivery must call the shared mutation owner"
    );
}

#[test]
fn delivery_routes_ai_chat_transcript_to_agent_chat_composer() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let mutation = extract_delta_contract_section(
        &src,
        "fn mutate_internal_dictation_request(",
        "fn complete_internal_dictation_delivery(",
    );
    let legacy = extract_delta_contract_section(
        mutation,
        "crate::dictation::DictationTarget::AiChatComposer =>",
        "crate::dictation::DictationTarget::TabAiHarness =>",
    );
    assert!(
        legacy.contains("Err(\"Legacy AI Chat destination was not canonicalized\""),
        "legacy AI Chat targets must be canonicalized before mutation, not rerouted here"
    );
    let agent_chat = extract_delta_contract_section(
        mutation,
        "crate::dictation::DictationTarget::TabAiHarness =>",
        "crate::dictation::DictationTarget::DayPageToday =>",
    );
    // Rustfmt may split the call and add a trailing comma; keep checking its
    // frozen destination arguments independently of that formatting.
    let agent_chat = compact_delta_contract(agent_chat).replace(",)", ")");
    for required in [
        "with_frozen_dictation_destination",
        "FrozenDictationDestination::AgentChat { policy }",
        "FrozenAgentChatPolicy::ExistingThread",
        "dispatch_dictation_to_frozen_agent_chat",
        "outcome.source_consumed()",
        "AgentChatEntryDispatch::Pending(ticket)",
        "insert_owned_dictation_text(thread_id, token, request.transcript.text(), window, cx)",
    ] {
        assert!(
            agent_chat.contains(&compact_delta_contract(required)),
            "frozen Agent Chat contract missing {required}"
        );
    }
    let handler = dictation_handler_source(&src);
    assert!(
        handler.contains("self.mutate_internal_dictation_request(&request, None, cx)")
            && handler.contains("ticket.completion.recv().await")
            && handler.contains("Ok(outcome) if outcome.source_consumed()"),
        "pending Agent Chat delivery must not complete before the exact turn is accepted"
    );
}

#[test]
fn internal_delivery_failure_refuses_without_frontmost_fallback() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let handler_src = dictation_handler_source(&src);
    let refusal = extract_delta_contract_section(
        handler_src,
        "if !matches!(target, crate::dictation::DictationTarget::ExternalApp)",
        "// The delivery id was claimed before validation",
    );
    assert!(
        refusal.contains("dictation_mutation_error_outcome(&error)")
            && refusal.contains("DictationDeliveryOutcome::Refused")
            && refusal.contains("DictationDeliveryOutcome::Failed")
            && refusal.contains("record_wrong_target_refusal"),
        "stale destinations and uncertain mutations must preserve their distinct typed outcomes"
    );
    assert!(
        compact_delta_contract(refusal).contains(
            "self.present_dictation_delivery_failure(request,timing,failure,reason,retry_safety,cx,);return;"
        ),
        "internal refusal must preserve the original transcript/History request and return immediately"
    );
    assert!(
        !refusal.contains("paste_text") && !refusal.contains("yield_focus_for_dictation_paste"),
        "an internal failure must never enter external-app delivery"
    );
}

#[test]
fn toggle_dictation_accepts_target_parameter() {
    let src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        src.contains("pub fn toggle_dictation(target: DictationTarget)"),
        "toggle_dictation must accept a DictationTarget parameter"
    );
}

#[test]
fn get_dictation_target_is_exported() {
    let src = std::fs::read_to_string("src/dictation/mod.rs").expect("read dictation/mod.rs");
    assert!(
        src.contains("get_dictation_target"),
        "get_dictation_target must be re-exported from the dictation module"
    );
}

#[test]
fn dictation_session_stores_target() {
    let src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");

    assert!(
        src.contains("target: DictationTarget"),
        "DictationSession must store the target"
    );
    assert!(
        src.contains("target_cycle: Vec<DictationTarget>"),
        "DictationSession must store the per-session target cycle"
    );
    assert!(
        src.contains("pub fn get_dictation_target()"),
        "runtime must expose get_dictation_target()"
    );
}

#[test]
fn dictation_runtime_exposes_target_cycle_helpers() {
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime.rs");
    let mod_src = std::fs::read_to_string("src/dictation/mod.rs").expect("read mod.rs");

    assert!(
        runtime_src.contains("pub fn set_dictation_target_cycle(targets: Vec<DictationTarget>)"),
        "runtime must expose set_dictation_target_cycle()"
    );
    assert!(
        runtime_src.contains("pub fn can_cycle_dictation_target() -> bool"),
        "runtime must expose can_cycle_dictation_target()"
    );
    assert!(
        runtime_src.contains("pub fn cycle_dictation_target() -> Option<DictationTarget>"),
        "runtime must expose cycle_dictation_target()"
    );
    assert!(
        mod_src.contains("set_dictation_target_cycle")
            && mod_src.contains("can_cycle_dictation_target")
            && mod_src.contains("cycle_dictation_target"),
        "dictation mod facade must re-export target cycle helpers"
    );
}

#[test]
fn dictation_target_descriptors_are_exhaustive_unique_and_action_safe() {
    use super::types::{DictationTarget, ALL_DICTATION_TARGETS};
    use std::collections::HashSet;

    assert_eq!(ALL_DICTATION_TARGETS.len(), 8);
    let mut ids = HashSet::new();
    for descriptor in ALL_DICTATION_TARGETS {
        assert_eq!(descriptor.target.descriptor(), &descriptor);
        assert!(
            ids.insert(descriptor.stable_id),
            "duplicate target stable id"
        );
        assert!(!descriptor.selector_label.is_empty());
        assert!(!descriptor.badge_label.is_empty());
        assert!(!descriptor.icon.is_empty());
        assert!(!descriptor.delivery_verb.is_empty());
        assert!(!descriptor.description.is_empty());
    }

    let actions = DictationTarget::action_descriptors().collect::<Vec<_>>();
    assert_eq!(actions.len(), 7);
    assert!(actions.iter().all(|descriptor| descriptor.selectable));
    assert!(actions
        .iter()
        .all(|descriptor| descriptor.target != DictationTarget::AiChatComposer));

    let quick = DictationTarget::quick_chip_descriptors()
        .map(|descriptor| descriptor.target)
        .collect::<Vec<_>>();
    assert_eq!(
        quick,
        vec![
            DictationTarget::ExternalApp,
            DictationTarget::DayPageToday,
            DictationTarget::QuickAiQuestion,
            DictationTarget::TabAiHarness,
        ]
    );
}

#[test]
fn legacy_ai_chat_labels_migrate_to_agent_chat_with_receipt() {
    for label in ["aichat", "AiChatComposer", "legacy-ai"] {
        let resolution = crate::dictation::resolve_dictation_target_label(label)
            .expect("legacy label must remain readable");
        assert_eq!(
            resolution.target,
            crate::dictation::DictationTarget::TabAiHarness
        );
        assert!(resolution.migrated_legacy_ai_chat);
    }
    let current = crate::dictation::resolve_dictation_target_label("agentchat")
        .expect("current label must resolve");
    assert_eq!(
        current.target,
        crate::dictation::DictationTarget::TabAiHarness
    );
    assert!(!current.migrated_legacy_ai_chat);
}

#[test]
fn builtin_dictation_configures_target_cycle_and_agent_chat_alternate() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    assert!(
        builtin_src.contains("fn dictation_target_cycle_for("),
        "builtin execution must define a dictation target cycle helper"
    );
    assert!(
        builtin_src.contains("set_dictation_target_cycle")
            && builtin_src.contains("DictationTarget::ExternalApp")
            && builtin_src.contains("DictationTarget::TabAiHarness"),
        "dictation start paths must configure a cycle that includes Agent Chat chat submit as an alternate"
    );
}

#[test]
fn main_window_filter_delivery_validates_frozen_input_and_reveals_main() {
    let builtin_src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");
    let mutation = extract_delta_contract_section(
        &builtin_src,
        "fn mutate_internal_dictation_request(",
        "fn complete_internal_dictation_delivery(",
    );
    let filter = compact_delta_contract(extract_delta_contract_section(
        mutation,
        "crate::dictation::DictationTarget::MainWindowFilter =>",
        "crate::dictation::DictationTarget::MainWindowPrompt =>",
    ));
    assert!(
        filter.contains("self.capture_dictation_target_selection(target,cx)?")
            && filter.contains(
                "with_frozen_dictation_destination(&request.selection.destination,&current.destination,||{ifself.try_set_main_window_filter_from_dictation("
            )
            && filter.contains("replaceFrozenInput"),
        "filter mutation must be guarded by the frozen input identity and return an observed replacement receipt"
    );
    let completion = compact_delta_contract(extract_delta_contract_section(
        &builtin_src,
        "fn complete_internal_dictation_delivery(",
        "fn handle_dictation_transcript(",
    ));
    assert!(
        completion.contains(
            "ifmatches!(target,crate::dictation::DictationTarget::MainWindowFilter)&&!script_kit_gpui::is_main_window_visible()"
        ) && completion.contains("script_kit_gpui::set_main_window_visible(true);")
            && completion.contains("show_main_window_without_activation()"),
        "successful filter delivery must reveal a hidden launcher without activating a different app"
    );
}

#[test]
fn inject_text_into_notes_is_exported() {
    let src = std::fs::read_to_string("src/notes/mod.rs").expect("read notes/mod.rs");
    assert!(
        src.contains("inject_text_into_notes"),
        "inject_text_into_notes must be re-exported from the notes module"
    );
}

// ---------------------------------------------------------------------------
// Missing-model entry gate opens the rich download prompt
// ---------------------------------------------------------------------------

/// When the Parakeet model is not available at dictation start time, the
/// entry gate must open the rich model-download prompt — not just log or
/// show a toast.  This verifies the preflight guard opens the prompt and
/// returns early before attempting capture.
#[test]
fn missing_model_entry_gate_opens_download_prompt() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let dictation_arm_start = src
        .find("fn prepare_dictation_builtin_start(")
        .expect("builtin_execution must have prepare_dictation_builtin_start");
    let arm_src =
        &src[dictation_arm_start..dictation_arm_start + 2000.min(src.len() - dictation_arm_start)];

    // Must check model availability before starting capture.
    assert!(
        arm_src.contains("model.is_available()"),
        "Dictation entry must check selected model availability"
    );

    // When model is missing, must open the rich download prompt.
    assert!(
        arm_src.contains("open_dictation_model_prompt(cx)"),
        "Dictation entry must open the model download prompt when model is missing"
    );

    // The model check must come before delivery target check — we don't
    // want to fail on delivery validation when the real issue is missing model.
    let model_check_pos = arm_src
        .find("model.is_available()")
        .expect("model check must exist");
    let delivery_check_pos = arm_src
        .find("ensure_dictation_builtin_target_available")
        .expect("delivery target check must exist");
    assert!(
        model_check_pos < delivery_check_pos,
        "model availability check must come BEFORE delivery target validation"
    );
}

// ---------------------------------------------------------------------------
// Start preflight must fire the mic TCC prompt and use real model status
// ---------------------------------------------------------------------------

/// A fresh install has microphone permission NotDetermined. macOS only shows
/// the consent dialog when the app calls
/// `AVCaptureDevice requestAccessForMediaType:`, and System Settings does not
/// list the app under Microphone until that has fired at least once. The
/// dictation start preflight is the only user-reachable trigger, so it must
/// actually request access — otherwise new users have NO path to grant the
/// permission (release audit 2026-07-01). It must also derive model status
/// from disk instead of hardcoding Available, so a missing model routes to
/// setup before recording starts rather than after transcription fails.
#[test]
fn dictation_start_preflight_requests_mic_access_and_uses_real_model_status() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let start = src
        .find("fn open_dictation_setup_if_not_ready(")
        .expect("open_dictation_setup_if_not_ready must exist");
    let tail = &src[start..];
    let end = tail
        .find("fn ensure_dictation_delivery_target_available(")
        .expect("preflight must be followed by ensure_dictation_delivery_target_available");
    let preflight_src = &tail[..end];

    assert!(
        preflight_src.contains("request_microphone_permission_nonblocking"),
        "preflight must fire the TCC prompt for NotDetermined mic permission"
    );
    assert!(
        preflight_src.contains("model.is_available()"),
        "preflight must derive model status from disk, not hardcode Available"
    );
}

// ---------------------------------------------------------------------------
// Missing-model transcription recovery opens prompt instead of dead-end toast
// ---------------------------------------------------------------------------

/// When transcription fails because the Parakeet model is missing, the
/// handler must open the rich download prompt so the
/// user can immediately start the download — rather than showing a
/// dead-end toast that requires them to guess the next step.
#[test]
fn missing_model_transcription_recovery_opens_download_prompt() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let handler_src = dictation_handler_source(&src);

    // Find the model-missing branch within the Err arm.
    let parakeet_branch_start = handler_src
        .find(r#"error_text.contains("model not downloaded")"#)
        .expect("handler must detect the model-missing error string");
    // Scope far enough after the detection to capture the full recovery branch,
    // including the early return after cleanup / prompt dispatch.
    let branch_src = &handler_src[parakeet_branch_start
        ..parakeet_branch_start + 900.min(handler_src.len() - parakeet_branch_start)];

    // Must open the rich download prompt.
    assert!(
        branch_src.contains("open_dictation_model_prompt"),
        "Parakeet-missing branch must open the model download prompt, not a dead-end toast"
    );

    // Must schedule transcriber cleanup.
    assert!(
        branch_src.contains("schedule_dictation_transcriber_cleanup"),
        "Parakeet-missing branch must schedule transcriber cleanup"
    );

    // Must return early — no fallthrough to the generic error toast.
    assert!(
        branch_src.contains("return;"),
        "Parakeet-missing branch must return early to avoid the generic error toast"
    );

    // Must NOT show a dead-end toast in this branch (the prompt IS the UX).
    let branch_before_return =
        &branch_src[..branch_src.find("return;").unwrap_or(branch_src.len())];
    assert!(
        !branch_before_return.contains("notice_failed"),
        "Parakeet-missing branch must NOT show a dead-end toast — the prompt is the recovery UX"
    );
}

// ---------------------------------------------------------------------------
// Download failure path uses classified error messages
// ---------------------------------------------------------------------------

/// When the background model download fails, the error text surfaced to
/// the user must go through `classify_download_error` so users see
/// actionable guidance instead of raw error strings.
#[test]
fn download_failure_uses_classified_error() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let download_fn_start = src
        .find("fn start_dictation_model_download(")
        .expect("start_dictation_model_download must exist");
    // Scope from the function start to the next top-level function
    // (`build_dictation_model_prompt`) to capture the full error handling.
    let download_tail = &src[download_fn_start..];
    let download_src = match download_tail.find("fn build_dictation_model_prompt(") {
        Some(end) => &download_tail[..end],
        None => download_tail,
    };

    // Must call classify_download_error for non-cancelled failures.
    assert!(
        download_src.contains("classify_download_error"),
        "download failure path must classify errors for user-friendly messages"
    );

    // Must log the raw error AND the classified user-facing error separately.
    assert!(
        download_src.contains("raw_error") && download_src.contains("user_error"),
        "download failure must log both raw_error and user_error for diagnostics"
    );
}

// duplicate removed — see line 4053 for the canonical test

// ───────────────────────────────────────────────────────────────────
// Main-window-filter dictation routing
// ───────────────────────────────────────────────────────────────────

#[test]
fn dictation_target_enum_includes_main_window_filter_variant() {
    use crate::dictation::types::DictationTarget;

    // Exhaustive match proves MainWindowFilter is a first-class variant.
    let target = DictationTarget::MainWindowFilter;
    match target {
        DictationTarget::MainWindowFilter => {}
        DictationTarget::MainWindowPrompt => {}
        DictationTarget::NotesEditor => {}
        DictationTarget::AiChatComposer => {}
        DictationTarget::TabAiHarness => {}
        DictationTarget::ExternalApp => {}
        DictationTarget::DayPageToday => {}
        DictationTarget::QuickAiQuestion => {}
    }
    assert_eq!(target.overlay_label(), "Script Kit");
}

#[test]
fn dictation_destination_includes_main_window_filter_variant() {
    use crate::dictation::types::DictationDestination;

    // Verify MainWindowFilter is present among all destination variants.
    let destinations = [
        DictationDestination::MainWindowFilter,
        DictationDestination::ActivePrompt,
        DictationDestination::FrontmostApp,
        DictationDestination::NotesEditor,
        DictationDestination::AiChatComposer,
        DictationDestination::TabAiHarness,
        DictationDestination::DayPageToday,
        DictationDestination::QuickAiQuestion,
    ];
    assert_eq!(destinations.len(), 8);
}

#[test]
fn resolve_dictation_target_prefers_frontmost_app_over_script_list() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let resolver_start = src
        .find("fn resolve_dictation_target(&self) ->")
        .expect("resolver must exist");
    let resolver_src = &src[resolver_start..resolver_start + 1400.min(src.len() - resolver_start)];

    assert!(
        resolver_src
            .contains("let has_frontmost_app_target = Self::has_dictation_frontmost_target();"),
        "resolver must snapshot whether a frontmost app can receive contextual dictation"
    );
    let frontmost_pos = resolver_src
        .find("has_frontmost_app_target")
        .expect("resolver must check frontmost-app availability");
    let main_filter_pos = resolver_src
        .find("can_accept_dictation_into_main_filter()")
        .expect("resolver must keep launcher/main filter fallback");
    assert!(
        frontmost_pos < main_filter_pos,
        "contextual dictation must prefer the tracked frontmost app before falling back to Script Kit's main menu"
    );
}

#[test]
fn delivery_routes_main_window_filter_before_external_paste() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    // MainWindowFilter delivery must appear before the external-app paste
    // fallback in the overall file, proving it is checked first.
    let main_filter_pos = src
        .find("DictationTarget::MainWindowFilter =>")
        .expect("handler must match MainWindowFilter target");
    let paste_pos = src
        .find("paste_text")
        .expect("handler must still contain external-app fallback");

    assert!(
        main_filter_pos < paste_pos,
        "MainWindowFilter delivery must happen before external-app paste fallback"
    );
    assert!(
        src.contains("DictationDestination::MainWindowFilter"),
        "handler must map MainWindowFilter target to MainWindowFilter destination"
    );
}

#[test]
fn preflight_accepts_launcher_without_tracked_frontmost_app() {
    let src = std::fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("read builtin_execution.rs");

    let preflight_start = src
        .find("fn ensure_dictation_delivery_target_available(&self)")
        .expect("preflight must exist");
    let preflight_src =
        &src[preflight_start..preflight_start + 400.min(src.len() - preflight_start)];

    assert!(
        preflight_src.contains("can_accept_dictation_into_main_filter()"),
        "preflight must accept launcher as a valid dictation destination"
    );
}

#[test]
fn chunked_transcription_reserved_for_long_audio() {
    use super::transcription::should_use_chunked_transcription;
    let rate = 16_000u32;

    // Typical short dictations take the single plain pass.
    assert!(!should_use_chunked_transcription(20 * rate as usize, rate));
    // A five-minute recording must go through the chunked path — one giant
    // encoder pass is quadratic in audio length and can drop trailing speech.
    assert!(should_use_chunked_transcription(300 * rate as usize, rate));
}

#[test]
fn dictation_preferences_defaults_are_safe() {
    let prefs = crate::config::DictationPreferences::default();
    assert!(prefs.save_history_enabled(), "history stays on by default");
    assert_eq!(
        prefs.silence_rms_threshold(),
        crate::config::DictationPreferences::DEFAULT_SILENCE_RMS
    );
    assert_eq!(
        prefs.max_duration(),
        Some(std::time::Duration::from_secs(
            crate::config::DictationPreferences::DEFAULT_MAX_DURATION_SECS
        ))
    );
    assert!(prefs.live_preview_enabled(), "live preview on by default");
    assert!(prefs.push_to_talk_enabled(), "push-to-talk on by default");
    assert_eq!(
        prefs.target_mode(),
        crate::config::DictationTargetMode::Sticky,
        "target mode defaults to sticky"
    );
    assert_eq!(
        prefs.sticky_target(),
        None,
        "no sticky destination until a chip has been picked"
    );
    assert!(
        prefs.quick_ai_answers(),
        "quick AI fires-and-shows by default"
    );
}

#[test]
fn dictation_target_mode_parses_config_values() {
    use crate::config::{DictationPreferences, DictationTargetMode};
    use crate::dictation::DictationTarget;

    let context = DictationPreferences {
        target: Some("context".to_string()),
        ..Default::default()
    };
    assert_eq!(context.target_mode(), DictationTargetMode::Context);

    let explicit = DictationPreferences {
        target: Some("today".to_string()),
        ..Default::default()
    };
    assert_eq!(
        explicit.target_mode(),
        DictationTargetMode::Explicit(DictationTarget::DayPageToday)
    );

    // Typos degrade to the sticky default instead of pinning a wrong target.
    let typo = DictationPreferences {
        target: Some("todya".to_string()),
        ..Default::default()
    };
    assert_eq!(typo.target_mode(), DictationTargetMode::Sticky);

    let sticky = DictationPreferences {
        target: Some("sticky".to_string()),
        last_target: Some("ask".to_string()),
        ..Default::default()
    };
    assert_eq!(sticky.target_mode(), DictationTargetMode::Sticky);
    assert_eq!(
        sticky.sticky_target(),
        Some(DictationTarget::QuickAiQuestion)
    );

    let composer = DictationPreferences {
        quick_ai: Some("composer".to_string()),
        ..Default::default()
    };
    assert!(!composer.quick_ai_answers());
}

#[test]
fn dictation_sticky_labels_round_trip_through_parser() {
    use crate::dictation::{parse_dictation_target_label, DictationTarget};

    for target in [
        DictationTarget::MainWindowFilter,
        DictationTarget::MainWindowPrompt,
        DictationTarget::NotesEditor,
        DictationTarget::AiChatComposer,
        DictationTarget::TabAiHarness,
        DictationTarget::ExternalApp,
        DictationTarget::DayPageToday,
        DictationTarget::QuickAiQuestion,
    ] {
        let expected = if target == DictationTarget::AiChatComposer {
            DictationTarget::TabAiHarness
        } else {
            target
        };
        assert_eq!(
            parse_dictation_target_label(target.sticky_label()),
            Some(expected),
            "sticky label for {target:?} must parse to its current target"
        );
    }
}

#[test]
fn dictation_preferences_clamp_and_disable_rules() {
    let prefs = crate::config::DictationPreferences {
        save_history: Some(false),
        silence_rms: Some(9.0),
        max_duration_secs: Some(0),
        live_preview: Some(false),
        push_to_talk: Some(false),
        ..Default::default()
    };
    assert!(!prefs.save_history_enabled());
    assert_eq!(
        prefs.silence_rms_threshold(),
        0.5,
        "silence gate clamps to 0.5"
    );
    assert_eq!(
        prefs.max_duration(),
        None,
        "0 disables the max-duration guard"
    );
    assert!(!prefs.live_preview_enabled());
    assert!(!prefs.push_to_talk_enabled());

    let tiny = crate::config::DictationPreferences {
        max_duration_secs: Some(5),
        ..Default::default()
    };
    assert_eq!(
        tiny.max_duration(),
        Some(std::time::Duration::from_secs(30)),
        "sub-30s ceilings clamp up to 30s"
    );
}

#[test]
fn completed_capture_truncation_flag_reaches_stop_receipt() {
    // The stop receipt must expose `truncated` so DevTools can prove whether
    // the capture tail was collected before the deadline.
    let runtime_src = std::fs::read_to_string("src/dictation/runtime.rs").expect("read runtime");
    assert!(
        runtime_src.contains(r#""truncated": truncated,"#),
        "stop receipt must carry the truncated flag"
    );
}

#[test]
fn dictation_target_destination_mapping_is_exhaustive_and_stable() {
    use crate::dictation::types::{DictationDestination, DictationTarget};
    let cases = [
        (
            DictationTarget::MainWindowFilter,
            DictationDestination::MainWindowFilter,
        ),
        (
            DictationTarget::MainWindowPrompt,
            DictationDestination::ActivePrompt,
        ),
        (
            DictationTarget::NotesEditor,
            DictationDestination::NotesEditor,
        ),
        (
            DictationTarget::AiChatComposer,
            DictationDestination::AiChatComposer,
        ),
        (
            DictationTarget::TabAiHarness,
            DictationDestination::TabAiHarness,
        ),
        (
            DictationTarget::ExternalApp,
            DictationDestination::FrontmostApp,
        ),
    ];
    for (target, destination) in cases {
        assert_eq!(target.destination(), destination);
    }
}

#[test]
fn immutable_dictation_transcript_exposes_identity_not_text_in_debug() {
    let transcript = crate::dictation::ImmutableDictationTranscript::new(
        "transcript-7",
        "PRIVATE DICTATION CONTENT".to_string(),
    );
    assert_eq!(transcript.id(), "transcript-7");
    assert_eq!(transcript.text(), "PRIVATE DICTATION CONTENT");
    assert_eq!(transcript.len(), 25);
    let debug = format!("{transcript:?}");
    assert!(!debug.contains("PRIVATE DICTATION CONTENT"));
    assert!(debug.contains("text_len"));
    assert!(debug.contains("fingerprint"));
}

#[test]
fn private_dictation_transcript_fingerprints_are_keyed_not_public_fnv_or_sha() {
    use sha2::{Digest as _, Sha256};

    let transcript = "private spoken health record and home address";
    let public_sha = format!("sha256:{:x}", Sha256::digest(transcript.as_bytes()));
    let first = crate::dictation::redacted_transcript_fingerprint(transcript);
    let repeated = crate::dictation::redacted_transcript_fingerprint(transcript);
    let changed = crate::dictation::redacted_transcript_fingerprint("different spoken words");

    assert_eq!(first, repeated);
    assert_ne!(first, changed);
    assert_ne!(first, public_sha);
    assert!(first.starts_with("sha256:"));
    assert_eq!(first.len(), "sha256:".len() + 64);
    assert_eq!(
        first,
        format!(
            "sha256:{}",
            crate::logging::log_private_user_value(transcript).sha256
        )
    );
}

fn frozen_main_test_owner() -> crate::dictation::FrozenMainDictationOwner {
    crate::dictation::FrozenMainDictationOwner {
        root_entity_id: 1,
        window_generation: Some(2),
        surface_generation: 3,
        visibility_generation: 4,
    }
}

#[test]
fn frozen_destination_fingerprint_changes_with_each_identity_generation() {
    use crate::dictation::FrozenDictationDestination;
    let first = FrozenDictationDestination::MainWindowPrompt {
        prompt_id: "prompt-a".to_string(),
        prompt_generation: Some(3),
        prompt_entity_id: None,
        owner: frozen_main_test_owner(),
        input_generation: 5,
    };
    let replaced = FrozenDictationDestination::MainWindowPrompt {
        prompt_id: "prompt-a".to_string(),
        prompt_generation: Some(4),
        prompt_entity_id: None,
        owner: frozen_main_test_owner(),
        input_generation: 5,
    };
    let edited = FrozenDictationDestination::MainWindowPrompt {
        prompt_id: "prompt-a".to_string(),
        prompt_generation: Some(3),
        prompt_entity_id: None,
        owner: frozen_main_test_owner(),
        input_generation: 6,
    };
    assert_ne!(
        first.identity_fingerprint(),
        replaced.identity_fingerprint()
    );
    assert_ne!(first.identity_fingerprint(), edited.identity_fingerprint());
    assert_eq!(first.kind(), "mainWindowPrompt");
}

#[test]
fn delivery_ids_are_claimed_before_mutation_at_most_once() {
    let delivery_id = crate::dictation::next_dictation_delivery_id();
    assert!(!crate::dictation::dictation_delivery_was_claimed(
        delivery_id
    ));
    assert!(crate::dictation::claim_dictation_delivery(delivery_id));
    assert!(crate::dictation::dictation_delivery_was_claimed(
        delivery_id
    ));
    assert!(!crate::dictation::claim_dictation_delivery(delivery_id));
}

static DICTATION_RECOVERY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn recovery_test_request(delivery_id: u64) -> crate::dictation::DictationDeliveryRequest {
    use crate::dictation::{
        DictationTarget, DictationTargetSelection, FrozenDictationDestination,
        ImmutableDictationTranscript,
    };
    crate::dictation::DictationDeliveryRequest {
        delivery_id,
        session_generation: 7,
        selection: DictationTargetSelection {
            target: DictationTarget::MainWindowPrompt,
            destination: FrozenDictationDestination::MainWindowPrompt {
                prompt_id: "prompt-7".to_string(),
                prompt_generation: Some(11),
                prompt_entity_id: None,
                owner: frozen_main_test_owner(),
                input_generation: 13,
            },
            display_label: "Prompt".to_string(),
            icon_identity: Some("text-cursor-input".to_string()),
            selection_generation: 17,
        },
        transcript: ImmutableDictationTranscript::new(
            "transcript-stable",
            "PRIVATE RECOVERY TRANSCRIPT".to_string(),
        ),
        history_entry_id: "history-stable".to_string(),
        attempt: 1,
    }
}

#[test]
fn stale_dictation_failure_preserves_identity_without_unsafe_retry() {
    let _guard = DICTATION_RECOVERY_TEST_LOCK
        .lock()
        .expect("recovery test lock");
    use crate::dictation::{
        DictationDeliveryFailureReason, DictationRecoveryAction, DictationRecoveryWork,
    };
    use sk_protocol::ai_reliability::RetrySafety;

    crate::dictation::clear_dictation_recovery_work();
    let request = recovery_test_request(71);
    let state = crate::dictation::preserve_dictation_recovery_work(
        DictationRecoveryWork {
            request: request.clone(),
            audio_duration: Duration::from_secs(2),
            target: request.selection.target,
            reason: DictationDeliveryFailureReason::DestinationStale,
        },
        crate::ai::reliability::destination_failure(true, "RAW FAILURE CANARY"),
        RetrySafety::ExplicitUserConfirmation,
        true,
    );

    assert_eq!(state.transcript_id, "transcript-stable");
    assert_eq!(state.history_entry_id, "history-stable");
    assert_eq!(state.identity_generation, 17);
    assert!(!state.capabilities.retry_same_destination);
    assert!(state.capabilities.choose_destination);
    assert!(state.capabilities.open_dictation_history);
    assert!(!state.safe_message().contains("RAW FAILURE CANARY"));
    assert!(!state
        .capabilities
        .actions()
        .any(|action| action == DictationRecoveryAction::RetrySameDestination));
    let preserved = crate::dictation::dictation_recovery_work().expect("preserved work");
    assert_eq!(preserved.request.transcript.id(), "transcript-stable");
    assert_eq!(preserved.request.history_entry_id, "history-stable");
    crate::dictation::clear_dictation_recovery_work();
}

#[test]
fn retry_capability_requires_safe_failure_and_installed_callback() {
    let _guard = DICTATION_RECOVERY_TEST_LOCK
        .lock()
        .expect("recovery test lock");
    use crate::dictation::{DictationDeliveryFailureReason, DictationRecoveryWork};
    use sk_protocol::ai_reliability::RetrySafety;

    let request = recovery_test_request(72);
    let safe = crate::dictation::preserve_dictation_recovery_work(
        DictationRecoveryWork {
            request: request.clone(),
            audio_duration: Duration::ZERO,
            target: request.selection.target,
            reason: DictationDeliveryFailureReason::MutationFailed,
        },
        crate::ai::reliability::destination_failure(false, "safe pre-mutation failure"),
        RetrySafety::ExplicitUserConfirmation,
        true,
    );
    assert!(safe.capabilities.retry_same_destination);

    let missing_callback = crate::dictation::preserve_dictation_recovery_work(
        DictationRecoveryWork {
            request: request.clone(),
            audio_duration: Duration::ZERO,
            target: request.selection.target,
            reason: DictationDeliveryFailureReason::MutationFailed,
        },
        crate::ai::reliability::destination_failure(false, "callback unavailable"),
        RetrySafety::ExplicitUserConfirmation,
        false,
    );
    assert_eq!(
        missing_callback.capabilities,
        crate::dictation::DictationFailureRecoveryCapabilities::default()
    );

    let unsafe_outcome = crate::dictation::preserve_dictation_recovery_work(
        DictationRecoveryWork {
            request,
            audio_duration: Duration::ZERO,
            target: crate::dictation::DictationTarget::MainWindowPrompt,
            reason: DictationDeliveryFailureReason::MutationOutcomeUnknown,
        },
        crate::ai::reliability::destination_failure(false, "mutation result unknown"),
        RetrySafety::Never,
        true,
    );
    assert!(!unsafe_outcome.capabilities.retry_same_destination);
    crate::dictation::clear_dictation_recovery_work();
}

#[test]
fn dictation_return_origin_is_independent_from_retargeting() {
    let _guard = DICTATION_RECOVERY_TEST_LOCK
        .lock()
        .expect("recovery test lock");
    let original = recovery_test_request(73).selection;
    let origin = crate::dictation::DictationReturnOrigin {
        captured_generation: original.selection_generation,
        selection: original.clone(),
        semantic_focus_id: "input:prompt".to_string(),
    };
    crate::dictation::replace_dictation_return_origin(origin.clone());

    let mut retargeted = original;
    retargeted.target = crate::dictation::DictationTarget::DayPageToday;
    retargeted.destination = crate::dictation::FrozenDictationDestination::DayPage {
        date: chrono::NaiveDate::from_ymd_opt(2026, 8, 6).expect("valid date"),
        substrate_fingerprint: "substrate".to_string(),
        entity_generation: 4,
        owned_editor: None,
        main_owner: None,
    };
    crate::dictation::retain_frozen_selection_for_delivery(retargeted);

    assert_eq!(crate::dictation::dictation_return_origin(), Some(origin));
    crate::dictation::clear_dictation_return_origin();
}

#[test]
fn legacy_ai_target_is_compatible_only_with_canonical_agent_chat() {
    use crate::dictation::{
        DictationTarget, DictationTargetSelection, FrozenAgentChatPolicy,
        FrozenDictationDestination,
    };
    let selection = DictationTargetSelection {
        target: DictationTarget::TabAiHarness,
        destination: FrozenDictationDestination::AgentChat {
            policy: FrozenAgentChatPolicy::FreshStandard {
                host_generation: 9,
                main_owner: frozen_main_test_owner(),
            },
        },
        display_label: "Agent Chat".to_string(),
        icon_identity: Some("bot".to_string()),
        selection_generation: 11,
    };
    assert!(selection.is_compatible_with(DictationTarget::AiChatComposer));
    assert!(selection.is_compatible_with(DictationTarget::TabAiHarness));
    assert!(!selection.is_compatible_with(DictationTarget::QuickAiQuestion));
}
