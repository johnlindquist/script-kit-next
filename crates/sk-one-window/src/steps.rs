//! A sentence with several jobs becomes steps in place: each step is read on its own, a step that
//! needs a service that isn't connected says so right there, and "Do all" runs them.

use crate::connect::{self, Connections};
use crate::enter::EnterTarget;
use crate::row::Row;

pub const VIEW: &str = "steps";
pub const LABEL: &str = "Steps";
/// The key of the local "Do these N things" row.
pub const ROW_KEY: &str = "steps";
pub const OFFLINE_UNAVAILABLE: &str = "steps:offline-unavailable";

/// Only these forms have a verified local executor. Never hand an unsupported
/// clause to an agent or write it as a note instead of performing its verb.
pub fn offline_note(text: &str) -> Option<&str> {
    let (head, body) = text.split_once(':')?;
    matches!(
        head.trim().to_ascii_lowercase().as_str(),
        "add to today's note" | "add to today’s note" | "note"
    )
    .then(|| body.trim())
    .filter(|s| !s.is_empty())
}

pub fn offline_supported(text: &str) -> bool {
    let parts = split(text);
    (2..=4).contains(&parts.len()) && parts.iter().all(|s| offline_note(s).is_some())
}

pub fn offline_row(text: &str) -> Option<Row> {
    let row = steps_row(text)?;
    Some(if offline_supported(text) {
        row
    } else {
        Row::new(OFFLINE_UNAVAILABLE, "Can't run all these steps offline")
            .with_sub("Only explicit additions to today's note can run here. Nothing will be sent.")
            .with_enter_label("Unavailable")
            .with_score(0)
            .fallback()
    })
}

/// Words that start a new job after "and" ("… and tell mo …").
const JOB_VERBS: &[&str] = &[
    "ping",
    "tell",
    "message",
    "send",
    "open",
    "remind",
    "post",
    "add",
    "put",
    "schedule",
    "mute",
    "tile",
    "move",
    "email",
    "text",
    "draft",
    "review",
    "fix",
    "write",
    "summarize",
    "run",
    "let",
    "append",
    "turn",
    "start",
    "launch",
    "toggle",
    "set",
];

/// Splits a sentence into its jobs the way the prototype's `splitSteps` does: at commas (eating an
/// "and"/"then" after one), at "and then" and "then", and at "and" when a job verb follows it.
pub fn split(sentence: &str) -> Vec<String> {
    let mut parts = Vec::new();
    for (i, chunk) in sentence.split(',').enumerate() {
        let mut words: Vec<&str> = chunk.split_whitespace().collect();
        if i > 0 && words.first().is_some_and(|w| is(w, "and") || is(w, "then")) {
            words.remove(0);
        }
        let mut current: Vec<&str> = Vec::new();
        let mut w = 0;
        while w < words.len() {
            let word = words[w];
            let next = words.get(w + 1).copied();
            let starts_job = next.is_some_and(|n| JOB_VERBS.iter().any(|v| is(n, v)));
            if is(word, "and") && next.is_some_and(|n| is(n, "then")) && !current.is_empty() {
                parts.push(current.join(" "));
                current.clear();
                w += 2;
                continue;
            }
            if (is(word, "then") && !current.is_empty() && next.is_some())
                || (is(word, "and") && starts_job && !current.is_empty())
            {
                parts.push(current.join(" "));
                current.clear();
                w += 1;
                continue;
            }
            current.push(word);
            w += 1;
        }
        if !current.is_empty() {
            parts.push(current.join(" "));
        }
    }
    parts
}

fn is(word: &str, target: &str) -> bool {
    word.eq_ignore_ascii_case(target)
}

/// The instant local row for a sentence with several jobs; the steps are read once it's opened.
pub fn steps_row(text: &str) -> Option<Row> {
    let parts = split(text);
    if parts.len() < 2 {
        return None;
    }
    let title = format!("Do these {} things", parts.len());
    Some(
        Row::new(ROW_KEY, title.clone())
            .with_icon("steps")
            .with_sub(parts.join(" · "))
            .with_enter_label(title),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    /// Waiting for its reading.
    Reading,
    Ready,
    Done,
    /// It ran and didn't do what it said.
    Failed,
}

/// What reading a step found: what it does and the service it needs, if any.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StepSpec {
    pub text: String,
    pub title: Option<String>,
    /// The integration it needs (`connect::INTEGRATIONS` key).
    pub needs: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub text: String,
    pub title: Option<String>,
    pub needs: Option<String>,
    /// The user chose to go without the service ("copy a message instead").
    pub without: bool,
    pub dropped: bool,
    pub state: StepState,
}

