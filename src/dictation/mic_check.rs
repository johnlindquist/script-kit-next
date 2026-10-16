//! A short microphone test for Dictation Setup.
//!
//! Permission says Script Kit *may* use the microphone; it doesn't say sound
//! comes through. This records a few seconds from the selected microphone and
//! reports how loud the loudest moment was, so a muted device, a wrong input
//! or a closed laptop lid shows up in setup instead of as an empty transcript.

use crate::dictation::types::{DictationCaptureConfig, DictationCaptureEvent, DictationDeviceId};
use parking_lot::Mutex;
use serde::Serialize;
use std::time::{Duration, Instant};

/// How long the test records.
pub const MICROPHONE_CHECK_DURATION: Duration = Duration::from_secs(3);

/// Loudest moment below this: nothing came through.
const SILENT_BELOW_DBFS: f32 = -55.0;
/// Loudest moment below this: something came through, but too quiet for
/// reliable transcription.
const QUIET_BELOW_DBFS: f32 = -35.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MicrophoneLevel {
    Silent,
    Quiet,
    Good,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrophoneCheck {
    pub level: MicrophoneLevel,
    /// Loudest sample, in dB relative to full scale (0 is the maximum).
    pub peak_dbfs: f32,
    /// Average loudness over the whole test.
    pub rms_dbfs: f32,
    pub seconds: f32,
}

impl MicrophoneCheck {
    /// The row title and description shown in Dictation Setup.
    pub fn summary(&self) -> (String, String) {
        match self.level {
            MicrophoneLevel::Good => (
                "Microphone test: sound comes through".to_string(),
                format!("Loudest moment {:.0} dB \u{00b7} run it again any time", self.peak_dbfs),
            ),
            MicrophoneLevel::Quiet => (
                "Microphone test: very quiet".to_string(),
                "Move closer, raise the input volume in System Settings \u{2192} Sound, or pick another microphone".to_string(),
            ),
            MicrophoneLevel::Silent => (
                "Microphone test: no sound came through".to_string(),
                "Speak during the test. If it stays silent, check the input device and its volume in System Settings \u{2192} Sound".to_string(),
            ),
        }
    }
}

fn dbfs(amplitude: f32) -> f32 {
    20.0 * amplitude.max(1e-9).log10()
}

/// Judge a recording by its loudest moment.
pub fn assess_samples(samples: &[f32], sample_rate_hz: u32) -> MicrophoneCheck {
    let peak = samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let rms = if samples.is_empty() {
        0.0
    } else {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    };
    let peak_dbfs = dbfs(peak);
    let level = if peak_dbfs < SILENT_BELOW_DBFS {
        MicrophoneLevel::Silent
    } else if peak_dbfs < QUIET_BELOW_DBFS {
        MicrophoneLevel::Quiet
    } else {
        MicrophoneLevel::Good
    };
    MicrophoneCheck {
        level,
        peak_dbfs,
        rms_dbfs: dbfs(rms),
        seconds: samples.len() as f32 / sample_rate_hz.max(1) as f32,
    }
}

fn last_check() -> &'static Mutex<Option<MicrophoneCheck>> {
    static LAST: std::sync::OnceLock<Mutex<Option<MicrophoneCheck>>> = std::sync::OnceLock::new();
    LAST.get_or_init(|| Mutex::new(None))
}

/// The most recent test result in this session.
pub fn last_microphone_check() -> Option<MicrophoneCheck> {
    last_check().lock().clone()
}

