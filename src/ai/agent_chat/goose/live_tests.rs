//! Live checks of Agent Chat's agent features against the real goose sidecar
//! and a local Ollama model. Opt-in and single-threaded (they set `SK_PATH`):
//!
//! ```sh
//! OLLAMA_HOST=127.0.0.1:11535 SCRIPT_KIT_LIVE_GOOSE_MODEL=qwen3:4b \
//!   ./scripts/agentic/agent-cargo.sh test --lib -- --ignored live_goose --test-threads=1
//! ```
//!
//! Everything lives in a temporary kit folder: goose state, the Script Kit
//! profile's workspace, and the files the model writes. Model behaviour is
//! nondeterministic, so each step records what happened and the test asserts
//! the parts Script Kit controls (approvals reach the person, answers are
//! honoured, reverts restore files, rewinds cut the session, steering is
//! accepted) only when the model actually called the tool.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::launch_spec::GooseProfileLaunch;
use super::runtime::GooseRuntime;
use crate::ai::agent_chat::approval::AgentApprovalMode;
use crate::ai::agent_chat::content::{ContentBlock, TextContent};
use crate::ai::agent_chat::events::AgentChatEvent;
use crate::ai::agent_chat::profiles::{built_in_script_kit_profile, AgentChatProfileContext};
use crate::ai::agent_chat::runtime::{AgentChatConnection, AgentChatTurnRequest};
use crate::ai::agent_chat::ui::capabilities::AgentChatToolPolicy;

const TURN_TIMEOUT: Duration = Duration::from_secs(240);

#[derive(Debug, Default)]
struct Turn {
    text: String,
    tools: Vec<String>,
    approvals: Vec<String>,
    failed: Option<String>,
}

/// Answer for each approval: `Some(kind)` picks the first option of that
/// kind (`AllowOnce`, `RejectOnce`), `None` cancels.
type Decide = dyn Fn(&str) -> Option<&'static str>;

fn run_turn(
    runtime: &GooseRuntime,
    cwd: &Path,
    text: &str,
    decide: &Decide,
    mut on_delta: impl FnMut(&str),
) -> Turn {
    let rx = runtime
        .start_turn(AgentChatTurnRequest {
            ui_thread_id: "live".into(),
            cwd: cwd.to_path_buf(),
            blocks: vec![ContentBlock::Text(TextContent::new(text))],
            model_id: None,
            tool_policy: AgentChatToolPolicy::Full,
            allow_interactive_sign_in: false,
        })
        .expect("turn starts");
    let mut turn = Turn::default();
    let started = Instant::now();
    loop {
        let remaining = TURN_TIMEOUT.saturating_sub(started.elapsed());
        let event = match smol::block_on(async {
            smol::future::or(async { rx.recv().await.ok() }, async {
                smol::Timer::after(remaining).await;
                None
            })
            .await
        }) {
            Some(event) => event,
            None => {
                turn.failed = Some("timed out or closed".into());
                return turn;
            }
        };
        match event {
            AgentChatEvent::AgentMessageDelta(delta) => {
                on_delta(&delta);
                turn.text.push_str(&delta);
            }
            AgentChatEvent::ToolCallStarted {
                tool_name, title, ..
            } => turn.tools.push(tool_name.unwrap_or(title)),
            AgentChatEvent::ApprovalRequested(request) => {
                turn.approvals.push(request.title.clone());
                let choice = decide(&request.title).and_then(|kind| {
                    request
                        .options
                        .iter()
                        .find(|option| option.kind == kind)
                        .map(|option| option.option_id.clone())
                });
                let _ = request.reply_tx.send_blocking(choice);
            }
            AgentChatEvent::TurnCompleted { .. } => return turn,
            AgentChatEvent::TurnFailed { failure } => {
                turn.failed = Some(format!("{:?}", failure.failure.code));
                return turn;
            }
            _ => {}
        }
    }
}

/// What goose last sent the model: the system prompt and the tool names,
/// from goose's own request log under the kit.
fn last_model_request(kit: &Path) -> (String, Vec<String>) {
    let logs = kit.join("goose/state/logs");
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(&logs).into_iter().flatten().flatten() {
        let path = entry.path();
        let is_request_log = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("llm_request"));
        let modified = entry.metadata().and_then(|meta| meta.modified()).ok();
        if let (true, Some(modified)) = (is_request_log, modified) {
            if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
                newest = Some((modified, path));
            }
        }
    }
    let Some((_, path)) = newest else {
        return (String::new(), Vec::new());
    };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let Some(line) = text.lines().next() else {
        return (String::new(), Vec::new());
    };
    let request: Value = serde_json::from_str(line).unwrap_or(Value::Null);
    let input = &request["input"];
    let system = input["messages"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|message| message["role"] == "system")
        .map(|message| message["content"].to_string())
        .unwrap_or_default();
    let tools = input["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|tool| {
            tool["function"]["name"]
                .as_str()
                .or_else(|| tool["name"].as_str())
                .map(str::to_string)
        })
        .collect();
    (system, tools)
}

