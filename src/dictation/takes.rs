//! Recent recordings ("takes") kept so a failed dictation can be retried,
//! and the reason a recording produced no words.
//!
//! Each finished recording is written as a 16-bit mono WAV file under
//! `~/.scriptkit/dictation/takes/` (a private folder) before it is
//! transcribed, and only the newest [`KEEP_TAKES`] are kept. Turn it off with
//! `dictation.keepRecordings: false` (or `dictation.saveHistory: false`,
//! which keeps nothing about a dictation).

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use super::types::CapturedAudioChunk;

/// How many recent recordings are kept.
pub const KEEP_TAKES: usize = 25;

/// Why a recording produced no words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoSpeechReason {
    /// Shorter than the minimum the engine needs.
    TooShort,
    /// Quieter than the silence threshold.
    TooQuiet,
    /// Loud enough, but the engine recognized no words.
    NothingRecognized,
}

impl NoSpeechReason {
    /// What the user sees.
    pub fn message(self) -> &'static str {
        match self {
            Self::TooShort => {
                "No speech heard: the recording was too short. Hold the hotkey a little longer"
            }
            Self::TooQuiet => {
                "No speech heard: the microphone was too quiet. Test it in Dictation Setup"
            }
            Self::NothingRecognized => {
                "No words recognized. Try again, a little closer to the microphone"
            }
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::TooShort => "tooShort",
            Self::TooQuiet => "tooQuiet",
            Self::NothingRecognized => "nothingRecognized",
        }
    }
}

static LAST_NO_SPEECH: Mutex<Option<NoSpeechReason>> = Mutex::new(None);

/// Records why the latest transcription produced nothing (`None` clears it).
pub fn set_last_no_speech_reason(reason: Option<NoSpeechReason>) {
    *LAST_NO_SPEECH.lock().unwrap_or_else(|e| e.into_inner()) = reason;
}

/// Why the latest transcription produced nothing.
pub fn last_no_speech_reason() -> Option<NoSpeechReason> {
    *LAST_NO_SPEECH.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether recordings are kept (both history and `keepRecordings` allow it).
pub fn keep_recordings_enabled(prefs: &crate::config::DictationPreferences) -> bool {
    prefs.save_history_enabled() && prefs.keep_recordings.unwrap_or(true)
}

pub fn takes_dir() -> PathBuf {
    crate::setup::get_kit_path().join("dictation/takes")
}

/// A kept recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Take {
    pub path: PathBuf,
    pub duration: Duration,
    pub recorded_at: std::time::SystemTime,
}

/// 16-bit PCM mono WAV bytes for `samples` (clamped to −1…1).
pub fn encode_wav(samples: &[f32], sample_rate_hz: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_len as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate_hz * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Reads a WAV written by [`encode_wav`] back into one audio chunk.
pub fn decode_wav(bytes: &[u8]) -> anyhow::Result<CapturedAudioChunk> {
    anyhow::ensure!(
        bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE",
        "not a WAV file"
    );
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate_hz = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    anyhow::ensure!(
        channels == 1 && bits == 16 && sample_rate_hz > 0,
        "unsupported WAV format"
    );
    let samples: Vec<f32> = bytes[44..]
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / i16::MAX as f32)
        .collect();
    let duration = Duration::from_secs_f64(samples.len() as f64 / sample_rate_hz as f64);
    Ok(CapturedAudioChunk {
        sample_rate_hz,
        samples,
        duration,
    })
}

fn take_name(now: chrono::DateTime<chrono::Local>) -> String {
    format!("take-{}.wav", now.format("%Y%m%d-%H%M%S%.3f"))
}

/// Saves a finished recording and prunes old ones. Returns the file, or
/// `None` when keeping recordings is off or the recording is empty.
pub fn save_take(chunks: &[CapturedAudioChunk]) -> Option<PathBuf> {
    let prefs = crate::config::load_user_preferences();
    if !keep_recordings_enabled(&prefs.dictation) {
        return None;
    }
    save_take_in(&takes_dir(), chunks, KEEP_TAKES)
}

