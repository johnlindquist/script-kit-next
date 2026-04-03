use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// FFT-based audio visualizer (ported from vercel-voice)
// ---------------------------------------------------------------------------

/// dB range for normalization.
const DB_MIN: f32 = -55.0;
const DB_MAX: f32 = -8.0;
/// Amplitude gain applied after normalization.
const GAIN: f32 = 1.3;
/// Power-law curve for perceptual linearity.
const CURVE_POWER: f32 = 0.7;
/// Noise floor adaptation rate (very slow — 1000-sample averaging).
const NOISE_ALPHA: f32 = 0.001;

/// Number of output frequency buckets (one per waveform bar).
pub const BUCKET_COUNT: usize = 9;

/// FFT-based audio visualizer that converts time-domain samples into
/// frequency-domain bar heights for the dictation waveform.
///
/// Ported from vercel-voice `AudioVisualiser` with identical parameters.
pub struct AudioVisualiser {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    bucket_ranges: Vec<(usize, usize)>,
    fft_input: Vec<Complex32>,
    noise_floor: Vec<f32>,
    buffer: Vec<f32>,
    window_size: usize,
}

impl AudioVisualiser {
    /// Create a new visualizer for the given sample rate.
    ///
    /// Uses speech-optimized defaults: 1024-sample window, 85–4000 Hz range.
    pub fn new_speech(sample_rate: u32) -> Self {
        Self::new(sample_rate, 1024, 85.0, 4000.0)
    }

    pub fn new(sample_rate: u32, window_size: usize, freq_min: f32, freq_max: f32) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(window_size);

        // Pre-compute Hann window
        let window: Vec<f32> = (0..window_size)
            .map(|i| {
                0.5 * (1.0
                    - (2.0 * std::f32::consts::PI * i as f32 / window_size as f32).cos())
            })
            .collect();

        // Pre-compute bucket frequency ranges (logarithmic spacing)
        let nyquist = sample_rate as f32 / 2.0;
        let freq_min = freq_min.min(nyquist);
        let freq_max = freq_max.min(nyquist);

        let mut bucket_ranges = Vec::with_capacity(BUCKET_COUNT);

        for b in 0..BUCKET_COUNT {
            let log_start = (b as f32 / BUCKET_COUNT as f32).powi(2);
            let log_end = ((b + 1) as f32 / BUCKET_COUNT as f32).powi(2);

            let start_hz = freq_min + (freq_max - freq_min) * log_start;
            let end_hz = freq_min + (freq_max - freq_min) * log_end;

            let start_bin = ((start_hz * window_size as f32) / sample_rate as f32) as usize;
            let mut end_bin = ((end_hz * window_size as f32) / sample_rate as f32) as usize;

            // Ensure each bucket has at least one bin
            if end_bin <= start_bin {
                end_bin = start_bin + 1;
            }

            // Clamp to valid range
            let start_bin = start_bin.min(window_size / 2);
            let end_bin = end_bin.min(window_size / 2);

            bucket_ranges.push((start_bin, end_bin));
        }

        Self {
            fft,
            window,
            bucket_ranges,
            fft_input: vec![Complex32::new(0.0, 0.0); window_size],
            noise_floor: vec![-40.0; BUCKET_COUNT],
            buffer: Vec::with_capacity(window_size * 2),
            window_size,
        }
    }

    /// Feed audio samples and optionally receive frequency-domain bar levels.
    ///
    /// Returns `Some([f32; BUCKET_COUNT])` when enough samples have accumulated
    /// for a full FFT window. Each value is 0.0–1.0.
    pub fn feed(&mut self, samples: &[f32]) -> Option<[f32; BUCKET_COUNT]> {
        self.buffer.extend_from_slice(samples);

        if self.buffer.len() < self.window_size {
            return None;
        }

        let window_samples = &self.buffer[..self.window_size];

        // Remove DC component
        let mean = window_samples.iter().sum::<f32>() / self.window_size as f32;

        // Apply Hann window and prepare FFT input
        for (i, &sample) in window_samples.iter().enumerate() {
            let windowed_sample = (sample - mean) * self.window[i];
            self.fft_input[i] = Complex32::new(windowed_sample, 0.0);
        }

        // Perform FFT
        self.fft.process(&mut self.fft_input);

        // Compute power spectrum and bucket levels
        let mut buckets = [0.0_f32; BUCKET_COUNT];

        for (bucket_idx, &(start_bin, end_bin)) in self.bucket_ranges.iter().enumerate() {
            if start_bin >= end_bin || end_bin > self.fft_input.len() / 2 {
                continue;
            }

            let mut power_sum = 0.0;
            for bin_idx in start_bin..end_bin {
                let magnitude = self.fft_input[bin_idx].norm();
                power_sum += magnitude * magnitude;
            }

            let avg_power = power_sum / (end_bin - start_bin) as f32;

            let db = if avg_power > 1e-12 {
                20.0 * (avg_power.sqrt() / self.window_size as f32).log10()
            } else {
                -80.0
            };

            // Adapt noise floor when signal is quiet
            if db < self.noise_floor[bucket_idx] + 10.0 {
                self.noise_floor[bucket_idx] =
                    NOISE_ALPHA * db + (1.0 - NOISE_ALPHA) * self.noise_floor[bucket_idx];
            }

            // Map dB range to 0–1 with gain and curve shaping
            let normalized = ((db - DB_MIN) / (DB_MAX - DB_MIN)).clamp(0.0, 1.0);
            buckets[bucket_idx] = (normalized * GAIN).powf(CURVE_POWER).clamp(0.0, 1.0);
        }

        // Neighbor smoothing to reduce jitter
        let mut smoothed = buckets;
        for i in 1..BUCKET_COUNT - 1 {
            smoothed[i] = buckets[i] * 0.7 + buckets[i - 1] * 0.15 + buckets[i + 1] * 0.15;
        }

        // Clear processed samples
        self.buffer.clear();

        Some(smoothed)
    }

    /// Reset internal state (call when starting a new recording session).
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.noise_floor.fill(-40.0);
    }
}

// ---------------------------------------------------------------------------
// Bar animation (unchanged from original — smooth easing between frames)
// ---------------------------------------------------------------------------

const MIN_BAR: f32 = 0.08;
const MAX_BAR: f32 = 1.0;
const ATTACK_HZ: f32 = 26.0;
const DECAY_HZ: f32 = 7.0;
const BAR_RESPONSE: [f32; BUCKET_COUNT] = [0.88, 0.92, 0.97, 1.02, 1.08, 1.02, 0.97, 0.92, 0.88];

pub fn animate_bars(
    current: [f32; BUCKET_COUNT],
    target: [f32; BUCKET_COUNT],
    dt: Duration,
) -> [f32; BUCKET_COUNT] {
    let dt_secs = dt.as_secs_f32().clamp(0.0, 0.050);
    let mut next = current;
    for index in 0..next.len() {
        next[index] = animate_value(current[index], target[index], dt_secs, BAR_RESPONSE[index]);
    }
    next
}

fn animate_value(current: f32, target: f32, dt_secs: f32, response_scale: f32) -> f32 {
    let rate = if target > current {
        ATTACK_HZ
    } else {
        DECAY_HZ
    } * response_scale;
    let alpha = 1.0 - (-rate * dt_secs).exp();
    (current + (target - current) * alpha).clamp(MIN_BAR, MAX_BAR)
}

/// Return silent bars (minimum height for all positions).
pub fn silent_bars() -> [f32; BUCKET_COUNT] {
    [MIN_BAR; BUCKET_COUNT]
}
