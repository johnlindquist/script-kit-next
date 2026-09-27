//! Notes in the main window, through the real `ScriptListApp` and its reducer: a note's lines are
//! rows under the input, the input adds or changes a line, a delete waits in the line above the
//! input, ⌘P finds a note by name, and ⌘↵ hands the note to the model. No window opens.

use sk_one_window::contract::ENTER_GUARD_MS;
use sk_one_window::Key;

use super::{in_launcher, key, launcher, type_into_launcher, wait, Jev};
use crate::notes::{Note, NoteId, NoteTarget};
use crate::one_window::notes::{DELETE_LINE_KEY, FIND_NOTE_KEY};
use crate::one_window::views::HostedView;
use crate::ScriptListApp;

/// A note of its own for each test, so tests sharing the notes store don't meet.
fn seed_note(title: &str, lines: &[&str]) -> NoteId {
    crate::notes::init_notes_db().expect("notes store");
    let note = Note::with_content(format!("# {title}\n{}", lines.join("\n")));
    crate::notes::save_note(&note).expect("save the note");
    note.id
}

fn stored_body(id: NoteId) -> String {
    crate::notes::get_note(id)
        .expect("read the note")
        .expect("the note")
        .content
}

fn open(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext, target: NoteTarget) {
    app.update(cx, |app, cx| app.one_window_open_note_target(target, cx))
        .expect("the note opens");
    wait(cx, ENTER_GUARD_MS);
}

