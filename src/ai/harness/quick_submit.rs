//! Deterministic quick-submit planner for Tab AI harness entries.
//!
//! Classifies raw dropped text (from Send to AI fallback or dictation
//! transcripts) into a structured plan that picks the right capture kind,
//! synthesizes a better `User intent:` turn, and always submits immediately.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Source / kind enums
// ---------------------------------------------------------------------------

/// Where the quick-submit input came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TabAiQuickSubmitSource {
    Fallback,
    Dictation,
}

/// Classification of what the dropped text represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TabAiQuickSubmitKind {
    GeneralAsk,
    VisualAsk,
    UrlDrop,
    FileDrop,
    ShellCommand,
    CodeBlock,
    DiffPatch,
    ErrorLog,
    TextTransform,
}

// ---------------------------------------------------------------------------
// Plan
// ---------------------------------------------------------------------------

/// A resolved quick-submit plan ready for harness injection.
///
/// `submit` is always `true` — the whole point of this planner is that
/// fast-entry flows always submit immediately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabAiQuickSubmitPlan {
    pub source: TabAiQuickSubmitSource,
    pub kind: TabAiQuickSubmitKind,
    pub raw_query: String,
    pub normalized_query: String,
    pub synthesized_intent: String,
    pub capture_kind: String,
    pub submit: bool,
}

impl TabAiQuickSubmitPlan {
    /// Convert the wire-format `capture_kind` string to the typed enum.
    pub fn capture_kind_enum(&self) -> super::TabAiCaptureKind {
        match self.capture_kind.as_str() {
            "fullScreen" => super::TabAiCaptureKind::FullScreen,
            "focusedWindow" => super::TabAiCaptureKind::FocusedWindow,
            "selectedText" => super::TabAiCaptureKind::SelectedText,
            "browserTab" => super::TabAiCaptureKind::BrowserTab,
            _ => super::TabAiCaptureKind::DefaultContext,
        }
    }
}

// ---------------------------------------------------------------------------
// Planner
// ---------------------------------------------------------------------------

/// Classify raw input and produce a quick-submit plan.
///
/// Returns `None` for empty or whitespace-only input.
pub fn plan_tab_ai_quick_submit(
    source: TabAiQuickSubmitSource,
    query: &str,
) -> Option<TabAiQuickSubmitPlan> {
    let raw_query = query.trim().to_string();
    if raw_query.is_empty() {
        return None;
    }

    let normalized_query = normalize_query(&raw_query);

    let (kind, capture_kind, synthesized_intent) = if wants_visual_context(&normalized_query) {
        (
            TabAiQuickSubmitKind::VisualAsk,
            visual_capture_kind(&normalized_query),
            raw_query.clone(),
        )
    } else if let Some(url) = normalize_url_drop(&raw_query) {
        (
            TabAiQuickSubmitKind::UrlDrop,
            "browserTab".to_string(),
            format!(
                "Analyze this URL. If the frontmost browser tab matches it, \
                 use the live browser context as the primary source of truth.\
                 \n\nURL:\n{}",
                url
            ),
        )
    } else if looks_like_file_path(&raw_query) {
        (
            TabAiQuickSubmitKind::FileDrop,
            "defaultContext".to_string(),
            format!(
                "Inspect this path and do the most useful next step. \
                 If it is a file, summarize it. If it is a directory, \
                 explain what matters inside it.\n\nPath:\n{}",
                raw_query
            ),
        )
    } else if looks_like_diff_patch(&raw_query) {
        (
            TabAiQuickSubmitKind::DiffPatch,
            "defaultContext".to_string(),
            format!(
                "Review this patch, explain the behavior change, point out \
                 the biggest risk, and suggest the next edit or test.\n\n\
                 Patch:\n{}",
                raw_query
            ),
        )
    } else if looks_like_error_log(&raw_query) {
        (
            TabAiQuickSubmitKind::ErrorLog,
            "defaultContext".to_string(),
            format!(
                "Diagnose this error output and give the next concrete fix.\n\n\
                 Error output:\n{}",
                raw_query
            ),
        )
    } else if looks_like_code_block(&raw_query) {
        (
            TabAiQuickSubmitKind::CodeBlock,
            "defaultContext".to_string(),
            format!(
                "Review this code or structured snippet. Explain what it does, \
                 identify the biggest issue, and suggest the next edit.\n\n\
                 Snippet:\n{}",
                raw_query
            ),
        )
    } else if looks_like_shell_command(&raw_query) {
        (
            TabAiQuickSubmitKind::ShellCommand,
            "defaultContext".to_string(),
            format!(
                "Explain this command, point out risks, and suggest a safer \
                 or better version if needed.\n\nCommand:\n{}",
                raw_query
            ),
        )
    } else if looks_like_browser_request(&normalized_query) {
        (
            TabAiQuickSubmitKind::GeneralAsk,
            "browserTab".to_string(),
            raw_query.clone(),
        )
    } else if looks_like_text_transform_request(&normalized_query) {
        (
            TabAiQuickSubmitKind::TextTransform,
            "selectedText".to_string(),
            format!(
                "Use the current selection as the primary input and apply \
                 this requested transform.\n\nRequested transform:\n{}",
                raw_query
            ),
        )
    } else {
        (
            TabAiQuickSubmitKind::GeneralAsk,
            "defaultContext".to_string(),
            raw_query.clone(),
        )
    };

    Some(TabAiQuickSubmitPlan {
        source,
        kind,
        raw_query,
        normalized_query,
        synthesized_intent,
        capture_kind,
        submit: true,
    })
}

