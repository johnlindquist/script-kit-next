use super::subject_tabs::{COPY_FRESH_MS, TabSource, TabsEvent};
use super::*;
fn source(kind: SubjectKind, body: &str, age_ms: Ms) -> TabSource {
    TabSource {
        subject: Subject {
            kind,
            title: body.into(),
            ..Default::default()
        },
        body: body.into(),
        label: format!("{kind:?}"),
        icon: "link".into(),
        age_ms,
        rows: vec![Row::new("read", "Read")],
    }
}
fn open(w: &mut OneWindow, now: Ms) {
    w.handle(
        now,
        Event::Subjects(TabsEvent::Open {
            window: Some(source(SubjectKind::Window, "Terminal", 65_000)),
            copy: Some(Box::new(source(
                SubjectKind::Clipboard,
                "https://example.org",
                40_000,
            ))),
        }),
    );
}
#[test]
fn preference_moves_only_a_real_action_and_enter_keeps_the_observed_subject() {
    use crate::personal::{ClipboardAction, Edit, LauncherRules, SelectionAction};
    let mut w = OneWindow::default();
    let rules = LauncherRules::default()
        .with(Edit::SelectionFirst(Some(SelectionAction::Note)))
        .with(Edit::ClipboardFirst(Some(ClipboardAction::Open)));
    w.handle(0, Event::Subjects(TabsEvent::Personal(rules)));
    open(&mut w, 1);
    let mut passage = source(SubjectKind::Selection, "the chosen passage", 0);
    passage.rows = vec![
        Row::new("ask", "Ask"),
        Row::new("note", "Add to note"),
        Row::new("handoff", "Share"),
    ];
    w.handle(2, Event::Subjects(TabsEvent::Observe(passage)));
    let selected = w.subject_page().unwrap();
    assert_eq!(
        selected
            .rows()
            .iter()
            .map(|r| r.title.as_str())
            .collect::<Vec<_>>(),
        ["Add to note", "Ask", "Share"]
    );
    w.handle(3, Event::Subjects(TabsEvent::Pick(selected.id.clone())));
    let mut later = source(SubjectKind::Clipboard, "unrelated plain text", 0);
    later.rows = vec![Row::new("ask", "Ask"), Row::new("note", "Note")];
    w.handle(600, Event::Subjects(TabsEvent::Observe(later)));
    let receipt = w.handle(1600, Event::Key(Key::Enter));
    assert_eq!(
        receipt.picked.unwrap().0.key,
        format!("subject/{}/note", selected.id)
    );
    assert_eq!(
        w.page_for_action().unwrap().source.body.as_ref(),
        "the chosen passage"
    );
    let copy = w
        .subject_pages()
        .into_iter()
        .find(|p| p.source.subject.kind == SubjectKind::Clipboard)
        .unwrap();
    assert_eq!(
        copy.rows()
            .iter()
            .map(|r| r.title.as_str())
            .collect::<Vec<_>>(),
        ["Ask", "Note"],
        "an absent Open action is never invented"
    );
}

#[test]
fn late_settings_move_only_the_automatic_default_and_guard_return() {
    use crate::personal::{Edit, LauncherRules, SelectionAction};
    let mut w = OneWindow::default();
    open(&mut w, 1);
    let mut passage = source(SubjectKind::Selection, "the untouched selection", 0);
    passage.rows = vec![Row::new("ask", "Ask"), Row::new("note", "Add to note")];
    w.handle(2, Event::Subjects(TabsEvent::Observe(passage)));
    let page = w.subject_page().unwrap().id;
    let rules = LauncherRules::default().with(Edit::SelectionFirst(Some(SelectionAction::Note)));
    let mut chosen = w.clone();
    chosen.handle(850, Event::Key(Key::Down));
    chosen.handle(900, Event::Key(Key::Up));
    for app in [&mut w, &mut chosen] {
        app.handle(900, Event::Subjects(TabsEvent::Personal(rules)));
    }
    assert_eq!(
        w.list.current_row().unwrap().key,
        format!("subject/{page}/note")
    );
    assert!(
        w.handle(901, Event::Key(Key::Enter)).picked.is_none(),
        "a return aimed at the old default cannot send the passage"
    );
    assert_eq!(
        w.handle(1500, Event::Key(Key::Enter)).picked.unwrap().0.key,
        format!("subject/{page}/note")
    );
    assert_eq!(
        chosen.list.current_row().unwrap().key,
        format!("subject/{page}/ask")
    );
    assert_eq!(
        chosen
            .handle(1500, Event::Key(Key::Enter))
            .picked
            .unwrap()
            .0
            .key,
        format!("subject/{page}/ask")
    );
}

