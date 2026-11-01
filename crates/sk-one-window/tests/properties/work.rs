//! Work in the window: the model's chat, steps, connecting a service, and the agent strip.

use proptest::prelude::*;
use sk_one_window::Row;
use sk_one_window::contract::ENTER_GUARD_MS;
use sk_one_window::steps::StepSpec;
use sk_one_window::strip::TaskState;
use sk_one_window::window::{Effect, Event, Key, OneWindow};
use sk_one_window::work::{WorkEffect, WorkEvent};

use super::WORDS;

#[derive(Clone, Debug)]
enum Move {
    Type(usize),
    Key(Key),
    Task(u8, u8),
    Wait(u16),
}

fn moves() -> impl Strategy<Value = Move> {
    prop_oneof![
        3 => (0..WORDS.len()).prop_map(Move::Type),
        3 => prop_oneof![Just(Key::Down), Just(Key::Up), Just(Key::Enter), Just(Key::CmdK)].prop_map(Move::Key),
        3 => (0u8..3, 0u8..4).prop_map(|(t, s)| Move::Task(t, s)),
        2 => (1u16..900).prop_map(Move::Wait),
    ]
}

fn state(s: u8) -> TaskState {
    [
        TaskState::Working,
        TaskState::NeedsYou,
        TaskState::Ready,
        TaskState::Failed,
    ][s as usize % 4]
}

/// The same list work as `Run::new`, fed to two windows; one also gets agent tasks.
fn pair() -> (OneWindow, OneWindow) {
    let mut a = OneWindow::default();
    a.handle(0, Event::Suggested(vec![Row::new("s:slack", "Slack")]));
    let mut b = a.clone();
    b.handle(
        0,
        Event::Work(WorkEvent::Agent {
            task: "fix SK-431".into(),
        }),
    );
    b.handle(0, Event::Key(Key::Escape));
    (a, b)
}

