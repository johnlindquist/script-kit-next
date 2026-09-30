//! What the chat is about, in the line above the input: the error, selection, file or note the
//! chat was opened with waits there ("Asking about · Script error in deploy.ts") until the next
//! turn takes it along. Otherwise the line names the latest reply, which is what ⌘K acts on.

use crate::chat::{Chat, Phase};
use crate::contract::words;
use crate::slot::{SlotKind, SlotLine};
use crate::stage::Subject;

pub const ASKING_ABOUT: &str = "Asking about";
/// The chat as ⌘K stages it before there's a reply to name.
pub const THE_CHAT: &str = "The model's chat";
pub const THE_REPLY: &str = "the model's reply";
/// How much of the reply the line quotes.
const REPLY_CHARS: usize = 56;

/// The line above the input while the chat is on top, when it has something to say. With
/// `names_reply` the line names the latest reply, and ⌘K stages that.
pub fn line(chat: &Chat, names_reply: bool) -> Option<SlotLine> {
    let reply = chat
        .reply()
        .filter(|_| names_reply && matches!(chat.phase, Phase::Ranking | Phase::Ready))
        .map(quote);
    if chat.acting.is_some() {
        let (title, sub) = match reply {
            Some(reply) => (reply, THE_REPLY.to_string()),
            None => (
                THE_CHAT.to_string(),
                chat.models.current_title().unwrap_or_default().to_string(),
            ),
        };
        return Some(
            SlotLine::new(SlotKind::Staged, words::ACTING_ON, title)
                .with_icon("sparkles")
                .with_sub(sub)
                .with_hint("esc", "let go"),
        );
    }
    let Some(first) = chat.with.first() else {
        return reply.map(|reply| {
            SlotLine::new(SlotKind::Focused, words::FOCUSED, reply)
                .with_icon("sparkles")
                .with_sub(THE_REPLY)
                .with_hint("⌘K", "act on it")
        });
    };
    let sub = match chat.with.len() {
        1 => first.sub.clone(),
        n => format!("+ {} more", n - 1),
    };
    Some(
        SlotLine::new(SlotKind::Staged, ASKING_ABOUT, first.title.clone())
            .with_icon(if first.icon.is_empty() {
                "attach".to_string()
            } else {
                first.icon.clone()
            })
            .with_sub(sub)
            .with_hint("↵", "goes with your question"),
    )
}

/// The reply's first words as the line quotes them, without Markdown's marks.
fn quote(reply: &str) -> String {
    let words = reply
        .lines()
        .map(|l| l.trim_start_matches(['#', '>', '-', ' ']))
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .replace(['*', '`', '_'], "");
    let words = words.split_whitespace().collect::<Vec<_>>().join(" ");
    let Some((cut, _)) = words.char_indices().nth(REPLY_CHARS) else {
        return format!("“{words}”");
    };
    let head = &words[..cut];
    let head = head.rfind(' ').map_or(head, |space| &head[..space]);
    format!("“{}…”", head.trim_end_matches([',', ';', ':', ' ']))
}

/// What a turn says went with it.
pub fn sent_with(with: &[Subject]) -> Option<String> {
    (!with.is_empty()).then(|| {
        format!(
            "With {}",
            with.iter()
                .map(|s| s.title.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(title: &str) -> Subject {
        Subject {
            label: ASKING_ABOUT.into(),
            title: title.into(),
            sub: "Script error".into(),
            ..Subject::default()
        }
    }

    #[test]
    fn the_line_says_what_goes_with_the_next_turn() {
        let chat = Chat::open_with(vec![about("deploy.ts failed")]);
        let line = line(&chat, true).expect("a line");
        assert_eq!(
            (line.kind, line.label.as_str(), line.title.as_str()),
            (SlotKind::Staged, ASKING_ABOUT, "deploy.ts failed")
        );
        assert_eq!(line.sub, "Script error");
        let two = Chat::open_with(vec![about("a"), about("b")]);
        assert_eq!(super::line(&two, true).expect("a line").sub, "+ 1 more");
        assert!(super::line(&Chat::open(""), true).is_none());
    }

    #[test]
    fn the_line_names_the_latest_reply_and_cmd_k_acts_on_it() {
        let mut chat = Chat::open("when is lebron's first game");
        chat.wrote("LeBron James' first scheduled game", false);
        assert!(line(&chat, true).is_none(), "not while it's still writing");
        chat.wrote(
            "LeBron James' first scheduled game of the 2026–27 regular season is **Wednesday, \
             October 21, 2026**, when the Lakers open against Golden State.",
            true,
        );
        chat.ranked(Vec::new());
        let focused = line(&chat, true).expect("a line");
        assert_eq!(
            (focused.kind, focused.label.as_str(), focused.sub.as_str()),
            (SlotKind::Focused, words::FOCUSED, THE_REPLY)
        );
        assert_eq!(
            focused.title,
            "“LeBron James' first scheduled game of the 2026–27…”"
        );
        chat.toggle_acting();
        let acting = line(&chat, true).expect("a line");
        assert_eq!(
            (acting.kind, acting.label.as_str(), acting.title.as_str()),
            (SlotKind::Staged, words::ACTING_ON, focused.title.as_str())
        );
        let old = line(&chat, false).expect("a line");
        assert_eq!(old.title, THE_CHAT, "without it ⌘K stages the chat");
        chat.let_go();
        assert!(line(&chat, false).is_none());
    }
}
