//! The main window learns, keeps what ran, lists automations, hears the voice key, lists the
//! microphones and what you said, and records shortcuts, each as rows or a view under the same
//! input, driven through the real `ScriptListApp` and its one-window reducer.

use sk_one_window::contract::{talk_words, words};
use sk_one_window::{Dest, Done, Event, Key, Lesson, Lessons, Row};

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
                    Some(SearchResult::DictationHistory(m)) => Some(format!("history:{}", m.id)),
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

fn top_view(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Option<String> {
    app.update(cx, |app, _| {
        app.one_window.views().top().map(|top| top.name.clone())
    })
}

/// A private kit folder for one test: dictation history and today's note live here, and the
/// real one is never read or written.
struct KitDir {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: Option<String>,
    dir: tempfile::TempDir,
}

impl KitDir {
    fn new() -> Self {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let dir = tempfile::tempdir().expect("kit folder");
        let previous = std::env::var(crate::setup::SK_PATH_ENV).ok();
        std::env::set_var(crate::setup::SK_PATH_ENV, dir.path());
        Self {
            _lock: lock,
            previous,
            dir,
        }
    }

    /// Saves the dictations oldest first, the way dictation appends them.
    fn said(&self, transcripts: &[&str]) -> Vec<crate::dictation::DictationHistoryEntry> {
        let entries: Vec<_> = transcripts
            .iter()
            .map(|said| {
                crate::dictation::build_history_entry(
                    said,
                    std::time::Duration::from_secs(2),
                    crate::dictation::DictationTarget::MainWindowFilter,
                )
            })
            .collect();
        let lines: String = entries
            .iter()
            .map(|entry| format!("{}\n", serde_json::to_string(entry).expect("entry json")))
            .collect();
        std::fs::write(self.dir.path().join("dictation-history.jsonl"), lines)
            .expect("history file");
        entries
    }
}

impl Drop for KitDir {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => std::env::set_var(crate::setup::SK_PATH_ENV, previous),
            None => std::env::remove_var(crate::setup::SK_PATH_ENV),
        }
    }
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
            app.one_window
                .staged()
                .map(|s| s.subject.label.clone())
                .unwrap_or_default()
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
        app.update(cx, |app, cx| app.one_window_talk_began(false, None, cx));
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
        app.update(cx, |app, cx| app.one_window_talk_began(false, None, cx));
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

#[test]
fn dictation_history_lists_what_you_said_newest_first_and_cmd_k_stages_the_row() {
    let (ids, header, rows, staged) = in_launcher("dictation_history_rows", 0, |cx| {
        let kit = KitDir::new();
        let entries = kit.said(&["pick up the dry cleaning", "email sarah about friday"]);
        let (app, _window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| app.one_window_open_dictation_history(cx));
        wait(cx, 20);
        let (header, rows) = listed(&app, cx);
        key(&app, cx, Key::CmdK);
        wait(cx, 20);
        let staged = app.update(cx, |app, _| {
            app.one_window.staged().map(|staged| {
                let actions: Vec<String> =
                    staged.rows.iter().map(|row| row.title.clone()).collect();
                (staged.subject.title.clone(), actions)
            })
        });
        let ids: Vec<String> = entries
            .iter()
            .map(|e| format!("history:{}", e.id))
            .collect();
        drop(kit);
        (ids, header, rows, staged)
    });
    assert_eq!(header.as_deref(), Some("What you said"));
    assert_eq!(rows, [ids[1].clone(), ids[0].clone()], "newest first");
    let (title, actions) = staged.expect("⌘K stages the highlighted dictation");
    assert_eq!(title, "email sarah about friday");
    assert!(
        actions.iter().any(|a| a == "Paste Dictation")
            && actions.iter().any(|a| a == "Add to Agent Chat"),
        "the staged row lists its actions: {actions:?}"
    );
}

