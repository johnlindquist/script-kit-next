//! What the window learns between queries.
//!
//! A sentence whose row ran and finished becomes an instant match: typing the start of it again
//! lists that row, marked learned, strong enough to own Enter before Jev answers. A lesson
//! learned while words are typed waits until the input is empty, so learning never changes the
//! list under the cursor.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::contract::{
    LEARNED_MIN_CHARS, LEARNED_SCORE_FLOOR, LEARNED_SENTENCE_SCORE, MAX_LESSONS,
};
use crate::row::{Row, RowAct};

/// A sentence and the row it ran.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub said: String,
    pub row: Row,
    pub at: Ms,
}

impl Lesson {
    /// The row the list shows for this lesson.
    pub fn key(&self) -> String {
        learned_key(&self.row.title)
    }
}

pub fn learned_key(title: &str) -> String {
    format!("learned:{title}")
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Lessons {
    learned: Vec<Lesson>,
    /// Learned while words were typed; offered once the input is empty.
    #[serde(skip)]
    pending: Vec<Lesson>,
    /// Times Space staged what's focused, counted up to when its hint stops teaching.
    #[serde(default)]
    grabs: u32,
}

impl Lessons {
    pub fn new(learned: Vec<Lesson>) -> Self {
        Self {
            learned,
            pending: Vec::new(),
            grabs: 0,
        }
    }

    /// Space staged what's focused. Returns whether that's worth saving: the hint still teaches.
    pub fn grabbed(&mut self) -> bool {
        if self.grabs >= crate::contract::USE_HINT_GRABS {
            return false;
        }
        self.grabs += 1;
        true
    }

    /// Whether the hint beside Space still says what it does.
    pub fn teaches_space(&self) -> bool {
        self.grabs < crate::contract::USE_HINT_GRABS
    }

    /// Lessons the list can offer, newest first.
    pub fn learned(&self) -> &[Lesson] {
        &self.learned
    }

    /// `row` ran for `said`. A sentence of two words or more is worth learning; a learned row that
    /// ran again, a fallback, or something the window did itself teaches nothing.
    pub fn learn(&mut self, now: Ms, said: &str, row: &Row) -> bool {
        let said = said.trim();
        if said.split_whitespace().count() < 2
            || row.learned
            || row.fallback
            || row.act != RowAct::Host
            || row.title.trim().is_empty()
            || [
                "learned:",
                "down:",
                "act:",
                "talk:",
                "fallback-",
                crate::contract::ALL_RESULTS_KEY,
            ]
            .iter()
            .any(|p| row.key.starts_with(p))
        {
            return false;
        }
        let mut row = row.clone();
        row.head = None;
        row.p = None;
        self.pending.retain(|l| l.row.title != row.title);
        self.pending.insert(
            0,
            Lesson {
                said: said.to_string(),
                row,
                at: now,
            },
        );
        true
    }

    /// The input is empty: what was learned during the last query can be offered now.
    pub fn between_queries(&mut self) -> bool {
        if self.pending.is_empty() {
            return false;
        }
        for lesson in std::mem::take(&mut self.pending).into_iter().rev() {
            self.learned.retain(|l| l.row.title != lesson.row.title);
            self.learned.insert(0, lesson);
        }
        self.learned.truncate(MAX_LESSONS);
        true
    }

    pub fn get(&self, key: &str) -> Option<&Lesson> {
        self.learned.iter().find(|l| l.key() == key)
    }

    /// Learned rows for the typed words: every lesson whose sentence starts with them, word by
    /// word, scored by how much of the sentence is typed, and above a name once two of its words
    /// are.
    pub fn rows(&self, text: &str) -> Vec<Row> {
        let typed = text.trim_start().to_lowercase();
        if typed.trim().chars().count() < LEARNED_MIN_CHARS {
            return Vec::new();
        }
        let sentence = typed.split_whitespace().count() >= 2;
        let mut rows: Vec<Row> = self
            .learned
            .iter()
            .filter_map(|lesson| {
                let said = lesson.said.to_lowercase();
                starts_word_by_word(&said, &typed).then(|| {
                    let share = typed.trim().chars().count() as f32 / said.chars().count() as f32;
                    let floor = if sentence {
                        LEARNED_SENTENCE_SCORE
                    } else {
                        LEARNED_SCORE_FLOOR
                    };
                    let score = floor as f32 + (100 - floor) as f32 * share.min(1.0);
                    let mut row = lesson.row.clone();
                    row.key = lesson.key();
                    row.learned = true;
                    row.score = score.round() as u8;
                    row.sub = Some(format!("you said “{}”", lesson.said));
                    row
                })
            })
            .collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.score));
        rows
    }
}

/// Whether `typed` is the start of `said`: every whole typed word matches, and the word being
/// typed starts the next one.
fn starts_word_by_word(said: &str, typed: &str) -> bool {
    let mut said_words = said.split_whitespace();
    let mut typed_words = typed.split_whitespace().peekable();
    let open = !typed.ends_with(char::is_whitespace);
    while let Some(word) = typed_words.next() {
        let Some(next) = said_words.next() else {
            return false;
        };
        let last = typed_words.peek().is_none();
        let fits = if last && open {
            next.starts_with(word)
        } else {
            next == word
        };
        if !fits {
            return false;
        }
    }
    true
}

