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
    assert!(
        win.recent_line(10)
            .is_none_or(|line| line.kind != SlotKind::Holding)
    );
    win.handle(
        20,
        Event::Done(Done {
            icon: "steps".into(),
            title: "Did 1 step".into(),
            ..Done::default()
        }),
    );
    let slot = win.recent_line(20).unwrap();
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
    assert_eq!(win.recent_line(20_000).unwrap().title, "Did 2 steps");
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
    assert_eq!(win.slot_line(DISMISS_SETTLE_MS).label, words::BEHIND);
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
    assert_eq!(win.recent_line(10).unwrap().kind, SlotKind::Typing);
    win.handle(20, Event::View(ViewOp::Back));
    assert!(win.recent_line(20).is_none());
    assert_eq!(
        win.slot_line(20).label,
        words::BEHIND,
        "nothing has run yet"
    );
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

fn dash_names() -> Vec<Row> {
    vec![
        Row::new("file:/dash", "dash").with_score(30),
        Row::new("chat:1", "Dashboard ideas").with_score(30),
        Row::new(
            format!("{}dash", crate::contract::ALL_RESULTS_KEY),
            "See all 14 results",
        ),
        Row::new("fallback-google:dash", "Search Google").fallback(),
        Row::new("fallback-search-files:dash", "Search Files").fallback(),
        Row::new("fallback-ai:dash", "Ask AI").fallback(),
        Row::new("fallback-search-duckduckgo:dash", "Search DuckDuckGo").fallback(),
    ]
}

#[test]
fn a_match_keeps_the_fallbacks_after_jevs_rows_under_their_own_header() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("dash".into()));
    win.handle(1, Event::Names(dash_names()));
    let list = win.list();
    let layout = list.layout();
    let keys: Vec<String> = layout
        .iter()
        .map(|item| match item {
            Item::Header(head) => format!("# {head}"),
            Item::Slot(i) => format!("slot {i}"),
            item => list.row(item).map(|r| r.key.clone()).unwrap_or_default(),
        })
        .collect();
    assert_eq!(
        keys,
        [
            "# Jev".to_string(),
            "slot 0".into(),
            "file:/dash".into(),
            "chat:1".into(),
            format!("{}dash", crate::contract::ALL_RESULTS_KEY),
            "# Use “dash” with…".into(),
            "fallback-google:dash".into(),
            "fallback-search-files:dash".into(),
            "fallback-ai:dash".into(),
        ]
    );
}

#[test]
fn the_row_that_opens_every_result_never_owns_enter() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("dash".into()));
    let mut names = dash_names();
    names[0].score = 100;
    names.remove(1);
    win.handle(1, Event::Names(names));
    assert_eq!(win.enter_target(1).id, "file:/dash");
    assert!(
        win.list()
            .names
            .iter()
            .filter(|r| r.key.starts_with(crate::contract::ALL_RESULTS_KEY))
            .all(|r| !r.is_strong())
    );
    // A strong name leads; the fallbacks still come last, after Jev's rows.
    let visible = win.list().visible();
    assert_eq!(visible.first(), Some(&Item::Name(0)));
    let last = visible.last().and_then(|item| win.list().row(item));
    assert!(last.is_some_and(|r| r.fallback));
}

#[test]
fn with_nothing_matched_the_fallbacks_stand_where_the_names_would() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("dash".into()));
    let names: Vec<Row> = dash_names().into_iter().filter(|r| r.fallback).collect();
    win.handle(1, Event::Names(names));
    assert_eq!(win.list().names.len(), 4);
    assert!(win.list().names.iter().all(|r| r.head.is_none()));
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

fn front_app() -> Subject {
    Subject {
        kind: SubjectKind::Window,
        label: words::FOCUSED.into(),
        title: "Ghostty".into(),
        icon: "window".into(),
        ..Subject::default()
    }
}

fn copied() -> Event {
    Event::Done(Done {
        icon: "copy".into(),
        title: "Copied".into(),
        ..Done::default()
    })
}

#[test]
fn the_chip_offers_whats_focused_and_the_line_reports_what_just_ran() {
    let mut win = OneWindow::default();
    let line = win.slot_line(0);
    assert_eq!(
        (line.label.as_str(), line.title.as_str()),
        (words::BEHIND, words::THE_DESKTOP),
        "with nothing in front and nothing run, the line still says something"
    );
    win.handle(0, Event::Focused(front_app()));
    let line = win.slot_line(0);
    assert_eq!(
        (line.label.as_str(), line.title.as_str()),
        (words::BEHIND, "Ghostty")
    );
    assert!(line.hints.is_empty(), "Space is on the chip, not the line");
    win.handle(0, copied());
    let chip = win.chip().expect("what's focused is offered");
    assert_eq!(
        (chip.title.as_str(), chip.attached, chip.key.as_str()),
        ("Ghostty", false, "space")
    );
    assert_eq!(chip.word, words::USE);
    let line = win.slot_line(0);
    assert_eq!(
        (line.kind, line.title.as_str()),
        (SlotKind::Recent, "Copied")
    );
    assert!(!line.lit);
    assert_eq!(
        line.icon,
        crate::contract::RECENT_ICON,
        "whatever ran, it reads as history"
    );
    assert!(
        line.hints.iter().any(|(key, _)| key == "↑"),
        "↑ is how it comes back: {:?}",
        line.hints
    );

    win.handle(ENTER_GUARD_MS, Event::Key(Key::Up));
    assert_eq!(
        win.staged().expect("↑ puts it in play").subject.title,
        "Copied"
    );
    let chip = win.chip().expect("attached");
    assert_eq!(
        (chip.title.as_str(), chip.attached, chip.key.as_str()),
        ("Copied", true, "esc")
    );
    let line = win.slot_line(ENTER_GUARD_MS);
    assert!(line.lit, "the run it came from is what's in play");
    assert!(line.hints.iter().all(|(key, _)| key != "↑"), "↑ was spent");

    win.handle(ENTER_GUARD_MS + 10, Event::Key(Key::Escape));
    assert!(win.staged().is_none());
    assert!(!win.slot_line(ENTER_GUARD_MS + 10).lit);
    assert!(!win.chip().unwrap().attached, "the offer comes back");
}

