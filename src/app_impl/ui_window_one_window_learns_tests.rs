//! The main window learns, keeps what ran, lists automations, hears the voice key and records
//! shortcuts, each as rows or a view under the same input, driven through the real
//! `ScriptListApp` and its one-window reducer.

use sk_one_window::contract::{talk_words, words};
use sk_one_window::{Done, Event, Key, Lesson, Lessons, Row};

use super::{in_launcher, key, launcher, type_into_launcher, wait, Jev};
use crate::list_item::GroupedListItem;
use crate::scripts::SearchResult;
use crate::ScriptListApp;

/// The first header and the reducer's own rows the list shows.
fn listed(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> (Option<String>, Vec<String>) {
    app.update(cx, |app, _| {
        let (grouped, flat) = app.main_menu_result_caches.clone_grouped_results();
        let header = match grouped.first() {
            Some(GroupedListItem::SectionHeader(label, _)) => Some(label.clone()),
            _ => None,
        };
        let rows = grouped
            .iter()
            .filter_map(|item| match item {
                GroupedListItem::Item(ix) => match flat.get(*ix) {
                    Some(SearchResult::OneWindow(m)) => Some(m.row.key.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        (header, rows)
    })
}

fn enter_label(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| app.one_window_enter_label())
}

#[test]
fn a_learned_sentence_is_listed_and_owns_enter_from_its_first_letters() {
    let (rows, label) = in_launcher("learned_row", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, _| {
            let mut row = Row::new(
                "jev:window:Move Chrome",
                "Move Google Chrome to Next Display",
            );
            row.icon = Some("window".into());
            let lessons = Lessons::new(vec![Lesson {
                said: "put the pr on my second screen".into(),
                row,
                at: 0,
            }]);
            let window = std::mem::take(&mut app.one_window);
            app.one_window = window.with_lessons(lessons);
            app.one_window_host.learns.lessons_loaded = true;
        });
        type_into_launcher(cx, window, &app, "put t");
        wait(cx, 20);
        (listed(&app, cx).1, enter_label(&app, cx))
    });
    assert_eq!(
        rows.first().map(String::as_str),
        Some("learned:Move Google Chrome to Next Display")
    );
    assert_eq!(label, "Move Google Chrome to Next Display");
}

#[test]
fn saying_list_my_automations_opens_them_under_the_input() {
    let (view, header) = in_launcher("automations_view", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "list my automations");
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let view = app.update(cx, |app, _| app.one_window_own_view().map(|v| v.name()));
        (view, listed(&app, cx).0)
    });
    assert_eq!(view, Some("automations"));
    assert_eq!(header.as_deref(), Some("Runs on its own"));
}

#[test]
fn share_my_last_action_lists_copy_link_and_send() {
    let (header, rows, line) = in_launcher("share_rows", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_handle(
                Event::Done(Done {
                    icon: "steps".into(),
                    title: "Did 2 steps: tell sarah · post in #launcher".into(),
                    ..Done::default()
                }),
                cx,
            );
        });
        wait(cx, sk_one_window::contract::ENTER_GUARD_MS);
        type_into_launcher(cx, window, &app, "share my last action");
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let (header, rows) = listed(&app, cx);
        let line = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock()).label
        });
        (header, rows, line)
    });
    assert_eq!(header.as_deref(), Some(words::SHARE_HOW));
    assert_eq!(rows.len(), 3, "copy, link, send: {rows:?}");
    assert!(rows[0].contains("Copy it to the clipboard"));
    assert!(rows[1].contains("Copy a link"));
    assert!(rows[2].contains("Send it to"));
    assert_eq!(line, words::SHARING);
}

#[test]
fn the_voice_view_lists_both_readings_and_asks_once_when_jev_cannot_tell() {
    let (listening, typing, close) = in_launcher("voice_view", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| app.one_window_talk_began(false, cx));
        wait(cx, 20);
        type_into_launcher(cx, window, &app, "hi sarah thanks for checking in");
        wait(cx, 20);
        let listening = listed(&app, cx);
        key(&app, cx, Key::CmdDigit(1));
        wait(cx, 20);
        let line = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock())
        });
        let typing = (listed(&app, cx).0, line.label, enter_label(&app, cx));
        app.update(cx, |app, cx| {
            let text = app.one_window.text().to_string();
            app.one_window_handle(Event::Text(format!("{text}!")), cx);
        });
        key(&app, cx, Key::CmdDigit(2));
        wait(cx, 20);
        key(&app, cx, Key::Escape);
        wait(cx, 20);
        app.update(cx, |app, cx| app.one_window_talk_began(false, cx));
        type_into_launcher(cx, window, &app, "maybe friday works for the review");
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, 500);
        (
            listening,
            typing,
            (listed(&app, cx).0, enter_label(&app, cx)),
        )
    });
    assert_eq!(listening.0.as_deref(), Some(talk_words::LISTENING));
    assert_eq!(listening.1, ["talk:type", "talk:ask"]);
    assert_eq!(typing.0.as_deref(), Some(talk_words::PICKED_TYPING));
    assert_eq!(typing.1, talk_words::TYPING_INTO);
    assert!(typing.2.starts_with("Type it into "), "{}", typing.2);
    assert_eq!(
        close.0.as_deref(),
        Some(talk_words::TOO_CLOSE),
        "Jev is off: letting go asks once"
    );
    assert_eq!(close.1, talk_words::PICK_ONE);
}

#[test]
fn giving_a_script_a_shortcut_records_the_keys_in_the_main_window() {
    let (view, label, saved_label, closed) = in_launcher("shortcut_view", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "give when is trash day a shortcut");
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let view = app.update(cx, |app, _| app.one_window_own_view().map(|v| v.name()));
        let label = enter_label(&app, cx);
        app.update(cx, |app, cx| {
            let keys = gpui::Keystroke::parse("cmd-shift-k").unwrap();
            assert!(app.one_window_shortcut_keystroke(&keys, cx));
        });
        let saved_label = enter_label(&app, cx);
        app.update(cx, |app, cx| {
            let esc = gpui::Keystroke::parse("escape").unwrap();
            app.one_window_shortcut_keystroke(&esc, cx);
        });
        let closed = app.update(cx, |app, _| {
            app.one_window_own_view().is_none() && app.shortcut_recorder_state.is_none()
        });
        (view, label, saved_label, closed)
    });
    assert_eq!(view, Some("shortcut"));
    assert_eq!(label, "Press the keys first");
    assert!(saved_label.starts_with("Save ⌘"), "{saved_label}");
    assert!(closed, "Esc backs out");
}
