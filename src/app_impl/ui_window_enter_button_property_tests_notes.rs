//! Notes in the main window, through the real `ScriptListApp`: every note opens in the one editor
//! that fills the window, the Notes list lists notes pinned first and opens one, `[[links]]` move
//! between notes and Esc walks back, and moving a note to the trash waits in the line above the
//! input. No window opens.

use sk_one_window::contract::{ENTER_GUARD_MS, MESSAGE_HOLD_MS};
use sk_one_window::Key;

use super::{in_launcher, key, launcher, wait, Jev};
use crate::notes::{Note, NoteId, NoteTarget};
use crate::one_window::notes::{order_found, Found, NoteDoc, NotesList, NotesSort};
use crate::one_window::views::HostedView;
use crate::{AppView, DayPageBack, DayPageView, ScriptListApp};

/// A note of its own for each test, so tests sharing the notes store don't meet.
fn seed_note(title: &str, body: &str) -> NoteId {
    crate::sandbox_library_notes_store();
    crate::notes::init_notes_db().expect("notes store");
    let note = Note::with_content(format!("# {title}\n{body}"));
    crate::notes::save_note(&note).expect("save the note");
    note.id
}

fn editor(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> Option<gpui::Entity<DayPageView>> {
    app.update(cx, |app, _| match &app.current_view {
        AppView::DayPage { entity } => Some(entity.clone()),
        _ => None,
    })
}

fn open_in_editor(
    app: &gpui::Entity<ScriptListApp>,
    window: gpui::AnyWindowHandle,
    cx: &mut gpui::TestAppContext,
    target: NoteTarget,
) {
    cx.update(|cx| {
        window
            .update(cx, |_, window, cx| {
                app.update(cx, |app, cx| {
                    app.open_note_in_editor(target, DayPageBack::Leave, window, cx)
                })
            })
            .expect("the test window")
            .expect("the note opens");
    });
    wait(cx, 20);
}

/// Runs `f` on the open editor with the window, as its key handlers do.
fn in_editor<T>(
    app: &gpui::Entity<ScriptListApp>,
    window: gpui::AnyWindowHandle,
    cx: &mut gpui::TestAppContext,
    f: impl FnOnce(&mut DayPageView, &mut gpui::Window, &mut gpui::Context<DayPageView>) -> T,
) -> T {
    let entity = editor(app, cx).expect("the note editor is open");
    let out = cx.update(|cx| {
        window
            .update(cx, |_, window, cx| {
                entity.update(cx, |view, cx| f(view, window, cx))
            })
            .expect("the test window")
    });
    wait(cx, 20);
    out
}

fn open_note_id(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> Option<NoteId> {
    let entity = editor(app, cx)?;
    entity.update(cx, |view, _| view.current_note_id())
}

#[test]
fn the_notes_list_puts_pinned_notes_first() {
    let titles = in_launcher("notes_list_pins", 0, |_| {
        seed_note("Quokka older", "- one");
        let pinned = seed_note("Quokka pinned", "- two");
        let mut note = crate::notes::get_note(pinned).unwrap().unwrap();
        note.is_pinned = true;
        crate::notes::save_note(&note).unwrap();
        seed_note("Quokka newest", "- three");
        NotesList::new("quokka".to_string())
            .found
            .into_iter()
            .map(|found| found.title)
            .collect::<Vec<_>>()
    });
    assert_eq!(
        titles.first().map(String::as_str),
        Some("Quokka pinned"),
        "the pinned note leads: {titles:?}"
    );
    assert_eq!(
        titles.last().map(String::as_str),
        Some("New Note “quokka”"),
        "a name no note has offers to start one: {titles:?}"
    );
}

/// A broad search past the list's length ends in "Show More" saying how many are hidden,
/// and Enter on it lists them; nothing that matched is silently cut.
#[test]
fn the_notes_list_says_how_many_more_match_and_shows_them() {
    let (first, more) = in_launcher("notes_list_more", 0, |_| {
        for n in 0..65 {
            seed_note(&format!("Wombatique {n}"), "- body");
        }
        let mut list = NotesList::new("wombatique".to_string());
        let first = list
            .found
            .iter()
            .map(|found| (found.title.clone(), found.sub.clone()))
            .collect::<Vec<_>>();
        list.sel = list
            .found
            .iter()
            .position(|found| found.doc == NoteDoc::More)
            .expect("a Show More row");
        list.show_more();
        let more = list
            .found
            .iter()
            .filter(|found| matches!(found.doc, NoteDoc::Note(_)))
            .count();
        (first, more)
    });
    assert_eq!(
        first
            .iter()
            .filter(|(title, _)| title.starts_with("Wombatique"))
            .count(),
        60
    );
    assert!(
        first.contains(&("Show 5 More".to_string(), "60 of 65 shown".to_string())),
        "{first:?}"
    );
    assert_eq!(more, 65, "Show More lists the rest");
}

fn found(title: &str, pinned: bool, changed: i64, created: i64) -> Found {
    Found {
        doc: NoteDoc::Note(NoteId::new()),
        title: title.to_string(),
        sub: String::new(),
        pinned,
        changed: chrono::DateTime::from_timestamp(changed, 0).unwrap(),
        created: chrono::DateTime::from_timestamp(created, 0).unwrap(),
    }
}

fn titles(found: &[Found]) -> Vec<&str> {
    found.iter().map(|found| found.title.as_str()).collect()
}

#[test]
fn pinned_notes_come_first_in_every_order() {
    let mut list = vec![
        found("banana", false, 30, 10),
        found("apple", false, 10, 30),
        found("zebra", true, 1, 1),
    ];
    order_found(&mut list, NotesSort::Changed, "");
    assert_eq!(titles(&list), ["zebra", "banana", "apple"]);
    order_found(&mut list, NotesSort::Title, "");
    assert_eq!(titles(&list), ["zebra", "apple", "banana"]);
    order_found(&mut list, NotesSort::Created, "");
    assert_eq!(titles(&list), ["zebra", "apple", "banana"]);
}

/// Searching, a note named exactly what's typed leads, then one starting with it, and only
/// then pinned notes — whatever order the list is in.
#[test]
fn a_typed_title_leads_pinned_notes_in_every_order() {
    let mut list = vec![
        found("Budget Archive", true, 30, 30),
        found("Q3 budget notes", true, 40, 40),
        found("Budgeting", false, 20, 20),
        found("Budget", false, 10, 10),
    ];
    for sort in [NotesSort::Changed, NotesSort::Title, NotesSort::Created] {
        order_found(&mut list, sort, " budget ");
        assert_eq!(titles(&list)[0], "Budget", "{sort:?}");
        assert_eq!(titles(&list)[3], "Q3 budget notes", "{sort:?}");
    }
}

#[test]
fn by_title_day_pages_follow_named_notes() {
    let mut list = vec![
        found("2026-09-30 · Tuesday", false, 9, 9),
        found("zebra", false, 1, 1),
    ];
    list[0].doc = NoteDoc::Day(chrono::NaiveDate::from_ymd_opt(2026, 9, 30).unwrap());
    order_found(&mut list, NotesSort::Title, "");
    assert_eq!(titles(&list), ["zebra", "2026-09-30 · Tuesday"]);
}

#[test]
fn enter_in_the_notes_list_opens_the_note_and_esc_comes_back_to_the_list() {
    let (opened, text, back_view, back_list) = in_launcher("notes_list_enter_esc", 0, |cx| {
        let _brain = OneBrain::new();
        let id = seed_note("Narwhal plan", "- tusks first");
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_open_notes_list("narwhal".into(), cx)
        });
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        let opened = open_note_id(&app, cx) == Some(id);
        let text = in_editor(&app, window, cx, |view, _, cx| {
            view.notes_editor.read(cx).content(cx)
        });
        in_editor(&app, window, cx, |view, window, cx| {
            view.escape_back(window, cx)
        });
        let (back_view, back_list) = app.update(cx, |app, _| {
            (
                matches!(app.current_view, AppView::ScriptList),
                matches!(app.one_window_hosted_view(), Some(HostedView::Notes(_))),
            )
        });
        (opened, text, back_view, back_list)
    });
    assert!(opened, "Enter opens the highlighted note in the editor");
    assert!(
        text.contains("- tusks first"),
        "the editor shows the note: {text:?}"
    );
    assert!(back_view && back_list, "Esc returns to the Notes list");
}

