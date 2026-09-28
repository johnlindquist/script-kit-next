//! Replays Jev answers recorded by the TypeScript prototype and requires the same plans.
//!
//! The fixture holds 138 requests across four screens, Jev's trimmed answers for each
//! round, and the plan the prototype composed from exactly those answers.

use std::collections::HashMap;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::engine::{build_questions, EngineOptions, Plan, Step};
use super::fixture_oracle::{answer_key, evaluation, evaluation_requests, Compact};
use super::jev::{Answers, JevError, JevResult, Oracle, Questions};
use super::{understand, Catalog, ScreenContext};

const FIXTURE: &str = include_str!("fixtures/replay.json");

#[derive(Deserialize, Debug, PartialEq)]
struct Expect {
    pathway: String,
    title: String,
    ready: bool,
    typed: Option<String>,
    missing: Option<String>,
    steps: Vec<String>,
    alternatives: Vec<String>,
    clauses: usize,
    complete: f64,
    reuse: f64,
}

#[derive(Deserialize)]
struct Case {
    ctx: String,
    request: String,
    answers: HashMap<String, HashMap<String, Compact>>,
    expect: Expect,
}

#[derive(Deserialize)]
struct Sample {
    ctx: String,
    request: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    catalog: Catalog,
    contexts: HashMap<String, ScreenContext>,
    cases: Vec<Case>,
    question_samples: Vec<Sample>,
}

fn fixture() -> Fixture {
    serde_json::from_str(FIXTURE).expect("replay fixture parses")
}

struct Recorded<'a> {
    answers: &'a HashMap<String, HashMap<String, Compact>>,
}

impl Oracle for Recorded<'_> {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError> {
        let key = answer_key(state, questions);
        let set = self
            .answers
            .get(&key)
            .ok_or_else(|| JevError::Decode(format!("no recorded answers for {key}")))?;
        let answers: Answers = set
            .iter()
            .map(|(name, a)| (name.clone(), a.expand()))
            .collect();
        Ok(JevResult {
            answers,
            ..JevResult::default()
        })
    }
}

fn step_ref(step: &Step) -> String {
    match step {
        Step::Command { command } => format!("command:{}", command.id),
        Step::Menu { path, .. } => format!("menu:{}", path.join(" › ")),
        Step::Window { op, app } => format!("window:{}:{app}", op.label),
        Step::OpenUrl { url, app } => format!("url:{url}:{}", app.as_deref().unwrap_or_default()),
        Step::Capture {
            target, body, when, ..
        } => format!(
            "capture:{target}:{body}:{}",
            when.as_deref().unwrap_or_default()
        ),
        Step::Remember { text, .. } => format!("remember:{text}"),
        Step::Recall { source, query } => {
            format!("recall:{source}:{}", query.as_deref().unwrap_or_default())
        }
        Step::Transform { tool, subject, .. } => format!(
            "transform:{}:{subject}",
            tool.as_ref().map_or("custom", |t| t.name.as_str())
        ),
        Step::Ask { question, subject } => {
            format!("ask:{question}:{}", subject.as_deref().unwrap_or_default())
        }
        Step::Delegate {
            task,
            project,
            subject,
        } => format!(
            "delegate:{task}:{}:{}",
            project.as_deref().unwrap_or_default(),
            subject.as_deref().unwrap_or_default()
        ),
    }
}

fn summarize(plan: &Plan) -> Expect {
    Expect {
        pathway: plan.pathway.key().to_string(),
        title: plan.title.clone(),
        ready: plan.ready,
        typed: plan.typed.clone(),
        missing: plan.missing.clone(),
        steps: plan.steps.iter().map(step_ref).collect(),
        alternatives: plan
            .alternatives
            .iter()
            .map(|a| format!("{}:{}", a.pathway.key(), a.title))
            .collect(),
        clauses: plan.clauses.len(),
        complete: plan.complete,
        reuse: plan.reuse,
    }
}

#[test]
fn every_recorded_request_composes_the_plan_the_prototype_composed() {
    let fixture = fixture();
    let mut mismatches = Vec::new();
    for case in &fixture.cases {
        let ctx = &fixture.contexts[&case.ctx];
        let oracle = Recorded {
            answers: &case.answers,
        };
        let plan = understand(
            &case.request,
            ctx,
            &fixture.catalog,
            &oracle,
            &EngineOptions::default(),
        )
        .unwrap_or_else(|error| panic!("{:?}: {error}", case.request));
        let got = summarize(&plan);
        if got != case.expect {
            mismatches.push(format!(
                "{:?} ({})\n  want {:?}\n  got  {:?}",
                case.request, case.ctx, case.expect, got
            ));
        }
    }
    assert!(fixture.cases.len() >= 100, "fixture lost its cases");
    assert!(
        mismatches.is_empty(),
        "{} of {} plans differ from the prototype:\n{}",
        mismatches.len(),
        fixture.cases.len(),
        mismatches.join("\n")
    );
}

