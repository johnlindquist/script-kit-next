use crate::ai::context_contract::ContextAttachmentKind;

pub(crate) struct SpineLivePreview {
    pub frontmost_app_name: Option<String>,
    pub active_window_title: Option<String>,
    pub browser_url: Option<String>,
    pub selection_text: Option<String>,
    pub clipboard_text: Option<String>,
}

impl SpineLivePreview {
    pub(crate) fn subtitle_for_context_kind(&self, kind: ContextAttachmentKind) -> Option<String> {
        match kind {
            ContextAttachmentKind::Selection => self.selection_text.as_ref().map(|t| {
                let preview = truncate_preview(t, 60);
                if preview.is_empty() {
                    "No text selected".to_string()
                } else {
                    format!("\u{201c}{preview}\u{201d}")
                }
            }),
            ContextAttachmentKind::Browser => self
                .browser_url
                .as_ref()
                .map(|url| truncate_preview(url, 60)),
            ContextAttachmentKind::Window => {
                match (&self.active_window_title, &self.frontmost_app_name) {
                    (Some(title), _) if !title.trim().is_empty() => {
                        Some(truncate_preview(title, 60))
                    }
                    (_, Some(app)) if !app.trim().is_empty() => Some(app.clone()),
                    _ => None,
                }
            }
            ContextAttachmentKind::FrontmostApp => {
                self.frontmost_app_name.as_ref().map(|name| name.clone())
            }
            ContextAttachmentKind::Clipboard => self.clipboard_text.as_ref().map(|t| {
                let preview = truncate_preview(t, 60);
                if preview.is_empty() {
                    "Clipboard empty".to_string()
                } else {
                    format!("\u{201c}{preview}\u{201d}")
                }
            }),
            _ => None,
        }
    }

    pub(crate) fn style_selection_preview(&self) -> Option<String> {
        self.selection_text.as_ref().and_then(|t| {
            let preview = truncate_preview(t, 80);
            if preview.is_empty() {
                None
            } else {
                Some(format!("Will rewrite: \u{201c}{preview}\u{201d}"))
            }
        })
    }
}

fn truncate_preview(input: &str, max_chars: usize) -> String {
    let single_line = input
        .chars()
        .map(|ch| {
            if matches!(ch, '\n' | '\r' | '\t') {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();
    let trimmed = single_line.trim();
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let mut out: String = trimmed.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('\u{2026}');
        out
    }
}
