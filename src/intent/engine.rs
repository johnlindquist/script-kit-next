//! Turns a request and what's on screen into Jev questions, and Jev's answers into a plan.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::LazyLock;
use std::time::Instant;

use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use super::catalog::{Catalog, Command, CommandKind, WindowOp};
use super::context::{describe, LastAction, LastStep, MenuCommand, ScreenContext};
use super::jev::{Answer, Answers, Criteria, JevError, Oracle, Probabilities, Question, Questions};
use super::spans::{
    span_candidates, split_clauses_for_apps, tokenize, Span, Word, MAX_CLAUSES, SPAN_LIMIT,
};
use super::words::{read_words, WordRead};

pub const NONE: &str = "none";
/// Unfinished requests scored 0.07–0.35 in testing; the least sure finished one scored 0.49.
pub const COMPLETE_THRESHOLD: f64 = 0.45;
const READY_PROBABILITY: f64 = 0.35;
/// Below this, the best command match is too weak to run, so the run reading has no steps.
const RUN_FLOOR: f64 = 0.3;
/// A window_target answer no single window step can carry out.
const EVERY_WINDOW: &str = "every window";
/// A catalog or menu match this strong is evidence enough on its own.
const STRONG_TARGET: f64 = 0.9;
/// Answers this close are noise; the words the user said, then the kind order, break the tie.
const NEAR_TIE: f64 = 0.03;
const ALTERNATIVE_FLOOR: f64 = 0.05;
/// Per-source retrieval budget, leaving ample room for an explicit abstention choice.
pub(super) const CANDIDATE_LIMIT: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pathway {
    Run,
    AppCommand,
    Window,
    Capture,
    Remember,
    Recall,
    Transform,
    Ask,
    Delegate,
    Automate,
    None,
}

impl Pathway {
    pub const ALL: [Pathway; 11] = [
        Pathway::Run,
        Pathway::AppCommand,
        Pathway::Window,
        Pathway::Capture,
        Pathway::Remember,
        Pathway::Recall,
        Pathway::Transform,
        Pathway::Ask,
        Pathway::Delegate,
        Pathway::Automate,
        Pathway::None,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::AppCommand => "app_command",
            Self::Window => "window",
            Self::Capture => "capture",
            Self::Remember => "remember",
            Self::Recall => "recall",
            Self::Transform => "transform",
            Self::Ask => "ask",
            Self::Delegate => "delegate",
            Self::Automate => "automate",
            Self::None => NONE,
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }

    fn description(self) -> &'static str {
        match self {
            Self::Run => {
                "Open or run one existing thing by name: an app, a website quick link, a Script Kit command, a script, or a system action like dark mode, volume, lock screen, empty trash"
            }
            Self::AppCommand => {
                "Do something inside the frontmost app using one of its own menu commands, like a new tab, reply, split pane, zoom, or reopen a closed tab"
            }
            Self::Window => {
                "Move, resize, tile, center, maximize, minimize, or close a window, send it to another display, or quit an app"
            }
            Self::Capture => {
                "Write down a todo, a reminder, a calendar event, a link, a social post, or a line in today's note"
            }
            Self::Remember => {
                "Store a fact in the user's personal memory so Script Kit can recall it later: people, preferences, decisions, numbers, where things are"
            }
            Self::Recall => {
                "Look something up in the user's own past: things they copied, pages they visited, things they dictated, notes, earlier AI chats, facts they asked to remember"
            }
            Self::Transform => {
                "Rewrite, summarize, translate, fix, shorten, or explain existing text such as the selected text or the clipboard"
            }
            Self::Ask => "Get a quick answer to a question from AI, with no follow-up work",
            Self::Delegate => {
                "Hand a multi-step job to an AI agent that works on its own: fix, build, research, investigate, refactor, write code, draft something long"
            }
            Self::Automate => {
                "Save something as a new reusable script or keyboard shortcut instead of doing it right now. The user says so outright, with words like make a script, create a shortcut, save that as a command, or turn what I just did into a shortcut"
            }
            Self::None => {
                "The words don't ask for anything yet, or ask for something none of these can do"
            }
        }
    }

    /// Pathways Script Kit carries out itself; LLM-bound ones keep a multi-part request whole.
    pub fn is_instant(self) -> bool {
        matches!(
            self,
            Self::Run | Self::AppCommand | Self::Window | Self::Capture
        )
    }
}

pub const CAPTURE_TARGETS: &[(&str, &str)] = &[
    ("todo", "A task to do, with no reminder time"),
    (
        "reminder",
        "Something to be reminded about, usually at a time",
    ),
    ("cal", "A calendar event or meeting at a date and time"),
    ("note", "A line for today's daily note or journal"),
    ("link", "A web link to save for later"),
    ("social", "A draft social media post"),
];

pub struct RecallSource {
    pub key: &'static str,
    pub description: &'static str,
    /// The launcher's `@source:` sub-search prefix.
    pub prefix: &'static str,
    /// Not a launcher prefix yet.
    pub proposed: bool,
    pub builtin: Option<&'static str>,
}

pub const RECALL_SOURCES: &[RecallSource] = &[
    RecallSource {
        key: "clipboard",
        description: "Things the user copied",
        prefix: "@clipboard:",
        proposed: false,
        builtin: Some("builtin/clipboard-history"),
    },
    RecallSource {
        key: "browser-history",
        description: "Web pages the user visited before",
        prefix: "@browser-history:",
        proposed: false,
        builtin: None,
    },
    RecallSource {
        key: "tabs",
        description: "Browser tabs open right now",
        prefix: ":type:window ",
        proposed: false,
        builtin: Some("builtin/browser-tabs"),
    },
    RecallSource {
        key: "dictation",
        description: "Things the user said with voice dictation",
        prefix: "@dictation:",
        proposed: false,
        builtin: Some("builtin/dictation-history"),
    },
    RecallSource {
        key: "notes",
        description: "The user's written notes",
        prefix: "@notes:",
        proposed: false,
        builtin: Some("builtin/search-notes"),
    },
    RecallSource {
        key: "history",
        description: "Earlier AI and Agent Chat conversations",
        prefix: "@history:",
        proposed: false,
        builtin: Some("builtin/agent_chat-history"),
    },
    RecallSource {
        key: "brain",
        description: "Facts the user asked Script Kit to remember",
        prefix: "@brain:",
        proposed: true,
        builtin: None,
    },
    RecallSource {
        key: "calendar",
        description: "Calendar events and meetings",
        prefix: "@calendar:",
        proposed: false,
        builtin: None,
    },
    RecallSource {
        key: "files",
        description: "Files and folders on disk",
        prefix: "@file:",
        proposed: false,
        builtin: Some("builtin/file-search"),
    },
];

pub fn recall_source(key: &str) -> Option<&'static RecallSource> {
    RECALL_SOURCES.iter().find(|source| source.key == key)
}

pub const SUBJECTS: &[(&str, &str)] = &[
    ("selection", "The text the user has selected right now"),
    ("clipboard", "What is on the clipboard"),
    ("page", "The web page the user is looking at"),
    ("window", "The frontmost window as it looks on screen"),
    ("typed", "Only the words the user typed; nothing on screen"),
];

const REUSE_RUBRIC: [&str; 4] = [
    "A one-off tied to this moment's specific content",
    "Might come up again now and then",
    "A routine the user probably repeats",
    "Clearly a recurring routine worth its own keyboard shortcut",
];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Step {
    Command {
        command: Command,
    },
    Menu {
        app: String,
        bundle_id: String,
        path: Vec<String>,
        shortcut: Option<String>,
    },
    Window {
        op: WindowOp,
        app: String,
        /// Authority is acquired before interpretation, never reconstructed from its label.
        #[serde(skip)]
        target: Option<super::window_target::WindowTarget>,
    },
    InspectFilename {
        app: String,
        pid: u32,
        window_id: u32,
        title: String,
        filename: String,
    },
    FocusTab {
        app: String,
        query: String,
    },
    OpenUrl {
        url: String,
        /// The default browser when none.
        app: Option<String>,
    },
    Capture {
        target: String,
        body: String,
        when: Option<String>,
        handler: Option<Command>,
        /// A note's screen context, read when the plan runs: "selection", "clipboard" or "page".
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<String>,
    },
    Remember {
        text: String,
        /// Read when the plan runs, when the words to keep are on screen rather than typed.
        subject: Option<String>,
    },
    Recall {
        source: String,
        query: Option<String>,
    },
    Transform {
        tool: Option<Command>,
        instruction: Option<String>,
        subject: String,
    },
    Ask {
        question: String,
        subject: Option<String>,
    },
    Delegate {
        task: String,
        project: Option<String>,
        subject: Option<String>,
    },
}

