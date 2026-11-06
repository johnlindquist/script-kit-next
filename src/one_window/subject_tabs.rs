//! Adapters from the existing show-time context sources to the native launcher tab strip.
use super::chat_entry::ChatEntry;
use crate::ScriptListApp;
use gpui::Context;
use sk_one_window::{
    window::subject_tabs::{SubjectPage, TabSource, TabsEvent},
    Event, Key, Row, Subject, SubjectKind,
};

fn action(id: &str, label: &str, icon: &str, verb: &str) -> Row {
    Row::new(id, label).with_icon(icon).with_enter_label(verb)
}
pub(crate) fn window_source(subject: Subject, age_ms: u64) -> TabSource {
    let label = super::staging::subject_app(&subject)
        .unwrap_or(&subject.title)
        .to_owned();
    let icon = match label.to_ascii_lowercase().as_str() {
        "ghostty" | "terminal" | "iterm2" => "terminal",
        "google chrome" | "safari" | "firefox" => "globe",
        "mail" => "mail",
        _ => subject.icon.as_str(),
    }
    .to_owned();
    TabSource {
        body: format!("{} · {}", subject.title, label).into(),
        subject,
        label,
        icon,
        age_ms,
        rows: vec![
            action("ask", "Ask about the window title", "sparkles", "Ask"),
            action(
                "left",
                "Move it to the left half",
                "panel-left",
                "Move left",
            ),
            action(
                "right",
                "Move it to the right half",
                "panel-right",
                "Move right",
            ),
        ],
    }
}
pub(crate) fn selection_source(subject: Subject, text: &str) -> TabSource {
    TabSource {
        subject,
        body: text.into(),
        label: "Selected text".into(),
        icon: "text-cursor".into(),
        age_ms: 0,
        rows: vec![
            action(
                "ask",
                if text.lines().next().is_some_and(|line| {
                    line.split_once(':')
                        .is_some_and(|(kind, _)| kind.ends_with("Error"))
                }) {
                    "Explain this error"
                } else {
                    "Ask about the selection"
                },
                "sparkles",
                "Ask",
            ),
            action(
                "note",
                "Add selection to today’s note",
                "notebook-pen",
                "Add to note",
            ),
            action("show", "Read the full selection", "text-cursor", "Read"),
            action(
                "handoff",
                "Share selection with a named agent…",
                "send",
                "Choose agent",
            ),
        ],
    }
}
pub(crate) fn copy_source(text: &str, age_ms: u64) -> Option<TabSource> {
    let subject = sk_one_window::clipboard::subject(text)?;
    let link = copied_url(text).is_some();
    let mut rows = vec![
        action(
            "ask",
            if link {
                "Ask about the link"
            } else {
                "Ask about the copied text"
            },
            "sparkles",
            "Ask",
        ),
        action(
            "note",
            "Add it to today’s note",
            "notebook-pen",
            "Add to note",
        ),
    ];
    if link {
        rows.insert(0, action("open", "Open the link", "external-link", "Open"));
    } else {
        rows.push(action("show", "Read the copied text", "clipboard", "Read"));
    }
    Some(TabSource {
        subject,
        body: text.into(),
        label: if link { "Copied link" } else { "Copied text" }.into(),
        icon: if link { "link" } else { "clipboard" }.into(),
        age_ms,
        rows,
    })
}
fn copied_url(text: &str) -> Option<url::Url> {
    let text = text.trim();
    if text.contains(char::is_whitespace) {
        return None;
    }
    let url = url::Url::parse(text).ok().or_else(|| {
        (text.split('/').next()?.contains('.') && text.contains('/'))
            .then(|| url::Url::parse(&format!("https://{text}")).ok())
            .flatten()
    })?;
    (matches!(url.scheme(), "https" | "http") && url.host_str().is_some()).then_some(url)
}
impl ScriptListApp {
    pub(crate) fn has_launcher_tabs(&self) -> bool {
        self.one_window.tabs_visible()
            && self.one_window_owns_list()
            && !self.show_info_panel
            && !self.one_window.subject_pages().is_empty()
    }
    pub(crate) fn change_subjects(&mut self, event: TabsEvent, cx: &mut Context<Self>) -> bool {
        let pick = matches!(event, TabsEvent::Pick(_) | TabsEvent::Paste(_));
        let before = self.one_window_jev_token();
        let taken = self.one_window_handle(Event::Subjects(event), cx).taken();
        if before != self.one_window_jev_token() {
            self.one_window_republish("subject_tabs", cx);
        }
        if taken && pick {
            self.one_window_show_selection(cx);
        }
        taken
    }
    pub(crate) fn tab_items(&self) -> Vec<crate::components::subject_tab_bar::TabItem> {
        if !self.has_launcher_tabs() {
            return Vec::new();
        }
        let selected = self.one_window.subject_page().map(|p| p.id);
        self.one_window
            .subject_pages()
            .into_iter()
            .map(|p| {
                let active = selected.as_ref() == Some(&p.id);
                let hint = if !self.filter_text.is_empty() {
                    None
                } else if active {
                    Some("␣")
                } else {
                    match p.source.subject.kind {
                        SubjectKind::Clipboard => Some("⌘V"),
                        SubjectKind::Entry => Some("↑"),
                        _ => None,
                    }
                };
                crate::components::subject_tab_bar::TabItem {
                    active,
                    hint,
                    fresh: self.one_window.tab_fresh(&p),
                    age: if p.source.subject.kind == SubjectKind::Thing {
                        String::new()
                    } else {
                        p.age_label(self.one_window_clock())
                    },
                    id: p.id,
                    label: p.source.label.clone(),
                    detail: p.source.subject.title.clone(),
                    icon: p.source.icon.clone(),
                }
            })
            .collect()
    }
    pub(crate) fn subject_tab_elements(&self) -> Vec<crate::protocol::ElementInfo> {
        self.tab_items()
            .iter()
            .enumerate()
            .map(|(i, t)| t.element(i))
            .collect()
    }
    pub(crate) fn stage_tab_tile(&mut self, id: &str, cx: &mut Context<Self>) -> bool {
        self.has_launcher_tabs()
            && self.filter_text.is_empty()
            && self.one_window.subject_page().is_some_and(|p| p.id == id)
            && self.one_window_key(Key::Space, cx).taken()
    }
    pub(crate) fn render_subject_bar(&self, cx: &Context<Self>) -> gpui::AnyElement {
        let app = cx.entity().downgrade();
        let items = self.tab_items();
        let cycle = items.len() > 1 && self.filter_text.is_empty();
        crate::components::subject_tab_bar::render(
            &self.theme,
            self.current_main_menu_theme.def(),
            items,
            cycle,
            self.one_window_host.tab_hints,
            std::rc::Rc::new(move |id, window, cx| {
                if let Some(app) = app.upgrade() {
                    app.update(cx, |view, cx| {
                        if view.has_launcher_tabs()
                            && view.change_subjects(TabsEvent::Pick(id.into()), cx)
                        {
                            view.focus_main_filter(window, cx);
                        }
                    });
                }
            }),
        )
    }
    pub(crate) fn cycle_launcher_tab(&mut self, back: bool, cx: &mut Context<Self>) -> bool {
        self.filter_text.is_empty()
            && self.has_launcher_tabs()
            && self
                .one_window_key(if back { Key::BackTab } else { Key::Tab }, cx)
                .taken()
    }
    /// Only an observation from the same show may add a subject. Never read a clipboard payload
    /// speculatively; the monitor supplies an identity only after its normal privacy filters.
    pub(crate) fn begin_subject_tabs(
        &mut self,
        window: Option<TabSource>,
        token: u64,
        cx: &mut Context<Self>,
    ) {
        self.reload_personal_rules(cx);
        self.spawn_permission_setup_refresh(cx);
        let window = window.or_else(|| {
            Some(TabSource {
                subject: Subject {
                    kind: SubjectKind::Thing,
                    title: "The desktop".into(),
                    icon: "monitor".into(),
                    ..Default::default()
                },
                body: "The desktop".into(),
                label: "Desktop".into(),
                icon: "monitor".into(),
                age_ms: u64::MAX,
                rows: vec![action("today", "Open today’s note", "calendar", "Today")],
            })
        });
        self.change_subjects(TabsEvent::Open { window, copy: None }, cx);
        if !self.main_services.is_production() {
            return;
        }
        cx.spawn(async move |this, cx| {
            let copy = cx
                .background_executor()
                .spawn(async {
                    let (text, age) = crate::clipboard_history::recent_tab_copy()?;
                    copy_source(&text, age)
                })
                .await;
            if let Some(copy) = copy {
                let _ = this.update(cx, |app, cx| {
                    if app.shown_selection_hint_token == token {
                        app.change_subjects(TabsEvent::Observe(copy), cx);
                    }
                });
            }
        })
        .detach();
    }
    pub(super) fn run_subject_action(
        &mut self,
        row: &Row,
        page: Option<SubjectPage>,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let page = page.ok_or_else(|| anyhow::anyhow!("That subject is no longer available"))?;
        let id = row
            .key
            .strip_prefix(&format!("subject/{}/", page.id))
            .ok_or_else(|| anyhow::anyhow!("That subject changed"))?;
        let source = &page.source;
        match id {
            "today" => self.open_note_in_editor_soon(
                crate::notes::NoteTarget::Today,
                crate::DayPageBack::Leave,
                cx,
            ),
            "remind" => {
                use sk_one_window::form::{Form, Reading, Verb};
                let mut subject = source.subject.clone();
                subject.title = source.body.to_string();
                let form = Form::new(&Reading {
                    verb: Verb::Remind,
                    sentence: String::new(),
                    slots: vec![],
                    subject: Some(subject),
                });
                self.one_window_open_hosted(
                    super::views::HostedView::Form(form),
                    Some(String::new()),
                    cx,
                );
            }
            "ask" | "ask-words" => {
                let part = crate::ai::message_parts::AiContextPart::TextBlock {
                    label: source.subject.title.clone(),
                    source: source.label.clone(),
                    text: source.body.to_string(),
                    mime_type: None,
                };
                // The typed instruction is the first turn, sent with the subject: Enter on
                // “Translate to French · Selected text” translates it.
                let entry = if id == "ask-words" {
                    ChatEntry::sending(self.filter_text.trim().to_string())
                } else if row.title == "Explain this error" {
                    ChatEntry::seeded("Explain this error")
                } else {
                    ChatEntry::quick()
                };
                self.one_window_open_chat(entry.with(part), cx);
            }
            "handoff" if source.subject.kind == SubjectKind::Selection => {
                self.choose_handoff_recipient(source.body.to_string(), cx)
            }
            "note" => self.one_window_open_note(source.body.to_string(), cx),
            "sort-lines" | "unique-lines" => {
                use sk_one_window::quick_lines::LineAction;
                let action = if id == "sort-lines" {
                    LineAction::Sort
                } else {
                    LineAction::Unique
                };
                self.one_window_open_local_answer(
                    action.title(),
                    action.apply(&source.body),
                    action.hint(),
                    sk_one_window::form::Source::Staged(source.subject.kind),
                    cx,
                );
            }
            "word-count" => self.one_window_open_local_answer(
                "Word count",
                sk_one_window::quick_tools::word_count_label(&source.body),
                "Counted locally, without sending your text",
                sk_one_window::form::Source::Staged(source.subject.kind),
                cx,
            ),
            "title-case" => self.one_window_open_local_answer(
                "Title case",
                sk_one_window::quick_tools::title_case(&source.body),
                "Each word capitalized; your original text is unchanged",
                sk_one_window::form::Source::Staged(source.subject.kind),
                cx,
            ),
            "show" => {
                use sk_one_window::{
                    answer::{Answer, Fact},
                    form::Source,
                };
                let mut answer = Answer::asking(source.label.clone());
                answer.arrived(vec![Fact::new(
                    source.body.to_string(),
                    source.subject.sub.clone(),
                    Source::Staged(source.subject.kind),
                )]);
                self.one_window_open_hosted(super::views::HostedView::Answer(answer), None, cx);
            }
            "open" => {
                let url =
                    copied_url(&source.body).ok_or_else(|| anyhow::anyhow!("Not a web link"))?;
                crate::browser_history::open_browser_history_url(url.as_str())?;
                self.one_window_notice(crate::line_notice::INFO, "Opened the copied link", cx);
            }
            "left" | "right" => {
                anyhow::ensure!(
                    self.main_services.is_production(),
                    "Desktop access is disabled in this session"
                );
                crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
                let op = crate::intent::native_window_ops()
                    .into_iter()
                    .find(|op| {
                        op.action()
                            .and_then(crate::intent::WindowAction::sdk_position)
                            == Some(id)
                    })
                    .ok_or_else(|| anyhow::anyhow!("Unknown window arrangement"))?;
                let target = self.target_for_window_subject(&source.subject);
                let app = source.label.clone();
                if !crate::platform::accessibility::permissions::has_accessibility_permission() {
                    self.ask_for_accessibility(
                        &[crate::intent::Step::Window { op, app, target }],
                        cx,
                    );
                    return Ok(());
                }
                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move {
                            crate::intent_live::run_window_step(&op, &app, target.as_ref())
                        })
                        .await;
                    let _ = this.update(cx, |app, cx| match result {
                        Ok(()) => {
                            app.one_window_notice(crate::line_notice::INFO, "Moved the window", cx)
                        }
                        Err(e) => app.one_window_notice(
                            crate::line_notice::FAILED,
                            format!("Couldn’t move the window: {e}"),
                            cx,
                        ),
                    });
                })
                .detach();
            }
            _ => anyhow::bail!("That subject action is not available"),
        }
        Ok(())
    }
}
