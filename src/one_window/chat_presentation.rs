//! Local paint state, never a second transcript. Keep parser/selection identities stable while
//! deltas append, even when tool cards appear before the reply in the reducer's turn list.
use crate::ScriptListApp;
use gpui::{AppContext as _, Context, Entity, EntityId, ScrollHandle};
use gpui_component::text::TextViewState;
use sk_one_window::chat::{Chat, Who};

#[derive(Default)]
pub(super) struct ChatPresentation {
    owner: Option<EntityId>,
    answers: Vec<(String, Entity<TextViewState>)>,
    pub transcript: ScrollHandle,
    pub choices: ScrollHandle,
    last_turn: Option<usize>,
    last_choice: Option<(String, usize)>,
}

impl ChatPresentation {
    pub fn reconcile(
        &mut self,
        owner: Option<EntityId>,
        chat: &Chat,
        cx: &mut Context<ScriptListApp>,
    ) -> Vec<Entity<TextViewState>> {
        if self.owner != owner {
            *self = Self {
                owner,
                ..Self::default()
            };
        }
        let mut count = 0;
        for turn in chat.turns.iter().filter(|t| t.who == Who::Model) {
            let source = turn.text.as_str();
            if let Some((old, state)) = self.answers.get_mut(count) {
                if old != source {
                    if let Some(delta) = source.strip_prefix(old.as_str()) {
                        state.update(cx, |s, cx| s.push_str(delta, cx));
                        old.push_str(delta);
                    } else {
                        state.update(cx, |s, cx| s.set_text(source, cx));
                        old.clear();
                        old.push_str(source);
                    }
                }
            } else {
                self.answers.push((
                    source.to_owned(),
                    cx.new(|cx| TextViewState::markdown_immediate(source, cx)),
                ));
            }
            count += 1;
        }
        self.answers.truncate(count);
        // Reveal the beginning of a new turn. Never jump again just because a token arrived,
        // the user selected some prose, or a spinner requested another frame.
        let last = chat.turns.len().checked_sub(1);
        if self.last_turn != last {
            if let Some(ix) = last {
                self.transcript.scroll_to_item(ix);
            }
            self.last_turn = last;
        }
        self.answers
            .iter()
            .map(|(_, state)| state.clone())
            .collect()
    }
    pub fn reveal_choice(&mut self, head: Option<&str>, index: usize) {
        let key = head.map(|h| (h.to_owned(), index));
        if self.last_choice != key {
            self.choices.scroll_to_item(index);
            self.last_choice = key;
        }
    }
}