impl Step {
    fn reading(text: String) -> Self {
        Self {
            text,
            title: None,
            needs: None,
            without: false,
            dropped: false,
            state: StepState::Reading,
        }
    }

    /// The service this step still waits on.
    pub fn waits_on(&self, connections: &Connections) -> Option<&str> {
        self.needs
            .as_deref()
            .filter(|n| !self.without && !connections.has(n))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Reading,
    Ready,
    Running,
}

/// What a key did in the steps view.
#[derive(Clone, Debug, PartialEq)]
pub enum StepsKey {
    Nothing,
    Moved,
    /// Enter while reading: it runs them once they're read.
    Held,
    /// A step needs a service first.
    Connect {
        step: usize,
        integration: String,
    },
    Run,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Steps {
    pub sentence: String,
    pub steps: Vec<Step>,
    pub phase: Phase,
    pub held: bool,
    pub sel: Option<usize>,
}

impl Steps {
    /// Opens on a sentence: one card per job, each still being read.
    pub fn open(sentence: &str) -> Self {
        Self {
            sentence: sentence.to_string(),
            steps: split(sentence).into_iter().map(Step::reading).collect(),
            phase: Phase::Reading,
            held: false,
            sel: None,
        }
    }

    pub fn live(&self) -> impl Iterator<Item = (usize, &Step)> {
        self.steps.iter().enumerate().filter(|(_, s)| !s.dropped)
    }

    /// The first step that waits on a service, with that service.
    pub fn missing(&self, connections: &Connections) -> Option<(usize, String)> {
        self.live()
            .find_map(|(i, s)| s.waits_on(connections).map(|n| (i, n.to_string())))
    }

    /// New words for the steps: they are read again, and a step with the same words keeps what it
    /// had. Returns whether the sentence changed.
    pub fn edit(&mut self, sentence: &str) -> bool {
        if sentence == self.sentence {
            return false;
        }
        let old = std::mem::take(&mut self.steps);
        self.sentence = sentence.to_string();
        self.steps = split(sentence)
            .into_iter()
            .map(|text| {
                old.iter()
                    .find(|s| s.text == text)
                    .cloned()
                    .unwrap_or_else(|| Step::reading(text))
            })
            .collect();
        self.phase = Phase::Reading;
        self.sel = None;
        true
    }

    /// Readings arrived for `sentence`. Returns whether an Enter was waiting for them.
    pub fn read(&mut self, sentence: &str, specs: Vec<StepSpec>) -> bool {
        if sentence != self.sentence || self.phase == Phase::Running {
            return false;
        }
        let old = std::mem::take(&mut self.steps);
        self.steps = specs
            .into_iter()
            .map(|spec| {
                let kept = old.iter().find(|s| s.text == spec.text);
                Step {
                    without: kept.is_some_and(|s| s.without),
                    dropped: kept.is_some_and(|s| s.dropped),
                    state: StepState::Ready,
                    text: spec.text,
                    title: spec.title,
                    needs: spec.needs,
                }
            })
            .collect();
        self.phase = Phase::Ready;
        std::mem::take(&mut self.held)
    }

    /// Everything reads again, as after a service connected.
    pub fn reread(&mut self) {
        self.phase = Phase::Reading;
    }

    fn all(&self) -> String {
        match self.live().count() {
            2 => "both".to_string(),
            n => format!("all {n}"),
        }
    }

