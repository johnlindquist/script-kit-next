use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use gpui::proptest::prelude::*;
use gpui::proptest::test_runner::TestCaseResult;

use super::catalog::{native_window_ops, Catalog, Command, CommandKind};
use super::context::{FrontApp, LastAction, LastStep, MenuCommand, OpenWindow, ScreenContext};
use super::engine::{
    build_questions, compose_plan, understand, EngineOptions, Pathway, Plan, Step, CAPTURE_TARGETS,
};
use super::jev::{
    decode_response, parse_env, Answer, Answers, Criteria, JevConfig, JevError, JevResult, Oracle,
    Probabilities, Question, Questions,
};
use super::launcher::{readings, IntentLauncher};
use super::script::{normalize_shortcut, plan_to_script, slugify, ScriptOptions};
use super::spans::{span_candidates, split_clauses, tokenize, SPAN_LIMIT};
use super::wants_reading;
use crate::test_utils::property::assert_rule;

fn command(id: &str, kind: CommandKind, name: &str, description: &str) -> Command {
    Command {
        id: id.into(),
        kind,
        name: name.into(),
        description: description.into(),
        plugin: None,
        capture_targets: Vec::new(),
        code: None,
    }
}

fn catalog() -> Catalog {
    Catalog {
        commands: vec![
            command(
                "builtin/toggle-dark-mode",
                CommandKind::Builtin,
                "Toggle Dark Mode",
                "Switch between light and dark",
            ),
            command(
                "builtin/lock-screen",
                CommandKind::Builtin,
                "Lock Screen",
                "Lock the Mac",
            ),
            command(
                "builtin/clipboard-history",
                CommandKind::Builtin,
                "Clipboard History",
                "Things you copied",
            ),
            command(
                "script/main/standup",
                CommandKind::Script,
                "Standup Notes",
                "Open today's standup doc",
            ),
            Command {
                capture_targets: vec!["reminder".into(), "todo".into()],
                ..command(
                    "script/main/create-reminder",
                    CommandKind::Script,
                    "Create Reminder",
                    "Adds a reminder",
                )
            },
            command("app/Slack", CommandKind::App, "Slack", "Application"),
            command(
                "app/Google Chrome",
                CommandKind::App,
                "Google Chrome",
                "Application",
            ),
            command("app/Ghostty", CommandKind::App, "Ghostty", "Application"),
        ],
        window_ops: native_window_ops(),
        text_tools: vec![
            command(
                "scriptlet/ai-text-tools/summarize",
                CommandKind::Scriptlet,
                "Summarize",
                "Summarize the selected text",
            ),
            command(
                "scriptlet/ai-text-tools/fix-grammar",
                CommandKind::Scriptlet,
                "Fix Grammar & Spelling",
                "Fix mistakes",
            ),
        ],
    }
}

fn screen() -> ScreenContext {
    ScreenContext {
        id: "chrome".into(),
        front_app: FrontApp {
            name: "Google Chrome".into(),
            bundle_id: "com.google.Chrome".into(),
        },
        window_title: "Inbox".into(),
        open_windows: vec![
            OpenWindow {
                app: "Google Chrome".into(),
                title: "Inbox".into(),
            },
            OpenWindow {
                app: "Slack".into(),
                title: "#launcher".into(),
            },
            OpenWindow {
                app: "Ghostty".into(),
                title: "zsh".into(),
            },
        ],
        menus: vec![
            MenuCommand {
                menu: "File".into(),
                label: "New Tab".into(),
                shortcut: Some("⌘T".into()),
            },
            MenuCommand {
                menu: "File".into(),
                label: "Reopen Closed Tab".into(),
                shortcut: Some("⇧⌘T".into()),
            },
            MenuCommand {
                menu: "View".into(),
                label: "Developer › Developer Tools".into(),
                shortcut: None,
            },
        ],
        projects: vec!["script-kit-gpui".into(), "jev-demos".into()],
        last_action: Some(LastAction {
            summary: "Tiled Chrome left and Ghostty right".into(),
            steps: vec![
                LastStep::Window {
                    op_label: "Tile Left Half".into(),
                    app: "Google Chrome".into(),
                },
                LastStep::Window {
                    op_label: "Tile Right Half".into(),
                    app: "Ghostty".into(),
                },
            ],
        }),
        memory: vec!["Sarah's birthday is May 3rd".into()],
        ..ScreenContext::default()
    }
}

fn choice_answer(choice: &str, probabilities: &[(&str, f64)]) -> Answer {
    Answer::Choice {
        choice: choice.into(),
        confidence: 0.0,
        probabilities: Probabilities::new(
            probabilities
                .iter()
                .map(|(l, p)| (l.to_string(), *p))
                .collect(),
        ),
    }
}

#[test]
fn words_drop_edge_punctuation_but_keep_offsets() {
    let request = "  open “slack”!!!  now, please.";
    let words = tokenize(request);
    let texts: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(texts, ["open", "slack", "now", "please"]);
    for word in &words {
        assert_eq!(&request[word.start..word.end], word.text);
    }
}

