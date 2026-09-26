//! Rules for the launcher's Enter button while typing, checked against
//! generated typing sessions.
//!
//! Each case builds a fresh launcher and types a generated sentence through
//! the real input handler, pausing between keystrokes for generated
//! intervals: some shorter than the 16 ms filter timer, some exactly on it,
//! some longer. The footer is read after every keystroke and every pause. A
//! failure shrinks to the shortest session that still breaks the rule and
//! prints the `SEED=<n>` that replays it.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use gpui::proptest::prelude::*;
use gpui::proptest::test_runner::{TestCaseError, TestCaseResult};

use super::{footer_frontmost_app_name, main_window_result_action_label};
use crate::footer_popup::FooterAction;
use crate::test_utils::property::assert_rule;
use crate::{AppView, ResolvedMainMenuSelection, ScriptListApp};

/// Words from the question in the bug report, plus words that land on
/// fixture commands with different Enter actions.
const WORDS: &[&str] = &[
    "When", "does", "LeBron", "James", "play", "next?", "Where", "is", "the", "game", "Play",
    "Next", "rain", "tonight",
];

/// Milliseconds between keystrokes. 16 is exactly the filter timer.
const PAUSES_MS: &[u64] = &[0, 4, 10, 16, 24, 60];

/// Fixture commands, each with its own Enter action so any change of the
/// highlighted row shows up as a change of label.
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

/// The intent row as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct IntentSeen {
    /// Position among the rows the highlight can land on.
    index: usize,
    request: String,
    title: String,
    verb: String,
    /// Still the placeholder: Jev hasn't answered.
    reading: bool,
}

/// What the launcher showed after one step.
struct Frame {
    step: String,
    text: String,
    /// The text the list was last computed for; it trails `text` while a search is pending.
    list_text: String,
    /// The Enter action of the row the list is painting as highlighted.
    highlighted_action: Option<String>,
    highlighted_key: Option<String>,
    /// Keys of the rows the highlight can land on, top to bottom.
    rows: Vec<String>,
    button: EnterButton,
    intent: Option<IntentSeen>,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>6}  {:<26} highlighted: {:<18} button: {}{}",
            self.step,
            format!("{:?}", self.text),
            self.highlighted_action.as_deref().unwrap_or("—"),
            self.button.label,
            if self.button.enabled {
                String::new()
            } else {
                format!(
                    " (greyed out: {})",
                    self.button.disabled_reason.as_deref().unwrap_or("disabled")
                )
            }
        )?;
        if let Some(intent) = &self.intent {
            if intent.reading {
                write!(f, "  row {}: reading {:?}", intent.index, intent.request)?;
            } else {
                write!(
                    f,
                    "  row {}: {:?} read from {:?}",
                    intent.index, intent.title, intent.request
                )?;
            }
        }
        Ok(())
    }
}