fn view_key(win: &mut OneWindow, now: u64, key: Key) -> Vec<Effect> {
    let mut fx = win.handle(now, Event::Key(key)).effects;
    let handed: Vec<Key> = fx
        .iter()
        .filter_map(|e| match e {
            Effect::ViewKey(k) => Some(k.clone()),
            _ => None,
        })
        .collect();
    for k in handed {
        fx.extend(win.handle(now, Event::Work(WorkEvent::ViewKey(k))).effects);
    }
    fx
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// A task starting, moving along, asking or finishing never changes the list you're in.
    #[test]
    fn tasks_never_move_the_list(moves in prop::collection::vec(moves(), 1..60)) {
        let (mut plain, mut busy) = pair();
        let id = busy.work().strip().tasks()[0].id.clone();
        // Past the guard the agent's unprompted view armed, so both windows start alike.
        let mut now = ENTER_GUARD_MS;
        for m in moves {
            now += 10;
            match m {
                Move::Type(i) => {
                    plain.handle(now, Event::Text(WORDS[i].into()));
                    busy.handle(now, Event::Text(WORDS[i].into()));
                }
                Move::Key(k) => {
                    plain.handle(now, Event::Key(k.clone()));
                    busy.handle(now, Event::Key(k));
                }
                Move::Task(p, s) => {
                    busy.handle(now, Event::Work(WorkEvent::TaskChanged {
                        id: id.clone(),
                        status: format!("step {p}"),
                        progress: f32::from(p) / 3.0,
                        state: state(s),
                        approve: None,
                    }));
                    plain.handle(now, Event::Tick);
                }
                Move::Wait(ms) => {
                    now += u64::from(ms);
                    plain.handle(now, Event::Tick);
                    busy.handle(now, Event::Tick);
                }
            }
            if busy.in_view() || plain.in_view() {
                continue;
            }
            prop_assert_eq!(plain.list().layout(), busy.list().layout());
            prop_assert_eq!(plain.list().sel, busy.list().sel);
            prop_assert_eq!(plain.list().header(), busy.list().header());
        }
    }

    /// ⌘J opens whatever needs you first, then the newest task.
    #[test]
    fn cmd_j_opens_what_needs_you(states in prop::collection::vec(0u8..4, 1..5)) {
        let mut win = OneWindow::default();
        let mut ids = Vec::new();
        for (i, s) in states.iter().enumerate() {
            let now = i as u64 * 10;
            let fx = win.handle(now, Event::Work(WorkEvent::Agent { task: format!("task {i}") })).effects;
            if let Some(Effect::Work(WorkEffect::StartAgent { id, .. })) = fx.iter().find(|e| matches!(e, Effect::Work(WorkEffect::StartAgent { .. }))) {
                ids.push(id.clone());
            }
            win.handle(now, Event::Key(Key::Escape));
            win.handle(now, Event::Work(WorkEvent::TaskChanged {
                id: ids[i].clone(), status: String::new(), progress: 0.0, state: state(*s), approve: None,
            }));
        }
        let want = states.iter().position(|s| state(*s) == TaskState::NeedsYou).unwrap_or(states.len() - 1);
        let fx = win.handle(1_000, Event::Key(Key::CmdJ)).effects;
        let reopened = Effect::Work(WorkEffect::Reopened { id: ids[want].clone() });
        prop_assert!(fx.contains(&reopened));
    }

    /// The steps never offer "Do all" while a step waits on a service; ↵ names the first one.
    #[test]
    fn steps_never_run_past_a_missing_service(needs in prop::collection::vec(prop::option::of(0usize..3), 2..5), connected in prop::collection::vec(0usize..3, 0..3)) {
        const KEYS: [&str; 3] = ["calendar", "slack", "github"];
        let sentence = (0..needs.len()).map(|i| format!("open app{i}")).collect::<Vec<_>>().join(" and then ");
        let mut win = OneWindow::default();
        win.handle(0, Event::Work(WorkEvent::Connections(connected.iter().map(|k| KEYS[*k].to_string()).collect())));
        win.handle(0, Event::Work(WorkEvent::OpenSteps { sentence: sentence.clone() }));
        let texts = sk_one_window::steps::split(&sentence);
        win.handle(10, Event::Work(WorkEvent::StepsRead {
            sentence: sentence.clone(),
            steps: texts.iter().zip(&needs).map(|(t, n)| StepSpec { text: t.clone(), title: None, needs: n.map(|k| KEYS[k].to_string()) }).collect(),
        }));
        let first_missing = needs.iter().enumerate().find_map(|(i, n)| n.filter(|k| !connected.contains(k)).map(|k| (i, KEYS[k])));
        let label = win.enter_target(10).label;
        let fx = view_key(&mut win, 10 + ENTER_GUARD_MS, Key::Enter);
        let ran = fx.iter().any(|e| matches!(e, Effect::Work(WorkEffect::RunSteps { .. })));
        match first_missing {
            Some((i, key)) => {
                let name = sk_one_window::connect::name_of(key);
                let named = label.starts_with(&format!("Connect {name} for step {}", i + 1));
                prop_assert!(named, "{label}");
                prop_assert!(!ran);
                prop_assert_eq!(win.views().top().map(|f| f.name.clone()), Some("connect".to_string()));
            }
            None => {
                prop_assert!(label.starts_with("Do "), "{label}");
                prop_assert!(ran);
            }
        }
    }

    /// Once a service connects the connect view hands back, the line says so, and an Enter
    /// inside the guard is ignored.
    #[test]
    fn connecting_hands_back_under_the_guard(gap in 0u64..ENTER_GUARD_MS, from_steps in any::<bool>()) {
        let mut win = OneWindow::default();
        if from_steps {
            let s = "move standup to 10 and then open linear";
            win.handle(0, Event::Text(s.into()));
            win.handle(0, Event::Work(WorkEvent::OpenSteps { sentence: s.into() }));
            let texts = sk_one_window::steps::split(s);
            win.handle(10, Event::Work(WorkEvent::StepsRead {
                sentence: s.into(),
                steps: vec![
                    StepSpec { text: texts[0].clone(), title: None, needs: Some("calendar".into()) },
                    StepSpec { text: texts[1].clone(), title: None, needs: None },
                ],
            }));
        } else {
            win.handle(0, Event::Text("move standup to 10".into()));
            win.handle(10, Event::Jev {
                text: "move standup to 10".into(),
                rows: vec![Row { needs: Some("calendar".into()), ..Row::new("jev:move", "Move standup to 10") }],
            });
        }
        view_key(&mut win, 1_000, Key::Enter);
        prop_assert_eq!(win.views().top().map(|f| f.name.clone()), Some("connect".to_string()));
        view_key(&mut win, 1_100, Key::Enter);
        win.handle(2_000, Event::Work(WorkEvent::Connected { integration: "calendar".into() }));
        prop_assert!(win.views().top().is_none_or(|f| f.name != "connect"));
        prop_assert_eq!(win.work().connections().has("calendar"), true);
        if !from_steps {
            prop_assert_eq!(win.recent_line(2_000).unwrap().title, "Google Calendar connected");
        }
        let out = win.handle(2_000 + gap, Event::Key(Key::Enter));
        prop_assert!(out.effects.contains(&Effect::EnterIgnored));
    }

    /// A follow-up stays a draft while a turn is active; Esc restores the launcher's sentence.
    #[test]
    fn the_chat_takes_your_words_and_esc_restores_the_sentence(sentence in 0..WORDS.len(), typed in 1..WORDS.len(), replies in 0u8..3, finished in any::<bool>(), ranked in any::<bool>()) {
        let sentence = WORDS[sentence];
        let mut win = OneWindow::default();
        win.handle(0, Event::Text(sentence.into()));
        win.handle(0, Event::Work(WorkEvent::AskModel { sentence: sentence.into() }));
        for r in 0..replies {
            win.handle(10 + u64::from(r), Event::Work(WorkEvent::ModelWrote { text: format!("reply {r}"), done: finished && r + 1 == replies }));
        }
        if ranked {
            win.handle(30, Event::Work(WorkEvent::ModelNext(vec![])));
        }
        win.handle(100, Event::Text(WORDS[typed].into()));
        let t = WORDS[typed].trim();
        if !sentence.is_empty() && !(replies > 0 && finished && ranked) {
            let status = if replies == 0 { "Waiting for the model…" } else if finished { "Finding next steps…" } else { "Writing the reply…" };
            prop_assert_eq!(win.enter_target(100).label, status);
            let fx = view_key(&mut win, 1_000, Key::Enter);
            let sent = fx.iter().any(|e| matches!(e, Effect::Work(WorkEffect::AskModel { .. })));
            prop_assert!(!sent, "Enter sent a second turn while the first was active");
            prop_assert_eq!(win.text(), t);
        } else {
            prop_assert_eq!(win.enter_target(100).label, format!("Ask the model: “{t}”"));
        }
        let fx = win.handle(1_100, Event::Key(Key::Escape)).effects;
        prop_assert!(!win.in_view());
        prop_assert_eq!(win.text(), sentence);
        prop_assert!(fx.contains(&Effect::SetInput(sentence.into())) || sentence == WORDS[typed]);
    }
}