impl Step {
    fn app(&self) -> Option<&str> {
        match self {
            Step::Command { command } if command.kind == CommandKind::App => Some(&command.name),
            Step::Window { app, .. } => Some(app),
            Step::OpenUrl { app, .. } => app.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reading {
    pub pathway: Pathway,
    pub probability: f64,
    pub title: String,
    pub steps: Vec<Step>,
    /// The line the user could have typed into the launcher to get the same result.
    pub typed: Option<String>,
    pub typed_is_proposed: bool,
    /// Why Enter can't run this reading yet.
    pub missing: Option<String>,
    pub tip: Option<String>,
}

impl Reading {
    fn new(pathway: Pathway, probability: f64, title: impl Into<String>) -> Self {
        Self {
            pathway,
            probability,
            title: title.into(),
            steps: Vec::new(),
            typed: None,
            typed_is_proposed: false,
            missing: None,
            tip: None,
        }
    }

    fn lacking(mut self, what: impl Into<String>) -> Self {
        self.missing = Some(what.into());
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Slots {
    pub body: Option<String>,
    pub task: Option<String>,
    pub script_steps: Option<String>,
    pub when: Option<String>,
    pub query: Option<String>,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Timing {
    pub ms: u64,
    pub input_tokens: u64,
    pub questions: usize,
    pub rounds: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Plan {
    #[serde(flatten)]
    pub reading: Reading,
    pub request: String,
    pub complete: f64,
    pub reuse: f64,
    pub ready: bool,
    pub alternatives: Vec<Reading>,
    pub slots: Slots,
    /// The seven words as the top reading fills them.
    pub words: Vec<WordRead>,
    pub clauses: Vec<Plan>,
    pub timing: Timing,
}

impl std::ops::Deref for Plan {
    type Target = Reading;
    fn deref(&self) -> &Reading {
        &self.reading
    }
}

impl Plan {
    /// Worth offering "Save as script" for.
    pub fn is_reusable(&self) -> bool {
        self.pathway == Pathway::Automate || (self.reuse >= 1.4 && !self.steps.is_empty())
    }
}

#[derive(Debug, Clone, Default)]
pub struct EngineOptions {
    pub exclude: Vec<Pathway>,
}

/// Labels are Jev's answer keys, so they must be unique; duplicates get a qualifier.
#[derive(Debug, Clone)]
pub struct Labeled<T> {
    entries: Vec<(String, T)>,
}

impl<T> Default for Labeled<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> Labeled<T> {
    fn build(
        items: Vec<T>,
        label: impl Fn(&T) -> String,
        qualifier: impl Fn(&T) -> String,
    ) -> Self {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for item in &items {
            *counts.entry(label(item)).or_default() += 1;
        }
        let mut seen = HashSet::new();
        let mut entries = Vec::with_capacity(items.len());
        for item in items {
            let mut key = label(&item);
            if counts.get(&key).copied().unwrap_or(0) > 1 || key == NONE {
                key = format!("{key} ({})", qualifier(&item));
            }
            if seen.insert(key.clone()) {
                entries.push((key, item));
            }
        }
        Self { entries }
    }

    fn insert_new(&mut self, label: String, item: T) {
        if !self.entries.iter().any(|(existing, _)| *existing == label) {
            self.entries.push((label, item));
        }
    }

    pub fn get(&self, label: &str) -> Option<&T> {
        self.entries
            .iter()
            .find(|(existing, _)| existing == label)
            .map(|(_, item)| item)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.entries
            .iter()
            .map(|(label, item)| (label.as_str(), item))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct Built {
    pub state: Value,
    pub questions: Questions,
    pub words: Vec<Word>,
    pub spans: Vec<Span>,
    pub web: Option<WebAddress>,
    span_labels: HashSet<String>,
    pub builtin: Labeled<Command>,
    pub script: Labeled<Command>,
    pub app: Labeled<Command>,
    pub menu: Labeled<MenuCommand>,
    pub window_op: Labeled<WindowOp>,
    pub window_target: Labeled<String>,
    pub text_tool: Labeled<Command>,
}

fn with_none(mut criteria: Criteria, none: &str) -> Criteria {
    criteria.set(NONE, Some(none.to_string()));
    criteria
}

fn choice(instructions: impl Into<String>, criteria: Criteria) -> Question {
    Question::Choice {
        instructions: instructions.into(),
        criteria,
    }
}

fn noul(instructions: impl Into<String>) -> Question {
    Question::Noul {
        instructions: instructions.into(),
    }
}

fn span_question(spans: &[Span], instructions: &str, none: &str) -> Question {
    let mut criteria = Criteria::default();
    for span in spans {
        criteria.set(span.label.clone(), None);
    }
    choice(instructions, with_none(criteria, none))
}

fn listed<'a>(items: impl IntoIterator<Item = (&'a str, Option<&'a str>)>) -> Criteria {
    let mut criteria = Criteria::default();
    for (label, description) in items {
        criteria.set(label, description.map(str::to_string));
    }
    criteria
}

/// Uses the launcher's matcher, retaining separate budgets for commands, scripts, apps
/// and menus so a busy source cannot crowd out another. Only selected items are cloned.
struct CandidateSearch {
    sentence: String,
    words: Vec<(String, crate::scripts::NucleoCtx)>,
}

impl CandidateSearch {
    fn new(request: &str) -> Self {
        let sentence = format!(" {} ", request.to_lowercase());
        let mut seen = HashSet::new();
        let words = request
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| word.len() > 1)
            .map(str::to_lowercase)
            .filter(|word| {
                ![
                    "the", "my", "me", "please", "a", "an", "to", "and", "it", "this", "that",
                ]
                .contains(&word.as_str())
            })
            .filter(|word| seen.insert(word.clone()))
            .map(|word| {
                let matcher = crate::scripts::NucleoCtx::new(&word);
                (word, matcher)
            })
            .collect();
        Self { sentence, words }
    }

    fn score(&mut self, name: &str, description: &str) -> u64 {
        let name_lower = name.to_lowercase();
        let exact = self.sentence.match_indices(&name_lower).any(|(at, _)| {
            let end = at + name_lower.len();
            !name_lower.is_empty()
                && self.sentence[..at]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !c.is_alphanumeric())
                && self.sentence[end..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric())
        });
        let mut score = if exact { 1_000_000 } else { 0 };
        for (word, matcher) in &mut self.words {
            if name_lower
                .split(|c: char| !c.is_alphanumeric())
                .any(|part| part == word.as_str())
            {
                score += 10_000;
            } else if let Some(fuzzy) = matcher.compact_score(name, word) {
                score += 1_000 + u64::from(fuzzy).min(999);
            }
            if !description.is_empty() && matcher.score(description).is_some() {
                score += 100;
            }
        }
        score
    }

    fn select<'a, T: Clone + 'a>(
        &mut self,
        items: impl Iterator<Item = &'a T>,
        fields: impl Fn(&T) -> (&str, &str),
    ) -> Vec<T> {
        let mut best: Vec<(u64, usize, &T)> = Vec::with_capacity(CANDIDATE_LIMIT + 1);
        for (index, item) in items.enumerate() {
            let (name, description) = fields(item);
            let score = self.score(name, description);
            let at = best.partition_point(|(other, _, _)| *other >= score);
            if at < CANDIDATE_LIMIT {
                best.insert(at, (score, index, item));
                best.truncate(CANDIDATE_LIMIT);
            }
        }
        // Stable catalog order keeps duplicate-label qualifiers and equal-score choices steady.
        best.sort_unstable_by_key(|(_, index, _)| *index);
        best.into_iter().map(|(_, _, item)| item.clone()).collect()
    }
}

pub fn build_questions(
    request: &str,
    ctx: &ScreenContext,
    catalog: &Catalog,
    opts: &EngineOptions,
) -> Built {
    let words = tokenize(request);
    let spans = span_candidates(request, &words, SPAN_LIMIT);
    let mut search = CandidateSearch::new(request);
    let of_kind = |kind| {
        catalog.commands.iter().filter(move |c| {
            c.kind == kind || (kind == CommandKind::Script && c.kind == CommandKind::Scriptlet)
        })
    };
    let builtin = Labeled::build(
        search.select(of_kind(CommandKind::Builtin), |c| (&c.name, &c.description)),
        |c| c.name.clone(),
        |c| c.id.clone(),
    );
    let script = Labeled::build(
        search.select(of_kind(CommandKind::Script), |c| (&c.name, &c.description)),
        |c| c.name.clone(),
        |c| {
            c.plugin.clone().unwrap_or_else(|| match c.kind {
                CommandKind::Scriptlet => "scriptlet".into(),
                _ => "script".into(),
            })
        },
    );
    let app = Labeled::build(
        search.select(of_kind(CommandKind::App), |c| (&c.name, &c.description)),
        |c| c.name.clone(),
        |c| c.id.clone(),
    );
    let menu = Labeled::build(
        search.select(ctx.menus.iter(), |m| (&m.label, &m.menu)),
        MenuCommand::question_label,
        |m| m.shortcut.clone().unwrap_or_else(|| "menu".into()),
    );
    let window_op = Labeled::build(
        catalog.window_ops.clone(),
        |w| w.label.clone(),
        |w| w.id.clone(),
    );
    let front = ctx.front_app.name.clone();
    let mut window_target = Labeled::default();
    window_target.insert_new(format!("the front window ({front})"), front.clone());
    for window in &ctx.open_windows {
        if window.app != front {
            window_target.insert_new(window.app.clone(), window.app.clone());
        }
    }
    let text_tool = Labeled::build(
        search.select(catalog.text_tools.iter(), |c| (&c.name, &c.description)),
        |c| c.name.clone(),
        |c| c.id.clone(),
    );

    let mut menu_names: Vec<&str> = Vec::new();
    for m in &ctx.menus {
        if !menu_names.contains(&m.leaf()) {
            menu_names.push(m.leaf());
        }
    }
    let menu_names: String = menu_names.join(", ").chars().take(700).collect();
    let mut pathways = Criteria::default();
    for pathway in Pathway::ALL {
        if opts.exclude.contains(&pathway) {
            continue;
        }
        let description = if pathway == Pathway::AppCommand {
            format!(
                "{}. {front}'s menus include: {menu_names}",
                pathway.description()
            )
        } else {
            pathway.description().to_string()
        };
        pathways.set(pathway.key(), Some(description));
    }

    let mut questions = Questions::default();
    questions.insert(
        "pathway",
        choice(
            "The user is typing or dictating into Script Kit, a Mac launcher, to say what they want to happen. What kind of action do they want?",
            pathways,
        ),
    );
    questions.insert(
        "complete",
        noul("Has the user finished saying what they want? Answer no if the words stop mid-thought, for example ending on 'to', 'the', 'and', 'about', 'my', or a verb still missing its object."),
    );
    questions.insert(
        "reuse",
        Question::Score {
            instructions: "How likely is the user to want exactly this again, so it's worth saving as a script with a keyboard shortcut?".into(),
            criteria: REUSE_RUBRIC.iter().map(|s| s.to_string()).collect(),
        },
    );
    questions.insert(
        "builtin",
        choice(
            "Which Script Kit command does the user want to run?",
            with_none(
                listed(
                    builtin
                        .iter()
                        .map(|(k, c)| (k, Some(c.description.as_str()))),
                ),
                "No listed command",
            ),
        ),
    );
    questions.insert(
        "script",
        choice(
            "Which of the user's scripts or scriptlets does the user want to run?",
            with_none(
                listed(
                    script
                        .iter()
                        .map(|(k, c)| (k, Some(c.description.as_str()).filter(|d| !d.is_empty()))),
                ),
                "No listed script",
            ),
        ),
    );
    questions.insert(
        "app",
        choice(
            "Which application does the user want to open?",
            with_none(
                listed(app.iter().map(|(k, _)| (k, None))),
                "No listed application",
            ),
        ),
    );
    let menu_shortcuts: Vec<(String, Option<String>)> = menu
        .iter()
        .map(|(k, m)| {
            (
                k.to_string(),
                m.shortcut.as_ref().map(|s| format!("shortcut {s}")),
            )
        })
        .collect();
    let mut menu_criteria = Criteria::default();
    for (label, description) in menu_shortcuts {
        menu_criteria.set(label, description);
    }
    questions.insert(
        "menu",
        choice(
            format!("Which {front} menu command does the request ask for?"),
            with_none(menu_criteria, &format!("No {front} menu command does it")),
        ),
    );
    questions.insert(
        "window_op",
        choice(
            "If the user wants to do something to a window, like move, resize, tile, minimize, or close it, or to quit an app, which action? Go by the words the user typed, not by what they did last.",
            with_none(
                listed(
                    window_op
                        .iter()
                        .map(|(k, w)| (k, Some(w.description.as_str()).filter(|d| !d.is_empty()))),
                ),
                "Not about arranging a window",
            ),
        ),
    );
    questions.insert(
        "window_target",
        choice(
            "Which app's window should be acted on? Choose the front window if the user says 'this' or names no app, and every window only when the user means all of them at once.",
            with_none(
                listed(
                    window_target
                        .iter()
                        .map(|(k, _)| (k, None))
                        .chain([(EVERY_WINDOW, Some("All windows at once, not one app's window"))]),
                ),
                "No window is meant",
            ),
        ),
    );
    questions.insert(
        "capture_target",
        choice(
            "If the user wants to write something down for later, where does it go?",
            with_none(
                listed(CAPTURE_TARGETS.iter().map(|(k, d)| (*k, Some(*d)))),
                "Not writing anything down",
            ),
        ),
    );
    questions.insert("body", span_question(&spans, "Which exact words are the content itself: the thing to save, remember, ask, or hand off? Leave out command words like 'remind me to', 'remember that', 'ask an agent to', and leave out time words.", "No words are the content"));
    questions.insert(
        "when",
        span_question(
            &spans,
            "Which exact words say when: a date or a time?",
            "No time is mentioned",
        ),
    );
    questions.insert("query", span_question(&spans, "If the user is looking for something they saw, copied, said, or saved before, which words are just the topic? For 'what did I copy about the invoice' the topic is 'the invoice'; for 'that video on sourdough I watched' it is 'sourdough'.", "No search words"));
    questions.insert("task", span_question(&spans, "Which exact words are the question or job itself, keeping its verb? Leave out only who should do it, like 'have an agent', 'ask claude to', 'hey'. For 'have an agent fix the build' the job is 'fix the build'.", "No question or job"));
    questions.insert("script_steps", span_question(&spans, "If the user is making a script or shortcut, which exact words say what it should do? Leave out the shortcut keys and words like 'make a script that' or 'bound to'.", "Not making a script, or no steps are described"));
    questions.insert(
        "shortcut",
        span_question(
            &spans,
            "Which exact words name a keyboard shortcut, like 'cmd shift L' or 'control option T'?",
            "No keyboard shortcut is named",
        ),
    );
    questions.insert(
        "recall_source",
        choice(
            "If the user is looking for something from their own past, where should Script Kit look?",
            with_none(
                listed(RECALL_SOURCES.iter().map(|s| (s.key, Some(s.description)))),
                "Not looking for something from the past",
            ),
        ),
    );
    let mut transform = listed(
        text_tool
            .iter()
            .map(|(k, c)| (k, Some(c.description.as_str()))),
    );
    transform.set(
        "custom",
        Some("Some other change the user describes in their own words".into()),
    );
    questions.insert(
        "transform",
        choice(
            "If the user wants existing text changed or explained, which tool fits?",
            with_none(transform, "Not changing or explaining existing text"),
        ),
    );
    questions.insert(
        "subject",
        choice(
            "What existing thing does the request act on or refer to?",
            listed(SUBJECTS.iter().map(|(k, d)| (*k, Some(*d)))),
        ),
    );
    questions.insert(
        "project",
        choice(
            "Which project folder should an AI agent work in?",
            with_none(
                listed(ctx.projects.iter().map(|p| (p.as_str(), None))),
                "No project is meant",
            ),
        ),
    );
    questions.insert(
        "compound",
        noul("Would Script Kit need two or more separate steps to do this, like moving two different windows to two places, or opening an app and then arranging it? Answer no when it is one step that mentions several things, like one todo 'buy milk and eggs' or one AI job 'research X and write a summary'."),
    );
    questions.insert(
        "refers_back",
        noul("Does the request point back at the last thing the user did (last_thing_user_did), with words like 'that', 'what I just did', or 'do that again', rather than at something on screen?"),
    );

    let span_labels = spans.iter().map(|span| span.label.clone()).collect();
    Built {
        state: describe(ctx, request),
        questions,
        words,
        spans,
        web: web_address(request),
        span_labels,
        builtin,
        script,
        app,
        menu,
        window_op,
        window_target,
        text_tool,
    }
}

pub(super) fn choice_answer<'a>(
    answers: &'a Answers,
    name: &str,
) -> Option<(&'a str, &'a Probabilities)> {
    match answers.get(name)? {
        Answer::Choice {
            choice,
            probabilities,
            ..
        } => Some((choice.as_str(), probabilities)),
        _ => None,
    }
}

pub(super) fn picked<'a>(answers: &'a Answers, name: &str) -> Option<&'a str> {
    choice_answer(answers, name)
        .map(|(choice, _)| choice)
        .filter(|choice| *choice != NONE)
}

/// Only a run of the user's own words; Jev cannot put words in their mouth.
pub(super) fn picked_span<'a>(answers: &'a Answers, b: &Built, name: &str) -> Option<&'a str> {
    picked(answers, name).filter(|label| b.span_labels.contains(*label))
}

fn picked_from<'a>(answers: &'a Answers, name: &str, known: &[(&str, &str)]) -> Option<&'a str> {
    picked(answers, name).filter(|label| known.iter().any(|(key, _)| key == label))
}

