//! The seven words every sentence fills (S04): Context, Subject, Focused, Action, Target,
//! Details and Owner. Context and Focused come from the screen. Jev reads the other five from
//! the same answers that pick the plan, and each keeps the run of the user's own words it was
//! read from, which the input marks in the word's color. Those runs are Jev's span answers, or
//! the words the sentence itself uses for what Jev chose; the engine never writes them.

use std::ops::Range;

use regex::Regex;
use serde::Serialize;

use super::context::ScreenContext;
use super::engine::{self, Built, Pathway, Reading, Step, CAPTURE_TARGETS};
use super::jev::Answers;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WordKind {
    Context,
    Subject,
    Focused,
    Action,
    Target,
    Details,
    Owner,
}

impl WordKind {
    pub const ALL: [WordKind; 7] = [
        WordKind::Context,
        WordKind::Subject,
        WordKind::Focused,
        WordKind::Action,
        WordKind::Target,
        WordKind::Details,
        WordKind::Owner,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WordKind::Context => "Context",
            WordKind::Subject => "Subject",
            WordKind::Focused => "Focused",
            WordKind::Action => "Action",
            WordKind::Target => "Target",
            WordKind::Details => "Details",
            WordKind::Owner => "Owner",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WordSource {
    /// Seen on screen; nobody types it.
    Screen,
    /// Read by Jev from the sentence.
    Jev,
    /// The sentence says it in so many words; no answer of Jev's is behind it.
    Words,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WordRead {
    pub kind: WordKind,
    pub value: String,
    /// Where the value came from, in a few words.
    pub sub: String,
    /// The run of the user's words it was read from, verbatim.
    pub said: Option<String>,
    pub source: WordSource,
    /// Jev's probability for the answer behind it, when Jev sent one.
    pub probability: Option<f64>,
    /// Nothing in the sentence fills it.
    pub dim: bool,
}

const SAYS_THIS: &str = r"(?i)\b(?:this|it|here)\b";
const COPIED: &str = r"(?i)\bwhat i (?:just )?copied\b|\b(?:the|my) clipboard\b|\bcopied\b";
const SELECTED: &str = r"(?i)\bthe selection\b|\b(?:the )?selected \w+";
const PAGE: &str = r"(?i)\b(?:this|the) (?:page|article|site|tab)\b";
const OWNER: &str = r"(?i)\b(?:have|get|ask|let)\s+(?:an?|the|my)\s+agent(?:\s+to)?\b|\b(?:have|ask|get|let)\s+claude(?:\s+to)?\b|\bagent\b";
const PLACEMENT: &str = r"(?i)\b(?:to|on|onto|into)\s+(?:the\s+|my\s+)?(?:(?:left|right|top|bottom|upper|lower|other|next|center|middle)(?:\s+(?:two|left|right|top|bottom))?\s+(?:half|third|thirds|quarter|corner|side|monitor|screen|display)|center)\b";
/// Verbs any sentence may lead with, for when its reading's own verbs aren't said.
const VERBS: &[&str] = &[
    "send",
    "email",
    "message",
    "text",
    "call",
    "tell",
    "share",
    "post",
    "reply",
    "forward",
    "ask",
    "remind",
    "add",
    "put",
    "move",
    "open",
    "close",
    "make",
    "write",
    "fix",
    "summarize",
    "rewrite",
    "translate",
    "find",
    "show",
    "save",
    "schedule",
    "book",
    "play",
    "check",
    "buy",
    "set",
    "turn",
    "take",
    "use",
    "read",
    "review",
    "update",
    "create",
    "delete",
    "remove",
    "clean",
    "run",
    "build",
    "test",
    "draft",
    "plan",
    "pay",
    "renew",
    "ship",
    "research",
    "refactor",
    "explain",
    "shorten",
    "copy",
    "paste",
    "start",
    "stop",
    "finish",
    "bring",
    "keep",
    "remember",
    "note",
    "jot",
    "search",
    "lock",
    "empty",
    "launch",
    "quit",
    "hide",
    "minimize",
    "maximize",
    "center",
    "split",
    "pin",
    "mark",
    "undo",
    "look",
    "get",
    "go",
    "do",
];
/// "to sarah", "to #launcher": who gets it, when nothing else says where it goes.
const RECIPIENT: &str = r"(?i)\bto\s+(#[\w-]+|[a-z][\w'-]*)";
const NOT_RECIPIENTS: &[&str] = &[
    "me", "you", "him", "her", "them", "us", "it", "this", "that", "these", "those", "my", "the",
    "a", "an", "your", "our", "their", "his", "its", "do", "be",
];
/// Little words that belong to the next phrase, not the one before them.
const DANGLING: &[&str] = &[
    "to", "in", "on", "at", "for", "by", "and", "with", "about", "into", "onto", "of", "the", "a",
    "an", "my",
];

fn verbs_for(pathway: Pathway) -> &'static [&'static str] {
    match pathway {
        Pathway::Capture => &[
            "remind me",
            "remind",
            "add",
            "put",
            "schedule",
            "set up",
            "jot down",
            "save",
            "book",
            "note",
        ],
        Pathway::Remember => &[
            "remember",
            "keep in mind",
            "don't forget",
            "memorize",
            "note",
        ],
        Pathway::Recall => &[
            "what did i copy",
            "what did i dictate",
            "what did i say",
            "show me",
            "find",
            "search",
            "look up",
        ],
        Pathway::Transform => &[
            "summarize",
            "summarise",
            "rewrite",
            "shorten",
            "explain",
            "translate",
            "fix",
            "proofread",
            "improve",
            "make",
            "turn",
        ],
        Pathway::Delegate => &[
            "look into",
            "fix",
            "research",
            "refactor",
            "write",
            "build",
            "review",
            "update",
            "add",
            "make",
            "test",
            "implement",
            "investigate",
            "find",
        ],
        Pathway::Run | Pathway::AppCommand => &[
            "turn on",
            "turn off",
            "pull up",
            "go to",
            "bring back",
            "open",
            "launch",
            "start",
            "run",
            "turn",
            "toggle",
            "lock",
            "take",
            "empty",
            "show",
            "copy",
            "play",
            "reply",
            "forward",
            "mark",
            "pin",
            "split",
            "make",
            "new",
        ],
        Pathway::Window => &[
            "get rid of",
            "put",
            "move",
            "send",
            "drag",
            "throw",
            "place",
            "snap",
            "tile",
            "push",
            "shove",
            "maximize",
            "minimize",
            "center",
            "close",
            "quit",
            "hide",
            "resize",
            "make",
        ],
        Pathway::Automate => &[
            "save that as a script",
            "make a script",
            "make a shortcut",
            "create a script",
            "do that again",
            "save",
            "turn",
            "make",
            "create",
        ],
        Pathway::Ask | Pathway::None => &[],
    }
}

fn fallback_verb(reading: &Reading) -> &'static str {
    match reading.pathway {
        Pathway::Run => "Run",
        Pathway::AppCommand => "Click",
        Pathway::Window => "Arrange",
        Pathway::Capture => "Add",
        Pathway::Remember => "Remember",
        Pathway::Recall => "Search",
        Pathway::Transform => "Rewrite",
        Pathway::Ask => "Answer it",
        Pathway::Delegate => "Hand off",
        Pathway::Automate => "Save as script",
        Pathway::None => "—",
    }
}

fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

/// The first match of `pattern` in `text` that none of the `taken` runs touch.
fn find_free(pattern: &str, text: &str, taken: &[Range<usize>]) -> Option<Range<usize>> {
    let regex = Regex::new(pattern).ok()?;
    let found = regex
        .find_iter(text)
        .map(|m| m.range())
        .find(|range| !taken.iter().any(|t| overlaps(t, range)));
    found
}

/// A whole-word pattern for phrases, longest first so "remind me" beats "remind"; the last
/// word takes any ending ("summarizes", "fixing").
fn phrase_pattern(phrases: &[&str]) -> Option<String> {
    let mut phrases: Vec<&str> = phrases.to_vec();
    phrases.sort_by_key(|p| std::cmp::Reverse(p.len()));
    let alternatives: Vec<String> = phrases
        .iter()
        .map(|phrase| {
            phrase
                .split_whitespace()
                .map(regex::escape)
                .collect::<Vec<_>>()
                .join(r"\s+")
        })
        .filter(|p| !p.is_empty())
        .collect();
    (!alternatives.is_empty()).then(|| format!(r"(?i)\b(?:{})\w*\b", alternatives.join("|")))
}

fn verb_pattern() -> String {
    format!(r"(?i)\b(?:{})(?:s|es|ed|ing)?\b", VERBS.join("|"))
}

/// The verb a sentence leads with, as "fix" in "fix this": a question that only mentions a verb
/// later ("when does he play next") doesn't lead with one.
pub fn lead_verb(text: &str) -> Option<Range<usize>> {
    let found = Regex::new(&verb_pattern()).ok()?.find(text)?;
    let lead = text[..found.start()].trim();
    (lead.is_empty() || lead.eq_ignore_ascii_case("please")).then(|| found.range())
}

/// Who a sentence sends something to, in its own words.
fn recipient(text: &str, taken: &[Range<usize>]) -> Option<(Range<usize>, String)> {
    let regex = Regex::new(RECIPIENT).ok()?;
    let found = regex.captures_iter(text).find_map(|captures| {
        let whole = captures.get(0)?.range();
        let name = captures.get(1)?.as_str();
        let lower = name.to_lowercase();
        let is_verb = VERBS.iter().any(|verb| {
            lower
                .strip_prefix(verb)
                .is_some_and(|rest| matches!(rest, "" | "s" | "es" | "ed" | "ing"))
        });
        (!is_verb
            && !NOT_RECIPIENTS.contains(&lower.as_str())
            && !taken.iter().any(|t| overlaps(t, &whole)))
        .then(|| (whole, name.to_string()))
    });
    found
}

/// The longest run of a name's words the sentence says, as in "script kit" for
/// `script-kit-gpui` or "dark mode" for "Toggle Dark Mode".
fn said_name(name: &str, text: &str, taken: &[Range<usize>]) -> Option<Range<usize>> {
    let words: Vec<String> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    for len in (1..=words.len()).rev() {
        for run in words.windows(len) {
            if len == 1 && (run[0].len() < 3 || DANGLING.contains(&run[0].as_str())) {
                continue;
            }
            let pattern = format!(
                r"(?i)\b{}\b",
                run.iter()
                    .map(|w| regex::escape(w))
                    .collect::<Vec<_>>()
                    .join(r"[\s_-]+")
            );
            if let Some(range) = find_free(&pattern, text, taken) {
                return Some(range);
            }
        }
    }
    None
}

/// `range` without the parts other words took, and without little words left hanging at its
/// ends ("the failing tests in" → "the failing tests").
fn own_part(text: &str, range: Range<usize>, taken: &[Range<usize>]) -> Option<Range<usize>> {
    let mut start = range.start;
    let mut end = range.end;
    for t in taken {
        if t.start <= start && start < t.end {
            start = t.end;
        }
    }
    for t in taken {
        if start < t.start && t.start < end {
            end = t.start;
        }
    }
    if start >= end {
        return None;
    }
    loop {
        let part = &text[start..end];
        let trimmed_start = start + (part.len() - part.trim_start().len());
        let trimmed_end = end - (part.len() - part.trim_end().len());
        start = trimmed_start;
        end = trimmed_end;
        if start >= end {
            return None;
        }
        let part = &text[start..end];
        let last = part.rsplit(char::is_whitespace).next().unwrap_or(part);
        if last.len() < part.len() && DANGLING.contains(&last.to_lowercase().as_str()) {
            end -= last.len();
            continue;
        }
        break;
    }
    Some(start..end)
}

fn clip(text: &str, max: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > max {
        format!("{}…", one.chars().take(max - 1).collect::<String>())
    } else {
        one
    }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn span_range(b: &Built, label: &str) -> Option<Range<usize>> {
    let span = b.spans.iter().find(|s| s.label == label)?;
    Some(b.words.get(span.from)?.start..b.words.get(span.to.checked_sub(1)?)?.end)
}

fn choice_probability(answers: &Answers, name: &str) -> Option<f64> {
    engine::choice_answer(answers, name).and_then(|(choice, p)| p.get(choice))
}

fn front_window(ctx: &ScreenContext) -> String {
    if ctx.window_title.is_empty() {
        ctx.front_app.name.clone()
    } else {
        format!("{} · {}", ctx.front_app.name, clip(&ctx.window_title, 56))
    }
}

fn focused_value(ctx: &ScreenContext) -> (String, String) {
    match ctx.selection.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(selection) => (
            format!("“{}”", clip(selection, 64)),
            format!("Selected text in {}", ctx.front_app.name),
        ),
        None => (front_window(ctx), "The front window".to_string()),
    }
}

fn context_value(ctx: &ScreenContext) -> String {
    let mut bits = vec![ctx.front_app.name.clone()];
    if ctx
        .selection
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty())
    {
        bits.push("selected text".into());
    }
    if ctx
        .clipboard
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty())
    {
        bits.push("clipboard".into());
    }
    if ctx.open_windows.len() > 1 {
        bits.push(format!("{} open windows", ctx.open_windows.len()));
    }
    bits.join(", ")
}

