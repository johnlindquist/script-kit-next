//! Every way into the model's chat opens the one chat under the main input: what goes with it
//! waits in the line above the input and is sent with the first turn, past chats open again from
//! their rows, and the model's questions are rows answered with Enter. No other window opens.

use super::*;
use crate::{PromptMessage, ScriptErrorAgentChatContext};

fn top_view(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Option<String> {
    app.update(cx, |app, _| {
        app.one_window.views().top().map(|top| top.name.clone())
    })
}

fn enter_label(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| app.one_window_enter_label())
}

fn windows(cx: &mut gpui::TestAppContext) -> usize {
    cx.update(|cx| cx.windows().len())
}

fn input(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| app.filter_text.clone())
}

/// How many things wait in the line above the input.
fn staged(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Option<usize> {
    app.update(cx, |app, _| app.one_window.chat().map(|c| c.with.len()))
}

/// The chat's turns as (who, words, what went with them).
fn turns(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> Vec<(sk_one_window::chat::Who, String, Option<String>)> {
    app.update(cx, |app, _| {
        app.one_window
            .chat()
            .map(|c| {
                c.turns
                    .iter()
                    .map(|t| (t.who, t.text.clone(), t.sub.clone()))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// What the model was sent: each user message and how many parts went with it.
fn sent(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Vec<(String, usize)> {
    app.update(cx, |app, cx| {
        app.one_window_chat_thread()
            .map(|thread| {
                thread
                    .read(cx)
                    .messages
                    .iter()
                    .filter(|m| {
                        matches!(
                            m.role,
                            crate::ai::agent_chat::ui::thread::AgentChatThreadMessageRole::User
                        )
                    })
                    .map(|m| (m.body.to_string(), m.attachments.len()))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The title of the first row the main list paints.
fn painted_first_row(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> Option<String> {
    app.update(cx, |app, _| {
        let (grouped, flat) = app.main_menu_result_caches.clone_grouped_results();
        grouped.iter().find_map(|item| match item {
            GroupedListItem::Item(ix) => flat.get(*ix).map(|result| result.name().to_string()),
            _ => None,
        })
    })
}

/// Runs `body` with the kit folder in a fresh temporary directory, so the chat's files never
/// touch the real one.
fn in_temp_kit<T>(body: impl FnOnce(&std::path::Path) -> T) -> T {
    let _lock = crate::test_utils::SK_PATH_TEST_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let kit = tempfile::tempdir().expect("temporary kit folder");
    let previous = std::env::var(crate::setup::SK_PATH_ENV).ok();
    std::env::set_var(crate::setup::SK_PATH_ENV, kit.path());
    crate::ai::agent_chat::ui::history::invalidate_history_cache();
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(kit.path())));
    match previous {
        Some(path) => std::env::set_var(crate::setup::SK_PATH_ENV, path),
        None => std::env::remove_var(crate::setup::SK_PATH_ENV),
    }
    crate::ai::agent_chat::ui::history::invalidate_history_cache();
    out.unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// A fresh launcher always highlights a real row, so a quick question must not take it along
/// unasked; only a row the user chose (⌘K) goes with the chat.
#[test]
fn a_quick_question_takes_nothing_along_from_the_launcher() {
    let (highlighted, view, with) = in_launcher("quick_question_clean", 0, |cx| {
        let (app, _window) = launcher(cx, Jev::Off);
        let highlighted = app.update(cx, |app, _| app.one_window.list().current().is_some());
        key(&app, cx, Key::CmdEnter);
        wait(cx, 20);
        (highlighted, top_view(&app, cx), staged(&app, cx))
    });
    assert!(highlighted, "the launcher highlights its first row");
    assert_eq!(view.as_deref(), Some("chat"));
    assert_eq!(with, Some(0), "nothing waits above the input");
}

#[test]
fn asking_about_the_highlighted_row_stages_it_and_sends_it_with_the_first_turn() {
    let (label, opened, input_after, sent, first) = in_launcher("chat_about_row", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        let before = windows(cx);
        type_into_launcher(cx, window, &app, "play");
        wait(cx, 20);
        key(&app, cx, Key::CmdK);
        wait(cx, ENTER_GUARD_MS);
        let leading = (enter_label(&app, cx), painted_first_row(&app, cx));
        key(&app, cx, Key::Down);
        wait(cx, ENTER_GUARD_MS);
        let label = (leading, enter_label(&app, cx));
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let opened = (top_view(&app, cx), staged(&app, cx), windows(cx) == before);
        type_into_launcher(cx, window, &app, "what does it do");
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, ENTER_GUARD_MS + 50);
        (
            label,
            opened,
            input(&app, cx),
            sent(&app, cx),
            turns(&app, cx).into_iter().next(),
        )
    });
    let ((leading, painted), about) = label;
    assert_eq!(
        Some(leading.as_str()),
        painted.as_deref(),
        "the painted list starts with the row the Enter button names"
    );
    assert_ne!(
        leading, "Ask the model about it",
        "⌘K then Enter does what the row does"
    );
    assert_eq!(about, "Ask the model about it", "asking is one ↓ away");
    assert_eq!(
        opened,
        (Some("chat".into()), Some(1), true),
        "the row waits above the chat's input, in the same window"
    );
    assert_eq!(input_after, "");
    assert_eq!(sent, vec![("what does it do".to_string(), 1)]);
    let (_, words, with) = first.expect("the first turn");
    assert_eq!(words, "what does it do");
    assert!(
        with.as_deref().is_some_and(|w| w.starts_with("With ")),
        "the turn says what went with it: {with:?}"
    );
}

#[test]
fn a_failed_script_opens_the_chat_with_its_report_staged_and_sent_with_the_first_turn() {
    let (opened, question, sent) = in_temp_kit(|kit| {
        let script = kit.join("deploy.ts");
        std::fs::write(&script, "throw new Error('boom')\n").expect("write the failing script");
        let script = script.to_string_lossy().into_owned();
        in_launcher("chat_from_script_error", 0, move |cx| {
            let (app, _window) = launcher(cx, Jev::Off);
            let before = windows(cx);
            app.update(cx, |app, cx| {
                app.route_script_error_to_agent_chat(
                    ScriptErrorAgentChatContext {
                        script_path: &script,
                        error_message: "boom",
                        stderr_output: Some("Error: boom"),
                        exit_code: Some(1),
                        stack_trace: None,
                        suggestions: &[],
                    },
                    cx,
                );
            });
            wait(cx, 20);
            let opened = (top_view(&app, cx), staged(&app, cx), windows(cx) == before);
            let question = input(&app, cx);
            wait(cx, ENTER_GUARD_MS);
            key(&app, cx, Key::Enter);
            wait(cx, ENTER_GUARD_MS + 50);
            (opened, question, sent(&app, cx))
        })
    });
    assert_eq!(
        opened,
        (Some("chat".into()), Some(2), true),
        "the error report and the script wait above the input"
    );
    assert_eq!(
        question,
        "Why did deploy.ts fail? Fix it and check that it runs"
    );
    assert_eq!(sent, vec![(question, 2)], "both go with the first turn");
}

#[test]
fn a_handoff_with_words_sends_them_as_the_chats_first_turn() {
    let (view, words, reply, one_window) = in_launcher("chat_handoff_words", 0, |cx| {
        let (app, _window) = launcher(cx, Jev::Off);
        let before = windows(cx);
        app.update(cx, |app, cx| {
            app.open_chat_about_current_view(Some("summarize the launch notes".into()), cx)
        });
        wait(cx, ENTER_GUARD_MS);
        let reply = app.update(cx, |app, _| {
            app.one_window
                .chat()
                .and_then(|c| c.reply().map(str::to_string))
        });
        (
            top_view(&app, cx),
            sent(&app, cx),
            reply,
            windows(cx) == before,
        )
    });
    assert_eq!(view.as_deref(), Some("chat"));
    assert_eq!(words, vec![("summarize the launch notes".to_string(), 0)]);
    assert_eq!(reply.as_deref(), Some("Fixture Agent Chat response."));
    assert!(one_window);
}

#[test]
fn an_sdk_chat_opens_under_the_input_with_its_words_and_parts() {
    let (view, words, with, sent_before) = in_launcher("chat_from_sdk", 0, |cx| {
        let (app, _window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.handle_prompt_message(
                PromptMessage::AiStartChat {
                    request_id: "sdk-1".into(),
                    message: "tidy this list".into(),
                    system_prompt: None,
                    image: None,
                    model_id: None,
                    no_response: true,
                    parts: vec![crate::protocol::AiContextPartInput::TextBlock {
                        label: "Groceries".into(),
                        source: "sdk".into(),
                        text: "eggs, milk".into(),
                        mime_type: None,
                    }],
                },
                cx,
            );
        });
        wait(cx, 20);
        (
            top_view(&app, cx),
            input(&app, cx),
            staged(&app, cx),
            sent(&app, cx),
        )
    });
    assert_eq!(view.as_deref(), Some("chat"));
    assert_eq!(
        words, "tidy this list",
        "noResponse leaves the words in the input"
    );
    assert_eq!(with, Some(1));
    assert!(sent_before.is_empty(), "nothing is sent until Enter");
}

#[test]
fn a_past_chat_row_opens_that_chat_again_with_its_turns() {
    let (header, rows, view, turns) = in_temp_kit(|_| {
        let conversation = crate::ai::agent_chat::ui::history::SavedConversation {
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            custom_title: None,
            messages: vec![
                crate::ai::agent_chat::ui::history::SavedMessage {
                    role: "User".into(),
                    body: "plan the launch".into(),
                },
                crate::ai::agent_chat::ui::history::SavedMessage {
                    role: "Assistant".into(),
                    body: "Tuesday, with a demo.".into(),
                },
            ],
        };
        let entry = crate::ai::agent_chat::ui::history::build_history_entry(&conversation)
            .expect("a history entry");
        crate::ai::agent_chat::ui::history::save_completed_conversation(&conversation, &entry)
            .expect("save the past chat");
        in_launcher("past_chat_reopens", 0, |cx| {
            let (app, _window) = launcher(cx, Jev::Off);
            app.update(cx, |app, cx| {
                app.one_window_open_past_chats(String::new(), cx)
            });
            wait(cx, 20);
            let header = app.update(cx, |app, _| {
                app.one_window
                    .past_chats()
                    .map(|p| (p.head().to_string(), p.chats.len()))
            });
            let rows = top_view(&app, cx);
            wait(cx, ENTER_GUARD_MS);
            key(&app, cx, Key::Enter);
            wait(cx, 50);
            (header, rows, top_view(&app, cx), turns(&app, cx))
        })
    });
    assert_eq!(header, Some(("Past chats".to_string(), 1)));
    assert_eq!(rows.as_deref(), Some("past-chats"));
    assert_eq!(view.as_deref(), Some("chat"), "Enter opens the chat again");
    let words: Vec<_> = turns.into_iter().map(|(_, text, _)| text).collect();
    assert_eq!(words, vec!["plan the launch", "Tuesday, with a demo."]);
}

/// Deleting a past chat can't be taken back, so it waits in the line and ⌘Z on the window
/// stops it before anything is removed.
#[test]
fn deleting_a_past_chat_waits_in_the_line_and_cmd_z_stops_it() {
    let (label, waiting, stopped, kept) = in_temp_kit(|_| {
        let conversation = crate::ai::agent_chat::ui::history::SavedConversation {
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            custom_title: None,
            messages: vec![
                crate::ai::agent_chat::ui::history::SavedMessage {
                    role: "User".into(),
                    body: "plan the launch".into(),
                },
                crate::ai::agent_chat::ui::history::SavedMessage {
                    role: "Assistant".into(),
                    body: "Tuesday, with a demo.".into(),
                },
            ],
        };
        let entry = crate::ai::agent_chat::ui::history::build_history_entry(&conversation)
            .expect("a history entry");
        crate::ai::agent_chat::ui::history::save_completed_conversation(&conversation, &entry)
            .expect("save the past chat");
        let id = conversation.session_id.clone();
        let (label, waiting, stopped) = in_launcher("past_chat_held_delete", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            app.update(cx, |app, cx| {
                app.one_window_open_past_chats(String::new(), cx)
            });
            wait(cx, ENTER_GUARD_MS);
            key(&app, cx, Key::CmdK);
            wait(cx, ENTER_GUARD_MS);
            for _ in 0..3 {
                key(&app, cx, Key::Down);
            }
            let label = enter_label(&app, cx);
            key(&app, cx, Key::Enter);
            wait(cx, 20);
            let line = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext| {
                app.update(cx, |app, _| {
                    app.one_window.slot_line(app.one_window_clock())
                })
            };
            let waiting = line(&app, cx);
            // Through the window, so the line's ⌘Z interceptor claims it before the input's undo.
            cx.dispatch_keystroke(window, gpui::Keystroke::parse("cmd-z").expect("cmd-z"));
            wait(cx, 20);
            let stopped = line(&app, cx);
            wait(cx, 9_000);
            (label, waiting, stopped)
        });
        let kept = crate::ai::agent_chat::ui::history::conversation_exists(&id);
        (label, waiting, stopped, kept)
    });
    assert_eq!(label, "Delete it · “plan the launch”");
    assert_eq!(
        waiting.kind,
        sk_one_window::SlotKind::Holding,
        "{waiting:?}"
    );
    assert_eq!(
        waiting.label,
        sk_one_window::contract::ledger_words::runs_in(8)
    );
    assert_eq!(waiting.title, "Delete “plan the launch”");
    assert_eq!(
        stopped.title,
        sk_one_window::contract::ledger_words::stopped(&waiting.title, true)
    );
    assert!(kept, "the chat is still saved after ⌘Z and the wait");
}

#[test]
fn the_models_question_is_a_row_answered_with_enter() {
    let (asked, label, answered, reply) = in_launcher("chat_permission_row", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        key(&app, cx, Key::CmdEnter);
        wait(cx, 20);
        type_into_launcher(cx, window, &app, "clean the build folder");
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, 50);
        let asked = app.update(cx, |app, _| {
            app.one_window
                .chat()
                .and_then(|c| c.asks.as_ref().map(|a| a.title.clone()))
        });
        wait(cx, ENTER_GUARD_MS);
        let label = enter_label(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, ENTER_GUARD_MS);
        let (answered, reply) = app.update(cx, |app, _| {
            let chat = app.one_window.chat();
            (
                chat.is_some_and(|c| c.asks.is_none()),
                chat.and_then(|c| c.reply().map(str::to_string)),
            )
        });
        (asked, label, answered, reply)
    });
    assert_eq!(asked.as_deref(), Some("Run rm -rf build"));
    assert_eq!(
        label, "Allow: Run rm -rf build",
        "Enter owns the one-time yes"
    );
    assert!(answered, "the question goes once it is answered");
    assert_eq!(reply.as_deref(), Some("Cleaned up the build folder."));
}
