//! A sentence typed without a pause: what ↵ says after every key, and which words are marked.

use proptest::prelude::*;
use sk_one_window::Row;
use sk_one_window::contract::READING_SETTLE_MS;
use sk_one_window::list::Item;
use sk_one_window::window::{Event, OneWindow};

use super::{JEV, NAMES, Run, WORDS};

/// One key of a sentence typed without a pause: how long after the last key it came (under
/// [`READING_SETTLE_MS`]), whether a name that owned Enter earlier in the sentence is listed again
/// (a keyword or alias hit), and whether Jev answered before the next key (for these words, or for
/// older ones). Jev answers the whole sentence once, after its last key.
#[derive(Clone, Debug)]
struct Keystroke {
    gap: u64,
    echo: bool,
    jev: Option<(bool, u64, Vec<usize>)>,
}

/// The file's words that no other word extends, typed one key at a time.
fn sentences() -> Vec<&'static str> {
    WORDS
        .iter()
        .copied()
        .filter(|w| !w.is_empty() && !WORDS.iter().any(|o| o.len() > w.len() && o.starts_with(w)))
        .collect()
}

/// The names a search lists for `text`, strongest first: a whole name, its start, the start of
/// one of its words, anywhere in it, or its letters in order (a fuzzy near-miss, too weak to own
/// Enter).
fn search(text: &str) -> Vec<Row> {
    let t = text.to_lowercase();
    let mut found: Vec<(u8, &str)> = NAMES
        .iter()
        .filter_map(|name| {
            let n = name.to_lowercase();
            let mut letters = n.chars();
            let score = if n == t {
                100
            } else if n.starts_with(&t) {
                95
            } else if n.split_whitespace().any(|w| w.starts_with(&t)) {
                90
            } else if n.contains(&t) {
                80
            } else if t
                .chars()
                .filter(|c| !c.is_whitespace())
                .all(|c| letters.any(|l| l == c))
            {
                40
            } else {
                return None;
            };
            Some((score, *name))
        })
        .collect();
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
    found
        .into_iter()
        .take(3)
        .map(|(score, name)| Row::new(format!("name:{name}"), name).with_score(score))
        .collect()
}

fn keystroke() -> impl Strategy<Value = Keystroke> {
    (
        1..READING_SETTLE_MS,
        prop::bool::weighted(0.2),
        prop::option::of((
            any::<bool>(),
            0u64..100,
            prop::collection::vec(0..JEV.len(), 0..3),
        )),
    )
        .prop_map(|(gap, echo, jev)| {
            let jev = jev.map(|(current, at, rows)| (current, at.min(gap - 1), rows));
            Keystroke { gap, echo, jev }
        })
}

/// What ↵ said after every page turn of a typed sentence, and why each change was allowed.
struct Typist {
    win: OneWindow,
    now: u64,
    wake: Option<u64>,
    last_key: u64,
    labels: Vec<String>,
    owner: Option<String>,
    /// Every name that owned Enter in the sentence.
    owners: Vec<String>,
}

impl Typist {
    fn feed(&mut self, ev: Event) {
        self.win.handle(self.now, ev);
        self.wake = self.win.next_wake();
    }

    fn until(&mut self, t: u64) {
        while let Some(at) = self.wake.filter(|at| *at <= t) {
            self.now = at;
            self.wake = None;
            self.feed(Event::Tick);
            self.look();
        }
        self.now = t;
    }

    /// The thing that owns Enter: a name by its key, or Jev's place.
    fn owner_now(&self) -> Option<String> {
        let list = self.win.list();
        match list.current()? {
            item @ Item::Name(_) => list.row(&item).map(|r| r.key.clone()),
            Item::Slot(_) => Some("jev".into()),
            _ => None,
        }
    }

