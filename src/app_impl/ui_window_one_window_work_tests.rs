//! The model's chat, steps, the connect view and agents, driven through the real launcher: every
//! one opens under the same input in the main window and no other window opens.

use super::*;

fn top_view(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Option<String> {
    app.update(cx, |app, _| {
        app.one_window.views().top().map(|top| top.name.clone())
    })
}

fn enter_label(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| app.one_window_enter_label())
}

fn slot_title(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| {
        app.one_window.slot_line(app.one_window_clock()).title
    })
}

fn windows(cx: &mut gpui::TestAppContext) -> usize {
    cx.update(|cx| cx.windows().len())
}

fn still_the_launcher(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> bool {
    app.update(cx, |app, _| matches!(app.current_view, AppView::ScriptList))
}

#[test]
fn cmd_enter_with_nothing_typed_asks_the_model_under_the_same_input() {
    let (opened, asked, replied, back) = in_launcher("model_chat_in_place", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        let before = windows(cx);
        key(&app, cx, Key::CmdEnter);
        wait(cx, 20);
        let opened = (
            top_view(&app, cx),
            still_the_launcher(&app, cx),
            windows(cx) == before,
        );
        type_into_launcher(cx, window, &app, "name the launch party");
        wait(cx, 20);
        let asked = enter_label(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, ENTER_GUARD_MS + 50);
        let replied = app.update(cx, |app, _| {
            (
                app.one_window
                    .chat()
                    .and_then(|c| c.reply().map(str::to_string)),
                app.one_window_enter_label(),
                app.filter_text.clone(),
            )
        });
        key(&app, cx, Key::Escape);
        wait(cx, 20);
        (
            opened,
            asked,
            replied,
            (top_view(&app, cx), windows(cx) == before),
        )
    });
    assert_eq!(
        opened,
        (Some("chat".into()), true, true),
        "the chat is a view, not a window"
    );
    assert_eq!(asked, "Ask the model: “name the launch party”");
    let (reply, label, input) = replied;
    assert_eq!(reply.as_deref(), Some("Fixture Agent Chat response."));
    assert_eq!(label, "Copy it", "the reply's first next step owns ↵");
    assert_eq!(
        input, "",
        "the input is the chat's and clears for the next turn"
    );
    assert_eq!(back, (None, true), "Esc goes back to the list");
}

#[test]
fn several_jobs_become_steps_that_connect_calendar_in_place_and_carry_on() {
    const SENTENCE: &str =
        "move standup to tomorrow at 9, tell mo i'm running late and open the beta notes";
    let receipt = in_launcher("steps_connect_in_place", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        let before = windows(cx);
        type_into_launcher(cx, window, &app, SENTENCE);
        wait(cx, 20);
        let row = enter_label(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let needs = (top_view(&app, cx), enter_label(&app, cx));
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let connect = (
            app.update(cx, |app, _| app.one_window.views().crumb()),
            enter_label(&app, cx),
        );
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let signing_in = enter_label(&app, cx);
        wait(cx, 700);
        let handed_back = (
            top_view(&app, cx),
            slot_title(&app, cx),
            app.update(cx, |app, _| app.filter_text.clone()),
            enter_label(&app, cx),
        );
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, 50);
        let done = app.update(cx, |app, _| {
            (
                app.one_window.in_view(),
                app.one_window.work().strip().tasks().len(),
                app.one_window
                    .ledger()
                    .entries()
                    .iter()
                    .map(|e| e.title.clone())
                    .collect::<Vec<_>>(),
            )
        });
        (
            row,
            needs,
            connect,
            signing_in,
            handed_back,
            done,
            windows(cx) == before,
        )
    });
    let (row, needs, connect, signing_in, handed_back, done, one_window) = receipt;
    assert_eq!(row, "Do these 3 things");
    assert_eq!(
        needs,
        (
            Some("steps".into()),
            "Connect Google Calendar for step 1, then do all 3".into()
        )
    );
    assert_eq!(
        connect,
        (
            "Steps › Connect Google Calendar".into(),
            "Connect Google Calendar".into()
        )
    );
    assert_eq!(signing_in, "Connecting Google Calendar…");
    assert_eq!(
        handed_back.0.as_deref(),
        Some("steps"),
        "the connect view handed back"
    );
    assert_eq!(handed_back.1, "Google Calendar connected");
    assert_eq!(
        handed_back.2, SENTENCE,
        "the sentence waited, nothing to retype"
    );
    assert_eq!(handed_back.3, "Do all 3");
    let (in_view, tasks, titles) = done;
    assert!(!in_view, "Do all finished and went home");
    assert!(
        titles
            .iter()
            .any(|t| t.starts_with("Did 3 steps: move standup")),
        "{titles:?}"
    );
    assert!(
        tasks >= 1,
        "steps nothing else can do go to an agent in the strip"
    );
    assert!(one_window, "no window opened");
}

#[test]
fn tab_hands_the_words_to_an_agent_and_esc_parks_it_in_the_strip() {
    let (opened, parked, list_kept, reopened, one_window) = in_launcher("agent_strip", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        let before = windows(cx);
        type_into_launcher(cx, window, &app, "fix SK-431");
        wait(cx, 20);
        let (taken, _) = app.update(cx, |app, cx| {
            let handled = app.one_window_key(Key::Tab, cx);
            (handled.taken(), ())
        });
        wait(cx, 20);
        let opened = (taken, top_view(&app, cx));
        key(&app, cx, Key::Escape);
        wait(cx, 20);
        let parked = app.update(cx, |app, _| {
            (
                app.one_window.in_view(),
                app.filter_text.clone(),
                app.one_window
                    .work()
                    .strip()
                    .tasks()
                    .iter()
                    .map(|t| (t.label.clone(), t.state))
                    .collect::<Vec<_>>(),
            )
        });
        type_into_launcher(cx, window, &app, "play");
        wait(cx, 20);
        let list = app.update(cx, |app, _| app.one_window.list().layout());
        wait(cx, 400);
        let list_kept = app.update(cx, |app, _| app.one_window.list().layout()) == list;
        type_into_launcher(cx, window, &app, "");
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::CmdJ);
        wait(cx, 20);
        let reopened = (top_view(&app, cx), enter_label(&app, cx));
        (opened, parked, list_kept, reopened, windows(cx) == before)
    });
    assert_eq!(opened, (true, Some("agent".into())));
    let (in_view, input, tasks) = parked;
    assert!(!in_view);
    assert_eq!(input, "", "the input is free again");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].0, "Agent · fix SK-431");
    assert!(
        list_kept,
        "the agent moving along never changes the list you're typing into"
    );
    assert_eq!(
        reopened.0.as_deref(),
        Some("agent"),
        "⌘J opens the agent's view again"
    );
    assert_eq!(
        reopened.1, "Looks good",
        "it finished while parked and waits for you"
    );
    assert!(one_window, "no window opened");
}
