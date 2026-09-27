//! What you copied is a subject like any other. Pasting into the empty input stages it in the line
//! above the input, and the list becomes what you can do with it, in place. Pasting into words
//! you've typed inserts the text as usual.

use crate::contract::words;
use crate::row::{Row, RowAct};
use crate::stage::{Subject, SubjectKind, act};

/// Copied text as the subject the line shows: the first line is its title.
pub fn subject(text: &str) -> Option<Subject> {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let title: String = if first.chars().count() > 80 {
        first.chars().take(79).chain(['…']).collect()
    } else {
        first.to_string()
    };
    let lines = text.lines().filter(|l| !l.trim().is_empty()).count();
    let sub = if is_link(first) {
        "Link".to_string()
    } else if lines > 1 {
        format!("{lines} lines")
    } else {
        format!("{} characters", text.trim().chars().count())
    };
    Some(Subject {
        kind: SubjectKind::Clipboard,
        label: words::ACTING_ON.into(),
        title,
        sub: format!("{} · {sub}", words::CLIPBOARD),
        icon: "copy".into(),
        entry: None,
        row_key: None,
    })
}

fn is_link(text: &str) -> bool {
    let t = text.trim();
    !t.contains(char::is_whitespace)
        && (t.contains("://")
            || t.starts_with("www.")
            || t.split('/').next().is_some_and(|host| {
                host.contains('.') && host.rsplit('.').next().is_some_and(|tld| tld.len() >= 2)
            }) && t.contains('/'))
}

/// What you can do with something copied. Each row also says the sentence that does the same.
pub fn actions(subject: &Subject) -> Vec<Row> {
    let mut rows = Vec::new();
    if is_link(&subject.title) {
        rows.push(act("open", "Open it", "open it", RowAct::Host));
    }
    rows.push(act(
        "message",
        "Send it to someone",
        "send this to sarah",
        RowAct::Host,
    ));
    rows.push(act(
        "schedule",
        "Remind me about it",
        "remind me about this at 4",
        RowAct::Host,
    ));
    rows.push(act(
        "append",
        "Add it to today's note",
        "add what I copied to my notes",
        RowAct::Host,
    ));
    rows.push(act(
        "copy",
        "Show all of it",
        "what did I copy",
        RowAct::Host,
    ));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copied_link_is_titled_by_itself_and_can_be_opened() {
        let s = subject("github.com/script-kit/app/pull/812\n").expect("text");
        assert_eq!(s.title, "github.com/script-kit/app/pull/812");
        assert_eq!(s.sub, "Clipboard · Link");
        assert_eq!(s.label, words::ACTING_ON);
        let rows = actions(&s);
        assert_eq!(rows[0].title, "Open it");
        assert!(rows.iter().all(|r| r.key.starts_with("act:")));
    }

    #[test]
    fn copied_prose_is_not_a_link_and_empty_text_is_nothing() {
        let s = subject("The beta slipped to oct 14.\nTell the team.").expect("text");
        assert_eq!(s.sub, "Clipboard · 2 lines");
        assert!(actions(&s).iter().all(|r| r.title != "Open it"));
        assert!(subject("  \n ").is_none());
    }
}