struct Found {
    value: String,
    sub: String,
    said: Option<Range<usize>>,
    probability: Option<f64>,
    dim: bool,
    /// Read from the words alone, with no answer of Jev's behind it.
    from_words: bool,
}

impl Found {
    fn said(value: String, range: Range<usize>, text: &str, probability: Option<f64>) -> Self {
        Found {
            value,
            sub: format!("Your words “{}”", &text[range.clone()]),
            said: Some(range),
            probability,
            dim: false,
            from_words: false,
        }
    }

    fn unsaid(value: impl Into<String>, sub: impl Into<String>, probability: Option<f64>) -> Self {
        Found {
            value: value.into(),
            sub: sub.into(),
            said: None,
            probability,
            dim: false,
            from_words: false,
        }
    }

    fn nothing(value: &str, sub: &str) -> Self {
        Found {
            dim: true,
            ..Found::unsaid(value, sub, None)
        }
    }
}

fn owner(request: &str, top: &Reading) -> Found {
    if top.pathway != Pathway::Delegate {
        return Found::nothing("You", "Nobody else named, so Script Kit does it");
    }
    match find_free(OWNER, request, &[]) {
        Some(range) => Found::said("An agent".into(), range, request, Some(top.probability)),
        None => Found::unsaid(
            "An agent",
            "Jev's reading of the sentence",
            Some(top.probability),
        ),
    }
}

