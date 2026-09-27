//! Holding the voice key: the two readings stay put, and letting go types only when it's sure.

use proptest::prelude::*;
use sk_one_window::window::{Effect, Event, Key, OneWindow};

#[derive(Clone, Debug)]
enum TalkStep {
    Word,
    Read { words_back: usize, type_p: u8 },
    Pick(u8),
    Release,
    Escape,
    Wait(u16),
    Start,
}

fn talk_step() -> impl Strategy<Value = TalkStep> {
    prop_oneof![
        5 => Just(TalkStep::Word),
        4 => (0usize..3, 0u8..=100).prop_map(|(words_back, type_p)| TalkStep::Read { words_back, type_p }),
        1 => (1u8..=2).prop_map(TalkStep::Pick),
        2 => Just(TalkStep::Release),
        1 => Just(TalkStep::Escape),
        2 => (1u16..800).prop_map(TalkStep::Wait),
        1 => Just(TalkStep::Start),
    ]
}

const SPOKEN: &[&str] = &[
    "Hi",
    "Sarah,",
    "the",
    "review",
    "moved",
    "to",
    "Thursday.",
    "remind",
    "me",
    "at",
    "ten",
];

proptest! {
    /// While talking: the two rows never reorder; Jev's answers change the highlighted reading at
    /// most once per session, and never after it was picked; and the words paste only when the
    /// user picked typing or Jev's answer to exactly the final words agrees with typing and is at
    /// least 65 % sure.
    #[test]
    fn voice_readings_are_steady_and_letting_go_acts_only_when_sure(
        steps in prop::collection::vec(talk_step(), 1..60)
    ) {
        use sk_one_window::talk::{Reading, TalkEvent};
        use sk_one_window::contract::TALK_SURE_P;
        let mut win = OneWindow::default();
        let mut now = 0;
        let mut spoken: Vec<&str> = Vec::new();
        let mut changes = 0;
        for step in steps {
            now += 30;
            let before = win.talk().cloned();
            let event = match &step {
                TalkStep::Start => {
                    spoken.clear();
                    changes = 0;
                    Event::Talk(TalkEvent::Start { app: "Mail".into(), field: "Reply".into(), to: None })
                }
                TalkStep::Word => {
                    spoken.push(SPOKEN[spoken.len() % SPOKEN.len()]);
                    Event::Talk(TalkEvent::Heard(spoken.join(" ")))
                }
                TalkStep::Read { words_back, type_p } => {
                    let n = spoken.len().saturating_sub(*words_back);
                    let p = f32::from(*type_p) / 100.0;
                    Event::Talk(TalkEvent::Read { words: spoken[..n].join(" "), type_p: p, ask_p: 1.0 - p, ask: None })
                }
                TalkStep::Pick(d) => Event::Key(Key::CmdDigit(*d)),
                TalkStep::Release => Event::Talk(TalkEvent::Release),
                TalkStep::Escape => Event::Key(Key::Escape),
                TalkStep::Wait(ms) => {
                    now += u64::from(*ms);
                    Event::Tick
                }
            };
            let out = win.handle(now, event.clone());
            let pasted = out.effects.iter().any(|fx| matches!(fx, Effect::Paste { .. }));
            if let Some(before) = &before {
                if pasted {
                    let picked = matches!(step, TalkStep::Pick(1)) || before.locked == Some(Reading::Type);
                    let sure = |words: &str, type_p: f32, ask_p: f32| {
                        words.trim() == before.heard.trim() && type_p >= ask_p && type_p >= TALK_SURE_P
                    };
                    // The clock runs before any event lands, so when the Enter guard runs out an
                    // answer that was already sure acts even if the event is a newer answer.
                    let was_sure = before.answer.as_ref().is_some_and(|a| sure(&a.words, a.type_p, a.ask_p));
                    let jev_sure = was_sure
                        || matches!(&event, Event::Talk(TalkEvent::Read { words, type_p, ask_p, .. }) if sure(words, *type_p, *ask_p));
                    prop_assert!(picked || jev_sure, "pasted without a pick or a sure answer: {:?}", before);
                    prop_assert!(!matches!(step, TalkStep::Escape), "Esc pastes nothing");
                }
            }
            if let Some(talk) = win.talk() {
                let rows = talk.rows();
                prop_assert_eq!(rows[0].key.as_str(), "talk:type");
                prop_assert_eq!(rows[1].key.as_str(), "talk:ask");
                let answered = matches!(step, TalkStep::Read { .. });
                if let Some(before) = &before {
                    if answered && before.reading.is_some() && before.reading != talk.reading {
                        prop_assert!(before.locked.is_none(), "a picked reading changed");
                        changes += 1;
                    }
                }
                prop_assert!(changes <= 1, "the reading changed {} times", changes);
            }
        }
    }
}