fn ranked<'a>(answers: &'a Answers, name: &str) -> Vec<(&'a str, f64)> {
    choice_answer(answers, name)
        .map(|(_, probabilities)| probabilities.ranked())
        .unwrap_or_default()
}

fn noul_answer(answers: &Answers, name: &str) -> Option<f64> {
    match answers.get(name)? {
        Answer::Noul { noul } => Some(*noul),
        _ => None,
    }
}

fn score_answer(answers: &Answers, name: &str) -> Option<f64> {
    match answers.get(name)? {
        Answer::Score { score, .. } => Some(*score),
        _ => None,
    }
}

/// Each kind has its own question with its own "none", so answers compare directly. A
/// separate "which kind?" question answered without seeing the names was the weakest
/// link in testing, so there isn't one.
/// "Copy Password" is two words the user can say; "1Password" is one.
fn name_words(name: &str) -> impl Iterator<Item = String> + '_ {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
}

pub fn rank_commands<'a>(b: &'a Built, answers: &Answers) -> Vec<(&'a Command, f64)> {
    let mut out = Vec::new();
    for (question, table) in [
        ("builtin", &b.builtin),
        ("script", &b.script),
        ("app", &b.app),
    ] {
        for (label, p) in ranked(answers, question) {
            if let Some(command) = table.get(label) {
                out.push((command, p));
            }
        }
    }
    out.sort_by(|x, y| y.1.total_cmp(&x.1));
    if let Some(&(_, top)) = out.first() {
        let spoken: HashSet<String> = b.words.iter().map(|w| w.text.to_lowercase()).collect();
        let said = |command: &Command| {
            name_words(&command.name)
                .filter(|word| spoken.contains(word))
                .count()
        };
        let winner = out
            .iter()
            .enumerate()
            .filter(|(_, (_, p))| top - p <= NEAR_TIE)
            .min_by_key(|(i, (command, _))| {
                (
                    std::cmp::Reverse(said(command)),
                    command.kind.tie_rank(),
                    *i,
                )
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        let best = out.remove(winner);
        out.insert(0, best);
    }
    out
}

fn not_none(answers: &Answers, name: &str) -> f64 {
    1.0 - choice_answer(answers, name)
        .and_then(|(_, probabilities)| probabilities.get(NONE))
        .unwrap_or(0.0)
}

/// How well a pathway's own target question found something; pathways without a
/// target list score 1.
pub fn target_fit(pathway: Pathway, b: &Built, answers: &Answers) -> f64 {
    match pathway {
        Pathway::Run if b.web.is_some() => 1.0,
        Pathway::Run => rank_commands(b, answers)
            .first()
            .map(|(_, p)| *p)
            .filter(|p| *p >= RUN_FLOOR)
            .unwrap_or(0.0),
        Pathway::AppCommand => not_none(answers, "menu"),
        Pathway::Window => not_none(answers, "window_op"),
        _ => 1.0,
    }
}

/// The pathway question only sees short descriptions, so "gimme a new password" reads
/// as "none" even when the script question is sure about Generate Password. A strong
/// run or menu match takes over the "none" share. Window arrangements are left out:
/// that question still picks a layout for requests like "lol".
pub fn rescue_from_none(
    order: Vec<(Pathway, f64)>,
    b: &Built,
    answers: &Answers,
) -> Vec<(Pathway, f64)> {
    let none = order
        .iter()
        .find(|(k, _)| *k == Pathway::None)
        .map_or(0.0, |(_, p)| *p);
    if none == 0.0 {
        return order;
    }
    let run = rank_commands(b, answers).first().map_or(0.0, |(_, p)| *p);
    let menu = target_fit(Pathway::AppCommand, b, answers);
    let (winner, fit) = if menu > run {
        (Pathway::AppCommand, menu)
    } else {
        (Pathway::Run, run)
    };
    if fit < STRONG_TARGET {
        return order;
    }
    let moved = none * fit;
    order
        .into_iter()
        .map(|(k, p)| {
            if k == Pathway::None {
                (k, p - moved)
            } else if k == winner {
                (k, p + moved)
            } else {
                (k, p)
            }
        })
        .collect()
}

pub fn resolve_last_steps(last: &LastAction, catalog: &Catalog) -> Vec<Step> {
    last.steps
        .iter()
        .filter_map(|step| match step {
            LastStep::Window { op_label, app } => catalog
                .window_ops
                .iter()
                .find(|w| w.label == *op_label)
                .map(|op| Step::Window {
                    op: op.clone(),
                    app: app.clone(),
                    // LastStep is a recipe, not authority over a live window.
                    target: None,
                }),
            LastStep::Command { id } => catalog.command(id).map(|command| Step::Command {
                command: command.clone(),
            }),
            LastStep::Menu {
                app,
                bundle_id,
                path,
            } => Some(Step::Menu {
                app: app.clone(),
                bundle_id: bundle_id.clone(),
                path: path.clone(),
                shortcut: None,
            }),
            LastStep::OpenUrl { url, app } => Some(Step::OpenUrl {
                url: url.clone(),
                app: app.clone(),
            }),
        })
        .collect()
}

fn quote(text: &str) -> String {
    if text.chars().any(char::is_whitespace) {
        format!("\"{}\"", text.replace('"', "'"))
    } else {
        text.to_string()
    }
}

static URL_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"https?://\S+").ok());
/// Words that name no steps of their own, only the last thing the user did.
static BACK_REFERENCE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)^(do )?(that|this|it|what i (just )?did)( again)?$").ok());
/// An automate reading whose steps come from a second round, one per clause.
const STEPS_NEXT: &str = "the steps (decomposed next)";
static POINTS_BACK: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)\b(it|that|them)\b").ok());

