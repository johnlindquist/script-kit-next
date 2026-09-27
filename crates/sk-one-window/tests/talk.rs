//! Talking to type, or talking to ask, through the reducer; and s17's trace replayed through it.

use std::path::Path;

use serde_json::Value;
use sk_one_window::contract::talk_words;
use sk_one_window::trace::slot_kind_words;
use sk_one_window::{Done, Effect, Event, Key, OneWindow, Risk, Row, TalkEvent};

fn start() -> Event {
    Event::Talk(TalkEvent::Start {
        app: "Mail".into(),
        field: "Reply to Sarah · Re: Q4 launch timeline".into(),
        to: Some("Sarah".into()),
    })
}

fn read(words: &str, type_p: f32) -> Event {
    Event::Talk(TalkEvent::Read {
        words: words.into(),
        type_p,
        ask_p: 1.0 - type_p,
        ask: None,
    })
}

fn heard(win: &mut OneWindow, t: u64, text: &str) -> Vec<Effect> {
    win.handle(t, Event::Talk(TalkEvent::Heard(text.into())))
        .effects
}

const SAID: &str = "Hi Sarah, thanks for checking in. The updated timeline is attached.";

#[test]
fn typing_pastes_at_the_cursor_and_the_window_goes() {
    let mut win = OneWindow::default();
    win.handle(0, start());
    assert_eq!(win.views().top().unwrap().name, "voice");
    assert_eq!(win.enter_target(0).label, talk_words::WAITS_FOR_JEV);
    let fx = heard(&mut win, 100, "Hi Sarah, thanks for");
    assert!(fx.contains(&Effect::TalkAsk {
        words: "Hi Sarah, thanks for".into(),
        last: false
    }));
    assert_eq!(win.text(), "Hi Sarah, thanks for");
    win.handle(200, read("Hi Sarah, thanks for", 0.9));
    let line = win.slot_line(200);
    assert_eq!(
        (line.label.as_str(), line.title.as_str()),
        ("Typing into", "Mail")
    );
    assert_eq!(win.enter_target(200).label, "Type it into Mail");
    heard(&mut win, 900, SAID);
    let fx = win.handle(1_000, Event::Talk(TalkEvent::Release)).effects;
    assert!(fx.contains(&Effect::TalkAsk {
        words: SAID.into(),
        last: true
    }));
    assert!(
        win.talk().is_some(),
        "waits for Jev to read the final words"
    );
    let fx = win.handle(1_300, read(SAID, 0.92)).effects;
    assert!(fx.contains(&Effect::Paste {
        text: SAID.into(),
        app: "Mail".into()
    }));
    assert!(fx.contains(&Effect::Hide {
        app: Some("Mail".into())
    }));
    assert!(fx.contains(&Effect::TalkEnded));
    assert!(win.talk().is_none() && !win.in_view() && win.is_away());
    let last = win.ledger().last().unwrap();
    assert_eq!(last.title, "Typed 11 words into Mail");
    assert_eq!(last.risk, Risk::Undo, "↑ takes it back");
}

#[test]
fn a_request_becomes_the_sentence_and_runs_jevs_reading() {
    let mut win = OneWindow::default();
    win.handle(0, start());
    let said = "remind me to send Sarah the deck at ten on Friday";
    heard(&mut win, 100, "remind me to send");
    let reminder = Row::new(
        "jev:schedule:Remind me",
        "Remind me Friday 10:00 · Send Sarah the deck",
    );
    win.handle(
        200,
        Event::Talk(TalkEvent::Read {
            words: "remind me to send".into(),
            type_p: 0.08,
            ask_p: 0.92,
            ask: Some(Box::new(reminder.clone())),
        }),
    );
    assert_ne!(
        win.slot_line(200).label,
        "Typing into",
        "a request keeps the usual line"
    );
    heard(&mut win, 800, said);
    win.handle(900, Key::Enter.into_event());
    let out = win.handle(1_000, read(said, 0.05));
    assert!(
        out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Run { row, .. } if row.key == reminder.key))
    );
    assert!(
        !out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. }))
    );
    assert_eq!(win.text(), said, "the sentence is in the input");
}

