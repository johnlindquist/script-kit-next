//! ⌘K in a prompt or built-in view through the real launcher: what the view has focused goes in
//! the line above the input, the list becomes its actions, typing filters them, Enter runs the
//! chosen one with the view's own handler, Esc gives the view back exactly as it was, and no
//! window opens at any point.

use super::*;

use crate::actions::ActionsDialog;
use crate::components::staged_actions::staged_action_rows;
use crate::protocol::{Choice, Message, ProtocolAction};
use crate::FileSearchPresentation;

fn in_window<T>(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    app: &gpui::Entity<ScriptListApp>,
    body: impl FnOnce(&mut ScriptListApp, &mut gpui::Window, &mut gpui::Context<ScriptListApp>) -> T,
) -> T {
    cx.update(|cx| {
        window
            .update(cx, |_, window, cx| app.update(cx, |app, cx| body(app, window, cx)))
            .expect("the test launcher window")
    })
}

fn press(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    app: &gpui::Entity<ScriptListApp>,
    key: &str,
) -> bool {
    let key_char = (key.chars().count() == 1).then(|| key.to_string());
    let used = in_window(cx, window, app, |app, window, cx| {
        app.one_window_staged_key(
            key,
            key_char.as_deref(),
            &gpui::Modifiers::default(),
            window,
            cx,
        )
    });
    wait(cx, 20);
    used
}

fn type_text(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    app: &gpui::Entity<ScriptListApp>,
    text: &str,
) {
    for ch in text.chars() {
        let key = if ch == ' ' { "space".to_string() } else { ch.to_string() };
        let used = in_window(cx, window, app, |app, window, cx| {
            app.one_window_staged_key(
                &key,
                Some(&ch.to_string()),
                &gpui::Modifiers::default(),
                window,
                cx,
            )
        });
        assert!(used, "the staged actions take {ch:?}");
    }
    wait(cx, 20);
}

/// What the window shows while something is staged.
#[derive(Debug)]
struct Staged {
    label: String,
    title: String,
    head: Option<String>,
    /// The reducer's rows, by title.
    rows: Vec<String>,
    /// The rows the list draws, by title, and which one is highlighted.
    drawn: Vec<String>,
    highlighted: Option<String>,
    elements: Vec<String>,
    /// What the ↵ capsule says.
    enter: Option<String>,
}