fn action(request: &str, top: &Reading, taken: &[Range<usize>]) -> Found {
    let said = phrase_pattern(verbs_for(top.pathway))
        .and_then(|pattern| find_free(&pattern, request, taken))
        .or_else(|| match top.pathway {
            Pathway::None => None,
            Pathway::Ask => {
                lead_verb(request).filter(|range| !taken.iter().any(|t| overlaps(t, range)))
            }
            _ => find_free(&verb_pattern(), request, taken),
        });
    match said {
        Some(range) => Found::said(
            capitalized(&request[range.clone()].to_lowercase()),
            range,
            request,
            Some(top.probability),
        ),
        None if top.pathway == Pathway::None => Found::nothing("—", "No verb said"),
        None => Found::unsaid(
            fallback_verb(top),
            "Jev's reading of the sentence",
            Some(top.probability),
        ),
    }
}

fn details(request: &str, b: &Built, answers: &Answers, taken: &[Range<usize>]) -> Found {
    let body = engine::picked_span(answers, b, "body");
    if let Some(label) = engine::picked_span(answers, b, "when") {
        let text = engine::with_time_lead_in(request, b, label, body);
        if let Some(span) = span_range(b, label) {
            let start = span.end.saturating_sub(text.len());
            let range = if request.get(start..span.end) == Some(text.as_str()) {
                start..span.end
            } else {
                span
            };
            if !taken.iter().any(|t| overlaps(t, &range)) {
                let value = capitalized(&request[range.clone()]);
                return Found::said(value, range, request, choice_probability(answers, "when"));
            }
        }
    }
    if let Some(label) = engine::picked_span(answers, b, "shortcut") {
        if let Some(range) = span_range(b, label).filter(|r| !taken.iter().any(|t| overlaps(t, r)))
        {
            let value = format!("Shortcut {}", &request[range.clone()]);
            return Found::said(
                value,
                range,
                request,
                choice_probability(answers, "shortcut"),
            );
        }
    }
    Found::nothing("Right away", "No when or how said")
}