#[test]
fn long_dictation_keeps_the_whole_request_and_both_ends() {
    let request = (0..40)
        .map(|i| format!("w{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let words = tokenize(&request);
    let spans = span_candidates(&request, &words, SPAN_LIMIT);
    assert_eq!(spans.len(), SPAN_LIMIT);
    assert!(spans.iter().any(|s| s.label == request));
    assert!(spans.iter().any(|s| s.label == "w0"));
    assert!(spans.iter().any(|s| s.label == "w39"));
}

#[test]
fn clauses_lend_the_first_verb_to_short_tails() {
    assert_eq!(
        split_clauses("open gmail, calendar and linear"),
        ["open gmail", "open calendar", "open linear"]
    );
    assert_eq!(
        split_clauses("put chrome on the left and then ghostty on the right"),
        ["put chrome on the left", "ghostty on the right"]
    );
    assert_eq!(split_clauses("buy milk"), ["buy milk"]);
}

#[test]
fn a_repeated_option_keeps_its_place_and_takes_the_new_description() {
    let mut criteria = Criteria::default();
    criteria.set("none", None);
    criteria.set("slack", None);
    criteria.set("none", Some("nothing".into()));
    assert_eq!(criteria.labels().collect::<Vec<_>>(), ["none", "slack"]);
    assert_eq!(criteria.description("none"), Some("nothing"));
}

#[test]
fn questions_serialize_in_jevs_wire_shape() {
    let mut criteria = Criteria::default();
    criteria.set("b", Some("second".into()));
    criteria.set("a", None);
    let mut questions = Questions::default();
    questions.insert(
        "pick",
        Question::Choice {
            instructions: "Which?".into(),
            criteria,
        },
    );
    questions.insert(
        "done",
        Question::Noul {
            instructions: "Done?".into(),
        },
    );
    assert_eq!(
        serde_json::to_string(&questions).unwrap(),
        r#"{"pick":{"type":"choice","instructions":"Which?","criteria":{"b":"second","a":null}},"done":{"type":"noul","instructions":"Done?"}}"#
    );
}

#[test]
fn a_choice_question_past_jevs_cap_is_refused_before_sending() {
    let mut criteria = Criteria::default();
    for i in 0..256 {
        criteria.set(format!("option {i}"), None);
    }
    let mut questions = Questions::default();
    questions.insert(
        "big",
        Question::Choice {
            instructions: "?".into(),
            criteria,
        },
    );
    assert!(matches!(
        questions.check_choice_limits(),
        Err(JevError::TooManyChoices { count: 256, .. })
    ));
}

#[test]
fn a_big_catalog_is_cut_to_jevs_cap_keeping_what_the_request_names() {
    let mut catalog = catalog();
    for i in 0..400 {
        catalog.commands.push(command(
            &format!("builtin/filler-{i}"),
            CommandKind::Builtin,
            &format!("Filler {i}"),
            "",
        ));
    }
    let built = build_questions(
        "toggle dark mode",
        &screen(),
        &catalog,
        &EngineOptions::default(),
    );
    built
        .questions
        .check_choice_limits()
        .expect("every question fits");
    assert!(built.builtin.get("Toggle Dark Mode").is_some());
}

#[test]
fn responses_decode_and_failures_say_why() {
    let body = r#"{"answers":{"pathway":{"type":"choice","choice":"run","confidence":0.9,"probabilities":{"run":0.9,"none":0.1}},"complete":{"type":"noul","noul":0.8}},"model":"jev-1","usage":{"input_tokens":12,"output_tokens":3}}"#;
    let result = decode_response(200, body, 42).expect("decodes");
    assert_eq!(result.input_tokens, 12);
    assert_eq!(result.ms, 42);
    assert!(
        matches!(result.answers["complete"], Answer::Noul { noul } if (noul - 0.8).abs() < 1e-9)
    );
    match decode_response(422, r#"{"detail":"bad question"}"#, 1) {
        Err(JevError::Http {
            status: 422,
            detail,
        }) => assert!(detail.contains("bad question")),
        other => panic!("expected an HTTP failure, got {other:?}"),
    }
    assert!(matches!(
        decode_response(200, "not json", 1),
        Err(JevError::Decode(_))
    ));
}

#[test]
fn env_files_yield_quoted_and_bare_values_only() {
    let parsed = parse_env(
        "# comment\nTYPESAFE_API_KEY=\"abc123\"\nTYPESAFE_MODEL = jev-1.13.0\nlower=nope\nEMPTY=\n",
    );
    assert_eq!(
        parsed.get("TYPESAFE_API_KEY").map(String::as_str),
        Some("abc123")
    );
    assert_eq!(
        parsed.get("TYPESAFE_MODEL").map(String::as_str),
        Some("jev-1.13.0")
    );
    assert!(!parsed.contains_key("lower"));
    assert!(!parsed.contains_key("EMPTY"));
}

#[test]
fn the_api_key_never_shows_in_debug_output() {
    let config = JevConfig::new("sk-secret-value", "jev-1");
    let printed = format!("{config:?}");
    assert!(!printed.contains("sk-secret-value"));
    assert!(printed.contains("jev-1"));
}

#[test]
fn shortcuts_normalize_from_words_and_glyphs() {
    assert_eq!(normalize_shortcut("Command Shift L"), "cmd shift l");
    assert_eq!(normalize_shortcut("control option w"), "ctrl alt w");
    assert_eq!(normalize_shortcut("cmd+opt+k"), "cmd alt k");
    assert_eq!(normalize_shortcut("⌘⇧K"), "cmd shift k");
}

#[test]
fn slugs_are_short_and_safe() {
    assert_eq!(
        slugify("Tile Chrome Left → Ghostty Right!"),
        "tile-chrome-left-ghostty-right"
    );
    assert_eq!(slugify("???"), "untitled");
    assert!(slugify(&"word ".repeat(40)).len() <= 48);
}

#[test]
fn only_plain_sentences_are_read() {
    assert!(wants_reading("remind me to call mom"));
    assert!(wants_reading("save https://zed.dev as a link"));
    assert!(wants_reading("lunch"));
    assert!(!wants_reading("todo; buy milk"));
    assert!(!wants_reading("@clipboard:invoice"));
    assert!(!wants_reading(">:dev fix it"));
    assert!(!wants_reading(";todo buy milk"));
    assert!(!wants_reading(":type:script deploy"));
    assert!(!wants_reading("   "));
}

fn answers_for_reminder() -> Answers {
    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer("capture", &[("capture", 0.95), ("none", 0.05)]),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert(
        "capture_target".into(),
        choice_answer("reminder", &[("reminder", 0.9)]),
    );
    answers.insert(
        "body".into(),
        choice_answer("call mom", &[("call mom", 0.9)]),
    );
    answers.insert(
        "when".into(),
        choice_answer("tomorrow at 5pm", &[("tomorrow at 5pm", 0.9)]),
    );
    answers
}

#[test]
fn a_reminder_compiles_to_the_launchers_capture_line() {
    let request = "remind me to call mom tomorrow at 5pm";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers_for_reminder());
    assert!(plan.ready);
    assert_eq!(plan.title, "Add reminder: call mom (tomorrow at 5pm)");
    assert_eq!(
        plan.typed.as_deref(),
        Some("reminder; call mom due:\"tomorrow at 5pm\"")
    );
    assert!(
        matches!(&plan.steps[0], Step::Capture { handler: Some(h), .. } if h.name == "Create Reminder")
    );
}

#[test]
fn a_row_still_on_screen_after_the_words_moved_on_runs_what_it_says() {
    let request = "remind me to call mom tomorrow at 5pm";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let rows = readings(compose_plan(
        request,
        &ctx,
        &catalog,
        &built,
        &answers_for_reminder(),
    ));
    let key = rows[0].key.clone();
    let mut launcher = IntentLauncher::default();
    launcher.remember(request, 7, &rows);
    launcher.show(request, &rows);
    let typed = "remind me to call mom tomorrow at 5pm!";
    launcher.show(typed, &[]);

    assert!(
        launcher.plan(typed, &key).is_none(),
        "the list moved on to the new words"
    );
    assert_eq!(
        launcher
            .plan_for_row(&key, 7)
            .map(|plan| plan.title.clone()),
        Some(rows[0].plan.title.clone()),
        "the row on screen still runs its plan"
    );
    assert!(
        launcher.plan_for_row(&key, 8).is_none(),
        "readings of another screen don't run here"
    );
}

#[test]
fn words_jev_invents_are_never_used() {
    let request = "remind me to call mom tomorrow at 5pm";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let mut answers = answers_for_reminder();
    answers.insert(
        "body".into(),
        choice_answer("wire $5,000 to a stranger", &[]),
    );
    answers.insert("capture_target".into(), choice_answer("rm -rf", &[]));
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers);
    assert!(!plan.ready);
    assert_eq!(plan.missing.as_deref(), Some("what to write down"));
    assert!(plan
        .typed
        .as_deref()
        .unwrap_or_default()
        .starts_with("todo; "));
}