#[test]
fn following_a_link_opens_the_note_and_esc_walks_back() {
    let (followed, back, made) = in_launcher("notes_follow_link", 0, |cx| {
        let _brain = OneBrain::new();
        let target = seed_note("Walrus target", "- blubber");
        let source = seed_note(
            "Walrus source",
            "See [[Walrus target]] and [[Walrus nowhere 7]]",
        );
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Note(source));
        let put_cursor_in =
            |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext, title: &str| {
                in_editor(app, window, cx, |view, window, cx| {
                    let input = view.notes_editor.read(cx).input_state().clone();
                    let at = input.read(cx).value().find(title).expect("the link") + 2;
                    input.update(cx, |state, cx| {
                        state.focus(window, cx);
                        state.set_selected_range(at..at, cx)
                    });
                });
            };
        put_cursor_in(&app, cx, "Walrus target");
        let cmd = gpui::Modifiers {
            platform: true,
            ..Default::default()
        };
        in_editor(&app, window, cx, |view, window, cx| {
            view.handle_key_parts(".", cmd, window, cx)
        });
        let followed = open_note_id(&app, cx) == Some(target);
        in_editor(&app, window, cx, |view, window, cx| {
            view.escape_back(window, cx)
        });
        let back = open_note_id(&app, cx) == Some(source);
        put_cursor_in(&app, cx, "Walrus nowhere 7");
        in_editor(&app, window, cx, |view, window, cx| {
            view.activate_wiki_link_under_cursor(window, cx)
        });
        let made = open_note_id(&app, cx)
            .and_then(|id| crate::notes::get_note(id).ok().flatten())
            .map(|note| note.title);
        (followed, back, made)
    });
    assert!(followed, "the link opens the note it names");
    assert!(back, "Esc comes back to the note the link was in");
    assert_eq!(
        made.as_deref(),
        Some("Walrus nowhere 7"),
        "a link to a title no note has starts that note"
    );
}