fn pattern_matches(pattern: &LazyLock<Option<Regex>>, text: &str) -> bool {
    pattern.as_ref().is_some_and(|regex| regex.is_match(text))
}

fn nonempty(text: Option<&str>) -> Option<&str> {
    text.filter(|t| !t.is_empty())
}

/// Words that belong with the time after them, and that the launcher's date field
/// still reads: "in an hour", "next friday".
const TIME_LEAD_INS: &[&str] = &["in", "at", "next", "this"];
/// Openers that say "keep this" rather than what to keep.
const REMEMBER_OPENERS: &[&str] = &[
    "remember", "that", "fyi", "btw", "note", "to", "self", "keep", "in", "mind", "please", "hey",
    "ok", "okay", "so", "just", "save", "memorize",
];
/// Verbs that move a window; with only an app named, the user isn't done.
const PLACEMENT_VERBS: &[&str] = &[
    "put", "move", "send", "drag", "throw", "place", "snap", "tile", "push", "shove",
];

fn one_of(word: &Word, set: &[&str]) -> bool {
    set.contains(&word.text.to_lowercase().as_str())
}

fn span_of<'a>(b: &'a Built, label: &str) -> Option<&'a Span> {
    b.spans.iter().find(|s| s.label == label)
}

fn widen(request: &str, b: &Built, from: usize, to: usize) -> String {
    request[b.words[from].start..b.words[to - 1].end].to_string()
}