fn target(
    request: &str,
    ctx: &ScreenContext,
    answers: &Answers,
    top: &Reading,
    taken: &[Range<usize>],
) -> Found {
    let found = where_it_goes(request, ctx, answers, top, taken);
    let names_someone = matches!(
        top.pathway,
        Pathway::Capture | Pathway::Delegate | Pathway::Run | Pathway::AppCommand
    );
    if found.said.is_some() || !names_someone {
        return found;
    }
    match recipient(request, taken) {
        Some((range, name)) => Found {
            from_words: true,
            ..Found::said(capitalized(&name), range, request, None)
        },
        None => found,
    }
}

fn where_it_goes(
    request: &str,
    ctx: &ScreenContext,
    answers: &Answers,
    top: &Reading,
    taken: &[Range<usize>],
) -> Found {
    let here = || Found::nothing("Right here", "Nothing said, so the result shows here");
    match top.pathway {
        Pathway::Capture => {
            let Some(key) = engine::picked(answers, "capture_target")
                .filter(|key| CAPTURE_TARGETS.iter().any(|(k, _)| k == key))
            else {
                return here();
            };
            let (value, pattern) = match key {
                "todo" => (
                    "Todo list",
                    Some(
                        r"(?i)\b(?:to|on|in|into)\s+(?:my\s+|the\s+)?todos?(?:\s+list)?\b|\btodo\b",
                    ),
                ),
                "cal" => (
                    "Calendar",
                    Some(r"(?i)\b(?:on|to|in|into)\s+(?:my\s+|the\s+)?calendar\b|\bcalendar\b"),
                ),
                "note" => (
                    "Today's note",
                    Some(
                        r"(?i)\b(?:to|in|into)\s+(?:my\s+|today's\s+|the\s+)?(?:daily\s+)?(?:note|notes|journal)\b",
                    ),
                ),
                "link" => (
                    "Saved links",
                    Some(r"(?i)\bas\s+a\s+link\b|\bfor\s+later\b"),
                ),
                "social" => ("A social post", Some(r"(?i)\b(?:tweet|post)\b")),
                _ => ("Reminders", None),
            };
            let probability = choice_probability(answers, "capture_target");
            match pattern.and_then(|pattern| find_free(pattern, request, taken)) {
                Some(range) => Found::said(value.into(), range, request, probability),
                None => Found::unsaid(value, "Jev's reading of the sentence", probability),
            }
        }
        Pathway::Window => {
            let op = top.steps.iter().find_map(|step| match step {
                Step::Window { op, .. } => Some(op.label.clone()),
                _ => None,
            });
            match (find_free(PLACEMENT, request, taken), op) {
                (Some(range), op) => Found::said(
                    op.unwrap_or_else(|| capitalized(&request[range.clone()])),
                    range,
                    request,
                    Some(top.probability),
                ),
                (None, _) => here(),
            }
        }
        Pathway::Delegate => {
            let Some(project) = engine::picked(answers, "project")
                .filter(|p| ctx.projects.iter().any(|known| known == p))
            else {
                return here();
            };
            let probability = choice_probability(answers, "project");
            match said_name(project, request, taken) {
                Some(range) => {
                    let lead = Regex::new(r"(?i)\b(?:in|for|on)\s+$")
                        .ok()
                        .and_then(|r| r.find(&request[..range.start]).map(|m| m.start()));
                    let range = match lead {
                        Some(start) if !taken.iter().any(|t| overlaps(t, &(start..range.end))) => {
                            start..range.end
                        }
                        _ => range,
                    };
                    Found::said(project.to_string(), range, request, probability)
                }
                None => Found::unsaid(project, "Jev's reading of the sentence", probability),
            }
        }
        _ => here(),
    }
}

