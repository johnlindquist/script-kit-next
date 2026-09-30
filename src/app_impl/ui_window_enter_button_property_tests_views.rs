//! Views under the same input, through the real `ScriptListApp` and its reducer: what you copied
//! becomes the subject, a staged row assembles a form in place, and an answer arriving in place
//! doesn't take an Enter meant for before it.

use sk_one_window::answer::{Answer, Fact, Shape};
use sk_one_window::contract::ENTER_GUARD_MS;
use sk_one_window::form::{FieldState, SlotName, Source, Verb};
use sk_one_window::{Key, SubjectKind};

use super::{in_launcher, key, launcher, type_into_launcher, wait, Jev};
use crate::one_window::views::HostedView;
use crate::ScriptListApp;

const LINK: &str = "github.com/script-kit/app/pull/812";

fn copy(cx: &mut gpui::TestAppContext, text: &str) {
    cx.update(|cx| cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.to_string())));
}

fn paste(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> bool {
    app.update(cx, |app, cx| app.one_window_stage_paste(cx))
}

fn hosted(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Option<HostedView> {
    app.update(cx, |app, _| app.one_window_hosted_view().cloned())
}

fn enter_label(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| app.one_window_enter_label())
}

#[test]
fn pasting_into_the_empty_input_stages_what_was_copied() {
    let (staged, slot, rows, drawn, typed_paste, typed_staged) =
        in_launcher("paste_stages_clipboard", 0, |cx| {
            let (app, window) = launcher(cx, Jev::Off);
            copy(cx, LINK);
            let staged = paste(&app, cx);
            wait(cx, 20);
            let (slot, rows, drawn, kind) = app.update(cx, |app, _| {
                let now = app.one_window_clock();
                let list = app.one_window.list();
                let rows: Vec<String> = list
                    .visible()
                    .iter()
                    .filter_map(|item| list.row(item).map(|row| row.title.clone()))
                    .collect();
                (
                    app.one_window.chip().map(|chip| chip.title).unwrap_or_default(),
                    rows,
                    app.one_window_view_element().is_some(),
                    app.one_window.staged().map(|s| s.subject.kind),
                )
            });
            assert_eq!(kind, Some(SubjectKind::Clipboard));
            key(&app, cx, Key::Escape);
            type_into_launcher(cx, window, &app, "tell");
            wait(cx, 20);
            let typed_paste = paste(&app, cx);
            let typed_staged = app.update(cx, |app, _| app.one_window.staged().is_some());
            (staged, slot, rows, drawn, typed_paste, typed_staged)
        });
    assert!(staged, "⌘V into the empty input is the window's");
    assert_eq!(slot, LINK, "the input's chip names what was copied");
    assert_eq!(
        rows,
        [
            "Open it",
            "Send it to someone",
            "Remind me about it",
            "Add it to today's note",
            "Show all of it"
        ],
        "the list is what you can do with it"
    );
    assert!(drawn, "its rows are drawn in place of the list");
    assert!(!typed_paste, "into typed words the paste is the input's");
    assert!(!typed_staged);
}

#[test]
fn a_copied_link_sent_to_someone_asks_who_and_then_waits_in_the_line() {
    let (asks, caret_on_to, named, held, closed) = in_launcher("clipboard_message_form", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        copy(cx, LINK);
        paste(&app, cx);
        wait(cx, 20);
        key(&app, cx, Key::Down);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let Some(HostedView::Form(form)) = hosted(&app, cx) else {
            panic!("Send it to someone opens a form in place");
        };
        let what = form.field(SlotName::What).expect("the message").clone();
        assert_eq!(what.value, LINK);
        assert_eq!(what.source_words(), "from the clipboard");
        let to = form.fields.iter().position(|f| f.name == SlotName::Who);
        let caret_on_to = to.is_some() && form.caret == to;
        assert_eq!(form.fields[to.unwrap()].state, FieldState::Needed);
        let asks = enter_label(&app, cx);
        type_into_launcher(cx, window, &app, "Sarah");
        wait(cx, 20);
        let named = enter_label(&app, cx);
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let held = app.update(cx, |app, _| app.one_window.holds().live().count());
        (asks, caret_on_to, named, held, hosted(&app, cx).is_none())
    });
    assert_eq!(
        asks, "Say who gets it",
        "the field nobody filled owns Enter"
    );
    assert!(caret_on_to, "and takes the caret");
    assert_eq!(named, format!("Send “{LINK}” to Sarah"));
    assert_eq!(held, 1, "the message waits in the line before it goes");
    assert!(closed, "the form closes back to the list");
}

#[test]
fn enter_runs_the_action_highlighted_after_pasting_again() {
    let opened = in_launcher("paste_again_enter", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        copy(cx, "84");
        paste(&app, cx);
        wait(cx, 20);
        for _ in 0..3 {
            key(&app, cx, Key::Down);
        }
        key(&app, cx, Key::Enter);
        wait(cx, ENTER_GUARD_MS);
        assert!(matches!(hosted(&app, cx), Some(HostedView::Answer(_))));
        key(&app, cx, Key::Escape);
        wait(cx, ENTER_GUARD_MS);
        paste(&app, cx);
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        hosted(&app, cx)
    });
    assert!(
        matches!(&opened, Some(HostedView::Form(form)) if form.verb == Verb::Message),
        "Enter after pasting again runs the first action, not the last paste's: {opened:?}"
    );
}

