//! Jev's reading of the words, drawn as marks on the input's text: the reducer's `marks` rule
//! decides which runs are marked, from the phrases Jev read into each word of its top reading.

use std::ops::Range;

use sk_one_window::marks::{marks_for, Word};

use crate::intent::Plan;
use crate::ScriptListApp;

/// The phrases a reading filled its words with, verbatim from the sentence.
fn read_words(plan: &Plan) -> Vec<(Word, &str)> {
    let slots = &plan.slots;
    [
        (Word::Subject, slots.task.as_deref()),
        (Word::Subject, slots.query.as_deref()),
        (Word::Details, slots.when.as_deref()),
        (Word::Details, slots.body.as_deref()),
        (Word::Details, slots.shortcut.as_deref()),
        (Word::Action, slots.script_steps.as_deref()),
    ]
    .into_iter()
    .filter_map(|(word, phrase)| phrase.map(|p| (word, p)))
    .collect()
}

impl ScriptListApp {
    /// The input's marks for `text`, as highlight ranges with their word's color and name.
    pub(crate) fn one_window_input_marks(
        &self,
        text: &str,
    ) -> Vec<(Range<usize>, gpui::Hsla, String)> {
        if !self.one_window_owns(text) {
            return Vec::new();
        }
        let list = self.one_window.list();
        let plan = (0..list.jev.len())
            .filter_map(|i| list.jev.get(i))
            .find_map(|slot| slot.row.as_ref())
            .and_then(|row| self.intent_launcher.shown_plan(&row.key));
        let read = plan.as_deref().map(read_words).unwrap_or_default();
        marks_for(text, &read)
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