#[test]
fn a_time_keeps_the_word_that_introduces_it() {
    let (ctx, catalog) = (screen(), catalog());
    let compose = |request: &str, body: &str, when: &str| {
        let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
        let mut answers = answers_for_reminder();
        answers.insert("body".into(), choice_answer(body, &[]));
        answers.insert("when".into(), choice_answer(when, &[]));
        compose_plan(request, &ctx, &catalog, &built, &answers)
            .reading
            .typed
    };
    assert_eq!(
        compose(
            "remind me in an hour to check the oven",
            "check the oven",
            "an hour"
        )
        .as_deref(),
        Some("reminder; check the oven due:\"in an hour\"")
    );
    assert_eq!(
        compose("remind me to put the pie in at 5", "put the pie in", "at 5").as_deref(),
        Some("reminder; put the pie in due:\"at 5\"")
    );
}

#[test]
fn a_named_open_app_beats_a_hesitant_front_window() {
    let request = "send slack to the bottom right corner";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let title = |slack: f64| {
        let mut answers = Answers::new();
        answers.insert(
            "pathway".into(),
            choice_answer("window", &[("window", 0.9), ("none", 0.1)]),
        );
        answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
        answers.insert(
            "window_op".into(),
            choice_answer("Tile Bottom Right", &[("Tile Bottom Right", 0.9)]),
        );
        let front = "the front window (Google Chrome)";
        answers.insert(
            "window_target".into(),
            choice_answer(front, &[(front, 1.0 - slack), ("Slack", slack)]),
        );
        compose_plan(request, &ctx, &catalog, &built, &answers)
            .reading
            .title
    };
    assert_eq!(title(0.28), "Tile Slack Bottom Right");
    assert_eq!(title(0.1), "Tile Google Chrome Bottom Right");
}

