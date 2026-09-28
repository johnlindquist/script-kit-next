//! Save as script: a draft Script Kit script that replays a plan.
//!
//! The structure is kept and the content becomes a prompt, so "remind me to call mom"
//! saves as a reminder script that asks what to be reminded about.

use std::collections::{BTreeMap, BTreeSet};

use super::catalog::CommandKind;
use super::engine::{recall_source, Plan, Step};

#[derive(Debug, Clone, Default)]
pub struct ScriptOptions {
    pub name: Option<String>,
    pub shortcut: Option<String>,
    /// Turn typed content (a reminder's text, a question) into a prompt.
    pub parameterize: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftScript {
    pub filename: String,
    pub name: String,
    pub shortcut: Option<String>,
    pub source: String,
}

/// "Command Shift L" → "cmd shift l", the form Script Kit metadata uses.
pub fn normalize_shortcut(phrase: &str) -> String {
    let spaced: String = phrase
        .chars()
        .flat_map(|c| match c {
            '⌘' | '⌥' | '⌃' | '⇧' => vec![' ', c, ' '],
            _ => vec![c],
        })
        .collect();
    spaced
        .to_lowercase()
        .replace(['+', '-'], " ")
        .split_whitespace()
        .map(|word| match word {
            "command" | "cmd" | "⌘" => "cmd",
            "control" | "ctrl" | "ctl" | "⌃" => "ctrl",
            "option" | "opt" | "alt" | "⌥" => "alt",
            "shift" | "⇧" => "shift",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut gap = false;
    for c in text.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            if gap && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(c);
            gap = false;
        } else {
            gap = true;
        }
    }
    let slug: String = slug.chars().take(48).collect();
    let slug = slug.trim_end_matches('-').to_string();
    if slug.is_empty() {
        "untitled".into()
    } else {
        slug
    }
}

/// A parameterized script is named for what it does, not for the content it was made from.
pub fn script_name(plan: &Plan, parameterized: bool) -> String {
    let title = plan
        .title
        .strip_prefix("Save as script: ")
        .or_else(|| plan.title.strip_prefix("New script: "))
        .unwrap_or(&plan.title);
    let title = if parameterized {
        title.split(':').next().unwrap_or(title)
    } else {
        title
    };
    title
        .chars()
        .take(60)
        .collect::<String>()
        .trim()
        .to_string()
}

#[derive(Default)]
struct Emitted {
    lines: Vec<String>,
    helpers: BTreeSet<&'static str>,
    imports: BTreeMap<&'static str, BTreeSet<&'static str>>,
}

impl Emitted {
    fn use_helper(&mut self, helper: &'static str) {
        self.helpers.insert(helper);
        let needs: &[(&str, &[&str])] = match helper {
            "capture" => &[
                ("node:fs/promises", &["writeFile"]),
                ("node:os", &["tmpdir"]),
                ("node:path", &["join"]),
            ],
            "remember" => &[
                ("node:fs/promises", &["appendFile", "mkdir"]),
                ("node:os", &["homedir"]),
                ("node:path", &["join"]),
            ],
            _ => &[],
        };
        for (module, names) in needs {
            self.imports.entry(module).or_default().extend(names.iter());
        }
    }

    fn block(&mut self, body: &str) {
        self.lines.push("{".into());
        self.lines.extend(
            body.lines()
                .map(|line| format!("  {line}").trim_end().to_string()),
        );
        self.lines.push("}".into());
    }
}

const CAPTURE_HELPER: &str = r#"async function capture(target: string, body: string, when?: string, handler?: string) {
  const payloadPath = join(tmpdir(), `capture-${Date.now()}.json`);
  const raw = `${target}; ${body}${when ? ` due:"${when}"` : ""}`;
  await writeFile(
    payloadPath,
    JSON.stringify({ version: 1, family: "capture.v1", target, raw, body, tags: [], dates: when ? [{ role: "due", phrase: when }] : [], handler: { script: handler } }),
  );
  process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH = payloadPath;
  if (handler) await run(handler);
}"#;

