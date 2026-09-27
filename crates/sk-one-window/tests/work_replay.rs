//! Chapters 10 and 11 replayed from their golden traces with the rules the reducer now owns for
//! steps, connecting a service and agents in the strip.
//!
//! The user's words and keys come from the trace and the page's own lines for these views are
//! dropped: the reducer opens, labels and hands back the views itself. What the host would learn
//! (the steps were read, Calendar connected, a step finished, the agent asked or finished) is fed
//! at the moment the trace shows the page learning it. What ↵ said must match the page.

use std::path::Path;

use serde_json::Value;
use sk_one_window::list::Item;
use sk_one_window::steps::StepSpec;
use sk_one_window::strip::TaskState;
use sk_one_window::window::{Effect, Event, OneWindow};
use sk_one_window::work::{WorkEffect, WorkEvent};

fn trace(name: &str) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("fixtures/traces/{name}.jsonl"));
    std::fs::read_to_string(path)
        .expect("trace reads")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("trace line is JSON"))
        .collect()
}

fn t(v: &Value) -> u64 {
    v["t"].as_u64().unwrap_or(0)
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}

/// The first line matching `pred`, by time.
fn when(lines: &[Value], pred: impl Fn(&Value) -> bool) -> u64 {
    lines
        .iter()
        .find(|l| pred(l))
        .map(t)
        .expect("the trace shows it")
}

struct Host {
    win: OneWindow,
    /// What ↵ said, with whether it was armed, each time it changed.
    enter: Vec<(u64, String, bool)>,
    effects: Vec<(u64, Effect)>,
}