/// Window steps that end something, so a named app must be the one they end.
const QUIT_APP: &str = "Quit App";
const CLOSE_WINDOW: &str = "Close Window";
/// Words that point at the front window, the only one ended without being named.
const FRONT_WORDS: &[&str] = &[
    "this",
    "it",
    "that",
    "current",
    "front",
    "frontmost",
    "window",
    "app",
    "here",
];

/// The catalog app whose whole name the user said, the longest when several match.
fn spoken_app<'a>(b: &Built, catalog: &'a Catalog) -> Option<&'a str> {
    let spoken = format!(
        " {} ",
        b.words
            .iter()
            .map(|w| w.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let utf16_len = |s: &str| s.encode_utf16().count();
    let mut best: Option<&str> = None;
    for command in &catalog.commands {
        if command.kind != CommandKind::App
            || !spoken.contains(&format!(" {} ", command.name.to_lowercase()))
        {
            continue;
        }
        if best.is_none_or(|best| utf16_len(&command.name) > utf16_len(best)) {
            best = Some(&command.name);
        }
    }
    best
}

/// How sure Jev must be of an app the user named before it beats the front window.
const NAMED_APP_FLOOR: f64 = 0.2;

/// Jev leans toward the front window even when the words name another open app, as in
/// "send messages to the bottom right corner"; a named app it also considered wins.
fn named_window_app<'a>(b: &'a Built, answers: &Answers, front: &str) -> Option<&'a str> {
    let spoken = format!(
        " {} ",
        b.words
            .iter()
            .map(|w| w.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ")
    );
    ranked(answers, "window_target")
        .into_iter()
        .filter(|(_, p)| *p >= NAMED_APP_FLOOR)
        .filter_map(|(label, _)| b.window_target.get(label))
        .find(|app| app.as_str() != front && spoken.contains(&format!(" {} ", app.to_lowercase())))
        .map(String::as_str)
}

/// "an hour" in "remind me in an hour" keeps its "in", unless the body already has it.
pub(super) fn with_time_lead_in(
    request: &str,
    b: &Built,
    when: &str,
    body: Option<&str>,
) -> String {
    let Some(span) = span_of(b, when).filter(|s| s.from > 0) else {
        return when.to_string();
    };
    let before = span.from - 1;
    let body_has_it = body
        .and_then(|body| span_of(b, body))
        .is_some_and(|s| s.from <= before && before < s.to);
    if !body_has_it && one_of(&b.words[before], TIME_LEAD_INS) {
        widen(request, b, before, span.to)
    } else {
        when.to_string()
    }
}

/// A one- or two-word fact ("pinecone42") keeps what it is about.
fn with_what_its_about(request: &str, b: &Built, body: &str) -> String {
    let Some(span) = span_of(b, body).filter(|s| s.to - s.from <= 2 && b.words.len() >= 5) else {
        return body.to_string();
    };
    match b.words.iter().position(|w| !one_of(w, REMEMBER_OPENERS)) {
        Some(start) if start < span.from => widen(request, b, start, span.to),
        _ => body.to_string(),
    }
}

/// A web address in the user's words: a full URL, or a bare domain like "johnlindquist.com".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAddress {
    /// As typed.
    pub said: String,
    /// Byte offsets into the request.
    pub range: Range<usize>,
    /// What the browser is handed.
    pub url: String,
}

/// Top-level domains only, so "deploy.sh", "Notes.app" and "index.ts" stay file names.
static WEB_ADDRESS: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bhttps?://[^\s<>]+|\b(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+(?:com|org|net|io|dev|ai|co|me|gg|tv|fm|ly|xyz|info|biz|edu|gov|us|uk|ca|de|fr|jp|au|nl)\b(?::\d+)?(?:/[^\s<>]*)?",
    )
    .ok()
});

pub fn web_address(request: &str) -> Option<WebAddress> {
    let regex = WEB_ADDRESS.as_ref()?;
    regex.find_iter(request).find_map(|found| {
        if matches!(request[..found.start()].chars().last(), Some('@' | '/')) {
            return None;
        }
        let said = found
            .as_str()
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '}', '"', '\'']);
        let lower = said.to_ascii_lowercase();
        let has_scheme = lower.starts_with("http://") || lower.starts_with("https://");
        Some(WebAddress {
            said: said.to_string(),
            range: found.start()..found.start() + said.len(),
            url: if has_scheme {
                said.to_string()
            } else {
                format!("https://{said}")
            },
        })
    })
}

/// Browsers that open a web address handed to them in a new tab, by name and bundle id.
const BROWSERS: &[(&str, &str)] = &[
    ("Safari", "com.apple.Safari"),
    (
        "Safari Technology Preview",
        "com.apple.SafariTechnologyPreview",
    ),
    ("Google Chrome", "com.google.Chrome"),
    ("Google Chrome Canary", "com.google.Chrome.canary"),
    ("Arc", "company.thebrowser.Browser"),
    ("Dia", "company.thebrowser.dia"),
    ("Brave Browser", "com.brave.Browser"),
    ("Microsoft Edge", "com.microsoft.edgemac"),
    ("Chromium", "org.chromium.Chromium"),
    ("Vivaldi", "com.vivaldi.Vivaldi"),
    ("Opera", "com.operasoftware.Opera"),
    ("Firefox", "org.mozilla.firefox"),
    (
        "Firefox Developer Edition",
        "org.mozilla.firefoxdeveloperedition",
    ),
    ("Zen", "app.zen-browser.zen"),
    ("Orion", "com.kagi.kagimacOS"),
];

pub fn is_browser(name: &str, bundle_id: &str) -> bool {
    BROWSERS
        .iter()
        .any(|(n, id)| n.eq_ignore_ascii_case(name) || (!bundle_id.is_empty() && *id == bundle_id))
}

/// Words that say "open it" without naming what, so they name no command or menu item.
const LAUNCH_WORDS: &[&str] = &[
    "open", "go", "to", "visit", "load", "launch", "show", "pull", "up", "bring", "navigate",
    "browse", "new", "tab", "window", "in", "the", "a", "an", "please", "website", "site", "page",
    "url", "link", "location",
];

pub(super) fn web_title(said: &str, app: Option<&str>) -> String {
    match app {
        Some(app) => format!("Open {said} in {app}"),
        None => format!("Open {said}"),
    }
}

/// Neither the catalog nor the front app's menus can say "go to this address", so Jev reads
/// "open a new tab to johnlindquist.com" as opening Chrome or its Open Location menu item. The
/// address is what to open, unless the user named the command or menu item they meant.
fn opens_web_address(
    reading: Reading,
    ctx: &ScreenContext,
    catalog: &Catalog,
    b: &Built,
) -> Reading {
    let Some(web) = &b.web else {
        return reading;
    };
    let said: HashSet<String> = b
        .words
        .iter()
        .filter(|w| w.end <= web.range.start || w.start >= web.range.end)
        .map(|w| w.text.to_lowercase())
        .collect();
    let said_of = |name: &str| {
        name_words(name)
            .filter(|word| {
                word.len() > 2 && !LAUNCH_WORDS.contains(&word.as_str()) && said.contains(word)
            })
            .count()
    };
    let takes_over = match (reading.pathway, reading.steps.first()) {
        (Pathway::Run, Some(Step::Command { command })) => {
            is_browser(&command.name, "") || said_of(&command.name) == 0
        }
        (Pathway::AppCommand, Some(Step::Menu { path, .. })) => {
            path.last().is_none_or(|leaf| said_of(leaf) == 0)
        }
        (Pathway::Run | Pathway::AppCommand, _) => true,
        (Pathway::None, _) => said
            .iter()
            .all(|word| LAUNCH_WORDS.contains(&word.as_str())),
        _ => false,
    };
    if !takes_over {
        return reading;
    }
    let front = &ctx.front_app;
    let front_is_browser = is_browser(&front.name, &front.bundle_id)
        || ctx.url.as_deref().is_some_and(|url| !url.is_empty());
    let app = catalog
        .commands
        .iter()
        .filter(|c| c.kind == CommandKind::App && is_browser(&c.name, ""))
        .filter(|c| said_of(&c.name) > 0)
        .max_by_key(|c| {
            (
                said_of(&c.name),
                std::cmp::Reverse(name_words(&c.name).count()),
            )
        })
        .map(|c| c.name.clone())
        .or_else(|| (front_is_browser && !front.name.is_empty()).then(|| front.name.clone()));
    let mut open = Reading::new(
        Pathway::Run,
        reading.probability,
        web_title(&web.said, app.as_deref()),
    );
    open.steps = vec![Step::OpenUrl {
        url: web.url.clone(),
        app,
    }];
    open
}

