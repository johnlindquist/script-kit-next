//! Jev as recorded, for owned UI evaluation: the app never reaches the network there, so the
//! launcher reads sentences against the screens and catalog Jev was asked about, with the answers
//! Jev gave. `replay.json` holds the prototype's recorded requests; `evaluation.json` the ones the
//! frames chapters type, including every word prefix of the sentences typed a word at a time.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use serde::Deserialize;

use super::jev::{
    Answer, Answers, JevError, JevResult, Oracle, Probabilities, Question, Questions,
};
use super::{Catalog, ScreenContext};

const REPLAY: &str = include_str!("fixtures/replay.json");
const EVALUATION: &str = include_str!("fixtures/evaluation.json");

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum Compact {
    Noul(f64),
    Choice(String, Probabilities),
    Score(f64, Probabilities),
}

impl Compact {
    pub(crate) fn expand(&self) -> Answer {
        match self {
            Compact::Noul(noul) => Answer::Noul { noul: *noul },
            Compact::Choice(choice, probabilities) => Answer::Choice {
                choice: choice.clone(),
                confidence: 0.0,
                probabilities: probabilities.clone(),
            },
            Compact::Score(score, probabilities) => Answer::Score {
                score: *score,
                confidence: 0.0,
                probabilities: probabilities.clone(),
            },
        }
    }
}

pub(crate) type RecordedAnswers = HashMap<String, HashMap<String, Compact>>;

#[derive(Deserialize)]
struct ReplayCase {
    ctx: String,
    request: String,
    answers: RecordedAnswers,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Replay {
    catalog: Catalog,
    contexts: HashMap<String, ScreenContext>,
    cases: Vec<ReplayCase>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct Sentence {
    pub(crate) ctx: String,
    pub(crate) request: String,
    /// Typed a word at a time, so Jev reads every word prefix too.
    #[serde(default)]
    pub(crate) prefixes: bool,
}

#[derive(Deserialize)]
pub(crate) struct Evaluation {
    pub(crate) sentences: Vec<Sentence>,
    #[serde(default)]
    pub(crate) answers: RecordedAnswers,
}

pub(crate) fn evaluation() -> Evaluation {
    serde_json::from_str(EVALUATION).unwrap_or(Evaluation {
        sentences: Vec::new(),
        answers: HashMap::new(),
    })
}

/// Every request each evaluation sentence has Jev read: the sentence, and its word prefixes when
/// it's typed a word at a time.
pub(crate) fn evaluation_requests(sentence: &Sentence) -> Vec<String> {
    let words: Vec<&str> = sentence.request.split(' ').collect();
    if !sentence.prefixes {
        return vec![sentence.request.clone()];
    }
    (1..=words.len()).map(|n| words[..n].join(" ")).collect()
}

/// First-round and clause-round answers for the same words differ, so the key says which.
pub(crate) fn answer_key(state: &serde_json::Value, questions: &Questions) -> String {
    let request = state["request"].as_str().unwrap_or_default();
    let first_round = matches!(
        questions.get("pathway"),
        Some(Question::Choice { criteria, .. }) if criteria.contains("automate")
    );
    if first_round {
        request.to_string()
    } else {
        format!("clause:{request}")
    }
}

pub struct FixtureWorld {
    catalog: Arc<Catalog>,
    contexts: HashMap<String, ScreenContext>,
    /// Which screen each recorded request was read against.
    screens: Vec<(String, String)>,
    answers: RecordedAnswers,
}

static WORLD: LazyLock<Option<Arc<FixtureWorld>>> = LazyLock::new(|| {
    let replay: Replay = serde_json::from_str(REPLAY).ok()?;
    let evaluation = evaluation();
    let mut answers = RecordedAnswers::new();
    let mut screens = Vec::new();
    for sentence in &evaluation.sentences {
        for request in evaluation_requests(sentence) {
            screens.push((request, sentence.ctx.clone()));
        }
    }
    answers.extend(evaluation.answers);
    for case in replay.cases {
        screens.push((case.request.clone(), case.ctx));
        for (key, set) in case.answers {
            answers.entry(key).or_insert(set);
        }
    }
    Some(Arc::new(FixtureWorld {
        catalog: Arc::new(replay.catalog),
        contexts: replay.contexts,
        screens,
        answers,
    }))
});

impl FixtureWorld {
    pub fn get() -> Option<Arc<FixtureWorld>> {
        WORLD.clone()
    }

    pub fn catalog(&self) -> Arc<Catalog> {
        self.catalog.clone()
    }

    /// The screen `text` was recorded against: its own, else that of the sentence it begins.
    pub fn screen_for(&self, text: &str) -> ScreenContext {
        let text = text.trim();
        let ctx = self
            .screens
            .iter()
            .find(|(request, _)| request == text)
            .or_else(|| {
                self.screens
                    .iter()
                    .find(|(request, _)| request.starts_with(text))
            })
            .map_or("chrome", |(_, ctx)| ctx.as_str());
        let mut screen = self.contexts.get(ctx).cloned().unwrap_or_default();
        screen.subject_read_at_run = true;
        screen
    }
}

impl Oracle for FixtureWorld {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError> {
        let key = answer_key(state, questions);
        let set = self
            .answers
            .get(&key)
            .ok_or_else(|| JevError::Decode(format!("no recorded answers for {key}")))?;
        let answers: Answers = set
            .iter()
            .map(|(name, answer)| (name.clone(), answer.expand()))
            .collect();
        Ok(JevResult {
            answers,
            ..JevResult::default()
        })
    }
}