fn subject(
    request: &str,
    ctx: &ScreenContext,
    b: &Built,
    answers: &Answers,
    top: &Reading,
    taken: &[Range<usize>],
) -> Found {
    let probability = choice_probability(answers, "subject");
    let says = |pattern: &str| find_free(pattern, request, taken);
    let named_step = top.steps.iter().find_map(|step| match step {
        Step::Window { app, .. } => Some(app.clone()),
        Step::Command { command } if top.pathway == Pathway::Run => Some(command.name.clone()),
        _ => None,
    });
    if let Some(name) = named_step {
        if let Some(range) = said_name(&name, request, taken) {
            return Found::said(name, range, request, Some(top.probability));
        }
    }
    let pointed = |value: String, sub_default: &str, own: Option<Range<usize>>| match own
        .or_else(|| says(SAYS_THIS))
    {
        Some(range) => {
            let word = request[range.clone()].to_string();
            let mut found = Found::said(value, range, request, probability);
            if matches!(word.to_lowercase().as_str(), "this" | "it" | "here") {
                found.sub = format!("Your words “{word}” mean the Focused thing");
            }
            found
        }
        None => Found::unsaid(value, sub_default, probability),
    };
    match engine::picked(answers, "subject") {
        Some("clipboard") => pointed(
            ctx.clipboard
                .as_deref()
                .filter(|c| !c.trim().is_empty())
                .map(|c| clip(c, 64))
                .unwrap_or_else(|| "The clipboard".into()),
            "The clipboard",
            says(COPIED),
        ),
        Some("selection") => pointed(
            ctx.selection
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("“{}”", clip(s, 64)))
                .unwrap_or_else(|| "The selected text".into()),
            "The selected text",
            says(SELECTED),
        ),
        Some("page") => pointed(front_window(ctx), "The page on screen", says(PAGE)),
        Some("window") if top.pathway != Pathway::Delegate && top.pathway != Pathway::Ask => {
            pointed(front_window(ctx), "The front window", None)
        }
        _ => {
            let content = match top.pathway {
                Pathway::Capture | Pathway::Remember | Pathway::Transform => "body",
                Pathway::Recall => "query",
                Pathway::Automate => "script_steps",
                _ => "task",
            };
            let own = engine::picked_span(answers, b, content)
                .and_then(|label| span_range(b, label))
                .and_then(|range| own_part(request, range, taken));
            match own {
                Some(range) => {
                    let value = capitalized(&request[range.clone()]);
                    Found::said(value, range, request, choice_probability(answers, content))
                }
                None => Found::nothing("—", "Nothing named"),
            }
        }
    }
}

