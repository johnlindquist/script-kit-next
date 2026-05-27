use crate::ai::context_contract::ContextAttachmentKind;

#[derive(Debug, Clone, Default)]
pub(crate) struct SpineLivePreview {
    pub frontmost_app_name: Option<String>,
    pub active_window_title: Option<String>,
    pub browser_url: Option<String>,
    pub selection_text: Option<String>,
    pub clipboard_text: Option<String>,
    pub menu_bar_summary: Option<String>,
    pub script_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct SpineLivePreviewCache {
    pub current: SpineLivePreview,
    pub generation: u64,
    last_expensive_refresh: Option<std::time::Instant>,
}

impl Default for SpineLivePreviewCache {
    fn default() -> Self {
        Self {
            current: SpineLivePreview::default(),
            generation: 0,
            last_expensive_refresh: None,
        }
    }
}

impl SpineLivePreviewCache {
    pub(crate) fn refresh_cheap_fields(&mut self) {
        let tracked_app = crate::frontmost_app_tracker::get_last_real_app();

        let new_app = tracked_app.as_ref().map(|a| a.name.clone());
        let new_title = tracked_app.as_ref().and_then(|a| a.window_title.clone());

        let menu_snapshot = crate::frontmost_app_tracker::get_cached_menu_snapshot();
        let new_menu = match (menu_snapshot.app.as_ref(), menu_snapshot.items.len()) {
            (Some(app), 0) => Some(format!("{} \u{b7} menu loading\u{2026}", app.name)),
            (Some(app), n) => Some(format!("{} \u{b7} {n} menus cached", app.name)),
            (None, _) => Some("No tracked app".to_string()),
        };

        let new_clipboard = arboard::Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok())
            .filter(|t| !t.trim().is_empty());

        if new_app != self.current.frontmost_app_name
            || new_title != self.current.active_window_title
            || new_menu != self.current.menu_bar_summary
            || new_clipboard != self.current.clipboard_text
        {
            self.current.frontmost_app_name = new_app;
            self.current.active_window_title = new_title;
            self.current.menu_bar_summary = new_menu;
            self.current.clipboard_text = new_clipboard;
            self.generation += 1;
        }
    }

    pub(crate) fn refresh_expensive_fields(&mut self) {
        const THROTTLE: std::time::Duration = std::time::Duration::from_secs(2);
        if let Some(last) = self.last_expensive_refresh {
            if last.elapsed() < THROTTLE {
                return;
            }
        }
        self.last_expensive_refresh = Some(std::time::Instant::now());

        let new_url = crate::platform::get_any_browser_tab_url();
        let new_selection = crate::selected_text::get_selected_text()
            .ok()
            .filter(|t| !t.trim().is_empty());

        if new_url != self.current.browser_url || new_selection != self.current.selection_text {
            self.current.browser_url = new_url;
            self.current.selection_text = new_selection;
            self.generation += 1;
        }
    }

    pub(crate) fn set_script_count(&mut self, count: usize) {
        self.current.script_count = Some(count);
    }
}

