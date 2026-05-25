use super::clipboard::write_plain_text_to_pasteboard;
use super::focused_text::{FocusedTextError, FocusedTextSessionId};
use super::metrics::TextMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextMutationOptions {
    pub allow_stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMutationResult {
    pub action: TextMutationAction,
    pub changed_text: bool,
    pub copied_to_clipboard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMutationAction {
    Replace,
    Append,
    Copy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedTextMutationSession {
    pub session_id: FocusedTextSessionId,
    pub captured_at_ms: u128,
    pub current_text: Option<String>,
    pub ttl_ms: u128,
}

impl FocusedTextMutationSession {
    pub fn is_stale_at(&self, now_ms: u128) -> bool {
        now_ms.saturating_sub(self.captured_at_ms) > self.ttl_ms
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppendMutationPlan {
    DirectSet {
        text: String,
        metrics: TextMetrics,
    },
    PasteOutputAtEnd {
        output: String,
        output_metrics: TextMetrics,
    },
    SelectAllAndPaste {
        text: String,
        metrics: TextMetrics,
    },
}

pub fn validate_mutation_session(
    session: &FocusedTextMutationSession,
    options: TextMutationOptions,
    now_ms: u128,
) -> Result<(), FocusedTextError> {
    if session.is_stale_at(now_ms) && !options.allow_stale {
        return Err(FocusedTextError::StaleSession);
    }
    Ok(())
}

pub fn plan_append_mutation(
    current_text: Option<&str>,
    output: &str,
    can_set_value_directly: bool,
    can_set_caret_to_end: bool,
) -> AppendMutationPlan {
    match (current_text, can_set_value_directly, can_set_caret_to_end) {
        (Some(current), true, _) => {
            let text = format!("{current}{output}");
            AppendMutationPlan::DirectSet {
                metrics: TextMetrics::from_text(&text),
                text,
            }
        }
        (None, true, true) => AppendMutationPlan::PasteOutputAtEnd {
            output: output.to_string(),
            output_metrics: TextMetrics::from_text(output),
        },
        (None, true, false) => AppendMutationPlan::SelectAllAndPaste {
            text: output.to_string(),
            metrics: TextMetrics::from_text(output),
        },
        (_, false, true) => AppendMutationPlan::PasteOutputAtEnd {
            output: output.to_string(),
            output_metrics: TextMetrics::from_text(output),
        },
        (Some(current), false, false) => {
            let text = format!("{current}{output}");
            AppendMutationPlan::SelectAllAndPaste {
                metrics: TextMetrics::from_text(&text),
                text,
            }
        }
        (None, false, false) => AppendMutationPlan::SelectAllAndPaste {
            text: output.to_string(),
            metrics: TextMetrics::from_text(output),
        },
    }
}

pub fn replace_focused_text(
    session_id: FocusedTextSessionId,
    text: &str,
    options: TextMutationOptions,
) -> Result<TextMutationResult, FocusedTextError> {
    #[cfg(target_os = "macos")]
    {
        return super::ax::replace_registered_focused_text(
            &session_id,
            text,
            options,
            current_time_ms(),
        );
    }

    #[cfg(not(target_os = "macos"))]
    Err(FocusedTextError::UnsupportedTarget)
}

pub fn append_focused_text(
    session_id: FocusedTextSessionId,
    text: &str,
    options: TextMutationOptions,
) -> Result<TextMutationResult, FocusedTextError> {
    #[cfg(target_os = "macos")]
    {
        return super::ax::append_registered_focused_text(
            &session_id,
            text,
            options,
            current_time_ms(),
        );
    }

    #[cfg(not(target_os = "macos"))]
    Err(FocusedTextError::UnsupportedTarget)
}

pub fn copy_text_output(text: &str) -> Result<TextMutationResult, FocusedTextError> {
    write_plain_text_to_pasteboard(text)
        .map_err(|err| FocusedTextError::Platform(err.to_string()))?;
    Ok(TextMutationResult {
        action: TextMutationAction::Copy,
        changed_text: false,
        copied_to_clipboard: true,
    })
}

fn current_time_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
