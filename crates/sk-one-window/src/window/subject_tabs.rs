//! The launcher's subjects. A tab selects a target, not a new text field or a staged chip.
//! Row keys carry the observation's identity: a late selection cannot retarget a painted action.
use super::*;
use std::sync::Arc;

pub const COPY_FRESH_MS: Ms = 120_000;

#[derive(Clone, Debug, PartialEq)]
pub struct TabSource {
    pub subject: Subject,
    /// The complete selection/copy. Never used as a row key, log field or tab label.
    pub body: Arc<str>,
    pub label: String,
    pub icon: String,
    pub rows: Vec<Row>,
    pub age_ms: Ms,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabsEvent {
    /// A new opening; passive inputs are snapshots, not a read of the system clipboard.
    Open {
        window: Option<TabSource>,
        copy: Option<Box<TabSource>>,
    },
    Observe(TabSource),
    /// One validated settings snapshot. Never changes the identity of a subject observation.
    Personal(crate::personal::LauncherRules),
    /// Explicit paste can reach a copy even when not offered passively.
    Paste(TabSource),
    Pick(String),
}

#[derive(Clone, Debug)]
pub struct SubjectPage {
    pub id: String,
    pub source: Arc<TabSource>,
    observed: Ms,
    first: Option<&'static str>,
}
impl SubjectPage {
    pub fn age(&self, now: Ms) -> Ms {
        self.source
            .age_ms
            .saturating_add(now.saturating_sub(self.observed))
    }
    pub fn age_label(&self, now: Ms) -> String {
        let s = self.age(now) / 1000;
        match s {
            0..=59 => format!("{s}s"),
            60..=3599 => format!("{}m", s / 60),
            3600..=86399 => format!("{}h", s / 3600),
            _ => format!("{}d", s / 86400),
        }
    }
    /// A local, explicitly scoped chat draft; no selected or copied text goes into a
    /// classifier request. Non-referential launcher searches keep their normal matches.
    pub fn reading(&self, text: &str) -> Option<Row> {
        let words: Vec<_> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_ascii_lowercase)
            .collect();
        let verb = words.first()?.as_str();
        if !matches!(
            verb,
            "explain" | "summarize" | "rewrite" | "translate" | "ask"
        ) || !words.iter().any(|w| matches!(w.as_str(), "this" | "it"))
        {
            return None;
        }
        Some(
            Row::new(
                format!("subject/{}/ask-words", self.id),
                format!("{} · {}", text, self.source.label),
            )
            .with_sub("Open a chat with this subject")
            .with_icon("sparkles")
            .with_enter_label("Open chat"),
        )
    }
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = self.source.rows.clone();
        if let Some(at) = self
            .first
            .and_then(|key| rows.iter().position(|r| r.key == key))
        {
            rows[..=at].rotate_right(1);
        }
        for (i, row) in rows.iter_mut().enumerate() {
            row.key = format!("subject/{}/{}", self.id, row.key);
            row.head = (i == 0).then(|| self.source.subject.title.clone());
        }
        rows
    }
}
#[derive(Clone, Debug, Default)]
pub(super) struct Tabs {
    personal: crate::personal::LauncherRules,
    pub enabled: bool,
    serial: u64,
    live: Vec<SubjectPage>,
    pick: Option<String>,
    history: Option<u64>,
    attached: Option<SubjectPage>,
    seen: Vec<String>,
    previous: Vec<SubjectPage>,
    ignored_copy: Option<String>,
}
impl Tabs {
    fn put(&mut self, now: Ms, s: TabSource) -> String {
        let kind = s.subject.kind;
        if let Some(page) = self.live.iter().find(|p| {
            p.source.subject == s.subject
                && p.source.body == s.body
                && (kind != SubjectKind::Clipboard || p.age(now).abs_diff(s.age_ms) < 1000)
        }) {
            return page.id.clone();
        }
        self.live.retain(|p| p.source.subject.kind != kind);
        if let Some(i) = self.previous.iter().position(|p| {
            p.source.subject == s.subject
                && p.source.body == s.body
                && (kind != SubjectKind::Clipboard || p.age(now).abs_diff(s.age_ms) < 1000)
        }) {
            let p = self.previous.remove(i);
            let id = p.id.clone();
            self.live.push(p);
            return id;
        }
        self.serial += 1;
        let id = format!(
            "{}-{}",
            match kind {
                SubjectKind::Window | SubjectKind::Thing => "window",
                SubjectKind::Selection => "selection",
                _ => "copy",
            },
            self.serial
        );
        self.live.push(SubjectPage {
            id: id.clone(),
            observed: now,
            first: None,
            source: Arc::new(s),
        });
        id
    }
}
impl OneWindow {
    pub fn personal_rules(&self) -> crate::personal::LauncherRules {
        self.tabs.personal
    }
    pub fn tabs_enabled(&self) -> bool {
        self.tabs.enabled
    }
    pub fn tabs_visible(&self) -> bool {
        self.tabs.enabled && !self.in_view() && self.staged.is_none() && !self.away
    }
    pub fn subject_pages(&self) -> Vec<SubjectPage> {
        if !self.tabs_visible() {
            return Vec::new();
        }
        let selection = self
            .tabs
            .live
            .iter()
            .find(|p| p.source.subject.kind == SubjectKind::Selection);
        let mut pages: Vec<_> = self
            .tabs
            .live
            .iter()
            .filter(|p| {
                p.source.subject.kind != SubjectKind::Clipboard
                    || ((p.age(self.now) < COPY_FRESH_MS
                        && self.tabs.ignored_copy.as_ref() != Some(&p.id)
                        || self.tabs.pick.as_ref() == Some(&p.id))
                        && !selection.is_some_and(|s| s.source.body == p.source.body))
            })
            .cloned()
            .collect();
        for page in &mut pages {
            page.first = self.tabs.personal.first(page.source.subject.kind);
        }
        pages.sort_by_key(|p| {
            (
                p.age(self.now),
                match p.source.subject.kind {
                    SubjectKind::Selection => 0,
                    SubjectKind::Clipboard => 1,
                    _ => 2,
                },
            )
        });
        if let Some(e) = self
            .tabs
            .history
            .and_then(|id| self.ledger.get(id))
            .or_else(|| self.ledger.last())
        {
            let mut rows = entry_rows(e);
            // Rerunning an arbitrary notice or a run from another process is not guaranteed.
            rows.retain(|r| r.act != RowAct::Again);
            if !e.said.trim().is_empty() {
                rows.insert(
                    usize::from(e.can_undo()),
                    Row::new("edit", "Edit the words")
                        .with_icon("pencil")
                        .with_enter_label("Edit"),
                );
            }
            let n = self
                .ledger
                .entries()
                .iter()
                .rev()
                .position(|x| x.id == e.id)
                .unwrap_or(0);
            pages.push(SubjectPage {
                id: format!("run-{}", e.id),
                observed: self.now,
                first: None,
                source: Arc::new(TabSource {
                    subject: entry_subject(e, words::LAST),
                    body: Arc::from(e.said.as_str()),
                    label: if n == 0 {
                        "Last run".into()
                    } else {
                        format!("{} runs ago", n + 1)
                    },
                    icon: "history".into(),
                    age_ms: e.age(self.now),
                    rows,
                }),
            });
        }
        pages
    }
    pub fn subject_page(&self) -> Option<SubjectPage> {
        let pages = self.subject_pages();
        self.tabs
            .pick
            .as_ref()
            .and_then(|id| pages.iter().find(|p| &p.id == id))
            .or(pages.first())
            .cloned()
    }
    pub fn page_for_action(&self) -> Option<SubjectPage> {
        if self.staged.is_some() {
            self.tabs
                .attached
                .as_ref()
                .filter(|p| {
                    self.staged
                        .as_ref()
                        .and_then(|s| s.subject.row_key.as_ref())
                        == Some(&p.id)
                })
                .cloned()
        } else {
            self.subject_page()
        }
    }
    pub fn tab_fresh(&self, p: &SubjectPage) -> bool {
        matches!(
            p.source.subject.kind,
            SubjectKind::Selection | SubjectKind::Clipboard
        ) && !self.tabs.seen.contains(&p.id)
            && self
                .subject_pages()
                .first()
                .is_some_and(|first| first.id != p.id)
    }
    fn pick_page(&mut self, id: String) -> bool {
        if !self.subject_pages().iter().any(|p| p.id == id) {
            return false;
        }
        if !self.tabs.seen.contains(&id) {
            self.tabs.seen.push(id.clone());
        }
        self.tabs.pick = Some(id);
        self.list.sel = 0;
        self.chose = false;
        self.refresh_rows();
        true
    }
    pub(super) fn subject_event(&mut self, now: Ms, event: TabsEvent) -> bool {
        match event {
            TabsEvent::Open { window, copy } => {
                if self.tabs.enabled
                    && !self.chose
                    && self.tabs.pick.is_none()
                    && self.text.is_empty()
                {
                    if let Some(p) = self
                        .subject_page()
                        .filter(|p| p.source.subject.kind == SubjectKind::Clipboard)
                    {
                        self.tabs.ignored_copy = Some(p.id);
                    }
                }
                self.tabs.enabled = true;
                self.tabs.pick = None;
                self.tabs.seen.clear();
                self.tabs.history = None;
                self.tabs.previous = std::mem::take(&mut self.tabs.live);
                self.chose = false;
                for s in [window, copy.map(|s| *s)].into_iter().flatten() {
                    self.tabs.put(now, s);
                }
            }
            TabsEvent::Personal(rules) => {
                self.tabs.personal = rules;
            }
            TabsEvent::Observe(s) => {
                self.tabs.put(now, s);
            }
            TabsEvent::Paste(s) => {
                if !self.tabs_visible() || !self.text.is_empty() {
                    return false;
                }
                let same = self
                    .tabs
                    .live
                    .iter()
                    .find(|p| {
                        p.source.subject.kind == SubjectKind::Selection && p.source.body == s.body
                    })
                    .map(|p| p.id.clone());
                let id = same.unwrap_or_else(|| self.tabs.put(now, s));
                self.tabs.ignored_copy = None;
                self.tabs.pick = Some(id.clone());
                return self.pick_page(id);
            }
            TabsEvent::Pick(id) => return self.pick_page(id),
        }
        self.refresh_rows();
        true
    }
    pub(super) fn subject_key(&mut self, now: Ms, key: &Key, fx: &mut Vec<Effect>) -> Option<bool> {
        if !self.tabs_visible() || !self.text.is_empty() || !self.context.is_empty() {
            return None;
        }
        let pages = self.subject_pages();
        let cur = self.subject_page()?;
        match key {
            Key::Tab | Key::BackTab => {
                let at = pages.iter().position(|p| p.id == cur.id)?;
                let n = pages.len();
                Some(
                    self.pick_page(
                        pages[(at + if *key == Key::Tab { 1 } else { n - 1 }) % n]
                            .id
                            .clone(),
                    ),
                )
            }
            Key::Up if self.list.sel == 0 && !self.ledger.is_empty() => {
                let entries = self.ledger.entries();
                let last = entries.len() - 1;
                let i = self
                    .tabs
                    .history
                    .filter(|_| {
                        cur.source.subject.entry.is_some()
                            && self.tabs.pick.as_ref() == Some(&cur.id)
                    })
                    .and_then(|id| entries.iter().position(|e| e.id == id))
                    .map_or(last, |i| i.saturating_sub(1));
                let id = entries[i].id;
                self.tabs.history = Some(id);
                Some(self.pick_page(format!("run-{id}")))
            }
            Key::Escape if self.tabs.pick.is_some() => {
                self.tabs.pick = None;
                self.tabs.history = None;
                self.list.sel = 0;
                self.chose = false;
                self.refresh_rows();
                Some(true)
            }
            Key::Space | Key::CmdK
                if *key == Key::Space
                    || !self.chose
                    || self
                        .list
                        .current_row()
                        .is_some_and(|r| r.key.starts_with("subject/")) =>
            {
                let mut subject = cur.source.subject.clone();
                subject.row_key = Some(cur.id.clone());
                let mut rows = cur.rows();
                if cur.source.subject.kind == SubjectKind::Clipboard {
                    rows.push(
                        Row::new(format!("subject/{}/remind", cur.id), "Remind me about it")
                            .with_icon("alarm-clock")
                            .with_enter_label("Remind"),
                    );
                }
                self.tabs.attached = Some(cur);
                self.stage(now, subject, Some(rows), words::STAGED, fx);
                Some(true)
            }
            _ => None,
        }
    }
}
