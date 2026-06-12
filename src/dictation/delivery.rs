use crate::brain::substrate::{BrainSubstrate, DayEntry};
use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};

use crate::dictation::DictationTarget;

/// Result of appending a dictated transcript onto today's day page file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayPageTranscriptDelivery {
    pub formatted_line: String,
    pub updated_content: String,
    pub caret_offset: usize,
}

/// Append a timestamped capture line to today's day page and return the
/// post-append editor snapshot with a caret offset at the end of the file.
pub fn deliver_transcript_to_day_page(
    substrate: &BrainSubstrate,
    now: DateTime<Utc>,
    transcript: &str,
) -> Result<DayPageTranscriptDelivery> {
    let text = transcript.trim();
    if text.is_empty() {
        anyhow::bail!("empty dictation transcript");
    }

    substrate
        .append_to_day(
            now,
            DayEntry::Capture {
                text: text.to_string(),
            },
        )
        .context("failed to append dictation capture to day page")?;

    let date = now.with_timezone(&substrate.timezone()).date_naive();
    let path = substrate.paths().day_page(date);
    let updated_content = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "reading day page after dictation append: {}",
            path.display()
        )
    })?;
    let caret_offset = updated_content.len();
    let formatted_line = updated_content
        .lines()
        .last()
        .context("day page empty after dictation append")?
        .to_string();

    Ok(DayPageTranscriptDelivery {
        formatted_line,
        updated_content,
        caret_offset,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationDeliveryTargetResolution {
    Deliver {
        target: DictationTarget,
        source: DictationDeliveryTargetSource,
    },
    Refuse(DictationWrongTargetRefusalDraft),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationDeliveryTargetSource {
    ExplicitLabel,
    ActiveSession,
    UiFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationWrongTargetReason {
    UnknownTargetLabel,
    TargetUnavailable,
    TargetStale,
}

impl DictationWrongTargetReason {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::UnknownTargetLabel => "unknownTargetLabel",
            Self::TargetUnavailable => "targetUnavailable",
            Self::TargetStale => "targetStale",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationWrongTargetRefusalDraft {
    pub reason: DictationWrongTargetReason,
    pub requested_target_label: Option<String>,
    pub requested_target: Option<DictationTarget>,
    pub fallback_target: Option<DictationTarget>,
    pub delivery_generation_before: u64,
}

pub fn parse_dictation_target_label(label: &str) -> Option<DictationTarget> {
    let normalized = label
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>();

    match normalized.as_str() {
        "mainwindowfilter" | "scriptkit" | "launcher" | "filter" => {
            Some(DictationTarget::MainWindowFilter)
        }
        "mainwindowprompt" | "prompt" => Some(DictationTarget::MainWindowPrompt),
        "noteseditor" | "notes" => Some(DictationTarget::NotesEditor),
        "aichatcomposer" | "aichat" | "legacyai" => Some(DictationTarget::AiChatComposer),
        "tabaiharness" | "agentchat" | "agentchatchat" | "ai" => {
            Some(DictationTarget::TabAiHarness)
        }
        "externalapp" | "frontmostapp" | "frontmost" | "app" => Some(DictationTarget::ExternalApp),
        "daypage" | "day" | "today" => Some(DictationTarget::DayPage),
        _ => None,
    }
}

pub fn resolve_delivery_target_request(
    explicit_label: Option<&str>,
    active_session_target: Option<DictationTarget>,
    ui_fallback_target: DictationTarget,
    delivery_generation_before: u64,
) -> DictationDeliveryTargetResolution {
    if let Some(label) = explicit_label {
        return match parse_dictation_target_label(label) {
            Some(target) => DictationDeliveryTargetResolution::Deliver {
                target,
                source: DictationDeliveryTargetSource::ExplicitLabel,
            },
            None => DictationDeliveryTargetResolution::Refuse(DictationWrongTargetRefusalDraft {
                reason: DictationWrongTargetReason::UnknownTargetLabel,
                requested_target_label: Some(label.to_string()),
                requested_target: None,
                fallback_target: Some(active_session_target.unwrap_or(ui_fallback_target)),
                delivery_generation_before,
            }),
        };
    }

    if let Some(target) = active_session_target {
        return DictationDeliveryTargetResolution::Deliver {
            target,
            source: DictationDeliveryTargetSource::ActiveSession,
        };
    }

    DictationDeliveryTargetResolution::Deliver {
        target: ui_fallback_target,
        source: DictationDeliveryTargetSource::UiFallback,
    }
}

#[cfg(test)]
mod day_page_delivery_tests {
    use super::*;
    use crate::brain::substrate::BrainSubstrate;
    use chrono_tz::Tz;

    fn test_substrate() -> (tempfile::TempDir, BrainSubstrate) {
        let dir = tempfile::tempdir().expect("tempdir");
        let substrate = BrainSubstrate::with_timezone(dir.path().join("brain"), Tz::UTC);
        (dir, substrate)
    }

    fn utc(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("parse time")
            .with_timezone(&Utc)
    }

    #[test]
    fn deliver_transcript_appends_timestamped_capture_line() {
        let (_dir, substrate) = test_substrate();
        let now = utc("2026-06-11T14:30:00Z");

        let delivery =
            deliver_transcript_to_day_page(&substrate, now, "hold thought").expect("deliver");

        assert_eq!(delivery.formatted_line, "14:30 hold thought");
        assert!(delivery.updated_content.contains("14:30 hold thought"));
        assert_eq!(delivery.caret_offset, delivery.updated_content.len());
    }

    #[test]
    fn empty_transcript_is_rejected_without_day_page_write() {
        let (_dir, substrate) = test_substrate();
        let now = utc("2026-06-11T14:30:00Z");

        let error =
            deliver_transcript_to_day_page(&substrate, now, "   ").expect_err("empty transcript");
        assert!(error.to_string().contains("empty dictation transcript"));

        let path = substrate.paths().day_page(now.date_naive());
        assert!(!path.exists());
    }

    #[test]
    fn day_page_target_label_aliases_resolve() {
        assert_eq!(
            parse_dictation_target_label("dayPage"),
            Some(DictationTarget::DayPage)
        );
        assert_eq!(
            parse_dictation_target_label("today"),
            Some(DictationTarget::DayPage)
        );
    }
}