/// The seven words for `request` as its top reading fills them.
pub(super) fn read_words(
    request: &str,
    ctx: &ScreenContext,
    b: &Built,
    answers: &Answers,
    top: &Reading,
) -> Vec<WordRead> {
    let mut taken: Vec<Range<usize>> = Vec::new();
    let keep = |found: &Found, taken: &mut Vec<Range<usize>>| {
        if let Some(range) = &found.said {
            taken.push(range.clone());
        }
    };
    let owner = owner(request, top);
    keep(&owner, &mut taken);
    let action = action(request, top, &taken);
    keep(&action, &mut taken);
    let details = details(request, b, answers, &taken);
    keep(&details, &mut taken);
    let target = target(request, ctx, answers, top, &taken);
    keep(&target, &mut taken);
    let subject = subject(request, ctx, b, answers, top, &taken);
    let focused_by_this = subject.said.as_ref().is_some_and(|r| {
        matches!(
            request[r.clone()].to_lowercase().as_str(),
            "this" | "it" | "here"
        )
    });
    let (focused, focused_where) = focused_value(ctx);
    let jev = |kind: WordKind, found: Found| WordRead {
        kind,
        said: found.said.map(|r| request[r].to_string()),
        value: found.value,
        sub: found.sub,
        source: if found.from_words {
            WordSource::Words
        } else {
            WordSource::Jev
        },
        probability: found.probability,
        dim: found.dim,
    };
    vec![
        WordRead {
            kind: WordKind::Context,
            value: context_value(ctx),
            sub: "What Jev sees without being told. Nobody types it.".into(),
            said: None,
            source: WordSource::Screen,
            probability: None,
            dim: false,
        },
        jev(WordKind::Subject, subject),
        WordRead {
            kind: WordKind::Focused,
            value: focused,
            sub: if focused_by_this {
                "What “this” means here".into()
            } else {
                format!("{focused_where}: what “this” would mean")
            },
            said: None,
            source: WordSource::Screen,
            probability: None,
            dim: !focused_by_this,
        },
        jev(WordKind::Action, action),
        jev(WordKind::Target, target),
        jev(WordKind::Details, details),
        jev(WordKind::Owner, owner),
    ]
}
