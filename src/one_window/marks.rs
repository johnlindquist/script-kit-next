//! The words under the sentence, drawn as marks on the input's text (S04): a bar in each word's
//! color under the run of the sentence that fills it, the text itself left as typed. Typed
//! symbols and the runs the launcher's grammars resolved are marked as they're typed; the rest
//! are the runs Jev read into the words of its top reading, where they still stand in the text.
//! A typed symbol wins its word, so Jev's run for that word isn't marked. While a symbol's list
//! owns the input Jev isn't asked, so a sentence with typed symbols marks the verb it leads with.

use std::ops::Range;
use std::sync::Arc;

use sk_one_window::marks::{compose, typed_symbols, Mark, Word};

use crate::intent::Plan;
use crate::menu_syntax::MenuSyntaxFragmentRole;
use crate::ScriptListApp;

/// The word a launcher-syntax fragment fills; `None` for the words around it.
pub(crate) fn menu_syntax_word(role: MenuSyntaxFragmentRole) -> Option<Word> {
    match role {
        MenuSyntaxFragmentRole::Prefix => Some(Word::Target),
        MenuSyntaxFragmentRole::ObjectRef => Some(Word::Subject),
        MenuSyntaxFragmentRole::Kv
        | MenuSyntaxFragmentRole::Tag
        | MenuSyntaxFragmentRole::Url
        | MenuSyntaxFragmentRole::Date
        | MenuSyntaxFragmentRole::DateRange
        | MenuSyntaxFragmentRole::Duration
        | MenuSyntaxFragmentRole::Priority
        | MenuSyntaxFragmentRole::Recurrence => Some(Word::Details),
        MenuSyntaxFragmentRole::Unresolved | MenuSyntaxFragmentRole::Subject => None,
    }
}

/// The word a completed spine segment fills.
pub(crate) fn spine_word(role: &str) -> Option<Word> {
    match role {
        "spine.context.completed" => Some(Word::Subject),
        "spine.command.completed" => Some(Word::Action),
        "spine.profile.completed" => Some(Word::Owner),
        "spine.style.completed" => Some(Word::Details),
        "spine.capture.completed" | "spine.cwd.completed" => Some(Word::Target),
        _ => None,
    }
}

/// Whether Jev's reading of `read` still speaks for `text`: the same sentence, or one that grew
/// from it or was cut back into it.
fn still_read(read: &str, text: &str) -> bool {
    let (read, text) = (read.trim(), text.trim());
    !read.is_empty() && !text.is_empty() && (text.starts_with(read) || read.starts_with(text))
}

impl ScriptListApp {
    /// Jev's top reading for `text`, when it still speaks for it and nothing but the list or the
    /// reading under → covers the input.
    pub(crate) fn one_window_jev_reading(&self, text: &str) -> Option<Arc<Plan>> {
        let covered = self
            .one_window_hosted_view()
            .is_some_and(|view| !matches!(view, super::views::HostedView::Reading(_)));
        if covered || !matches!(self.current_view, crate::AppView::ScriptList) {
            return None;
        }
        let (read, plan) = self.intent_launcher.shown_top()?;
        still_read(read, text).then(|| plan.clone())
    }

    /// The input's marks for `text`, `exact` first, as ranges with their word's color and name.
    pub(crate) fn one_window_input_marks(
        &self,
        text: &str,
        exact: Vec<Mark>,
    ) -> Vec<(Range<usize>, gpui::Hsla, String)> {
        let typed: Vec<Word> = exact
            .iter()
            .filter(|mark| {
                text.get(mark.range.clone())
                    .is_some_and(|run| run.chars().any(char::is_alphanumeric))
            })
            .chain(typed_symbols(text).iter())
            .map(|mark| mark.word)
            .collect();
        let plan = self.one_window_jev_reading(text);
        let mut read: Vec<(Word, &str)> = plan
            .as_deref()
            .map(|plan| plan.words.as_slice())
            .unwrap_or_default()
            .iter()
            .filter_map(|w| Some((super::reading::word_of(w.kind)?, w.said.as_deref()?)))
            .filter(|(word, _)| !typed.contains(word))
            .collect();
        if plan.is_none() && !typed.is_empty() && !typed.contains(&Word::Action) {
            read.extend(
                crate::intent::words::lead_verb(text).map(|verb| (Word::Action, &text[verb])),
            );
        }
        compose(text, exact, &read)
            .into_iter()
            .map(|mark| {
                (
                    mark.range,
                    gpui::rgb(mark.word.color()).into(),
                    mark.word.name().to_string(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jevs_words_follow_the_sentence_it_read_as_it_grows_or_is_cut_back() {
        assert!(still_read(
            "send what I copied",
            "send what I copied to sarah"
        ));
        assert!(still_read("send what I copied to sarah", "send what I"));
        assert!(!still_read("send what I copied", "open slack"));
        assert!(!still_read("", "open slack"));
    }

    #[test]
    fn every_grammar_role_that_names_a_word_is_marked_as_that_word() {
        assert_eq!(spine_word("spine.context.completed"), Some(Word::Subject));
        assert_eq!(spine_word("spine.command.completed"), Some(Word::Action));
        assert_eq!(spine_word("spine.profile.completed"), Some(Word::Owner));
        assert_eq!(spine_word("spine.style.completed"), Some(Word::Details));
        assert_eq!(spine_word("spine.cwd.completed"), Some(Word::Target));
        assert_eq!(spine_word("spine.capture.completed"), Some(Word::Target));
        assert_eq!(
            menu_syntax_word(MenuSyntaxFragmentRole::Prefix),
            Some(Word::Target)
        );
        assert_eq!(
            menu_syntax_word(MenuSyntaxFragmentRole::Kv),
            Some(Word::Details)
        );
        assert_eq!(menu_syntax_word(MenuSyntaxFragmentRole::Unresolved), None);
    }
}
