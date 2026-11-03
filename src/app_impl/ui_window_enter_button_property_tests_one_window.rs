//! The one-window list through the real launcher: the empty input's list is the reducer's, ⌘K
//! stages in place and lists actions, and the ↵ capsule stays steady while a sentence is typed.

use super::*;

const SENTENCE: &str = "when does lebron james play next";

/// Milliseconds between keys: a steady typist, well inside a sentence.
const TYPING_GAP_MS: u64 = 120;

fn enter_capsule(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, cx| {
        app.main_window_footer_config_with_cx(Some(cx))
            .and_then(|footer| {
                footer
                    .buttons
                    .iter()
                    .find(|button| button.action == FooterAction::Run)
                    .map(|button| button.label.to_string())
            })
            .unwrap_or_default()
    })
}

#[test]
fn typing_a_sentence_keeps_the_enter_capsule_steady() {
    let (labels, reading) = in_launcher("enter_capsule_steady", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Reading);
        app.update(cx, |app, cx| {
            app.scripts.clear();
            app.filter_text.clear();
            app.flush_pending_main_menu_query(cx);
        });
        let mut labels = Vec::new();
        let mut text = String::new();
        for ch in SENTENCE.chars() {
            text.push(ch);
            type_into_launcher(cx, window, &app, &text);
            wait(cx, TYPING_GAP_MS);
            labels.push((text.clone(), enter_capsule(&app, cx)));
        }
        wait(cx, 1_000);
        labels.push(("(keys paused)".to_string(), enter_capsule(&app, cx)));
        let reading = app.update(cx, |app, _| {
            app.one_window.list().current_row().map(|row| row.title)
        });
        (labels, reading)
    });
    let transcript: Vec<String> = labels
        .iter()
        .map(|(text, label)| format!("{text:?} → {label}"))
        .collect();
    eprintln!("↵ after every key:\n{}", transcript.join("\n"));

    let mut shown: Vec<&str> = Vec::new();
    for (_, label) in &labels {
        if shown.last() != Some(&label.as_str()) {
            assert!(
                !shown.contains(&label.as_str()),
                "↵ went back to {label:?}:\n{}",
                transcript.join("\n")
            );
            shown.push(label);
        }
    }
    let jev_took = shown
        .iter()
        .position(|label| *label == words::ENTER_JEV_FIRST)
        .expect("no name owns the sentence past its first word");
    assert!(
        shown.len() - jev_took <= 2,
        "once Jev took Enter, ↵ showed {:?}",
        &shown[jev_took..]
    );
    let reading = reading.expect("Jev's reading owns Enter once the keys pause");
    assert_eq!(
        labels.last().map(|(_, label)| label.as_str()),
        Some(reading.as_str()),
        "the last label is Jev's reading"
    );
}

#[test]
fn cached_preflight_follows_the_capsule_when_a_reading_settles_on_the_same_row() {
    in_launcher("preflight_after_reading", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Reading);
        type_into_launcher(cx, window, &app, "zzqxframeproof");
        let observe = |cx: &mut gpui::TestAppContext| {
            let capsule = enter_capsule(&app, cx);
            app.update(cx, |app, _| {
                app.rebuild_main_window_preflight_if_needed();
                let receipt = app
                    .cached_main_window_preflight
                    .as_ref()
                    .expect("launcher receipt");
                let action = receipt.enter_action.as_ref().expect("selected action");
                assert_eq!(action.label, capsule);
                (receipt.selected_result_key.clone(), capsule)
            })
        };
        // First let the reading arrive; its own typing-pause timer still owns the label.
        wait(cx, 120);
        let (before_row, before_label) = observe(cx);
        assert_eq!(before_label, words::ENTER_JEV_FIRST);
        wait(cx, sk_one_window::contract::READING_SETTLE_MS);
        let (after_row, after_label) = observe(cx);
        assert!(before_row.is_some());
        assert_eq!(before_row, after_row);
        assert_ne!(before_label, after_label);
    });
}

fn first_header(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> Option<String> {
    app.update(cx, |app, _| {
        let (grouped, _) = app.main_menu_result_caches.clone_grouped_results();
        match grouped.first() {
            Some(GroupedListItem::SectionHeader(label, _)) => Some(label.clone()),
            _ => None,
        }
    })
}

#[test]
fn the_empty_input_lists_what_the_reducer_lays_out() {
    let (drawn, laid_out, rows) = in_launcher("empty_input_layout", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        wait(cx, 20);
        let drawn = first_header(&app, cx);
        app.update(cx, |app, _| {
            let list = app.one_window.list();
            let (grouped, _) = app.main_menu_result_caches.clone_grouped_results();
            let rows = grouped
                .iter()
                .filter(|item| matches!(item, GroupedListItem::Item(_)))
                .count();
            (drawn, list.header(), (rows, list.visible().len()))
        })
    });
    assert_eq!(
        drawn.as_deref(),
        Some(laid_out.as_str()),
        "the reducer's header leads"
    );
    assert!(rows.0 > 0, "the empty input is never an empty list");
    assert_eq!(
        rows.0, rows.1,
        "every row drawn is one the reducer laid out"
    );
}

