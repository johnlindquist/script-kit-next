//! What `@` attached, as chips at the main input's start (after the subject chip).
//!
//! The reducer owns the chips and their keys ([`sk_one_window::window::OneWindow::context`]).
//! Here is the boundary between what the input shows and the sentence everything else reads:
//! the input holds only the words typed, and the sentence (`filter_text`, the spine parse, Jev,
//! the prompt plan) is those words with each chip's token after them, `summarize @selection `,
//! exactly as it read before the token became a chip. So a chip resolves to context the same way
//! its typed token always has.

use gpui::{Context, Window};
use sk_one_window::{ContextChip, Event, Key};

use crate::{AppView, ScriptListApp};

/// The row of chips at the main input's start, as automation measures it.
pub(crate) const INPUT_CHIPS_ID: &str = "main-input-chips";

/// The `@` token that ends right before `caret`, followed by one space: what a finished `@`
/// completion (or typing `@selection` and a space) leaves behind. Its byte range, including the
/// space.
pub(crate) fn finished_mention(text: &str, caret: usize) -> Option<std::ops::Range<usize>> {
    let before = text.get(..caret)?;
    let token_end = before.strip_suffix(' ')?.len();
    let start = before[..token_end]
        .rfind(char::is_whitespace)
        .map_or(0, |ix| {
            ix + before[ix..].chars().next().map_or(1, char::len_utf8)
        });
    let token = &before[start..token_end];
    (token.len() > 1 && token.starts_with('@')).then_some(start..caret)
}

/// The sentence for the input's words and the chips' tokens.
pub(crate) fn compose(visible: &str, tokens: &str) -> String {
    if tokens.is_empty() {
        visible.to_string()
    } else if visible.is_empty() || visible.ends_with(char::is_whitespace) {
        format!("{visible}{tokens}")
    } else {
        format!("{visible} {tokens}")
    }
}

/// Where the spine reads the caret in the sentence for a caret in the input's words. The words
/// start the sentence, so a caret inside them is the same offset; once the words are finished
/// (empty, or ending in a space) the caret stands after the chips' tokens, as if they had been
/// typed there, so no token becomes the segment being completed.
pub(crate) fn spine_caret(visible: &str, caret: usize, sentence: &str) -> usize {
    let finished = visible.is_empty() || visible.ends_with(char::is_whitespace);
    if caret >= visible.len() && finished {
        sentence.len()
    } else {
        caret.min(visible.len())
    }
}

impl ScriptListApp {
    /// The chips' tokens as the sentence carries them, each followed by a space; empty when the
    /// list isn't showing or nothing is attached.
    fn one_window_chip_tokens(&self) -> String {
        if !matches!(self.current_view, AppView::ScriptList) || self.one_window.in_view() {
            return String::new();
        }
        self.one_window
            .context()
            .iter()
            .map(|chip| format!("{} ", chip.token))
            .collect()
    }

    /// The sentence the input's words stand for.
    pub(crate) fn one_window_sentence(&self, visible: &str) -> String {
        compose(visible, &self.one_window_chip_tokens())
    }

    /// The input's words for a sentence that ends with the chips' tokens; `None` when it doesn't.
    /// `compose` puts a space between words and tokens unless the words already end with one,
    /// so "tile" and "tile " give the same sentence: the input's own value says which it is, and
    /// otherwise the words are taken without the joining space.
    pub(crate) fn one_window_visible_words(
        &self,
        sentence: &str,
        cx: &gpui::App,
    ) -> Option<String> {
        let tokens = self.one_window_chip_tokens();
        if tokens.is_empty() {
            return Some(sentence.to_string());
        }
        let rest = sentence.strip_suffix(tokens.as_str())?;
        let Some(bare) = rest
            .strip_suffix(' ')
            .filter(|words| !words.is_empty() && !words.ends_with(char::is_whitespace))
        else {
            return Some(rest.to_string());
        };
        let input = self.gpui_input_state.read(cx).value();
        Some(if input.as_ref() == rest { rest } else { bare }.to_string())
    }

    /// A chip for `token` when it names context: a built-in (`@selection`) or something a
    /// `@source:` search attached (`@file:notes.md`).
    fn one_window_context_chip(&self, token: &str) -> Option<ContextChip> {
        if let Some((title, icon)) = crate::spine::builtin_context_chip(token) {
            return Some(ContextChip {
                token: token.to_string(),
                title: title.to_string(),
                icon: icon.to_string(),
            });
        }
        let part = self.spine_mention_aliases.get(token)?;
        Some(ContextChip {
            token: token.to_string(),
            title: part.label().to_string(),
            icon: crate::spine::subsearch_chip_icon(token).to_string(),
        })
    }

    /// Takes each finished `@` token before the caret out of `text` and attaches it as a chip.
    /// Returns the words and caret left, when anything was taken.
    pub(crate) fn one_window_take_chips(
        &mut self,
        text: &str,
        caret: usize,
        cx: &mut Context<Self>,
    ) -> Option<(String, usize)> {
        if !matches!(self.current_view, AppView::ScriptList) || self.one_window.in_view() {
            return None;
        }
        let mut text = text.to_string();
        let mut caret = caret.min(text.len());
        let mut taken = false;
        while let Some(range) = finished_mention(&text, caret) {
            let token = text[range.start..range.end - 1].to_string();
            let Some(chip) = self.one_window_context_chip(&token) else {
                break;
            };
            self.one_window_handle(Event::Attach(chip), cx);
            text.replace_range(range.clone(), "");
            caret = range.start;
            taken = true;
        }
        taken.then_some((text, caret))
    }