#[test]
fn typing_takes_the_offer_away_and_emptying_the_input_brings_it_back() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Focused(front_app()));
    win.handle(10, Event::Text("tell".into()));
    assert!(win.chip().is_none());
    win.handle(20, Event::Text(String::new()));
    assert_eq!(win.chip().unwrap().title, "Ghostty");
}

#[test]
fn space_attaches_whats_focused_and_cmd_k_takes_a_row_you_chose() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Focused(front_app()));
    win.handle(
        0,
        Event::Suggested(vec![
            Row::new("s:slack", "Slack").with_enter_label("Open Slack"),
            Row::new("s:linear", "Linear"),
        ]),
    );
    win.handle(10, Event::Key(Key::Space));
    assert_eq!(
        win.staged().expect("space stages it").subject.title,
        "Ghostty"
    );
    assert!(win.chip().unwrap().attached);
    assert_eq!(win.staged().unwrap().head, words::OR_PICK_ONE);
    assert_eq!(win.placeholder(), Some(words::SAY_WHAT_TO_DO));
    win.handle(20, Event::Key(Key::Escape));
    assert!(win.staged().is_none());
    assert_eq!(
        win.placeholder(),
        None,
        "the launcher's own placeholder comes back"
    );

    win.handle(30, Event::Key(Key::CmdK));
    assert_eq!(
        win.staged()
            .expect("⌘K takes what the chip offers")
            .subject
            .title,
        "Ghostty"
    );
    win.handle(35, Event::Key(Key::Escape));

    win.handle(40, Event::Key(Key::Down));
    win.handle(50, Event::Key(Key::CmdK));
    assert_eq!(
        win.staged().expect("⌘K stages the row").subject.title,
        "Linear"
    );
}

#[test]
fn typing_after_the_chip_offers_what_you_said_before_then_its_actions() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Focused(front_app()));
    win.handle(10, Event::Key(Key::Space));
    win.handle(
        10,
        Event::Actions(vec![
            Row::new("act:new-tab", "New Tab"),
            Row::new("act:new-window", "New Window"),
        ]),
    );
    win.handle(20, Event::Text("new".into()));
    let mut learned = Row::new("learned:new tab in ghostty", "New Tab in Ghostty");
    learned.learned = true;
    learned.score = 90;
    win.handle(
        30,
        Event::Names(vec![Row::new("script:news", "News"), learned]),
    );
    let names: Vec<&str> = win.list().names.iter().map(|r| r.title.as_str()).collect();
    assert_eq!(names, ["New Tab in Ghostty", "New Tab", "New Window"]);
    let out = win.handle(30 + JEV_DEBOUNCE_MS, Event::Tick);
    assert!(
        out.effects
            .iter()
            .any(|fx| matches!(fx, Effect::AskJev { .. })),
        "Jev reads the words with the chip attached"
    );
}

#[test]
fn the_use_word_goes_once_space_is_learned() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Focused(front_app()));
    for i in 0..crate::contract::USE_HINT_GRABS {
        let t = 10 + i as Ms * 20;
        let out = win.handle(t, Event::Key(Key::Space));
        assert!(out.effects.contains(&Effect::Learned), "grab {i} is saved");
        win.handle(t + 10, Event::Key(Key::Escape));
    }
    assert!(win.chip().unwrap().word.is_empty(), "just the keycap");
    let out = win.handle(1_000, Event::Key(Key::Space));
    assert!(
        !out.effects.contains(&Effect::Learned),
        "nothing more to learn"
    );
}

#[test]
fn clicking_the_receipt_puts_it_in_play_and_clicking_again_lets_go() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Focused(front_app()));
    win.handle(0, copied());
    win.handle(ENTER_GUARD_MS, Event::Key(Key::Recent));
    assert_eq!(win.staged().expect("in play").subject.title, "Copied");
    win.handle(ENTER_GUARD_MS + 10, Event::Key(Key::Recent));
    assert!(win.staged().is_none());
}

#[test]
fn what_happened_while_the_window_was_away_shows_when_it_comes_back() {
    let mut win = OneWindow::default();
    win.handle(0, copied());
    win.handle(ENTER_GUARD_MS, Event::Key(Key::Down));
    win.handle(ENTER_GUARD_MS + 10, Event::Dismissed);
    win.handle(
        ENTER_GUARD_MS + 20,
        Event::Notice(crate::notice::Notice::new("check", "Backup finished")),
    );
    win.handle(ENTER_GUARD_MS + 30, Event::Summon);
    let line = win.recent_line(ENTER_GUARD_MS + 30).unwrap();
    assert_eq!(
        (line.kind, line.title.as_str()),
        (SlotKind::Recent, "Backup finished")
    );
}

#[test]
fn the_prototypes_rules_keep_the_last_run_in_the_line() {
    let mut win = OneWindow::new(Config {
        subject_chip: false,
        ..Config::default()
    });
    win.handle(0, Event::Focused(front_app()));
    win.handle(0, copied());
    win.handle(ENTER_GUARD_MS, Event::Key(Key::Down));
    assert_eq!(win.slot_line(ENTER_GUARD_MS).kind, SlotKind::Recent);
    win.handle(ENTER_GUARD_MS + 10, Event::Key(Key::CmdK));
    assert_eq!(win.staged().expect("staged").subject.title, "Ghostty");
}
