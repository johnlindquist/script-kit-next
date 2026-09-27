//! The line above the input.
//!
//! It is always there and always one height. Its words swap in place; only its dimming eases.
//! What it says, first match wins: an override a view set, the staged subject ("Acting on"),
//! the newest message waiting to go ("Sending in 8s"), the last thing that ran ("Last", dimmed),
//! and before anything ran, what "this" means ("Focused", dimmed).

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

impl SlotKind {
    /// Focused and Last read as background; the others are live.
    pub fn dimmed(self) -> bool {
        matches!(self, SlotKind::Focused | SlotKind::Recent)
    }
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
    pub fn dimmed(&self) -> bool {
        self.kind.dimmed()
    }

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
    pub fn new(kind: SlotKind, typing: bool, in_view: bool) -> Self {
        use crate::contract::slot_opacity as o;
        if in_view && kind != SlotKind::Holding {
            return Self {
                opacity: o::IN_VIEW_NOT_HOLDING,
                hints: false,
            };
        }
        let opacity = match kind {
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

    #[test]
    fn context_is_dimmed_and_dims_further_while_typing() {
        assert_eq!(
            SlotStyle::new(SlotKind::Focused, false, false).opacity,
            o::FOCUSED
        );
        assert_eq!(
            SlotStyle::new(SlotKind::Recent, false, false).opacity,
            o::RECENT
        );
        assert_eq!(
            SlotStyle::new(SlotKind::Focused, true, false).opacity,
            o::TYPING_FOCUSED
        );
        assert_eq!(
            SlotStyle::new(SlotKind::Recent, true, false).opacity,
            o::TYPING_RECENT
        );
    }

    #[test]
    fn staged_and_holding_lines_are_full_strength() {
        for typing in [false, true] {
            assert_eq!(SlotStyle::new(SlotKind::Staged, typing, false).opacity, 1.0);
            assert_eq!(
                SlotStyle::new(SlotKind::Holding, typing, false).opacity,
                1.0
            );
        }
    }

    #[test]
    fn inside_a_view_only_a_waiting_message_keeps_strength_and_hints() {
        let focused = SlotStyle::new(SlotKind::Focused, false, true);
        assert_eq!(focused.opacity, o::IN_VIEW_NOT_HOLDING);
        assert!(!focused.hints);
        let holding = SlotStyle::new(SlotKind::Holding, false, true);
        assert_eq!(holding.opacity, 1.0);
        assert!(holding.hints);
    }
}
