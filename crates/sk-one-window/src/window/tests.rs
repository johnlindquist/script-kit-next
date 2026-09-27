use super::*;

fn steps_view() -> ViewSpec {
    ViewSpec {
        name: "steps".into(),
        label: "Steps".into(),
        takes_input: true,
        ..ViewSpec::default()
    }
}

/// Runs a steps view that holds two messages on its own and finishes as a run.
fn ran_steps(win: &mut OneWindow) -> (u64, u64) {
    win.handle(0, Event::View(ViewOp::Open(steps_view())));
    win.handle(
        10,
        Event::Hold {
            label: "To Sarah: “friday”".into(),
            request: "tell Sarah friday".into(),
        },
    );
    win.handle(
        20,
        Event::Hold {
            label: "To #launcher: “slipped”".into(),
            request: "post in #launcher slipped".into(),
        },
    );
    win.handle(
        30,
        Event::Done(Done {
            icon: "steps".into(),
            title: "Did 2 steps".into(),
            ..Done::default()
        }),
    );
    let ids: Vec<u64> = win.holds().live().map(|h| h.id).collect();
    (ids[0], ids[1])
}

fn reading(key: &str, verb: &str, target: &str) -> Row {
    Row {
        verb: Some(verb.into()),
        source: Some("last".into()),
        target: Some(target.into()),
        p: Some(0.9),
        ..Row::new(key, key)
    }
}

#[test]
fn a_message_a_view_holds_waits_for_the_view_to_finish_before_the_line_shows_it() {
    let mut win = OneWindow::default();
    win.handle(0, Event::View(ViewOp::Open(steps_view())));
    win.handle(
        10,
        Event::Hold {
            label: "To Sarah: “friday”".into(),
            request: String::new(),
        },
    );
    assert_ne!(win.slot_line(10).kind, SlotKind::Holding);
    win.handle(
        20,
        Event::Done(Done {
            icon: "steps".into(),
            title: "Did 1 step".into(),
            ..Done::default()
        }),
    );
    let slot = win.slot_line(20);
    assert_eq!(slot.kind, SlotKind::Holding);
    assert_eq!(slot.title, "To Sarah: “friday”");
}

#[test]
fn messages_a_run_sent_leave_the_run_last_unless_they_were_changed() {
    let mut win = OneWindow::default();
    ran_steps(&mut win);
    win.handle(
        40,
        Event::Rewrite {
            target: "To Sarah: “friday”".into(),
            text: "thursday".into(),
        },
    );
    win.handle(8_030, Event::Tick);
    assert_eq!(win.ledger().last().unwrap().title, "Did 2 steps");
    win.handle(9_000, Event::Tick);
    let last = win.ledger().last().unwrap();
    assert_eq!(
        last.title, "To Sarah: “thursday”",
        "a changed message is its own thing"
    );
    let mut win = OneWindow::default();
    ran_steps(&mut win);
    win.handle(20_000, Event::Tick);
    assert_eq!(win.ledger().last().unwrap().title, "Did 2 steps");
    assert_eq!(win.slot_line(20_000).title, "Did 2 steps");
}

#[test]
fn a_reading_that_takes_back_stops_the_message_it_names() {
    let mut win = OneWindow::default();
    ran_steps(&mut win);
    win.handle(500, Event::Text("don't send the launcher one".into()));
    win.handle(
        600,
        Event::Jev {
            text: "don't send the launcher one".into(),
            rows: vec![reading("jev:unsend", "undo", "To #launcher: “slipped”")],
        },
    );
    let out = win.handle(700, Event::Key(Key::Enter));
    assert_eq!(
        out.picked.map(|(r, _)| r.key).as_deref(),
        Some("jev:unsend")
    );
    assert!(!out.effects.iter().any(|e| matches!(e, Effect::Run { .. })));
    let live: Vec<&str> = win.holds().live().map(|h| h.label.as_str()).collect();
    assert_eq!(live, ["To Sarah: “friday”"]);
    assert!(win.text().is_empty(), "it finished and went home");
}