#[test]
fn minimal_home_keeps_scripts_and_setup_but_not_stale_subject_actions() {
    use crate::personal::{Edit, HomeMode, LauncherRules};
    let mut w = OneWindow::default();
    open(&mut w, 0);
    let stock = LauncherRules::default();
    let script = "script/main:source-sha256-example";
    let setup = crate::setup::needs_setup_rows(&[crate::setup::Missing::Bun]);
    let mut rows = setup.clone();
    rows.extend([
        Row::new("app/mail", "Mail"),
        Row::new(script, "Hello World"),
        Row::new("scriptlet/main:stamp", "Date stamp"),
    ]);
    w.handle(1, Event::Suggested(rows));
    w.handle(
        800,
        Event::Subjects(TabsEvent::Personal(
            stock.with(Edit::Home(HomeMode::Minimal)),
        )),
    );
    assert!(!w.tabs_visible());
    assert_eq!(
        w.list
            .pinned
            .iter()
            .map(|r| r.key.as_str())
            .collect::<Vec<_>>(),
        [crate::setup::BUN_KEY, script, "scriptlet/main:stamp"]
    );
    assert_eq!(w.list.pinned[1].head.as_deref(), Some("Scripts"));
    // A delayed clipboard observation must not restore the now-hidden action or retarget Return.
    w.handle(
        801,
        Event::Subjects(TabsEvent::Observe(source(
            SubjectKind::Clipboard,
            "later",
            0,
        ))),
    );
    assert!(w.handle(802, Event::Key(Key::Enter)).picked.is_none());
    w.handle(1500, Event::Key(Key::Down));
    assert_eq!(
        w.handle(2000, Event::Key(Key::Enter)).picked.unwrap().0.key,
        script
    );
    w.handle(2200, Event::Text("Mail".into()));
    w.handle(2201, Event::Names(vec![Row::new("app/mail", "Mail")]));
    assert_eq!(w.list.names[0].key, "app/mail");
    w.handle(2300, Event::Text(String::new()));
    w.handle(2301, Event::Subjects(TabsEvent::Personal(stock)));
    assert!(w.tabs_visible());
    assert!(w.list.pinned.iter().any(|r| r.key == "app/mail"));
}

#[test]
fn late_subjects_do_not_retarget_a_deliberate_pick() {
    let mut w = OneWindow::default();
    open(&mut w, 100);
    assert_eq!(
        w.subject_page().unwrap().source.subject.kind,
        SubjectKind::Clipboard
    );
    w.handle(110, Event::Key(Key::Tab));
    let window = w.subject_page().unwrap();
    w.handle(
        300,
        Event::Subjects(TabsEvent::Observe(source(
            SubjectKind::Selection,
            "error",
            0,
        ))),
    );
    assert_eq!(w.subject_page().unwrap().id, window.id);
    let run = w.handle(310, Event::Key(Key::Enter));
    assert_eq!(
        run.picked.unwrap().0.key,
        format!("subject/{}/read", window.id)
    );
    w.handle(311, Event::Key(Key::Escape));
    assert_eq!(
        w.subject_page().unwrap().source.subject.kind,
        SubjectKind::Selection
    );
    w.handle(312, Event::Key(Key::BackTab));
    assert_eq!(w.subject_page().unwrap().id, window.id);
    w.handle(313, Event::Text("explain this".into()));
    w.handle(314, Event::Key(Key::Tab));
    assert!(w.agent_owns());
    assert_eq!(w.subject_page().unwrap().id, window.id);
}
#[test]
fn an_ignored_or_expired_copy_can_still_be_chosen_and_staged_without_retargeting() {
    let mut w = OneWindow::default();
    open(&mut w, 0);
    let old = w.subject_page().unwrap();
    open(&mut w, 100);
    assert!(!w.subject_pages().iter().any(|p| p.id == old.id));
    w.handle(
        COPY_FRESH_MS,
        Event::Subjects(TabsEvent::Paste(source(
            SubjectKind::Clipboard,
            "first line\nsecond line",
            0,
        ))),
    );
    w.handle(COPY_FRESH_MS + 1, Event::Key(Key::Space));
    w.handle(
        COPY_FRESH_MS + 2,
        Event::Subjects(TabsEvent::Observe(source(
            SubjectKind::Clipboard,
            "replacement",
            0,
        ))),
    );
    assert_eq!(
        w.page_for_action().unwrap().source.body.as_ref(),
        "first line\nsecond line"
    );
    w.handle(COPY_FRESH_MS + 3, Event::Key(Key::Escape));
    assert!(
        !w.handle(COPY_FRESH_MS + 4, Event::Subjects(TabsEvent::Pick(old.id)))
            .consumed
    );
    w.handle(2 * COPY_FRESH_MS + 5, Event::Tick);
    assert_eq!(w.subject_pages().len(), 1);
}
#[test]
fn selection_and_copy_of_the_same_text_count_once_and_history_walks_back() {
    let mut w = OneWindow::default();
    open(&mut w, 0);
    w.handle(
        1,
        Event::Subjects(TabsEvent::Observe(source(
            SubjectKind::Selection,
            "https://example.org",
            0,
        ))),
    );
    assert_eq!(w.subject_pages().len(), 2);
    for title in ["First run", "Second run"] {
        w.handle(
            2,
            Event::Done(Done {
                title: title.into(),
                ..Done::default()
            }),
        );
    }
    w.handle(3, Event::Key(Key::Up));
    assert_eq!(w.subject_page().unwrap().source.subject.title, "Second run");
    w.handle(4, Event::Key(Key::Up));
    assert_eq!(w.subject_page().unwrap().source.subject.title, "First run");
}
