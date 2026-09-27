//! Services a reading can need, and connecting one without leaving the window.
//!
//! A row that needs a service the user hasn't connected says so ("Needs Google Calendar", ↵
//! "Connect Google Calendar") and never runs as if it could. Enter opens the connect view in place:
//! the sentence waits there, nothing to retype, and when the service connects the view hands back
//! to whoever opened it — the steps carry on, or the waiting sentence runs.

use std::collections::BTreeSet;

use crate::enter::EnterTarget;
use crate::row::Row;
use crate::window::ViewSpec;

pub const VIEW: &str = "connect";

#[derive(Debug, PartialEq, Eq)]
pub struct Integration {
    pub key: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    /// What connecting lets Script Kit do, as the sign-in asks for it.
    pub scope: &'static str,
    /// What else works once it's connected.
    pub can: &'static [&'static str],
    /// Words in a sentence that name something only this service has.
    words: &'static [&'static str],
}

pub const INTEGRATIONS: &[Integration] = &[
    Integration {
        key: "calendar",
        name: "Google Calendar",
        icon: "move",
        scope: "See and edit the events on all your calendars",
        can: &[
            "Move events and tell whoever's in them",
            "Tell your next meeting you're running late",
            "Join calls from the launcher",
            "Answer “what's next?”",
        ],
        words: &[
            "calendar",
            "standup",
            "meeting",
            "event",
            "invite",
            "reschedule",
        ],
    },
    Integration {
        key: "slack",
        name: "Slack",
        icon: "message",
        scope: "Read and post messages in channels you're in, and set your status",
        can: &[
            "Post in channels and DMs",
            "Reply to what's waiting on you",
            "Set your status",
            "Catch up on what you missed",
        ],
        words: &["slack", "channel"],
    },
    Integration {
        key: "gmail",
        name: "Gmail",
        icon: "message",
        scope: "Read, draft and send your email",
        can: &[
            "Reply to the email you're looking at",
            "Find a thread by who sent it",
            "Draft a follow-up",
        ],
        words: &["gmail", "email", "inbox"],
    },
    Integration {
        key: "github",
        name: "GitHub",
        icon: "delegate",
        scope: "Read your repositories, issues and pull requests",
        can: &[
            "Open a pull request for review",
            "Read an issue before an agent starts",
            "See what's waiting on your review",
        ],
        words: &["github", "pr", "repo"],
    },
    Integration {
        key: "linear",
        name: "Linear",
        icon: "delegate",
        scope: "Read and update your issues",
        can: &[
            "Read an issue by its ID",
            "Move an issue along",
            "File a bug from what's on screen",
        ],
        words: &["linear", "issue"],
    },
    Integration {
        key: "notion",
        name: "Notion",
        icon: "append",
        scope: "Read and edit the pages you share with Script Kit",
        can: &["Add a line to a page", "Find a page by its title"],
        words: &["notion"],
    },
];

pub fn integration(key: &str) -> Option<&'static Integration> {
    INTEGRATIONS.iter().find(|i| i.key == key)
}

/// The service's name, or its key when it isn't one this knows.
pub fn name_of(key: &str) -> &str {
    integration(key).map_or(key, |i| i.name)
}

/// The service a step's words name, for when Jev hasn't read them: a word only that service has.
pub fn named_in(words: &str) -> Option<&'static str> {
    let lower = words.to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    INTEGRATIONS
        .iter()
        .find(|i| {
            tokens.iter().any(|t| {
                i.words
                    .iter()
                    .any(|w| t == w || t.strip_suffix('s') == Some(w))
            })
        })
        .map(|i| i.key)
}

/// What the user has connected.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Connections {
    connected: BTreeSet<String>,
}

impl Connections {
    pub fn has(&self, key: &str) -> bool {
        self.connected.contains(key)
    }

    pub fn add(&mut self, key: &str) -> bool {
        self.connected.insert(key.to_string())
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.connected.iter().map(String::as_str)
    }
}

/// The service a row still waits on.
pub fn waits_on<'a>(row: &'a Row, connections: &Connections) -> Option<&'a str> {
    row.needs.as_deref().filter(|n| !connections.has(n))
}

/// A row as the list shows it: one that needs a service says so and ↵ connects it.
pub fn decorate(mut row: Row, connections: &Connections) -> Row {
    let Some(key) = waits_on(&row, connections).map(str::to_string) else {
        return row;
    };
    let does = row.sub.take().unwrap_or_else(|| row.title.clone());
    row.sub = Some(format!("{does} · ↵ connects it"));
    row.enter_label = Some(format!("Connect {}", name_of(&key)));
    row
}