#[test]
fn cmd_t_on_today_leaves_esc_going_out_of_the_editor() {
    let (back_label, from_note) = in_launcher("notes_cmd_t_on_today", 0, |cx| {
        let _brain = OneBrain::new();
        let note = seed_note("Heron plan", "- start");
        let (app, window) = launcher(cx, Jev::Off);
        let cmd = gpui::Modifiers {
            platform: true,
            ..Default::default()
        };
        open_in_editor(&app, window, cx, NoteTarget::Today);
        let back_label = in_editor(&app, window, cx, |view, window, cx| {
            view.handle_key_parts("t", cmd, window, cx);
            view.back_label()
        });
        open_in_editor(&app, window, cx, NoteTarget::Note(note));
        let from_note = in_editor(&app, window, cx, |view, window, cx| {
            view.handle_key_parts("t", cmd, window, cx);
            view.back_label()
        });
        (back_label, from_note)
    });
    assert_eq!(
        back_label, None,
        "⌘T on today's page stays put; Esc still leaves the editor"
    );
    assert_eq!(
        from_note.as_deref(),
        Some("Back to Heron plan"),
        "⌘T from a note still comes back to it"
    );
}

#[test]
fn leaving_an_edited_note_never_moves_the_main_data_revision_backwards() {
    let (before, after, left) = in_launcher("notes_leave_revision", 0, |cx| {
        let _brain = OneBrain::new();
        seed_note("Ibis plan", "- start");
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, cx| {
            app.one_window_open_notes_list("ibis".into(), cx)
        });
        wait(cx, ENTER_GUARD_MS);
        key(&app, cx, Key::Enter);
        wait(cx, 20);
        // Test windows don't paint, so bind the identity observers the way each render does.
        let render = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext| {
            app.update(cx, |app, cx| app.bind_owned_surface_revision_observers(cx));
            wait(cx, 20);
        };
        render(&app, cx);
        for words in ["- a", "- a\n- b", "- a\n- b\n- c", "- a\n- b\n- c\n- d"] {
            in_editor(&app, window, cx, |view, window, cx| {
                let text = format!("# Ibis plan\n{words}");
                view.notes_editor
                    .update(cx, |editor, cx| editor.set_value(text, window, cx));
            });
        }
        let data = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext| {
            cx.update(|cx| {
                let app = app.read(cx);
                app.owned_revision_facts()
                    .data_generation
                    .strict_add(app.owned_child_semantic_revision(cx))
            })
        };
        let before = data(&app, cx);
        in_editor(&app, window, cx, |view, window, cx| {
            view.escape_back(window, cx)
        });
        render(&app, cx);
        let left = editor(&app, cx).is_none();
        (before, data(&app, cx), left)
    });
    assert!(left, "Esc leaves the note");
    assert!(
        after > before,
        "leaving the note moves the data revision forward: {before} -> {after}"
    );
}

#[test]
fn esc_on_today_opened_from_the_launcher_comes_back_to_the_launcher() {
    let (opened, back) = in_launcher("notes_today_esc", 0, |cx| {
        let _brain = OneBrain::new();
        let (app, window) = launcher(cx, Jev::Off);
        // What ⌘T and the Today button run.
        cx.update(|cx| {
            window
                .update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| app.show_day_page_view(window, cx))
                })
                .expect("the test window");
        });
        wait(cx, 20);
        let opened = editor(&app, cx).is_some();
        in_editor(&app, window, cx, |view, window, cx| {
            view.escape_back(window, cx)
        });
        wait(cx, 20);
        let back = app.read_with(cx, |app, _| matches!(app.current_view, AppView::ScriptList));
        (opened, back)
    });
    assert!(opened, "⌘T opens Today");
    assert!(back, "Esc on Today goes back to the launcher it came from");
}

#[test]
fn a_note_keeps_saving_after_it_is_pinned_from_its_actions() {
    let (saved, pinned, on_disk, conflicts, shown) = in_launcher("notes_save_after_pin", 0, |cx| {
        let _brain = OneBrain::new();
        let id = seed_note("Egret plan", "- start");
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Note(id));
        let type_and_save =
            |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext, text: &str| {
                in_editor(app, window, cx, |view, window, cx| {
                    let text = text.to_string();
                    view.notes_editor
                        .update(cx, |editor, cx| editor.set_value(text, window, cx));
                    view.save(window, cx)
                })
            };
        let first = type_and_save(&app, cx, "# Egret plan\n- start\n- wade");
        let entity = editor(&app, cx).expect("the note editor");
        cx.update(|cx| {
            window
                .update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.execute_day_page_note_action(entity, "day_page:note:pin", window, cx)
                    })
                })
                .expect("the test window")
        });
        wait(cx, 20);
        // The pin rewrote the file; the editor's next look at the disk must keep only the words.
        let shown = in_editor(&app, window, cx, |view, window, cx| {
            view.last_external_poll = None;
            view.poll_external_disk_changes(window, cx);
            view.notes_editor.read(cx).content(cx)
        });
        let second = type_and_save(&app, cx, "# Egret plan\n- start\n- wade\n- fish");
        let note = crate::notes::get_note(id).unwrap().expect("the note");
        let path = crate::notes::note_file_path(id)
            .unwrap()
            .expect("the note's file");
        let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
        let conflicts = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                name.starts_with("egret-plan") && name.contains(".conflict-")
            })
            .count();
        (first && second, note.is_pinned, on_disk, conflicts, shown)
    });
    assert!(saved, "both saves succeed");
    assert!(pinned, "the note is pinned");
    assert_eq!(
        shown, "# Egret plan\n- start\n- wade",
        "the editor shows the note's words, not the file's front matter"
    );
    assert!(
        on_disk.contains("- fish"),
        "the file has the last words: {on_disk:?}"
    );
    assert_eq!(conflicts, 0, "no copy is set aside as someone else's edit");
}