/// A named existing tab is neither an app launch nor a menu command. Resolve only explicit
/// focus/switch requests against a known browser; never send the browser's tab inventory to Jev.
pub fn named_tab_plan(request: &str, browser: &str) -> Option<Plan> {
    static NAMED: std::sync::LazyLock<Option<regex::Regex>> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)^(?:focus(?: on)?|switch to|go to)\s+(?:the\s+)?(.+?)\s+tab[.!]?$")
            .ok()
    });
    let app = crate::browser_tabs::named_tab_browser(browser)?;
    let cap = NAMED.as_ref()?.captures(request.trim())?;
    let query = cap
        .get(1)?
        .as_str()
        .trim()
        .trim_matches(['“', '”', '"'])
        .trim();
    if query.is_empty()
        || query.len() > 256
        || query.chars().any(char::is_control)
        || matches!(
            query.to_lowercase().as_str(),
            "next"
                | "previous"
                | "last"
                | "current"
                | "new"
                | "another"
                | "the"
                | "first"
                | "second"
                | "third"
        )
    {
        return None;
    }
    if query
        .split_whitespace()
        .any(|w| w.eq_ignore_ascii_case("then"))
    {
        return None;
    }
    use super::words::{WordKind, WordSource};
    let mark = |kind, said: &str| WordRead {
        kind,
        value: said.to_owned(),
        sub: "From your words".into(),
        said: Some(said.to_owned()),
        source: WordSource::Words,
        probability: None,
        dim: false,
    };
    let verb = request.split_whitespace().next()?;
    let words = vec![mark(WordKind::Action, verb), mark(WordKind::Target, query)];
    let mut reading = Reading::new(
        Pathway::AppCommand,
        1.0,
        format!("Focus “{query}” tab in {app}"),
    );
    reading.steps.push(Step::FocusTab {
        app: app.into(),
        query: query.into(),
    });
    Some(Plan {
        reading,
        request: request.into(),
        complete: 1.,
        reuse: 0.,
        ready: true,
        alternatives: vec![],
        slots: Default::default(),
        words,
        clauses: vec![],
        timing: Default::default(),
    })
}

