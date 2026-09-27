//! Rules for the launcher's list and Enter button while typing, checked against generated
//! typing sessions through the real `ScriptListApp` and its one-window reducer.
//!
//! Each case builds a fresh launcher and types a generated sentence through the real input
//! handler, pausing between keystrokes for generated intervals: some shorter than the 16 ms
//! filter timer, some exactly on it, some longer. The list and footer are read after every
//! keystroke and every pause. A failure shrinks to the shortest session that still breaks the
//! rule and prints the `SEED=<n>` that replays it.

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gpui::proptest::prelude::*;
use gpui::proptest::test_runner::{TestCaseError, TestCaseResult};
use sk_one_window::contract::{words, ENTER_GUARD_MS};
use sk_one_window::{Item, Key};

use super::{footer_frontmost_app_name, main_window_result_action_label};
use crate::footer_popup::FooterAction;
use crate::intent::jev::{Answer, Answers, JevResult, Probabilities, Question, Questions};
use crate::intent::{JevError, Oracle};
use crate::list_item::GroupedListItem;
use crate::test_utils::property::assert_rule;
use crate::{AppView, ResolvedMainMenuSelection, ScriptListApp};

/// Words from the question in the bug report, words that land on fixture commands with
/// different Enter actions, and "lunch", which only fuzzily matches the built-in launchers.
const WORDS: &[&str] = &[
    "When", "does", "LeBron", "James", "play", "next?", "Where", "is", "the", "game", "Play",
    "Next", "rain", "tonight", "lunch",
];

/// Milliseconds between keystrokes. 16 is exactly the filter timer.
const PAUSES_MS: &[u64] = &[0, 4, 10, 16, 24, 60];

/// Pauses long enough for Jev to answer mid-sentence, not just at the end.
const READING_PAUSES_MS: &[u64] = &[0, 4, 16, 60, 150, 300];

/// Fixture commands, each with its own Enter action so any change of the highlighted row shows
/// up as a change of label.
const COMMANDS: &[(&str, &str)] = &[
    ("When Is Trash Day", "Check Schedule"),
    ("Where Am I", "Show Location"),
    ("Does It Rain", "Get Forecast"),
    ("LeBron Highlights", "Watch Highlights"),
    ("James Bond Quotes", "Copy Quote"),
    ("Play Next Track", "Play"),
    ("Next Game Tonight", "Show Game"),
    ("The Game Plan", "Open Plan"),
];

#[derive(Clone, Copy)]
enum Step {
    Type(char),
    Backspace,
    Pause(u64),
}

#[derive(Clone)]
struct Session {
    steps: Vec<Step>,
    scheduler_seed: u64,
}

/// Prints a session as a timeline: `W 0ms h 16ms ⌫ 4ms`.
impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timeline: Vec<String> = self
            .steps
            .iter()
            .map(|step| match step {
                Step::Type(' ') => "␣".to_string(),
                Step::Type(ch) => ch.to_string(),
                Step::Backspace => "⌫".to_string(),
                Step::Pause(ms) => format!("{ms}ms"),
            })
            .collect();
        write!(
            f,
            "{} (scheduler seed {})",
            timeline.join(" "),
            self.scheduler_seed
        )
    }
}

fn session() -> impl Strategy<Value = Session> {
    session_with(PAUSES_MS)
}

fn reading_session() -> impl Strategy<Value = Session> {
    session_with(READING_PAUSES_MS)
}