    /// Words set wholesale (a view closing, `setFilter`, a completion) as the sentence and the
    /// input's words: a finished `@` token becomes a chip, a sentence that still ends with the
    /// chips' tokens keeps them, and any other sentence lets them go. Returns the sentence, the
    /// input's words and the caret in them.
    pub(crate) fn one_window_split_set_text(
        &mut self,
        text: String,
        caret: usize,
        cx: &mut Context<Self>,
    ) -> (String, String, usize) {
        if !matches!(self.current_view, AppView::ScriptList) || self.one_window.in_view() {
            return (text.clone(), text, caret);
        }
        if self.one_window.context().is_empty() {
            return match self.one_window_take_chips(&text, caret, cx) {
                Some((visible, caret)) => (self.one_window_sentence(&visible), visible, caret),
                None => (text.clone(), text, caret),
            };
        }
        let tokens = self.one_window_chip_tokens();
        let Some(rest) = text.strip_suffix(tokens.as_str()).map(str::to_string) else {
            self.one_window_handle(Event::DropChips, cx);
            return match self.one_window_take_chips(&text, caret, cx) {
                Some((visible, caret)) => (self.one_window_sentence(&visible), visible, caret),
                None => (text.clone(), text, caret),
            };
        };
        // A completion next to the chips gets no space of its own (the chips' tokens already
        // start after one), so the caret stops right before the space that finishes it.
        let finishing = if caret + 1 == rest.len() && rest.ends_with(' ') {
            rest.len()
        } else {
            caret.min(rest.len())
        };
        if let Some((visible, caret)) = self.one_window_take_chips(&rest, finishing, cx) {
            return (self.one_window_sentence(&visible), visible, caret);
        }
        let visible = self
            .one_window_visible_words(&text, cx)
            .unwrap_or_else(|| rest.clone());
        let caret = caret.min(visible.len());
        (text, visible, caret)
    }

    /// ⌫ and ← with the caret at the input's start, and → and Esc while a chip is highlighted,
    /// go to the chips. When a chip went, the sentence follows the input again.
    pub(crate) fn one_window_chip_key(
        &mut self,
        key: &str,
        modifiers: &gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::ScriptList)
            || self.one_window.in_view()
            || modifiers.platform
            || modifiers.alt
            || modifiers.control
            || modifiers.shift
        {
            return false;
        }
        let highlighted = self.one_window.chip_at().is_some();
        let at_start = {
            let input = self.gpui_input_state.read(cx);
            input.selected_range().is_empty() && input.cursor() == 0
        };
        let key = match key.to_ascii_lowercase().as_str() {
            "backspace" if at_start || highlighted => Key::Backspace,
            "left" | "arrowleft" if at_start || highlighted => Key::Left,
            "right" | "arrowright" if highlighted => Key::Right,
            "escape" | "esc" if highlighted => Key::Escape,
            // Typing, ↑ and ↓ reach the reducer on their own paths and put the caret back in
            // the words there.
            _ => return false,
        };
        let chips = |app: &Self| {
            app.one_window.context().len() + usize::from(app.one_window.staged().is_some())
        };
        let before = chips(self);
        if !self.one_window_key(key, cx).taken() {
            return false;
        }
        if chips(self) != before {
            self.handle_filter_input_change(window, cx);
        }
        cx.notify();
        true
    }

    /// The chips at the input's start: the attached subject, then what `@` attached, with the
    /// one ← stands on highlighted. `None` when there are none.
    #[allow(dead_code)] // The binary's main render draws it.
    pub(crate) fn render_input_chips(
        &self,
        def: crate::designs::MainMenuThemeDef,
    ) -> Option<gpui::AnyElement> {
        use gpui::{div, px, InteractiveElement, IntoElement, ParentElement, Styled};
        let subject = self.one_window.chip().filter(|chip| chip.attached);
        let context =
            if matches!(self.current_view, AppView::ScriptList) && !self.one_window.in_view() {
                self.one_window.context()
            } else {
                &[]
            };
        if subject.is_none() && context.is_empty() {
            return None;
        }
        let tokens = def.slot_line.chip;
        let at = self.one_window.chip_at();
        let offset = usize::from(subject.is_some());
        let mut row = div()
            .debug_selector(|| INPUT_CHIPS_ID.to_string())
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(tokens.between_px))
            .mr(px(tokens.attached_margin_px));
        if let Some(chip) = subject {
            row = row.child(super::slot_line::render_subject_chip(
                &self.theme,
                def,
                &chip,
                at == Some(0),
            ));
        }
        for (index, chip) in context.iter().enumerate() {
            row = row.child(super::slot_line::render_context_chip(
                &self.theme,
                def,
                index,
                chip,
                at == Some(index + offset),
            ));
        }
        Some(row.into_any_element())
    }
}
