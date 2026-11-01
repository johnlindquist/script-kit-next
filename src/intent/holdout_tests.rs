//! Explicit synthetic holdout. The Rust test never calls a provider. The separate recorder
//! posts a reviewed batch and saves the raw typed replies; score the resulting *plans*, not
//! confidence or a hand-picked subset. Keep expected actions independent of the responses.
use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

const CASES: &[(&str, &str)] = &[
    ("Open Violet Notebook for me", "open:Violet Notebook"),
    ("Launch the Copper Clock app", "open:Copper Clock"),
    (
        "Put Violet Notebook on the right half of the screen",
        "window:Tile Right Half:Violet Notebook",
    ),
    (
        "Center Copper Clock on the display",
        "window:Center:Copper Clock",
    ),
    (
        "Use the Duplicate Card menu item",
        "menu:Card/Duplicate Card",
    ),
    (
        "Make a note: the purple kite arrived",
        "capture:note:the purple kite arrived",
    ),
    (
        "Remind me to water the silver fern tomorrow",
        "capture:reminder:water the silver fern",
    ),
    (
        "Remember that the workshop mascot is a violet owl",
        "remember:the workshop mascot is a violet owl",
    ),
    ("Search my calendar for kite practice", "unavailable"),
    ("Quit Copper Clock", "held-quit:Copper Clock"),
];

fn scenario() -> (ScreenContext, Catalog) {
    let apps = ["Violet Notebook", "Copper Clock"];
    let ctx = ScreenContext {
        id: "live".into(),
        front_app: FrontApp {
            name: apps[0].into(),
            bundle_id: "lab.notebook".into(),
        },
        window_title: "PRIVATE TITLE NOT FOR PROVIDER".into(),
        open_windows: apps
            .iter()
            .map(|n| OpenWindow {
                app: n.to_string(),
                title: "PRIVATE TITLE".into(),
            })
            .collect(),
        menus: vec![MenuCommand {
            menu: "Card".into(),
            label: "Duplicate Card".into(),
            shortcut: None,
        }],
        ..Default::default()
    };
    let catalog = Catalog {
        commands: apps
            .iter()
            .map(|n| Command {
                id: format!("app/{n}"),
                name: n.to_string(),
                kind: CommandKind::App,
                description: "Application".into(),
                capture_targets: vec![],
                plugin: None,
                code: None,
            })
            .collect(),
        window_ops: native_window_ops(),
        text_tools: vec![],
    };
    (ctx, catalog)
}
fn outcome(plan: &Plan) -> String {
    if launcher::unavailable(plan).is_some() {
        return "unavailable".into();
    }
    if !plan.ready {
        return "not ready".into();
    }
    match plan.steps.as_slice() {
        [Step::Command { command }] => format!("open:{}", command.name),
        [s @ Step::Window { op, app, .. }] if launcher::quits_application(s) => {
            format!("held-quit:{app}")
        }
        [Step::Window { op, app, .. }] => format!("window:{}:{app}", op.label),
        [Step::Menu { path, .. }] => format!("menu:{}", path.join("/")),
        [Step::Capture { target, body, .. }] => format!("capture:{target}:{body}"),
        [Step::Remember { text, .. }] => format!("remember:{text}"),
        _ => format!("{}:{}", plan.pathway.key(), plan.title),
    }
}

#[test]
fn pinned_synthetic_launcher_holdout() {
    let (ctx, catalog) = scenario();
    let dir = std::env::var_os("SCRIPT_KIT_JEV_HOLDOUT_DIR").map(PathBuf::from);
    let mut scores = vec![];
    let mut requests = vec![];
    for (id, (request, want)) in CASES.iter().enumerate() {
        let built = engine::build_questions(request, &ctx, &catalog, &Default::default());
        let payload =
            json!({"model":jev::DEFAULT_MODEL,"state":built.state,"questions":built.questions});
        assert!(!payload.to_string().contains("PRIVATE TITLE"));
        requests.push(json!({"id":id,"payload":payload}));
        let Some(path) = dir
            .as_ref()
            .map(|d| d.join(format!("reply-{id}.json")))
            .filter(|p| p.exists())
        else {
            continue;
        };
        let reply = jev::decode_response(200, &std::fs::read_to_string(path).unwrap(), 0).unwrap();
        assert_eq!(reply.model, jev::DEFAULT_MODEL);
        built.questions.validate_answers(&reply.answers).unwrap();
        let plan = engine::compose_plan(request, &ctx, &catalog, &built, &reply.answers);
        let got = readings(plan)
            .first()
            .map(|r| outcome(&r.plan))
            .unwrap_or("no reading".into());
        scores.push(
            json!({"id":id,"request":request,"expected":want,"actual":got,"match":got==*want}),
        );
    }
    if let Some(dir) = dir {
        std::fs::create_dir_all(&dir).unwrap();
        let save = |name: &str, val: Value| {
            std::fs::write(dir.join(name), serde_json::to_vec_pretty(&val).unwrap()).unwrap()
        };
        if let Ok(prev) = std::fs::read(dir.join("requests.json")) {
            assert_eq!(serde_json::from_slice::<Value>(&prev).unwrap(),json!(requests),"Request shape changed. Use a fresh directory; old answers do not score new questions.");
        } else {
            save("requests.json", json!(requests));
        }
        let typing_dir = dir.join("typing");
        std::fs::create_dir_all(&typing_dir).unwrap();
        let mut win = sk_one_window::OneWindow::default();
        let mut now = 0;
        let mut text = String::new();
        let mut asks = Vec::new();
        for fragment in [
            "Put",
            " Violet Notebook",
            " on the right half of the screen",
        ] {
            for c in fragment.chars() {
                text.push(c);
                win.handle(now, sk_one_window::Event::Text(text.clone()));
                now += 30;
                assert!(!win
                    .handle(now, sk_one_window::Event::Tick)
                    .effects
                    .iter()
                    .any(|e| matches!(e, sk_one_window::Effect::AskJev { .. })));
            }
            now += 250;
            for e in win.handle(now, sk_one_window::Event::Tick).effects {
                if let sk_one_window::Effect::AskJev { text } = e {
                    let b = engine::build_questions(&text, &ctx, &catalog, &Default::default());
                    asks.push(json!({"id":asks.len(),"payload":{"model":jev::DEFAULT_MODEL,"state":b.state,"questions":b.questions}}));
                }
            }
        }
        assert_eq!(asks.len(), 3);
        let path = typing_dir.join("requests.json");
        if !path.exists() {
            std::fs::write(path, serde_json::to_vec_pretty(&asks).unwrap()).unwrap();
        }
        save(
            "score.json",
            json!({"total":CASES.len(),"answered":scores.len(),"correct":scores.iter().filter(|s|s["match"]==true).count(),"cases":scores}),
        );
    }
}
