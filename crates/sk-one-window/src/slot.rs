//! The line above the input.
//!
//! It is always there and always one height. Its words swap in place; only its dimming eases.
//! What it says, first match wins: an override a view set, the staged subject ("Acting on"),
//! the newest message waiting to go ("Sending in 8s"), the last thing that ran ("Last", dimmed),
//! and before anything ran, what "this" means ("Focused", dimmed). How dim it is depends on the
//! window too, so the window decides it (`OneWindow::slot_opacity`).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotKind {
    /// Nothing has run yet: what `⌘K` would act on.
    Focused,
    /// The last thing that ran.
    Recent,
    /// A subject put there by `⌘K` or `↑`.
    Staged,
    /// A message waiting to go.
    Holding,
    /// Words heard by voice are landing in the front app.
    Typing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotLine {
    pub kind: SlotKind,
    /// "Last", "Acting on", "Sending in 6s", "Focused", "Typing into".
    pub label: String,
    pub icon: String,
    pub title: String,
    /// Small text after the title: "just now", "+ 1 more", "in Mail".
    pub sub: String,
    /// Keys the line offers, like `("⌘Z", "undo")`.
    pub hints: Vec<(String, String)>,
    /// A held message's progress toward sending, 0..=1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
}

impl SlotLine {
    pub fn new(kind: SlotKind, label: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            kind,
            label: label.into(),
            icon: String::new(),
            title: title.into(),
            sub: String::new(),
            hints: Vec::new(),
            progress: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn with_sub(mut self, sub: impl Into<String>) -> Self {
        self.sub = sub.into();
        self
    }

    pub fn with_hint(mut self, key: impl Into<String>, label: impl Into<String>) -> Self {
        self.hints.push((key.into(), label.into()));
        self
    }

    /// The same words and state, ignoring a held message's progress bar.
    pub fn same_words(&self, other: &SlotLine) -> bool {
        self.kind == other.kind
            && self.label == other.label
            && self.title == other.title
            && self.sub == other.sub
    }
}