impl Host {
    fn new() -> Self {
        Self {
            win: OneWindow::default(),
            enter: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn feed(&mut self, now: u64, ev: Event) {
        let fx = self.win.handle(now, ev).effects;
        for e in fx {
            if let Effect::ViewKey(k) = &e {
                let k = k.clone();
                self.feed(now, Event::Work(WorkEvent::ViewKey(k)));
            }
            if let Effect::Run { row, .. } = &e {
                if row.key == "steps" {
                    let sentence = self.win.said();
                    self.feed(now, Event::Work(WorkEvent::OpenSteps { sentence }));
                } else if row.key.contains("(Agent)") {
                    let task = self.win.said();
                    self.feed(now, Event::Work(WorkEvent::Agent { task }));
                }
            }
            self.effects.push((now, e));
        }
        let target = self.win.enter_target(now);
        if self.enter.last().is_none_or(|(_, l, _)| *l != target.label) {
            self.enter
                .push((now, target.label, self.win.enter_armed(now)));
        }
    }

    /// Feeds a trace input the way the replay does, except the page's own lines for `skip_views`.
    fn input(&mut self, line: &Value, skip_views: &[&str]) {
        let now = t(line);
        if matches!(s(line, "ev"), "view" | "viewEnter") {
            let name = line["name"]
                .as_str()
                .or(line["from"].as_str())
                .unwrap_or("");
            let id = s(line, "id");
            if skip_views
                .iter()
                .any(|v| *v == name || id == *v || id.starts_with(&format!("{v}:")))
            {
                return;
            }
        }
        let Ok(sk_one_window::trace::Line::Ev(ev)) =
            sk_one_window::trace::parse_line(&line.to_string())
        else {
            return;
        };
        for e in sk_one_window::trace::events_of(ev, None, &self.win) {
            self.feed(now, e);
        }
    }

    fn labels_between(&self, from: u64, to: u64) -> Vec<String> {
        self.enter
            .iter()
            .filter(|(at, _, _)| (from..=to).contains(at))
            .map(|(_, l, _)| l.clone())
            .collect()
    }
}

fn page_labels_between(lines: &[Value], from: u64, to: u64) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in lines
        .iter()
        .filter(|l| s(l, "out") == "enter" && (from..=to).contains(&t(l)))
    {
        let label = s(l, "label").to_string();
        if out.last() != Some(&label) {
            out.push(label);
        }
    }
    out
}

const S10_SENTENCE: &str = "move standup to tomorrow at 9, tell mo i'm running late and add to today's note: ship beta notes";

fn s10_steps() -> Vec<StepSpec> {
    // The page read step 1 as moving a Calendar event ("needs":"calendar" on that reading).
    sk_one_window::steps::split(S10_SENTENCE)
        .into_iter()
        .enumerate()
        .map(|(i, text)| StepSpec {
            needs: (i == 0).then(|| "calendar".to_string()),
            title: Some(text.clone()),
            text,
        })
        .collect()
}

#[test]
fn s10_steps_connect_and_carry_on_like_the_page() {
    let lines = trace("s10-steps");
    let opened = when(&lines, |l| s(l, "ev") == "view" && s(l, "name") == "steps");
    let read = when(&lines, |l| s(l, "id") == "steps:connect:calendar");
    let connected = when(&lines, |l| {
        s(l, "ev") == "view" && s(l, "from") == "connect"
    });
    let reread = when(&lines, |l| {
        s(l, "ev") == "viewEnter" && s(l, "label") == "Do all 3"
    });
    let held = when(&lines, |l| s(l, "ev") == "hold");
    let finished = when(&lines, |l| s(l, "ev") == "done");

    let mut facts: Vec<(u64, Event)> = vec![
        (
            read,
            Event::Work(WorkEvent::StepsRead {
                sentence: S10_SENTENCE.into(),
                steps: s10_steps(),
            }),
        ),
        (
            connected,
            Event::Work(WorkEvent::Connected {
                integration: "calendar".into(),
            }),
        ),
        (
            reread,
            Event::Work(WorkEvent::StepsRead {
                sentence: S10_SENTENCE.into(),
                steps: s10_steps(),
            }),
        ),
        (
            held,
            Event::Hold {
                label: "To Mo: “I'm running late”".into(),
                request: "tell mo i'm running late".into(),
            },
        ),
    ];
    facts.extend((0..3).map(|i| (finished, Event::Work(WorkEvent::StepDone { index: i }))));

    let mut host = Host::new();
    for line in &lines {
        while facts.first().is_some_and(|(at, _)| *at <= t(line)) {
            let (at, ev) = facts.remove(0);
            host.feed(at, ev);
        }
        if s(line, "ev") == "hold" || s(line, "ev") == "done" {
            continue;
        }
        host.input(line, &["steps", "connect"]);
    }

    assert_eq!(
        host.labels_between(opened - 1, finished - 1),
        page_labels_between(&lines, opened - 1, finished - 1),
        "↵ in the steps and connect views"
    );
    let armed_at = |label: &str, at_least: u64| {
        host.enter
            .iter()
            .find(|(at, l, _)| l == label && *at >= at_least)
            .map(|(_, _, armed)| *armed)
    };
    assert_eq!(
        armed_at("Connect Google Calendar for step 1, then do all 3", opened),
        Some(true),
        "a reading that changes ↵ on its own guards Enter, as the page did"
    );
    assert_eq!(
        armed_at("Reading the steps…", connected),
        Some(true),
        "the hand-back guards Enter, as the page did"
    );
    let signed_in = host
        .effects
        .iter()
        .any(|(_, e)| matches!(e, Effect::Work(WorkEffect::SignIn { integration }) if integration == "calendar"));
    assert!(signed_in, "Enter in the connect view signs in");
    assert!(!host.win.in_view(), "Do all finished and went home");
    let last = host
        .win
        .ledger()
        .entries()
        .iter()
        .map(|e| e.title.clone())
        .collect::<Vec<_>>();
    assert!(
        last.iter().any(|t| t == "Google Calendar connected"),
        "the line read “Last · Google Calendar connected” after the hand-back: {last:?}"
    );
    assert!(
        last.iter()
            .any(|t| t.starts_with("Did 3 steps: move standup"))
    );
}

#[test]
fn s11_agent_parks_in_the_strip_and_cmd_j_opens_it_like_the_page() {
    let lines = trace("s11-agents");
    let asks = when(&lines, |l| {
        s(l, "id") == "agent:answer" && l["onTop"] == false
    });
    let finished = when(&lines, |l| {
        s(l, "id") == "agent:approve" && l["onTop"] == false
    });
    let reopened: Vec<u64> = lines
        .iter()
        .filter(|l| s(l, "ev") == "key" && s(l, "key") == "CmdJ")
        .map(t)
        .collect();
    assert_eq!(reopened.len(), 2);

    let mut host = Host::new();
    let mut id = None;
    let mut lists_while_busy: Vec<(u64, Vec<Item>)> = Vec::new();
    for line in &lines {
        let now = t(line);
        if id.is_none() {
            id = host.effects.iter().find_map(|(_, e)| match e {
                Effect::Work(WorkEffect::StartAgent { id, .. }) => Some(id.clone()),
                _ => None,
            });
        }
        if let Some(id) = &id {
            for (at, state, status, approve) in [
                (
                    asks,
                    TaskState::NeedsYou,
                    "Needs you · which list flickers?",
                    None,
                ),
                (
                    finished,
                    TaskState::Ready,
                    "Ready for you",
                    Some("Open PR #815 for review"),
                ),
            ] {
                if at == now {
                    let before = (!host.win.in_view()).then(|| host.win.list().layout());
                    host.feed(
                        now,
                        Event::Work(WorkEvent::TaskChanged {
                            id: id.clone(),
                            status: status.into(),
                            progress: 0.5,
                            state,
                            approve: approve.map(str::to_string),
                        }),
                    );
                    if let Some(before) = before {
                        lists_while_busy.push((now, before));
                        assert_eq!(
                            host.win.list().layout(),
                            lists_while_busy.last().unwrap().1,
                            "the list you're typing into doesn't change when the agent moves along"
                        );
                    }
                }
            }
        }
        if s(line, "ev") == "done" && s(line, "icon") == "delegate" {
            continue;
        }
        host.input(line, &["agent"]);
        // The page's answer to the agent's question and its approval are the agent's to do.
        if s(line, "ev") == "key" && s(line, "key") == "Enter" {
            let said = host.effects.iter().rev().find_map(|(at, e)| match e {
                Effect::Work(WorkEffect::AgentSay { id, .. }) if *at == now => Some(id.clone()),
                _ => None,
            });
            if let Some(task) = said {
                host.feed(
                    now,
                    Event::Work(WorkEvent::TaskChanged {
                        id: task,
                        status: "Working".into(),
                        progress: 0.6,
                        state: TaskState::Working,
                        approve: None,
                    }),
                );
            }
        }
    }

    let parked = host
        .effects
        .iter()
        .filter(|(_, e)| matches!(e, Effect::Work(WorkEffect::Parked { .. })))
        .count();
    assert!(
        parked >= 2,
        "Esc parks the agent's view into the strip each time"
    );
    for (at, want) in reopened
        .iter()
        .zip(["Pick an answer, or say it", "Open PR #815 for review"])
    {
        let said = host
            .enter
            .iter()
            .rev()
            .find(|(t, _, _)| t == at)
            .map(|(_, l, _)| l.as_str());
        assert_eq!(said, Some(want), "⌘J at {at} opens what needs you");
        assert_eq!(
            page_labels_between(&lines, *at, *at + 5)
                .last()
                .map(String::as_str),
            Some(want)
        );
    }
    assert!(
        !lists_while_busy.is_empty(),
        "the agent moved along while a list was showing"
    );
}
