use crate::dictation::capture::{mix_to_mono, normalize_chunk, resample_linear};
use crate::dictation::transcription::{
    build_session_result, merge_captured_chunks, DictationEngine, DictationTranscriber,
    DictationTranscriptionConfig,
};
use crate::dictation::types::{
    CapturedAudioChunk, DictationCaptureConfig, DictationDestination, DictationLevel,
    RawAudioChunk,
};
use crate::dictation::visualizer::{bars_for_level, compute_level};
use anyhow::Result;
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
    assert_eq!(
        transcriber.transcribe_samples(&[0.0, 0.0, 0.0, 0.0])?,
        None
    );
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