#[test]
fn a_short_fact_keeps_what_it_is_about() {
    let request = "fyi the wifi password at the cabin is pinecone42";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer("remember", &[("remember", 0.9), ("none", 0.1)]),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert("body".into(), choice_answer("pinecone42", &[]));
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers);
    assert_eq!(
        plan.title,
        "Remember: the wifi password at the cabin is pinecone42"
    );
}

#[test]
fn a_script_that_names_no_steps_saves_the_last_action() {
    let request = "make that a command";
    let catalog = catalog();
    let read = |ctx: &ScreenContext, steps: Option<&str>| {
        let built = build_questions(request, ctx, &catalog, &EngineOptions::default());
        let mut answers = Answers::new();
        answers.insert(
            "pathway".into(),
            choice_answer("automate", &[("automate", 0.95), ("none", 0.05)]),
        );
        answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
        answers.insert("refers_back".into(), Answer::Noul { noul: 0.3 });
        answers.insert("body".into(), choice_answer("none", &[]));
        answers.insert(
            "script_steps".into(),
            choice_answer(steps.unwrap_or("none"), &[]),
        );
        let plan = compose_plan(request, ctx, &catalog, &built, &answers);
        (
            plan.reading.title.clone(),
            plan.ready,
            plan.reading.missing.clone(),
        )
    };
    let with_last = screen();
    let saved = (
        "Save as script: Tiled Chrome left and Ghostty right".to_string(),
        true,
        None,
    );
    assert_eq!(read(&with_last, None), saved);
    assert_eq!(read(&with_last, Some("that")), saved);

    let nothing_yet = ScreenContext {
        last_action: None,
        ..screen()
    };
    assert_eq!(
        read(&nothing_yet, Some("that")),
        (
            "Save as script: what you just did".to_string(),
            false,
            Some("something done first to save".to_string()),
        )
    );
    assert_eq!(
        read(&nothing_yet, Some("a command")).2.as_deref(),
        Some("the steps (decomposed next)")
    );
}

#[test]
fn the_window_question_names_minimize_and_close() {
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions("minimize it", &ctx, &catalog, &EngineOptions::default());
    let questions = serde_json::to_string(&built.questions).expect("questions serialize");
    assert!(questions.contains("move, resize, tile, minimize, or close it"));
    assert!(questions.contains("not by what they did last"));
}

#[test]
fn every_window_is_never_carried_out_on_one_window() {
    let request = "close all my windows";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let Some(Question::Choice { criteria, .. }) = built.questions.get("window_target") else {
        panic!("the window target is a choice");
    };
    assert!(criteria.labels().any(|label| label == "every window"));

    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer("window", &[("window", 0.9), ("none", 0.1)]),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert(
        "window_op".into(),
        choice_answer("Close Window", &[("Close Window", 0.99)]),
    );
    answers.insert(
        "window_target".into(),
        choice_answer("every window", &[("every window", 1.0)]),
    );
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers);
    assert_eq!(plan.title, "Close Window: every window");
    assert!(plan.steps.is_empty());
    assert!(!plan.ready);
    assert_eq!(
        plan.missing.as_deref(),
        Some("a way to act on every window at once")
    );
}

#[test]
fn a_run_reading_with_nothing_to_run_gives_way_to_one_that_can() {
    let request = "close slack";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer(
            "run",
            &[
                ("run", 0.65),
                ("app_command", 0.19),
                ("window", 0.10),
                ("none", 0.06),
            ],
        ),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert(
        "app".into(),
        choice_answer("none", &[("none", 0.81), ("Slack", 0.17)]),
    );
    answers.insert(
        "menu".into(),
        choice_answer("none", &[("none", 0.96), ("File › New Tab", 0.04)]),
    );
    answers.insert(
        "window_op".into(),
        choice_answer("Close Window", &[("Close Window", 1.0)]),
    );
    answers.insert(
        "window_target".into(),
        choice_answer("Slack", &[("Slack", 0.99)]),
    );
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers);
    assert_eq!(plan.pathway, Pathway::Window);
    assert_eq!(plan.title, "Close Slack Window");
    assert!(plan.ready);
}