/// Every frame, grouped by the keystroke that preceded it. Group 0 is the
/// launcher before the first keystroke.
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
    script.typed_metadata = Some(crate::metadata_parser::TypedMetadata {
        enter: Some(enter.to_string()),
        ..Default::default()
    });
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
        let (highlighted_action, highlighted_key) = app
            .painted_main_menu_selected_subject()
            .map(|subject| match subject {
                ResolvedMainMenuSelection::SearchResult { row, result } => (
                    main_window_result_action_label(result, footer_frontmost_app_name().as_deref()),
                    row.stable_key.clone(),
                ),
                ResolvedMainMenuSelection::Calculator { row, .. } => {
                    ("Copy".to_string(), row.stable_key.clone())
                }
            })
            .unzip();
        let rows: Vec<String> = app
            .main_menu_committed_rows()
            .iter()
            .filter(|row| row.eligibility.selectable)
            .map(|row| row.stable_key.clone())
            .collect();
        let footer = app.main_window_footer_config_with_cx(Some(cx))?;
        let enter = footer
            .buttons
            .iter()
            .find(|button| button.action == FooterAction::Run)?;
        let (_, flat) = app.main_menu_result_caches.clone_grouped_results();
        let intent = flat.iter().find_map(|result| match result {
            crate::scripts::SearchResult::Intent(m) => Some(IntentSeen {
                index: rows
                    .iter()
                    .position(|key| key == crate::scripts::INTENT_ROW_KEY)
                    .unwrap_or(usize::MAX),
                request: m.request.clone(),
                title: m.title.clone(),
                verb: m.verb.to_string(),
                reading: m.reading,
            }),
            _ => None,
        });
        Some(Frame {
            step,
            text: app.filter_text.clone(),
            list_text: app.computed_filter_text.clone(),
            intent,
            highlighted_action,
            highlighted_key,
            rows,
            button: EnterButton {
                label: enter.label.to_string(),
                enabled: enter.enabled,
                disabled_reason: enter.disabled_reason.as_ref().map(ToString::to_string),
            },
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

/// Whether Jev reads the typed text, and how it answers.
#[derive(Clone, Copy, Debug)]
enum Jev {
    Off,
    Failing,
    Reading,
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
                crate::intent::IntentLauncher::with_oracle(Arc::new(jev_rules::Offline))
            }
            Jev::Reading => {
                crate::intent::IntentLauncher::with_oracle(Arc::new(jev_rules::SentenceReader))
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
                cx.executor().advance_clock(Duration::from_millis(ms));
                cx.run_until_parked();
                format!("+{ms}ms")
            }
        };
        let Some(frame) = read_frame(cx, &app, label) else {
            // The session opened another view (a bare space opens Day Page);
            // the launcher footer is no longer on screen.
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

/// Run one session in its own launcher and scheduler.
fn run_session(session: &Session) -> Recording {
    run_session_with(session, Jev::Off)
}

fn run_session_with(session: &Session, jev: Jev) -> Recording {
    let session = session.clone();
    gpui::run_test_once(
        session.scheduler_seed,
        Box::new(move |dispatcher| {
            let mut cx = gpui::TestAppContext::build(dispatcher, Some("enter_button_while_typing"));
            let recording = record(&mut cx, &session, jev);
            cx.run_until_parked();
            cx.update(|cx| cx.quit());
            cx.run_until_parked();
            recording
        }),
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
        let Some(expected) = &frame.highlighted_action else {
            continue;
        };
        if &frame.button.label != expected
            || frame.button.disabled_reason.as_deref() == Some("No current selected result.")
        {
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
        "While a row is highlighted, the Enter button shows that row's action and is not greyed out for lack of a selection",
        48,
        session(),
        |session| -> TestCaseResult {
            let recording = run_session(&session);
            for frame in recording.frames() {
                let Some(expected) = &frame.highlighted_action else {
                    continue;
                };
                if &frame.button.label != expected
                    || frame.button.disabled_reason.as_deref()
                        == Some("No current selected result.")
                {
                    return Err(broken(
                        format!(
                            "after {:?} the highlighted row's action is {expected:?} but the Enter button shows {:?}",
                            frame.step, frame.button
                        ),
                        &recording,
                    ));
                }
            }
            Ok(())
        },
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
            button_changes_at_most_once_per_keystroke(&run_session(&session))
        },
    );
}

/// Rules for the intent row: every sentence Jev reads lists it second, under the
/// closest match. It reads until Jev answers, then shows the plan in place, and ⌘↵
/// runs it from any row.
mod jev_rules {
    use super::*;
    use crate::intent::jev::{Answer, Answers, JevResult, Probabilities, Question, Questions};
    use crate::intent::{JevError, Oracle};

    /// Pauses long enough for Jev to answer mid-sentence, not just at the end.
    const READING_PAUSES_MS: &[u64] = &[0, 4, 16, 60, 150, 300];

    /// Every reading fails, as when the network is down.
    pub(super) struct Offline;

    impl Oracle for Offline {
        fn ask(&self, _: &serde_json::Value, _: &Questions) -> Result<JevResult, JevError> {
            Err(JevError::Transport("offline".into()))
        }
    }

    /// Reads "play next" as the fixture command, "game plan" as another, and
    /// anything else of three or more words as a question for Quick AI.
    pub(super) struct SentenceReader;

    fn choice(label: &str, p: f64) -> Answer {
        Answer::Choice {
            choice: label.to_string(),
            confidence: p,
            probabilities: Probabilities::new(vec![(label.to_string(), p)]),
        }
    }

    fn labels(questions: &Questions, name: &str) -> Vec<String> {
        match questions.get(name) {
            Some(Question::Choice { criteria, .. }) => {
                criteria.labels().map(str::to_string).collect()
            }
            _ => Vec::new(),
        }
    }

    impl Oracle for SentenceReader {
        fn ask(
            &self,
            state: &serde_json::Value,
            questions: &Questions,
        ) -> Result<JevResult, JevError> {
            let request = state["request"].as_str().unwrap_or_default().to_lowercase();
            let words = request.split_whitespace().count();
            let command = if request.contains("play next") {
                Some("Play Next Track")
            } else if request.contains("game plan") {
                Some("The Game Plan")
            } else {
                None
            };
            let mut answers = Answers::new();
            let pathway = if command.is_some() { "run" } else { "ask" };
            answers.insert(
                "pathway".into(),
                Answer::Choice {
                    choice: pathway.into(),
                    confidence: 0.9,
                    probabilities: Probabilities::new(vec![
                        (pathway.to_string(), 0.9),
                        ("none".to_string(), 0.1),
                    ]),
                },
            );
            answers.insert(
                "complete".into(),
                Answer::Noul {
                    noul: if words >= 3 { 0.9 } else { 0.2 },
                },
            );
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

    fn reading_session() -> impl Strategy<Value = Session> {
        session_with(READING_PAUSES_MS)
    }

    fn select_intent_row(app: &mut ScriptListApp, cx: &mut gpui::Context<ScriptListApp>) -> bool {
        let Some(index) = app
            .main_menu_committed_rows()
            .iter()
            .find(|row| row.stable_key == crate::scripts::INTENT_ROW_KEY)
            .map(|row| row.grouped_index)
        else {
            return false;
        };
        app.select_main_menu_row(index, crate::MainMenuSelectionOrigin::Keyboard, cx)
    }

    fn selected_intent(app: &mut ScriptListApp) -> Option<crate::scripts::IntentMatch> {
        match app.selected_main_list_search_result_owned()? {
            crate::scripts::SearchResult::Intent(m) => Some(m),
            _ => None,
        }
    }

    fn footer_label(
        app: &ScriptListApp,
        cx: &mut gpui::Context<ScriptListApp>,
        action: FooterAction,
    ) -> Option<String> {
        app.main_window_footer_config_with_cx(Some(cx))?
            .buttons
            .iter()
            .find(|button| button.action == action)
            .map(|button| button.label.to_string())
    }

    #[test]
    fn every_sentence_jev_reads_lists_the_intent_row_second() {
        assert_rule(
            "From the first frame that lists a sentence, the intent row is second, under the closest match, or first when nothing else matches",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                let recording = run_session_with(&session, Jev::Reading);
                for frame in recording.frames() {
                    if frame.list_text != frame.text || !crate::intent::wants_reading(&frame.text)
                    {
                        continue;
                    }
                    let expected = 1.min(frame.rows.len().saturating_sub(1));
                    let placed = frame
                        .intent
                        .as_ref()
                        .is_some_and(|intent| intent.index == expected && intent.request == frame.text);
                    if !placed {
                        return Err(broken(
                            format!(
                                "after {:?} the list for {:?} shows the intent row as {:?}, not as row {expected}",
                                frame.step, frame.text, frame.intent
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
    fn jevs_answer_never_moves_a_row_or_the_highlight() {
        assert_rule(
            "When Jev answers, every row already listed keeps its place and the highlight stays where it was",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                let recording = run_session_with(&session, Jev::Reading);
                let frames: Vec<&Frame> = recording.frames().collect();
                for pair in frames.windows(2) {
                    let (before, after) = (pair[0], pair[1]);
                    let answered = before.list_text == after.list_text
                        && before.intent.as_ref().is_some_and(|intent| intent.reading)
                        && after.intent.as_ref().is_some_and(|intent| !intent.reading);
                    if !answered {
                        continue;
                    }
                    if !after.rows.starts_with(&before.rows)
                        || before.highlighted_key != after.highlighted_key
                    {
                        return Err(broken(
                            format!(
                                "when Jev answered after {:?} the rows went from {:?} to {:?} and the highlight from {:?} to {:?}",
                                after.step,
                                before.rows,
                                after.rows,
                                before.highlighted_key,
                                after.highlighted_key
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
    fn the_intent_row_only_shows_a_reading_of_the_text_on_screen() {
        assert_rule(
            "Whenever the intent row shows Jev's words, they were read from exactly the text in the input",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                let recording = run_session_with(&session, Jev::Reading);
                for frame in recording.frames() {
                    let Some(intent) = &frame.intent else {
                        continue;
                    };
                    if !intent.reading && intent.request != frame.text {
                        return Err(broken(
                            format!(
                                "after {:?} the intent row shows {:?}, read from {:?}, while the input says {:?}",
                                frame.step, intent.title, intent.request, frame.text
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
    fn the_enter_button_names_the_highlighted_row_while_jev_reads() {
        assert_rule(
            "With Jev reading, the Enter button still shows the highlighted row's action",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                button_names_highlighted_row(&run_session_with(&session, Jev::Reading))
            },
        );
    }

    #[test]
    fn one_keystroke_changes_the_enter_button_at_most_once_while_jev_reads() {
        assert_rule(
            "With Jev reading, one keystroke changes the Enter button at most once, however Jev answers",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                button_changes_at_most_once_per_keystroke(&run_session_with(&session, Jev::Reading))
            },
        );
    }

    /// The keystroke whose text the list at keystroke `at` was built for: the latest
    /// one at or before `at` that left exactly that text in the input.
    fn list_built_at(recording: &Recording, at: usize) -> Option<usize> {
        let list_text = &recording.keystrokes[at].first()?.list_text;
        (0..=at).rev().find(|&k| {
            recording.keystrokes[k]
                .first()
                .is_some_and(|frame| &frame.text == list_text)
        })
    }

    #[test]
    fn typing_never_waits_on_jev() {
        assert_rule(
            "Right after each keystroke the list is never further behind the typed text with Jev than without it, and when both show the list for the same text, they highlight the same row",
            24,
            reading_session(),
            |session| -> TestCaseResult {
                let without = run_session_with(&session, Jev::Off);
                let with = run_session_with(&session, Jev::Reading);
                let keystrokes = without.keystrokes.len().min(with.keystrokes.len());
                for at in 1..keystrokes {
                    let (Some(plain), Some(read)) =
                        (without.keystrokes[at].first(), with.keystrokes[at].first())
                    else {
                        continue;
                    };
                    let plain_built = list_built_at(&without, at);
                    let read_built = list_built_at(&with, at);
                    let behind = read_built < plain_built;
                    let differs = read_built == plain_built
                        && (plain.highlighted_action != read.highlighted_action
                            || plain.button != read.button);
                    if behind || differs {
                        return Err(broken(
                            format!(
                                "right after {:?} the launcher showed {read:?} with Jev but {plain:?} without",
                                read.step
                            ),
                            &with,
                        ));
                    }
                }
                Ok(())
            },
        );
    }

    #[test]
    fn a_failing_jev_never_changes_what_enter_runs() {
        assert_rule(
            "With a failing Jev, every frame highlights the same row and shows the same Enter button as without Jev",
            24,
            reading_session(),
            |session| -> TestCaseResult {
                let without = run_session_with(&session, Jev::Off);
                let failing = run_session_with(&session, Jev::Failing);
                let plain: Vec<&Frame> = without.frames().collect();
                let failed: Vec<&Frame> = failing.frames().collect();
                if plain.len() != failed.len() {
                    return Err(broken(
                        format!("without Jev the launcher showed:\n{}", without.transcript()),
                        &failing,
                    ));
                }
                for (plain, failed) in plain.iter().zip(&failed) {
                    if plain.highlighted_key != failed.highlighted_key
                        || plain.button != failed.button
                    {
                        return Err(broken(
                            format!(
                                "after {:?} a failing Jev showed {failed:?} but without Jev {plain:?}",
                                failed.step
                            ),
                            &failing,
                        ));
                    }
                }
                Ok(())
            },
        );
    }

    #[test]
    fn a_reading_that_needs_more_says_what_and_runs_nothing() {
        let (intent, activatable, listed) = gpui::run_test_once(
            0,
            Box::new(|dispatcher| {
                let mut cx = gpui::TestAppContext::build(dispatcher, Some("intent_needs_more"));
                let (app, window) = launcher(&mut cx, Jev::Reading);
                type_into_launcher(&mut cx, window, &app, "game plan");
                cx.executor().advance_clock(Duration::from_millis(400));
                cx.run_until_parked();
                let seen = app.update(&mut cx, |app, cx| {
                    let listed = format!(
                        "input {:?}, list for {:?}, rows {:?}, intent row {:?}",
                        app.filter_text,
                        app.computed_filter_text,
                        app.main_menu_committed_rows()
                            .iter()
                            .map(|row| row.stable_key.clone())
                            .collect::<Vec<_>>(),
                        app.intent_launcher.row_for(&app.computed_filter_text),
                    );
                    if !select_intent_row(app, cx) {
                        return (None, true, listed);
                    }
                    let activatable = app
                        .main_menu_committed_rows()
                        .iter()
                        .find(|row| row.stable_key == crate::scripts::INTENT_ROW_KEY)
                        .is_some_and(|row| row.eligibility.activatable);
                    (selected_intent(app), activatable, listed)
                });
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                seen
            }),
        );
        let intent = intent.unwrap_or_else(|| {
            panic!("no intent row for a two-word sentence; the launcher showed {listed}")
        });
        assert!(!intent.reading, "Jev answered: {intent:?}");
        assert!(intent.plan.is_none(), "a plan that isn't ready never runs: {intent:?}");
        assert!(
            intent.subtitle.starts_with("Needs "),
            "the row says what it needs: {intent:?}"
        );
        assert!(!activatable, "Enter can't land on a plan that isn't ready");
    }

    #[test]
    fn the_intent_row_offers_save_as_script_on_cmd_s() {
        let (plan, actions) = gpui::run_test_once(
            0,
            Box::new(|dispatcher| {
                let mut cx = gpui::TestAppContext::build(dispatcher, Some("intent_actions"));
                let (app, window) = launcher(&mut cx, Jev::Reading);
                type_into_launcher(&mut cx, window, &app, "where is the game tonight");
                cx.executor().advance_clock(Duration::from_millis(400));
                cx.run_until_parked();
                let seen = app.update(&mut cx, |app, cx| {
                    select_intent_row(app, cx);
                    let plan = selected_intent(app).and_then(|m| m.plan);
                    let actions: Vec<(String, Option<String>)> = app
                        .get_focused_script_info()
                        .map(|info| crate::actions::get_script_context_actions(&info))
                        .unwrap_or_default()
                        .into_iter()
                        .map(|action| (action.id.clone(), action.shortcut.clone()))
                        .collect();
                    (plan, actions)
                });
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                seen
            }),
        );
        assert!(plan.is_some(), "the highlighted intent row carries a ready plan");
        let save = actions.iter().find(|(id, _)| id == "save_intent_as_script");
        assert_eq!(
            save.and_then(|(_, shortcut)| shortcut.as_deref()),
            Some("⌘S"),
            "actions offered: {actions:?}"
        );
        assert!(
            !actions
                .iter()
                .any(|(id, _)| id.contains("alias") || id.contains("favorite")),
            "a sentence has no script preferences: {actions:?}"
        );
    }

    #[test]
    fn a_row_the_user_picked_keeps_the_highlight_when_jev_answers() {
        let (picked, highlighted, intent_index) = gpui::run_test_once(
            0,
            Box::new(|dispatcher| {
                let mut cx = gpui::TestAppContext::build(dispatcher, Some("intent_picked_row"));
                let (app, window) = launcher(&mut cx, Jev::Reading);
                type_into_launcher(&mut cx, window, &app, "where is the game tonight");
                cx.executor().advance_clock(Duration::from_millis(20));
                cx.run_until_parked();
                let picked = app.update(&mut cx, |app, cx| {
                    let (index, key) = app
                        .main_menu_committed_rows()
                        .iter()
                        .filter(|row| row.eligibility.selectable)
                        .filter(|row| row.stable_key != crate::scripts::INTENT_ROW_KEY)
                        .nth(1)
                        .map(|row| (row.grouped_index, row.stable_key.clone()))?;
                    app.select_main_menu_row(index, crate::MainMenuSelectionOrigin::Keyboard, cx)
                        .then_some(key)
                });
                cx.executor().advance_clock(Duration::from_millis(400));
                cx.run_until_parked();
                let seen = app.update(&mut cx, |app, _| {
                    let highlighted =
                        app.resolved_main_menu_selected_subject()
                            .map(|subject| match subject {
                                ResolvedMainMenuSelection::SearchResult { row, .. }
                                | ResolvedMainMenuSelection::Calculator { row, .. } => {
                                    row.stable_key.clone()
                                }
                            });
                    let intent_index = app
                        .main_menu_committed_rows()
                        .iter()
                        .filter(|row| row.eligibility.selectable)
                        .position(|row| row.stable_key == crate::scripts::INTENT_ROW_KEY);
                    (highlighted, intent_index)
                });
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                (picked, seen.0, seen.1)
            }),
        );
        let picked = picked.expect("the sentence lists at least three rows to pick from");
        assert_eq!(intent_index, Some(1), "the intent row is second");
        assert_eq!(highlighted.as_deref(), Some(picked.as_str()));
    }

    #[test]
    fn cmd_enter_runs_the_sentence_from_whatever_row_is_highlighted() {
        let seen = gpui::run_test_once(
            0,
            Box::new(|dispatcher| {
                let mut cx = gpui::TestAppContext::build(dispatcher, Some("intent_cmd_enter"));
                let (app, window) = launcher(&mut cx, Jev::Reading);
                type_into_launcher(&mut cx, window, &app, "where is the game tonight");
                cx.executor().advance_clock(Duration::from_millis(20));
                cx.run_until_parked();
                let seen = app.update(&mut cx, |app, cx| {
                    let before = app.resolved_main_menu_selected_subject().map(|subject| {
                        match subject {
                            ResolvedMainMenuSelection::SearchResult { row, .. }
                            | ResolvedMainMenuSelection::Calculator { row, .. } => {
                                row.stable_key.clone()
                            }
                        }
                    });
                    let label = footer_label(app, cx, FooterAction::Ai);
                    let routed = app.route_main_window_cmd_enter(cx);
                    let after = app.resolved_main_menu_selected_subject().map(|subject| {
                        match subject {
                            ResolvedMainMenuSelection::SearchResult { row, .. }
                            | ResolvedMainMenuSelection::Calculator { row, .. } => {
                                row.stable_key.clone()
                            }
                        }
                    });
                    let held = app.intent_launcher.held_enter().is_some();
                    (before, label, routed, after, held)
                });
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                seen
            }),
        );
        let (before, label, routed, after, held) = seen;
        assert_ne!(
            before.as_deref(),
            Some(crate::scripts::INTENT_ROW_KEY),
            "the closest match starts highlighted"
        );
        assert_eq!(label.as_deref(), Some("Do It"), "the footer names what ⌘↵ does");
        assert!(routed, "⌘↵ runs the sentence instead of opening Agent Chat");
        assert_eq!(
            after.as_deref(),
            Some(crate::scripts::INTENT_ROW_KEY),
            "⌘↵ shows the row it runs"
        );
        assert!(held, "Enter waits on the intent row while Jev reads");
    }

    #[test]
    fn without_jev_cmd_enter_stays_agent() {
        let label = gpui::run_test_once(
            0,
            Box::new(|dispatcher| {
                let mut cx = gpui::TestAppContext::build(dispatcher, Some("intent_off_footer"));
                let (app, window) = launcher(&mut cx, Jev::Off);
                type_into_launcher(&mut cx, window, &app, "where is the game tonight");
                cx.executor().advance_clock(Duration::from_millis(400));
                cx.run_until_parked();
                let label = app.update(&mut cx, |app, cx| footer_label(app, cx, FooterAction::Ai));
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                label
            }),
        );
        assert_eq!(label.as_deref(), Some("Agent"));
    }
}