    /// What ↵ says in the steps view.
    pub fn enter(&self, connections: &Connections) -> EnterTarget {
        let n = self.live().count();
        match self.phase {
            Phase::Reading if self.held => EnterTarget::new(
                "steps:all",
                format!(
                    "Does {} as soon as each step is read…",
                    if n > 1 { self.all() } else { "it".to_string() }
                ),
            ),
            Phase::Reading => EnterTarget::new("steps:all", "Reading the steps…"),
            Phase::Ready => match self.missing(connections) {
                Some((i, key)) => EnterTarget::new(
                    format!("steps:connect:{key}"),
                    format!(
                        "Connect {} for step {}, then do {}",
                        connect::name_of(&key),
                        i + 1,
                        self.all()
                    ),
                ),
                None if n == 0 => EnterTarget::new("steps:all", "Nothing left to do"),
                None => EnterTarget::new("steps:all", format!("Do {}", self.all())),
            },
            Phase::Running => EnterTarget::new("steps:running", "Working through them…"),
        }
    }

    pub fn move_sel(&mut self, down: bool) -> bool {
        let n = self.steps.len();
        if n == 0 {
            return false;
        }
        self.sel = match self.sel {
            None if down => Some(0),
            None => Some(n - 1),
            Some(i) if down => (i + 1 < n).then_some(i + 1),
            Some(i) => i.checked_sub(1),
        };
        true
    }

    /// Enter: waits while reading, asks for a service a step needs, or runs them.
    pub fn press_enter(&mut self, connections: &Connections) -> StepsKey {
        match self.phase {
            Phase::Reading => {
                self.held = true;
                StepsKey::Held
            }
            Phase::Ready => match self.missing(connections) {
                Some((step, integration)) => {
                    self.sel = Some(step);
                    StepsKey::Connect { step, integration }
                }
                None if self.live().count() == 0 => StepsKey::Nothing,
                None => {
                    self.phase = Phase::Running;
                    StepsKey::Run
                }
            },
            Phase::Running => StepsKey::Nothing,
        }
    }

    /// The step goes without its service: it copies a message instead.
    pub fn go_without(&mut self, step: usize) {
        if let Some(s) = self.steps.get_mut(step) {
            s.without = true;
        }
    }

    pub fn step_done(&mut self, step: usize) {
        if let Some(s) = self.steps.get_mut(step) {
            s.state = StepState::Done;
        }
    }

    pub fn step_failed(&mut self, step: usize) {
        if let Some(s) = self.steps.get_mut(step) {
            s.state = StepState::Failed;
        }
    }

    /// Every step ran, whether it did what it said or not.
    pub fn all_done(&self) -> bool {
        self.phase == Phase::Running
            && self
                .live()
                .all(|(_, s)| matches!(s.state, StepState::Done | StepState::Failed))
    }

    /// Some step didn't do what it said.
    pub fn any_failed(&self) -> bool {
        self.live().any(|(_, s)| s.state == StepState::Failed)
    }

    /// The line the run leaves: "Did 3 steps: a · b · c", or "Did 2 of 3 steps: a · b" when a step
    /// failed: only the steps that did what they said are counted.
    pub fn done_title(&self) -> String {
        let all = self.live().count();
        let did: Vec<&str> = self
            .live()
            .filter(|(_, s)| s.state == StepState::Done)
            .map(|(_, s)| s.text.as_str())
            .collect();
        let noun = if all == 1 { "step" } else { "steps" };
        if did.len() == all {
            format!("Did {all} {noun}: {}", did.join(" · "))
        } else if did.is_empty() {
            format!("Didn't do the {noun}")
        } else {
            format!("Did {} of {all} {noun}: {}", did.len(), did.join(" · "))
        }
    }