#[test]
fn a_near_tie_goes_to_the_command_whose_name_the_user_said_most_of() {
    let ctx = screen();
    let mut catalog = catalog();
    catalog.commands.extend([
        command(
            "scriptlet/1password/copy-password",
            CommandKind::Scriptlet,
            "Copy Password",
            "Copy a password to the clipboard",
        ),
        command(
            "app/1Password",
            CommandKind::App,
            "1Password",
            "Application",
        ),
        command(
            "scriptlet/web/slack",
            CommandKind::Scriptlet,
            "Slack",
            "Open Slack in the browser",
        ),
    ]);
    let title = |request: &str, script: &str, app: &str| {
        let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
        let mut answers = Answers::new();
        answers.insert(
            "pathway".into(),
            choice_answer("run", &[("run", 0.9), ("none", 0.1)]),
        );
        answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
        answers.insert("script".into(), choice_answer(script, &[(script, 0.97)]));
        answers.insert("app".into(), choice_answer(app, &[(app, 0.97)]));
        compose_plan(request, &ctx, &catalog, &built, &answers)
            .reading
            .title
    };
    assert_eq!(
        title(
            "copy my password from 1password",
            "Copy Password",
            "1Password"
        ),
        "Copy Password"
    );
    assert_eq!(title("open slack", "Slack", "Slack"), "Open Slack");
}

/// Answers each request, first round or clause, from a table keyed by its words.
struct Scripted(fn(&str) -> Answers);

impl Oracle for Scripted {
    fn ask(&self, state: &serde_json::Value, _: &Questions) -> Result<JevResult, JevError> {
        Ok(JevResult {
            answers: (self.0)(state["request"].as_str().unwrap_or_default()),
            ..JevResult::default()
        })
    }
}

fn after_opening_slack(request: &str) -> Answers {
    let mut answers = Answers::new();
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    if request.starts_with("open slack") {
        let compound = if request == "open slack" { 0.1 } else { 0.9 };
        answers.insert("compound".into(), Answer::Noul { noul: compound });
        answers.insert("pathway".into(), choice_answer("run", &[("run", 0.95)]));
        answers.insert("app".into(), choice_answer("Slack", &[("Slack", 0.95)]));
        return answers;
    }
    answers.insert(
        "menu".into(),
        choice_answer("File › New Tab", &[("File › New Tab", 0.9)]),
    );
    if request == "put it on the right half" {
        answers.insert(
            "pathway".into(),
            choice_answer("app_command", &[("app_command", 0.61), ("window", 0.39)]),
        );
        answers.insert(
            "window_op".into(),
            choice_answer("Tile Right Half", &[("Tile Right Half", 0.99)]),
        );
        let front = "the front window (Google Chrome)";
        answers.insert(
            "window_target".into(),
            choice_answer("Ghostty", &[("Ghostty", 0.5), (front, 0.47)]),
        );
    } else {
        answers.insert(
            "pathway".into(),
            choice_answer("app_command", &[("app_command", 0.9)]),
        );
        answers.insert("window_op".into(), choice_answer("none", &[("none", 0.99)]));
    }
    answers
}

#[test]
fn after_opening_an_app_later_steps_act_on_it() {
    let (ctx, catalog) = (screen(), catalog());
    let read = |request: &str| {
        understand(
            request,
            &ctx,
            &catalog,
            &Scripted(after_opening_slack),
            &EngineOptions::default(),
        )
        .expect("scripted answers never fail")
    };

    let placed = read("open slack and put it on the right half");
    assert_eq!(placed.title, "Open Slack → Tile Slack Right Half");
    assert!(placed.ready);

    let keys = read("open slack and press cmd k");
    assert_eq!(keys.title, "Open Slack → Do it in Slack");
    assert_eq!(keys.missing.as_deref(), Some("a Slack menu command"));
    assert!(!keys.ready);
}

#[test]
fn a_saved_reminder_asks_for_its_content_next_time() {
    let request = "remind me to call mom tomorrow at 5pm";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers_for_reminder());
    let script = plan_to_script(
        &plan,
        &ScriptOptions {
            shortcut: Some("cmd shift r".into()),
            ..ScriptOptions::default()
        },
    );
    assert_eq!(script.filename, "add-reminder.ts");
    assert!(script.source.starts_with("import \"@scriptkit/sdk\";\n"));
    assert!(script.source.contains("shortcut: \"cmd shift r\""));
    assert!(script.source.contains(
        "await capture(\"reminder\", await arg({ placeholder: \"New reminder\", hint: \"e.g. call mom\" }), await arg({ placeholder: \"When?\", hint: \"e.g. tomorrow at 5pm\" }), \"create-reminder\");"
    ));
}

#[test]
fn saving_what_i_just_did_replays_each_window_move_by_app() {
    let request = "save that as a script";
    let (ctx, catalog) = (screen(), catalog());
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer("automate", &[("automate", 0.97)]),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert("refers_back".into(), Answer::Noul { noul: 0.9 });
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers);
    assert!(plan.ready);
    assert_eq!(plan.steps.len(), 2);
    let script = plan_to_script(&plan, &ScriptOptions::default());
    assert!(script
        .source
        .contains("(await getWindows()).find((w) => w.appName === \"Google Chrome\")"));
    assert!(script
        .source
        .contains("if (win) await tileWindow(win.windowId, \"left\");"));
    assert!(script
        .source
        .contains("if (win) await tileWindow(win.windowId, \"right\");"));
}