    fn look(&mut self) {
        let label = self.win.enter_target(self.now).label;
        let owner = self.owner_now();
        if self.labels.last() != Some(&label) {
            let list = self.win.list();
            let settled_reading = matches!(list.current(), Some(Item::Slot(_)))
                && list.jev.settled_for(self.win.text())
                && self.now >= self.last_key + READING_SETTLE_MS
                && list.current_row(self.now).is_some_and(|r| r.title == label);
            assert!(
                self.labels.is_empty() || owner != self.owner || settled_reading,
                "↵ went from {:?} to {label:?} at {} without a stronger name or a settled reading",
                self.labels.last(),
                self.now
            );
            assert!(
                !self.labels.contains(&label),
                "↵ went back to {label:?} after {:?}",
                self.labels
            );
            self.labels.push(label);
        }
        if let Some(key) = owner.as_ref().filter(|k| k.starts_with("name:")) {
            if !self.owners.contains(key) {
                self.owners.push(key.clone());
            }
        }
        self.owner = owner;
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn the_enter_button_never_flickers_while_a_sentence_is_typed(
        pick in 0..sentences().len(),
        keys in prop::collection::vec(keystroke(), 40),
        final_wait in 0u64..600,
        final_rows in prop::collection::vec(0..JEV.len(), 1..3),
    ) {
        let sentence = sentences()[pick];
        let mut t = Typist {
            win: Run::new().win,
            now: 0,
            wake: None,
            last_key: 0,
            labels: Vec::new(),
            owner: None,
            owners: Vec::new(),
        };
        let prefixes: Vec<&str> = sentence
            .char_indices()
            .map(|(i, c)| &sentence[..i + c.len_utf8()])
            .collect();
        for (i, (text, key)) in prefixes.iter().zip(keys.iter().cycle()).enumerate() {
            t.until(t.last_key + key.gap);
            t.last_key = t.now;
            t.feed(Event::Text(text.to_string()));
            let mut names = search(text);
            if let Some(key) = t.owners.first().filter(|k| key.echo && !names.iter().any(|n| &n.key == *k)) {
                let name = key.trim_start_matches("name:");
                names.push(Row::new(key.clone(), name).with_score(70));
                names.sort_by_key(|n| std::cmp::Reverse(n.score));
            }
            t.feed(Event::Names(names));
            t.look();
            if let Some((current, at, rows)) = key.jev.as_ref().filter(|_| i + 1 < prefixes.len()) {
                t.until(t.last_key + at);
                let text = if *current || i == 0 { text.to_string() } else { prefixes[i - 1].to_string() };
                let rows = rows.iter().map(|j| Row::new(format!("jev:{}", JEV[*j]), JEV[*j])).collect();
                t.feed(Event::Jev { text, rows });
                t.look();
            }
        }
        t.until(t.now + final_wait);
        let rows: Vec<Row> = final_rows.iter().map(|j| Row::new(format!("jev:{}", JEV[*j]), JEV[*j])).collect();
        t.feed(Event::Jev { text: sentence.to_string(), rows: rows.clone() });
        t.look();
        t.until(t.now + 2_000);
        if let Some(row) = t.win.list().current_row(t.now).filter(|_| t.owner.as_deref() == Some("jev")) {
            prop_assert_eq!(
                t.labels.last(),
                Some(&row.title),
                "once the keys pause, ↵ says the reading Enter runs: {:?}",
                t.labels
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn marks_never_guess_and_never_overlap(
        pick in 0..sentences().len(),
        typed in 0usize..48,
        reads in prop::collection::vec((0usize..5, 0usize..8, 1usize..4), 0..4),
    ) {
        use sk_one_window::marks::{Word, marks_for};
        let sentence = sentences()[pick];
        let words: Vec<&str> = sentence.split(' ').collect();
        let kinds = [Word::Subject, Word::Action, Word::Target, Word::Details, Word::Owner];
        let phrases: Vec<(Word, String)> = reads
            .iter()
            .filter(|(_, from, _)| *from < words.len())
            .map(|(k, from, n)| (kinds[*k], words[*from..(*from + n).min(words.len())].join(" ")))
            .collect();
        let read: Vec<(Word, &str)> = phrases.iter().map(|(w, p)| (*w, p.as_str())).collect();
        let end = sentence.char_indices().nth(typed).map_or(sentence.len(), |(i, _)| i);
        let text = &sentence[..end];
        let marks = marks_for(text, &read);
        for pair in marks.windows(2) {
            prop_assert!(pair[0].range.end <= pair[1].range.start, "marks overlap: {marks:?}");
        }
        for mark in &marks {
            let marked = &text[mark.range.clone()];
            let symbol = marked.starts_with('@') || marked.starts_with(">:") || marked.starts_with('/') || marked.ends_with(';');
            prop_assert!(
                symbol || read.iter().any(|(w, p)| *w == mark.word && p.eq_ignore_ascii_case(marked)),
                "{marked:?} in {text:?} is marked though Jev didn't read it as {:?}",
                mark.word
            );
        }
    }
}
