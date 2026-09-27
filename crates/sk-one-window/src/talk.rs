//! Talking to type, or talking to ask, in the same window.
//!
//! Holding the voice key opens a view under the same input and the heard words fill it. The list
//! is one header and two rows in a fixed order: type the words at the front app's cursor (⌘1), or
//! do them as a request (⌘2). Jev reads the words at the fourth word, at each clause end, and at
//! the final words. The first answer picks the highlighted reading and a later one may change it
//! once; a reading you picked never changes. Letting go acts on exactly the final words, and only
//! when Jev agrees with the highlight and is sure enough; otherwise the window asks once.

use crate::contract::{TALK_FIRST_READ_WORDS, TALK_SURE_P, talk_words};
use crate::enter::EnterTarget;
use crate::row::Row;
use crate::slot::{SlotKind, SlotLine};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// The words are written to a reader and paste at the front app's cursor.
    Type,
    /// The words are a request to the computer.
    Ask,
}

impl Reading {
    fn id(self) -> &'static str {
        match self {
            Reading::Type => "type",
            Reading::Ask => "ask",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Talking,
    /// Let go; waiting for Jev's answer to the final words.
    Releasing,
    /// Too close to call: nothing runs until ⌘1 or ⌘2.
    Asking,
}

/// Jev's answer for the words heard so far.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub words: String,
    pub type_p: f32,
    pub ask_p: f32,
}

impl Answer {
    pub fn pick(&self) -> Reading {
        if self.type_p >= self.ask_p {
            Reading::Type
        } else {
            Reading::Ask
        }
    }

    fn sure(&self) -> f32 {
        self.type_p.max(self.ask_p)
    }
}

/// What the host tells the window about talking.
#[derive(Clone, Debug, PartialEq)]
pub enum TalkEvent {
    /// The voice key went down over `app`, whose text field is `field` (its cursor keeps it).
    Start {
        app: String,
        field: String,
        /// Who the words would be written to, when the screen says.
        to: Option<String>,
    },
    /// Everything heard so far.
    Heard(String),
    /// Jev's reading of `words`; `ask` is what the request would run.
    Read {
        words: String,
        type_p: f32,
        ask_p: f32,
        ask: Option<Box<Row>>,
    },
    /// Jev can't read `words`.
    Unread { words: String },
    /// The voice key came up: Enter for the final words.
    Release,
}

/// What letting go does, once Jev has read the final words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Act(Reading),
    AskOnce,
    Drop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Talk {
    /// Which session this is, so each one's Enter targets are new.
    pub n: u64,
    pub app: String,
    pub field: String,
    pub to: Option<String>,
    pub heard: String,
    pub reading: Option<Reading>,
    pub switched: bool,
    pub locked: Option<Reading>,
    pub phase: Phase,
    /// The last answer applied.
    pub answer: Option<Answer>,
    /// An answer for exactly the final words.
    pub last: Option<Answer>,
    /// Jev couldn't read the final words.
    pub unread: bool,
    /// What the request reading would run.
    pub ask_row: Option<Row>,
}

pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

impl Talk {
    pub fn new(n: u64, app: String, field: String, to: Option<String>) -> Self {
        Self {
            n,
            app,
            field,
            to,
            heard: String::new(),
            reading: None,
            switched: false,
            locked: None,
            phase: Phase::Talking,
            answer: None,
            last: None,
            unread: false,
            ask_row: None,
        }
    }

    /// The reading letting go would act on now.
    pub fn current(&self) -> Option<Reading> {
        self.locked.or(self.reading)
    }

    /// New words heard. Returns the words Jev should read now, and whether they are the final
    /// words: while talking, the fourth word and each clause end after it; after letting go, the
    /// transcription catching up replaces the final words.
    pub fn hear(&mut self, text: &str) -> Option<(String, bool)> {
        match self.phase {
            Phase::Asking => return None,
            Phase::Releasing => {
                if text.trim() == self.heard.trim() {
                    return None;
                }
                self.heard = text.to_string();
                self.last = None;
                self.unread = false;
                return Some((text.to_string(), true));
            }
            Phase::Talking => {}
        }
        self.heard = text.to_string();
        let n = word_count(text);
        let clause_end = text.trim_end().ends_with(['.', ',', '?', '!', ';'])
            && !text.ends_with(char::is_whitespace);
        (n == TALK_FIRST_READ_WORDS && !text.ends_with(char::is_whitespace)
            || n > TALK_FIRST_READ_WORDS && clause_end)
            .then(|| (text.to_string(), false))
    }

