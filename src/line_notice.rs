//! Word for the line above the main window's input from code that can't reach the main window
//! (Notes, Agent Chat, Dictation, background tasks, a script's `hud()`).
//!
//! The main window installs the sink when it starts. The line says it at once while the window
//! shows, and when the window comes back if it was hidden. Nothing pops up and nothing asks
//! first: what can be taken back offers ⌘Z, and what can't waits in the line before it runs.

use std::rc::Rc;

use gpui::{App, Global};

/// Work the main window runs later: an undo, or a run whose wait ran out.
pub type Later = Box<dyn FnOnce(&mut App)>;

/// What a notice carries besides its words.
pub enum Said {
    /// Just words.
    Notice,
    /// Something happened; `Later` takes it back if the user presses ⌘Z.
    Undoable(Later),
    /// Something that can't be taken back; `Later` runs when its wait runs out untouched.
    Held(Later),
    /// A question: it goes in the line, its answers become the rows, and letting go of it runs
    /// `cancel`.
    Ask {
        detail: String,
        choices: Vec<(String, Later)>,
        cancel: Option<Later>,
    },
}

type Sink = Rc<dyn Fn(&mut App, &'static str, String, Said)>;

struct LineNoticeSink(Sink);

impl Global for LineNoticeSink {}

/// The icon beside a notice: something worked.
pub const DONE: &str = "check";
/// Something went wrong. The reducer reads this icon as a failure: the line keeps it strong and
/// says only the failure's sentence.
pub const FAILED: &str = sk_one_window::contract::FAILED_ICON;
/// Something to know.
pub const INFO: &str = "info";

pub fn install(cx: &mut App, sink: impl Fn(&mut App, &'static str, String, Said) + 'static) {
    cx.set_global(LineNoticeSink(Rc::new(sink)));
}

/// Says `text` in the line above the input, with `icon` beside it.
pub fn say(cx: &mut App, icon: &'static str, text: impl Into<String>) {
    send(cx, icon, text.into(), Said::Notice);
}

/// Says that something happened and offers ⌘Z; `undo` runs if the user takes it.
pub fn undoable(
    cx: &mut App,
    icon: &'static str,
    text: impl Into<String>,
    undo: impl FnOnce(&mut App) + 'static,
) {
    send(cx, icon, text.into(), Said::Undoable(Box::new(undo)));
}

/// Holds something irreversible in the line; `run` happens when the wait runs out untouched.
pub fn hold(
    cx: &mut App,
    icon: &'static str,
    text: impl Into<String>,
    run: impl FnOnce(&mut App) + 'static,
) {
    send(cx, icon, text.into(), Said::Held(Box::new(run)));
}

/// Asks `text` in the line; each choice is a row, and letting go of the question runs `cancel`.
pub fn ask(
    cx: &mut App,
    icon: &'static str,
    text: impl Into<String>,
    detail: impl Into<String>,
    choices: Vec<(String, Later)>,
    cancel: Option<Later>,
) {
    send(
        cx,
        icon,
        text.into(),
        Said::Ask {
            detail: detail.into(),
            choices,
            cancel,
        },
    );
}

fn send(cx: &mut App, icon: &'static str, text: String, said: Said) {
    let Some(sink) = cx.try_global::<LineNoticeSink>().map(|sink| sink.0.clone()) else {
        tracing::info!(
            target: "script_kit::line_notice",
            event = "no_main_window",
            icon,
            text_bytes = text.len(),
        );
        return;
    };
    cx.defer(move |cx| sink(cx, icon, text, said));
}