/// Types into the open editor without saving, as a person does between autosaves.
fn type_into(
    app: &gpui::Entity<ScriptListApp>,
    window: gpui::AnyWindowHandle,
    cx: &mut gpui::TestAppContext,
    text: &str,
) {
    let text = text.to_string();
    in_editor(app, window, cx, |view, window, cx| {
        view.notes_editor
            .update(cx, |editor, cx| editor.set_value(text, window, cx));
        let content = view.notes_editor.read(cx).content(cx);
        view.session.apply_editor_content(&content);
    });
}

/// Whether Esc took the window out of the editor.
fn esc_leaves(
    app: &gpui::Entity<ScriptListApp>,
    window: gpui::AnyWindowHandle,
    cx: &mut gpui::TestAppContext,
) -> bool {
    in_editor(app, window, cx, |view, window, cx| {
        view.escape_back(window, cx)
    });
    editor(app, cx).is_none()
}

#[test]
fn a_line_added_in_another_app_while_typing_lands_in_the_note() {
    let (shown, on_disk, left) = in_launcher("notes_outside_edit_merges", 0, |cx| {
        let _brain = OneBrain::new();
        let id = seed_note("Plover plan", "- start");
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Note(id));
        type_into(&app, window, cx, "# Plover plan\n- start, typed here");
        let path = crate::notes::note_file_path(id)
            .unwrap()
            .expect("the note's file");
        let outside = std::fs::read_to_string(&path)
            .unwrap()
            .replace("- start", "- start\n- added in Obsidian");
        std::fs::write(&path, outside).unwrap();
        let shown = in_editor(&app, window, cx, |view, window, cx| {
            view.last_external_poll = None;
            view.poll_external_disk_changes(window, cx);
            view.save(window, cx);
            view.notes_editor.read(cx).content(cx)
        });
        let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
        (shown, on_disk, esc_leaves(&app, window, cx))
    });
    assert_eq!(
        shown, "# Plover plan\n- start, typed here\n- added in Obsidian",
        "the editor shows both people's lines"
    );
    assert!(
        on_disk.contains("- added in Obsidian") && on_disk.contains("typed here"),
        "the file keeps both: {on_disk:?}"
    );
    assert!(left, "Esc leaves the editor");
}

#[test]
fn a_note_with_unreadable_front_matter_saves_and_esc_leaves() {
    let (on_disk, left, still_open) = in_launcher("notes_broken_front_matter", 0, |cx| {
        let _brain = OneBrain::new();
        let id = seed_note("Sandpiper plan", "- start");
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Note(id));
        type_into(
            &app,
            window,
            cx,
            "---\ntags: [unclosed\n---\n# Sandpiper plan\n- start",
        );
        let left = esc_leaves(&app, window, cx);
        let path = crate::notes::note_file_path(id)
            .unwrap()
            .expect("the note's file");
        // Distinguish a pending suggestion, a failed save and a navigation refusal. Only
        // observe after a failed exit; never send a second Escape or replay a save.
        let still_open = editor(&app, cx).map(|entity| entity.update(cx, |view, cx| {
            let input = view.notes_editor.read(cx);
            serde_json::json!({
                "sameNote":view.current_note_id()==Some(id), "dirty":view.session.is_dirty(),
                "inlineCompletion":input.has_inline_completion(cx), "trailDepth":view.trail.len(),
                "saveFailure":view.save_failure, "storedMatchesEditor":view.session.verify_saved_content(&input.content(cx)).is_ok()
            })
        }));
        (
            std::fs::read_to_string(&path).unwrap_or_default(),
            left,
            still_open,
        )
    });
    assert!(left, "Esc leaves the editor: {still_open:?}");
    assert!(
        on_disk.contains("tags: [unclosed"),
        "the block is kept as typed: {on_disk:?}"
    );
}

