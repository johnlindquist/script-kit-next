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
            copy: Some(source(
                SubjectKind::Clipboard,
                "https://example.org",
                40_000,
            )),
        }),
    );
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