    /// Jev answered. An answer for words older than the last one applied, or for words that were
    /// never heard, changes nothing; neither does one that comes while the window asks.
    pub fn answered(&mut self, answer: Answer, ask: Option<Row>) -> bool {
        if self.phase == Phase::Asking
            || !self.heard.starts_with(answer.words.trim_end())
            || self
                .answer
                .as_ref()
                .is_some_and(|a| word_count(&a.words) > word_count(&answer.words))
        {
            return false;
        }
        if ask.is_some() {
            self.ask_row = ask;
        }
        let pick = answer.pick();
        if self.locked.is_none() {
            match self.reading {
                None => self.reading = Some(pick),
                Some(r) if r != pick && !self.switched => {
                    self.reading = Some(pick);
                    self.switched = true;
                }
                _ => {}
            }
        }
        if self.phase == Phase::Releasing && answer.words.trim() == self.heard.trim() {
            self.last = Some(answer.clone());
        }
        self.answer = Some(answer);
        true
    }

    /// Jev couldn't read the words it was asked about.
    pub fn unread(&mut self, words: &str) {
        if self.phase == Phase::Releasing && words.trim() == self.heard.trim() {
            self.unread = true;
        }
    }

    /// The voice key came up. Returns the final words Jev must read, unless it already has.
    pub fn release(&mut self) -> Option<String> {
        if self.phase != Phase::Talking {
            return None;
        }
        self.phase = Phase::Releasing;
        let read = self
            .answer
            .as_ref()
            .filter(|a| a.words.trim() == self.heard.trim());
        match read {
            Some(a) => {
                self.last = Some(a.clone());
                None
            }
            None if self.heard.trim().is_empty() => None,
            None => Some(self.heard.clone()),
        }
    }

    /// What letting go does now, or `None` while Jev still reads the final words.
    pub fn decision(&self) -> Option<Decision> {
        if self.phase != Phase::Releasing {
            return None;
        }
        if self.heard.trim().is_empty() {
            return Some(Decision::Drop);
        }
        if let Some(locked) = self.locked {
            return Some(Decision::Act(locked));
        }
        if self.unread {
            return Some(Decision::AskOnce);
        }
        let last = self.last.as_ref()?;
        Some(match self.reading {
            Some(r) if last.pick() == r && last.sure() >= TALK_SURE_P => Decision::Act(r),
            _ => Decision::AskOnce,
        })
    }

    pub fn ask_once(&mut self) {
        self.phase = Phase::Asking;
    }

    /// ⌘1 or ⌘2: while asking it decides; while talking it locks the reading.
    pub fn pick(&mut self, reading: Reading) -> Option<Reading> {
        if self.phase == Phase::Asking {
            return Some(reading);
        }
        self.locked = Some(reading);
        None
    }

    pub fn header(&self) -> String {
        if self.phase == Phase::Asking {
            return talk_words::TOO_CLOSE.into();
        }
        match (self.locked, self.reading) {
            (Some(Reading::Type), _) => talk_words::PICKED_TYPING.into(),
            (Some(Reading::Ask), _) => talk_words::PICKED_REQUEST.into(),
            (None, Some(Reading::Type)) => talk_words::writing_to(self.to.as_deref()),
            (None, Some(Reading::Ask)) => talk_words::SOUNDS_LIKE_REQUEST.into(),
            (None, None) => talk_words::LISTENING.into(),
        }
    }

    fn ask_title(&self) -> String {
        self.ask_row
            .as_ref()
            .map(|r| r.title.clone())
            .unwrap_or_else(|| talk_words::DO_IT_AS_REQUEST.into())
    }

    pub fn enter(&self) -> EnterTarget {
        let (id, label) = match (self.phase, self.current()) {
            (Phase::Asking, _) => ("pick", talk_words::PICK_ONE.to_string()),
            (_, Some(Reading::Type)) => ("type", talk_words::type_into(&self.app)),
            (_, Some(Reading::Ask)) => ("ask", self.ask_title()),
            (_, None) => ("listening", talk_words::WAITS_FOR_JEV.to_string()),
        };
        EnterTarget::new(format!("voice:{}:{id}", self.n), label)
    }

