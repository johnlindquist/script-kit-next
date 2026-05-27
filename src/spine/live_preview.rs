use crate::ai::context_contract::ContextAttachmentKind;
use std::sync::{Arc, Mutex};

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

#[derive(Debug)]
struct ExpensiveResult {
    browser_url: Option<String>,
    selection_text: Option<String>,
}

#[derive(Debug, Default)]
struct ExpensiveSlot {
    next_request_id: u64,
    in_flight: Option<(u64, std::time::Instant)>,
    ready: Option<(u64, ExpensiveResult)>,
}

#[derive(Debug)]
pub(crate) struct SpineLivePreviewCache {
    pub current: SpineLivePreview,
    pub generation: u64,
    last_expensive_kick: Option<std::time::Instant>,
    pending_expensive: Arc<Mutex<ExpensiveSlot>>,
}

impl Default for SpineLivePreviewCache {
    fn default() -> Self {
        Self {
            current: SpineLivePreview::default(),
            generation: 0,
            last_expensive_kick: None,
            pending_expensive: Arc::new(Mutex::new(ExpensiveSlot::default())),
        }
    }
}

impl SpineLivePreviewCache {
    pub(crate) fn refresh_cheap_fields(&mut self) {
        let tracked_app = crate::frontmost_app_tracker::get_last_real_app();

        let new_app = tracked_app.as_ref().map(|a| a.name.clone());
        let new_title = tracked_app.as_ref().and_then(|a| a.window_title.clone());

        let menu = crate::frontmost_app_tracker::get_cached_menu_summary();
        let new_menu = match (menu.app.as_ref(), menu.status) {
            (Some(app), crate::frontmost_app_tracker::CachedMenuStatus::Ready) => Some(format!(
                "{} \u{b7} {} menus cached",
                app.name, menu.item_count
            )),
            (Some(app), crate::frontmost_app_tracker::CachedMenuStatus::Fetching) => {
                Some(format!("{} \u{b7} menu loading\u{2026}", app.name))
            }
            (Some(app), crate::frontmost_app_tracker::CachedMenuStatus::NoCache) => {
                Some(format!("{} \u{b7} menu not cached yet", app.name))
            }
            (Some(app), crate::frontmost_app_tracker::CachedMenuStatus::StaleCacheHidden) => {
                Some(format!("{} \u{b7} menu cache stale", app.name))
            }
            _ => Some("No tracked app".to_string()),
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
            self.generation = self.generation.wrapping_add(1);
        }
    }

    fn collect_pending_expensive(&mut self) {
        let result = self
            .pending_expensive
            .lock()
            .ok()
            .and_then(|mut slot| slot.ready.take().map(|(_, r)| r));
        if let Some(r) = result {
            if r.browser_url != self.current.browser_url
                || r.selection_text != self.current.selection_text
            {
                self.current.browser_url = r.browser_url;
                self.current.selection_text = r.selection_text;
                self.generation = self.generation.wrapping_add(1);
            }
        }
    }

    pub(crate) fn refresh_expensive_fields_nonblocking(&mut self) {
        self.collect_pending_expensive();

        const THROTTLE: std::time::Duration = std::time::Duration::from_secs(2);
        const MAX_IN_FLIGHT: std::time::Duration = std::time::Duration::from_secs(8);

        if self
            .last_expensive_kick
            .is_some_and(|last| last.elapsed() < THROTTLE)
        {
            return;
        }

        let request_id = {
            let Ok(mut slot) = self.pending_expensive.lock() else {
                tracing::warn!(target: "script_kit::spine", "preview expensive slot poisoned");
                return;
            };
            if let Some((_, started_at)) = slot.in_flight {
                if started_at.elapsed() < MAX_IN_FLIGHT {
                    return;
                }
            }
            slot.next_request_id = slot.next_request_id.wrapping_add(1).max(1);
            let id = slot.next_request_id;
            slot.in_flight = Some((id, std::time::Instant::now()));
            id
        };

        self.last_expensive_kick = Some(std::time::Instant::now());

        let slot = Arc::clone(&self.pending_expensive);
        std::thread::spawn(move || {
            let result = ExpensiveResult {
                browser_url: crate::platform::get_any_browser_tab_url(),
                selection_text: crate::selected_text::get_selected_text()
                    .ok()
                    .filter(|t| !t.trim().is_empty()),
            };
            if let Ok(mut slot) = slot.lock() {
                if slot.in_flight.map(|(id, _)| id) == Some(request_id) {
                    slot.ready = Some((request_id, result));
                    slot.in_flight = None;
                }
            }
        });
    }

    pub(crate) fn set_script_count(&mut self, count: usize) {
        if self.current.script_count != Some(count) {
            self.current.script_count = Some(count);
            self.generation = self.generation.wrapping_add(1);
        }
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

    pub(crate) fn subtitle_for_context_kind(&self, kind: ContextAttachmentKind) -> String {
        match kind {
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
        }
    }

    pub(crate) fn style_selection_preview(&self) -> String {
        if let Some(t) = &self.selection_text {
            let preview = truncate_preview(t, 80);
            if !preview.is_empty() {
                return format!("Will rewrite: \u{201c}{preview}\u{201d}");
            }
        }
        "Will rewrite selected text (select text first)".to_string()
    }
}

fn truncate_preview(input: &str, max_chars: usize) -> String {
    super::text_preview::single_line_truncate(input, max_chars)
}