#[test]
fn jevs_rows_lead_with_its_choice_and_every_alternative_runs() {
    let fixture = fixture();
    let mut with_alternatives = 0;
    for case in &fixture.cases {
        let oracle = Recorded {
            answers: &case.answers,
        };
        let plan = understand(
            &case.request,
            &fixture.contexts[&case.ctx],
            &fixture.catalog,
            &oracle,
            &EngineOptions::default(),
        )
        .unwrap_or_else(|error| panic!("{:?}: {error}", case.request));
        let top = (plan.pathway != super::engine::Pathway::None).then(|| plan.title.clone());
        with_alternatives += usize::from(!plan.alternatives.is_empty());
        let rows = super::readings(plan);
        assert!(rows.len() <= super::READINGS, "{:?}", case.request);
        let keys: std::collections::HashSet<&str> = rows.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(keys.len(), rows.len(), "{:?} repeats a row", case.request);
        if let Some(title) = &top {
            assert_eq!(&rows[0].plan.title, title, "{:?}", case.request);
        }
        for alternative in rows.iter().skip(usize::from(top.is_some())) {
            assert!(
                alternative.plan.missing.is_none() && !alternative.plan.steps.is_empty(),
                "{:?}: alternative {:?} can't run",
                case.request,
                alternative.plan.title
            );
            assert!(alternative.plan.alternatives.is_empty());
        }
    }
    assert!(
        with_alternatives > 0,
        "no recorded plan weighs alternatives"
    );
}

/// Keys sorted by UTF-16 code unit, as JavaScript's default sort orders them.
fn canonical_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(canonical_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by_key(|k| k.encode_utf16().collect::<Vec<u16>>());
            let fields: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{}:{}",
                        serde_json::Value::from(k.as_str()),
                        canonical_value(&map[k])
                    )
                })
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        other => other.to_string(),
    }
}

fn canonical(built: &super::engine::Built) -> String {
    let value = serde_json::json!({ "state": built.state, "questions": built.questions });
    canonical_value(&value)
}

#[test]
fn the_questions_jev_sees_match_the_prototype_byte_for_byte() {
    let fixture = fixture();
    assert!(!fixture.question_samples.is_empty());
    for sample in &fixture.question_samples {
        let built = build_questions(
            &sample.request,
            &fixture.contexts[&sample.ctx],
            &fixture.catalog,
            &EngineOptions::default(),
        );
        let text = canonical(&built);
        let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
        if digest != sample.sha256 {
            let path = std::env::temp_dir().join(format!(
                "intent-questions-{}.json",
                super::script::slugify(&sample.request)
            ));
            let _ = std::fs::write(&path, &text);
            panic!(
                "questions for {:?} differ from the prototype; Rust's are in {}",
                sample.request,
                path.display()
            );
        }
    }
}

fn words_for(fixture: &Fixture, request: &str) -> Vec<super::WordRead> {
    let case = fixture
        .cases
        .iter()
        .find(|case| case.request == request)
        .unwrap_or_else(|| panic!("no recorded case for {request:?}"));
    let oracle = Recorded {
        answers: &case.answers,
    };
    understand(
        &case.request,
        &fixture.contexts[&case.ctx],
        &fixture.catalog,
        &oracle,
        &EngineOptions::default(),
    )
    .unwrap_or_else(|error| panic!("{request:?}: {error}"))
    .words
}

fn said(words: &[super::WordRead]) -> Vec<(super::WordKind, &str)> {
    words
        .iter()
        .filter_map(|w| w.said.as_deref().map(|said| (w.kind, said)))
        .collect()
}