#[test]
fn a_close_call_asks_once_and_cmd_picks() {
    let mut win = OneWindow::default();
    win.handle(0, start());
    let said = "let me know if thursday still works for the review";
    heard(&mut win, 100, said);
    win.handle(200, read(said, 0.45));
    win.handle(1_000, Key::Enter.into_event());
    let talk = win.talk().expect("the window stays");
    assert_eq!(talk.header(), talk_words::TOO_CLOSE);
    assert_eq!(win.enter_target(1_000).label, talk_words::PICK_ONE);
    let out = win.handle(1_010, Key::Enter.into_event());
    assert!(
        !out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. } | Effect::Run { .. })),
        "Enter doesn't pick for you"
    );
    let out = win.handle(1_500, Key::CmdDigit(1).into_event());
    assert!(
        out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. }))
    );
}

#[test]
fn escape_drops_everything() {
    let mut win = OneWindow::default();
    win.handle(0, start());
    heard(&mut win, 100, SAID);
    win.handle(200, read(SAID, 0.9));
    let out = win.handle(300, Key::Escape.into_event());
    assert!(
        !out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. } | Effect::Run { .. }))
    );
    assert!(out.effects.contains(&Effect::TalkEnded));
    assert!(win.talk().is_none() && win.ledger().last().is_none());
}

#[test]
fn letting_go_right_after_the_reading_changed_waits_out_the_guard() {
    let mut win = OneWindow::default();
    win.handle(0, start());
    heard(&mut win, 1_000, SAID);
    win.handle(1_100, read(SAID, 0.9));
    let out = win.handle(1_150, Event::Talk(TalkEvent::Release));
    assert!(
        !out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. })),
        "not under the key"
    );
    let wake = win.next_wake().expect("a timer for the guard");
    let out = win.handle(wake, Event::Tick);
    assert!(
        out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. }))
    );
}

#[test]
fn typed_words_in_the_voice_view_are_heard() {
    let mut win = OneWindow::default();
    win.handle(0, start());
    let out = win.handle(10, Event::Text("one two three four".into()));
    assert!(
        out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::TalkAsk { .. }))
    );
    assert_eq!(win.talk().unwrap().heard, "one two three four");
}

#[test]
fn talking_to_the_model_opens_its_chat_with_the_words() {
    let mut win = OneWindow::default();
    win.handle(
        0,
        Event::Talk(TalkEvent::StartTo {
            dest: sk_one_window::Dest::Chat,
        }),
    );
    assert_eq!(win.talk().unwrap().header(), talk_words::talking_to(true));
    heard(&mut win, 500, "what changed in the launcher this week");
    let out = win.handle(1_000, Event::Talk(TalkEvent::Release));
    assert!(out.effects.iter().any(|fx| matches!(
        fx,
        Effect::Work(sk_one_window::work::WorkEffect::AskModel { text }) if text == "what changed in the launcher this week"
    )));
    assert!(win.talk().is_none());
    assert!(
        !out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::Paste { .. }))
    );
}

#[test]
fn talking_to_todays_note_adds_the_words() {
    let mut win = OneWindow::default();
    win.handle(
        0,
        Event::Talk(TalkEvent::StartTo {
            dest: sk_one_window::Dest::Note,
        }),
    );
    heard(&mut win, 500, "pick up milk on the way home");
    let out = win.handle(1_000, Event::Talk(TalkEvent::Release));
    assert!(out.effects.contains(&Effect::AddToNote {
        text: "pick up milk on the way home".into()
    }));
}

#[test]
fn a_row_a_view_lists_can_be_staged() {
    let mut win = OneWindow::default();
    let subject = sk_one_window::Subject {
        kind: sk_one_window::SubjectKind::Thing,
        label: "Acting on".into(),
        title: "hi sarah thanks for checking in".into(),
        row_key: Some("history:1".into()),
        ..Default::default()
    };
    let out = win.handle(0, Event::Stage(subject.clone()));
    assert!(out.effects.contains(&Effect::ActionsFor(subject)));
    assert_eq!(win.slot_line(0).title, "hi sarah thanks for checking in");
}

trait IntoEvent {
    fn into_event(self) -> Event;
}

impl IntoEvent for Key {
    fn into_event(self) -> Event {
        Event::Key(self)
    }
}