#[test]
fn a_note_deleted_elsewhere_while_open_is_saved_again_when_you_leave() {
    let (left, kept) = in_launcher("notes_deleted_while_open", 0, |cx| {
        let _brain = OneBrain::new();
        let id = seed_note("Curlew plan", "- start");
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Note(id));
        script_kit_gpui::day_page::delete_note_permanently(&id.as_str())
            .expect("deleted elsewhere");
        type_into(&app, window, cx, "# Curlew plan\n- start\n- still here");
        let left = esc_leaves(&app, window, cx);
        let kept = crate::notes::note_file_path(id)
            .ok()
            .flatten()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();
        (left, kept)
    });
    assert!(left, "Esc leaves the editor");
    assert!(kept.contains("- still here"), "the note is back: {kept:?}");
}

#[test]
fn moving_a_note_to_the_trash_waits_in_the_line_and_then_runs() {
    let (line, kept, trashed) = in_launcher("notes_trash_hold", 0, |cx| {
        let _brain = OneBrain::new();
        let id = seed_note("Heron draft", "- wings");
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Note(id));
        let entity = editor(&app, cx).expect("the note editor");
        cx.update(|cx| {
            window
                .update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.execute_day_page_note_action(entity, "day_page:note:delete", window, cx)
                    })
                })
                .expect("the test window")
        });
        wait(cx, 20);
        let line = app.update(cx, |app, _| {
            app.one_window.slot_line(app.one_window_clock()).title
        });
        let deleted = |id| {
            crate::notes::get_note(id)
                .unwrap()
                .is_some_and(|note| note.is_deleted())
        };
        let kept = !deleted(id);
        wait(cx, MESSAGE_HOLD_MS + 1_000);
        (line, kept, deleted(id))
    });
    assert_eq!(
        line, "Move “Heron draft” to the trash",
        "it waits in the line above the input"
    );
    assert!(kept, "nothing is trashed while it waits");
    assert!(trashed, "the note is trashed once the wait runs out");
}

#[test]
fn a_new_note_left_empty_is_removed_when_you_leave_it() {
    let (opened, gone) = in_launcher("notes_empty_new", 0, |cx| {
        let _brain = OneBrain::new();
        seed_note(
            "Ibis keeps the store from being empty",
            "- so New starts a blank note",
        );
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::New);
        let opened = open_note_id(&app, cx);
        in_editor(&app, window, cx, |view, window, cx| {
            view.escape_back(window, cx)
        });
        let gone = opened.map(|id| crate::notes::get_note(id).unwrap().is_none());
        (opened.is_some(), gone)
    });
    assert!(opened, "New Note opens a note in the editor");
    assert_eq!(
        gone,
        Some(true),
        "an untouched new note leaves nothing behind"
    );
}

#[test]
fn a_new_note_is_listed_on_today_and_its_links_follow_a_rename() {
    let (listed, renamed, file_name) = in_launcher("notes_new_listed_on_today", 0, |cx| {
        let _brain = OneBrain::new();
        seed_note(
            "Osprey keeps the store from being empty",
            "- so New skips the welcome note",
        );
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::New);
        let id = open_note_id(&app, cx).expect("a new note is open");
        in_editor(&app, window, cx, |view, window, cx| {
            set_text(view, "# Kestrel trip\n\nLisbon in May", window, cx);
            view.escape_back(window, cx)
        });
        let today = |cx: &mut gpui::TestAppContext| {
            open_in_editor(&app, window, cx, NoteTarget::Today);
            let text = in_editor(&app, window, cx, |view, _, cx| text_of(view, cx));
            in_editor(&app, window, cx, |view, window, cx| {
                view.escape_back(window, cx)
            });
            text
        };
        let listed = today(cx);
        open_in_editor(&app, window, cx, NoteTarget::Note(id));
        in_editor(&app, window, cx, |view, window, cx| {
            set_text(view, "# Kestrel journey\n\nLisbon in May", window, cx);
            view.escape_back(window, cx)
        });
        let file_name = crate::notes::note_file_path(id)
            .unwrap()
            .and_then(|path| Some(path.file_name()?.to_str()?.to_string()));
        (listed, today(cx), file_name)
    });
    assert!(
        listed
            .lines()
            .any(|line| line.starts_with("- ") && line.ends_with(" [[Kestrel trip]]")),
        "leaving a new note lists it on today's page at its time: {listed:?}"
    );
    assert_eq!(file_name.as_deref(), Some("Kestrel journey.md"));
    assert!(
        renamed.contains("[[Kestrel journey]]") && !renamed.contains("[[Kestrel trip]]"),
        "the link follows the note's new name: {renamed:?}"
    );
}

fn set_text(
    view: &mut DayPageView,
    text: &str,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<DayPageView>,
) {
    view.notes_editor.update(cx, |editor, cx| {
        editor.set_value_preserving_scroll(text.to_string(), text.len(), window, cx)
    });
    view.on_editor_change(window, cx);
}

fn text_of(view: &DayPageView, cx: &gpui::App) -> String {
    view.notes_editor.read(cx).content(cx)
}

