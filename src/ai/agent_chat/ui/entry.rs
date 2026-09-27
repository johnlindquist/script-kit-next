//! A way into the model's chat, which lives under the main window's input. Surfaces that only the
//! library knows (a chat prompt, the SDK) open it through the host the app registers at startup.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::ai::message_parts::AiContextPart;

/// A way into the chat: its words, whether they are the first turn, and what goes with it.
#[derive(Clone, Debug, Default)]
pub(crate) struct ChatEntry {
    pub(crate) text: String,
    /// The words go to the model now; otherwise they wait in the input.
    pub(crate) send: bool,
    pub(crate) parts: Vec<AiContextPart>,
    /// What's on screen goes with it once it has been read.
    pub(crate) capture: Option<crate::ai::TabAiCaptureKind>,
}

impl ChatEntry {
    /// A quick question: an empty chat, nothing attached.
    pub(crate) fn quick() -> Self {
        Self::default()
    }

    /// Words that wait in the input for Enter.
    pub(crate) fn seeded(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    /// Words that are the first turn.
    pub(crate) fn sending(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            send: true,
            ..Self::default()
        }
    }

    pub(crate) fn with(mut self, part: AiContextPart) -> Self {
        self.parts.push(part);
        self
    }

    pub(crate) fn with_parts(mut self, parts: impl IntoIterator<Item = AiContextPart>) -> Self {
        self.parts.extend(parts);
        self
    }

    pub(crate) fn capturing(mut self, kind: crate::ai::TabAiCaptureKind) -> Self {
        self.capture = Some(kind);
        self
    }
}

/// What the app does for a way into the chat.
pub(crate) struct ChatHost {
    pub(crate) open: fn(ChatEntry, &mut gpui::App) -> bool,
}

static HOST: OnceLock<ChatHost> = OnceLock::new();
static CHAT_ON_TOP: AtomicBool = AtomicBool::new(false);

pub(crate) fn register_host(host: ChatHost) {
    let _ = HOST.set(host);
}

/// Opens the one chat for `entry`; `false` when no main window can take it yet.
pub(crate) fn open_chat(entry: ChatEntry, cx: &mut gpui::App) -> bool {
    HOST.get().is_some_and(|host| (host.open)(entry, cx))
}

/// The host says whether the chat is the view on top of the main window's list.
pub(crate) fn note_chat_on_top(on_top: bool) {
    CHAT_ON_TOP.store(on_top, Ordering::Relaxed);
}

/// Whether the main window is showing the chat.
pub(crate) fn chat_showing() -> bool {
    CHAT_ON_TOP.load(Ordering::Relaxed) && crate::is_main_window_visible()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quick_question_carries_no_words_and_no_context() {
        let quick = ChatEntry::quick();
        assert!(quick.text.is_empty() && !quick.send);
        assert!(quick.parts.is_empty() && quick.capture.is_none());
        assert!(ChatEntry::sending("hi").send);
        assert!(!ChatEntry::seeded("hi").send);
    }
}
