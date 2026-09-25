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

/// The intent row pinned at the top of the list, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pinned {
    request: String,
    title: String,
    verb: String,
}

/// What the launcher showed after one step.
struct Frame {
    step: String,
    text: String,
    /// The text the list was last computed for; it trails `text` while a search is pending.
    list_text: String,
    /// The Enter action of the row the list is painting as highlighted.
    highlighted_action: Option<String>,
    button: EnterButton,
    pinned: Option<Pinned>,
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
        if let Some(pinned) = &self.pinned {
            write!(
                f,
                "  pinned: {:?} read from {:?}",
                pinned.title, pinned.request
            )?;
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
        let highlighted_action =
            app.painted_main_menu_selected_subject()
                .map(|subject| match subject {
                    ResolvedMainMenuSelection::SearchResult { result, .. } => {
                        main_window_result_action_label(
                            result,
                            footer_frontmost_app_name().as_deref(),
                        )
                    }
                    ResolvedMainMenuSelection::Calculator { .. } => "Copy".to_string(),
                });
        let footer = app.main_window_footer_config_with_cx(Some(cx))?;
        let enter = footer
            .buttons
            .iter()
            .find(|button| button.action == FooterAction::Run)?;
        let (_, flat) = app.main_menu_result_caches.clone_grouped_results();
        let pinned = flat.iter().find_map(|result| match result {
            crate::scripts::SearchResult::Intent(m) => Some(Pinned {
                request: m.plan.request.clone(),
                title: m.plan.title.clone(),
                verb: m.verb.to_string(),
            }),
            _ => None,
        });
        Some(Frame {
            step,
            text: app.filter_text.clone(),
            list_text: app.computed_filter_text.clone(),
            pinned,
            highlighted_action,
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

#[test]
fn one_keystroke_changes_the_enter_button_at_most_once() {
    assert_rule(
        "One keystroke changes the Enter button at most once",
        48,
        session(),
        |session| -> TestCaseResult {
            let recording = run_session(&session);
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
                        &recording,
                    ));
                }
            }
            Ok(())
        },
    );
}

/// Rules for the intent row: Jev reads the sentence while it is typed, and a
/// ready plan for exactly the current text is pinned first.
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

    #[test]
    fn a_pinned_plan_offers_save_as_script_on_cmd_s() {
        let (pinned, actions) = gpui::run_test_once(
            0,
            Box::new(|dispatcher| {
                let mut cx = gpui::TestAppContext::build(dispatcher, Some("intent_actions"));
                let (app, window) = launcher(&mut cx, Jev::Reading);
                type_into_launcher(&mut cx, window, &app, "where is the game tonight");
                cx.executor().advance_clock(Duration::from_millis(400));
                cx.run_until_parked();
                let seen = app.update(&mut cx, |app, _| {
                    let pinned = app
                        .selected_main_list_search_result_owned()
                        .and_then(|result| match result {
                            crate::scripts::SearchResult::Intent(m) => Some(m.plan.title.clone()),
                            _ => None,
                        });
                    let actions: Vec<(String, Option<String>)> = app
                        .get_focused_script_info()
                        .map(|info| crate::actions::get_script_context_actions(&info))
                        .unwrap_or_default()
                        .into_iter()
                        .map(|action| (action.id.clone(), action.shortcut.clone()))
                        .collect();
                    (pinned, actions)
                });
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                seen
            }),
        );
        assert!(
            pinned.is_some(),
            "the plan for the sentence is the highlighted row"
        );
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
    fn a_plan_never_takes_the_highlight_from_a_row_the_user_picked() {
        let (picked, highlighted, pinned) = gpui::run_test_once(
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
                    let pinned = app
                        .main_menu_committed_rows()
                        .iter()
                        .find(|row| row.eligibility.selectable)
                        .is_some_and(|row| {
                            row.stable_key
                                .starts_with(crate::scripts::INTENT_ROW_KEY_PREFIX)
                        });
                    (highlighted, pinned)
                });
                cx.update(|cx| cx.quit());
                cx.run_until_parked();
                (picked, seen.0, seen.1)
            }),
        );
        let picked = picked.expect("the sentence lists at least two rows to pick from");
        assert!(pinned, "the plan for the sentence is the first row");
        assert_eq!(highlighted.as_deref(), Some(picked.as_str()));
    }

    #[test]
    fn a_plan_is_only_ever_pinned_for_the_text_on_screen() {
        assert_rule(
            "The pinned intent row was read from exactly the text in the input",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                let recording = run_session_with(&session, Jev::Reading);
                for frame in recording.frames() {
                    if let Some(pinned) = &frame.pinned {
                        if pinned.request != frame.text {
                            return Err(broken(
                                format!(
                                    "after {:?} the list pins {:?}, read from {:?}, while the input says {:?}",
                                    frame.step, pinned.title, pinned.request, frame.text
                                ),
                                &recording,
                            ));
                        }
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
    fn typing_never_waits_on_jev() {
        assert_rule(
            "Right after each keystroke the launcher shows what it shows without Jev, or a list already rebuilt for exactly that text",
            24,
            reading_session(),
            |session| -> TestCaseResult {
                let without = run_session_with(&session, Jev::Off);
                let with = run_session_with(&session, Jev::Reading);
                for (plain, read) in without.keystrokes.iter().zip(&with.keystrokes).skip(1) {
                    let (Some(plain), Some(read)) = (plain.first(), read.first()) else {
                        continue;
                    };
                    let caught_up = read.list_text == read.text;
                    let same = plain.text == read.text
                        && plain.highlighted_action == read.highlighted_action
                        && plain.button == read.button;
                    if !same && !caught_up {
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
    fn when_jev_fails_the_launcher_is_exactly_the_launcher_without_jev() {
        assert_rule(
            "Every frame with a failing Jev matches the frame without Jev",
            24,
            reading_session(),
            |session| -> TestCaseResult {
                let without = run_session_with(&session, Jev::Off).transcript();
                let failing = run_session_with(&session, Jev::Failing);
                if failing.transcript() != without {
                    return Err(broken(
                        format!("without Jev the launcher showed:\n{without}"),
                        &failing,
                    ));
                }
                Ok(())
            },
        );
    }

    #[test]
    fn a_plan_moves_the_enter_button_at_most_once_after_a_keystroke() {
        assert_rule(
            "After a keystroke the Enter button changes at most once for the list and once more when Jev's plan for that text arrives, and that change names the plan",
            32,
            reading_session(),
            |session| -> TestCaseResult {
                let recording = run_session_with(&session, Jev::Reading);
                let mut shown = recording.frames().next().map(|frame| frame.button.clone());
                for frames in recording.keystrokes.iter().skip(1) {
                    let mut list_changes = 0;
                    let mut plan_changes = 0;
                    for frame in frames {
                        if shown.as_ref() == Some(&frame.button) {
                            continue;
                        }
                        shown = Some(frame.button.clone());
                        match &frame.pinned {
                            Some(pinned)
                                if pinned.request == frame.text
                                    && frame.button.label == pinned.verb
                                    && frame.step.starts_with('+') =>
                            {
                                plan_changes += 1
                            }
                            _ => list_changes += 1,
                        }
                    }
                    if list_changes > 1 || plan_changes > 1 {
                        let step = frames.first().map_or("?", |frame| frame.step.as_str());
                        return Err(broken(
                            format!(
                                "after {step:?} the Enter button changed {list_changes} times for the list and {plan_changes} times for Jev"
                            ),
                            &recording,
                        ));
                    }
                }
                Ok(())
            },
        );
    }
}