#[test]
fn a_new_note_starts_titled_with_the_time_selected_and_return_keeps_it() {
    let (opened, selected, heading, kept, saved) = in_launcher("notes_new_time_title", 0, |cx| {
        let _brain = OneBrain::new();
        seed_note(
            "Heron keeps the store from being empty",
            "- so New skips the welcome note",
        );
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::New);
        let (opened, selected, heading) = in_editor(&app, window, cx, |view, _, cx| {
            let text = text_of(view, cx);
            let selection = view.notes_editor.read(cx).selection(cx);
            let selected = text[selection].to_string();
            (text, selected, view.heading(cx))
        });
        let title = opened.trim_start_matches("# ").to_string();
        // Return with the title still selected replaces it with a line break, as an editor does.
        let kept = in_editor(&app, window, cx, |view, window, cx| {
            set_text(view, "# \n", window, cx);
            text_of(view, cx)
        });
        let id = open_note_id(&app, cx);
        in_editor(&app, window, cx, |view, window, cx| {
            set_text(view, &format!("# {title}\n\nHeron sighting"), window, cx);
            view.escape_back(window, cx)
        });
        let saved = id
            .and_then(|id| crate::notes::get_note(id).unwrap())
            .map(|note| note.content);
        (opened, selected, heading, kept, saved)
    });
    let title = opened.trim_start_matches("# ");
    assert!(
        opened.starts_with("# ") && title.contains(", ") && title.ends_with('M'),
        "a new note starts titled with the time: {opened:?}"
    );
    assert_eq!(
        selected, title,
        "the time title is selected so typing renames the note"
    );
    assert_eq!(
        heading.map(|heading| (heading.title, heading.context)),
        Some((None, "New note".to_string())),
        "the line above says it's a new note; its title is in the text"
    );
    assert_eq!(
        kept,
        format!("# {title}\n\n"),
        "Return keeps the title and starts two lines below"
    );
    assert!(
        saved.is_some_and(|content| content.contains("Heron sighting")),
        "a new note with words in it is kept"
    );
}

#[test]
fn return_after_a_typed_title_starts_two_lines_below_it() {
    let text = in_launcher("notes_title_return", 0, |cx| {
        let _brain = OneBrain::new();
        seed_note(
            "Heron keeps the store from being empty",
            "- so New skips the welcome note",
        );
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::New);
        in_editor(&app, window, cx, |view, window, cx| {
            set_text(view, "# Trip ideas", window, cx);
        });
        in_editor(&app, window, cx, |view, window, cx| {
            set_text(view, "# Trip ideas\n", window, cx);
            text_of(view, cx)
        })
    });
    assert_eq!(text, "# Trip ideas\n\n");
}

#[test]
fn today_names_its_date_above_the_text() {
    let (heading, today) = in_launcher("notes_today_heading", 0, |cx| {
        let _brain = OneBrain::new();
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Today);
        in_editor(&app, window, cx, |view, _, cx| {
            (view.heading(cx), view.local_today())
        })
    });
    let heading = heading.expect("Today has a heading");
    assert_eq!(heading.title, Some(today.format("%A, %B %-d").to_string()));
    assert_eq!(heading.context, "Today");
}

#[test]
fn the_heading_names_each_kind_of_document() {
    use script_kit_gpui::day_page::DayPageBinding;
    let date = |y, m, d| chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap();
    let today = date(2026, 10, 1);
    let day =
        |on| crate::day_page_heading(&DayPageBinding::Day, Some(on), today, false, false, None);
    let said = |heading: Option<crate::DayPageHeading>| {
        heading.map(|heading| (heading.title.unwrap_or_default(), heading.context))
    };
    assert_eq!(
        said(day(today)),
        Some(("Thursday, October 1".into(), "Today".into()))
    );
    assert_eq!(
        said(day(date(2026, 9, 30))),
        Some(("Wednesday, September 30".into(), "Yesterday".into()))
    );
    assert_eq!(
        said(day(date(2026, 9, 27))),
        Some(("Sunday, September 27".into(), "4 days ago".into()))
    );
    assert_eq!(
        said(day(date(2025, 12, 31))),
        Some(("Wednesday, December 31, 2025".into(), "274 days ago".into()))
    );

    let note = DayPageBinding::Note {
        note_id: "id".into(),
        title: "Trip ideas".into(),
        return_day_path: std::path::PathBuf::new(),
        return_day_date: today,
    };
    let note_heading = |pinned, first_line| {
        said(crate::day_page_heading(
            &note, None, today, pinned, false, first_line,
        ))
    };
    assert_eq!(
        note_heading(false, Some("# Trip ideas")),
        Some((String::new(), "Note · Trip ideas".into())),
        "a note whose text starts with its title gets only the line naming it"
    );
    assert_eq!(
        note_heading(true, Some("# Trip ideas")),
        Some((String::new(), "Pinned note · Trip ideas".into()))
    );
    assert_eq!(
        said(crate::day_page_heading(
            &note,
            None,
            today,
            true,
            true,
            Some("# Trip ideas")
        )),
        Some((String::new(), "Pinned note".into())),
        "pinning a new note says so at once"
    );
    assert_eq!(
        note_heading(false, Some("- packing list")),
        Some(("Trip ideas".into(), "Note".into())),
        "a note without a title line gets its title shown above the text"
    );
}

