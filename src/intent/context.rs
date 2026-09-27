//! What Script Kit can see when the user starts typing, and the short form Jev reads.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontApp {
    pub name: String,
    #[serde(default)]
    pub bundle_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenWindow {
    pub app: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuCommand {
    /// The top-level menu, like "File".
    pub menu: String,
    /// The item under it; submenus are joined with " › ".
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
}

impl MenuCommand {
    pub fn question_label(&self) -> String {
        format!("{} › {}", self.menu, self.label)
    }

    pub fn leaf(&self) -> &str {
        self.label.rsplit(" › ").next().unwrap_or(&self.label)
    }

    pub fn path(&self) -> Vec<String> {
        std::iter::once(self.menu.clone())
            .chain(self.label.split(" › ").map(str::to_string))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum LastStep {
    Window {
        op_label: String,
        app: String,
    },
    Command {
        id: String,
    },
    Menu {
        app: String,
        bundle_id: String,
        path: Vec<String>,
    },
}

/// What the user just did through Script Kit, kept so "save that as a script" can replay it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastAction {
    pub summary: String,
    pub steps: Vec<LastStep>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenContext {
    #[serde(default)]
    pub id: String,
    pub front_app: FrontApp,
    #[serde(default)]
    pub window_title: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub selection: Option<String>,
    #[serde(default)]
    pub clipboard: Option<String>,
    #[serde(default)]
    pub open_windows: Vec<OpenWindow>,
    #[serde(default)]
    pub menus: Vec<MenuCommand>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub last_action: Option<LastAction>,
    /// Facts already in the brain; only the ones sharing words with the request are shown.
    #[serde(default)]
    pub memory: Vec<String>,
    /// The launcher reads the selection and clipboard when a plan runs, so a plan that
    /// works on them is never held back for lack of text beforehand.
    #[serde(default)]
    pub subject_read_at_run: bool,
}

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "to", "of", "in", "on", "at", "for", "and",
    "or", "my", "me", "i", "you", "it", "this", "that", "what", "when", "where", "who", "how",
    "do", "did", "does", "with", "about",
];

static POSSESSIVE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"'s\b").ok());

fn content_words(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    let stripped = match POSSESSIVE.as_ref() {
        Some(possessive) => possessive.replace_all(&lower, ""),
        None => lower.as_str().into(),
    };
    stripped
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| word.len() > 1 && !STOPWORDS.contains(word))
        .map(str::to_string)
        .collect()
}

/// Remembered facts that share words with the request, most overlap first.
pub fn memory_matches(ctx: &ScreenContext, request: &str, limit: usize) -> Vec<String> {
    let words = content_words(request);
    let mut scored: Vec<(usize, &String)> = ctx
        .memory
        .iter()
        .map(|fact| {
            let shared = content_words(fact)
                .iter()
                .filter(|w| words.contains(*w))
                .count();
            (shared, fact)
        })
        .filter(|(shared, _)| *shared > 0)
        .collect();
    scored.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, fact)| fact.clone())
        .collect()
}

fn clip(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

/// What Jev sees about the screen. Kept short: long state costs latency on every question.
pub fn describe(ctx: &ScreenContext, request: &str) -> Value {
    let mut screen = Map::new();
    screen.insert("frontmost_app".into(), json!(ctx.front_app.name));
    screen.insert("window_title".into(), json!(ctx.window_title));
    if let Some(url) = ctx.url.as_deref().filter(|u| !u.is_empty()) {
        screen.insert("url".into(), json!(url));
    }
    screen.insert(
        "selected_text".into(),
        match ctx.selection.as_deref().filter(|s| !s.is_empty()) {
            Some(selection) => json!(clip(selection, 280)),
            None => Value::Null,
        },
    );
    if let Some(clipboard) = ctx.clipboard.as_deref().filter(|c| !c.is_empty()) {
        screen.insert("clipboard".into(), json!(clip(clipboard, 120)));
    }
    screen.insert(
        "open_windows".into(),
        json!(ctx
            .open_windows
            .iter()
            .map(|w| w.app.as_str())
            .collect::<Vec<_>>()),
    );
    if let Some(last) = &ctx.last_action {
        screen.insert("last_thing_user_did".into(), json!(last.summary));
    }
    let remembered = memory_matches(ctx, request, 3);
    if !remembered.is_empty() {
        screen.insert(
            "already_remembered_facts_matching_request".into(),
            json!(remembered),
        );
    }
    json!({ "request": request, "screen": Value::Object(screen) })
}