#[test]
fn the_microphone_view_checks_the_one_dictation_listens_to() {
    let device = |id: &str, name: &str, is_default: bool| crate::dictation::DictationDeviceInfo {
        id: crate::dictation::DictationDeviceId(id.to_string()),
        name: name.to_string(),
        is_default,
        transport: crate::dictation::DictationDeviceTransport::Usb,
    };
    let items = crate::dictation::build_device_menu_items(
        &[
            device("built-in", "MacBook Pro Microphone", true),
            device("usb-1", "Podcast Mic", false),
        ],
        Some("usb-1"),
    );
    let rows = crate::one_window::voice_lists::microphone_rows(items);
    let keys: Vec<&str> = rows.iter().map(|row| row.key.as_str()).collect();
    assert_eq!(keys.len(), 3, "System Default, then each input: {keys:?}");
    assert!(keys.iter().all(|key| key.starts_with("mic:")));
    let listening: Vec<&Row> = rows
        .iter()
        .filter(|row| row.icon.as_deref() == Some("check"))
        .collect();
    assert_eq!(listening.len(), 1, "exactly one row is checked");
    assert_eq!(listening[0].title, "Podcast Mic");
    assert!(listening[0]
        .sub
        .as_deref()
        .unwrap_or("")
        .starts_with("Listening · "));

    let view = in_launcher("microphone_rows", 0, |cx| {
        let (app, _window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| app.one_window_open_microphones(cx));
        wait(cx, 20);
        app.update(cx, |app, _| app.one_window_own_view().map(|v| v.name()))
    });
    assert_eq!(
        view,
        Some("microphones"),
        "Select Microphone opens the list view"
    );
}

#[test]
fn dictation_to_agent_chat_opens_the_chat_view_with_the_words_as_its_first_message() {
    let (listening, opened, reply) = in_launcher("dictation_to_chat", 0, |cx| {
        let (app, _window) = launcher(cx, Jev::Off);
        let before = cx.update(|cx| cx.windows().len());
        app.update(cx, |app, cx| {
            app.one_window_talk_began(false, Some(Dest::Chat), cx)
        });
        wait(cx, 20);
        let listening = listed(&app, cx);
        app.update(cx, |app, cx| {
            app.one_window_talk_final("what changed in the launcher this week".into(), None, cx)
        });
        wait(cx, sk_one_window::contract::ENTER_GUARD_MS + 50);
        let opened = (
            top_view(&app, cx),
            cx.update(|cx| cx.windows().len()) == before,
        );
        let reply = app.update(cx, |app, _| {
            app.one_window
                .chat()
                .and_then(|chat| chat.reply().map(str::to_string))
        });
        (listening, opened, reply)
    });
    assert_eq!(listening.0.as_deref(), Some(talk_words::talking_to(true)));
    assert_eq!(listening.1.len(), 1, "one row: {:?}", listening.1);
    assert!(listening.1[0].ends_with(":chat"), "{:?}", listening.1);
    assert_eq!(
        opened,
        (Some("chat".into()), true),
        "the chat is a view, not a window"
    );
    assert_eq!(reply.as_deref(), Some("Fixture Agent Chat response."));
}

#[test]
fn dictation_to_notes_adds_the_words_to_todays_note_and_says_so() {
    let (listening, line, today) = in_launcher("dictation_to_note", 0, |cx| {
        let kit = KitDir::new();
        let (app, _window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_talk_began(false, Some(Dest::Note), cx)
        });
        wait(cx, 20);
        let listening = listed(&app, cx);
        app.update(cx, |app, cx| {
            app.one_window_talk_final("pick up milk on the way home".into(), None, cx)
        });
        wait(cx, 20);
        let line = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock()).title
        });
        let brain = crate::brain::substrate::BrainSubstrate::default_kit();
        let (date, _) = brain.capture_day_destination(chrono::Utc::now());
        let today = std::fs::read_to_string(brain.paths().day_page(date)).unwrap_or_default();
        drop(kit);
        (listening, line, today)
    });
    assert_eq!(listening.0.as_deref(), Some(talk_words::talking_to(false)));
    assert!(listening.1[0].ends_with(":note"), "{:?}", listening.1);
    assert_eq!(
        line,
        "Added to today's note: “pick up milk on the way home”"
    );
    assert!(today.contains("pick up milk on the way home"), "{today}");
}