#[test]
fn tab_moves_the_caret_through_the_form_and_the_input_follows_it() {
    let (carets, input) = in_launcher("form_tab", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        copy(cx, LINK);
        paste(&app, cx);
        wait(cx, 20);
        key(&app, cx, Key::Down);
        key(&app, cx, Key::Down);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let caret = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext| match hosted(
            app, cx,
        ) {
            Some(HostedView::Form(form)) => form.caret,
            _ => None,
        };
        let mut carets = vec![caret(&app, cx)];
        for _ in 0..2 {
            key(&app, cx, Key::Tab);
            wait(cx, 20);
            carets.push(caret(&app, cx));
        }
        let input = app.update(cx, |app, _| app.filter_text.clone());
        let what = match hosted(&app, cx) {
            Some(HostedView::Form(form)) => form.field(SlotName::What).cloned(),
            _ => None,
        };
        assert_eq!(
            what.and_then(|f| f.from),
            Some(Source::Staged(SubjectKind::Clipboard)),
            "Tab onto a field keeps where its value came from"
        );
        (carets, input)
    });
    assert_eq!(
        carets,
        [Some(1), Some(0), Some(1)],
        "a reminder about the clipboard asks when; Tab goes round its two fields"
    );
    assert_eq!(input, "", "the caret's field is the input's words");
}

#[test]
fn the_calculator_answers_in_place_with_a_number_enter_copies() {
    let (shape, label, copied, closed) = in_launcher("calculator_answer", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "12*7");
        wait(cx, 20);
        let row_label = app.update(cx, |app, cx| {
            app.main_window_footer_config_with_cx(Some(cx))
                .and_then(|footer| {
                    footer
                        .buttons
                        .iter()
                        .find(|button| button.action == crate::footer_popup::FooterAction::Run)
                        .map(|button| button.label.to_string())
                })
        });
        assert_eq!(
            row_label.as_deref(),
            Some("Show 84"),
            "↵ on the calculator's row says it opens the answer"
        );
        let _ = app.update(cx, |app, cx| app.execute_selected(cx));
        let shape = match hosted(&app, cx) {
            Some(HostedView::Answer(answer)) => answer.shape,
            _ => None,
        };
        let label = enter_label(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let copied = cx.update(|cx| cx.read_from_clipboard().and_then(|item| item.text()));
        (shape, label, copied, hosted(&app, cx).is_none())
    });
    assert!(
        matches!(&shape, Some(Shape::Number { value, .. }) if value == "84"),
        "a calculation is a number: {shape:?}"
    );
    assert_eq!(label, "Copy 84");
    assert_eq!(copied.as_deref(), Some("84"), "Enter copies the number");
    assert!(closed, "and the answer closes back to the list");
}

#[test]
fn an_answer_that_arrives_later_does_not_take_an_enter_meant_for_before() {
    let (waiting, early, late, closed) = in_launcher("answer_arrives", 0, |cx| {
        let (app, _) = launcher(cx, Jev::Off);
        let question = "when's my next meeting";
        app.update(cx, |app, cx| {
            app.one_window_open_hosted(HostedView::Answer(Answer::asking(question)), None, cx)
        });
        let waiting = enter_label(&app, cx);
        wait(cx, 120);
        app.update(cx, |app, cx| {
            let mut answer = Answer::asking(question);
            answer.arrived(vec![Fact::new(
                "Standup · 9:30 AM",
                "with Sarah, Mo, Priya",
                Source::Known("calendar".into()),
            )]);
            app.one_window_answer_arrived(answer, cx);
        });
        let early = key(&app, cx, Key::Enter);
        let still_open = hosted(&app, cx).is_some();
        wait(cx, ENTER_GUARD_MS);
        let late = key(&app, cx, Key::Enter);
        wait(cx, 20);
        assert!(still_open, "the ignored Enter left the answer open");
        (waiting, early, late, hosted(&app, cx).is_none())
    });
    assert_eq!(waiting, "Reading…");
    assert_eq!(
        early,
        (false, true),
        "an Enter as the answer arrives is ignored"
    );
    assert_eq!(
        late,
        (false, false),
        "after the guard Enter goes to the answer"
    );
    assert!(closed, "which copies it and closes");
}

#[test]
fn a_note_opens_in_the_main_window_with_the_words_as_its_new_line() {
    let (view, label, windows) = in_launcher("note_view", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "buy oat milk");
        wait(cx, 400);
        key(&app, cx, Key::CmdDigit(3));
        wait(cx, 20);
        let windows = cx.update(|cx| cx.windows().len());
        (hosted(&app, cx), enter_label(&app, cx), windows)
    });
    let Some(HostedView::Note(note)) = view else {
        panic!("Add it to today's note opens the note in place: {view:?}");
    };
    assert_eq!(note.line, "buy oat milk");
    assert_eq!(label, "Add it to “Today”");
    assert_eq!(windows, 1, "no Notes window opens");
}