/// Record from `device_id` (default: the system default input) for
/// `duration` and judge the result. Blocks; run it off the UI thread.
pub fn run_microphone_check(
    device_id: Option<&DictationDeviceId>,
    duration: Duration,
) -> anyhow::Result<MicrophoneCheck> {
    if crate::dictation::is_dictation_recording() {
        anyhow::bail!("Dictation is recording; try the microphone test after it stops");
    }
    let config = DictationCaptureConfig::default();
    let sample_rate_hz = config.sample_rate_hz;
    let (events, handle) = crate::dictation::start_capture(config, device_id)?;
    let deadline = Instant::now() + duration;
    let mut samples = Vec::new();
    while Instant::now() < deadline {
        match events.try_recv() {
            Ok(DictationCaptureEvent::Chunk(chunk)) => samples.extend_from_slice(&chunk.samples),
            Ok(DictationCaptureEvent::Bars(_)) => {}
            Ok(DictationCaptureEvent::EndOfStream) => break,
            Err(async_channel::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(async_channel::TryRecvError::Closed) => break,
        }
    }
    drop(handle);
    while let Ok(event) = events.try_recv() {
        if let DictationCaptureEvent::Chunk(chunk) = event {
            samples.extend_from_slice(&chunk.samples);
        }
    }
    let check = assess_samples(&samples, sample_rate_hz);
    tracing::info!(
        category = "DICTATION",
        event = "microphone_check",
        level = ?check.level,
        peak_dbfs = check.peak_dbfs,
        rms_dbfs = check.rms_dbfs,
        seconds = check.seconds,
        "Microphone test finished"
    );
    *last_check().lock() = Some(check.clone());
    Ok(check)
}

/// Whether a laptop's lid is closed (`None` on desktops or when unknown).
/// With the lid closed, the built-in microphone hears nothing.
#[cfg(target_os = "macos")]
pub fn laptop_lid_closed() -> Option<bool> {
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::string::CFString;
    use std::ffi::c_void;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOServiceMatching(name: *const std::os::raw::c_char) -> *mut c_void;
        fn IOServiceGetMatchingService(main_port: u32, matching: *mut c_void) -> u32;
        fn IORegistryEntryCreateCFProperty(
            entry: u32,
            key: core_foundation::string::CFStringRef,
            allocator: *const c_void,
            options: u32,
        ) -> core_foundation::base::CFTypeRef;
        fn IOObjectRelease(object: u32) -> i32;
    }

    // SAFETY: IOServiceGetMatchingService consumes the matching dictionary;
    // the service and the copied property are released below.
    unsafe {
        let matching = IOServiceMatching(c"IOPMrootDomain".as_ptr());
        if matching.is_null() {
            return None;
        }
        // 0 is MACH_PORT_NULL: the default main port.
        let service = IOServiceGetMatchingService(0, matching);
        if service == 0 {
            return None;
        }
        let key = CFString::from_static_string("AppleClamshellState");
        let value = IORegistryEntryCreateCFProperty(
            service,
            key.as_concrete_TypeRef(),
            std::ptr::null(),
            0,
        );
        IOObjectRelease(service);
        if value.is_null() {
            return None;
        }
        let value = CFType::wrap_under_create_rule(value);
        value.downcast::<CFBoolean>().map(bool::from)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn laptop_lid_closed() -> Option<bool> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(amplitude: f32) -> Vec<f32> {
        (0..16_000)
            .map(|i| amplitude * (i as f32 * 0.05).sin())
            .collect()
    }

    #[test]
    fn digital_silence_is_silent() {
        let check = assess_samples(&vec![0.0; 16_000], 16_000);
        assert_eq!(check.level, MicrophoneLevel::Silent);
        assert!((check.seconds - 1.0).abs() < 1e-3);
    }

    #[test]
    fn faint_hiss_is_silent_and_soft_speech_is_quiet() {
        assert_eq!(
            assess_samples(&tone(0.001), 16_000).level,
            MicrophoneLevel::Silent
        );
        assert_eq!(
            assess_samples(&tone(0.01), 16_000).level,
            MicrophoneLevel::Quiet
        );
    }

    #[test]
    fn normal_speech_level_is_good() {
        let check = assess_samples(&tone(0.3), 16_000);
        assert_eq!(check.level, MicrophoneLevel::Good);
        assert!(
            check.peak_dbfs > -11.0 && check.peak_dbfs < -10.0,
            "{}",
            check.peak_dbfs
        );
    }

    #[test]
    fn an_empty_recording_is_silent_not_a_panic() {
        assert_eq!(assess_samples(&[], 16_000).level, MicrophoneLevel::Silent);
    }

    #[test]
    fn summaries_say_what_to_do() {
        let silent = assess_samples(&[], 16_000).summary();
        assert!(silent.1.contains("System Settings"));
        let good = assess_samples(&tone(0.3), 16_000).summary();
        assert!(good.0.contains("comes through"));
    }
}