pub fn reading_for(
    pathway: Pathway,
    probability: f64,
    request: &str,
    ctx: &ScreenContext,
    catalog: &Catalog,
    b: &Built,
    answers: &Answers,
) -> Reading {
    let body = picked_span(answers, b, "body");
    let task = picked_span(answers, b, "task");
    let when =
        picked_span(answers, b, "when").map(|when| with_time_lead_in(request, b, when, body));
    let when = when.as_deref();
    let query = picked_span(answers, b, "query");
    let subject = picked_from(answers, "subject", SUBJECTS).unwrap_or("typed");
    let subject_text = match subject {
        "selection" => nonempty(ctx.selection.as_deref()),
        "clipboard" => nonempty(ctx.clipboard.as_deref()),
        _ => None,
    };
    let front = ctx.front_app.name.as_str();
    let base = |title: String| Reading::new(pathway, probability, title);

    match pathway {
        Pathway::Run => {
            let ranking = rank_commands(b, answers);
            let Some(&(command, _)) = ranking.first().filter(|(_, p)| *p >= RUN_FLOOR) else {
                return base("Run a command".into()).lacking("which command to run");
            };
            let mut reading = base(if command.kind == CommandKind::App {
                format!("Open {}", command.name)
            } else {
                command.name.clone()
            });
            reading.steps = vec![Step::Command {
                command: command.clone(),
            }];
            reading.typed = Some(command.name.clone());
            if command.kind == CommandKind::App
                && b.words.first().is_some_and(|w| one_of(w, PLACEMENT_VERBS))
            {
                reading.missing = Some("where to put it".into());
            }
            reading
        }
        Pathway::AppCommand => {
            let Some(m) = picked(answers, "menu").and_then(|label| b.menu.get(label)) else {
                return base(format!("Do it in {front}"))
                    .lacking(format!("a {front} menu command"));
            };
            let mut reading = base(format!("{} in {front}", m.leaf()));
            reading.steps = vec![Step::Menu {
                app: front.to_string(),
                bundle_id: ctx.front_app.bundle_id.clone(),
                path: m.path(),
                shortcut: m.shortcut.clone(),
            }];
            reading.tip = m
                .shortcut
                .as_ref()
                .map(|shortcut| format!("{shortcut} does this directly in {front}"));
            reading
        }
        Pathway::Window => {
            let op = picked(answers, "window_op").and_then(|label| b.window_op.get(label));
            if picked(answers, "window_target") == Some(EVERY_WINDOW) {
                let what = op.map_or("Arrange", |op| op.label.as_str());
                return base(format!("{what}: every window"))
                    .lacking("a way to act on every window at once");
            }
            let app = picked(answers, "window_target")
                .and_then(|label| b.window_target.get(label))
                .filter(|app| !app.is_empty())
                .map_or(front, String::as_str);
            let mut app = if app == front {
                named_window_app(b, answers, front).unwrap_or(app)
            } else {
                app
            };
            let Some(op) = op else {
                return base("Arrange a window".into()).lacking("how to arrange the window");
            };
            // Ending the front app is never the fallback: "quit spotify" with no Spotify
            // window on screen quits Spotify or nothing.
            if op.label == QUIT_APP || op.label == CLOSE_WINDOW {
                let quits = op.label == QUIT_APP;
                match spoken_app(b, catalog) {
                    Some(named) if named.to_lowercase() != app.to_lowercase() => {
                        let open = ctx
                            .open_windows
                            .iter()
                            .any(|w| w.app.eq_ignore_ascii_case(named));
                        if !quits && !open {
                            return base(op.title_for(named))
                                .lacking(format!("an open {named} window"));
                        }
                        app = named;
                    }
                    Some(_) => {}
                    None if app == front && b.words.iter().any(|w| one_of(w, FRONT_WORDS)) => {}
                    None if quits => {
                        return base("Quit an app".into()).lacking("which app to quit");
                    }
                    None => {
                        return base("Close a window".into()).lacking("which window to close");
                    }
                }
            }
            // A model-selected name is not authority to move a different application. The
            // words must actually name it ("Chrome" also names "Google Chrome"); conversely,
            // never substitute the front window when the request explicitly named another app.
            let named = spoken_app(b, catalog);
            let says_app = named.is_some_and(|n| n.eq_ignore_ascii_case(app))
                || app
                    .split_whitespace()
                    .last()
                    .is_some_and(|tail| b.words.iter().any(|w| w.text.eq_ignore_ascii_case(tail)));
            let target = if named.is_some_and(|n| !n.eq_ignore_ascii_case(app))
                || (app != front && !says_app)
            {
                None
            } else {
                ctx.window_snapshot.bind(app, app == front && !says_app)
            };
            let title = op.title_for(app);
            let mut reading = base(title.clone());
            reading.steps = vec![Step::Window {
                op: op.clone(),
                app: app.to_string(),
                target,
            }];
            reading.typed = Some(title);
            reading
        }
        Pathway::Capture => {
            let target = picked_from(answers, "capture_target", CAPTURE_TARGETS).unwrap_or("todo");
            let handler = catalog
                .commands
                .iter()
                .find(|c| c.capture_targets.iter().any(|t| t == target))
                .cloned();
            let url = body
                .and_then(|b| URL_PATTERN.as_ref()?.find(b))
                .or_else(|| URL_PATTERN.as_ref()?.find(request))
                .map(|m| m.as_str().to_string())
                .or_else(|| {
                    (target == "link")
                        .then(|| ctx.url.clone())
                        .flatten()
                        .filter(|u| !u.is_empty())
                });
            // A note can take what's on screen along: the words become its title.
            let note_subject =
                (target == "note")
                    .then_some(subject)
                    .filter(|subject| match *subject {
                        "selection" | "clipboard" => {
                            subject_text.is_some() || ctx.subject_read_at_run
                        }
                        "page" => {
                            ctx.url.as_deref().is_some_and(|url| !url.is_empty())
                                || ctx.subject_read_at_run
                        }
                        _ => false,
                    });
            let text = if target == "link" {
                url.unwrap_or_default()
            } else if let (Some(subject), None) = (note_subject, body) {
                format!("the {subject}")
            } else {
                body.unwrap_or_default().to_string()
            };
            let date_field = if target == "cal" { "start" } else { "due" };
            let dated = when.filter(|_| target != "link" && target != "note");
            let typed = format!(
                "{target}; {text}{}",
                dated.map_or(String::new(), |w| format!(" {date_field}:{}", quote(w)))
            );
            let noun = match target {
                "reminder" => "reminder",
                "cal" => "event",
                "note" => "note",
                "link" => "link",
                "social" => "post draft",
                _ => "todo",
            };
            let missing = if text.is_empty() {
                Some(if target == "link" {
                    "a link to save"
                } else {
                    "what to write down"
                })
            } else if target == "cal" && when.is_none() {
                Some("a date and time")
            } else {
                None
            };
            let shown_when = when.filter(|_| target != "link");
            let mut reading = base(format!(
                "Add {noun}: {}{}{}",
                if text.is_empty() { "…" } else { &text },
                shown_when.map_or(String::new(), |w| format!(" ({w})")),
                match (note_subject, body) {
                    (Some(subject), Some(_)) => format!(" + the {subject}"),
                    _ => String::new(),
                }
            ));
            if !text.is_empty() {
                reading.steps = vec![Step::Capture {
                    target: target.to_string(),
                    body: text.clone(),
                    when: when.map(str::to_string),
                    handler,
                    subject: note_subject.map(str::to_string),
                }];
            }
            reading.typed = Some(typed);
            reading.missing = missing.map(str::to_string);
            reading
        }
        Pathway::Remember => {
            let read_later = body.is_none()
                && ctx.subject_read_at_run
                && matches!(subject, "selection" | "clipboard");
            let fact = body.map(|body| with_what_its_about(request, b, body));
            let text = fact.as_deref().or(subject_text);
            let mut reading = match (text, read_later) {
                (Some(text), _) => base(format!("Remember: {text}")),
                (None, true) => base(format!("Remember the {subject}")),
                (None, false) => base("Remember: …".into()).lacking("what to remember"),
            };
            if text.is_some() || read_later {
                reading.steps = vec![Step::Remember {
                    text: text.unwrap_or_default().to_string(),
                    subject: read_later.then(|| subject.to_string()),
                }];
            }
            reading.typed = Some(format!("brain; {}", text.unwrap_or_default()));
            reading.typed_is_proposed = true;
            reading
        }
        Pathway::Recall => {
            let info = picked(answers, "recall_source").and_then(recall_source);
            let source = info.map(|info| info.key);
            let place = info.map_or("everything".to_string(), |info| {
                info.description.to_lowercase()
            });
            let mut reading = base(match query {
                Some(q) => format!("Search {place} for “{q}”"),
                None => format!("Browse {place}"),
            });
            reading.steps = vec![Step::Recall {
                source: source.unwrap_or("all").to_string(),
                query: query.map(str::to_string),
            }];
            reading.typed = Some(match info {
                // A space would end the `@notes:` token and leave its picker, so the words are
                // joined the way picked notes are written; notes search reads `-` as a gap.
                Some(info) if info.key == "notes" => format!(
                    "{}{}",
                    info.prefix,
                    query
                        .unwrap_or_default()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join("-")
                ),
                Some(info) => format!("{}{}", info.prefix, query.unwrap_or_default()),
                None => query.unwrap_or_default().to_string(),
            });
            reading.typed_is_proposed = info.is_some_and(|i| i.proposed);
            reading
        }
        Pathway::Transform => {
            let tool = picked(answers, "transform")
                .filter(|label| *label != "custom")
                .and_then(|label| b.text_tool.get(label));
            let target = if subject == "typed" {
                if ctx.subject_read_at_run || nonempty(ctx.selection.as_deref()).is_some() {
                    "selection"
                } else {
                    "clipboard"
                }
            } else {
                subject
            };
            let available = ctx.subject_read_at_run
                || match target {
                    "selection" => nonempty(ctx.selection.as_deref()).is_some(),
                    "clipboard" => nonempty(ctx.clipboard.as_deref()).is_some(),
                    _ => true,
                };
            let mut reading = base(match tool {
                Some(tool) => format!("{} ({target})", tool.name),
                None => format!("Rewrite the {target}: {request}"),
            });
            reading.steps = vec![Step::Transform {
                tool: tool.cloned(),
                instruction: tool.is_none().then(|| request.to_string()),
                subject: target.to_string(),
            }];
            reading.typed = Some(format!(
                "@{target} {}",
                tool.map_or(request.to_string(), |t| t.name.to_lowercase())
            ));
            if !available {
                reading.missing = Some(format!("some {target} to work on"));
            }
            reading
        }
        Pathway::Ask => {
            let question = task.unwrap_or(request);
            let attach = (subject != "typed").then_some(subject);
            let mut reading = base(format!("Ask AI: {question}"));
            reading.steps = vec![Step::Ask {
                question: question.to_string(),
                subject: attach.map(str::to_string),
            }];
            reading.typed = Some(format!(
                "{}{question} ⇥",
                attach.map_or(String::new(), |a| format!("@{a} "))
            ));
            if task.is_none() {
                reading.missing = Some("the question".into());
            }
            reading
        }
        Pathway::Delegate => {
            let job = task.unwrap_or(request);
            let project =
                picked(answers, "project").filter(|p| ctx.projects.iter().any(|known| known == p));
            let attach = (subject != "typed").then_some(subject);
            let mut reading = base(format!(
                "Hand to an agent: {job}{}",
                project.map_or(String::new(), |p| format!(" (in {p})"))
            ));
            reading.steps = vec![Step::Delegate {
                task: job.to_string(),
                project: project.map(str::to_string),
                subject: attach.map(str::to_string),
            }];
            reading.typed = Some(format!(
                "{}{}{job}",
                project.map_or(String::new(), |p| format!(">:{p} ")),
                attach.map_or(String::new(), |a| format!("@{a} "))
            ));
            if task.is_none() {
                reading.missing = Some("what the agent should do".into());
            }
            reading
        }
        Pathway::Automate => {
            let refers_back = noul_answer(answers, "refers_back").unwrap_or(0.0);
            let script_steps = picked_span(answers, b, "script_steps");
            let points_back = refers_back >= 0.5
                || pattern_matches(&BACK_REFERENCE, body.unwrap_or_default())
                || pattern_matches(&BACK_REFERENCE, script_steps.unwrap_or_default());
            if let Some(last) = ctx
                .last_action
                .as_ref()
                .filter(|_| points_back || script_steps.is_none())
            {
                let mut reading = base(format!("Save as script: {}", last.summary));
                reading.steps = resolve_last_steps(last, catalog);
                if reading.steps.is_empty() {
                    reading.missing = Some("steps Script Kit can replay".into());
                }
                return reading;
            }
            if points_back {
                return base("Save as script: what you just did".into())
                    .lacking("something done first to save");
            }
            base(format!("New script: {}", body.unwrap_or(request))).lacking(STEPS_NEXT)
        }
        Pathway::None => base("Keep typing…".into()).lacking("a request"),
    }
}

pub fn compose_plan(
    request: &str,
    ctx: &ScreenContext,
    catalog: &Catalog,
    b: &Built,
    answers: &Answers,
) -> Plan {
    let pathways: Vec<(Pathway, f64)> = ranked(answers, "pathway")
        .into_iter()
        .filter_map(|(key, p)| Pathway::from_key(key).map(|pathway| (pathway, p)))
        .collect();
    let weighted: Vec<(Pathway, f64)> = rescue_from_none(pathways, b, answers)
        .into_iter()
        .map(|(pathway, p)| (pathway, p * target_fit(pathway, b, answers)))
        .collect();
    let total: f64 = weighted.iter().map(|(_, p)| p).sum();
    let total = if total > 0.0 { total } else { 1.0 };
    let mut order: Vec<(Pathway, f64)> = weighted
        .into_iter()
        .map(|(pathway, p)| (pathway, p / total))
        .collect();
    order.sort_by(|x, y| y.1.total_cmp(&x.1));
    let mut readings: Vec<Reading> = Vec::with_capacity(order.len());
    for (pathway, p) in order {
        let reading = opens_web_address(
            reading_for(pathway, p, request, ctx, catalog, b, answers),
            ctx,
            catalog,
            b,
        );
        match readings
            .iter_mut()
            .find(|r| r.pathway == reading.pathway && r.title == reading.title)
        {
            Some(same) => same.probability += reading.probability,
            None => readings.push(reading),
        }
    }
    readings.sort_by(|x, y| y.probability.total_cmp(&x.probability));
    let top = if readings.is_empty() {
        reading_for(Pathway::None, 0.0, request, ctx, catalog, b, answers)
    } else {
        readings.remove(0)
    };
    let complete = noul_answer(answers, "complete")
        .filter(|p| p.is_finite() && (0.0..=1.0).contains(p))
        .unwrap_or(0.0);
    let reuse = score_answer(answers, "reuse").unwrap_or(0.0);
    let ready = top.pathway != Pathway::None
        && top.missing.is_none()
        && !top.steps.is_empty()
        && complete >= COMPLETE_THRESHOLD
        && top.probability >= READY_PROBABILITY;
    let alternatives = readings
        .into_iter()
        .filter(|r| {
            r.probability >= ALTERNATIVE_FLOOR && r.pathway != Pathway::None && r.missing.is_none()
        })
        .take(3)
        .collect();
    let slot = |name: &str| picked_span(answers, b, name).map(str::to_string);
    let body = picked_span(answers, b, "body");
    let when =
        picked_span(answers, b, "when").map(|when| with_time_lead_in(request, b, when, body));
    let body = match top.pathway {
        Pathway::Remember => body.map(|body| with_what_its_about(request, b, body)),
        _ => body.map(str::to_string),
    };
    let words = read_words(request, ctx, b, answers, &top);
    Plan {
        reading: top,
        request: request.to_string(),
        complete,
        reuse,
        ready,
        alternatives,
        slots: Slots {
            body,
            task: slot("task"),
            script_steps: slot("script_steps"),
            when,
            query: slot("query"),
            shortcut: slot("shortcut"),
        },
        words,
        clauses: Vec::new(),
        timing: Timing::default(),
    }
}

