//! A capture typed into the input (`todo; buy milk`, `;note idea`) is a row in the list. The
//! symbol already says what to do, so Jev isn't asked, and Enter saves it through the capture's
//! handler.

use gpui::Context;
use sk_one_window::Row;

use crate::menu_syntax::payload::{resolve_capture_target, CaptureInvocation, IncompleteKind};
use crate::scripts::{OneWindowMatch, SearchResult};
use crate::ScriptListApp;

const KEY: &str = "capture:";

/// The same words as the `;` picker's header, so the header stays put as the capture is typed.
const HEAD: &str = "Capture";

/// What Enter adds, in the words Jev's own capture rows use ("Add reminder: …").
fn noun(target: &str) -> String {
    match target.to_ascii_lowercase().as_str() {
        "todo" | "todos" | "snooze" | "defer" => "todo".into(),
        "reminder" => "reminder".into(),
        "note" | "notes" => "note".into(),
        "link" | "links" => "link".into(),
        "snippet" | "snippets" => "snippet".into(),
        "cal" | "mcal" => "event".into(),
        "social" => "post draft".into(),
        other => other.into(),
    }
}

fn title(noun: &str, invocation: Option<&CaptureInvocation>) -> String {
    let body = invocation
        .map(|invocation| invocation.body.trim())
        .filter(|body| !body.is_empty())
        .unwrap_or("…");
    let when: Vec<&str> = invocation
        .iter()
        .flat_map(|invocation| &invocation.date_phrases)
        .map(|phrase| phrase.source.as_str())
        .collect();
    if when.is_empty() {
        format!("Add {noun}: {body}")
    } else {
        format!("Add {noun}: {body} ({})", when.join(", "))
    }
}

const NO_WORDS: &str = "Type what to write down";

/// Where it goes, then the tags, priority and link typed with it.
fn sub(target: &str, invocation: Option<&CaptureInvocation>) -> String {
    let mut parts = vec![resolve_capture_target(target)
        .map_or_else(|| target.to_string(), |resolved| resolved.title.to_string())];
    match invocation {
        Some(invocation) => {
            parts.extend(invocation.tags.iter().map(|tag| format!("#{tag}")));
            parts.extend(invocation.priority.map(|p| format!("p{p}")));
            parts.extend(invocation.url.clone());
        }
        None => parts.push(NO_WORDS.into()),
    }
    parts.join(" · ")
}

impl ScriptListApp {
    /// The row for a capture typed into the input, or `None` when the words aren't one.
    pub(super) fn one_window_capture_row(&self, text: &str) -> Option<(Row, SearchResult)> {
        let mode = &self.menu_syntax_mode;
        if !mode.capture_composer_owns_input_for(text) {
            return None;
        }
        let invocation = mode.capture_for(text);
        let target = match invocation {
            Some(invocation) => invocation.target.as_str(),
            None => match &mode.incomplete_for(text)?.kind {
                IncompleteKind::MissingCaptureBody(target) => target.as_str(),
                _ => return None,
            },
        };
        let noun = noun(target);
        let enter = match invocation {
            Some(_) => format!("Add {noun}"),
            None => NO_WORDS.to_string(),
        };
        let row = Row::new(format!("{KEY}{target}"), title(&noun, invocation))
            .with_head(HEAD)
            .with_icon("append")
            .with_sub(sub(target, invocation))
            .with_enter_label(enter);
        let result = SearchResult::OneWindow(OneWindowMatch {
            row: row.clone(),
            icon: "plus",
        });
        Some((row, result))
    }

    /// Enter on the capture row saves what's typed.
    pub(super) fn one_window_run_capture(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> Option<anyhow::Result<()>> {
        row.key
            .starts_with(KEY)
            .then(|| self.submit_typed_capture(cx).map(|_| ()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::MenuSyntaxMode;

    fn parts(raw: &str) -> (String, String) {
        let mode = MenuSyntaxMode::from_input(raw);
        let invocation = mode.capture_for(raw);
        let target = invocation.map_or("todo", |invocation| invocation.target.as_str());
        (title(&noun(target), invocation), sub(target, invocation))
    }

    #[test]
    fn a_typed_todo_reads_like_jevs_capture_rows() {
        let (title, sub) = parts("todo; buy milk due:friday");
        assert_eq!(title, "Add todo: buy milk (friday)");
        assert_eq!(sub, "Todo inbox");
    }

    #[test]
    fn tags_and_priority_follow_where_it_goes() {
        let (title, sub) = parts(";todo Renew passport #errands p1");
        assert_eq!(title, "Add todo: Renew passport");
        assert_eq!(sub, "Todo inbox · #errands · p1");
    }

    #[test]
    fn a_capture_with_no_words_yet_asks_for_them() {
        assert_eq!(title("todo", None), "Add todo: …");
        assert_eq!(sub("todo", None), "Todo inbox · Type what to write down");
    }
}