/// Deterministic noise keyed by seed, request, and question, so a case replays exactly.
struct NoisyOracle {
    seed: u64,
    garbage: bool,
}

impl NoisyOracle {
    fn roll(&self, request: &str, name: &str, salt: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        (self.seed, request, name, salt).hash(&mut hasher);
        hasher.finish()
    }

    fn unit(&self, request: &str, name: &str, salt: u64) -> f64 {
        (self.roll(request, name, salt) % 10_000) as f64 / 10_000.0
    }
}

impl Oracle for NoisyOracle {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError> {
        let request = state["request"].as_str().unwrap_or_default().to_string();
        let mut answers = Answers::new();
        for (name, question) in questions.iter() {
            if self.roll(&request, name, 99) % 11 == 0 {
                continue;
            }
            let answer = match question {
                Question::Noul { .. } => Answer::Noul {
                    noul: self.unit(&request, name, 0),
                },
                Question::Score { .. } => Answer::Score {
                    score: 3.0 * self.unit(&request, name, 0),
                    confidence: 0.0,
                    probabilities: Probabilities::default(),
                },
                Question::Choice { criteria, .. } => {
                    let labels: Vec<&str> = criteria.labels().collect();
                    let entries: Vec<(String, f64)> = labels
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| self.roll(&request, name, *i as u64) % 3 == 0)
                        .map(|(i, l)| (l.to_string(), self.unit(&request, name, 1000 + i as u64)))
                        .collect();
                    let choice = if self.garbage && self.roll(&request, name, 7) % 4 == 0 {
                        "a label nobody offered".to_string()
                    } else {
                        labels[self.roll(&request, name, 5) as usize % labels.len()].to_string()
                    };
                    Answer::Choice {
                        choice,
                        confidence: 0.0,
                        probabilities: Probabilities::new(entries),
                    }
                }
            };
            answers.insert(name.to_string(), answer);
        }
        Ok(JevResult {
            answers,
            ..JevResult::default()
        })
    }
}

const VOCABULARY: &[&str] = &[
    "open",
    "slack",
    "and",
    "then",
    "put",
    "it",
    "on",
    "the",
    "left",
    "remind",
    "me",
    "to",
    "call",
    "mom",
    "tomorrow",
    "at",
    "5pm",
    "make",
    "a",
    "script",
    "that",
    "save",
    "as",
    "shortcut",
    "cmd",
    "shift",
    "L",
    "what",
    "did",
    "I",
    "copy",
    "about",
    "invoice,",
    "summarize",
    "this",
    "ghostty",
    "right",
    "https://zed.dev",
    "“quoted”",
    "!!!",
    "sarah's",
    "birthday",
];

fn request_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(VOCABULARY), 1..24).prop_map(|words| words.join(" "))
}

fn plan_keeps_the_rules(plan: &Plan, request: &str, is_clause: bool) -> TestCaseResult {
    prop_assert!(!plan.title.is_empty(), "empty title for {request:?}");
    if plan.ready {
        prop_assert_ne!(
            plan.pathway,
            Pathway::None,
            "a ready plan must ask for something"
        );
        prop_assert!(plan.missing.is_none(), "a ready plan lacks nothing");
        prop_assert!(
            !plan.steps.is_empty(),
            "a ready plan has a step to run: {plan:?}"
        );
    }
    if is_clause {
        prop_assert_ne!(
            plan.pathway,
            Pathway::Automate,
            "a clause of {:?} tried to save a script",
            request
        );
    }
    for slot in [
        &plan.slots.body,
        &plan.slots.task,
        &plan.slots.when,
        &plan.slots.query,
        &plan.slots.shortcut,
        &plan.slots.script_steps,
    ]
    .into_iter()
    .flatten()
    {
        prop_assert!(
            plan.request.contains(slot.as_str()),
            "slot {slot:?} is not in {:?}",
            plan.request
        );
    }
    for step in &plan.steps {
        if let Step::Capture { target, body, .. } = step {
            prop_assert!(CAPTURE_TARGETS.iter().any(|(t, _)| t == target));
            prop_assert!(!body.is_empty());
        }
    }
    if !plan.clauses.is_empty() {
        let clause_steps: Vec<&Step> = plan.clauses.iter().flat_map(|c| c.steps.iter()).collect();
        prop_assert_eq!(clause_steps, plan.steps.iter().collect::<Vec<_>>());
        for clause in &plan.clauses {
            plan_keeps_the_rules(clause, &clause.request, true)?;
            if !clause.ready {
                prop_assert!(
                    !plan.ready,
                    "{:?} is ready though {:?} is not",
                    request,
                    clause.request
                );
            }
        }
    }
    Ok(())
}