/// Host names and learned rows in one list, strongest first; a learned row replaces a host
/// name with the same title.
pub fn merge(names: Vec<Row>, learned: Vec<Row>) -> Vec<Row> {
    let mut out: Vec<Row> = names
        .into_iter()
        .filter(|n| {
            n.is_open_tab()
                || n.key.starts_with("app/")
                || !learned
                    .iter()
                    .any(|l| l.title.eq_ignore_ascii_case(&n.title))
        })
        .collect();
    for row in learned {
        let at = out
            .iter()
            .position(|n| {
                !n.fallback
                    && (n.score < row.score
                        || (row.learned && row.score == 100 && !n.learned && n.score == row.score))
            })
            .or_else(|| out.iter().position(|n| n.fallback))
            .unwrap_or(out.len());
        out.insert(at, row);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chrome() -> Row {
        Row::new(
            "jev:Move Google Chrome to Next Display",
            "Move Google Chrome to Next Display",
        )
        .with_icon("window")
    }

    fn taught(said: &str) -> Lessons {
        let mut l = Lessons::default();
        assert!(l.learn(0, said, &chrome()));
        l.between_queries();
        l
    }

    #[test]
    fn a_sentence_that_ran_matches_from_its_first_letters() {
        let l = taught("put the pr on my second screen");
        assert!(l.rows("p").is_empty(), "one letter is too little");
        for typed in [
            "pu",
            "put",
            "put ",
            "put t",
            "put the",
            "put the pr on my second screen",
        ] {
            let rows = l.rows(typed);
            assert_eq!(rows.len(), 1, "{typed:?}");
            assert_eq!(rows[0].key, "learned:Move Google Chrome to Next Display");
            assert!(rows[0].learned && rows[0].is_strong());
        }
        assert!(l.rows("put a").is_empty());
        assert!(l.rows("pull").is_empty());
        assert!(l.rows("put the pr on my second screen now").is_empty());
    }

    #[test]
    fn more_of_the_sentence_scores_higher() {
        let l = taught("put the pr on my second screen");
        assert!(l.rows("put t")[0].score > l.rows("pu")[0].score);
        assert!(
            l.rows("put t")[0].score >= LEARNED_SENTENCE_SCORE,
            "two words of a learned sentence outrank a name that starts with them"
        );
        assert!(l.rows("put")[0].score < LEARNED_SENTENCE_SCORE);
    }

    #[test]
    fn nothing_learned_shows_until_the_input_is_empty() {
        let mut l = Lessons::default();
        l.learn(0, "put the pr on my second screen", &chrome());
        assert!(l.rows("put").is_empty(), "the lesson waits");
        assert!(l.between_queries());
        assert_eq!(l.rows("put").len(), 1);
    }

    #[test]
    fn a_word_is_no_sentence() {
        let mut l = Lessons::default();
        assert!(!l.learn(0, "chrome", &chrome()));
        assert!(!l.learn(0, "  ", &chrome()));
        let learned = Row {
            learned: true,
            ..chrome()
        };
        assert!(!l.learn(0, "put the pr", &learned));
        assert!(!l.learn(0, "undo that now", &chrome().with_act(RowAct::Undo)));
    }

    #[test]
    fn the_newest_sentence_for_a_row_wins() {
        let mut l = taught("put the pr on my second screen");
        l.learn(1, "send chrome to the other display", &chrome());
        l.between_queries();
        assert_eq!(l.learned().len(), 1);
        assert!(l.rows("put").is_empty());
        assert_eq!(l.rows("send").len(), 1);
    }

    #[test]
    fn merged_rows_keep_strength_order() {
        let names = vec![
            Row::new("sleep", "Put Mac to Sleep").with_score(95),
            Row::new("tab", "Select Previous Tab").with_score(27),
            Row::new("fallback-ai:x", "Ask AI").fallback(),
        ];
        let learned = taught("put the pr on my second screen").rows("pu");
        let merged = merge(names, learned);
        let keys: Vec<&str> = merged.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "sleep",
                "learned:Move Google Chrome to Next Display",
                "tab",
                "fallback-ai:x"
            ]
        );
    }

    #[test]
    fn a_learned_row_replaces_the_same_name() {
        let names =
            vec![Row::new("name/chrome", "Move Google Chrome to Next Display").with_score(30)];
        let merged = merge(names, taught("put the pr on my second screen").rows("put"));
        assert_eq!(merged.len(), 1);
        assert!(merged[0].learned);
    }

    #[test]
    fn a_learned_sentence_keeps_its_namesake_application_reachable() {
        let lesson = taught("open notes");
        let same_title = Row::new("app/notes", chrome().title).with_score(100);
        let rows = merge(vec![same_title], lesson.rows("open notes"));
        assert!(rows[0].learned);
        assert_eq!(rows[1].key, "app/notes");
    }

    #[test]
    fn lessons_survive_a_restart() {
        let l = taught("put the pr on my second screen");
        let json = serde_json::to_string(&l).unwrap();
        let back: Lessons = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rows("put").len(), 1);
    }
}
