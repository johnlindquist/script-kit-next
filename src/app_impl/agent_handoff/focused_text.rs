//! Text from the front app for the model's chat: the focused field or the selection, read while
//! that app still has focus, staged in the line above the input.

use super::*;
use crate::ai::message_parts::AiContextPart;
use crate::one_window::chat_entry::{ChatEntry, SELECTION_SOURCE};

/// The captured text as the part the chat takes along: "Selection — Safari", "Draft — Mail".
fn captured_part(captured: crate::selected_text::CapturedText) -> AiContextPart {
    let what = match captured.kind {
        crate::selected_text::CapturedTextKind::Selection => "Selection",
        crate::selected_text::CapturedTextKind::FocusedField => "Draft",
    };
    AiContextPart::TextBlock {
        label: match &captured.source_app {
            Some(app) => format!("{what} \u{2014} {app}"),
            None => what.to_string(),
        },
        source: SELECTION_SOURCE.to_string(),
        text: captured.text,
        mime_type: None,
    }
}

/// Replaces the lazy `@selection` resource with the text captured now, while the source app
/// still owns focus: a fresh read first, then what the Spine preview read seconds earlier. With
/// neither, the lazy part stays so the missing selection is reported instead of dropped.
pub(super) fn materialize_selection_context_parts(
    parts: Vec<AiContextPart>,
    cached_selection: Option<crate::selected_text::CapturedText>,
    capture_live: impl FnOnce() -> Option<crate::selected_text::CapturedText>,
) -> Vec<AiContextPart> {
    let selection_uri = crate::ai::context_contract::ContextAttachmentKind::Selection
        .spec()
        .uri;
    let needs_selection = parts
        .iter()
        .any(|part| matches!(part, AiContextPart::ResourceUri { uri, .. } if uri == selection_uri));
    if !needs_selection {
        return parts;
    }
    let Some(captured) = capture_live()
        .filter(|captured| !captured.text.trim().is_empty())
        .or_else(|| cached_selection.filter(|captured| !captured.text.trim().is_empty()))
    else {
        tracing::warn!(
            target: "script_kit::spine",
            event = "spine_selection_capture_empty_at_handoff",
        );
        return parts;
    };
    let selected = captured_part(captured);
    parts
        .into_iter()
        .map(|part| match part {
            AiContextPart::ResourceUri { uri, .. } if uri == selection_uri => selected.clone(),
            other => other,
        })
        .collect()
}

impl ScriptListApp {
    /// The focused field's text, or the selection when the app hides its fields from
    /// accessibility (Chrome, Electron).
    pub(crate) fn capture_focused_text_part(&self, source: &'static str) -> Option<AiContextPart> {
        match crate::platform::accessibility::capture_focused_text_field(
            crate::platform::accessibility::CaptureFocusedTextOptions::default(),
        ) {
            Ok(snapshot) if !snapshot.text.trim().is_empty() => {
                tracing::info!(
                    target: "script_kit::focused_text",
                    event = "focused_text_captured",
                    source,
                    chars = snapshot.metrics.chars,
                );
                return Some(captured_part(crate::selected_text::CapturedText {
                    text: snapshot.text,
                    kind: crate::selected_text::CapturedTextKind::FocusedField,
                    source_app: Some(snapshot.app.name),
                }));
            }
            Ok(_) => {}
            Err(error) => tracing::info!(
                target: "script_kit::focused_text",
                event = "focused_text_ax_capture_failed",
                source,
                error = %error,
            ),
        }
        crate::selected_text::capture_selection_or_focused_text()
            .ok()
            .flatten()
            .filter(|captured| !captured.text.trim().is_empty())
            .map(captured_part)
    }

    /// The focused text goes to the model's chat: staged, or sent at once with `instruction`
    /// (the rewrite hotkey, a Spine style). With nothing to read, the window says so instead.
    pub(crate) fn open_focused_text_in_chat(
        &mut self,
        source: &'static str,
        instruction: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(part) = self.capture_focused_text_part(source) else {
            self.one_window_notice(
                crate::line_notice::FAILED,
                "Nothing to rewrite — select text or focus a text field",
                cx,
            );
            return;
        };
        let entry = match instruction {
            Some(instruction) => ChatEntry::sending(instruction),
            None => ChatEntry::quick(),
        };
        self.one_window_open_chat(entry.with(part), cx);
    }
}