fn session_with(pauses: &'static [u64]) -> impl Strategy<Value = Session> {
    (
        prop::collection::vec(prop::sample::select(WORDS), 1..6),
        prop::collection::vec(prop::sample::select(pauses), 1..10),
        prop::collection::vec(any::<bool>(), 0..6),
        any::<u64>(),
    )
        .prop_map(|(words, pauses, typos, scheduler_seed)| {
            let mut pause = pauses.into_iter().cycle();
            let mut steps = Vec::new();
            for (index, word) in words.iter().enumerate() {
                let typed = if index == 0 {
                    word.to_string()
                } else {
                    format!(" {word}")
                };
                for ch in typed.chars() {
                    steps.push(Step::Type(ch));
                    steps.push(Step::Pause(pause.next().unwrap_or(0)));
                }
                if typos.get(index).copied().unwrap_or(false) {
                    steps.push(Step::Type('x'));
                    steps.push(Step::Pause(pause.next().unwrap_or(0)));
                    steps.push(Step::Backspace);
                    steps.push(Step::Pause(pause.next().unwrap_or(0)));
                }
            }
            steps.push(Step::Pause(200));
            Session {
                steps,
                scheduler_seed,
            }
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EnterButton {
    label: String,
    enabled: bool,
    disabled_reason: Option<String>,
}

/// What the launcher showed after one step.
struct Frame {
    step: String,
    text: String,
    /// The text the list was last computed for; it trails `text` while a search is pending.
    list_text: String,
    /// The list's first item, when it is a header.
    first_header: Option<String>,
    /// The Enter action of a launcher row the list paints as highlighted.
    highlighted_action: Option<String>,
    button: EnterButton,
    /// The row Enter would run, as the reducer says.
    current: Option<Item>,
    current_strong: Option<bool>,
    /// Keys of the name matches the list shows, in order. The Ask AI fallback isn't one: it
    /// steps aside while Jev's own rows offer the model.
    names: Vec<String>,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>6}  {:<26} header: {:<12} button: {}{}  on {:?}",
            self.step,
            format!("{:?}", self.text),
            self.first_header.as_deref().unwrap_or("—"),
            self.button.label,
            if self.button.enabled {
                String::new()
            } else {
                format!(
                    " (greyed out: {})",
                    self.button.disabled_reason.as_deref().unwrap_or("disabled")
                )
            },
            self.current,
        )
    }
}

/// Every frame, grouped by the keystroke that preceded it. Group 0 is the launcher before the
/// first keystroke.
struct Recording {
    keystrokes: Vec<Vec<Frame>>,
}

impl Recording {
    fn frames(&self) -> impl Iterator<Item = &Frame> {
        self.keystrokes.iter().flatten()
    }