// ---------------------------------------------------------------------------
// Heuristics
// ---------------------------------------------------------------------------

/// Lowercase + strip non-alphanumeric for keyword matching.
fn normalize_query(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Recognize full URLs, `www.` prefixed domains, and bare domain-like strings.
/// Returns the normalized `https://` URL if the input looks like a URL drop.
fn normalize_url_drop(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Some(trimmed.to_string());
    }
    if lower.starts_with("www.") && !trimmed.contains(' ') {
        return Some(format!("https://{trimmed}"));
    }
    let host = trimmed.split('/').next()?;
    let looks_like_bare_domain = !trimmed.contains(' ')
        && !trimmed.starts_with('/')
        && !trimmed.starts_with("./")
        && !trimmed.starts_with("../")
        && !trimmed.starts_with("~/")
        && host.contains('.')
        && host
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'));
    looks_like_bare_domain.then(|| format!("https://{trimmed}"))
}

fn looks_like_file_path(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with("~/")
        || trimmed.starts_with('/')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || (trimmed.contains('/')
            && !trimmed.contains(' ')
            && normalize_url_drop(trimmed).is_none())
}

fn looks_like_shell_command(input: &str) -> bool {
    if input.contains('\n') || input.trim_end().ends_with('?') {
        return false;
    }
    let first = input
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        first.as_str(),
        "git"
            | "npm"
            | "pnpm"
            | "bun"
            | "cargo"
            | "kubectl"
            | "docker"
            | "cd"
            | "ls"
            | "cat"
            | "grep"
            | "find"
            | "sed"
            | "awk"
            | "curl"
            | "ssh"
            | "ffmpeg"
            | "python"
            | "node"
            | "open"
            | "defaults"
            | "osascript"
    )
}

fn looks_like_error_log(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    lower.contains("error:")
        || lower.contains("exception")
        || lower.contains("traceback")
        || lower.contains("panic")
        || lower.contains("stack trace")
        || lower.contains("failed:")
}

fn looks_like_code_block(input: &str) -> bool {
    input.contains('\n')
        && (input.contains("fn ")
            || input.contains("const ")
            || input.contains("let ")
            || input.contains("class ")
            || input.contains("function ")
            || input.contains("=>")
            || input.contains('{')
            || input.contains("</"))
}