fn recv_one(rx: crate::ai::agent_chat::events::AgentChatEventRx) -> Option<AgentChatEvent> {
    smol::block_on(async {
        smol::future::or(async { rx.recv().await.ok() }, async {
            smol::Timer::after(Duration::from_secs(30)).await;
            None
        })
        .await
    })
}

/// Allow only the tools a step is about; anything else the model wanders
/// into (screen capture, shell, deletes) is refused.
fn allow(title: &str) -> Option<&'static str> {
    let title = title.to_lowercase();
    let expected = ["write", "codeword", "todo"];
    Some(if expected.iter().any(|word| title.contains(word)) {
        "AllowOnce"
    } else {
        "RejectOnce"
    })
}

fn deny(_: &str) -> Option<&'static str> {
    Some("RejectOnce")
}

fn goose_binary() -> PathBuf {
    std::env::var_os("SCRIPT_KIT_GOOSE_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/goose-sidecar/goose")
        })
}

#[test]
#[ignore = "needs the goose sidecar and a local Ollama server (see module docs)"]
fn live_goose_agent_features() {
    let Ok(ollama_host) = std::env::var("OLLAMA_HOST") else {
        panic!("set OLLAMA_HOST to a running Ollama server");
    };
    let model = std::env::var("SCRIPT_KIT_LIVE_GOOSE_MODEL").unwrap_or_else(|_| "qwen3:4b".into());
    let kit = tempfile::tempdir().expect("temp kit");
    let kit_path = kit.path().canonicalize().unwrap();
    // `SCRIPT_KIT_LIVE_KEEP=1` keeps the kit (goose logs) for diagnosis.
    let kit_path = if std::env::var("SCRIPT_KIT_LIVE_KEEP").as_deref() == Ok("1") {
        let kept = kit.keep();
        eprintln!("LIVE_GOOSE_KIT {}", kept.display());
        kept.canonicalize().unwrap()
    } else {
        kit_path
    };
    // `SCRIPT_KIT_LIVE_STEPS=1,3` runs only those steps.
    let steps: Option<Vec<u32>> = std::env::var("SCRIPT_KIT_LIVE_STEPS").ok().map(|value| {
        value
            .split(',')
            .filter_map(|step| step.trim().parse().ok())
            .collect()
    });
    let wants = |step: u32| steps.as_ref().is_none_or(|steps| steps.contains(&step));
    std::env::set_var("SK_PATH", &kit_path);
    let mut report = serde_json::Map::new();

    // Script Kit's MCP server, so the session gets Script Kit's tools.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let server = crate::mcp_server::McpServer::new(port, kit_path.clone()).expect("mcp server");
    let server_handle = server.start().expect("mcp server starts");

    let ctx = AgentChatProfileContext {
        kit_path: kit_path.clone(),
    };
    let mut profile = built_in_script_kit_profile(&ctx);
    profile.provider = Some("ollama".into());
    profile.model = Some(model.clone());
    profile.thinking = None;
    let cwd = ctx.script_kit_cwd();
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::write(
        cwd.join("AGENTS.md"),
        "# Project notes\n\nWhen asked for the project codeword, answer PELICAN-42.\n",
    )
    .unwrap();

    let launch = GooseProfileLaunch::from_profile(&profile, goose_binary(), cwd.clone(), &kit_path);
    let mut spec = launch.spec_for_spawn();
    spec.env.insert("OLLAMA_HOST".into(), ollama_host);
    spec.thinking = None;
    let policy_id = spec
        .agent_policy_id
        .clone()
        .expect("Script Kit tools policy");
    // The user's MCP servers, as `config.ts` would declare them.
    let mcp: crate::config::McpConfig = serde_json::from_value(json!({
        "servers": {"codeword": {
            "transport": "stdio",
            "command": "python3",
            "args": [PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mcp/codeword_server.py")],
        }}
    }))
    .unwrap();
    let user_mcp = super::recipe::user_mcp_extensions(&mcp);
    spec.recipe["extensions"]
        .as_array_mut()
        .unwrap()
        .extend(user_mcp);
    let extension_names: Vec<Value> = spec.recipe["extensions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|extension| extension["name"].clone())
        .collect();
    report.insert("extensions".into(), json!(extension_names));
    let runtime = GooseRuntime::spawn(spec).expect("goose starts");

    if wants(1) {
        // 1. AGENTS.md in the working folder reaches the model.
        let turn = run_turn(
            &runtime,
            &cwd,
            "What is the project codeword? Reply with only the codeword.",
            &allow,
            |_| {},
        );
        let (system, tools) = last_model_request(&kit_path);
        let has = |name: &str| tools.iter().any(|tool| tool == name);
        report.insert(
            "agents_md".into(),
            json!({"reply": turn.text, "failed": turn.failed, "in_system_prompt": system.contains("PELICAN-42")}),
        );
        report.insert(
            "tools_offered".into(),
            json!({"count": tools.len(), "user_mcp": has("codeword__lookup_codeword"), "app": has("scriptkit__kit_notes_create"), "subagents": has("delegate"), "skills": has("load_skill"), "todo": has("todo__todo_write"), "write": has("scriptkit__write"), "shell": has("shell")}),
        );
        assert!(turn.failed.is_none(), "{turn:?}");
        assert!(
            system.contains("PELICAN-42"),
            "AGENTS.md did not reach the model"
        );
        for name in [
            "codeword__lookup_codeword",
            "scriptkit__kit_notes_create",
            "scriptkit__kit_read",
            "scriptkit__chats_search",
            "delegate",
            "load_skill",
            "todo__todo_write",
            "scriptkit__write",
            "scriptkit__edit",
            "shell",
        ] {
            assert!(has(name), "{name} was not offered: {tools:?}");
        }
    }

    if wants(2) {
        // 2. A read-only Script Kit app tool runs without asking.
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the scriptkit__kit_resources tool now, then reply with one uri from its result.",
            &deny,
            |_| {},
        );
        report.insert(
            "app_tool_read_only".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "text": turn.text}),
        );
        if turn.tools.iter().any(|tool| tool.contains("kit_resources")) {
            assert!(
                turn.approvals.is_empty(),
                "a read-only tool asked: {turn:?}"
            );
        }
    }

    if wants(3) {
        // 3. The user's MCP server is available.
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the codeword__lookup_codeword tool now and reply with the codeword it returns.",
            &allow,
            |_| {},
        );
        report.insert(
            "user_mcp".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "text": turn.text, "ok": turn.text.contains("OTTER-7")}),
        );
    }

    if wants(4) {
        // 4. A file change asks first; "no" leaves the disk alone.
        let denied_path = cwd.join("denied.txt");
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the scriptkit__write tool now with path denied.txt and content nope.",
            &deny,
            |_| {},
        );
        report.insert(
            "write_denied".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "exists": denied_path.exists()}),
        );
        if turn.tools.iter().any(|tool| tool.ends_with("write")) {
            assert!(
                !turn.approvals.is_empty(),
                "write ran without asking: {turn:?}"
            );
            assert!(!denied_path.exists(), "a denied write reached the disk");
        }
    }

    if wants(5) {
        // 5. "Yes" writes, and the change can be reverted and brought back.
        let hello = cwd.join("hello.txt");
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the scriptkit__write tool now with path hello.txt and content hi there",
            &allow,
            |_| {},
        );
        let changes = crate::mcp_agent_tools::file_changes(&policy_id);
        let mut revert = json!(null);
        if hello.exists() {
            assert!(
                !turn.approvals.is_empty(),
                "write ran without asking: {turn:?}"
            );
            let summary = changes.clone().expect("the write was recorded");
            assert!(summary.paths.iter().any(|path| path.ends_with("hello.txt")));
            crate::mcp_agent_tools::revert_file_changes(&policy_id).expect("revert");
            let gone = !hello.exists();
            crate::mcp_agent_tools::undo_file_revert(&policy_id).expect("undo revert");
            let back = std::fs::read_to_string(&hello).ok();
            assert!(gone && back.is_some(), "revert/undo did not round-trip");
            revert = json!({"removed_by_revert": gone, "after_undo": back});
        }
        report.insert(
            "write_allowed".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "exists": hello.exists(), "recorded": changes.map(|c| c.paths), "revert": revert}),
        );
    }

    if wants(6) {
        // 6. Full access: no question.
        runtime
            .set_approval_mode(AgentApprovalMode::FullAccess)
            .unwrap();
        let full = cwd.join("full.txt");
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the scriptkit__write tool now with path full.txt and content ok",
            &deny,
            |_| {},
        );
        report.insert(
            "full_access".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "exists": full.exists()}),
        );
        if full.exists() {
            assert!(
                turn.approvals.is_empty(),
                "full access still asked: {turn:?}"
            );
        }
        runtime
            .set_approval_mode(AgentApprovalMode::AskBeforeChanges)
            .unwrap();
    }

    if wants(7) {
        // 7. Steering: words added while the model writes are accepted.
        let steer_result = std::sync::Mutex::new(None);
        let turn = run_turn(
            &runtime,
            &cwd,
            "Write a numbered list of 60 different animals, one per line, no tools.",
            &allow,
            |_| {
                let mut result = steer_result.lock().unwrap();
                if result.is_none() {
                    let rx = runtime
                        .steer(vec![ContentBlock::Text(TextContent::new(
                            "Change of plan: stop the list and reply with only the word STEERED.",
                        ))])
                        .unwrap();
                    *result = Some(smol::block_on(rx.recv()).unwrap_or(Err("closed".into())));
                }
            },
        );
        let steer_result = steer_result.into_inner().unwrap();
        report.insert(
            "steer".into(),
            json!({"accepted": steer_result.as_ref().map(|r| r.is_ok()), "error": steer_result.as_ref().and_then(|r| r.clone().err()), "obeyed": turn.text.contains("STEERED"), "chars": turn.text.len()}),
        );
        assert_eq!(steer_result, Some(Ok(())), "steering was refused");
    }

    if wants(8) {
        // 8. Going back before the last message.
        let points = match recv_one(runtime.fork_points().unwrap()) {
            Some(AgentChatEvent::ForkPointsAvailable { entries }) => entries,
            other => panic!("fork points: {other:?}"),
        };
        assert!(points.len() >= 2, "{points:?}");
        let last = points.last().unwrap().clone();
        let forked = match recv_one(runtime.fork_to_entry(last.entry_id.clone()).unwrap()) {
            Some(AgentChatEvent::ForkCompleted { text }) => text,
            other => panic!("fork: {other:?}"),
        };
        assert_eq!(forked, last.text);
        let after = match recv_one(runtime.fork_points().unwrap()) {
            Some(AgentChatEvent::ForkPointsAvailable { entries }) => entries,
            other => panic!("fork points after: {other:?}"),
        };
        assert_eq!(after.len(), points.len() - 1, "the rewound message is gone");
        let turn = run_turn(
            &runtime,
            &cwd,
            "Reply with only the word AGAIN.",
            &allow,
            |_| {},
        );
        report.insert(
            "rewind".into(),
            json!({"points_before": points.len(), "points_after": after.len(), "forked_text": forked, "next_turn": turn.text, "failed": turn.failed}),
        );
        assert!(
            turn.failed.is_none(),
            "the forked session can't continue: {turn:?}"
        );
    }

    if wants(9) {
        // 9. Agent extensions: todo list and subagents (model permitting).
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the todo__todo_write tool now to record a todo list with one item: buy milk. Then say done.",
            &allow,
            |_| {},
        );
        report.insert(
            "todo".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "failed": turn.failed}),
        );
        assert!(turn.failed.is_none(), "{turn:?}");
        assert!(
            turn.approvals.is_empty(),
            "the todo list never asks: {turn:?}"
        );
    }

    if wants(10) {
        // 10. Delegation asks first (subagents run without approvals), and a
        //     "no" keeps the subagent from starting.
        let turn = run_turn(
            &runtime,
            &cwd,
            "Call the delegate tool now with instructions: reply with the word HI. Then tell me what happened.",
            &allow,
            |_| {},
        );
        report.insert(
            "delegate".into(),
            json!({"tools": turn.tools, "approvals": turn.approvals, "failed": turn.failed, "text": turn.text}),
        );
        if turn.tools.iter().any(|tool| tool == "delegate") {
            assert!(
                turn.approvals
                    .iter()
                    .any(|title| title.contains("delegate")),
                "delegation must ask: {turn:?}"
            );
        }
    }

    eprintln!(
        "LIVE_GOOSE_REPORT {}",
        serde_json::to_string_pretty(&Value::Object(report)).unwrap()
    );
    drop(runtime);
    server_handle.stop();
}