#[test]
fn jev_dissects_each_sentence_into_its_words_from_the_recorded_answers() {
    use super::WordKind::{Action, Details, Owner, Subject, Target};
    let fixture = fixture();
    let cases: &[(&str, &[(super::WordKind, &str)])] = &[
        (
            "remind me to call mom tomorrow at 5pm",
            &[
                (Subject, "call mom"),
                (Action, "remind me"),
                (Details, "tomorrow at 5pm"),
            ],
        ),
        (
            "have an agent fix the failing tests in script kit",
            &[
                (Subject, "the failing tests"),
                (Action, "fix"),
                (Target, "in script kit"),
                (Owner, "have an agent"),
            ],
        ),
        (
            "summarize this",
            &[(Subject, "this"), (Action, "summarize")],
        ),
        (
            "add buy oat milk to my todo list",
            &[
                (Subject, "buy oat milk"),
                (Action, "add"),
                (Target, "to my todo list"),
            ],
        ),
        (
            "put chrome on the left half",
            &[
                (Subject, "chrome"),
                (Action, "put"),
                (Target, "on the left half"),
            ],
        ),
        ("open slack", &[(Subject, "slack"), (Action, "open")]),
    ];
    let mut wrong = Vec::new();
    for (request, want) in cases {
        let words = words_for(&fixture, request);
        let got = said(&words);
        if got.as_slice() != *want {
            wrong.push(format!("{request:?}\n  want {want:?}\n  got  {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_word_jev_reads_is_its_own_run_of_the_users_words() {
    let fixture = fixture();
    for case in &fixture.cases {
        let words = words_for(&fixture, &case.request);
        assert_eq!(
            words.iter().map(|w| w.kind).collect::<Vec<_>>(),
            super::WordKind::ALL.to_vec(),
            "{:?}",
            case.request
        );
        let mut runs: Vec<std::ops::Range<usize>> = Vec::new();
        for word in &words {
            let Some(said) = word.said.as_deref() else {
                continue;
            };
            assert!(
                !said.trim().is_empty(),
                "{:?}: empty {:?}",
                case.request,
                word.kind
            );
            let at = case
                .request
                .find(said)
                .unwrap_or_else(|| panic!("{:?}: {said:?} isn't in it", case.request));
            let run = at..at + said.len();
            assert!(
                runs.iter()
                    .all(|r| run.end <= r.start || r.end <= run.start),
                "{:?}: {:?} {said:?} overlaps another word",
                case.request,
                word.kind
            );
            runs.push(run);
        }
    }
}

/// Answers from the evaluation fixture; a round it has none for is written where
/// `record-jev-answers.ts` picks it up.
struct Evaluated<'a> {
    answers: &'a HashMap<String, HashMap<String, Compact>>,
    ctx: &'a str,
    missing: std::sync::Mutex<Vec<String>>,
}

impl Oracle for Evaluated<'_> {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError> {
        let key = answer_key(state, questions);
        if let Some(set) = self.answers.get(&key) {
            return Ok(JevResult {
                answers: set.iter().map(|(n, a)| (n.clone(), a.expand())).collect(),
                ..JevResult::default()
            });
        }
        let dir = std::env::temp_dir().join("intent-evaluation");
        let _ = std::fs::create_dir_all(&dir);
        let round = serde_json::json!({
            "key": key,
            "ctx": self.ctx,
            "request": state["request"],
            "state": state,
            "questions": questions,
        });
        let path = dir.join(format!("{}.json", super::script::slugify(&key)));
        let _ = std::fs::write(&path, round.to_string());
        if let Ok(mut missing) = self.missing.lock() {
            missing.push(key.clone());
        }
        Err(JevError::Decode(format!("no recorded answers for {key}")))
    }
}

#[test]
fn evaluation_sentences_have_jevs_recorded_answers() {
    let fixture = fixture();
    let evaluation = evaluation();
    assert!(
        !evaluation.sentences.is_empty(),
        "evaluation fixture lost its sentences"
    );
    let mut missing = Vec::new();
    for sentence in &evaluation.sentences {
        for request in evaluation_requests(sentence) {
            let oracle = Evaluated {
                answers: &evaluation.answers,
                ctx: &sentence.ctx,
                missing: std::sync::Mutex::new(Vec::new()),
            };
            let _ = understand(
                &request,
                &fixture.contexts[&sentence.ctx],
                &fixture.catalog,
                &oracle,
                &EngineOptions::default(),
            );
            missing.extend(oracle.missing.into_inner().unwrap_or_default());
        }
    }
    assert!(
        missing.is_empty(),
        "{} rounds have no recorded answers: {missing:?}\nTheir questions are in {}; run \
         `bun scripts/agentic/one-window/record-jev-answers.ts`, then this test again.",
        missing.len(),
        std::env::temp_dir().join("intent-evaluation").display()
    );
}

#[test]
fn evaluation_jev_reads_the_s04_sentences_into_their_words() {
    use super::WordKind::{Action, Details, Owner, Subject, Target};
    let fixture = fixture();
    let evaluation = evaluation();
    let cases: &[(&str, &str, &[(super::WordKind, &str)])] = &[
        (
            "chrome",
            "send what I copied to sarah tomorrow at 9",
            &[
                (Subject, "what I copied"),
                (Action, "send"),
                (Target, "to sarah"),
                (Details, "tomorrow at 9"),
            ],
        ),
        (
            "mail",
            "summarize this @clipboard .concise",
            &[(Subject, "this"), (Action, "summarize")],
        ),
        (
            "ghostty",
            "have the agent fix this",
            &[
                (Subject, "this"),
                (Action, "fix"),
                (Owner, "have the agent"),
            ],
        ),
        ("ghostty", "fix this", &[(Subject, "this"), (Action, "fix")]),
    ];
    let mut wrong = Vec::new();
    for (ctx, request, want) in cases {
        let oracle = Recorded {
            answers: &evaluation.answers,
        };
        let plan = understand(
            request,
            &fixture.contexts[*ctx],
            &fixture.catalog,
            &oracle,
            &EngineOptions::default(),
        )
        .unwrap_or_else(|error| panic!("{request:?}: {error}"));
        let got = said(&plan.words);
        if got.as_slice() != *want {
            wrong.push(format!("{request:?}\n  want {want:?}\n  got  {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
