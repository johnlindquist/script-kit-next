//! `@` context as chips at the main input's start, through the real launcher: a finished token
//! leaves the words and becomes a chip, the sentence everything reads still carries the token,
//! and ⌫ / ← at the input's start reach the chips.

use super::*;
use crate::one_window::chips::{compose, finished_mention, spine_caret};

fn chip_key(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    app: &gpui::Entity<ScriptListApp>,
    key: &str,
) -> bool {
    let taken = cx.update(|cx| {
        window
            .update(cx, |_, window, cx| {
                app.update(cx, |app, cx| {
                    app.one_window_chip_key(key, &gpui::Modifiers::default(), window, cx)
                })
            })
            .expect("the test launcher window")
    });
    wait(cx, 20);
    taken
}

/// The input's words, the sentence, and each chip's title.
fn read(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext) -> (String, String, Vec<String>) {
    app.update(cx, |app, cx| {
        (
            app.gpui_input_state.read(cx).value().to_string(),
            app.filter_text.clone(),
            app.one_window
                .context()
                .iter()
                .map(|chip| chip.title.clone())
                .collect(),
        )
    })
}

#[test]
fn a_finished_mention_becomes_a_chip_and_the_sentence_keeps_its_token() {
    let (typed, cleared) = in_launcher("mention_becomes_chip", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "summarize @selection ");
        wait(cx, 20);
        let typed = read(&app, cx);
        // ⌫ with the caret at the input's start deletes the chip; the words stay.
        cx.update(|cx| {
            window
                .update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.gpui_input_state
                            .update(cx, |input, cx| input.set_selection_focused(0, 0, window, cx));
                    })
                })
                .expect("the test launcher window")
        });
        assert!(chip_key(cx, window, &app, "backspace"), "⌫ at the start goes to the chip");
        (typed, read(&app, cx))
    });
    assert_eq!(typed.0, "summarize ", "the input shows only the words");
    assert_eq!(typed.1, "summarize @selection ", "the sentence reads as typed");
    assert_eq!(typed.2.len(), 1, "one chip: {:?}", typed.2);
    assert_eq!(cleared, ("summarize ".into(), "summarize ".into(), Vec::new()));
}

#[test]
fn left_walks_back_over_the_chips_and_backspace_deletes_the_one_it_stands_on() {
    let (after_left, after_delete) = in_launcher("left_over_chips", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "@selection ");
        type_into_launcher(cx, window, &app, "@clipboard ");
        wait(cx, 20);
        assert_eq!(read(&app, cx).2.len(), 2, "two chips");
        assert!(chip_key(cx, window, &app, "left"), "← at the start goes to the last chip");
        assert!(chip_key(cx, window, &app, "left"), "← again moves to the first chip");
        let after_left = app.update(cx, |app, _| app.one_window.chip_at());
        assert!(chip_key(cx, window, &app, "backspace"), "⌫ deletes the highlighted chip");
        (after_left, read(&app, cx))
    });
    assert_eq!(after_left, Some(0), "← stops on the first chip");
    assert_eq!(after_delete.0, "", "no words were typed");
    assert_eq!(after_delete.1, "@clipboard ", "the other chip's token stays in the sentence");
    assert_eq!(after_delete.2.len(), 1, "{:?}", after_delete.2);
}

#[test]
fn keys_away_from_the_start_stay_in_the_words() {
    let taken = in_launcher("keys_in_words", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "@selection ");
        type_into_launcher(cx, window, &app, "hi");
        wait(cx, 20);
        chip_key(cx, window, &app, "backspace") || chip_key(cx, window, &app, "left")
    });
    assert!(!taken, "with the caret after words, ⌫ and ← edit the words");
}

/// Next to a chip, a half-typed `@cl` is still what the spine completes (its caret is the end
/// of the words, not of the chips' tokens), and the completion `@clipboard` becomes a chip even
/// though the chips' tokens already supply the space after it.
#[test]
fn a_mention_next_to_chips_completes_into_a_chip() {
    let (active, completed, after) = in_launcher("mention_next_to_chips", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "summarize @selection ");
        type_into_launcher(cx, window, &app, "@cl");
        wait(cx, 20);
        let active = app.update(cx, |app, _| {
            let projection = app.spine_projection.clone().expect("the spine projects");
            app.spine_parse
                .segments
                .get(projection.active_segment_index)
                .map(|segment| segment.raw.clone())
        });
        // What a completion row does: rewrite the segment in the sentence, caret after it.
        let sentence = "summarize @clipboard @selection ".to_string();
        let caret = "summarize @clipboard".len();
        cx.update(|cx| {
            window
                .update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.set_filter_text_and_cursor_immediate(sentence, caret, window, cx)
                    })
                })
                .expect("the test launcher window")
        });
        wait(cx, 20);
        // Once completed, no chip's token is the segment the list answers.
        let after = app.update(cx, |app, _| {
            let projection = app.spine_projection.clone().expect("the spine projects");
            app.spine_parse
                .segments
                .get(projection.active_segment_index)
                .map(|segment| segment.raw.clone())
        });
        (active, read(&app, cx), after)
    });
    assert_eq!(active.as_deref(), Some("@cl"), "the half-typed mention is the active segment");
    assert_eq!(completed.0, "summarize ", "the completed mention left the words");
    assert_eq!(completed.1, "summarize @selection @clipboard ");
    assert_eq!(completed.2.len(), 2, "{:?}", completed.2);
    assert_eq!(after, None, "the list lets go of the completed mention");
}


#[test]
fn a_token_is_finished_once_a_space_follows_it() {
    let text = "summarize @selection ";
    assert_eq!(finished_mention(text, text.len()), Some(10..21));
    assert_eq!(finished_mention("summarize @selection", 20), None, "still typing it");
    assert_eq!(finished_mention("@clipboard this", 11), Some(0..11));
    assert_eq!(finished_mention("mail me@x.com ", 14), None, "an address isn't a token");
    assert_eq!(finished_mention("summarize @ ", 12), None);
    assert_eq!(finished_mention("summarize this ", 15), None);
}

#[test]
fn the_sentence_is_the_words_then_the_tokens() {
    assert_eq!(compose("summarize", "@selection "), "summarize @selection ");
    assert_eq!(compose("summarize ", "@selection "), "summarize @selection ");
    assert_eq!(compose("", "@selection @clipboard "), "@selection @clipboard ");
    assert_eq!(compose("summarize", ""), "summarize");
}

#[test]
fn the_spine_reads_the_word_being_typed_never_a_chip_token() {
    // Finished words: the caret stands after the tokens, as if they had been typed last.
    let sentence = "summarize @selection ";
    assert_eq!(spine_caret("summarize ", 10, sentence), sentence.len());
    assert_eq!(spine_caret("", 0, "@selection "), "@selection ".len());
    // A word still being typed keeps the caret on it, short of the tokens.
    assert_eq!(spine_caret("summarize @cl", 13, "summarize @cl @selection "), 13);
    // A caret inside the words stays there.
    assert_eq!(spine_caret("summarize ", 3, sentence), 3);
    // No chips: the words are the sentence and the caret is unchanged.
    assert_eq!(spine_caret("tile left", 9, "tile left"), 9);
}
