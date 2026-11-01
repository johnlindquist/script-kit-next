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

/// Presses a key the way the physical key reaches the launcher: the test window's root is not
/// the launcher, so a window keystroke reaches no key handler, and simulateKey takes the same
/// routes the key interceptors do.
fn simulate_key(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    app: &gpui::Entity<ScriptListApp>,
    key: &str,
) -> bool {
    cx.update(|cx| {
        window
            .update(cx, |_, window, cx| {
                app.update(cx, |app, cx| {
                    app.dispatch_simulate_key(
                        window,
                        cx,
                        crate::simulate_key_dispatch::SimulatedKeyInput {
                            key,
                            modifiers: &[],
                            target: None,
                        },
                    )
                })
            })
            .expect("press a key in the test launcher")
            .status
            == crate::action_helpers::ActionOutcomeStatus::Success
    })
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
fn unsupported_offline_steps_never_connect_a_calendar_or_report_partial_success() {
    const SENTENCE: &str = "add to today's note: blue kite and tell Mo the bell rang";
    in_launcher("steps_offline_no_effect", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Failing);
        let before = windows(cx);
        type_into_launcher(cx, window, &app, SENTENCE);
        wait(cx, 1_400);
        assert_eq!(enter_label(&app, cx), "Unavailable");
        key(&app, cx, Key::Enter);
        wait(cx, 700);
        assert_eq!(windows(cx), before);
        assert!(
            top_view(&app, cx).is_none(),
            "no pretend sign-in or agent handoff"
        );
        assert!(slot_title(&app, cx).starts_with("Can't run these steps offline."));
        app.update(cx, |app, _| {
            assert!(!app.one_window.work().connections().has("calendar"));
            assert!(app.one_window.work().strip().tasks().is_empty());
            assert!(!app
                .one_window
                .ledger()
                .entries()
                .iter()
                .any(|entry| entry.title.starts_with("Did ")));
        });
    });
}

#[test]
fn tab_makes_the_agent_the_owner_enter_starts_it_and_esc_parks_it_in_the_strip() {
    let (owner, opened, parked, list_kept, reopened, one_window) =
        in_launcher("agent_strip", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            let before = windows(cx);
            type_into_launcher(cx, window, &app, "fix SK-431");
            wait(cx, 20);
            let form_takes_tab = cx.update(|cx| {
                window
                    .update(cx, |_, window, cx| {
                        app.update(cx, |app, cx| {
                            app.handle_menu_syntax_form_key_input(
                                "tab",
                                Some("\t"),
                                &gpui::Modifiers::default(),
                                window,
                                cx,
                            )
                        })
                    })
                    .expect("offer Tab to the capture form")
            });
            let taken = simulate_key(cx, window, &app, "tab");
            wait(cx, 20);
            let owns = app.update(cx, |app, _| app.one_window.agent_owns());
            let owner = (
                !form_takes_tab && taken,
                owns,
                top_view(&app, cx),
                enter_label(&app, cx),
            );
            key(&app, cx, Key::Enter);
            wait(cx, 20);
            let opened = top_view(&app, cx);
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
            (
                owner,
                opened,
                parked,
                list_kept,
                reopened,
                windows(cx) == before,
            )
        });
    assert_eq!(
        owner,
        (true, true, None, sk_one_window::contract::words::START_AGENT.to_string()),
        "no capture form takes ⇥, which makes the agent the Owner and starts nothing; ↵ says it will start it"
    );
    assert_eq!(opened, Some("agent".into()), "↵ starts the agent in place");
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

#[test]
fn esc_leaves_a_view_opened_over_a_symbol_picker_with_every_typed_word() {
    let seen = in_launcher("view_over_picker", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        let mut seen = Vec::new();
        for typed in ["summarize this @clipboard", ";"] {
            type_into_launcher(cx, window, &app, "summarize this");
            wait(cx, 20);
            type_into_launcher(cx, window, &app, typed);
            wait(cx, 20);
            let picker_owns = app.update(cx, |app, _| !app.one_window_owns_list());
            app.update(cx, |app, cx| {
                app.one_window_open_clipboard_answer("what did I copy".into(), cx)
            });
            wait(cx, 20);
            let opened = top_view(&app, cx);
            let escaped = simulate_key(cx, window, &app, "escape");
            wait(cx, 20);
            let input = app.update(cx, |app, _| app.filter_text.clone());
            seen.push((
                typed,
                picker_owns,
                opened,
                escaped,
                top_view(&app, cx),
                input,
            ));
        }
        seen
    });
    for (typed, picker_owns, opened, escaped, after, input) in seen {
        assert!(picker_owns, "{typed:?}: a symbol picker owns the list");
        assert!(opened.is_some(), "{typed:?}: the answer opens as a view");
        assert!(escaped, "{typed:?}: Esc is handled");
        assert_eq!(
            after, None,
            "{typed:?}: Esc leaves the view, not the picker under it"
        );
        assert_eq!(
            input, typed,
            "{typed:?}: every typed word is still in the input"
        );
    }
}

#[test]
fn a_typed_capture_is_a_row_under_the_capture_header() {
    let seen = in_launcher("typed_capture", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        ["todo;", "todo; buy milk due:friday", ";todo buy milk"].map(|typed| {
            type_into_launcher(cx, window, &app, typed);
            wait(cx, 20);
            let (owns, header, first) = app.update(cx, |app, _| {
                let list = app.one_window.list();
                let first = list
                    .visible()
                    .first()
                    .and_then(|item| list.row(item))
                    .map(|row| row.title.clone());
                (app.one_window_owns_list(), list.header(), first)
            });
            (typed, owns, header, first, enter_label(&app, cx))
        })
    });
    let want = [
        ("Add todo: …", "Type what to write down"),
        ("Add todo: buy milk (friday)", "Add todo"),
        ("Add todo: buy milk", "Add todo"),
    ];
    for ((typed, owns, header, first, enter), (title, label)) in seen.into_iter().zip(want) {
        assert!(owns, "{typed:?}: the one-window list owns a typed capture");
        assert_eq!(
            header, "Capture",
            "{typed:?}: the `;` picker's header stays"
        );
        assert_eq!(
            first.as_deref(),
            Some(title),
            "{typed:?}: the capture leads"
        );
        assert_eq!(enter, label, "{typed:?}: ↵ says what it does");
    }
}