#[test]
fn a_reading_that_takes_back_undoes_the_last_run_when_nothing_waits() {
    let mut win = OneWindow::default();
    win.handle(
        0,
        Event::Done(Done {
            icon: "move".into(),
            title: "Moved standup".into(),
            ..Done::default()
        }),
    );
    win.handle(500, Event::Text("undo that".into()));
    win.handle(
        600,
        Event::Jev {
            text: "undo that".into(),
            rows: vec![reading("jev:undo", "undo", "Moved standup")],
        },
    );
    win.handle(700, Event::Key(Key::Enter));
    assert_eq!(win.ledger().last().unwrap().title, "Undid · Moved standup");
}

#[test]
fn an_enter_that_waited_for_jev_asks_once_when_the_readings_are_close() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("lunch with sarah".into()));
    win.handle(10, Event::Key(Key::Enter));
    let close = |k: &str, p: f32| Row {
        p: Some(p),
        ..Row::new(k, k)
    };
    let out = win.handle(
        200,
        Event::Jev {
            text: "lunch with sarah".into(),
            rows: vec![close("a", 0.5), close("b", 0.45)],
        },
    );
    assert!(out.picked.is_none(), "too close to run unasked");
    assert_eq!(win.enter_target(200).label, words::ask_again("a"));
    let out = win.handle(900, Event::Key(Key::Enter));
    assert_eq!(out.picked.map(|(r, _)| r.key).as_deref(), Some("a"));
}

#[test]
fn going_elsewhere_hides_the_window_and_records_nothing() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Dismissed);
    assert!(win.is_away());
    assert!(!win.handle(10, Event::Key(Key::Enter)).consumed);
    win.handle(DISMISS_SETTLE_MS, Event::Tick);
    assert!(win.ledger().is_empty());
    assert_eq!(win.slot_line(DISMISS_SETTLE_MS).kind, SlotKind::Focused);
}

#[test]
fn actions_the_host_lists_keep_what_the_window_does_for_its_own_rows() {
    let mut win = OneWindow::default();
    win.handle(
        0,
        Event::Done(Done {
            icon: "move".into(),
            title: "Moved standup".into(),
            ..Done::default()
        }),
    );
    win.handle(1_000, Event::Key(Key::Up));
    win.handle(
        1_000,
        Event::Actions(vec![
            Row::new("act:Open it", "Open it"),
            Row::new("act:Undo it", "Undo it"),
        ]),
    );
    let rows = &win.staged().unwrap().rows;
    assert_eq!(rows[0].act, RowAct::Host);
    assert_eq!(rows[1].act, RowAct::Undo);
}

#[test]
fn a_line_a_view_set_goes_with_the_view() {
    let mut win = OneWindow::default();
    win.handle(0, Event::View(ViewOp::Open(steps_view())));
    win.handle(
        10,
        Event::Line(Some(SlotLine::new(SlotKind::Typing, "Typing into", "Mail"))),
    );
    let slot = win.slot_line(10);
    assert_eq!(slot.kind, SlotKind::Typing);
    assert!(
        win.slot_opacity(&slot) < 1.0,
        "inside a view it steps back too"
    );
    win.handle(20, Event::View(ViewOp::Back));
    assert_eq!(win.slot_line(20).kind, SlotKind::Focused);
}

#[test]
fn going_back_to_the_same_words_keeps_their_list() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("tell sarah".into()));
    win.handle(
        1,
        Event::Names(vec![Row::new("steps", "Do these 2 things")]),
    );
    win.handle(10, Event::Key(Key::Enter));
    win.handle(11, Event::View(ViewOp::Open(steps_view())));
    win.handle(20, Event::Text("both".into()));
    win.handle(30, Event::View(ViewOp::Back));
    assert_eq!(win.text(), "tell sarah");
    assert_eq!(win.list().header(), words::MATCHES);
}

#[test]
fn when_jev_is_down_the_ask_ai_fallback_goes_at_once() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("book a table".into()));
    win.handle(
        1,
        Event::Names(vec![
            Row::new("fallback-google:book a table", "Search Google").fallback(),
            Row::new("fallback-ai:book a table", "Ask AI").fallback(),
        ]),
    );
    win.handle(
        500,
        Event::JevDown {
            text: "book a table".into(),
        },
    );
    let keys: Vec<&str> = win.list().names.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["fallback-google:book a table"]);
}