#[test]
fn whatever_jev_answers_a_plan_keeps_the_rules() {
    let (ctx, catalog) = (screen(), catalog());
    assert_rule(
        "a ready plan runs something, uses only the user's words, and its steps are its clauses' steps",
        192,
        (request_strategy(), any::<u64>(), any::<bool>()),
        |(request, seed, garbage)| {
            let oracle = NoisyOracle { seed, garbage };
            let plan = understand(&request, &ctx, &catalog, &oracle, &EngineOptions::default())
                .expect("the noisy oracle always answers");
            plan_keeps_the_rules(&plan, &request, false)?;
            let script = plan_to_script(&plan, &ScriptOptions::default());
            prop_assert!(script.source.starts_with("import \"@scriptkit/sdk\";\n"));
            prop_assert!(script.filename.ends_with(".ts"));
            prop_assert!(
                script
                    .filename
                    .trim_end_matches(".ts")
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            );
            Ok(())
        },
    );
}

#[test]
fn any_text_tokenizes_into_slices_of_itself() {
    assert_rule(
        "every word is a non-empty slice of the request at its own offsets",
        256,
        any::<String>(),
        |request| {
            for word in tokenize(&request) {
                prop_assert!(!word.text.is_empty());
                prop_assert_eq!(&request[word.start..word.end], word.text.as_str());
            }
            for clause in split_clauses(&request) {
                prop_assert!(!clause.trim().is_empty());
            }
            Ok(())
        },
    );
}

#[test]
fn spans_are_the_users_own_words_and_fit_under_the_cap() {
    assert_rule(
        "spans are unique substrings of the request, at most the limit",
        128,
        request_strategy(),
        |request| {
            let words = tokenize(&request);
            let spans = span_candidates(&request, &words, SPAN_LIMIT);
            prop_assert!(spans.len() <= SPAN_LIMIT);
            let mut seen = std::collections::HashSet::new();
            for span in &spans {
                prop_assert!(request.contains(span.label.as_str()));
                prop_assert!(
                    seen.insert(span.label.to_lowercase()),
                    "duplicate span {:?}",
                    span.label
                );
            }
            Ok(())
        },
    );
}

fn window_answers(op: &str, target: &str) -> Answers {
    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer("window", &[("window", 0.9), ("none", 0.1)]),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert("window_op".into(), choice_answer(op, &[(op, 0.9)]));
    answers.insert(
        "window_target".into(),
        choice_answer(target, &[(target, 1.0)]),
    );
    answers
}

#[test]
fn quitting_or_closing_ends_only_the_app_the_user_named() {
    let (ctx, catalog) = (screen(), catalog());
    let front = "the front window (Google Chrome)";
    let read = |request: &str, op: &str| {
        let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
        let plan = compose_plan(request, &ctx, &catalog, &built, &window_answers(op, front));
        (plan.title.clone(), plan.ready, plan.missing.clone())
    };
    assert_eq!(
        read("quit slack", "Quit App"),
        ("Quit Slack".to_string(), true, None)
    );
    assert_eq!(
        read("close slack", "Close Window"),
        ("Close Slack Window".to_string(), true, None)
    );
    assert_eq!(
        read("quit spotify", "Quit App"),
        (
            "Quit an app".to_string(),
            false,
            Some("which app to quit".to_string())
        )
    );
    assert_eq!(
        read("close the thing", "Close Window"),
        (
            "Close a window".to_string(),
            false,
            Some("which window to close".to_string())
        )
    );
    assert_eq!(
        read("quit this app", "Quit App"),
        ("Quit Google Chrome".to_string(), true, None)
    );
}

#[test]
fn closing_a_named_app_with_no_window_open_waits() {
    let ctx = ScreenContext {
        open_windows: screen()
            .open_windows
            .into_iter()
            .filter(|w| w.app != "Slack")
            .collect(),
        ..screen()
    };
    let catalog = catalog();
    let request = "close slack";
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let answers = window_answers("Close Window", "the front window (Google Chrome)");
    let plan = compose_plan(request, &ctx, &catalog, &built, &answers);
    assert!(!plan.ready);
    assert!(plan.steps.is_empty());
    assert_eq!(plan.missing.as_deref(), Some("an open Slack window"));
}