fn looks_like_text_transform_request(normalized: &str) -> bool {
    let implicit_target = normalized.contains(" this ")
        || normalized.starts_with("this ")
        || normalized.ends_with(" this")
        || normalized.contains(" it ")
        || normalized.starts_with("it ")
        || normalized.ends_with(" it")
        || normalized.contains(" that ")
        || normalized.contains(" selected ")
        || normalized.contains(" current ")
        || normalized.contains(" focused ");

    let transform_verb = normalized.contains("rewrite")
        || normalized.contains("summarize")
        || normalized.contains("translate")
        || normalized.contains("fix")
        || normalized.contains("format")
        || normalized.contains("shorten")
        || normalized.contains("expand")
        || normalized.contains("clean up");

    implicit_target && transform_verb
}

fn looks_like_diff_patch(input: &str) -> bool {
    input.contains('\n')
        && (input.starts_with("diff --git ")
            || input.contains("\ndiff --git ")
            || input.starts_with("@@ ")
            || input.contains("\n@@ ")
            || (input.contains("\n+++ ") && input.contains("\n--- ")))
}

fn looks_like_browser_request(normalized: &str) -> bool {
    normalized.contains("page")
        || normalized.contains("site")
        || normalized.contains("article")
        || normalized.contains("tab")
        || normalized.contains("url")
        || normalized.contains("browser")
        || normalized.contains("docs")
        || normalized.contains("documentation")
        || normalized.contains("repo")
        || normalized.contains("issue")
        || normalized.contains("pull request")
        || normalized.contains("link")
}

fn wants_visual_context(normalized: &str) -> bool {
    normalized.contains("screen")
        || normalized.contains("screenshot")
        || normalized.contains("ui")
        || normalized.contains("layout")
        || normalized.contains("dialog")
        || normalized.contains("button")
        || normalized.contains("menu")
        || normalized.contains("what am i looking at")
}

