use crate::dictation::DictationTarget;

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
        "tabaiharness" | "acp" | "acpchat" | "ai" => Some(DictationTarget::TabAiHarness),
        "externalapp" | "frontmostapp" | "frontmost" | "app" => Some(DictationTarget::ExternalApp),
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