#[test]
fn cmd_k_stages_the_row_and_lists_its_actions_in_place() {
    let (staged, header, rows, windows_before, windows_after, unstaged) =
        in_launcher("cmd_k_in_place", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            type_into_launcher(cx, window, &app, "play");
            wait(cx, 20);
            let windows_before = cx.update(|cx| cx.windows().len());
            app.update(cx, |app, cx| app.one_window_stage(cx));
            wait(cx, 20);
            let windows_after = cx.update(|cx| cx.windows().len());
            let staged = app.update(cx, |app, _| {
                let label = app
                    .one_window
                    .staged()
                    .map(|s| s.subject.label.clone())
                    .unwrap_or_default();
                let chip = app.one_window.chip().filter(|chip| chip.attached);
                (label, chip.map(|chip| chip.title).unwrap_or_default())
            });
            let header = first_header(&app, cx);
            let rows = app.update(cx, |app, _| {
                let list = app.one_window.list();
                list.visible()
                    .iter()
                    .filter_map(|item| list.row(item).map(|row| row.key.clone()))
                    .collect::<Vec<_>>()
            });
            key(&app, cx, Key::Escape);
            wait(cx, 20);
            let unstaged = app.update(cx, |app, _| app.one_window.staged().is_none());
            (
                staged,
                header,
                rows,
                windows_before,
                windows_after,
                unstaged,
            )
        });
    assert_eq!(staged.0, words::ACTING_ON);
    assert_eq!(
        staged.1, "Play Next Track",
        "the highlighted row is what's staged"
    );
    assert_eq!(header.as_deref(), Some(words::OR_PICK_ONE));
    assert!(
        !rows.is_empty()
            && rows.iter().all(|key| {
                key.starts_with(crate::one_window::staging::ACTION_KEY)
                    || key == crate::one_window::chat_entry::ABOUT_KEY
            }),
        "the list is the row's actions: {rows:?}"
    );
    assert_eq!(windows_after, windows_before, "no popup window opened");
    assert!(unstaged, "Esc lets go");
}

#[test]
fn esc_after_cmd_k_highlights_the_reducers_row() {
    let (highlighted, sel) = in_launcher("cmd_k_esc_highlight", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "play");
        wait(cx, 20);
        app.update(cx, |app, cx| app.one_window_stage(cx));
        wait(cx, 20);
        key(&app, cx, Key::Escape);
        wait(cx, 20);
        app.update(cx, |app, _| {
            let highlighted = app
                .main_menu_committed_rows()
                .iter()
                .find(|row| row.grouped_index == app.selected_index)
                .and_then(|row| row.selectable_ordinal);
            (highlighted, app.one_window.list().sel)
        })
    });
    assert_eq!(
        highlighted,
        Some(sel),
        "after Esc the list highlights the row Enter runs, not nothing"
    );
}

#[test]
fn space_on_the_front_window_lists_what_can_be_done_to_that_window() {
    let rows = in_launcher("space_front_window", 0, |cx| {
        let (app, _window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_handle(
                sk_one_window::Event::Focused(sk_one_window::Subject {
                    kind: sk_one_window::SubjectKind::Window,
                    label: "Front window".into(),
                    title: "~/dev · zsh".into(),
                    sub: "Ghostty".into(),
                    ..Default::default()
                }),
                cx,
            );
        });
        wait(cx, 20);
        key(&app, cx, Key::Space);
        wait(cx, 20);
        app.update(cx, |app, _| {
            let list = app.one_window.list();
            list.visible()
                .iter()
                .filter_map(|item| list.row(item).map(|row| row.title.clone()))
                .collect::<Vec<_>>()
        })
    });
    assert_eq!(
        rows.first().map(String::as_str),
        Some("Ask the model about it"),
        "{rows:?}"
    );
    assert_eq!(
        rows.get(1).map(String::as_str),
        Some("Tile Ghostty Left Half"),
        "{rows:?}"
    );
    assert!(
        rows.iter().skip(1).all(|title| title.contains("Ghostty")),
        "every action names the window's app, none are the launcher's own: {rows:?}"
    );
}

#[test]
fn typing_give_a_script_a_shortcut_ends_on_record_a_shortcut() {
    let label = in_launcher("shortcut_sentence", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, _| {
            app.scripts.push(fixture_command("Launch Alpha", "Launch"));
        });
        let mut text = String::new();
        for ch in "give launch alpha a shortcut".chars() {
            text.push(ch);
            type_into_launcher(cx, window, &app, &text);
            wait(cx, TYPING_GAP_MS);
        }
        wait(cx, 1_000);
        enter_capsule(&app, cx)
    });
    assert!(
        label.contains("Record a shortcut"),
        "the shortcut row owns ↵ at the end of the sentence, got {label:?}"
    );
}