#[test]
fn return_on_a_lone_title_leaves_a_blank_line_and_nothing_else_changes() {
    use crate::heading_enter_fixup as fixup;
    assert_eq!(
        fixup("# Trip", "# Trip\n", 7, None),
        Some(("# Trip\n\n".into(), 8))
    );
    assert_eq!(
        fixup("# Oct 1, 9:42 PM", "# \n", 3, Some("Oct 1, 9:42 PM")),
        Some(("# Oct 1, 9:42 PM\n\n".into(), 18)),
        "Return on the selected time title keeps it"
    );
    assert_eq!(
        fixup("# Trip\n- a", "# Trip\n- a\n", 11, None),
        None,
        "only a title that is the whole note"
    );
    assert_eq!(
        fixup("# Tr", "# T\nr", 4, None),
        None,
        "a break inside the title is the person's"
    );
    assert_eq!(
        fixup("plain", "plain\n", 6, None),
        None,
        "only a `# ` title"
    );
    assert_eq!(
        fixup("# ", "# \n", 3, None),
        None,
        "an empty title isn't one"
    );
}

#[test]
fn return_after_a_timed_line_starts_the_next_one_at_the_current_time() {
    use crate::timed_line_enter_fixup as fixup;
    let page = "# Today\n- 09:12 called Sam";
    assert_eq!(
        fixup(page, &format!("{page}\n"), page.len() + 1, "14:05"),
        Some((format!("{page}\n- 14:05 "), page.len() + 9))
    );
    assert_eq!(
        fixup("09:12 old style", "09:12 old style\n", 16, "14:05"),
        Some(("09:12 old style\n- 14:05 ".into(), 24)),
        "lines written before the list format keep going too"
    );
    let empty = "- 09:12 called Sam\n- 14:05 ";
    assert_eq!(
        fixup(empty, &format!("{empty}\n"), empty.len() + 1, "14:06"),
        Some(("- 09:12 called Sam\n".into(), 19)),
        "Return on a line with only its time ends the run"
    );
    assert_eq!(
        fixup("plain words", "plain words\n", 12, "14:05"),
        None,
        "plain lines stay plain"
    );
    assert_eq!(
        fixup("- [ ] 09:12 book", "- [ ] 09:12 book\n", 17, "14:05"),
        None,
        "tasks are left alone"
    );
    assert_eq!(
        fixup("- 09:12 called Sam", "- 09:12 called\n Sam", 15, "14:05"),
        None,
        "a break inside a line is the person's"
    );
}

#[test]
fn cmd_p_on_today_opens_the_switcher() {
    let open = in_launcher("notes_cmd_p_switcher", 0, |cx| {
        let _brain = OneBrain::new();
        let (app, window) = launcher(cx, Jev::Off);
        open_in_editor(&app, window, cx, NoteTarget::Today);
        in_editor(&app, window, cx, |view, window, cx| {
            let modifiers = gpui::Modifiers {
                platform: true,
                ..Default::default()
            };
            view.handle_key_parts("p", modifiers, window, cx);
            view.is_day_switcher_open()
        })
    });
    assert!(open, "⌘P opens the switcher");
}

#[test]
fn a_link_being_typed_completes_to_the_first_matching_title() {
    let titles = |_: &str| vec!["Launch plan".to_string(), "Launch".to_string()];
    let text = "See [[lau";
    assert_eq!(
        crate::day_page_ghost_suffix(text, text.len()..text.len(), &titles).as_deref(),
        Some("nch plan]]")
    );
    let closed = "See [[lau]]";
    assert_eq!(
        crate::day_page_ghost_suffix(closed, 9..9, &titles).as_deref(),
        Some("nch plan"),
        "the brackets the editor closed still count as the end of the line"
    );
    let mid_line = "See [[lau and more";
    assert_eq!(crate::day_page_ghost_suffix(mid_line, 9..9, &titles), None);
    assert_eq!(
        crate::day_page_ghost_suffix(text, 6..9, &titles),
        None,
        "nothing over a selection"
    );
    for text in ["See [[Launch]]", "Launch", "See [["] {
        assert_eq!(
            crate::day_page_ghost_suffix(text, text.len()..text.len(), &titles),
            None,
            "{text}"
        );
    }
}

#[test]
fn a_typed_note_takes_its_title_from_the_words_or_the_first_attachment() {
    let clip = vec![("Clipboard".to_string(), "  copied text \n".to_string())];
    assert_eq!(
        crate::typed_note_content("buy milk", &clip),
        "# buy milk\n\n## Clipboard\n\ncopied text\n"
    );
    assert_eq!(
        crate::typed_note_content("", &clip),
        "# Clipboard\n\n## Clipboard\n\ncopied text\n"
    );
    assert_eq!(
        crate::typed_note_content("plan\nstep one", &[]),
        "# plan\n\nstep one\n"
    );
    assert_eq!(
        crate::context_block_text(
            "<context source=\"x\" mimeType=\"text/plain\">\nhello\nworld\n</context>"
        ),
        "hello\nworld",
        "an attachment's tags stay out of the note"
    );
}

#[test]
fn a_duplicate_names_itself_a_copy() {
    assert_eq!(
        crate::duplicate_note_content("# Launch plan\nbody"),
        "# Launch plan copy\nbody"
    );
    assert_eq!(crate::duplicate_note_content(""), "Untitled Note copy");
}