/// The source line a row that waits on a service shows instead of its own: "Needs Google Calendar".
pub fn needs_label(row: &Row, connections: &Connections) -> Option<String> {
    waits_on(row, connections).map(|k| format!("Needs {}", name_of(k)))
}

/// Who opened the connect view, and so who gets the window back when it connects.
#[derive(Clone, Debug, PartialEq)]
pub enum Opener {
    /// A step in the steps view under it.
    Step(usize),
    /// A row of the list: it runs once connected.
    Row(Box<Row>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Ask,
    Connecting,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Connecting {
    pub integration: String,
    /// The words that wait to run, shown so nothing needs retyping.
    pub waiting: String,
    /// What it does, in the reading's words.
    pub does: String,
    pub opener: Opener,
    pub phase: Phase,
    /// Why the last try didn't connect.
    pub error: Option<String>,
}

impl Connecting {
    pub fn new(integration: &str, waiting: &str, does: &str, opener: Opener) -> Self {
        Self {
            integration: integration.to_string(),
            waiting: waiting.to_string(),
            does: does.to_string(),
            opener,
            phase: Phase::Ask,
            error: None,
        }
    }

    pub fn name(&self) -> &str {
        name_of(&self.integration)
    }

    pub fn enter(&self) -> EnterTarget {
        match self.phase {
            Phase::Ask => EnterTarget::new(
                format!("connect:{}", self.integration),
                format!("Connect {}", self.name()),
            ),
            Phase::Connecting => EnterTarget::new(
                format!("connect:{}:wait", self.integration),
                format!("Connecting {}…", self.name()),
            ),
        }
    }

    pub fn spec(&self) -> ViewSpec {
        ViewSpec {
            name: VIEW.into(),
            label: format!("Connect {}", self.name()),
            enter: Some(self.enter()),
            takes_input: true,
            text: Some(String::new()),
            leave_text: None,
        }
    }

    /// The line the window leaves once it's connected: "Google Calendar connected".
    pub fn done_title(&self) -> String {
        format!("{} connected", self.name())
    }

    /// The line the window leaves when the user goes without it.
    pub fn without_title(&self) -> String {
        format!("Copied a message you can paste into {}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn needing(key: &str) -> Row {
        Row {
            needs: Some(key.into()),
            ..Row::new("jev:calendar: move", "Move standup to 10").with_sub("Moves the event")
        }
    }

    #[test]
    fn a_row_that_needs_a_service_says_so_and_connects_it_on_enter() {
        let conns = Connections::default();
        let row = decorate(needing("calendar"), &conns);
        assert_eq!(row.enter_words(), "Connect Google Calendar");
        assert_eq!(row.sub.as_deref(), Some("Moves the event · ↵ connects it"));
        assert_eq!(
            needs_label(&row, &conns).as_deref(),
            Some("Needs Google Calendar")
        );
    }

    #[test]
    fn a_connected_service_leaves_the_row_alone() {
        let mut conns = Connections::default();
        conns.add("calendar");
        let row = needing("calendar");
        assert_eq!(decorate(row.clone(), &conns), row);
        assert_eq!(needs_label(&row, &conns), None);
    }

    #[test]
    fn words_name_the_service_only_it_has() {
        assert_eq!(named_in("move standup to tomorrow at 9"), Some("calendar"));
        assert_eq!(named_in("post it in the #launch channel"), Some("slack"));
        assert_eq!(named_in("tell mo i'm running late"), None);
        assert_eq!(named_in("add to today's note: ship beta notes"), None);
        assert_eq!(named_in("reply to the email from sarah"), Some("gmail"));
    }

    #[test]
    fn the_connect_view_asks_then_waits_then_hands_back() {
        let mut c = Connecting::new(
            "calendar",
            "move standup to 10",
            "Moves the event",
            Opener::Step(0),
        );
        assert_eq!(c.enter().label, "Connect Google Calendar");
        assert_eq!(c.spec().label, "Connect Google Calendar");
        c.phase = Phase::Connecting;
        assert_eq!(
            c.enter(),
            EnterTarget::new("connect:calendar:wait", "Connecting Google Calendar…")
        );
        assert_eq!(c.done_title(), "Google Calendar connected");
    }
}
