//! What waits in the line above the input: irreversible runs and notices.

use proptest::prelude::*;
use sk_one_window::SlotKind;
use sk_one_window::notice::{Irreversible, Notice};
use sk_one_window::window::{Effect, Event, HeldOp, Key, OneWindow};

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Something that can't be taken back runs only once its whole wait has passed untouched,
    /// and never once it was stopped.
    #[test]
    fn an_irreversible_run_goes_only_after_its_wait(
        ticks in prop::collection::vec(1u64..3_000, 1..12),
        stop_at in prop::option::of(0usize..12),
    ) {
        let mut win = OneWindow::default();
        let out = win.handle(0, Event::Irreversible(Irreversible::new("trash", "Empty Trash")));
        let id = out.effects.iter().find_map(|e| match e {
            Effect::Held { op: HeldOp::Start, id, .. } => Some(*id),
            _ => None,
        }).expect("it waits");
        let (mut now, mut stopped, mut sent) = (0u64, false, 0);
        for (i, dt) in ticks.into_iter().enumerate() {
            now += dt;
            let ev = if stop_at == Some(i) { Event::Key(Key::CmdZ) } else { Event::Tick };
            stopped |= stop_at == Some(i) && now < sk_one_window::contract::MESSAGE_HOLD_MS;
            for fx in win.handle(now, ev).effects {
                if let Effect::Held { op: HeldOp::Sent, id: sent_id, .. } = fx {
                    prop_assert_eq!(sent_id, id);
                    prop_assert!(now >= sk_one_window::contract::MESSAGE_HOLD_MS, "ran early at {}", now);
                    prop_assert!(!stopped, "ran after it was stopped");
                    sent += 1;
                }
            }
        }
        prop_assert!(sent <= 1, "ran {} times", sent);
    }

    /// A notice is the line's last thing at once, whether the window shows or comes back later.
    #[test]
    fn a_notice_is_what_the_line_says_next(hidden in any::<bool>(), later in 0u64..60_000) {
        let mut win = OneWindow::default();
        if hidden {
            win.handle(0, Event::Dismissed);
        }
        win.handle(1, Event::Notice(Notice::new("check", "Backup finished")));
        if hidden {
            win.handle(1 + later, Event::Summon);
        }
        let line = win.recent_line(1 + later).unwrap();
        prop_assert_eq!(line.kind, SlotKind::Recent);
        prop_assert_eq!(line.title.as_str(), "Backup finished");
    }
}