    fn transcript(&self) -> String {
        self.frames()
            .map(|frame| format!("{frame:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn fixture_command(name: &str, enter: &str) -> Arc<crate::scripts::Script> {
    let mut script = (*crate::main_menu_selection_test_script(name)).clone();
    let mut metadata = crate::metadata_parser::TypedMetadata {
        enter: Some(enter.to_string()),
        ..Default::default()
    };
    // An invalid declaration makes running a fixture stop before any process starts.
    metadata
        .extra
        .insert("sdkCapabilities".to_string(), serde_json::json!("none"));
    script.typed_metadata = Some(metadata);
    Arc::new(script)
}

fn read_frame(
    cx: &mut gpui::TestAppContext,
    app: &gpui::Entity<ScriptListApp>,
    step: String,
) -> Option<Frame> {
    app.update(cx, |app, cx| {
        if !matches!(app.current_view, AppView::ScriptList) {
            return None;
        }
        let highlighted_action = app
            .painted_main_menu_selected_subject()
            .and_then(|subject| match subject {
                ResolvedMainMenuSelection::SearchResult { result, .. }
                    if !matches!(result, crate::scripts::SearchResult::Intent(_)) =>
                {
                    Some(main_window_result_action_label(
                        result,
                        footer_frontmost_app_name().as_deref(),
                    ))
                }
                _ => None,
            });
        let footer = app.main_window_footer_config_with_cx(Some(cx))?;
        let enter = footer
            .buttons
            .iter()
            .find(|button| button.action == FooterAction::Run)?;
        let (grouped, _) = app.main_menu_result_caches.clone_grouped_results();
        let list = app.one_window.list();
        let current = list.current();
        Some(Frame {
            step,
            text: app.filter_text.clone(),
            list_text: app.computed_filter_text.clone(),
            first_header: match grouped.first() {
                Some(GroupedListItem::SectionHeader(label, _)) => Some(label.clone()),
                _ => None,
            },
            highlighted_action,
            button: EnterButton {
                label: enter.label.to_string(),
                enabled: enter.enabled,
                disabled_reason: enter.disabled_reason.as_ref().map(ToString::to_string),
            },
            current_strong: match &current {
                Some(item @ Item::Name(_)) => list.row(item).map(|row| row.is_strong()),
                _ => None,
            },
            current,
            names: list
                .names
                .iter()
                .filter(|row| !row.key.starts_with("fallback-ai:"))
                .map(|row| row.key.clone())
                .collect(),
        })
    })
}

fn type_into_launcher(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    app: &gpui::Entity<ScriptListApp>,
    text: &str,
) {
    cx.update(|cx| {
        window
            .update(cx, |_, window, cx| {
                app.update(cx, |app, cx| {
                    app.gpui_input_state.update(cx, |input, cx| {
                        input.set_value(text.to_string(), window, cx)
                    });
                    app.handle_filter_input_change(window, cx);
                });
            })
            .expect("type into the test launcher");
    });
}

fn wait(cx: &mut gpui::TestAppContext, ms: u64) {
    cx.executor().advance_clock(Duration::from_millis(ms));
    cx.run_until_parked();
}

/// Whether Jev reads the typed text, and how it answers.
#[derive(Clone, Copy, Debug)]
enum Jev {
    Off,
    Failing,
    Reading,
    TwoReadings,
}

/// A fresh launcher holding only the fixture commands, with Jev set up as asked.
fn launcher(
    cx: &mut gpui::TestAppContext,
    jev: Jev,
) -> (gpui::Entity<ScriptListApp>, gpui::AnyWindowHandle) {
    let app = crate::main_menu_selection_test_app(cx);
    let window = cx
        .update(|cx| cx.windows().into_iter().next())
        .expect("test launcher window");
    app.update(cx, |app, cx| {
        app.scripts = COMMANDS
            .iter()
            .map(|(name, enter)| fixture_command(name, enter))
            .collect();
        app.scriptlets.clear();
        app.skills.clear();
        app.apps.clear();
        app.intent_launcher = match jev {
            Jev::Off => crate::intent::IntentLauncher::off("test"),
            Jev::Failing => {
                crate::intent::IntentLauncher::with_oracle(Arc::new(Offline::default()))
            }
            Jev::Reading => crate::intent::IntentLauncher::with_oracle(Arc::new(SentenceReader {
                alternative: false,
            })),
            Jev::TwoReadings => {
                crate::intent::IntentLauncher::with_oracle(Arc::new(SentenceReader {
                    alternative: true,
                }))
            }
        };
        app.filter_text = "zzwarmup".to_string();
        app.flush_pending_main_menu_query(cx);
        app.filter_text.clear();
        app.flush_pending_main_menu_query(cx);
    });
    (app, window)
}

/// Type one session into a fresh launcher and record every frame.
fn record(cx: &mut gpui::TestAppContext, session: &Session, jev: Jev) -> Recording {
    let (app, window) = launcher(cx, jev);
    let mut recording = Recording {
        keystrokes: vec![Vec::new()],
    };
    let Some(start) = read_frame(cx, &app, "start".to_string()) else {
        return recording;
    };
    recording.keystrokes[0].push(start);

    let mut text = String::new();
    for step in &session.steps {
        let label = match *step {
            Step::Type(ch) => {
                text.push(ch);
                type_into_launcher(cx, window, &app, &text);
                recording.keystrokes.push(Vec::new());
                format!("type {}", if ch == ' ' { '␣' } else { ch })
            }
            Step::Backspace => {
                text.pop();
                type_into_launcher(cx, window, &app, &text);
                recording.keystrokes.push(Vec::new());
                "⌫".to_string()
            }
            Step::Pause(ms) => {
                wait(cx, ms);
                format!("+{ms}ms")
            }
        };
        let Some(frame) = read_frame(cx, &app, label) else {
            // The session opened another view (a bare space opens Day Page); the launcher
            // footer is no longer on screen.
            break;
        };
        recording
            .keystrokes
            .last_mut()
            .expect("a keystroke group")
            .push(frame);
    }
    recording
}

fn in_launcher<T: Send + 'static>(
    name: &'static str,
    seed: u64,
    body: impl FnOnce(&mut gpui::TestAppContext) -> T + Send + std::panic::UnwindSafe + 'static,
) -> T {
    gpui::run_test_once(
        seed,
        Box::new(move |dispatcher| {
            let mut cx = gpui::TestAppContext::build(dispatcher, Some(name));
            let out = body(&mut cx);
            cx.run_until_parked();
            cx.update(|cx| cx.quit());
            cx.run_until_parked();
            out
        }),
    )
}

fn run_session_with(session: &Session, jev: Jev) -> Recording {
    let session = session.clone();
    in_launcher(
        "enter_button_while_typing",
        session.scheduler_seed,
        move |cx| record(cx, &session, jev),
    )
}

fn broken(what: String, recording: &Recording) -> TestCaseError {
    TestCaseError::fail(format!(
        "{what}\n\nwhat the launcher showed:\n{}",
        recording.transcript()
    ))
}

fn button_names_highlighted_row(recording: &Recording) -> TestCaseResult {
    for frame in recording.frames() {
        if frame.button.disabled_reason.as_deref() == Some("No current selected result.") {
            return Err(broken(
                format!("after {:?} the Enter button is greyed out", frame.step),
                recording,
            ));
        }
        if !matches!(frame.current, Some(Item::Name(_))) {
            continue;
        }
        let Some(expected) = &frame.highlighted_action else {
            continue;
        };
        if &frame.button.label != expected {
            return Err(broken(
                format!(
                    "after {:?} the highlighted row's action is {expected:?} but the Enter button shows {:?}",
                    frame.step, frame.button
                ),
                recording,
            ));
        }
    }
    Ok(())
}

#[test]
fn the_enter_button_names_the_highlighted_rows_action() {
    assert_rule(
        "While a name is highlighted, the Enter button shows that row's action and is never greyed out for lack of a selection",
        48,
        session(),
        |session| -> TestCaseResult { button_names_highlighted_row(&run_session_with(&session, Jev::Off)) },
    );
}

fn button_changes_at_most_once_per_keystroke(recording: &Recording) -> TestCaseResult {
    let mut shown = recording.frames().next().map(|frame| frame.button.clone());
    for frames in recording.keystrokes.iter().skip(1) {
        let mut changes = 0;
        for frame in frames {
            if shown.as_ref() != Some(&frame.button) {
                changes += 1;
                shown = Some(frame.button.clone());
            }
        }
        if changes > 1 {
            let step = frames.first().map_or("?", |frame| frame.step.as_str());
            return Err(broken(
                format!("the Enter button changed {changes} times after {step:?}"),
                recording,
            ));
        }
    }
    Ok(())
}

#[test]
fn one_keystroke_changes_the_enter_button_at_most_once() {
    assert_rule(
        "One keystroke changes the Enter button at most once",
        48,
        session(),
        |session| -> TestCaseResult {
            button_changes_at_most_once_per_keystroke(&run_session_with(&session, Jev::Off))
        },
    );
}

#[test]
fn typing_never_adds_or_removes_the_first_header() {
    assert_rule(
        "Every list the launcher shows while typing starts with a header, whatever Jev does",
        32,
        reading_session(),
        |session| -> TestCaseResult {
            let recording = run_session_with(&session, Jev::Reading);
            for frame in recording.frames() {
                if frame.list_text == frame.text && frame.first_header.is_none() {
                    return Err(broken(
                        format!(
                            "after {:?} the list for {:?} starts without a header",
                            frame.step, frame.text
                        ),
                        &recording,
                    ));
                }
            }
            Ok(())
        },
    );
}

#[test]
fn weak_names_never_own_enter() {
    assert_rule(
        "Right after a keystroke, Enter never sits on a name too weak to own it",
        32,
        session(),
        |session| -> TestCaseResult {
            let recording = run_session_with(&session, Jev::Off);
            for frames in recording.keystrokes.iter().skip(1) {
                let Some(frame) = frames.first() else {
                    continue;
                };
                if frame.current_strong == Some(false) {
                    return Err(broken(
                        format!("after {:?} Enter sits on a weak name", frame.step),
                        &recording,
                    ));
                }
            }
            Ok(())
        },
    );
}

#[test]
fn a_fuzzy_near_miss_is_listed_but_jev_owns_enter() {
    let (header, target, names) = in_launcher("weak_name", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "lunch");
        wait(cx, 20);
        app.update(cx, |app, _| {
            let list = app.one_window.list();
            (
                list.header(),
                app.one_window_enter_label(),
                list.names
                    .iter()
                    .map(|row| {
                        (
                            row.title.clone(),
                            row.is_strong(),
                            row.enter_words().to_string(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
    });
    assert!(
        !names.is_empty() && names.iter().all(|(_, strong, _)| !strong),
        "only near-misses match: {names:?}"
    );
    assert_eq!(header, words::JEV, "Jev's rows come first");
    assert!(
        names.iter().all(|(_, _, says)| *says != target),
        "no near-miss owns Enter: the button says {target:?}, names {names:?}"
    );
}

#[test]
fn names_never_wait_on_jev() {
    assert_rule(
        "Right after each keystroke the name matches are the same whether Jev reads, fails or is off",
        24,
        reading_session(),
        |session| -> TestCaseResult {
            let without = run_session_with(&session, Jev::Off);
            for jev in [Jev::Reading, Jev::Failing] {
                let with = run_session_with(&session, jev);
                for (plain, read) in without.frames().zip(with.frames()) {
                    if plain.list_text == read.list_text && plain.names != read.names {
                        return Err(broken(
                            format!(
                                "after {:?} with Jev {jev:?} the names were {:?}, without Jev {:?}",
                                read.step, read.names, plain.names
                            ),
                            &with,
                        ));
                    }
                }
            }
            Ok(())
        },
    );
}

#[test]
fn enter_runs_exactly_what_the_button_said() {
    assert_rule(
        "Enter on a name runs that name, and the ↵ button said so",
        32,
        session(),
        |session| -> TestCaseResult {
            let session = session.clone();
            let (said, ran, current) = in_launcher(
                "enter_runs_what_it_said",
                session.scheduler_seed,
                move |cx| {
                    let (app, window) = launcher(cx, Jev::Off);
                    let mut text = String::new();
                    for step in &session.steps {
                        match *step {
                            Step::Type(ch) => {
                                text.push(ch);
                                type_into_launcher(cx, window, &app, &text);
                            }
                            Step::Backspace => {
                                text.pop();
                                type_into_launcher(cx, window, &app, &text);
                            }
                            Step::Pause(ms) => wait(cx, ms),
                        }
                    }
                    app.update(cx, |app, cx| {
                        if !matches!(app.current_view, AppView::ScriptList) {
                            return (None, None, None);
                        }
                        let list = app.one_window.list();
                        let current = list.current();
                        let row = current.as_ref().and_then(|item| list.row(item)).cloned();
                        if !matches!(current, Some(Item::Name(_))) {
                            return (None, None, current);
                        }
                        let said = app.one_window_enter_label();
                        let _ = app.execute_selected(cx);
                        let ran = app.one_window_last_run().cloned();
                        (Some((said, row)), ran, current)
                    })
                },
            );
            let Some((said, Some(row))) = said else {
                return Ok(());
            };
            prop_assert_eq!(&said, row.enter_words(), "the button said the row's action");
            let ran = ran
                .ok_or_else(|| TestCaseError::fail(format!("Enter on {current:?} ran nothing")))?;
            prop_assert_eq!(ran.key, row.key);
            Ok(())
        },
    );
}

/// Reads "play next" as the fixture command and anything else as a question. With an
/// alternative, the question comes first and the command is the second reading.
struct SentenceReader {
    alternative: bool,
}

/// Every reading fails, as when the network is down. Counts how often it was asked.
#[derive(Default)]
struct Offline {
    asked: AtomicUsize,
}

impl Oracle for Offline {
    fn ask(&self, _: &serde_json::Value, _: &Questions) -> Result<JevResult, JevError> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        Err(JevError::Transport("offline".into()))
    }
}

fn choice(label: &str, p: f64) -> Answer {
    Answer::Choice {
        choice: label.to_string(),
        confidence: p,
        probabilities: Probabilities::new(vec![(label.to_string(), p)]),
    }
}

fn labels(questions: &Questions, name: &str) -> Vec<String> {
    match questions.get(name) {
        Some(Question::Choice { criteria, .. }) => criteria.labels().map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

impl Oracle for SentenceReader {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError> {
        let request = state["request"].as_str().unwrap_or_default().to_lowercase();
        let command = request.contains("play next").then_some("Play Next Track");
        let mut answers = Answers::new();
        let pathways: Vec<(String, f64)> = match (command, self.alternative) {
            (Some(_), true) => vec![
                ("ask".into(), 0.6),
                ("run".into(), 0.35),
                ("none".into(), 0.05),
            ],
            (Some(_), false) => vec![("run".into(), 0.9), ("none".into(), 0.1)],
            (None, _) => vec![("ask".into(), 0.9), ("none".into(), 0.1)],
        };
        answers.insert(
            "pathway".into(),
            Answer::Choice {
                choice: pathways[0].0.clone(),
                confidence: pathways[0].1,
                probabilities: Probabilities::new(pathways),
            },
        );
        answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
        if let Some(name) = command {
            if labels(questions, "script")
                .iter()
                .any(|label| label == name)
            {
                answers.insert("script".into(), choice(name, 0.95));
            }
        }
        if let Some(longest) = labels(questions, "task")
            .into_iter()
            .max_by_key(|l| l.len())
        {
            answers.insert("task".into(), choice(&longest, 0.9));
        }
        Ok(JevResult {
            answers,
            ..JevResult::default()
        })
    }
}

#[path = "ui_window_enter_button_ledger_tests.rs"]
mod ledger;

fn key(app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext, key: Key) -> (bool, bool) {
    app.update(cx, |app, cx| {
        let handled = app.one_window_key(key, cx);
        (handled.ran, handled.ignored)
    })
}

fn last_run_key(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
) -> Option<String> {
    app.update(cx, |app, _| {
        app.one_window_last_run().map(|row| row.key.clone())
    })
}

fn jev_row_key(
    app: &gpui::Entity<ScriptListApp>,
    cx: &mut gpui::TestAppContext,
    slot: usize,
) -> Option<(String, bool)> {
    app.update(cx, |app, _| {
        app.one_window
            .list()
            .jev
            .get(slot)
            .and_then(|s| s.row.as_ref().map(|row| (row.key.clone(), s.stale)))
    })
}

#[test]
fn three_jev_rows_are_listed_from_the_first_key() {
    let slots = in_launcher("jev_rows_first_key", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Reading);
        type_into_launcher(cx, window, &app, "p");
        wait(cx, 20);
        app.update(cx, |app, _| {
            let (grouped, flat) = app.main_menu_result_caches.clone_grouped_results();
            grouped
                .iter()
                .filter_map(|item| match item {
                    GroupedListItem::Item(ix) => match flat.get(*ix) {
                        Some(crate::scripts::SearchResult::Intent(m)) => Some(m.title.clone()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
    });
    assert_eq!(slots.len(), 3, "Jev's rows: {slots:?}");
    assert_eq!(slots[0], "Understanding…");
}

#[test]
fn a_stale_jev_row_never_runs() {
    let (stale, ran_while_stale, fresh, ran) = in_launcher("stale_jev_row", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Reading);
        type_into_launcher(cx, window, &app, "play next track");
        wait(cx, 400);
        assert!(
            jev_row_key(&app, cx, 0).is_some(),
            "Jev answered the first words"
        );
        type_into_launcher(cx, window, &app, "play next track now");
        wait(cx, 20);
        let stale = jev_row_key(&app, cx, 0);
        key(&app, cx, Key::CmdDigit(1));
        let ran_while_stale = last_run_key(&app, cx);
        wait(cx, 400);
        (
            stale,
            ran_while_stale,
            jev_row_key(&app, cx, 0),
            last_run_key(&app, cx),
        )
    });
    assert!(
        stale.as_ref().is_some_and(|(_, stale)| *stale),
        "the old answer is stale: {stale:?}"
    );
    assert_eq!(
        ran_while_stale, None,
        "a row read from older words never runs"
    );
    let (fresh, fresh_stale) = fresh.expect("Jev answered the new words");
    assert!(!fresh_stale);
    assert_eq!(
        ran.as_deref(),
        Some(fresh.as_str()),
        "the held Enter ran the fresh answer"
    );
}

#[test]
fn enter_within_the_guard_after_something_finishes_is_ignored() {
    let (first, guarded, later) = in_launcher("enter_guard_after_done", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        type_into_launcher(cx, window, &app, "play");
        wait(cx, 20);
        let first = key(&app, cx, Key::Enter);
        app.update(cx, |app, cx| app.one_window_done(cx));
        wait(cx, 20);
        type_into_launcher(cx, window, &app, "play");
        wait(cx, 20);
        let guarded = key(&app, cx, Key::Enter);
        wait(cx, ENTER_GUARD_MS);
        (first, guarded, key(&app, cx, Key::Enter))
    });
    assert_eq!(first, (true, false), "Enter ran the name");
    assert_eq!(
        guarded,
        (false, true),
        "Enter right after it finished is ignored"
    );
    assert_eq!(later, (true, false), "once the guard ends Enter runs again");
}

#[test]
fn cmd_2_runs_jevs_second_row() {
    let (second, ran) = in_launcher("cmd_2", 0, |cx| {
        let (app, window) = launcher(cx, Jev::TwoReadings);
        type_into_launcher(cx, window, &app, "play next track please");
        wait(cx, 400);
        let second = jev_row_key(&app, cx, 1);
        key(&app, cx, Key::CmdDigit(2));
        (second, last_run_key(&app, cx))
    });
    let (second, _) = second.expect("Jev filled its second row");
    assert!(
        second.starts_with("jev:run:"),
        "the second reading runs the command: {second}"
    );
    assert_eq!(ran.as_deref(), Some(second.as_str()));
}

#[test]
fn cmd_2_on_a_failed_reading_asks_jev_again() {
    let asked = in_launcher("cmd_2_down", 0, |cx| {
        let oracle = Arc::new(Offline::default());
        let (app, window) = launcher(cx, Jev::Off);
        app.update(cx, |app, _| {
            app.intent_launcher = crate::intent::IntentLauncher::with_oracle(oracle.clone());
        });
        type_into_launcher(cx, window, &app, "book a table");
        wait(cx, 400);
        let before = oracle.asked.load(Ordering::SeqCst);
        key(&app, cx, Key::CmdDigit(2));
        wait(cx, 400);
        (before, oracle.asked.load(Ordering::SeqCst))
    });
    assert_eq!(
        asked.1,
        asked.0 + 1,
        "row 2 of a failed reading asks Jev again"
    );
}

#[test]
fn without_a_sentence_cmd_enter_stays_agent() {
    let (typed, empty) = in_launcher("cmd_enter_footer", 0, |cx| {
        let (app, window) = launcher(cx, Jev::Off);
        let label = |app: &gpui::Entity<ScriptListApp>, cx: &mut gpui::TestAppContext| {
            app.update(cx, |app, cx| {
                app.main_window_footer_config_with_cx(Some(cx))
                    .and_then(|footer| {
                        footer
                            .buttons
                            .iter()
                            .find(|button| button.action == FooterAction::Ai)
                            .map(|button| button.label.to_string())
                    })
            })
        };
        let empty = label(&app, cx);
        type_into_launcher(cx, window, &app, "where is the game tonight");
        wait(cx, 20);
        (label(&app, cx), empty)
    });
    assert_eq!(empty.as_deref(), Some("Agent"));
    assert_eq!(typed.as_deref(), Some("Do It"));
}

#[path = "ui_window_enter_button_property_tests_one_window.rs"]
mod one_window;

#[path = "ui_window_one_window_work_tests.rs"]
mod work;

#[path = "ui_window_one_window_chat_tests.rs"]
mod chat;

#[path = "ui_window_one_window_learns_tests.rs"]
mod learns;

#[path = "ui_window_enter_button_property_tests_views.rs"]
mod views;

#[path = "ui_window_enter_button_property_tests_notes.rs"]
mod notes;

#[path = "ui_window_enter_button_property_tests_staging.rs"]
mod staging;
