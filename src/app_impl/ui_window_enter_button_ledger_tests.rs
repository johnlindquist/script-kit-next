//! The ledger through the real launcher: what still needs setting up, what the app says, what
//! waits before it runs, what ran today, and ↑ on an empty input.

use sk_one_window::contract::{ledger_words, words, ENTER_GUARD_MS};
use sk_one_window::{Key, SlotKind};

use super::{in_launcher, key, launcher, wait, Jev};
use crate::list_item::GroupedListItem;
use crate::ScriptListApp;

fn line(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> sk_one_window::SlotLine {
    app.update(cx, |app, _| {
        app.one_window.slot_line(app.one_window_clock())
    })
}

/// The drawn list's headers and row names, in order.
fn drawn(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Vec<String> {
    app.update(cx, |app, _| {
        let (grouped, flat) = app.main_menu_result_caches.clone_grouped_results();
        grouped
            .iter()
            .map(|item| match item {
                GroupedListItem::SectionHeader(label, _) => format!("# {label}"),
                GroupedListItem::Item(ix) => flat
                    .get(*ix)
                    .map(|r| r.name().to_string())
                    .unwrap_or_default(),
                GroupedListItem::Status(_) => String::new(),
            })
            .collect()
    })
}

#[test]
fn a_missing_bun_leads_the_empty_list_with_needs_setup() {
    let list = in_launcher("needs_setup", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        drawn(&app, cx)
    });
    assert_eq!(
        list.first().map(String::as_str),
        Some("# Needs setup"),
        "{list:?}"
    );
    assert_eq!(
        list.get(1).map(String::as_str),
        Some(ledger_words::BUN_MISSING),
        "{list:?}"
    );
}

#[test]
fn a_script_hud_is_the_line_now_and_after_the_window_comes_back() {
    let (showing, after_summon) = in_launcher("hud_in_the_line", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.handle_prompt_message(
                crate::PromptMessage::ShowHud {
                    text: "Backup finished".to_string(),
                    duration_ms: None,
                },
                cx,
            )
        });
        wait(cx, 20);
        let showing = line(&app, cx);
        app.update(cx, |app, cx| {
            app.one_window_handle(sk_one_window::Event::Dismissed, cx);
        });
        cx.update(|cx| crate::line_notice::say(cx, crate::line_notice::INFO, "Sync done"));
        wait(cx, 5_000);
        app.update(cx, |app, cx| app.one_window_summon(cx));
        wait(cx, 20);
        (showing, line(&app, cx))
    });
    assert_eq!(
        (showing.kind, showing.title.as_str()),
        (SlotKind::Recent, "Backup finished")
    );
    assert_eq!(after_summon.label, words::LAST);
    assert_eq!(after_summon.title, "Sync done");
    assert!(
        after_summon.sub.contains(words::JUST_NOW),
        "{after_summon:?}"
    );
}

#[test]
fn a_builtin_that_used_to_ask_first_waits_and_cmd_z_stops_it() {
    let (waiting, stopped) = in_launcher("held_builtin", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            let entry = crate::builtins::get_builtin_entries(&app.config.get_builtins())
                .into_iter()
                .find(|entry| entry.id == "builtin/clear-suggested")
                .expect("Clear Suggested is a built-in");
            app.execute_builtin(&entry, cx);
        });
        wait(cx, 20);
        let waiting = line(&app, cx);
        // Through the window, so the line's ⌘Z interceptor claims it before the input's undo.
        cx.dispatch_keystroke(window, gpui::Keystroke::parse("cmd-z").expect("cmd-z"));
        wait(cx, 20);
        (waiting, line(&app, cx))
    });
    assert_eq!(waiting.kind, SlotKind::Holding, "{waiting:?}");
    assert_eq!(waiting.label, ledger_words::runs_in(8));
    assert_eq!(stopped.title, ledger_words::stopped(&waiting.title, true));
}

#[test]
fn what_ran_today_opens_history_and_enter_acts_on_the_entry() {
    let (list, staged) = in_launcher("history_view", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_notice(crate::line_notice::DONE, "Copied the build log", cx);
            app.one_window_open_history(cx);
        });
        wait(cx, ENTER_GUARD_MS + 20);
        let list = drawn(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        (list, line(&app, cx))
    });
    assert_eq!(
        list.first().map(|h| h.trim_start_matches("# ")),
        Some(ledger_words::HISTORY_HEAD)
    );
    assert_eq!(
        list.get(1).map(String::as_str),
        Some("Copied the build log"),
        "{list:?}"
    );
    assert_eq!(
        (staged.kind, staged.title.as_str()),
        (SlotKind::Staged, "Copied the build log")
    );
}

#[test]
fn up_on_an_empty_input_stages_the_last_run_and_lists_what_to_do_with_it() {
    let (staged, list) = in_launcher("up_stages_last", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_notice(crate::line_notice::DONE, "Copied the build log", cx)
        });
        wait(cx, ENTER_GUARD_MS + 20);
        key(&app, cx, Key::Up);
        wait(cx, 20);
        (line(&app, cx), drawn(&app, cx))
    });
    assert_eq!(
        (staged.kind, staged.title.as_str()),
        (SlotKind::Staged, "Copied the build log")
    );
    assert_eq!(
        list.first().map(|h| h.trim_start_matches("# ")),
        Some(words::STAGED),
        "{list:?}"
    );
    assert!(list.iter().any(|row| row == "Run it again"), "{list:?}");
}

#[test]
fn a_change_happens_at_once_and_cmd_z_takes_it_back() {
    let (before, offered, changed, undone, after) = in_launcher("undo_offer", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        let before = app.update(cx, |app, cx| {
            let before = app.background_effect;
            let entry = crate::builtins::get_builtin_entries(&app.config.get_builtins())
                .into_iter()
                .find(|entry| entry.id == "builtin/background-effect-next")
                .expect("Next Background Effect is a built-in");
            app.execute_builtin(&entry, cx);
            before
        });
        wait(cx, ENTER_GUARD_MS + 20);
        let offered = line(&app, cx);
        let changed = app.update(cx, |app, _| app.background_effect);
        key(&app, cx, Key::CmdZ);
        wait(cx, 20);
        let undone = line(&app, cx);
        (before, offered, changed, undone, app.update(cx, |app, _| app.background_effect))
    });
    assert_ne!(changed, before, "the change happened at once");
    assert!(
        offered
            .hints
            .iter()
            .any(|(key, label)| key == "⌘Z" && label == ledger_words::UNDO),
        "{offered:?}"
    );
    assert_eq!(undone.title, format!("Undid · {}", offered.title));
    assert_eq!(after, before, "⌘Z put the previous effect back");
}