#[test]
fn select_microphone_from_the_launcher_opens_the_microphone_list() {
    let (label, handled, view) = in_launcher("select_microphone_route", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "select microphone");
        wait(cx, sk_one_window::contract::ENTER_GUARD_MS + 20);
        let label = enter_label(&app, cx);
        let handled = key(&app, cx, Key::Enter);
        wait(cx, 20);
        let view = app.update(cx, |app, _| app.one_window_own_view().map(|v| v.name()));
        (label, handled, view)
    });
    assert_eq!(label, "Select Microphone");
    assert_eq!(view, Some("microphones"), "Enter ran: {handled:?}");
}

#[test]
fn saying_dictation_history_opens_what_you_said_rather_than_talking() {
    let (label, view) = in_launcher("dictation_history_words", 0, |cx| {
        let kit = KitDir::new();
        kit.said(&["email sarah about friday"]);
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "dictation history");
        wait(cx, sk_one_window::contract::ENTER_GUARD_MS + 20);
        let label = enter_label(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let view = app.update(cx, |app, _| app.one_window_own_view().map(|v| v.name()));
        drop(kit);
        (label, view)
    });
    assert!(
        label.contains("Dictation History"),
        "↵ names the history, not talking: {label}"
    );
    assert_eq!(view, Some("dictation-history"));
}

