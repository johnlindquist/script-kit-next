//! Replays Jev answers recorded by the TypeScript prototype and requires the same plans.
//!
//! The fixture holds 132 requests across four screens, Jev's trimmed answers for each
//! round, and the plan the prototype composed from exactly those answers.

use std::collections::HashMap;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::engine::{build_questions, EngineOptions, Plan, Step};
use super::jev::{
    Answer, Answers, JevError, JevResult, Oracle, Probabilities, Question, Questions,
};
use super::{understand, Catalog, ScreenContext};

const FIXTURE: &str = include_str!("fixtures/replay.json");

#[derive(Deserialize)]
#[serde(untagged)]
enum Compact {
    Noul(f64),
    Choice(String, Probabilities),
    Score(f64, Probabilities),
}

impl Compact {
    fn expand(&self) -> Answer {
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

/// First-round and clause-round answers for the same words differ, so the key says which.
fn answer_key(state: &serde_json::Value, questions: &Questions) -> String {
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