fn staged(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Option<Staged> {
    app.update(cx, |app, cx| {
        let (_, dialog) = app.one_window_view_staged()?;
        let line = app.one_window.slot_line(app.one_window_clock());
        let staged = app.one_window.staged();
        let drawn = staged_action_rows(dialog.read(cx));
        Some(Staged {
            label: line.label,
            title: line.title,
            head: staged.map(|staged| staged.head.clone()),
            rows: staged
                .map(|staged| staged.rows.iter().map(|row| row.title.clone()).collect())
                .unwrap_or_default(),
            highlighted: drawn
                .iter()
                .find(|row| row.selected)
                .map(|row| row.title.clone()),
            drawn: drawn.into_iter().map(|row| row.title).collect(),
            enter: app.main_window_footer_config_with_cx(Some(cx)).and_then(|footer| {
                footer
                    .buttons
                    .iter()
                    .find(|button| button.action == FooterAction::Run)
                    .map(|button| button.label.to_string())
            }),
            elements: app
                .one_window_view_stage_elements(100, true, cx)
                .map(|outcome| {
                    outcome
                        .elements
                        .into_iter()
                        .map(|element| element.semantic_id)
                        .collect()
                })
                .unwrap_or_default(),
        })
    })
}

fn windows(cx: &mut gpui::TestAppContext) -> usize {
    cx.update(|cx| cx.windows().len())
}

fn sdk_action(name: &str) -> ProtocolAction {
    ProtocolAction {
        has_action: true,
        ..ProtocolAction::new(name.to_string())
    }
}

fn open_arg_prompt(app: &mut ScriptListApp) {
    let actions = vec![
        sdk_action("Copy Value"),
        sdk_action("Open Details"),
        sdk_action("Pin It"),
    ];
    app.current_view = AppView::ArgPrompt {
        id: "arg-staging".to_string(),
        placeholder: "Pick a fruit".to_string(),
        choices: vec![
            Choice::new("Apple".to_string(), "apple".to_string()),
            Choice::new("Banana".to_string(), "banana".to_string()),
            Choice::new("Cherry".to_string(), "cherry".to_string()),
        ],
        actions: Some(actions.clone()),
    };
    app.arg_selected_index = 1;
    app.set_sdk_actions_and_shortcuts(actions, "UI", false);
}

#[test]
fn cmd_k_on_an_arg_prompt_stages_the_choice_and_its_sdk_actions_run_in_place() {
    let (opened, filtered, sent, after_enter, windows_opened) =
        in_launcher("arg_cmd_k_in_place", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            let (sender, receiver) = std::sync::mpsc::sync_channel(8);
            let before = windows(cx);
            in_window(cx, window, &app, |app, window, cx| {
                app.response_sender = Some(sender);
                open_arg_prompt(app);
                assert!(app.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k"));
            });
            wait(cx, 20);
            let opened = staged(&app, cx);
            type_text(cx, window, &app, "pin");
            let filtered = staged(&app, cx);
            let windows_opened = windows(cx) - before;
            press(cx, window, &app, "enter");
            let sent = receiver.try_iter().collect::<Vec<_>>();
            (opened, filtered, sent, staged(&app, cx), windows_opened)
        });

    let opened = opened.expect("⌘K staged the arg prompt's choice");
    assert_eq!(opened.label, words::ACTING_ON);
    assert_eq!(opened.title, "Banana", "the highlighted choice is what's staged");
    assert_eq!(opened.head.as_deref(), Some(words::STAGED));
    assert_eq!(opened.drawn, ["Copy Value", "Open Details", "Pin It"]);
    assert_eq!(opened.rows, opened.drawn, "the reducer lists what the view draws");
    assert!(
        opened
            .elements
            .iter()
            .any(|id| id == "staged-action:Pin It"),
        "getElements sees the staged rows: {:?}",
        opened.elements
    );

    let filtered = filtered.expect("still staged while typing");
    assert_eq!(filtered.drawn, ["Pin It"], "typing filters the actions");
    assert_eq!(filtered.rows, filtered.drawn);
    assert_eq!(filtered.highlighted.as_deref(), Some("Pin It"));
    assert_eq!(
        filtered.enter.as_deref(),
        Some("Pin It"),
        "↵ names the action Enter runs"
    );

    assert!(
        sent.iter()
            .any(|message| matches!(message, Message::ActionTriggered { action, .. } if action == "Pin It")),
        "Enter runs the chosen action through the SDK handler: {sent:?}"
    );
    assert!(after_enter.is_none(), "running an action lets go");
    assert_eq!(windows_opened, 0, "nothing opened");
}

#[test]
fn esc_gives_the_arg_prompt_back_with_its_selection() {
    let (unstaged, line_label, view, selection, show_actions) =
        in_launcher("arg_esc_restores", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            in_window(cx, window, &app, |app, window, cx| {
                open_arg_prompt(app);
                assert!(app.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k"));
            });
            wait(cx, 20);
            type_text(cx, window, &app, "op");
            assert!(press(cx, window, &app, "escape"));
            let unstaged = staged(&app, cx).is_none();
            app.update(cx, |app, _| {
                (
                    unstaged,
                    app.one_window.slot_line(app.one_window_clock()).label,
                    matches!(app.current_view, AppView::ArgPrompt { .. }),
                    app.arg_selected_index,
                    app.show_actions_popup,
                )
            })
        });
    assert!(unstaged, "Esc lets go");
    assert_ne!(line_label, words::ACTING_ON, "the line no longer acts on it");
    assert!(view, "the arg prompt is still there");
    assert_eq!(selection, 1, "its selection is exactly as it was");
    assert!(!show_actions);
}

fn open_file_search(app: &mut ScriptListApp) {
    app.current_view = AppView::FileSearchView {
        query: "/tmp/".to_string(),
        selected_index: 1,
        presentation: FileSearchPresentation::Full,
    };
    app.cached_file_results = ["alpha.txt", "beta.txt", "gamma.txt"]
        .iter()
        .enumerate()
        .map(|(index, name)| crate::file_search::FileResult {
            path: format!("/tmp/{name}"),
            name: name.to_string(),
            size: 10,
            modified: index as u64,
            file_type: crate::file_search::FileType::File,
        })
        .collect();
    app.file_search_display_indices = vec![0, 1, 2];
}

#[test]
fn cmd_k_in_file_search_stages_the_file_and_its_actions_run_in_place() {
    let (opened, filtered, sort_mode, after_enter, selected_after, windows_opened) =
        in_launcher("file_search_cmd_k_in_place", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            let before = windows(cx);
            in_window(cx, window, &app, |app, window, cx| {
                open_file_search(app);
                assert!(app.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k"));
            });
            wait(cx, 20);
            let opened = staged(&app, cx);
            type_text(cx, window, &app, "newest");
            let filtered = staged(&app, cx);
            let windows_opened = windows(cx) - before;
            press(cx, window, &app, "enter");
            app.update(cx, |app, _| {
                let selected = app
                    .selected_file_search_result_owned()
                    .map(|(_, file)| file.name);
                (
                    opened,
                    filtered,
                    app.file_search_sort_mode,
                    app.one_window_view_staged().is_none(),
                    selected,
                    windows_opened,
                )
            })
        });

    let opened = opened.expect("⌘K staged the highlighted file");
    assert_eq!(opened.label, words::ACTING_ON);
    assert_eq!(opened.title, "beta.txt");
    assert_eq!(opened.head.as_deref(), Some(words::STAGED));
    assert!(
        opened.drawn.iter().any(|title| title == "Reveal in Finder"),
        "the rows are the file's actions: {:?}",
        opened.drawn
    );
    assert_eq!(opened.rows, opened.drawn);

    let filtered = filtered.expect("still staged while typing");
    assert_eq!(
        filtered.highlighted.as_deref(),
        Some("Sort by Modified (Newest)"),
        "typing filters the actions: {:?}",
        filtered.drawn
    );
    assert!(filtered.drawn.len() < opened.drawn.len());

    assert_eq!(
        sort_mode,
        crate::actions::FileSearchSortMode::ModifiedDesc,
        "Enter ran the chosen action with file search's own handler"
    );
    assert!(after_enter, "running an action lets go");
    assert!(selected_after.is_some(), "the files are listed again");
    assert_eq!(windows_opened, 0, "nothing opened");
}

#[test]
fn esc_gives_file_search_back_with_its_query_and_selection() {
    let (unstaged, view) = in_launcher("file_search_esc_restores", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        in_window(cx, window, &app, |app, window, cx| {
            open_file_search(app);
            assert!(app.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k"));
        });
        wait(cx, 20);
        type_text(cx, window, &app, "rev");
        assert!(press(cx, window, &app, "escape"));
        let unstaged = staged(&app, cx).is_none();
        (unstaged, app.update(cx, |app, _| app.current_view.clone()))
    });
    assert!(unstaged, "Esc lets go");
    match view {
        AppView::FileSearchView {
            query,
            selected_index,
            ..
        } => {
            assert_eq!(query, "/tmp/", "the query is exactly as it was");
            assert_eq!(selected_index, 1, "and so is the selection");
        }
        other => panic!("file search is still the view, not {other:?}"),
    }
}

fn open_clipboard_history(app: &mut ScriptListApp) {
    app.cached_clipboard_entries = ["first copy", "github.com/script-kit/app/pull/812", "third"]
        .iter()
        .enumerate()
        .map(|(index, text)| crate::clipboard_history::ClipboardEntryMeta {
            id: format!("entry-{index}"),
            content_type: crate::clipboard_history::ContentType::Text,
            timestamp: 1_000 - index as i64,
            pinned: false,
            text_preview: text.to_string(),
            image_width: None,
            image_height: None,
            byte_size: text.len(),
            ocr_text: None,
        })
        .collect();
    app.current_view = AppView::ClipboardHistoryView {
        filter: String::new(),
        selected_index: 1,
    };
}

#[test]
fn cmd_k_in_clipboard_history_stages_the_entry_and_esc_gives_the_list_back() {
    let (opened, filtered, unstaged, view, windows_opened) =
        in_launcher("clipboard_cmd_k_in_place", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            let before = windows(cx);
            in_window(cx, window, &app, |app, window, cx| {
                open_clipboard_history(app);
                assert!(app.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k"));
            });
            wait(cx, 20);
            let opened = staged(&app, cx);
            type_text(cx, window, &app, "pin");
            let filtered = staged(&app, cx);
            let windows_opened = windows(cx) - before;
            assert!(press(cx, window, &app, "escape"));
            let unstaged = staged(&app, cx).is_none();
            (
                opened,
                filtered,
                unstaged,
                app.update(cx, |app, _| app.current_view.clone()),
                windows_opened,
            )
        });

    let opened = opened.expect("⌘K staged the highlighted entry");
    assert_eq!(opened.label, words::ACTING_ON);
    assert_eq!(opened.title, "github.com/script-kit/app/pull/812");
    assert_eq!(opened.head.as_deref(), Some(words::STAGED));
    assert!(
        opened.drawn.iter().any(|title| title.starts_with("Paste")),
        "the rows are the entry's actions: {:?}",
        opened.drawn
    );
    let filtered = filtered.expect("still staged while typing");
    assert!(
        filtered.drawn.len() < opened.drawn.len(),
        "typing filters the actions: {:?}",
        filtered.drawn
    );
    assert_eq!(filtered.highlighted.as_deref(), Some("Pin Entry"));
    assert!(unstaged, "Esc lets go");
    assert!(
        matches!(
            view,
            AppView::ClipboardHistoryView { ref filter, selected_index: 1 } if filter.is_empty()
        ),
        "the clipboard list and its selection are exactly as they were: {view:?}"
    );
    assert_eq!(windows_opened, 0, "nothing opened");
}

#[test]
fn a_staged_dialog_is_the_same_entity_the_actions_come_from() {
    let same = in_launcher("staged_dialog_identity", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        in_window(cx, window, &app, |app, window, cx| {
            open_arg_prompt(app);
            app.dispatch_actions_toggle_for_current_view(window, cx, "cmd_k");
        });
        app.update(cx, |app, _| {
            let staged: Option<gpui::Entity<ActionsDialog>> =
                app.one_window_view_staged().map(|(_, dialog)| dialog);
            staged.map(|dialog| dialog.entity_id()) == app.actions_dialog.as_ref().map(|d| d.entity_id())
                && crate::actions::is_actions_window_open()
        })
    });
    assert!(same, "surfaces outside the app see what the main window staged");
}