const REMEMBER_HELPER: &str = r#"async function remember(text: string) {
  const dir = join(process.env.SK_PATH ?? join(homedir(), ".scriptkit"), "brain", "days");
  await mkdir(dir, { recursive: true });
  const now = new Date();
  await appendFile(join(dir, `${now.toISOString().slice(0, 10)}.md`), `${now.toTimeString().slice(0, 5)} - ${text} #remember\n`);
}"#;

const WITH_SUBJECT_HELPER: &str = r#"async function withSubject(text: string, subject?: string) {
  if (subject === "selection") return `${text}\n\n${await getSelectedText()}`;
  if (subject === "clipboard") return `${text}\n\n${await clipboard.readText()}`;
  return text;
}"#;

fn js(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".into())
}

/// `arg` has no pre-filled value, so the original words become the hint.
fn value(text: &str, prompt: &str, parameterize: bool) -> String {
    if parameterize {
        format!(
            "await arg({{ placeholder: {}, hint: {} }})",
            js(prompt),
            js(&format!("e.g. {text}"))
        )
    } else {
        js(text)
    }
}

fn last_segment(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

fn emit_step(step: &Step, parameterize: bool, out: &mut Emitted) {
    match step {
        Step::Command { command } => {
            let deeplink = || {
                format!(
                    "await exec(\"open\", [{}]);",
                    js(&format!("scriptkit://commands/{}", command.id))
                )
            };
            let line = match command.kind {
                CommandKind::App => {
                    format!("await exec(\"open\", [\"-a\", {}]);", js(&command.name))
                }
                CommandKind::Builtin => deeplink(),
                CommandKind::Script => format!("await run({});", js(last_segment(&command.id))),
                CommandKind::Scriptlet => match &command.code {
                    Some(code) if code.lang == "open" => {
                        format!("await exec(\"open\", [{}]);", js(code.source.trim()))
                    }
                    Some(code) if code.lang == "ts" => {
                        out.block(&code.source);
                        return;
                    }
                    _ => deeplink(),
                },
            };
            out.lines.push(line);
        }
        Step::OpenUrl { url, app } => {
            out.lines.push(match app {
                Some(app) => format!("await exec(\"open\", [\"-a\", {}, {}]);", js(app), js(url)),
                None => format!("await exec(\"open\", [{}]);", js(url)),
            });
        }
        Step::Menu {
            bundle_id, path, ..
        } => {
            let path = serde_json::to_string(path).unwrap_or_else(|_| "[]".into());
            out.lines.push(format!(
                "await executeMenuAction({}, {path});",
                js(bundle_id)
            ));
        }
        Step::Window { op, app } => {
            let find = format!(
                "(await getWindows()).find((w) => w.appName === {})",
                js(app)
            );
            let call = match op.action() {
                Some(super::catalog::WindowAction::Quit) => {
                    let quit = format!("tell application \"{}\" to quit", app.replace('"', "\\\""));
                    out.lines.push(format!(
                        "await exec(\"osascript\", [\"-e\", {}]);",
                        js(&quit)
                    ));
                    return;
                }
                Some(action) => match action.sdk_position() {
                    Some(position) => format!("await tileWindow(win.windowId, {});", js(position)),
                    None => match action {
                        super::catalog::WindowAction::NextDisplay => {
                            "await moveToNextDisplay(win.windowId);".into()
                        }
                        super::catalog::WindowAction::PreviousDisplay => {
                            "await moveToPreviousDisplay(win.windowId);".into()
                        }
                        super::catalog::WindowAction::Minimize => {
                            "await minimizeWindow(win.windowId);".into()
                        }
                        _ => "await closeWindow(win.windowId);".into(),
                    },
                },
                None if !op.code.is_empty() => {
                    out.block(&op.code.replace("await getFrontmostWindow()", &find));
                    return;
                }
                None => return,
            };
            out.block(&format!("const win = {find};\nif (win) {call}"));
        }
        Step::Capture {
            target,
            body,
            when,
            handler,
        } => {
            out.use_helper("capture");
            let when = when
                .as_ref()
                .map_or("undefined".to_string(), |w| value(w, "When?", parameterize));
            let handler = handler
                .as_ref()
                .map_or(String::new(), |h| format!(", {}", js(last_segment(&h.id))));
            out.lines.push(format!(
                "await capture({}, {}, {when}{handler});",
                js(target),
                value(body, &format!("New {target}"), parameterize)
            ));
        }
        Step::Remember { text, subject } => {
            out.use_helper("remember");
            let text = match subject.as_deref() {
                Some("selection") => "await getSelectedText()".to_string(),
                Some("clipboard") => "await clipboard.readText()".to_string(),
                _ => value(text, "Remember", parameterize),
            };
            out.lines.push(format!("await remember({text});"));
        }
        Step::Recall { source, .. } => {
            let builtin = recall_source(source)
                .and_then(|s| s.builtin)
                .unwrap_or("builtin/main-window");
            out.lines.push(format!(
                "await exec(\"open\", [{}]);",
                js(&format!("scriptkit://commands/{builtin}"))
            ));
        }
        Step::Transform {
            tool,
            instruction,
            subject,
        } => match tool
            .as_ref()
            .and_then(|t| t.code.as_ref())
            .filter(|c| c.lang == "ts")
        {
            Some(code) => out.block(&code.source),
            None => {
                out.use_helper("withSubject");
                out.lines.push(format!(
                    "await aiStartChat(await withSubject({}, {}));",
                    js(instruction.as_deref().unwrap_or_default()),
                    js(subject)
                ));
            }
        },
        Step::Ask { question, subject } => {
            out.use_helper("withSubject");
            out.lines.push(format!(
                "await aiStartChat(await withSubject({}, {}));",
                value(question, "Ask", parameterize && subject.is_none()),
                js(subject.as_deref().unwrap_or("typed"))
            ));
        }
        Step::Delegate {
            task,
            project,
            subject,
        } => {
            out.use_helper("withSubject");
            let options = project.as_ref().map_or(String::new(), |p| {
                format!(
                    ", {{ systemPrompt: {} }}",
                    js(&format!("Work in ~/dev/{p}."))
                )
            });
            out.lines.push(format!(
                "await aiStartChat(await withSubject({}, {}){options});",
                value(task, "Task", parameterize && subject.is_none()),
                js(subject.as_deref().unwrap_or("typed"))
            ));
        }
    }
}

/// A draft script that replays the plan; content slots become prompts when parameterized.
pub fn plan_to_script(plan: &Plan, opts: &ScriptOptions) -> DraftScript {
    let shortcut = opts
        .shortcut
        .clone()
        .or_else(|| plan.slots.shortcut.as_deref().map(normalize_shortcut))
        .filter(|s| !s.is_empty());
    let parameterize = opts.parameterize.unwrap_or(plan.steps.len() == 1);
    let has_content = plan.steps.iter().any(|s| {
        matches!(
            s,
            Step::Capture { .. } | Step::Remember { .. } | Step::Ask { .. } | Step::Delegate { .. }
        )
    });
    let name = opts
        .name
        .clone()
        .unwrap_or_else(|| script_name(plan, parameterize && has_content));
    let mut out = Emitted::default();
    for step in &plan.steps {
        emit_step(step, parameterize, &mut out);
    }

    let mut lines = vec!["import \"@scriptkit/sdk\";".to_string()];
    for (module, names) in &out.imports {
        lines.push(format!(
            "import {{ {} }} from \"{module}\";",
            names.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    lines.push(String::new());
    lines.push("export const metadata = {".into());
    lines.push(format!("  name: {},", js(&name)));
    lines.push(format!(
        "  description: {},",
        js(&format!("Made from: “{}”", plan.request))
    ));
    if let Some(shortcut) = &shortcut {
        lines.push(format!("  shortcut: {},", js(shortcut)));
    }
    lines.push("};".into());
    lines.push(String::new());
    for helper in ["capture", "remember", "withSubject"] {
        if out.helpers.contains(helper) {
            lines.push(
                match helper {
                    "capture" => CAPTURE_HELPER,
                    "remember" => REMEMBER_HELPER,
                    _ => WITH_SUBJECT_HELPER,
                }
                .to_string(),
            );
            lines.push(String::new());
        }
    }
    lines.extend(out.lines);
    lines.push(String::new());
    DraftScript {
        filename: format!("{}.ts", slugify(&name)),
        name,
        shortcut,
        source: lines.join("\n"),
    }
}