/// The page's voice sessions, fed to the reducer's talk rule: the voice key starts a session, the
/// heard words are typed into it, and each reading the page showed (`voice:<n>:type|ask`) is fed
/// as Jev's answer for the words heard then. The page doesn't record Jev's numbers, so an answer
/// is sure (0.9) unless the page asked once after letting go, where the final answer is close.
/// The line above the input, the ↵ label, and the header must follow the page's.
#[test]
fn s17_replays_through_the_talk_rule() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/traces/s17-dictation.jsonl");
    let lines: Vec<Value> = std::fs::read_to_string(path)
        .expect("trace")
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let mut win = OneWindow::default();
    let (mut want_slot, mut want_enter, mut want_head) = (Vec::new(), Vec::new(), Vec::new());
    let (mut got_slot, mut got_enter, mut got_head) = (Vec::new(), Vec::new(), Vec::new());
    let push = |v: &mut Vec<String>, s: String| {
        if v.last() != Some(&s) {
            v.push(s);
        }
    };
    for (ix, line) in lines.iter().enumerate() {
        let t = line["t"].as_u64().unwrap_or(0);
        let s = |k: &str| line[k].as_str().unwrap_or_default().to_string();
        if let Some(out) = line["out"].as_str() {
            match out {
                "slot" => push(
                    &mut want_slot,
                    format!("{}|{}|{}", s("kind"), s("label"), s("title")),
                ),
                "enter" if s("id").starts_with("voice:") => push(&mut want_enter, s("label")),
                "header" => push(
                    &mut want_head,
                    line["head"]
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or(s("words")),
                ),
                _ => {}
            }
            continue;
        }
        if line["ev"].is_null() {
            continue;
        }
        let via = line["via"].as_str();
        let talking = win.talk().is_some();
        let event = match (line["ev"].as_str(), line["key"].as_str()) {
            (Some("init"), _) => Some(Event::Focused(sk_one_window::trace::subject_of(
                serde_json::from_value(line["focused"].clone()).unwrap(),
            ))),
            (Some("key"), Some("Fn")) => Some(start()),
            (Some("key"), Some("Enter")) if talking => {
                let mut effects = win.handle(t, Event::Key(Key::Enter)).effects;
                let heard = win
                    .talk()
                    .map(|talk| talk.heard.clone())
                    .unwrap_or_default();
                let asks_once = lines[ix + 1..]
                    .iter()
                    .find(|l| l["ev"] == "viewEnter" || l["ev"] == "key")
                    .is_some_and(|l| l["id"].as_str().is_some_and(|id| id.ends_with(":pick")));
                let current = win.talk().and_then(|talk| talk.current());
                let p = match (asks_once, current) {
                    (true, _) => 0.55,
                    (false, Some(sk_one_window::Reading::Type)) => 0.9,
                    (false, _) => 0.1,
                };
                effects.extend(win.handle(t, read(&heard, p)).effects);
                let ran = effects.iter().find_map(|fx| match fx {
                    Effect::Run { row, .. } => Some(row.title.clone()),
                    _ => None,
                });
                if let Some(title) = ran {
                    win.handle(
                        t,
                        Event::Done(Done {
                            icon: "schedule".into(),
                            title,
                            risk: Some(Risk::Undo),
                            ..Done::default()
                        }),
                    );
                }
                None
            }
            (Some("key"), Some(key)) if via.is_none() => {
                Some(Event::Key(sk_one_window::trace::parse_key(key)))
            }
            (Some("text"), _) if talking => Some(Event::Text(s("text"))),
            (Some("viewEnter"), _) if talking && via.is_none() => {
                let id = s("id");
                let label = s("label");
                let heard = win.talk().unwrap().heard.clone();
                let type_p = if id.ends_with(":type") { 0.9 } else { 0.1 };
                let ask = (id.ends_with(":ask") && label != talk_words::DO_IT_AS_REQUEST)
                    .then(|| Box::new(Row::new(format!("jev:{label}"), label)));
                Some(Event::Talk(TalkEvent::Read {
                    words: heard,
                    type_p,
                    ask_p: 1.0 - type_p,
                    ask,
                }))
            }
            (Some("summon"), _) => Some(Event::Summon),
            (Some("dismiss"), _) if !talking && via.is_none() => Some(Event::Dismissed),
            _ => None,
        };
        if let Some(event) = event {
            win.handle(t, event);
        }
        let slot = win.slot_line(t);
        push(
            &mut got_slot,
            format!(
                "{}|{}|{}",
                slot_kind_words(slot.kind),
                slot.label,
                slot.title
            ),
        );
        let enter = win.enter_target(t);
        if enter.id.starts_with("voice:") {
            push(&mut got_enter, enter.label);
        }
        push(
            &mut got_head,
            match win.talk() {
                Some(talk) => talk.header(),
                None if win.in_view() => String::new(),
                None => win.list().header(),
            },
        );
    }
    assert_eq!(got_slot, want_slot, "the line above the input");
    assert_eq!(got_enter, want_enter, "what ↵ says while talking");
    assert_eq!(got_head, want_head, "the header");
}