    /// The two readings in their fixed order, each with Jev's likelihood.
    pub fn rows(&self) -> [Row; 2] {
        let p = |r: Reading| {
            self.answer.as_ref().map(|a| match r {
                Reading::Type => a.type_p,
                Reading::Ask => a.ask_p,
            })
        };
        let mut typing = Row::new(
            format!("talk:{}", Reading::Type.id()),
            talk_words::type_into(&self.app),
        )
        .with_icon("type")
        .with_sub(if self.field.is_empty() {
            talk_words::AT_YOUR_CURSOR.to_string()
        } else {
            format!("{} · {}", talk_words::AT_YOUR_CURSOR, self.field)
        });
        typing.p = p(Reading::Type);
        let mut asking = Row::new(format!("talk:{}", Reading::Ask.id()), self.ask_title())
            .with_icon("ask")
            .with_sub(
                self.ask_row
                    .as_ref()
                    .and_then(|r| r.sub.clone())
                    .unwrap_or_else(|| talk_words::LIKE_ONE_YOU_TYPE.into()),
            );
        asking.p = p(Reading::Ask);
        [typing, asking]
    }

    /// Which row is highlighted; none while the window asks.
    pub fn selected(&self) -> Option<usize> {
        if self.phase == Phase::Asking {
            return None;
        }
        self.current().map(|r| match r {
            Reading::Type => 0,
            Reading::Ask => 1,
        })
    }

    /// The line above the input while the typing reading leads.
    pub fn line(&self) -> Option<SlotLine> {
        (self.phase != Phase::Asking && self.current() == Some(Reading::Type)).then(|| {
            SlotLine::new(SlotKind::Typing, talk_words::TYPING_INTO, self.app.clone())
                .with_icon("type")
                .with_sub(self.field.clone())
                .with_hint("esc", "drop")
        })
    }

