use crate::dictation::types::DictationLevel;
use std::time::Duration;

const MIN_BAR: f32 = 0.08;
const MAX_BAR: f32 = 1.0;
const LEVEL_GATE: f32 = 0.02;
const ATTACK_HZ: f32 = 26.0;
const DECAY_HZ: f32 = 7.0;
const BAR_WEIGHTS: [f32; 9] = [0.28, 0.44, 0.62, 0.82, 1.0, 0.82, 0.62, 0.44, 0.28];
const BAR_RESPONSE: [f32; 9] = [0.88, 0.92, 0.97, 1.02, 1.08, 1.02, 0.97, 0.92, 0.88];

fn gate(level: f32) -> f32 {
    ((level.clamp(0.0, 1.0) - LEVEL_GATE) / (1.0 - LEVEL_GATE)).clamp(0.0, 1.0)
}

pub fn compute_level(samples: &[f32]) -> DictationLevel {
    if samples.is_empty() {
        return DictationLevel {
            rms: 0.0,
            peak: 0.0,
        };
    }

    let mut peak = 0.0_f32;
    let mut sum_squares = 0.0_f32;

    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let magnitude = clamped.abs();
        peak = peak.max(magnitude);
        sum_squares += clamped * clamped;
    }

    let rms = (sum_squares / samples.len() as f32).sqrt().clamp(0.0, 1.0);

    DictationLevel {
        rms,
        peak: peak.clamp(0.0, 1.0),
    }
}

pub fn bars_for_level(level: DictationLevel) -> [f32; 9] {
    let rms = gate(level.rms);
    let peak = gate(level.peak);

    BAR_WEIGHTS.map(|weight| {
        let sustained = rms.powf(0.88) * weight * 0.78;
        let transient = peak.powf(0.72) * weight.powf(1.35) * 0.32;
        let shaped = (sustained + transient).clamp(0.0, 1.0);
        (MIN_BAR + shaped * (MAX_BAR - MIN_BAR)).clamp(MIN_BAR, MAX_BAR)
    })
}

pub fn animate_bars(current: [f32; 9], target: [f32; 9], dt: Duration) -> [f32; 9] {
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
