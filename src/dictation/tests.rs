use crate::dictation::capture::{mix_to_mono, normalize_chunk, resample_linear};
use crate::dictation::types::{DictationCaptureConfig, DictationLevel, RawAudioChunk};
use crate::dictation::visualizer::{bars_for_level, compute_level};
use std::time::Duration;

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
