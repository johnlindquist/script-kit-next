//! What the chat is about, in the line above the input: the error, selection, file or note the
//! chat was opened with waits there ("Asking about · Script error in deploy.ts") until the next
//! turn takes it along. ⌘K puts the chat itself there.

use crate::chat::Chat;
use crate::contract::words;
use crate::slot::{SlotKind, SlotLine};
use crate::stage::Subject;

pub const ASKING_ABOUT: &str = "Asking about";
/// The chat as ⌘K stages it.
pub const THE_CHAT: &str = "The model's chat";

/// The line above the input while the chat is on top, when it has something to say.
pub fn line(chat: &Chat) -> Option<SlotLine> {
    if chat.acting.is_some() {
        return Some(
            SlotLine::new(SlotKind::Staged, words::ACTING_ON, THE_CHAT)
                .with_icon("sparkles")
                .with_sub(chat.models.current_title().unwrap_or_default())
                .with_hint("esc", "let go"),
        );
    }
    let first = chat.with.first()?;
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
        let line = line(&chat).expect("a line");
        assert_eq!(
            (line.kind, line.label.as_str(), line.title.as_str()),
            (SlotKind::Staged, ASKING_ABOUT, "deploy.ts failed")
        );
        assert_eq!(line.sub, "Script error");
        let two = Chat::open_with(vec![about("a"), about("b")]);
        assert_eq!(super::line(&two).expect("a line").sub, "+ 1 more");
        assert!(super::line(&Chat::open("")).is_none());
    }
}
