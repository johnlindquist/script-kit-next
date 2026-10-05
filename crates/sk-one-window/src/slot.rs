//! The line above the input, and the subject chip inside the input.
//!
//! The line is always there and always one height. Its words swap in place; only its dimming
//! eases. It reports: a message waiting to go, what a view or a notice says, and otherwise the
//! last thing that ran. What you act on is a chip in the input: what's focused is offered at the
//! input's end while it is empty, and what Space, ⌘K or ↑ staged sits at its start. How dim the
//! line is depends on the window too, so the window decides it (`OneWindow::slot_style`). The
//! prototype's traces show all of it in the one line.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotKind {
    /// What Space would put in the line: the front app, its selection, the chat's latest reply.
    Focused,
    /// The last thing that ran.
    Recent,
    /// A subject put there by Space, `⌘K` or `↑`.
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
    /// It is what's in play: Space, ⌘K or ↑ staged it and Esc lets it go.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lit: bool,
    /// It says something went wrong: drawn with the failure icon at full strength.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub failed: bool,
}

/// What you act on, as a chip in the input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectChip {
    pub title: String,
    pub icon: String,
    /// Solid at the input's start once staged; dashed at its end while only offered.
    pub attached: bool,
    /// The key on the chip: `space` attaches an offer, `esc` lets go of what's attached.
    pub key: String,
    /// What the key does, while Space is still being learned ("use"); otherwise empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub word: String,
}

impl SubjectChip {
    /// A front window reads as its app ("Ghostty"), everything else as its title.
    pub fn of(s: &crate::stage::Subject, attached: bool) -> Self {
        let title = if s.kind == crate::stage::SubjectKind::Window && !s.sub.is_empty() {
            s.sub.clone()
        } else {
            s.title.clone()
        };
        Self {
            title,
            icon: match (s.icon.is_empty(), attached) {
                (false, _) => s.icon.clone(),
                (true, true) => "builtin".to_string(),
                (true, false) => "window".to_string(),
            },
            attached,
            word: String::new(),
            key: if attached { "esc" } else { "space" }.to_string(),
        }
    }
}

/// Something `@` attached to what's said ("@selection", "@file:notes.md"), as a chip in the
/// input after the subject. The input shows the chip; the sentence still carries its token, so
/// everything that reads the words resolves it the way it resolves the typed token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextChip {
    /// The token the sentence carries: `@selection`, `@clipboard`, `@file:notes.md`.
    pub token: String,
    pub title: String,
    pub icon: String,
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
            lit: false,
            failed: false,
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

    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// What a waiting or typing line is doing ("Runs in 8s", "Paused while you type",
    /// "Typing into"), drawn before its title. Other lines' labels only name the line for agents
    /// ("Last", "Behind") and are never drawn.
    pub fn state_words(&self) -> Option<&str> {
        matches!(self.kind, SlotKind::Holding | SlotKind::Typing)
            .then_some(self.label.as_str())
            .filter(|label| !label.is_empty())
    }

    /// The same words and state, ignoring a held message's progress bar.
    pub fn same_words(&self, other: &SlotLine) -> bool {
        self.kind == other.kind
            && self.label == other.label
            && self.title == other.title
            && self.sub == other.sub
    }
}

/// How strongly the line is drawn. Only this eases, over [`SLOT_DIM_EASE_MS`].
///
/// [`SLOT_DIM_EASE_MS`]: crate::contract::SLOT_DIM_EASE_MS
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotStyle {
    pub opacity: f32,
    /// Inside a view its keys belong to the view, so the line's own hints are hidden.
    pub hints: bool,
}

impl SlotStyle {
    pub fn new(line: &SlotLine, typing: bool, in_view: bool) -> Self {
        use crate::contract::slot_opacity as o;
        if line.failed {
            return Self {
                opacity: 1.0,
                hints: !in_view,
            };
        }
        if in_view && line.kind != SlotKind::Holding {
            return Self {
                opacity: o::IN_VIEW_NOT_HOLDING,
                hints: false,
            };
        }
        let opacity = match line.kind {
            SlotKind::Focused if typing => o::TYPING_FOCUSED,
            SlotKind::Recent if typing => o::TYPING_RECENT,
            SlotKind::Focused => o::FOCUSED,
            SlotKind::Recent => o::RECENT,
            SlotKind::Staged | SlotKind::Holding | SlotKind::Typing => 1.0,
        };
        Self {
            opacity,
            hints: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::slot_opacity as o;

    fn of(kind: SlotKind) -> SlotLine {
        SlotLine::new(kind, "", "")
    }

    #[test]
    fn context_is_dimmed_and_dims_further_while_typing() {
        assert_eq!(
            SlotStyle::new(&of(SlotKind::Focused), false, false).opacity,
            o::FOCUSED
        );
        assert_eq!(
            SlotStyle::new(&of(SlotKind::Recent), false, false).opacity,
            o::RECENT
        );
        assert_eq!(
            SlotStyle::new(&of(SlotKind::Focused), true, false).opacity,
            o::TYPING_FOCUSED
        );
        assert_eq!(
            SlotStyle::new(&of(SlotKind::Recent), true, false).opacity,
            o::TYPING_RECENT
        );
    }

    #[test]
    fn staged_and_holding_lines_are_full_strength() {
        for typing in [false, true] {
            assert_eq!(
                SlotStyle::new(&of(SlotKind::Staged), typing, false).opacity,
                1.0
            );
            assert_eq!(
                SlotStyle::new(&of(SlotKind::Holding), typing, false).opacity,
                1.0
            );
        }
    }

    #[test]
    fn inside_a_view_only_a_waiting_message_keeps_strength_and_hints() {
        for kind in [SlotKind::Focused, SlotKind::Typing] {
            let style = SlotStyle::new(&of(kind), false, true);
            assert_eq!(style.opacity, o::IN_VIEW_NOT_HOLDING);
            assert!(!style.hints);
        }
        let holding = SlotStyle::new(&of(SlotKind::Holding), false, true);
        assert_eq!(holding.opacity, 1.0);
        assert!(holding.hints);
    }

    /// A failure is news you must not miss: it never fades while you type or inside a view.
    #[test]
    fn a_failure_keeps_full_strength_while_typing_and_in_a_view() {
        let mut failed = of(SlotKind::Recent);
        failed.failed = true;
        for (typing, in_view) in [(false, false), (true, false), (false, true), (true, true)] {
            assert_eq!(SlotStyle::new(&failed, typing, in_view).opacity, 1.0);
        }
    }

    #[test]
    fn only_waiting_and_typing_lines_draw_their_state_words() {
        let holding = SlotLine::new(SlotKind::Holding, "Runs in 8s", "Empty Trash");
        assert_eq!(holding.state_words(), Some("Runs in 8s"));
        let typing = SlotLine::new(SlotKind::Typing, "Typing into", "Mail");
        assert_eq!(typing.state_words(), Some("Typing into"));
        for (kind, label) in [(SlotKind::Recent, "Last"), (SlotKind::Focused, "Behind")] {
            assert_eq!(SlotLine::new(kind, label, "x").state_words(), None);
        }
    }
}
