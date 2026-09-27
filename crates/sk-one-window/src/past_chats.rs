//! Chats from before, as rows under the same input: "Past chats" heads the list, newest first,
//! typing filters, ↵ opens that chat again, and ⌘K on a row puts it in the line with what you can
//! do to it (continue, copy, export, delete). Asking for "my last chat" or "chats about the
//! launch" opens it; so does the chat's own "See past chats".

use crate::contract::words;
use crate::enter::EnterTarget;
use crate::row::Row;
use crate::slot::{SlotKind, SlotLine};
use crate::window::ViewSpec;

pub const VIEW: &str = "past-chats";
pub const LABEL: &str = "Past chats";
/// The chat's next step and ⌘K row that opens the past chats.
pub const ROW_KEY: &str = "chat:past";

pub const CONTINUE: &str = "continue";
pub const COPY: &str = "copy";
pub const EXPORT: &str = "export";
pub const DELETE: &str = "delete";

#[derive(Clone, Debug, PartialEq)]
pub struct PastChat {
    pub id: String,
    pub title: String,
    /// When it was, in words ("Yesterday", "Sep 12").
    pub when: String,
    pub preview: String,
    /// Lowercased words typing is matched against.
    pub search: String,
}

/// What Enter did in the past chats.
#[derive(Clone, Debug, PartialEq)]
pub enum PastKey {
    Nothing,
    Reopen(String),
    Act { id: String, act: &'static str },
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PastChats {
    /// Newest first, as the host listed them.
    pub chats: Vec<PastChat>,
    pub query: String,
    pub sel: usize,
    pub loaded: bool,
    /// The chat ⌘K put in the line; the rows are its actions.
    pub staged: Option<PastChat>,
    /// The chat Enter asked to open again, until it is open.
    pub opening: Option<String>,
}

fn actions() -> [(&'static str, &'static str, &'static str); 4] {
    [
        (CONTINUE, "Continue it", "message"),
        (COPY, "Copy it", "copy"),
        (EXPORT, "Export it as Markdown", "export"),
        (DELETE, "Delete it", "trash"),
    ]
}

impl PastChats {
    pub fn open(query: &str) -> Self {
        Self {
            query: query.trim().to_string(),
            ..Self::default()
        }
    }

    pub fn spec(&self) -> ViewSpec {
        ViewSpec {
            name: VIEW.into(),
            label: LABEL.into(),
            enter: Some(self.enter()),
            takes_input: true,
            text: Some(self.query.clone()),
            leave_text: None,
        }
    }

    pub fn listed(&mut self, chats: Vec<PastChat>) {
        self.chats = chats;
        self.loaded = true;
        self.clamp();
    }

    pub fn filter(&mut self, text: &str) {
        let query = text.trim().to_string();
        if query != self.query {
            self.query = query;
            self.sel = 0;
        }
    }

    /// The chats typing leaves, newest first.
    pub fn shown(&self) -> Vec<&PastChat> {
        let words: Vec<String> = self
            .query
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        self.chats
            .iter()
            .filter(|c| {
                let hay = format!("{} {} {}", c.title, c.preview, c.search).to_lowercase();
                words.iter().all(|w| hay.contains(w.as_str()))
            })
            .collect()
    }

    /// The rows under the header: the chats, or the staged chat's actions.
    pub fn rows(&self) -> Vec<Row> {
        match &self.staged {
            Some(chat) => actions()
                .into_iter()
                .map(|(act, title, icon)| {
                    Row::new(format!("past:{}:{act}", chat.id), title)
                        .with_icon(icon)
                        .with_enter_label(format!("{title} · “{}”", chat.title))
                })
                .collect(),
            None => self
                .shown()
                .into_iter()
                .map(|c| {
                    let sub = match (c.when.is_empty(), c.preview.is_empty()) {
                        (false, false) => format!("{} · {}", c.when, c.preview),
                        (false, true) => c.when.clone(),
                        _ => c.preview.clone(),
                    };
                    Row::new(format!("past:{}", c.id), c.title.clone())
                        .with_icon("message")
                        .with_sub(sub)
                        .with_enter_label(format!("Open “{}”", c.title))
                })
                .collect(),
        }
    }

    /// The header over the rows.
    pub fn head(&self) -> &'static str {
        if self.staged.is_some() {
            words::STAGED
        } else {
            LABEL
        }
    }

    pub fn enter(&self) -> EnterTarget {
        if let Some(id) = &self.opening {
            return EnterTarget::new(format!("past:{id}:opening"), "Opening it…");
        }
        if !self.loaded {
            return EnterTarget::new("past:reading", "Reading past chats…");
        }
        match self.rows().get(self.sel) {
            Some(row) => EnterTarget::new(row.key.clone(), row.enter_words()),
            None if self.query.is_empty() => EnterTarget::new("past:none", "No past chats yet"),
            None => EnterTarget::new("past:none", format!("No chats about “{}”", self.query)),
        }
    }

    fn clamp(&mut self) {
        self.sel = self.sel.min(self.rows().len().saturating_sub(1));
    }

    pub fn move_sel(&mut self, down: bool) -> bool {
        let n = self.rows().len();
        if n == 0 {
            return false;
        }
        self.sel = if down {
            (self.sel + 1).min(n - 1)
        } else {
            self.sel.saturating_sub(1)
        };
        true
    }

    pub fn press_enter(&mut self) -> PastKey {
        if self.opening.is_some() {
            return PastKey::Nothing;
        }
        if let Some(chat) = self.staged.clone() {
            let Some((act, ..)) = actions().get(self.sel).copied() else {
                return PastKey::Nothing;
            };
            self.let_go();
            if act == CONTINUE {
                self.opening = Some(chat.id.clone());
                return PastKey::Reopen(chat.id);
            }
            return PastKey::Act { id: chat.id, act };
        }
        let Some(id) = self.shown().get(self.sel).map(|c| c.id.clone()) else {
            return PastKey::Nothing;
        };
        self.opening = Some(id.clone());
        PastKey::Reopen(id)
    }

    /// ⌘K: the highlighted chat goes in the line and the rows become its actions.
    pub fn stage(&mut self) -> bool {
        if self.staged.is_some() {
            return self.let_go();
        }
        let Some(chat) = self.shown().get(self.sel).map(|c| (*c).clone()) else {
            return false;
        };
        self.staged = Some(chat);
        self.sel = 0;
        true
    }

    pub fn let_go(&mut self) -> bool {
        let Some(chat) = self.staged.take() else {
            return false;
        };
        self.sel = self
            .shown()
            .iter()
            .position(|c| c.id == chat.id)
            .unwrap_or(0);
        true
    }

    pub fn removed(&mut self, id: &str) {
        self.chats.retain(|c| c.id != id);
        if self.staged.as_ref().is_some_and(|c| c.id == id) {
            self.staged = None;
        }
        self.clamp();
    }

    /// The line above the input while a chat is staged.
    pub fn line(&self) -> Option<SlotLine> {
        let chat = self.staged.as_ref()?;
        Some(
            SlotLine::new(SlotKind::Staged, words::ACTING_ON, chat.title.clone())
                .with_icon("message")
                .with_sub(chat.when.clone())
                .with_hint("esc", "let go"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chat(id: &str, title: &str, when: &str) -> PastChat {
        PastChat {
            id: id.into(),
            title: title.into(),
            when: when.into(),
            preview: String::new(),
            search: title.to_lowercase(),
        }
    }

    fn listed() -> PastChats {
        let mut past = PastChats::open("");
        past.listed(vec![
            chat("c3", "Launch party names", "Today"),
            chat("c2", "Fix the deploy script", "Yesterday"),
            chat("c1", "Launch checklist", "Sep 12"),
        ]);
        past
    }

    #[test]
    fn newest_first_typing_filters_and_enter_reopens() {
        let mut past = PastChats::open("");
        assert_eq!(past.enter().label, "Reading past chats…");
        past = listed();
        assert_eq!(past.enter().label, "Open “Launch party names”");
        past.filter("launch check");
        let titles: Vec<String> = past.rows().into_iter().map(|r| r.title).collect();
        assert_eq!(titles, ["Launch checklist"]);
        assert_eq!(past.press_enter(), PastKey::Reopen("c1".into()));
        assert_eq!(past.enter().label, "Opening it…");
        past.filter("nothing like it");
        past.opening = None;
        assert_eq!(past.enter().label, "No chats about “nothing like it”");
    }

    #[test]
    fn cmd_k_stages_a_chat_with_its_actions_and_esc_lets_go() {
        let mut past = listed();
        past.move_sel(true);
        assert!(past.stage());
        assert_eq!(past.head(), words::STAGED);
        let line = past.line().expect("staged");
        assert_eq!(
            (line.label.as_str(), line.title.as_str()),
            (words::ACTING_ON, "Fix the deploy script")
        );
        let titles: Vec<String> = past.rows().into_iter().map(|r| r.title).collect();
        assert_eq!(
            titles,
            [
                "Continue it",
                "Copy it",
                "Export it as Markdown",
                "Delete it"
            ]
        );
        past.move_sel(true);
        assert_eq!(past.enter().label, "Copy it · “Fix the deploy script”");
        assert_eq!(
            past.press_enter(),
            PastKey::Act {
                id: "c2".into(),
                act: COPY
            }
        );
        assert!(past.staged.is_none());
        assert_eq!(past.sel, 1, "back on the chat it acted on");
        past.stage();
        assert!(past.let_go());
        assert_eq!(past.head(), LABEL);
        past.removed("c2");
        assert_eq!(past.chats.len(), 2);
    }
}
