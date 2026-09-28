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
fn a_jev_answer_with_nothing_worth_a_row_leaves_no_blank_place() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("open".into()));
    win.handle(
        10,
        Event::Jev {
            text: "open".into(),
            rows: Vec::new(),
        },
    );
    let list = win.list();
    let jev: Vec<&str> = (0..list.jev.len())
        .filter_map(|i| list.jev.get(i)?.row.as_ref())
        .map(|row| row.key.as_str())
        .collect();
    assert_eq!(jev, ["down:model", "down:note"]);
    let out = win.handle(1_000, Event::Key(Key::Enter));
    assert_eq!(
        out.picked.map(|(r, _)| r.key).as_deref(),
        Some("down:model")
    );
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
    assert_eq!(win.slot_line(10).kind, SlotKind::Typing);
    assert!(
        win.slot_style(10).opacity < 1.0,
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

fn typed(win: &mut OneWindow, text: &str) {
    win.handle(0, Event::Text(text.into()));
    win.handle(
        0,
        Event::Names(vec![
            Row::new("name:a", "Alpha"),
            Row::new("name:b", "Beta"),
        ]),
    );
}

#[test]
fn selecting_a_row_moves_enter_to_it() {
    let mut win = OneWindow::default();
    typed(&mut win, "a");
    assert_eq!(win.enter_target(0).id, "name:a");
    assert!(win.handle(10, Event::Select(1)).consumed);
    assert_eq!(win.enter_target(10).id, "name:b");
    let out = win.handle(20, Event::Key(Key::Enter));
    assert!(matches!(
        out.effects.first(),
        Some(Effect::Run { row, .. }) if row.key == "name:b"
    ));
}

#[test]
fn selecting_past_the_end_lands_on_the_last_row() {
    let mut win = OneWindow::default();
    typed(&mut win, "a");
    win.handle(10, Event::Select(99));
    assert_eq!(win.list().current(), Some(Item::Slot(0)));
}

#[test]
fn selecting_is_the_users_own_change_and_never_arms_enter() {
    let mut win = OneWindow::default();
    typed(&mut win, "a");
    win.handle(10, Event::Select(1));
    assert!(!win.enter_armed(10));
}

#[test]
fn jev_going_down_under_an_unchanged_enter_keeps_enter_live() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("open notes".into()));
    win.handle(1, Event::Names(vec![Row::new("open-notes", "Open Notes")]));
    assert_eq!(win.enter_target(1).id, "open-notes");
    win.handle(
        150,
        Event::JevDown {
            text: "open notes".into(),
        },
    );
    assert_eq!(win.enter_target(150).id, "open-notes");
    assert!(!win.enter_armed(150));
    let out = win.handle(185, Event::Key(Key::Enter));
    assert!(
        matches!(out.effects.first(), Some(Effect::Run { row, .. }) if row.key == "open-notes"),
        "{:?}",
        out.effects
    );
}

#[test]
fn jev_going_down_under_its_own_row_guards_enter() {
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
        150,
        Event::JevDown {
            text: "book a table".into(),
        },
    );
    assert!(win.enter_armed(150));
    let out = win.handle(185, Event::Key(Key::Enter));
    assert_eq!(out.effects.first(), Some(&Effect::EnterIgnored));
}

#[test]
fn words_typed_in_a_view_never_read_into_the_list() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("open notes".into()));
    win.handle(1, Event::Names(vec![Row::new("open-notes", "Open Notes")]));
    win.handle(
        10,
        Event::View(ViewOp::Open(ViewSpec {
            name: "note".into(),
            label: "Note".into(),
            takes_input: true,
            text: Some(String::new()),
            ..ViewSpec::default()
        })),
    );
    win.handle(20, Event::Text("call the venue".into()));
    win.handle(
        20,
        Event::JevDown {
            text: "call the venue".into(),
        },
    );
    win.handle(21, Event::Names(Vec::new()));
    win.handle(900, Event::View(ViewOp::Back));
    win.handle(
        901,
        Event::Names(vec![Row::new("open-notes", "Open Notes")]),
    );
    assert_eq!(win.text(), "open notes");
    assert_eq!(win.list().header(), words::MATCHES);
    assert_eq!(win.enter_target(901).id, "open-notes");
}

#[test]
fn jev_answering_the_lists_words_during_a_view_lands_in_the_list() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("book a table".into()));
    win.handle(
        10,
        Event::View(ViewOp::Open(ViewSpec {
            name: "note".into(),
            label: "Note".into(),
            takes_input: true,
            text: Some(String::new()),
            ..ViewSpec::default()
        })),
    );
    let out = win.handle(
        500,
        Event::JevDown {
            text: "book a table".into(),
        },
    );
    assert!(out.consumed);
    assert!(!win.enter_armed(500), "the view's Enter didn't change");
    win.handle(600, Event::View(ViewOp::Back));
    assert!(win.list().jev.settled_for("book a table"));
}

#[test]
fn asking_jev_again_asks_at_once() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("book a table".into()));
    win.handle(
        100,
        Event::JevDown {
            text: "book a table".into(),
        },
    );
    let out = win.handle(200, Event::Key(Key::CmdDigit(2)));
    assert!(
        out.effects.contains(&Effect::AskJev {
            text: "book a table".into()
        }),
        "{:?}",
        out.effects
    );
    assert!(
        !win.list().jev.settled_for("book a table"),
        "its rows read again"
    );
}