/// The titles of the reducer's own rows the list shows.
fn own_titles(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Vec<String> {
    app.update(cx, |app, _| {
        let (_, flat) = app.main_menu_result_caches.clone_grouped_results();
        flat.iter()
            .filter_map(|result| match result {
                SearchResult::OneWindow(m) => Some(m.row.title.clone()),
                _ => None,
            })
            .collect()
    })
}

fn press(cx: &mut gpui::TestAppContext, window: gpui::AnyWindowHandle, keys: &str) {
    cx.dispatch_keystroke(window, gpui::Keystroke::parse(keys).expect("keystroke"));
    wait(cx, 20);
}

#[test]
fn a_scripts_hotkey_prompt_records_under_the_input_and_answers_with_the_same_json() {
    struct Seen {
        view: Option<String>,
        header: Option<String>,
        titles: Vec<String>,
        label: String,
    }
    let seen = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext| Seen {
        view: top_view(app, cx),
        header: listed(app, cx).0,
        titles: own_titles(app, cx),
        label: enter_label(app, cx),
    };
    let (opened, partial, complete, sent, after, elements, cancelled) =
        in_launcher("hotkey_prompt", 0, move |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            let (sender, receiver) = std::sync::mpsc::sync_channel(8);
            let ask = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext, id| {
                let sender = sender.clone();
                app.update(cx, |app, cx| {
                    app.response_sender = Some(sender);
                    app.handle_prompt_message(
                        crate::PromptMessage::ShowHotkey {
                            id,
                            placeholder: None,
                        },
                        cx,
                    );
                });
                wait(cx, 20);
            };
            ask(&app, cx, "hotkey-1".to_string());
            let opened = seen(&app, cx);
            let elements: Vec<String> = app.update(cx, |app, cx| {
                app.collect_visible_elements(100, cx)
                    .elements
                    .into_iter()
                    .map(|element| element.semantic_id)
                    .collect()
            });
            press(cx, window, "k");
            let partial = seen(&app, cx);
            press(cx, window, "cmd-shift-k");
            let complete = seen(&app, cx);
            press(cx, window, "enter");
            let sent: Vec<String> = receiver
                .try_iter()
                .map(|message| serde_json::to_string(&message).expect("message json"))
                .collect();
            let after = (
                top_view(&app, cx),
                app.update(cx, |app, _| {
                    matches!(app.current_view, crate::AppView::ScriptList)
                }),
                own_titles(&app, cx),
            );
            ask(&app, cx, "hotkey-2".to_string());
            press(cx, window, "escape");
            let cancelled: Vec<String> = receiver
                .try_iter()
                .map(|message| serde_json::to_string(&message).expect("message json"))
                .collect();
            (opened, partial, complete, sent, after, elements, cancelled)
        });

    assert_eq!(
        opened.view.as_deref(),
        Some("shortcut"),
        "a view under the input"
    );
    assert_eq!(opened.header.as_deref(), Some("Press a shortcut"));
    assert_eq!(opened.titles, ["Waiting for keys…"]);
    assert_eq!(opened.label, "Press the keys first");
    assert!(
        !elements.iter().any(|id| {
            id.contains("shortcut-recorder")
                || id.contains("shortcut-modal")
                || id == "hotkey-capture"
        }),
        "no recorder card: {elements:?}"
    );
    assert!(
        elements
            .iter()
            .any(|id| id == crate::one_window::slot_line::SLOT_LINE_ID),
        "the line above the input stays: {elements:?}"
    );

    assert_eq!(partial.titles, ["K"], "one key shows as its keycap");
    assert_eq!(partial.label, "Press the keys first");
    assert_eq!(complete.titles, ["⌘⇧K"]);
    assert_eq!(complete.label, "Use ⌘⇧K");

    assert_eq!(
        sent,
        [
            r#"{"type":"submit","id":"hotkey-1","value":"{\"key\":\"K\",\"command\":true,\"shift\":true,\"option\":false,\"control\":false,\"shortcut\":\"⌘⇧K\",\"keyCode\":\"K\"}"}"#
        ],
        "↵ answers the script with the HotkeyInfo it always got"
    );
    assert_eq!(
        after,
        (None, true, Vec::<String>::new()),
        "the view and its rows leave once answered"
    );
    assert_eq!(
        cancelled,
        [r#"{"type":"submit","id":"hotkey-2","value":null}"#],
        "Esc cancels the prompt"
    );
}

#[test]
fn a_successful_tab_ai_run_keeps_its_script_and_cmd_z_deletes_it() {
    let (saved, line, undone, left) = in_launcher("tab_ai_saved", 0, |cx| {
        let kit = KitDir::new();
        let scripts = kit.dir.path().join("plugins/main/scripts");
        let listed_scripts = || -> Vec<String> {
            std::fs::read_dir(&scripts)
                .map(|dir| {
                    dir.filter_map(Result::ok)
                        .map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .filter(|name| name.ends_with(".ts"))
                        .collect()
                })
                .unwrap_or_default()
        };
        let (app, window) = launcher(cx, Jev::Off);
        let ran = kit.dir.path().join("tab-ai-run.ts");
        std::fs::write(&ran, "await div(\"ran\");\n").expect("temp run script");
        let record = crate::ai::TabAiExecutionRecord::from_parts(
            "Review the launch checklist".into(),
            "// Name: Launch Review\nimport \"@scriptkit/sdk\";\nawait div(\"Launch review\");\n"
                .into(),
            ran.to_string_lossy().into_owned(),
            "launch-review".into(),
            "ScriptList".into(),
            None,
            "test-model".into(),
            "test-provider".into(),
            0,
            "2026-09-27T00:00:00Z".into(),
        );
        app.update(cx, |app, cx| {
            app.pending_tab_ai_execution = Some(record);
            app.complete_tab_ai_execution(true, None, cx);
        });
        let saved = listed_scripts();
        wait(cx, sk_one_window::contract::ENTER_GUARD_MS + 20);
        let line = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock())
        });
        press(cx, window, "cmd-z");
        let undone = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock()).title
        });
        let left = listed_scripts();
        drop(kit);
        (saved, line, undone, left)
    });
    assert_eq!(saved.len(), 1, "the script is written at once: {saved:?}");
    assert_eq!(line.title, format!("Saved as {}", saved[0]));
    assert!(
        line.hints.iter().any(|(key, _)| key == "⌘Z"),
        "the line offers ⌘Z: {:?}",
        line.hints
    );
    assert_eq!(undone, format!("Undid · Saved as {}", saved[0]));
    assert!(left.is_empty(), "⌘Z deleted the file it wrote: {left:?}");
}