    pub fn texts(&self) -> Vec<String> {
        self.live().map(|(_, s)| s.text.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S10: &str = "move standup to tomorrow at 9, tell mo i'm running late and add to today's note: ship beta notes";

    #[test]
    fn offline_steps_never_send_delegate_or_apply_a_partial_list() {
        let blocked = offline_row("add to today's note: lunch and tell Mo I'm late").unwrap();
        assert_eq!(blocked.key, OFFLINE_UNAVAILABLE);
        assert_eq!(blocked.enter_words(), "Unavailable");
        assert!(!offline_supported(
            "delete my note: lunch then add to today's note: lunch"
        ));
        assert!(offline_supported(
            "add to today's note: apples then add to today's note: pears"
        ));
    }

    fn spec(text: &str, needs: Option<&str>) -> StepSpec {
        StepSpec {
            text: text.into(),
            title: Some(text.into()),
            needs: needs.map(str::to_string),
        }
    }

    #[test]
    fn splits_like_the_prototype() {
        assert_eq!(
            split(S10),
            [
                "move standup to tomorrow at 9",
                "tell mo i'm running late",
                "add to today's note: ship beta notes"
            ]
        );
        assert_eq!(
            split("open gmail and then open linear"),
            ["open gmail", "open linear"]
        );
        assert_eq!(split("buy milk and eggs"), ["buy milk and eggs"]);
        assert_eq!(
            split("tile chrome left, then ghostty right"),
            ["tile chrome left", "ghostty right"]
        );
        assert!(split("  ").is_empty());
    }

    #[test]
    fn the_local_row_counts_the_jobs() {
        let row = steps_row(S10).expect("three jobs");
        assert_eq!(row.title, "Do these 3 things");
        assert_eq!(row.enter_words(), "Do these 3 things");
        assert!(steps_row("buy milk").is_none());
    }

    #[test]
    fn enter_reads_then_asks_for_the_service_then_does_all() {
        let mut conns = Connections::default();
        let mut s = Steps::open(S10);
        assert_eq!(s.enter(&conns).label, "Reading the steps…");
        let texts = s.texts();
        s.read(
            S10,
            vec![
                spec(&texts[0], Some("calendar")),
                spec(&texts[1], None),
                spec(&texts[2], None),
            ],
        );
        assert_eq!(
            s.enter(&conns),
            EnterTarget::new(
                "steps:connect:calendar",
                "Connect Google Calendar for step 1, then do all 3"
            )
        );
        assert_eq!(
            s.press_enter(&conns),
            StepsKey::Connect {
                step: 0,
                integration: "calendar".into()
            }
        );
        conns.add("calendar");
        assert_eq!(s.enter(&conns).label, "Do all 3");
        assert_eq!(s.press_enter(&conns), StepsKey::Run);
        assert_eq!(s.enter(&conns).label, "Working through them…");
        for i in 0..3 {
            assert!(!s.all_done());
            s.step_done(i);
        }
        assert!(s.all_done());
        assert_eq!(
            s.done_title(),
            "Did 3 steps: move standup to tomorrow at 9 · tell mo i'm running late · add to today's note: ship beta notes"
        );
    }

    #[test]
    fn an_enter_while_reading_waits_and_says_so() {
        let conns = Connections::default();
        let mut s = Steps::open("open gmail and then open linear");
        assert_eq!(s.press_enter(&conns), StepsKey::Held);
        assert_eq!(
            s.enter(&conns).label,
            "Does both as soon as each step is read…"
        );
        let texts = s.texts();
        let waited = s.read(
            "open gmail and then open linear",
            texts.iter().map(|t| spec(t, None)).collect(),
        );
        assert!(waited);
        assert_eq!(s.enter(&conns).label, "Do both");
    }

    #[test]
    fn going_without_a_service_settles_its_step() {
        let conns = Connections::default();
        let mut s = Steps::open("move standup to 10 and tell mo");
        let texts = s.texts();
        s.read(
            "move standup to 10 and tell mo",
            vec![spec(&texts[0], Some("calendar")), spec(&texts[1], None)],
        );
        s.go_without(0);
        assert_eq!(s.enter(&conns).label, "Do both");
    }

    #[test]
    fn saying_a_step_differently_reads_it_again_and_keeps_the_rest() {
        let conns = Connections::default();
        let mut s = Steps::open("open gmail and then open linear");
        let texts = s.texts();
        s.read(
            "open gmail and then open linear",
            texts.iter().map(|t| spec(t, None)).collect(),
        );
        assert!(s.edit("open gmail and then open notion"));
        assert_eq!(s.phase, Phase::Reading);
        assert_eq!(
            s.steps[0].state,
            StepState::Ready,
            "the unchanged step keeps its reading"
        );
        assert_eq!(s.steps[1].state, StepState::Reading);
        assert_eq!(s.enter(&conns).label, "Reading the steps…");
        assert!(
            !s.read("open gmail and then open linear", Vec::new()),
            "old words are ignored"
        );
        assert_eq!(s.phase, Phase::Reading);
    }
}