/// Points the kit, where day pages live, at one temporary folder; notes stay in the shared store.
struct OneBrain {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: Option<String>,
    _dir: tempfile::TempDir,
}

impl OneBrain {
    fn new() -> Self {
        crate::sandbox_library_notes_store();
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
            _dir: dir,
        }
    }
}

impl Drop for OneBrain {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(crate::setup::SK_PATH_ENV, value),
            None => std::env::remove_var(crate::setup::SK_PATH_ENV),
        }
    }
}

#[test]
fn dictating_to_notes_lands_the_words_in_todays_page() {
    let (frozen, line, today, windows) = in_launcher("dictation_to_notes_editor", 0, |cx| {
        let brain = OneBrain::new();
        let (app, window) = launcher(cx, Jev::Off);
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
        open_in_editor(&app, window, cx, NoteTarget::Today);
        let today = in_editor(&app, window, cx, |view, _, cx| {
            view.notes_editor.read(cx).content(cx)
        });
        let windows = cx.update(|cx| cx.windows().len());
        drop(brain);
        (frozen, line, today, windows)
    });
    assert!(
        matches!(
            frozen,
            Ok(crate::dictation::FrozenDictationDestination::DayPage { .. })
        ),
        "a Notes target freezes today's page: {frozen:?}"
    );
    assert_eq!(
        line,
        "Added to today's note: “pick up milk on the way home”"
    );
    assert!(
        today.contains("pick up milk on the way home"),
        "today's page has the words: {today:?}"
    );
    assert_eq!(windows, 1, "no window opens");
}

#[test]
fn add_it_to_todays_note_opens_today_with_the_words_added() {
    let (on_today, today, windows) = in_launcher("note_view", 0, |cx| {
        let _brain = OneBrain::new();
        let (app, window) = launcher(cx, Jev::Off);
        super::type_into_launcher(cx, window, &app, "buy oat milk");
        wait(cx, 400);
        key(&app, cx, Key::CmdDigit(3));
        wait(cx, 20);
        let on_today = editor(&app, cx).is_some() && open_note_id(&app, cx).is_none();
        let today = in_editor(&app, window, cx, |view, _, cx| {
            view.notes_editor.read(cx).content(cx)
        });
        (on_today, today, cx.update(|cx| cx.windows().len()))
    });
    assert!(
        on_today,
        "Add it to today's note opens today's page in the editor"
    );
    assert!(
        today.contains("buy oat milk"),
        "with the words added: {today:?}"
    );
    assert_eq!(windows, 1, "no Notes window opens");
}

/// Opening a kept entry from ⌘K on today's page shows it in the editor's place. Closing ⌘K
/// first asks the editor to take focus, and part of that request lands after the entry is
/// already showing; the hidden editor must not take the keys, or Esc never reaches the page.
/// The page's Escape closes the entry, leaves today's page open, and the editor types again.
#[test]
fn esc_closes_a_kept_entry_opened_from_today() {
    let (keys_while_showing, preview_after_esc, editor_typing_again, still_on_today) =
        in_launcher("notes_kept_entry_esc", 0, |cx| {
            let _brain = OneBrain::new();
            let (app, window) = launcher(cx, Jev::Off);
            open_in_editor(&app, window, cx, NoteTarget::Today);
            in_editor(&app, window, cx, |view, window, cx| {
                view.fixture_shelf_previews = Some(std::collections::HashMap::from([(
                    "kept-esc".to_string(),
                    "# Kept\n- a line".to_string(),
                )]));
                view.adopt_clipboard_shelf_from(
                    "- coffee\n09:20 [Clipboard entry](kit://clipboard-history?id=kept-esc)\n",
                );
                // What closing ⌘K does, then the row's action.
                view.focus_editor(window, cx);
                assert!(view.open_clipboard_shelf_entry("kept-esc", window, cx));
            });
            wait(cx, 100);
            let keys_while_showing = in_editor(&app, window, cx, |view, window, cx| {
                (
                    view.focus_handle.is_focused(window),
                    view.notes_editor.read(cx).is_focused(window, cx),
                )
            });
            in_editor(&app, window, cx, |view, window, cx| {
                view.handle_key_parts("escape", gpui::Modifiers::default(), window, cx);
            });
            wait(cx, 100);
            let (preview_after_esc, editor_typing_again) =
                in_editor(&app, window, cx, |view, window, cx| {
                    (
                        view.kit_resource_preview.is_some(),
                        view.notes_editor.read(cx).is_focused(window, cx),
                    )
                });
            let still_on_today = editor(&app, cx).is_some() && open_note_id(&app, cx).is_none();
            (
                keys_while_showing,
                preview_after_esc,
                editor_typing_again,
                still_on_today,
            )
        });
    assert_eq!(
        keys_while_showing,
        (true, false),
        "the page, not the hidden editor, has the keys while the entry shows"
    );
    assert!(!preview_after_esc, "Esc closes the entry");
    assert!(editor_typing_again, "the editor has the keys again");
    assert!(still_on_today, "today's page is open again");
}
