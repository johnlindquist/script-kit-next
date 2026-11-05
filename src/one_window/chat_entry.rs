//! Every way into the model's chat opens the one chat under the main input: the launcher, a
//! script error, a prompt's handoff, the SDK, stdin, the tray, dictation, a note. A hidden window
//! shows first. What the way in carries (the error, the selection, a file, a note) waits in the
//! line above the input and goes with the first turn; its words go now or wait in the input, as
//! the way in asked.

use gpui::Context;
use sk_one_window::chat_with::ASKING_ABOUT;
use sk_one_window::work::WorkEvent;
use sk_one_window::{Event, Subject, SubjectKind};

pub(crate) use crate::ai::agent_chat::ui::entry::ChatEntry;
use crate::ai::message_parts::AiContextPart;
use crate::{AppView, FocusTarget, ScriptListApp};

/// The row ⌘K offers after what it staged's own action: the chat opens about it.
pub(crate) const ABOUT_KEY: &str = "chat:about";

pub(crate) fn about_row() -> sk_one_window::Row {
    sk_one_window::Row::new(ABOUT_KEY, "Ask the model about it")
        .with_enter_label("Ask the model about it")
        .with_sub("or say “what is this”")
        .with_icon("sparkles")
}

/// How the line above the input names a part.
fn subject_of(key: String, part: &AiContextPart) -> Subject {
    let (kind, sub, icon) = match part {
        AiContextPart::FilePath { path, .. } => (
            SubjectKind::Thing,
            std::path::Path::new(path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map_or_else(|| "File".to_string(), |dir| format!("File in {dir}")),
            "file",
        ),
        AiContextPart::SkillFile { owner_label, .. } => (
            SubjectKind::Thing,
            format!("Skill · {owner_label}"),
            "skill",
        ),
        AiContextPart::FocusedTarget { target, .. } => {
            let kind = match target.kind.as_str() {
                "app" => SubjectKind::App,
                "script" | "scriptlet" => SubjectKind::Script,
                _ => SubjectKind::Thing,
            };
            (kind, String::new(), "builtin")
        }
        AiContextPart::TextBlock { source, .. } if source == SELECTION_SOURCE => {
            (SubjectKind::Selection, "Selected text".to_string(), "text")
        }
        AiContextPart::TextBlock { source, .. } => (SubjectKind::Thing, source.clone(), "text"),
        AiContextPart::ResourceUri { .. } => (SubjectKind::Thing, String::new(), "attach"),
        AiContextPart::AmbientContext { .. } => (
            SubjectKind::Window,
            "What's on screen".to_string(),
            "window",
        ),
    };
    Subject {
        kind,
        label: ASKING_ABOUT.into(),
        title: part.label().to_string(),
        sub,
        icon: icon.into(),
        entry: None,
        row_key: Some(key),
    }
}

/// The source a part of the front app's selected or focused text is tagged with.
pub(crate) const SELECTION_SOURCE: &str = "frontmost-app#selection=full";

/// What was read off the screen for an entry that asked for it.
fn captured_parts(
    kind: crate::ai::TabAiCaptureKind,
    artifacts: crate::agent_handoff::TabAiDeferredCaptureArtifacts,
) -> Vec<AiContextPart> {
    use crate::ai::TabAiCaptureKind as Kind;
    let desktop = artifacts.desktop;
    let app = desktop
        .frontmost_app
        .as_ref()
        .map(|app| app.name.clone())
        .unwrap_or_default();
    let mut parts = Vec::new();
    if matches!(kind, Kind::SelectedText | Kind::DefaultContext) {
        if let Some(text) = desktop.selected_text.filter(|t| !t.trim().is_empty()) {
            parts.push(AiContextPart::TextBlock {
                label: if app.is_empty() {
                    "Selected text".into()
                } else {
                    format!("Selected text in {app}")
                },
                source: SELECTION_SOURCE.into(),
                text,
                mime_type: None,
            });
        }
    }
    if matches!(kind, Kind::BrowserTab | Kind::DefaultContext) {
        if let Some(browser) = desktop.browser {
            parts.push(AiContextPart::ResourceUri {
                label: browser.url.clone(),
                uri: browser.url,
            });
        }
    }
    if let Some(path) = artifacts.screenshot_path {
        parts.push(AiContextPart::FilePath {
            path,
            label: match kind {
                Kind::FullScreen => "The screen".into(),
                _ if app.is_empty() => "The front window".into(),
                _ => format!("{app}'s window"),
            },
        });
    }
    parts
}

impl ScriptListApp {
    /// Opens the one chat for `entry`: the window shows, the chat opens under its input (or the
    /// open chat takes it), and what the entry carries waits in the line above the input.
    pub(super) fn one_window_review_with_launcher_author(
        &mut self,
        words: String,
        cx: &mut Context<Self>,
    ) {
        if self.personal_root().is_err() {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Launcher customization is unavailable in this workspace",
                cx,
            );
            return;
        }
        let Some(profile) = crate::ai::agent_chat::launch::selected_launcher_author() else {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Choose a launcher-only profile, or type Customize Kit for offline examples",
                cx,
            );
            return;
        };
        // A clean draft only. No attachment or capture, no model call until another Return.
        self.one_window_open_chat(ChatEntry::seeded(words), cx);
        self.one_window_host.work.reviewed_author = Some(profile);
    }
    pub(crate) fn one_window_open_chat(&mut self, entry: ChatEntry, cx: &mut Context<Self>) {
        tracing::info!(
            target: "script_kit::one_window",
            event = "chat_entry",
            send = entry.send,
            has_text = !entry.text.trim().is_empty(),
            parts = entry.parts.len(),
            capture = ?entry.capture,
        );
        self.one_window_leave_for_chat(cx);
        let chat_on_top = self
            .one_window
            .views()
            .top()
            .is_some_and(|top| top.name == sk_one_window::chat::VIEW);
        if !chat_on_top {
            self.one_window_host.work.chat = None;
            self.one_window_host.work.reviewed_author = None;
            self.one_window_host.work.parts.clear();
        }
        let with = self.one_window_stage_parts(entry.parts);
        self.one_window_handle(
            Event::Work(WorkEvent::OpenChat {
                text: entry.text,
                send: entry.send,
                with,
            }),
            cx,
        );
        if let Some(kind) = entry.capture {
            self.one_window_capture_for_chat(kind, cx);
        }
    }

    /// Parts the chat holds until a turn takes them, as the subjects the line shows.
    fn one_window_stage_parts(&mut self, parts: Vec<AiContextPart>) -> Vec<Subject> {
        let work = &mut self.one_window_host.work;
        parts
            .into_iter()
            .map(|part| {
                work.next_part += 1;
                let key = format!("ctx:{}", work.next_part);
                let subject = subject_of(key.clone(), &part);
                work.parts.insert(key, part);
                subject
            })
            .collect()
    }

    /// The chat and past chats are under the launcher's input: another view steps aside until
    /// they close, and a hidden window shows.
    pub(crate) fn one_window_leave_for_chat(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.current_view, AppView::ScriptList) {
            let from = self.current_view.clone();
            let focus = Self::tab_ai_return_focus_target_for_view(&from);
            self.one_window_host.work.return_to = Some((from, focus));
            self.restore_current_view_with_focus(AppView::ScriptList, FocusTarget::MainFilter);
        }
        if !script_kit_gpui::is_main_window_visible() {
            self.dispatch_window_event(
                crate::window_orchestrator::WindowEvent::ShowMain { activate_app: true },
                cx,
            );
        }
    }

    /// After each turn of the reducer: the SDK can tell whether the chat is showing, and the view
    /// that stepped aside for the chat comes back once the chat and whatever it opened have
    /// closed.
    pub(crate) fn one_window_after_turn(&mut self) {
        crate::ai::agent_chat::ui::entry::note_chat_on_top(
            self.one_window
                .views()
                .top()
                .is_some_and(|top| top.name == sk_one_window::chat::VIEW),
        );
        if self.one_window.in_view() {
            return;
        }
        if let Some((view, focus)) = self.one_window_host.work.return_to.take() {
            if matches!(self.current_view, AppView::ScriptList) {
                self.restore_current_view_with_focus(view, focus);
            }
        }
    }

    /// Reads the screen off the UI thread, then puts what it read in the line with the rest.
    fn one_window_capture_for_chat(
        &mut self,
        kind: crate::ai::TabAiCaptureKind,
        cx: &mut Context<Self>,
    ) {
        let captured = Self::spawn_tab_ai_pre_switch_capture(kind);
        cx.spawn(async move |this, cx| {
            let Ok(Ok(artifacts)) = captured.recv().await else {
                tracing::warn!(target: "script_kit::one_window", event = "chat_capture_failed");
                return;
            };
            let _ = this.update(cx, |app, cx| {
                let with = app.one_window_stage_parts(captured_parts(kind, artifacts));
                if with.is_empty() {
                    return;
                }
                app.one_window_handle(
                    Event::Work(WorkEvent::OpenChat {
                        text: String::new(),
                        send: false,
                        with,
                    }),
                    cx,
                );
            });
        })
        .detach();
    }
}

