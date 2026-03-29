use crate::dictation::types::DictationLevel;

const MIN_BAR: f32 = 0.08;
const MAX_BAR: f32 = 1.0;
const BAR_WEIGHTS: [f32; 9] = [0.28, 0.42, 0.6, 0.82, 1.0, 0.82, 0.6, 0.42, 0.28];

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
    let energy = (level.rms.mul_add(0.7, level.peak * 0.3))
        .clamp(0.0, 1.0)
        .powf(0.85);

    BAR_WEIGHTS
        .map(|weight| (MIN_BAR + energy * weight * (MAX_BAR - MIN_BAR)).clamp(MIN_BAR, MAX_BAR))
}