fn visual_capture_kind(normalized: &str) -> String {
    if normalized.contains("window")
        || normalized.contains("dialog")
        || normalized.contains("modal")
    {
        "focusedWindow".to_string()
    } else {
        "fullScreen".to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_none() {
        assert!(plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "").is_none());
        assert!(plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "   ").is_none());
    }

    #[test]
    fn plans_visual_query_as_full_screen() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "what's wrong with this UI?",
        )
        .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::VisualAsk);
        assert_eq!(plan.capture_kind, "fullScreen");
        assert_eq!(plan.synthesized_intent, "what's wrong with this UI?");
        assert!(plan.submit);
    }

    #[test]
    fn plans_visual_window_query_as_focused_window() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "what's wrong with this dialog?",
        )
        .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::VisualAsk);
        assert_eq!(plan.capture_kind, "focusedWindow");
    }

    #[test]
    fn plans_shell_command_drop() {
        let plan = plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "git status")
            .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::ShellCommand);
        assert_eq!(plan.capture_kind, "defaultContext");
        assert!(plan.synthesized_intent.contains("Command:\ngit status"));
    }

    #[test]
    fn plans_path_drop() {
        let plan =
            plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "~/Downloads/report.csv")
                .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::FileDrop);
        assert!(plan
            .synthesized_intent
            .contains("Path:\n~/Downloads/report.csv"));
    }

    #[test]
    fn plans_url_drop() {
        let plan = plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "https://zed.dev")
            .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::UrlDrop);
        assert_eq!(plan.capture_kind, "browserTab");
        assert!(plan.synthesized_intent.contains("URL:\nhttps://zed.dev"));
    }

    #[test]
    fn plans_error_log() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "error: cannot find module 'foo'\nat /src/main.rs:42",
        )
        .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::ErrorLog);
        assert!(plan.synthesized_intent.contains("Error output:"));
    }

    #[test]
    fn plans_code_block() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "fn main() {\n    println!(\"hello\");\n}",
        )
        .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::CodeBlock);
        assert!(plan.synthesized_intent.contains("Snippet:"));
    }

    #[test]
    fn plans_selection_transform() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Dictation,
            "rewrite this to sound calmer",
        )
        .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::TextTransform);
        assert_eq!(plan.capture_kind, "selectedText");
        assert_eq!(plan.source, TabAiQuickSubmitSource::Dictation);
    }

    #[test]
    fn plans_browser_request() {
        let plan =
            plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "summarize this page")
                .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::GeneralAsk);
        assert_eq!(plan.capture_kind, "browserTab");
    }

    #[test]
    fn plans_general_ask_for_unknown_input() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "how do I deploy to production?",
        )
        .expect("plan should exist");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::GeneralAsk);
        assert_eq!(plan.capture_kind, "defaultContext");
        assert_eq!(plan.synthesized_intent, "how do I deploy to production?");
    }

    #[test]
    fn all_plans_have_submit_true() {
        let inputs = vec![
            "what's wrong with this UI?",
            "git status",
            "~/Downloads/report.csv",
            "https://example.com",
            "rewrite this to sound calmer",
            "hello world",
        ];
        for input in inputs {
            let plan = plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, input)
                .expect("plan should exist");
            assert!(plan.submit, "plan for {:?} should have submit=true", input);
        }
    }

    #[test]
    fn capture_kind_enum_roundtrip() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "what's wrong with this UI?",
        )
        .expect("plan");
        assert_eq!(
            plan.capture_kind_enum(),
            super::super::TabAiCaptureKind::FullScreen
        );

        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "rewrite this to sound calmer",
        )
        .expect("plan");
        assert_eq!(
            plan.capture_kind_enum(),
            super::super::TabAiCaptureKind::SelectedText
        );
    }

    #[test]
    fn shell_command_with_question_mark_is_not_command() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "git rebase --onto main HEAD~3?",
        )
        .expect("plan");
        // Trailing `?` disqualifies shell command heuristic
        assert_ne!(plan.kind, TabAiQuickSubmitKind::ShellCommand);
    }

    #[test]
    fn git_cherry_pick_is_shell_command() {
        let plan =
            plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "git cherry-pick --abort")
                .expect("plan");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::ShellCommand);
    }

    #[test]
    fn plans_bare_domain_as_url_drop() {
        let plan =
            plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "www.example.test/docs")
                .expect("plan");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::UrlDrop);
        assert_eq!(plan.capture_kind, "browserTab");
        assert!(
            plan.synthesized_intent
                .contains("https://www.example.test/docs"),
            "bare domains should be normalized to https URLs in the synthesized intent"
        );
    }

    #[test]
    fn plans_bare_hostname_as_url_drop() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "github.com/zed-industries",
        )
        .expect("plan");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::UrlDrop);
        assert_eq!(plan.capture_kind, "browserTab");
    }

    #[test]
    fn plans_diff_patch_as_diff_patch() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "diff --git a/src/main.rs b/src/main.rs\n@@ -1,1 +1,2 @@\n fn main() {}\n+println!(\"hello\");",
        )
        .expect("plan");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::DiffPatch);
        assert!(
            plan.synthesized_intent.contains("Review this patch"),
            "diff drops should get a patch-review intent, not a generic code review"
        );
    }

    #[test]
    fn plans_unified_diff_as_diff_patch() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "--- a/file.rs\n+++ b/file.rs\n@@ -1 +1 @@\n-old\n+new",
        )
        .expect("plan");
        assert_eq!(plan.kind, TabAiQuickSubmitKind::DiffPatch);
    }

    #[test]
    fn plans_docs_query_as_browser_request() {
        let plan = plan_tab_ai_quick_submit(
            TabAiQuickSubmitSource::Fallback,
            "summarize the docs on this page",
        )
        .expect("plan");
        assert_eq!(plan.capture_kind, "browserTab");
    }
}