/// [`save_take`] into an explicit folder (tests).
pub fn save_take_in(dir: &Path, chunks: &[CapturedAudioChunk], keep: usize) -> Option<PathBuf> {
    let sample_rate_hz = chunks.first()?.sample_rate_hz;
    let samples = super::transcription::merge_captured_chunks(chunks);
    if samples.is_empty() {
        return None;
    }
    if let Err(error) = crate::atomic_file::ensure_private_directory(dir) {
        tracing::warn!(category = "DICTATION", %error, "dictation_take_dir_unavailable");
        return None;
    }
    let path = dir.join(take_name(chrono::Local::now()));
    if let Err(error) =
        crate::atomic_file::write_private_atomic(&path, &encode_wav(&samples, sample_rate_hz))
    {
        tracing::warn!(category = "DICTATION", %error, "dictation_take_not_saved");
        return None;
    }
    prune_takes(dir, keep);
    Some(path)
}

fn take_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "wav")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("take-"))
        })
        .collect();
    // Names sort by time.
    files.sort();
    files
}

fn prune_takes(dir: &Path, keep: usize) {
    let files = take_files(dir);
    let excess = files.len().saturating_sub(keep);
    for old in files.into_iter().take(excess) {
        let _ = std::fs::remove_file(old);
    }
}

/// The newest kept recording in `dir`.
pub fn latest_take_in(dir: &Path) -> Option<Take> {
    let path = take_files(dir).pop()?;
    let metadata = std::fs::metadata(&path).ok()?;
    let data_bytes = metadata.len().saturating_sub(44);
    let sample_rate = std::fs::read(&path)
        .ok()
        .filter(|bytes| bytes.len() >= 28)
        .map(|bytes| u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]))
        .filter(|rate| *rate > 0)
        .unwrap_or(16_000);
    Some(Take {
        duration: Duration::from_secs_f64(data_bytes as f64 / 2.0 / sample_rate as f64),
        recorded_at: metadata.modified().ok()?,
        path,
    })
}

/// The newest kept recording.
pub fn latest_take() -> Option<Take> {
    latest_take_in(&takes_dir())
}

/// Loads a kept recording for transcription.
pub fn load_take(path: &Path) -> anyhow::Result<Vec<CapturedAudioChunk>> {
    let bytes = std::fs::read(path)?;
    Ok(vec![decode_wav(&bytes)?])
}

/// How many recordings are kept now.
pub fn take_count() -> usize {
    take_files(&takes_dir()).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(samples: Vec<f32>) -> CapturedAudioChunk {
        CapturedAudioChunk {
            sample_rate_hz: 16_000,
            duration: Duration::from_secs_f64(samples.len() as f64 / 16_000.0),
            samples,
        }
    }

    #[test]
    fn wav_round_trips_within_16_bit_precision() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0, 2.0];
        let decoded = decode_wav(&encode_wav(&samples, 16_000)).expect("decode");
        assert_eq!(decoded.sample_rate_hz, 16_000);
        assert_eq!(decoded.samples.len(), samples.len());
        for (a, b) in decoded.samples.iter().zip([0.0, 0.5, -0.5, 1.0, -1.0, 1.0]) {
            assert!((a - b).abs() < 1e-3, "{a} vs {b}");
        }
        assert!(decode_wav(b"not audio").is_err());
    }

    #[test]
    fn only_the_newest_takes_are_kept() {
        let dir = tempfile::tempdir().expect("tempdir");
        for _ in 0..4 {
            save_take_in(dir.path(), &[chunk(vec![0.1; 1600])], 3).expect("saved");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(take_files(dir.path()).len(), 3);
        let latest = latest_take_in(dir.path()).expect("latest");
        assert!((latest.duration.as_secs_f64() - 0.1).abs() < 0.01);
        let loaded = load_take(&latest.path).expect("load");
        assert_eq!(loaded[0].samples.len(), 1600);
        assert!(save_take_in(dir.path(), &[], 3).is_none());
    }

    #[test]
    fn no_speech_reasons_say_what_to_do() {
        assert!(NoSpeechReason::TooQuiet
            .message()
            .contains("Dictation Setup"));
        assert!(NoSpeechReason::TooShort.message().contains("too short"));
        set_last_no_speech_reason(Some(NoSpeechReason::NothingRecognized));
        assert_eq!(
            last_no_speech_reason(),
            Some(NoSpeechReason::NothingRecognized)
        );
        set_last_no_speech_reason(None);
    }

    #[test]
    fn keeping_recordings_follows_history_and_the_switch() {
        let mut prefs = crate::config::DictationPreferences::default();
        assert!(keep_recordings_enabled(&prefs));
        prefs.keep_recordings = Some(false);
        assert!(!keep_recordings_enabled(&prefs));
        prefs.keep_recordings = None;
        prefs.save_history = Some(false);
        assert!(!keep_recordings_enabled(&prefs));
    }
}