#[test]
fn a_quit_plan_reads_and_saves_as_a_quit() {
    let (ctx, catalog) = (screen(), catalog());
    let request = "quit slack";
    let built = build_questions(request, &ctx, &catalog, &EngineOptions::default());
    let plan = compose_plan(
        request,
        &ctx,
        &catalog,
        &built,
        &window_answers("Quit App", "Slack"),
    );
    assert_eq!(super::present::plan_verb(&plan), "Quit");
    assert!(super::present::plan_subtitle(&plan).starts_with("Quits an app"));
    let script = plan_to_script(&plan, &ScriptOptions::default());
    assert!(script
        .source
        .contains(r#"await exec("osascript", ["-e", "tell application \"Slack\" to quit"]);"#));
    assert!(!script.source.contains("closeWindow"));
}

/// Jev as it read "open a new tab to johnlindquist.com" in Chrome: open Chrome, or its
/// New Tab menu item, neither of which goes to the address.
fn open_web_answers(app: &str) -> Answers {
    let mut answers = Answers::new();
    answers.insert(
        "pathway".into(),
        choice_answer(
            "run",
            &[("run", 0.55), ("app_command", 0.35), ("none", 0.1)],
        ),
    );
    answers.insert("complete".into(), Answer::Noul { noul: 0.9 });
    answers.insert("app".into(), choice_answer(app, &[(app, 0.9)]));
    answers.insert(
        "menu".into(),
        choice_answer("File › New Tab", &[("File › New Tab", 0.8), ("none", 0.2)]),
    );
    answers
}

fn web_plan(request: &str, ctx: &ScreenContext, answers: &Answers) -> Plan {
    let catalog = catalog();
    let built = build_questions(request, ctx, &catalog, &EngineOptions::default());
    compose_plan(request, ctx, &catalog, &built, answers)
}

#[test]
fn only_web_addresses_the_user_typed_are_read_as_pages() {
    let web = |request: &str| super::engine::web_address(request).map(|w| (w.said, w.url));
    assert_eq!(
        web("open a new tab to johnlindquist.com"),
        Some((
            "johnlindquist.com".into(),
            "https://johnlindquist.com".into()
        ))
    );
    assert_eq!(
        web("go to https://github.com/johnlindquist/kit."),
        Some((
            "https://github.com/johnlindquist/kit".into(),
            "https://github.com/johnlindquist/kit".into()
        ))
    );
    assert_eq!(
        web("pull up www.egghead.io/courses, please"),
        Some((
            "www.egghead.io/courses".into(),
            "https://www.egghead.io/courses".into()
        ))
    );
    for request in [
        "email john@johnlindquist.com",
        "run deploy.sh",
        "open Notes.app",
        "open johnlindquist.company",
        "open ~/dev/site.io",
        "open a new tab",
    ] {
        assert_eq!(web(request), None, "{request:?}");
    }
}

#[test]
fn a_new_tab_to_an_address_opens_the_address_in_the_front_browser() {
    let request = "open a new tab to johnlindquist.com";
    let plan = web_plan(request, &screen(), &open_web_answers("Google Chrome"));
    assert!(plan.ready);
    assert_eq!(plan.title, "Open johnlindquist.com in Google Chrome");
    assert_eq!(
        plan.steps,
        [Step::OpenUrl {
            url: "https://johnlindquist.com".into(),
            app: Some("Google Chrome".into()),
        }]
    );
    assert!(
        plan.alternatives.iter().all(|a| a.title != plan.title),
        "the menu reading is the same page, not a second row"
    );
    assert_eq!(super::present::plan_verb(&plan), "Open");
    assert_eq!(
        super::present::plan_subtitle(&plan),
        "Opens in Google Chrome"
    );
    let word = |kind| plan.words.iter().find(|w| w.kind == kind).cloned().unwrap();
    let subject = word(super::words::WordKind::Subject);
    assert_eq!(subject.said.as_deref(), Some("johnlindquist.com"));
    let target = word(super::words::WordKind::Target);
    assert_eq!(target.value, "Google Chrome");
    assert_eq!(target.said.as_deref(), Some("a new tab"));
    let script = plan_to_script(&plan, &ScriptOptions::default());
    assert!(script
        .source
        .contains(r#"await exec("open", ["-a", "Google Chrome", "https://johnlindquist.com"]);"#));
}

#[test]
fn an_address_opens_in_the_browser_named_or_else_the_default_one() {
    let ghostty = ScreenContext {
        front_app: FrontApp {
            name: "Ghostty".into(),
            bundle_id: "com.mitchellh.ghostty".into(),
        },
        menus: Vec::new(),
        ..screen()
    };
    let named = web_plan(
        "open johnlindquist.com in chrome",
        &ghostty,
        &open_web_answers("Google Chrome"),
    );
    assert_eq!(named.title, "Open johnlindquist.com in Google Chrome");
    let unnamed = web_plan(
        "go to johnlindquist.com",
        &ghostty,
        &open_web_answers("Ghostty"),
    );
    assert_eq!(unnamed.title, "Open johnlindquist.com");
    assert_eq!(
        unnamed.steps,
        [Step::OpenUrl {
            url: "https://johnlindquist.com".into(),
            app: None,
        }]
    );
    assert_eq!(
        super::present::plan_subtitle(&unnamed),
        "Opens in your browser"
    );
}

#[test]
fn an_address_is_not_opened_when_the_words_name_what_to_do_with_it() {
    let sent = web_plan(
        "send johnlindquist.com to slack",
        &screen(),
        &open_web_answers("Slack"),
    );
    assert_eq!(sent.title, "Open Slack");

    let mut saved = answers_for_reminder();
    saved.insert(
        "capture_target".into(),
        choice_answer("link", &[("link", 0.9)]),
    );
    saved.insert(
        "body".into(),
        choice_answer("johnlindquist.com", &[("johnlindquist.com", 0.9)]),
    );
    saved.remove("when");
    let plan = web_plan("save johnlindquist.com for later", &screen(), &saved);
    assert_eq!(plan.pathway, Pathway::Capture);
}