/// Clauses are read in parallel against the screen as it is now, so after "open slack"
/// the next clause still sees the old front app. Its menu commands no longer apply, so a
/// menu reading gives way to the clause's window reading, or else needs the new app's
/// menu. And "it" means the app the previous step named, not whichever window Jev
/// guessed, unless the clause names that window's app.
pub fn link_pronouns(clauses: &mut [Plan], ctx: &ScreenContext) {
    for i in 1..clauses.len() {
        let Some((previous, opened)) = clauses[i - 1].steps.last().and_then(|last| {
            let app = last.app()?.to_string();
            let opened = matches!(last, Step::Command { .. }) && app != ctx.front_app.name;
            Some((app, opened))
        }) else {
            continue;
        };
        let clause = &mut clauses[i];
        if opened && is_browser(&previous, "") {
            let spoken = format!(" {} ", clause.request.to_lowercase());
            let said = web_address(&clause.request).map(|web| web.said);
            if let (Some(Step::OpenUrl { app, .. }), Some(said)) =
                (clause.reading.steps.first_mut(), said)
            {
                let named = app.as_deref().is_some_and(|app| {
                    name_words(app).any(|word| spoken.contains(&format!(" {word} ")))
                });
                if !named {
                    *app = Some(previous.clone());
                    clause.reading.title = web_title(&said, Some(&previous));
                }
            }
        }
        if opened && clause.pathway == Pathway::AppCommand {
            let window = clause
                .alternatives
                .iter()
                .find(|reading| reading.pathway == Pathway::Window)
                .cloned();
            match window {
                Some(window) => clause.reading = window,
                None => {
                    clause.reading = Reading::new(
                        Pathway::AppCommand,
                        clause.probability,
                        format!("Do it in {previous}"),
                    )
                    .lacking(format!("a {previous} menu command"));
                    clause.ready = false;
                }
            }
        }
        if !pattern_matches(&POINTS_BACK, &clause.request) {
            continue;
        }
        let spoken = format!(" {} ", clause.request.to_lowercase());
        let Some(Step::Window { op, app, target }) = clause.reading.steps.first_mut() else {
            continue;
        };
        if previous == *app || spoken.contains(&format!(" {} ", app.to_lowercase())) {
            continue;
        }
        *app = previous.clone();
        *target = ctx.window_snapshot.bind(&previous, false);
        let title = op.title_for(&previous);
        clause.reading.title = title.clone();
        clause.reading.typed = Some(title);
    }
}

pub fn understand(
    request: &str,
    ctx: &ScreenContext,
    catalog: &Catalog,
    oracle: &dyn Oracle,
    opts: &EngineOptions,
) -> Result<Plan, JevError> {
    if let Some(plan) = super::filename::read(request, ctx, oracle) {
        return plan;
    }
    let built = build_questions(request, ctx, catalog, opts);
    let result = oracle.ask(&built.state, &built.questions)?;
    let mut plan = compose_plan(request, ctx, catalog, &built, &result.answers);
    plan.timing = Timing {
        ms: result.ms,
        input_tokens: result.input_tokens,
        questions: built.questions.len(),
        rounds: 1,
    };

    let is_clause = opts.exclude.contains(&Pathway::Automate);
    if is_clause {
        return Ok(plan);
    }
    let apps: Vec<&str> = ctx
        .open_windows
        .iter()
        .map(|w| w.app.as_str())
        .chain(
            catalog
                .commands
                .iter()
                .filter(|c| c.kind == CommandKind::App)
                .map(|c| c.name.as_str()),
        )
        .collect();
    let compound = noul_answer(&result.answers, "compound").unwrap_or(0.0);
    if plan.pathway == Pathway::Automate && plan.missing.as_deref() == Some(STEPS_NEXT) {
        let mut source = plan
            .slots
            .script_steps
            .clone()
            .or_else(|| plan.slots.body.clone())
            .unwrap_or_else(|| request.to_string());
        if let Some(when) = &plan.slots.when {
            source = source.replacen(when.as_str(), "", 1);
        }
        if let Some(shortcut) = &plan.slots.shortcut {
            source = source.replacen(shortcut.as_str(), "", 1);
        }
        let clauses = split_clauses_for_apps(source.trim(), &apps);
        if !clauses.is_empty() {
            decompose(&mut plan, &clauses, "New script: ", ctx, catalog, oracle);
        }
    } else if compound >= 0.5 && plan.pathway.is_instant() {
        let clauses = split_clauses_for_apps(request, &apps);
        if clauses.len() >= 2 {
            decompose(&mut plan, &clauses, "", ctx, catalog, oracle);
        }
    }
    Ok(plan)
}

/// Reads each clause in its own parallel round. If any clause can't be read the plan
/// stays unready: running half of what was asked is worse than running nothing.
fn decompose(
    plan: &mut Plan,
    clauses: &[String],
    prefix: &str,
    ctx: &ScreenContext,
    catalog: &Catalog,
    oracle: &dyn Oracle,
) {
    // A first-round alternative is not an alternative interpretation of all clauses.
    plan.alternatives.clear();
    if clauses.len() > MAX_CLAUSES {
        plan.reading.steps.clear();
        plan.reading.missing = Some(format!("at most {MAX_CLAUSES} separate steps"));
        plan.ready = false;
        return;
    }
    let started = Instant::now();
    let opts = EngineOptions {
        exclude: vec![Pathway::Automate, Pathway::None],
    };
    let results: Vec<Result<Plan, JevError>> = std::thread::scope(|scope| {
        let handles: Vec<_> = clauses
            .iter()
            .map(|clause| {
                let opts = &opts;
                scope.spawn(move || understand(clause, ctx, catalog, oracle, opts))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err(JevError::Transport("clause reader panicked".into())))
            })
            .collect()
    });
    plan.timing.rounds = 2;
    plan.timing.ms += started.elapsed().as_millis() as u64;
    let mut subs = Vec::with_capacity(results.len());
    for result in results {
        match result {
            Ok(sub) => subs.push(sub),
            Err(error) => {
                tracing::warn!(target: "script_kit::intent", %error, "clause round failed");
                plan.reading.missing = Some("the separate steps".into());
                plan.ready = false;
                return;
            }
        }
    }
    link_pronouns(&mut subs, ctx);
    plan.timing.input_tokens += subs.iter().map(|s| s.timing.input_tokens).sum::<u64>();
    plan.reading.steps = subs.iter().flat_map(|s| s.steps.clone()).collect();
    plan.reading.title = format!(
        "{prefix}{}",
        subs.iter()
            .map(|s| s.title.as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    );
    plan.reading.missing = subs.iter().find_map(|s| {
        s.missing
            .clone()
            .or_else(|| (!s.ready).then(|| format!("what “{}” should do", s.request)))
    });
    plan.ready =
        plan.missing.is_none() && !plan.steps.is_empty() && plan.complete >= COMPLETE_THRESHOLD;
    plan.clauses = subs;
}
