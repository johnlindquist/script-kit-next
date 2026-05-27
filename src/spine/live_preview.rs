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
    /// Live data promoted to the row title. Returns `None` when the static
    /// label from `ContextAttachmentSpec.label` should be used as-is.
    pub(crate) fn title_for_context_kind(&self, kind: ContextAttachmentKind) -> Option<String> {
        match kind {
            ContextAttachmentKind::Current | ContextAttachmentKind::Full => None,
            ContextAttachmentKind::Selection => {
                if let Some(t) = &self.selection_text {
                    let preview = truncate_preview(t, 50);
                    if !preview.is_empty() {
                        return Some(format!("\u{201c}{preview}\u{201d}"));
                    }
                }
                None
            }
            ContextAttachmentKind::Browser => self
                .browser_url
                .as_ref()
                .map(|url| truncate_preview(url, 55)),
            ContextAttachmentKind::Window => {
                match (&self.active_window_title, &self.frontmost_app_name) {
                    (Some(title), Some(app)) if !title.trim().is_empty() => {
                        Some(format!("{} \u{b7} {}", truncate_preview(title, 40), app))
                    }
                    (Some(title), _) if !title.trim().is_empty() => {
                        Some(truncate_preview(title, 55))
                    }
                    (_, Some(app)) if !app.trim().is_empty() => Some(app.clone()),
                    _ => None,
                }
            }
            ContextAttachmentKind::Screenshot => self
                .frontmost_app_name
                .as_ref()
                .map(|app| format!("Screenshot \u{b7} {app}")),
            ContextAttachmentKind::Clipboard => {
                if let Some(t) = &self.clipboard_text {
                    let preview = truncate_preview(t, 50);
                    if !preview.is_empty() {
                        return Some(format!("\u{201c}{preview}\u{201d}"));
                    }
                }
                None
            }
            ContextAttachmentKind::FrontmostApp => self.frontmost_app_name.clone(),
            ContextAttachmentKind::MenuBar => self.menu_bar_summary.clone(),
            ContextAttachmentKind::RecentScripts => {
                self.script_count.map(|c| format!("{c} scripts indexed"))
            }
            _ => None,
        }
    }

    /// Action-oriented subtitle shown below the title in the @ context list.
    pub(crate) fn subtitle_for_context_kind(&self, kind: ContextAttachmentKind) -> Option<String> {
        Some(match kind {
            ContextAttachmentKind::Current => {
                let mut parts = Vec::new();
                parts.push("screenshot");
                if let Some(app) = &self.frontmost_app_name {
                    parts.push(app.as_str());
                }
                if self.active_window_title.is_some() {
                    parts.push("focused window");
                }
                if self.browser_url.is_some() {
                    parts.push("browser URL");
                }
                format!("Includes {}", parts.join(" \u{b7} "))
            }
            ContextAttachmentKind::Full => {
                let mut parts = Vec::new();
                parts.push("screenshot");
                if self.clipboard_text.is_some() {
                    parts.push("clipboard");
                }
                if let Some(app) = &self.frontmost_app_name {
                    parts.push(app.as_str());
                }
                if self.browser_url.is_some() {
                    parts.push("browser");
                }
                if self.active_window_title.is_some() {
                    parts.push("window");
                }
                parts.push("menu bar");
                format!("Includes {}", parts.join(" \u{b7} "))
            }
            ContextAttachmentKind::Selection => {
                if self.selection_text.is_some() {
                    "Attach the selected text to your command".into()
                } else {
                    "No text selected \u{2014} select text first".into()
                }
            }
            ContextAttachmentKind::Browser => {
                if self.browser_url.is_some() {
                    "Attach this page\u{2019}s URL to your command".into()
                } else {
                    "No supported browser focused".into()
                }
            }
            ContextAttachmentKind::Window => {
                "Attach the focused window info to your command".into()
            }
            ContextAttachmentKind::Diagnostics => {
                "Attach capture source health and permissions".into()
            }
            ContextAttachmentKind::Screenshot => {
                "Attach a screenshot of the screen behind Script Kit".into()
            }
            ContextAttachmentKind::Clipboard => {
                if self.clipboard_text.is_some() {
                    "Attach clipboard contents to your command".into()
                } else {
                    "Clipboard is empty".into()
                }
            }
            ContextAttachmentKind::FrontmostApp => {
                "Attach frontmost app info to your command".into()
            }
            ContextAttachmentKind::MenuBar => "Attach menu bar items to your command".into(),
            ContextAttachmentKind::RecentScripts => "Attach recent script invocations".into(),
            ContextAttachmentKind::GitStatus => "Attach current repository status".into(),
            ContextAttachmentKind::GitDiff => "Attach current repository diff".into(),
            ContextAttachmentKind::Processes => "Attach running processes snapshot".into(),
            ContextAttachmentKind::System => "Attach system, hardware, and battery info".into(),
            ContextAttachmentKind::Dictation => "Attach dictation transcript".into(),
            ContextAttachmentKind::Calendar => "Attach calendar events".into(),
            ContextAttachmentKind::Notifications => "Attach recent notifications".into(),
        })
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