fn note_rows(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> Vec<String> {
    app.update(cx, |app, _| match app.one_window_hosted_view() {
        Some(HostedView::Note(note)) => note.rows().into_iter().map(|row| row.title).collect(),
        _ => Vec::new(),
    })
}

fn enter_label(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> String {
    app.update(cx, |app, _| app.one_window_enter_label())
}

#[test]
fn a_note_reads_as_rows_and_the_input_adds_a_line() {
    let (rows, label, stored, after, windows) = in_launcher("note_add_line", 0, |cx| {
        let id = seed_note("Groceries", &["- oat milk", "- eggs"]);
        let (app, window) = launcher(cx, Jev::Off);
        open(&app, cx, NoteTarget::Note(id));
        let rows = note_rows(&app, cx);
        type_into_launcher(cx, window, &app, "bread");
        wait(cx, 20);
        let label = enter_label(&app, cx);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let windows = cx.update(|cx| cx.windows().len());
        (rows, label, stored_body(id), note_rows(&app, cx), windows)
    });
    assert_eq!(rows, ["Groceries", "oat milk", "eggs", "Write a line…"]);
    assert_eq!(label, "Add it to “Groceries”");
    assert!(
        stored.ends_with("- eggs\nbread"),
        "the line is stored: {stored:?}"
    );
    assert_eq!(
        after,
        ["Groceries", "oat milk", "eggs", "bread", "Write a line…"],
        "the note stays open with the line added"
    );
    assert_eq!(windows, 1, "no Notes window opens");
}

#[test]
fn enter_on_a_line_puts_it_in_the_input_and_enter_again_saves_the_change() {
    let (editing, input, stored) = in_launcher("note_edit_line", 0, |cx| {
        let id = seed_note("Trip", &["- book the hotel", "07:56 - pack the charger"]);
        let (app, window) = launcher(cx, Jev::Off);
        open(&app, cx, NoteTarget::Note(id));
        key(&app, cx, Key::Up);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let editing = enter_label(&app, cx);
        let input = app.update(cx, |app, _| app.filter_text.clone());
        type_into_launcher(cx, window, &app, "pack both chargers");
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        (editing, input, stored_body(id))
    });
    assert_eq!(editing, "Save the line");
    assert_eq!(
        input, "pack the charger",
        "the line's words are in the input"
    );
    assert!(
        stored.contains("\n07:56 - pack both chargers") && !stored.contains("pack the charger"),
        "the line is changed in place, keeping its time and bullet: {stored:?}"
    );
}

#[test]
fn deleting_a_line_waits_in_the_line_above_the_input_and_can_be_stopped() {
    let (hidden, held, still_stored, open_after, stopped_rows, stored_after_stop) =
        in_launcher("note_delete_line", 0, |cx| {
            let id = seed_note("Ideas", &["- keep this", "- drop this"]);
            let (app, _) = launcher(cx, Jev::Off);
            open(&app, cx, NoteTarget::Note(id));
            key(&app, cx, Key::Up);
            key(&app, cx, Key::Other(DELETE_LINE_KEY.into()));
            wait(cx, 20);
            let hidden = note_rows(&app, cx);
            let held = app.update(cx, |app, _| {
                app.one_window.slot_line(app.one_window_clock()).title
            });
            let still_stored = stored_body(id);
            let open_after = app.update(cx, |app, _| app.one_window_note_open());
            let cmd = gpui::Modifiers {
                platform: true,
                ..gpui::Modifiers::default()
            };
            let taken = app.update(cx, |app, cx| app.one_window_view_takes_key("z", &cmd, cx));
            assert!(taken, "⌘Z in the note is the note's, not the input's undo");
            wait(cx, 20);
            let stopped_rows = note_rows(&app, cx);
            wait(cx, 9_000);
            (
                hidden,
                held,
                still_stored,
                open_after,
                stopped_rows,
                stored_body(id),
            )
        });
    assert!(
        !hidden.contains(&"drop this".to_string()),
        "the line leaves the note at once: {hidden:?}"
    );
    assert_eq!(
        held, "Delete “drop this”",
        "it waits in the line above the input"
    );
    assert!(
        still_stored.contains("- drop this"),
        "nothing is deleted while it waits"
    );
    assert!(open_after, "the note stays open while the delete waits");
    assert!(
        stopped_rows.contains(&"drop this".to_string()),
        "⌘Z brings the line back: {stopped_rows:?}"
    );
    assert!(
        stored_after_stop.contains("- drop this"),
        "a stopped delete never runs"
    );
}

#[test]
fn a_delete_that_isnt_stopped_runs_when_the_wait_ends() {
    let stored = in_launcher("note_delete_runs", 0, |cx| {
        let id = seed_note("Chores", &["- sweep", "- recycle"]);
        let (app, _) = launcher(cx, Jev::Off);
        open(&app, cx, NoteTarget::Note(id));
        key(&app, cx, Key::Up);
        key(&app, cx, Key::Other(DELETE_LINE_KEY.into()));
        wait(cx, 9_000);
        stored_body(id)
    });
    assert!(
        stored.contains("- sweep") && !stored.contains("- recycle"),
        "the line is gone once the wait ran out: {stored:?}"
    );
}

#[test]
fn cmd_p_lists_notes_by_name_and_enter_opens_one() {
    let (listed, opened) = in_launcher("notes_find", 0, |cx| {
        seed_note("Quarterly zebra plan", &["- stripes"]);
        let (app, window) = launcher(cx, Jev::Off);
        open(&app, cx, NoteTarget::New);
        key(&app, cx, Key::Other(FIND_NOTE_KEY.into()));
        wait(cx, 20);
        type_into_launcher(cx, window, &app, "zebra");
        wait(cx, 20);
        let listed = app.update(cx, |app, _| match app.one_window_hosted_view() {
            Some(HostedView::Notes(list)) => list
                .found
                .iter()
                .map(|found| found.title.clone())
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        });
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let opened = app.update(cx, |app, _| match app.one_window_hosted_view() {
            Some(HostedView::Note(note)) => Some(note.title.clone()),
            _ => None,
        });
        (listed, opened)
    });
    assert_eq!(listed, ["Quarterly zebra plan"]);
    assert_eq!(opened.as_deref(), Some("Quarterly zebra plan"));
}

#[test]
fn cmd_enter_hands_the_note_to_the_model_under_the_same_input() {
    let (top, windows) = in_launcher("note_to_model", 0, |cx| {
        let id = seed_note("Pricing", &["- try 12 dollars"]);
        let (app, _) = launcher(cx, Jev::Off);
        open(&app, cx, NoteTarget::Note(id));
        key(&app, cx, Key::CmdEnter);
        wait(cx, 20);
        let top = app.update(cx, |app, _| {
            app.one_window.views().top().map(|v| v.name.clone())
        });
        (top, cx.update(|cx| cx.windows().len()))
    });
    assert!(
        top.as_deref()
            .is_some_and(|name| crate::one_window::work::WORK_VIEWS.contains(&name)),
        "the model's chat opens as a view: {top:?}"
    );
    assert_eq!(windows, 1, "no window opens");
}

/// Points the kit, and the notes brain the note view reads, at one temporary folder.
struct OneBrain {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: [(&'static str, Option<String>); 2],
    _dir: tempfile::TempDir,
}

impl OneBrain {
    const NOTES_BRAIN: &'static str = "SCRIPT_KIT_TEST_NOTES_BRAIN_PATH";

    fn new() -> Self {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let dir = tempfile::tempdir().expect("kit folder");
        let previous = [crate::setup::SK_PATH_ENV, Self::NOTES_BRAIN]
            .map(|name| (name, std::env::var(name).ok()));
        std::env::set_var(crate::setup::SK_PATH_ENV, dir.path());
        std::env::set_var(Self::NOTES_BRAIN, dir.path().join("brain"));
        Self {
            _lock: lock,
            previous,
            _dir: dir,
        }
    }
}

impl Drop for OneBrain {
    fn drop(&mut self) {
        for (name, value) in &self.previous {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}

#[test]
fn dictating_to_notes_lands_the_words_in_todays_note_under_the_input() {
    let (frozen, line, rows, windows) = in_launcher("dictation_to_notes_editor", 0, |cx| {
        let brain = OneBrain::new();
        let (app, _) = launcher(cx, Jev::Off);
        let frozen = app.update(cx, |app, cx| {
            app.capture_dictation_target_selection(
                crate::dictation::DictationTarget::NotesEditor,
                cx,
            )
            .map(|selection| selection.destination)
        });
        app.update(cx, |app, cx| {
            app.handle_dictation_started(
                crate::DictationBuiltinAction::Notes,
                crate::dictation::DictationTarget::NotesEditor,
                cx,
            )
        });
        wait(cx, 20);
        app.update(cx, |app, cx| {
            app.one_window_talk_final("pick up milk on the way home".into(), None, cx)
        });
        wait(cx, 20);
        let line = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock()).title
        });
        open(&app, cx, NoteTarget::Today);
        let rows = note_rows(&app, cx);
        let windows = cx.update(|cx| cx.windows().len());
        drop(brain);
        (frozen, line, rows, windows)
    });
    assert!(
        matches!(
            frozen,
            Ok(crate::dictation::FrozenDictationDestination::DayPage { .. })
        ),
        "a Notes target freezes today's note: {frozen:?}"
    );
    assert_eq!(
        line,
        "Added to today's note: “pick up milk on the way home”"
    );
    assert_eq!(
        rows,
        ["pick up milk on the way home", "Write a line…"],
        "the words are a line in today's note under the main input"
    );
    assert_eq!(windows, 1, "no window opens");
}

#[test]
fn esc_from_a_note_found_by_name_goes_back_to_the_launcher_with_nothing_half_drawn() {
    let (in_view, hosted, input) = in_launcher("notes_find_then_esc", 0, |cx| {
        seed_note("Quarterly otter plan", &["- whiskers"]);
        let (app, window) = launcher(cx, Jev::Off);
        open(&app, cx, NoteTarget::New);
        key(&app, cx, Key::Other(FIND_NOTE_KEY.into()));
        wait(cx, 20);
        type_into_launcher(cx, window, &app, "otter");
        wait(cx, 20);
        key(&app, cx, Key::Enter);
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Escape);
        wait(cx, 20);
        app.update(cx, |app, _| {
            (
                app.one_window.in_view(),
                app.one_window_hosted_view().is_some(),
                app.filter_text.clone(),
            )
        })
    });
    assert!(
        !in_view,
        "Esc leaves the note rather than a list nothing draws"
    );
    assert!(!hosted);
    assert_eq!(
        input, "",
        "the input has the words from before the note opened"
    );
}