    /// What the ledger calls a paste.
    pub fn typed_title(&self) -> String {
        talk_words::typed_into(word_count(&self.heard), &self.app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn talk() -> Talk {
        Talk::new(
            1,
            "Mail".into(),
            "Reply to Sarah".into(),
            Some("Sarah".into()),
        )
    }

    fn answer(words: &str, type_p: f32) -> Answer {
        Answer {
            words: words.into(),
            type_p,
            ask_p: 1.0 - type_p,
        }
    }

    #[test]
    fn jev_reads_at_the_fourth_word_and_each_clause_end() {
        let mut t = talk();
        let said = "Hi Sarah, thanks for checking in. The updated timeline is attached";
        let mut asked = Vec::new();
        let mut heard = String::new();
        for word in said.split(' ') {
            if !heard.is_empty() {
                heard.push(' ');
                assert_eq!(t.hear(&heard), None, "a space asks nothing");
            }
            heard.push_str(word);
            if let Some((words, last)) = t.hear(&heard) {
                assert!(!last);
                asked.push(words);
            }
        }
        assert_eq!(
            asked,
            ["Hi Sarah, thanks for", "Hi Sarah, thanks for checking in."]
        );
    }

    #[test]
    fn the_first_answer_picks_and_a_later_one_changes_it_once() {
        let mut t = talk();
        t.hear("Remind the team the");
        t.answered(answer("Remind the team the", 0.2), None);
        assert_eq!(t.reading, Some(Reading::Ask));
        t.hear("Remind the team the review moved,");
        t.answered(answer("Remind the team the review moved,", 0.8), None);
        assert_eq!(t.reading, Some(Reading::Type), "one change");
        t.hear("Remind the team the review moved, and I'll send notes.");
        t.answered(
            answer(
                "Remind the team the review moved, and I'll send notes.",
                0.1,
            ),
            None,
        );
        assert_eq!(t.reading, Some(Reading::Type), "never a second change");
    }

    #[test]
    fn an_older_answer_changes_nothing() {
        let mut t = talk();
        t.hear("one two three four five six.");
        assert!(t.answered(answer("one two three four five six.", 0.9), None));
        assert!(!t.answered(answer("one two three four", 0.1), None));
        assert!(!t.answered(answer("words never heard", 0.1), None));
        assert_eq!(t.answer.as_ref().unwrap().type_p, 0.9);
    }

    #[test]
    fn a_picked_reading_never_changes() {
        let mut t = talk();
        t.hear("let me know if");
        t.answered(answer("let me know if", 0.2), None);
        assert_eq!(t.pick(Reading::Type), None);
        t.answered(answer("let me know if", 0.05), None);
        assert_eq!(t.current(), Some(Reading::Type));
        assert_eq!(t.header(), talk_words::PICKED_TYPING);
        assert_eq!(t.selected(), Some(0));
    }

    #[test]
    fn letting_go_acts_only_when_jev_agrees_and_is_sure() {
        let words = "Hi Sarah thanks for checking in";
        let mut sure = talk();
        sure.hear(words);
        sure.answered(answer("Hi Sarah thanks for", 0.9), None);
        assert_eq!(
            sure.release().as_deref(),
            Some(words),
            "reads the final words"
        );
        assert_eq!(sure.decision(), None, "waits for them");
        sure.answered(answer(words, 0.8), None);
        assert_eq!(sure.decision(), Some(Decision::Act(Reading::Type)));

        let mut close = talk();
        close.hear(words);
        close.answered(answer(words, 0.6), None);
        assert_eq!(close.release(), None, "already read");
        assert_eq!(
            close.decision(),
            Some(Decision::AskOnce),
            "60 % is too close"
        );

        let mut turned = talk();
        turned.hear(words);
        turned.answered(answer("Hi Sarah thanks for", 0.9), None);
        turned.switched = true;
        turned.release();
        turned.answered(answer(words, 0.1), None);
        assert_eq!(
            turned.decision(),
            Some(Decision::AskOnce),
            "Jev disagrees with the highlight"
        );
    }

    #[test]
    fn words_that_arrive_after_letting_go_are_the_final_words() {
        let mut t = talk();
        t.hear("Hi Sarah thanks for");
        t.answered(answer("Hi Sarah thanks for", 0.9), None);
        assert_eq!(t.release(), None, "Jev read these words already");
        assert!(t.last.is_some());
        let caught_up = "Hi Sarah thanks for checking in";
        assert_eq!(t.hear(caught_up), Some((caught_up.to_string(), true)));
        assert_eq!(
            t.decision(),
            None,
            "waits for Jev to read what was really said"
        );
        t.answered(answer(caught_up, 0.9), None);
        assert_eq!(t.decision(), Some(Decision::Act(Reading::Type)));
    }

    #[test]
    fn a_locked_reading_acts_without_asking() {
        let mut t = talk();
        t.hear("whatever this is");
        t.pick(Reading::Ask);
        t.release();
        assert_eq!(t.decision(), Some(Decision::Act(Reading::Ask)));
    }

    #[test]
    fn nothing_heard_drops() {
        let mut t = talk();
        assert_eq!(t.release(), None);
        assert_eq!(t.decision(), Some(Decision::Drop));
    }

    #[test]
    fn unreadable_final_words_ask_once() {
        let mut t = talk();
        t.hear("hmm hello there you");
        t.release();
        t.unread("hmm hello there you");
        assert_eq!(t.decision(), Some(Decision::AskOnce));
        t.ask_once();
        assert_eq!(t.enter().label, talk_words::PICK_ONE);
        assert_eq!(t.header(), talk_words::TOO_CLOSE);
        assert_eq!(t.selected(), None);
        assert_eq!(t.pick(Reading::Type), Some(Reading::Type));
    }

    #[test]
    fn rows_keep_their_order_and_the_line_names_the_app_while_typing() {
        let mut t = talk();
        assert!(t.line().is_none());
        t.hear("Hi Sarah thanks for");
        t.answered(answer("Hi Sarah thanks for", 0.85), None);
        let [typing, asking] = t.rows();
        assert_eq!(typing.title, "Type it into Mail");
        assert_eq!(typing.p, Some(0.85));
        assert_eq!(asking.title, talk_words::DO_IT_AS_REQUEST);
        let line = t.line().expect("typing line");
        assert_eq!(
            (line.label.as_str(), line.title.as_str()),
            ("Typing into", "Mail")
        );
        assert_eq!(line.sub, "Reply to Sarah");
        assert_eq!(t.enter().label, "Type it into Mail");
        assert_eq!(t.header(), "Sounds like you're writing to Sarah");
    }

    #[test]
    fn the_request_row_takes_jevs_reading_of_the_sentence() {
        let mut t = talk();
        t.hear("remind me to send");
        let reminder = Row::new(
            "jev:schedule:Remind me",
            "Remind me Friday 10:00 · Send Sarah the deck",
        )
        .with_sub("Reminders");
        t.answered(answer("remind me to send", 0.1), Some(reminder));
        assert_eq!(
            t.rows()[1].title,
            "Remind me Friday 10:00 · Send Sarah the deck"
        );
        assert_eq!(t.enter().id, "voice:1:ask");
        assert!(t.line().is_none(), "a request keeps the usual line");
    }
}