/// Lets library surfaces (a chat prompt's hand-off, the SDK) open the chat in the main window.
pub(crate) fn register_chat_host() {
    crate::ai::agent_chat::ui::entry::register_host(crate::ai::agent_chat::ui::entry::ChatHost {
        open: |entry, cx| {
            main_app(cx).is_some_and(|app| {
                app.update(cx, |app, cx| app.one_window_open_chat(entry, cx));
                true
            })
        },
    });
}

fn main_app(cx: &gpui::App) -> Option<gpui::Entity<ScriptListApp>> {
    let handle = crate::get_main_window_handle()?.downcast::<gpui_component::Root>()?;
    let root = handle.read(cx).ok()?;
    root.view().clone().downcast::<ScriptListApp>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_names_a_file_by_its_folder_and_a_selection_as_selected_text() {
        let file = subject_of(
            "ctx:1".into(),
            &AiContextPart::FilePath {
                path: "/tmp/script-error-context/abc/deploy-error-report.md".into(),
                label: "deploy-error-report.md".into(),
            },
        );
        assert_eq!(file.title, "deploy-error-report.md");
        assert_eq!(file.sub, "File in abc");
        assert_eq!(file.label, ASKING_ABOUT);
        let selection = subject_of(
            "ctx:2".into(),
            &AiContextPart::TextBlock {
                label: "Selected text in Mail".into(),
                source: SELECTION_SOURCE.into(),
                text: "Beta ships Oct 14".into(),
                mime_type: None,
            },
        );
        assert_eq!(selection.kind, SubjectKind::Selection);
        assert_eq!(selection.row_key.as_deref(), Some("ctx:2"));
    }
}