impl SpineLivePreview {
    pub(crate) fn subtitle_for_context_kind(&self, kind: ContextAttachmentKind) -> Option<String> {
        match kind {
            ContextAttachmentKind::Current => {
                let mut parts = Vec::new();
                if let Some(app) = &self.frontmost_app_name {
                    parts.push(app.clone());
                }
                if self.active_window_title.is_some() {
                    parts.push("Window".to_string());
                }
                if self.browser_url.is_some() {
                    parts.push("Browser".to_string());
                }
                if parts.is_empty() {
                    Some("Snapshot of current app context".to_string())
                } else {
                    Some(parts.join(" \u{b7} "))
                }
            }
            ContextAttachmentKind::Full => {
                let mut parts = Vec::new();
                if self.selection_text.is_some() {
                    parts.push("Selection");
                }
                if self.clipboard_text.is_some() {
                    parts.push("Clipboard");
                }
                if self.browser_url.is_some() {
                    parts.push("Browser");
                }
                if self.active_window_title.is_some() {
                    parts.push("Window");
                }
                if parts.is_empty() {
                    Some("All available context except screenshots".to_string())
                } else {
                    Some(format!("{} included", parts.join(" \u{b7} ")))
                }
            }
            ContextAttachmentKind::Selection => {
                if let Some(t) = &self.selection_text {
                    let preview = truncate_preview(t, 60);
                    if preview.is_empty() {
                        Some("No text selected".to_string())
                    } else {
                        Some(format!("\u{201c}{preview}\u{201d}"))
                    }
                } else {
                    Some("No text selected".to_string())
                }
            }
            ContextAttachmentKind::Browser => {
                if let Some(url) = &self.browser_url {
                    Some(truncate_preview(url, 60))
                } else {
                    Some("No supported browser focused".to_string())
                }
            }
            ContextAttachmentKind::Window => {
                match (&self.active_window_title, &self.frontmost_app_name) {
                    (Some(title), Some(app)) if !title.trim().is_empty() => {
                        Some(format!("{} \u{b7} {}", truncate_preview(title, 45), app))
                    }
                    (Some(title), _) if !title.trim().is_empty() => {
                        Some(truncate_preview(title, 60))
                    }
                    (_, Some(app)) if !app.trim().is_empty() => Some(app.clone()),
                    _ => Some("No tracked window".to_string()),
                }
            }
            ContextAttachmentKind::Diagnostics => {
                Some("Capture source health and permissions".to_string())
            }
            ContextAttachmentKind::Screenshot => {
                if let Some(app) = &self.frontmost_app_name {
                    Some(format!("Ready to capture \u{b7} {app}"))
                } else {
                    Some("Ready to capture screen behind Script Kit".to_string())
                }
            }
            ContextAttachmentKind::Clipboard => {
                if let Some(t) = &self.clipboard_text {
                    let preview = truncate_preview(t, 60);
                    if preview.is_empty() {
                        Some("Clipboard empty".to_string())
                    } else {
                        Some(format!("\u{201c}{preview}\u{201d}"))
                    }
                } else {
                    Some("Clipboard empty".to_string())
                }
            }
            ContextAttachmentKind::FrontmostApp => {
                if let Some(name) = &self.frontmost_app_name {
                    Some(name.clone())
                } else {
                    Some("No tracked app".to_string())
                }
            }
            ContextAttachmentKind::MenuBar => self
                .menu_bar_summary
                .clone()
                .or_else(|| Some("No tracked app".to_string())),
            ContextAttachmentKind::RecentScripts => {
                if let Some(count) = self.script_count {
                    Some(format!("{count} scripts indexed"))
                } else {
                    Some("Script Kit scripts and recent invocations".to_string())
                }
            }
            ContextAttachmentKind::GitStatus => {
                Some("Current repository status at attach time".to_string())
            }
            ContextAttachmentKind::GitDiff => {
                Some("Current repository diff at attach time".to_string())
            }
            ContextAttachmentKind::Processes => {
                Some("Running processes snapshot at attach time".to_string())
            }
            ContextAttachmentKind::System => {
                Some("System, hardware, memory, and battery snapshot".to_string())
            }
            ContextAttachmentKind::Dictation => {
                Some("Dictation transcript at attach time".to_string())
            }
            ContextAttachmentKind::Calendar => Some("Calendar events at attach time".to_string()),
            ContextAttachmentKind::Notifications => {
                Some("Recent notifications at attach time".to_string())
            }
        }
    }

    pub(crate) fn style_selection_preview(&self) -> Option<String> {
        if let Some(t) = &self.selection_text {
            let preview = truncate_preview(t, 80);
            if !preview.is_empty() {
                return Some(format!("Will rewrite: \u{201c}{preview}\u{201d}"));
            }
        }
        Some("Will rewrite selected text (select text first)".to_string())
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
