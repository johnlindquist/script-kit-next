//! Sentences that ran become instant matches, offered from the next query on.

use super::*;
use crate::lessons::{Lesson, Lessons};

impl OneWindow {
    /// Starts with what was learned before this session.
    pub fn with_lessons(mut self, lessons: Lessons) -> Self {
        self.lessons = lessons;
        self
    }

    pub fn lessons(&self) -> &Lessons {
        &self.lessons
    }

    /// Learned rows for these words, for the host to list among its name matches.
    pub fn learned_rows(&self, text: &str) -> Vec<Row> {
        self.lessons.rows(text)
    }

    /// The lesson behind a learned row.
    pub fn lesson(&self, key: &str) -> Option<&Lesson> {
        self.lessons.get(key)
    }

    /// A row that ran and finished on the spot teaches its sentence; one that opened something
    /// teaches nothing.
    pub(super) fn learn_from(&mut self, now: Ms, said: &str, row: &Row) {
        self.lessons.learn(now, said, row);
    }

    /// Between queries, what was learned in the last one becomes a match.
    pub(super) fn settle_lessons(&mut self, fx: &mut Vec<Effect>) {
        if self.text.trim().is_empty() && self.lessons.between_queries() {
            fx.push(Effect::Learned);
        }
    }
}
